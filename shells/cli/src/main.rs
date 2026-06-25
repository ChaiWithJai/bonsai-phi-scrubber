//! airplane-cli — the reproduction front door (Tier 1).
//!
//! Same `airplane-core` the iOS app runs, with the **llama-server** adapter for the
//! `InferenceProvider` port. Verbs:
//!   airplane eval            reproduce recall/leakage over the golden set -> eval/golden-run.txt
//!   airplane scrub "<text>"  scrub arbitrary text on the CLI
//!   airplane gates           run the M1 harness gates (pack-blindness, recall, leakage)

use airplane_core::{
    finalize, scrub, Expected, InferenceProvider, InferenceRequest, Pack, Sampling, Score,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const PACK_DIR: &str = "packs/coach-session";
const GOLDEN_RUN: &str = "eval/golden-run.txt";

// ---- the llama-server adapter (the InferenceProvider port, CLI side) ----------

struct LlamaServer {
    url: String,
    model: String,
}

impl Default for LlamaServer {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8080/v1/chat/completions".to_string(),
            model: "ternary-bonsai-1.7b".to_string(),
        }
    }
}

impl InferenceProvider for LlamaServer {
    fn complete(&self, req: &InferenceRequest<'_>) -> Result<String> {
        // Constrained decoding happens server-side via `response_format: json_schema`
        // (llama.cpp compiles the schema to a GBNF grammar).
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": req.system},
                {"role": "user", "content": req.user},
            ],
            "temperature": req.sampling.temperature,
            "top_k": req.sampling.top_k,
            "top_p": req.sampling.top_p,
            "max_tokens": req.sampling.max_tokens,
            "seed": req.sampling.seed,
            "response_format": {
                "type": "json_schema",
                "json_schema": { "name": "redactions", "strict": true, "schema": req.json_schema }
            }
        });
        let resp = ureq::post(&self.url).send_json(body).map_err(|e| {
            anyhow::anyhow!(
                "llama-server request failed ({e}). Is it running?  ./scripts/serve-model.sh"
            )
        })?;
        let v: serde_json::Value = resp.into_json().context("parse llama-server response")?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .context("no message content in llama-server response")?;
        Ok(content.to_string())
    }
}

// ---- eval --------------------------------------------------------------------

fn note_ids(golden_dir: &Path) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for entry in
        std::fs::read_dir(golden_dir).with_context(|| format!("read {}", golden_dir.display()))?
    {
        let p = entry?.path();
        if p.extension().and_then(|e| e.to_str()) == Some("txt") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                ids.push(stem.to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

struct EvalOutcome {
    score: Score,
    blocked_notes: Vec<String>,
    report: String,
}

/// Number of seeded contextual passes to union. Override with AIRPLANE_EVAL_PASSES.
fn eval_passes() -> u32 {
    // Union across seeded passes is recall-first; 5 clears the 99% gate on the golden set.
    std::env::var("AIRPLANE_EVAL_PASSES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

fn run_eval(use_model: bool) -> Result<EvalOutcome> {
    let pack_dir = Path::new(PACK_DIR);
    let pack = Pack::load(pack_dir).context("load coach-session pack")?;
    let golden = pack_dir.join("eval/golden");
    let expected = pack_dir.join("eval/expected");
    let provider = LlamaServer::default();
    let passes = eval_passes();

    let mut acc = Score::default();
    let mut blocked = Vec::new();
    let mut per_note = String::new();

    for id in note_ids(&golden)? {
        let text = std::fs::read_to_string(golden.join(format!("{id}.txt")))?;
        let exp: Expected = serde_json::from_str(&std::fs::read_to_string(
            expected.join(format!("{id}.json")),
        )?)
        .with_context(|| format!("parse expected for {id}"))?;

        let model: Option<&dyn InferenceProvider> = if use_model { Some(&provider) } else { None };
        let res = scrub(&text, &pack.rules, model, Sampling::eval(), passes)?;
        airplane_core::score_note(&res.redaction_map, &exp, &mut acc);

        let gate = if res.gate.is_pass() {
            "PASS"
        } else {
            blocked.push(id.clone());
            "BLOCK"
        };
        per_note.push_str(&format!(
            "{id}  gate={gate}  redactions={}\n",
            res.redaction_map.len()
        ));
    }
    finalize(&mut acc);

    let report = format!(
        "Airplane Mode — eval/golden-run\n\
         pack            : {}\n\
         model layer     : {}\n\
         decoding        : {} seeded passes unioned (temp 0.5, seeds 42..)  [deterministic]\n\
         notes           : {}\n\
         recall          : {:.1}%  ({}/{})\n\
         hard-case recall: {:.1}%  ({}/{})\n\
         leakage (missed): {}\n\
         over-redactions : {}\n\
         recall gate     : >= {:.0}%\n\
         \n{}",
        pack.name,
        if use_model {
            "rules ∪ Bonsai-1.7B"
        } else {
            "rules only"
        },
        passes,
        acc.notes,
        acc.recall * 100.0,
        acc.caught,
        acc.total_labels,
        acc.hard_recall * 100.0,
        acc.hard_caught,
        acc.hard_total,
        acc.leakage,
        acc.over_redactions,
        pack.policy.deidentification.recall_threshold * 100.0,
        per_note,
    );

    Ok(EvalOutcome {
        score: acc,
        blocked_notes: blocked,
        report,
    })
}

fn cmd_eval() -> Result<()> {
    let out = run_eval(true)?;
    print!("{}", out.report);

    if !out.score.missed.is_empty() {
        println!("LEAKS ({}):", out.score.missed.len());
        for m in &out.score.missed {
            println!(
                "  {}  [{}]{}  {}",
                m.note,
                m.entity,
                if m.hard { " HARD" } else { "" },
                m.text
            );
        }
    }

    // Write/refresh the committed reproduction target + machine-readable score.
    std::fs::create_dir_all("eval").ok();
    std::fs::write(GOLDEN_RUN, &out.report).context("write golden-run.txt")?;
    std::fs::write(
        "eval/last-run.json",
        serde_json::to_string_pretty(&out.score)?,
    )
    .ok();
    println!("\nwrote {GOLDEN_RUN}");
    if !out.blocked_notes.is_empty() {
        println!(
            "note: gate BLOCKED {} note(s): {:?}",
            out.blocked_notes.len(),
            out.blocked_notes
        );
    }
    Ok(())
}

// ---- scrub -------------------------------------------------------------------

fn cmd_scrub(text: &str) -> Result<()> {
    let pack = Pack::load(Path::new(PACK_DIR))?;
    let provider = LlamaServer::default();
    let res = scrub(text, &pack.rules, Some(&provider), Sampling::eval(), 3)?;
    println!("scrubbed : {}", res.scrubbed_text);
    println!(
        "gate     : {}",
        if res.gate.is_pass() {
            "PASS"
        } else {
            "BLOCK (residual identifier)"
        }
    );
    println!("redactions ({}):", res.redaction_map.len());
    for s in &res.redaction_map {
        println!("  [{}] {}  <- {}", s.entity, s.text, s.layer);
    }
    Ok(())
}

// ---- gates -------------------------------------------------------------------

fn cmd_gates() -> Result<()> {
    let pack_dir = PathBuf::from(PACK_DIR);
    let mut failed = false;

    // pack-blindness (structural, no model)
    match Pack::validate_blindness(&pack_dir) {
        Ok(()) => println!("gate pack-blindness : PASS"),
        Err(e) => {
            println!("gate pack-blindness : FAIL — {e}");
            failed = true;
        }
    }

    // recall + leakage (needs the model)
    let pack = Pack::load(&pack_dir)?;
    let threshold = pack.policy.deidentification.recall_threshold;
    let out = run_eval(true)?;
    if out.score.recall + 1e-9 >= threshold {
        println!(
            "gate recall         : PASS  ({:.1}% >= {:.0}%)",
            out.score.recall * 100.0,
            threshold * 100.0
        );
    } else {
        println!(
            "gate recall         : FAIL  ({:.1}% < {:.0}%)",
            out.score.recall * 100.0,
            threshold * 100.0
        );
        failed = true;
    }
    if out.score.leakage == 0 {
        println!("gate leakage        : PASS  (0 residual identifiers)");
    } else {
        println!("gate leakage        : FAIL  ({} leaked)", out.score.leakage);
        for m in out.score.missed.iter().take(10) {
            println!("    leak {} [{}] {}", m.note, m.entity, m.text);
        }
        failed = true;
    }

    match pack.validate_reward_lint() {
        Ok(()) => println!("gate reward-lint    : PASS"),
        Err(e) => {
            println!("gate reward-lint    : FAIL — {e}");
            failed = true;
        }
    }
    match pack.validate_scope_boundary() {
        Ok(()) => println!("gate scope-boundary : PASS"),
        Err(e) => {
            println!("gate scope-boundary : FAIL — {e}");
            failed = true;
        }
    }

    if failed {
        anyhow::bail!("one or more gates failed");
    }
    println!("\nall active gates PASS");
    Ok(())
}

// ---- main --------------------------------------------------------------------

fn usage() {
    eprintln!(
        "airplane — on-device PHI scrubber (CLI shell)\n\
         \n\
         USAGE:\n\
           airplane eval            reproduce recall/leakage over the golden set\n\
           airplane scrub \"<text>\"  scrub arbitrary text\n\
           airplane gates           run the harness gates\n\
         \n\
         Needs llama-server for the model layer:  ./scripts/serve-model.sh"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verb = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    let result = match verb {
        "eval" => cmd_eval(),
        "gates" => cmd_gates(),
        "scrub" => match args.get(2) {
            Some(text) => cmd_scrub(text),
            None => {
                usage();
                std::process::exit(2);
            }
        },
        _ => {
            usage();
            return;
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
