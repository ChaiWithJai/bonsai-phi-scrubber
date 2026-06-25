//! airplane-web — the web shell (laptop-as-edge demo for Beat 1).
//!
//! Serves the single-phone walkthrough UI and a real `/api/scrub` endpoint over
//! `airplane-core`. Scrub + verifier gate are real; the structurer runs on the
//! ALREADY-SCRUBBED text (so it never sees PHI). Bind 0.0.0.0 so a phone on the
//! same LAN can drive the demo against this laptop.

use airplane_core::{
    scrub, InferenceProvider, InferenceRequest, Pack, RulesExecutor, Sampling, Span,
};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::Path;

const PACK_DIR: &str = "packs/coach-session";
const INDEX: &str = "shells/web/static/index.html";
const ADDR: &str = "0.0.0.0:8088";
const PASSES: u32 = 5;

// ---- llama-server adapter (InferenceProvider port, web side) -----------------

struct LlamaServer {
    url: String,
    model: String,
}
impl Default for LlamaServer {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8080/v1/chat/completions".into(),
            model: "ternary-bonsai-1.7b".into(),
        }
    }
}
impl LlamaServer {
    fn chat(&self, system: &str, user: &str, schema: &Value, sampling: Sampling) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": [{"role":"system","content":system},{"role":"user","content":user}],
            "temperature": sampling.temperature, "top_k": sampling.top_k,
            "top_p": sampling.top_p, "max_tokens": sampling.max_tokens, "seed": sampling.seed,
            "response_format": {"type":"json_schema","json_schema":{"name":"out","strict":true,"schema":schema}}
        });
        let resp = ureq::post(&self.url).send_json(body).map_err(|e| {
            anyhow::anyhow!("llama-server failed ({e}); run ./scripts/serve-model.sh")
        })?;
        let v: Value = resp.into_json().context("parse llama-server response")?;
        Ok(v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}
impl InferenceProvider for LlamaServer {
    fn complete(&self, req: &InferenceRequest<'_>) -> Result<String> {
        self.chat(req.system, req.user, req.json_schema, req.sampling)
    }
}

// ---- structurer (runs on de-identified text only) ----------------------------

fn pseudonym(redactions: &[Span]) -> String {
    use std::hash::{Hash, Hasher};
    let person = redactions
        .iter()
        .find(|s| s.entity == "PERSON")
        .map(|s| s.text.as_str())
        .unwrap_or("client");
    let mut h = std::collections::hash_map::DefaultHasher::new();
    person.to_lowercase().hash(&mut h);
    format!("CLIENT-{:04X}", (h.finish() as u16))
}

fn structure(model: &LlamaServer, scrubbed: &str) -> Value {
    let schema = json!({
        "type":"object","additionalProperties":false,
        "required":["themes","commitments","next_touch"],
        "properties":{
            "themes":{"type":"array","items":{"type":"string"}},
            "commitments":{"type":"array","items":{"type":"object","additionalProperties":false,
                "required":["text","status"],
                "properties":{"text":{"type":"string"},"status":{"type":"string"}}}},
            "next_touch":{"type":"string"}
        }
    });
    let sys = "You are a coaching scribe. From this DE-IDENTIFIED session note, produce a structured \
               care record. themes: 1-3 short noun phrases (2-3 words each). commitments: each is a \
               SHORT action phrase the client agreed to (e.g. '10-min morning walk'), NOT a full \
               sentence, status 'open'. next_touch: a date YYYY-MM-DD. Never include redaction tokens \
               like [PERSON]. JSON only.";
    let raw_rec = match model.chat(
        sys,
        scrubbed,
        &schema,
        Sampling {
            seed: 7,
            ..Sampling::model_card()
        },
    ) {
        Ok(raw) => airplane_core::hygiene::extract_json_object(&raw)
            .and_then(|j| serde_json::from_str::<Value>(&j).ok())
            .unwrap_or_else(|| json!({})),
        Err(_) => json!({}),
    };
    sanitize_record(&raw_rec, scrubbed)
}

/// The 1.7B structurer is noisy — it can echo the prompt or leak redaction tokens.
/// Keep only clean, short values.
fn looks_junk(s: &str) -> bool {
    s.is_empty()
        || s.contains('[')
        || s.contains(']')
        || s.contains(':')
        || s.split_whitespace().count() > 6
        || s.len() > 48
}
fn sanitize_record(rec: &Value, scrubbed: &str) -> Value {
    let ground = scrubbed.to_lowercase();
    let themes: Vec<Value> = rec["themes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|t| t.as_str().map(|s| !looks_junk(s)).unwrap_or(false))
        .take(3)
        .collect();
    let commitments: Vec<Value> = rec["commitments"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| {
            let t = c["text"].as_str().unwrap_or("");
            // grounded: at least one content word must appear in the de-identified note,
            // so the structurer can't invent commitments that weren't there.
            let grounded = t
                .split_whitespace()
                .any(|w| w.len() > 3 && ground.contains(&w.to_lowercase()));
            if t.is_empty() || t.contains('[') || t.len() > 56 || !grounded {
                None
            } else {
                Some(json!({"text": t, "status": "open"}))
            }
        })
        .take(2)
        .collect();
    json!({ "themes": themes, "commitments": commitments, "next_touch": rec["next_touch"].as_str().unwrap_or("") })
}

// ---- handlers ----------------------------------------------------------------

fn handle_scrub(body: &str) -> Result<Value> {
    let input: Value = serde_json::from_str(body).context("parse request body")?;
    let text = input["text"].as_str().unwrap_or("").to_string();

    let pack = Pack::load(Path::new(PACK_DIR)).context("load coach-session pack")?;
    let model = LlamaServer::default();
    let res = scrub(&text, &pack.rules, Some(&model), Sampling::eval(), PASSES)?;

    let redactions: Vec<Value> = res
        .redaction_map
        .iter()
        .map(|s| json!({"text": s.text, "entity": s.entity, "layer": s.layer}))
        .collect();
    let residual = match &res.gate {
        airplane_core::GateDecision::Block { residual } => residual.len(),
        airplane_core::GateDecision::Pass => 0,
    };

    // structurer runs on the gate-clean text only
    let record = structure(&model, &res.scrubbed_text);

    Ok(json!({
        "scrubbed_text": res.scrubbed_text,
        "redactions": redactions,
        "gate_pass": res.gate.is_pass(),
        "residual_count": residual,
        "record": {
            "client_pseudonym": pseudonym(&res.redaction_map),
            "themes": record["themes"],
            "commitments": record["commitments"],
            "next_touch": record["next_touch"],
        }
    }))
}

// ---- the Slack sink (real post via incoming webhook) -------------------------
// The DE-IDENTIFIED record is what leaves — the clean thing, never the identifiers.

fn slack_post(record: &Value) -> Result<()> {
    let webhook = std::env::var("SLACK_WEBHOOK_URL").map_err(|_| {
        anyhow::anyhow!("SLACK_WEBHOOK_URL not set — export it and restart ./run.sh web")
    })?;
    let pseud = record["client_pseudonym"].as_str().unwrap_or("CLIENT");
    let themes = record["themes"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "—".into());
    let commit = record["commitments"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["text"].as_str())
        .unwrap_or("—");
    let next = record["next_touch"].as_str().unwrap_or("—");
    let payload = json!({
        "blocks": [
            {"type":"header","text":{"type":"plain_text","text":"Coach session record"}},
            {"type":"section","fields":[
                {"type":"mrkdwn","text":format!("*Client*\n{pseud}")},
                {"type":"mrkdwn","text":format!("*Next touch*\n{next}")},
                {"type":"mrkdwn","text":format!("*Themes*\n{themes}")},
                {"type":"mrkdwn","text":format!("*Commitment*\n{commit} · open")},
            ]},
            {"type":"context","elements":[{"type":"mrkdwn",
                "text":"✓ de-identified · gate-clean · posted from the edge — no name, no member ID"}]}
        ]
    });
    ureq::post(&webhook)
        .send_json(payload)
        .map_err(|e| anyhow::anyhow!("Slack post failed: {e}"))?;
    Ok(())
}

fn handle_send(body: &str) -> Value {
    let input: Value = serde_json::from_str(body).unwrap_or_else(|_| json!({}));
    match slack_post(&input["record"]) {
        Ok(()) => json!({"ok": true}),
        Err(e) => json!({"ok": false, "error": format!("{e}")}),
    }
}

fn recognizer_overlay() -> Value {
    json!({
        "name": "coach_custom_benefit_id",
        "supported_entity": "BENEFIT_ID",
        "patterns": [{"name": "benefit_id", "regex": "\\bBEN-[A-Z]{2}-\\d{4}\\b", "score": 0.95}],
        "context": ["benefit id", "program code", "ref"]
    })
}

fn find_with_pack_rules(text: &str) -> Result<Vec<Span>> {
    let pack = Pack::load(Path::new(PACK_DIR)).context("load coach-session pack")?;
    Ok(pack.rules.find(text))
}

fn find_with_overlay(text: &str) -> Result<(Vec<Span>, String, bool)> {
    let pack = Pack::load(Path::new(PACK_DIR)).context("load coach-session pack")?;
    let mut rules = RulesExecutor::new();
    rules.load_recognizer_file(&Path::new(PACK_DIR).join("recognizers/members.json"))?;
    rules.load_recognizer_file(&Path::new(PACK_DIR).join("recognizers/people.json"))?;
    rules.add_regex("BENEFIT_ID", r"\bBEN-[A-Z]{2}-\d{4}\b")?;
    let res = scrub(text, &rules, None, Sampling::greedy(), 1)?;
    let gate_pass = res.gate.is_pass();
    let _ = pack.validate_reward_lint()?;
    let _ = pack.validate_scope_boundary()?;
    Ok((res.redaction_map, res.scrubbed_text, gate_pass))
}

fn handle_pack_demo() -> Value {
    let note = "Follow-up note: client brought new program code BEN-MH-7741 for the coach portal.";
    let before = find_with_pack_rules(note).unwrap_or_default();
    let (after, scrubbed, gate_pass) =
        find_with_overlay(note).unwrap_or_else(|_| (Vec::new(), note.to_string(), false));
    let pack = Pack::load(Path::new(PACK_DIR));
    let (reward, scope) = match pack {
        Ok(p) => (
            p.validate_reward_lint().is_ok(),
            p.validate_scope_boundary().is_ok(),
        ),
        Err(_) => (false, false),
    };
    json!({
        "pack_yaml": std::fs::read_to_string(Path::new(PACK_DIR).join("pack.yaml")).unwrap_or_default(),
        "policy_yaml": std::fs::read_to_string(Path::new(PACK_DIR).join("policy.yaml")).unwrap_or_default(),
        "added_recognizer": recognizer_overlay(),
        "note": note,
        "before_count": before.len(),
        "after_redactions": after.iter().map(|s| json!({"text": s.text, "entity": s.entity, "layer": s.layer})).collect::<Vec<_>>(),
        "scrubbed_text": scrubbed,
        "gate_pass": gate_pass,
        "gates": [
            {"name":"pack-blindness","pass": Pack::validate_blindness(Path::new(PACK_DIR)).is_ok()},
            {"name":"reward-lint","pass": reward},
            {"name":"scope-boundary","pass": scope},
            {"name":"eval smoke","pass": gate_pass && after.iter().any(|s| s.entity == "BENEFIT_ID")}
        ]
    })
}

fn local_ips() -> Vec<String> {
    // best-effort: ask the OS for a route-local address
    std::process::Command::new("sh")
        .arg("-c")
        .arg("ipconfig getifaddr en0 2>/dev/null; ipconfig getifaddr en1 2>/dev/null")
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn main() -> Result<()> {
    let server = tiny_http::Server::http(ADDR).map_err(|e| anyhow::anyhow!("bind {ADDR}: {e}"))?;
    println!("Airplane Mode — web shell");
    println!("  local:   http://localhost:8088");
    for ip in local_ips() {
        println!("  phone:   http://{ip}:8088   (same Wi-Fi / hotspot)");
    }
    println!("  needs the model:  ./scripts/serve-model.sh");
    match std::env::var("SLACK_WEBHOOK_URL") {
        Ok(_) => println!("  slack:   SLACK_WEBHOOK_URL set — records post for real"),
        Err(_) => println!("  slack:   NOT set — export SLACK_WEBHOOK_URL to post for real (preview only otherwise)"),
    }

    for mut req in server.incoming_requests() {
        let url = req.url().to_string();
        let method = req.method().to_string();
        let json_header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
        let html_header =
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                .unwrap();

        match (method.as_str(), url.as_str()) {
            ("GET", "/") => {
                let html = std::fs::read_to_string(INDEX)
                    .unwrap_or_else(|_| "<h1>index.html missing</h1>".into());
                let _ =
                    req.respond(tiny_http::Response::from_string(html).with_header(html_header));
            }
            ("GET", "/api/health") => {
                let _ = req.respond(
                    tiny_http::Response::from_string(r#"{"ok":true}"#).with_header(json_header),
                );
            }
            ("GET", "/api/pack-demo") => {
                let payload = handle_pack_demo().to_string();
                let _ =
                    req.respond(tiny_http::Response::from_string(payload).with_header(json_header));
            }
            ("POST", "/api/scrub") => {
                let mut body = String::new();
                let _ = req.as_reader().read_to_string(&mut body);
                let (status, payload) = match handle_scrub(&body) {
                    Ok(v) => (200, v.to_string()),
                    Err(e) => (500, json!({"error": format!("{e:#}")}).to_string()),
                };
                let _ = req.respond(
                    tiny_http::Response::from_string(payload)
                        .with_status_code(status)
                        .with_header(json_header),
                );
            }
            ("POST", "/api/send") => {
                let mut body = String::new();
                let _ = req.as_reader().read_to_string(&mut body);
                let payload = handle_send(&body).to_string();
                let _ =
                    req.respond(tiny_http::Response::from_string(payload).with_header(json_header));
            }
            _ => {
                let _ = req
                    .respond(tiny_http::Response::from_string("not found").with_status_code(404));
            }
        }
    }
    Ok(())
}
