//! airplane-cli — the reproduction front door (Tier 1).
//!
//! Same `airplane-core` the iOS app runs, with the **llama-server** adapter for the
//! `InferenceProvider` port. Verbs:
//!   airplane eval            check recall/leakage against eval/golden-run.txt
//!   airplane eval --update   refresh eval/golden-run.txt intentionally
//!   airplane scrub "<text>"  scrub arbitrary text on the CLI
//!   airplane gates           run the M1 harness gates (pack-blindness, recall, leakage)
//!   airplane gates-fast      run no-model gates for fast iteration

use airplane_core::{
    finalize, scrub, Expected, InferenceProvider, InferenceRequest, Pack, Sampling, Score,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEFAULT_PACK_DIR: &str = "packs/coach-session";
const GOLDEN_RUN: &str = "eval/golden-run.txt";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvalMode {
    Check,
    Update,
}

fn parse_eval_mode(arg: Option<&str>) -> Option<EvalMode> {
    match arg {
        None | Some("--check") => Some(EvalMode::Check),
        Some("--update") => Some(EvalMode::Update),
        _ => None,
    }
}

fn pack_dir() -> PathBuf {
    std::env::var("PACK")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_PACK_DIR))
}

fn is_default_pack(path: &Path) -> bool {
    path == Path::new(DEFAULT_PACK_DIR)
}

// ---- the llama-server adapter (the InferenceProvider port, CLI side) ----------

struct LlamaServer {
    url: String,
    model: String,
    timeout: Duration,
}

impl Default for LlamaServer {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8080/v1/chat/completions".to_string(),
            model: "ternary-bonsai-1.7b".to_string(),
            timeout: model_timeout(),
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
        let started = Instant::now();
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();
        let resp = agent.post(&self.url).send_json(body).map_err(|e| {
            anyhow::anyhow!(
                "llama-server request failed after {:.1}s ({e}). Is it running?  ./scripts/serve-model.sh",
                started.elapsed().as_secs_f32()
            )
        })?;
        let v: serde_json::Value = resp.into_json().context("parse llama-server response")?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .context("no message content in llama-server response")?;
        Ok(content.to_string())
    }
}

fn model_timeout() -> Duration {
    let secs = std::env::var("AIRPLANE_MODEL_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|s| *s > 0)
        .unwrap_or(120);
    Duration::from_secs(secs)
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
    let pack_dir = pack_dir();
    let pack =
        Pack::load(&pack_dir).with_context(|| format!("load pack {}", pack_dir.display()))?;
    let golden = pack_dir.join("eval/golden");
    let expected = pack_dir.join("eval/expected");
    let provider = LlamaServer::default();
    let passes = eval_passes();

    let mut acc = Score::default();
    let mut blocked = Vec::new();
    let mut per_note = String::new();
    let ids = note_ids(&golden)?;
    let total_notes = ids.len();
    let started = Instant::now();

    if use_model {
        eprintln!(
            "eval plan: {} notes x {} passes = {} model calls (request timeout {}s)",
            total_notes,
            passes,
            total_notes as u32 * passes.max(1),
            provider.timeout.as_secs()
        );
    } else {
        eprintln!("eval plan: {} notes, rules only", total_notes);
    }

    for (idx, id) in ids.into_iter().enumerate() {
        let note_started = Instant::now();
        eprintln!(
            "eval {}/{} {} ({} passes)",
            idx + 1,
            total_notes,
            id,
            passes
        );
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
        eprintln!(
            "eval {}/{} {} -> {} in {:.1}s",
            idx + 1,
            total_notes,
            id,
            gate,
            note_started.elapsed().as_secs_f32()
        );
        per_note.push_str(&format!("{id}  gate={gate}\n"));
    }
    finalize(&mut acc);
    eprintln!("eval complete in {:.1}s", started.elapsed().as_secs_f32());

    let report = format!(
        "Airplane Mode — eval/golden-run\n\
         pack            : {}\n\
         model layer     : {}\n\
         decoding        : {} seeded passes unioned (temp 0.5, seeds 42..)  [deterministic]\n\
         notes           : {}\n\
         recall          : {:.1}%  ({}/{})\n\
         precision       : {:.1}%  ({}/{} predicted)\n\
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
        acc.precision * 100.0,
        acc.caught,
        acc.caught + acc.over_redactions,
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

fn print_misses(out: &EvalOutcome) {
    if out.score.missed.is_empty() {
        return;
    }
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

fn cmd_eval(mode: EvalMode) -> Result<()> {
    let pack_dir = pack_dir();
    if mode == EvalMode::Update && !is_default_pack(&pack_dir) {
        anyhow::bail!(
            "refusing to update {GOLDEN_RUN} for non-default pack {}; unset PACK to refresh the committed reference target",
            pack_dir.display()
        );
    }

    let out = run_eval(true)?;
    print!("{}", out.report);
    print_misses(&out);

    // Always write the ignored machine-readable score for local inspection.
    std::fs::create_dir_all("eval").ok();
    std::fs::write(
        "eval/last-run.json",
        serde_json::to_string_pretty(&out.score)?,
    )
    .ok();
    if !out.blocked_notes.is_empty() {
        println!(
            "note: gate BLOCKED {} note(s): {:?}",
            out.blocked_notes.len(),
            out.blocked_notes
        );
    }

    if mode == EvalMode::Update {
        std::fs::write(GOLDEN_RUN, &out.report).context("write golden-run.txt")?;
        println!("\nwrote {GOLDEN_RUN}");
        return Ok(());
    }

    if !is_default_pack(&pack_dir) {
        println!(
            "\ncustom pack {}; report printed only; {GOLDEN_RUN} is the default-pack reproduction target",
            pack_dir.display()
        );
        return Ok(());
    }

    let expected = std::fs::read_to_string(GOLDEN_RUN).context("read golden-run.txt")?;
    if expected != out.report {
        anyhow::bail!(
            "{GOLDEN_RUN} mismatch; inspect eval/last-run.json and run `./run.sh eval --update` only if the new report is intentional"
        );
    }
    println!("\n{GOLDEN_RUN} matches");
    Ok(())
}

// ---- scrub -------------------------------------------------------------------

fn cmd_scrub(text: &str) -> Result<()> {
    let pack_dir = pack_dir();
    let pack =
        Pack::load(&pack_dir).with_context(|| format!("load pack {}", pack_dir.display()))?;
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

struct PreModelGateOutcome {
    pack: Pack,
    failed: bool,
}

fn run_pre_model_gates() -> Result<PreModelGateOutcome> {
    let pack_dir = pack_dir();
    let mut failed = false;

    // pack-blindness (structural, no model)
    match Pack::validate_blindness(&pack_dir) {
        Ok(()) => println!("gate pack-blindness : PASS"),
        Err(e) => {
            println!("gate pack-blindness : FAIL — {e}");
            failed = true;
        }
    }

    let pack = Pack::load(&pack_dir)?;
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
    match Pack::validate_signature_provenance(&pack_dir) {
        Ok(()) => println!("gate signature/prov : PASS"),
        Err(e) => {
            println!("gate signature/prov : FAIL — {e}");
            failed = true;
        }
    }
    match Pack::validate_manifest_revocation(Path::new("manifest.yaml"), &pack_dir) {
        Ok(()) => println!("gate manifest/revoke: PASS"),
        Err(e) => {
            println!("gate manifest/revoke: FAIL — {e}");
            failed = true;
        }
    }

    Ok(PreModelGateOutcome { pack, failed })
}

fn cmd_gates_fast() -> Result<()> {
    let out = run_pre_model_gates()?;
    if out.failed {
        anyhow::bail!("one or more pre-model gates failed");
    }
    println!("\nall fast gates PASS (model eval not run; use `./run.sh gates` for recall/leakage)");
    Ok(())
}

fn cmd_gates() -> Result<()> {
    let mut failed = false;
    let pre_model = run_pre_model_gates()?;

    if pre_model.failed {
        anyhow::bail!("one or more pre-model gates failed");
    }

    // recall + leakage (needs the model)
    let threshold = pre_model.pack.policy.deidentification.recall_threshold;
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
           airplane eval            check recall/leakage against eval/golden-run.txt\n\
           airplane eval --update   refresh eval/golden-run.txt intentionally\n\
           airplane scrub \"<text>\"  scrub arbitrary text\n\
           airplane gates           run the harness gates\n\
           airplane gates-fast      run no-model gates for fast iteration\n\
           \n\
           Needs llama-server for eval/gates model layer:  ./scripts/serve-model.sh"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verb = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    let result = match verb {
        "eval" => match parse_eval_mode(args.get(2).map(|s| s.as_str())) {
            Some(mode) => cmd_eval(mode),
            None => {
                usage();
                std::process::exit(2);
            }
        },
        "gates" => cmd_gates(),
        "gates-fast" => cmd_gates_fast(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_mode_defaults_to_check_and_update_is_explicit() {
        assert_eq!(parse_eval_mode(None), Some(EvalMode::Check));
        assert_eq!(parse_eval_mode(Some("--check")), Some(EvalMode::Check));
        assert_eq!(parse_eval_mode(Some("--update")), Some(EvalMode::Update));
        assert_eq!(parse_eval_mode(Some("--write")), None);
    }

    #[test]
    fn only_reference_pack_can_update_reference_golden_run() {
        assert!(is_default_pack(Path::new(DEFAULT_PACK_DIR)));
        assert!(!is_default_pack(Path::new("packs/my-pack")));
    }
}
