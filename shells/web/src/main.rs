//! airplane-web — the web shell (laptop-as-edge demo for Beat 1).
//!
//! Serves the single-phone walkthrough UI and a real `/api/scrub` endpoint over
//! `airplane-core`. Scrub + verifier gate are real; the structurer runs on the
//! ALREADY-SCRUBBED text (so it never sees PHI). Bind 0.0.0.0 so a phone on the
//! same LAN can drive the demo against this laptop.

use airplane_core::{
    scrub, GateDecision, InferenceProvider, InferenceRequest, Pack, RulesExecutor, Sampling, Span,
    VerifierGate,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const PACK_DIR: &str = "packs/coach-session";
const INDEX: &str = "shells/web/static/index.html";
const DEFAULT_ADDR: &str = "0.0.0.0:8088";
const PASSES: u32 = 5;
static TRAJECTORY_COUNT: AtomicUsize = AtomicUsize::new(0);

fn repo_path(rel: &str) -> PathBuf {
    let direct = PathBuf::from(rel);
    if direct.exists() {
        direct
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel)
    }
}

fn pack_path() -> PathBuf {
    repo_path(PACK_DIR)
}

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
    let mut themes: Vec<String> = rec["themes"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|t| t.as_str().map(|s| s.trim().to_string()))
        .filter(|t| {
            let grounded = t
                .split_whitespace()
                .any(|w| w.len() > 3 && ground.contains(&w.to_lowercase()));
            !looks_junk(t) && grounded
        })
        .take(3)
        .collect();
    if themes.is_empty() {
        themes = fallback_themes(scrubbed);
    }
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

fn fallback_themes(scrubbed: &str) -> Vec<String> {
    let lower = scrubbed.to_lowercase();
    let candidates = [
        ("family transition", ["daughter", "college", "family"]),
        ("daily movement", ["walk", "morning", "exercise"]),
        ("routine building", ["daily", "routine", "committed"]),
        ("follow-through", ["commit", "committed", "plan"]),
        ("support planning", ["support", "next", "touch"]),
    ];
    let mut out = Vec::new();
    for (label, needles) in candidates {
        if needles.iter().any(|n| lower.contains(n)) {
            out.push(label.to_string());
        }
        if out.len() == 3 {
            break;
        }
    }
    if out.is_empty() {
        out.push("coaching follow-up".to_string());
    }
    out
}

fn clinical_risk_flags(raw_text: &str) -> Vec<String> {
    let lower = raw_text.to_lowercase();
    let mut flags = Vec::new();
    let self_harm = [
        "suicide",
        "self-harm",
        "self harm",
        "kill myself",
        "hurt myself",
    ];
    if self_harm.iter().any(|term| lower.contains(term)) {
        flags.push("self_harm_risk".to_string());
    }
    let crisis = ["crisis", "unsafe", "danger to myself", "danger to others"];
    if crisis.iter().any(|term| lower.contains(term)) {
        flags.push("crisis_language".to_string());
    }
    flags
}

fn follow_up_record(record: &Value, risk_flags: &[String]) -> Value {
    if !risk_flags.is_empty() {
        return json!({
            "follow_ups": ["Pause coaching follow-up; surface human escalation path."],
            "autonomy_delta": {
                "logged": true,
                "signals": ["surface_human_escalation"],
                "direction": "safety_first"
            }
        });
    }

    let commitment = record["commitments"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| c["text"].as_str())
        .unwrap_or("");
    let nudge = if commitment.is_empty() {
        "Choose one self-directed next step before the next touch.".to_string()
    } else {
        format!("Before the next touch, try this once on your own: {commitment}.")
    };
    json!({
        "follow_ups": [nudge],
        "autonomy_delta": {
            "logged": true,
            "signals": ["self_initiated", "commitment_completed"],
            "direction": "client_led"
        }
    })
}

// ---- handlers ----------------------------------------------------------------

fn handle_scrub(body: &str) -> Result<Value> {
    let input: Value = serde_json::from_str(body).context("parse request body")?;
    let text = input["text"].as_str().unwrap_or("").to_string();

    let pack = Pack::load(&pack_path()).context("load coach-session pack")?;
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
    let risk_flags = clinical_risk_flags(&text);
    let followup = follow_up_record(&record, &risk_flags);

    Ok(json!({
        "scrubbed_text": res.scrubbed_text,
        "redactions": redactions,
        "gate_pass": res.gate.is_pass(),
        "residual_count": residual,
        "record": {
            "client_pseudonym": pseudonym(&res.redaction_map),
            "themes": record["themes"],
            "commitments": record["commitments"],
            "follow_ups": followup["follow_ups"],
            "risk_flags": risk_flags,
            "autonomy_delta": followup["autonomy_delta"],
            "next_touch": record["next_touch"],
        }
    }))
}

// ---- the Slack sink (real post via webhook or bot token) ---------------------
// The DE-IDENTIFIED record is what leaves — the clean thing, never the identifiers.

#[derive(Debug, Deserialize)]
struct SinkConfig {
    #[serde(rename = "channelMap", default)]
    channel_map: HashMap<String, String>,
    #[serde(default)]
    credentials: SinkCredentials,
}

#[derive(Debug, Default, Deserialize)]
struct SinkCredentials {
    #[serde(default)]
    ref_name: String,
    #[serde(rename = "ref", default)]
    ref_alias: String,
}

impl SinkCredentials {
    fn keychain_ref(&self) -> &str {
        if self.ref_alias.is_empty() {
            &self.ref_name
        } else {
            &self.ref_alias
        }
    }
}

fn load_sink_config() -> Result<SinkConfig> {
    let text = std::fs::read_to_string(pack_path().join("sink.yaml")).context("read sink.yaml")?;
    serde_yaml::from_str(&text).context("parse sink.yaml")
}

fn slack_channel(config: &SinkConfig) -> String {
    std::env::var("SLACK_CHANNEL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| config.channel_map.get("default").cloned())
        .unwrap_or_else(|| "#coach-records".to_string())
}

fn keychain_secret(service: &str) -> Option<String> {
    if service.trim().is_empty() {
        return None;
    }
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", service, "-w"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!secret.is_empty()).then_some(secret)
}

fn slack_bot_token(config: &SinkConfig) -> Option<String> {
    std::env::var("SLACK_BOT_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| keychain_secret(config.credentials.keychain_ref()))
}

fn slack_status() -> Value {
    let config = match load_sink_config() {
        Ok(c) => c,
        Err(e) => return json!({"configured": false, "route": "unavailable", "error": format!("{e:#}")}),
    };
    let channel = slack_channel(&config);
    if std::env::var("SLACK_WEBHOOK_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_some()
    {
        return json!({"configured": true, "route": "webhook", "channel": channel});
    }
    if slack_bot_token(&config).is_some() {
        return json!({"configured": true, "route": "bot_token", "channel": channel});
    }
    json!({
        "configured": false,
        "route": "preview",
        "channel": channel,
        "error": "set SLACK_WEBHOOK_URL or SLACK_BOT_TOKEN"
    })
}

fn slack_blocks(record: &Value) -> Value {
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
    let follow = record["follow_ups"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|f| f.as_str())
        .unwrap_or("—");
    let risk = record["risk_flags"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|r| r.as_str())
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "none".into());
    let next = record["next_touch"].as_str().unwrap_or("—");
    json!([
        {"type":"header","text":{"type":"plain_text","text":"Coach session record"}},
        {"type":"section","fields":[
            {"type":"mrkdwn","text":format!("*Client*\n{pseud}")},
            {"type":"mrkdwn","text":format!("*Next touch*\n{next}")},
            {"type":"mrkdwn","text":format!("*Themes*\n{themes}")},
            {"type":"mrkdwn","text":format!("*Commitment*\n{commit} · open")},
            {"type":"mrkdwn","text":format!("*Follow-up*\n{follow}")},
            {"type":"mrkdwn","text":format!("*Risk flags*\n{risk}")},
        ]},
        {"type":"context","elements":[{"type":"mrkdwn",
            "text":"✓ de-identified · gate-clean · autonomy signals only · no name, no member ID"}]}
    ])
}

fn slack_post(record: &Value) -> Result<()> {
    let config = load_sink_config()?;
    let blocks = slack_blocks(record);
    if let Some(webhook) = std::env::var("SLACK_WEBHOOK_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        let payload = json!({ "blocks": blocks });
        ureq::post(&webhook)
            .send_json(payload)
            .map_err(|e| anyhow::anyhow!("Slack webhook post failed: {e}"))?;
        return Ok(());
    }

    let token = slack_bot_token(&config).ok_or_else(|| {
        anyhow::anyhow!(
            "Slack sink not configured — set SLACK_WEBHOOK_URL, or SLACK_BOT_TOKEN plus SLACK_CHANNEL / sink.yaml channelMap"
        )
    })?;
    let channel = slack_channel(&config);
    let payload = json!({
        "channel": channel,
        "blocks": blocks
    });
    let resp = ureq::post("https://slack.com/api/chat.postMessage")
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json; charset=utf-8")
        .send_json(payload)
        .map_err(|e| anyhow::anyhow!("Slack bot post failed: {e}"))?;
    let body: Value = resp.into_json().context("parse Slack response")?;
    if body["ok"].as_bool() != Some(true) {
        anyhow::bail!(
            "Slack bot post failed: {}",
            body["error"].as_str().unwrap_or("unknown_error")
        );
    }
    Ok(())
}

fn handle_send(body: &str) -> Value {
    let input: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return json!({"ok": false, "error": format!("parse request body: {e}")}),
    };
    let text = trajectory_text(&input["record"]);
    if text.trim().is_empty() {
        return json!({"ok": false, "error": "Slack record is empty"});
    }
    let pack = match Pack::load(&pack_path()).context("load coach-session pack") {
        Ok(p) => p,
        Err(e) => return json!({"ok": false, "error": format!("{e:#}")}),
    };
    if let GateDecision::Block { residual } = VerifierGate::new(&pack.rules).check(&text) {
        return json!({
            "ok": false,
            "error": "Slack gate blocked residual identifiers",
            "residual_count": residual.len()
        });
    }
    match slack_post(&input["record"]) {
        Ok(()) => json!({"ok": true}),
        Err(e) => json!({"ok": false, "error": format!("{e}")}),
    }
}

fn trajectory_text(record: &Value) -> String {
    let mut parts = Vec::new();
    for key in ["themes", "follow_ups"] {
        if let Some(items) = record[key].as_array() {
            parts.extend(items.iter().filter_map(|v| v.as_str()).map(str::to_string));
        }
    }
    if let Some(items) = record["commitments"].as_array() {
        parts.extend(
            items
                .iter()
                .filter_map(|v| v["text"].as_str())
                .map(str::to_string),
        );
    }
    parts.join("\n")
}

fn handle_trajectory(body: &str) -> Value {
    let input: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return json!({"ok": false, "error": format!("parse request body: {e}")}),
    };
    let text = trajectory_text(&input["record"]);
    if text.trim().is_empty() {
        return json!({"ok": false, "error": "trajectory record is empty"});
    }
    let pack = match Pack::load(&pack_path()).context("load coach-session pack") {
        Ok(p) => p,
        Err(e) => return json!({"ok": false, "error": format!("{e:#}")}),
    };
    match VerifierGate::new(&pack.rules).check(&text) {
        GateDecision::Pass => {
            let count = TRAJECTORY_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
            json!({"ok": true, "count": count})
        }
        GateDecision::Block { residual } => json!({
            "ok": false,
            "error": "trajectory gate blocked residual identifiers",
            "residual_count": residual.len()
        }),
    }
}

fn benefit_recognizer(pack_dir: &Path) -> Value {
    std::fs::read_to_string(pack_dir.join("recognizers/benefits.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn find_with_baseline_rules(text: &str) -> Result<Vec<Span>> {
    let mut rules = RulesExecutor::new();
    let pack_dir = pack_path();
    rules.load_recognizer_file(&pack_dir.join("recognizers/members.json"))?;
    rules.load_recognizer_file(&pack_dir.join("recognizers/people.json"))?;
    Ok(rules.find(text))
}

fn find_with_real_pack(text: &str) -> Result<(Vec<Span>, String, bool)> {
    let pack = Pack::load(&pack_path()).context("load coach-session pack")?;
    let res = scrub(text, &pack.rules, None, Sampling::greedy(), 1)?;
    let gate_pass = res.gate.is_pass();
    pack.validate_reward_lint()?;
    pack.validate_scope_boundary()?;
    Ok((res.redaction_map, res.scrubbed_text, gate_pass))
}

fn pack_reveal_files(pack_dir: &Path) -> Vec<Value> {
    [
        ("recognizers", "recognizers/benefits.json"),
        ("schema", "schema.yaml"),
        ("policy", "policy.yaml"),
        ("sink", "sink.yaml"),
        ("eval", "eval/expected/note-01.json"),
    ]
    .into_iter()
    .map(|(role, rel)| {
        let text = std::fs::read_to_string(pack_dir.join(rel)).unwrap_or_default();
        let preview = text.lines().take(12).collect::<Vec<_>>().join("\n");
        json!({"role": role, "path": rel, "preview": preview})
    })
    .collect()
}

fn handle_pack_demo() -> Value {
    let note = "Follow-up note: client brought new program code BEN-MH-7741 for the coach portal.";
    let before = find_with_baseline_rules(note).unwrap_or_default();
    let (after, scrubbed, gate_pass) =
        find_with_real_pack(note).unwrap_or_else(|_| (Vec::new(), note.to_string(), false));
    let pack_dir = pack_path();
    let pack = Pack::load(&pack_dir);
    let (reward, scope) = match pack {
        Ok(p) => (
            p.validate_reward_lint().is_ok(),
            p.validate_scope_boundary().is_ok(),
        ),
        Err(_) => (false, false),
    };
    json!({
        "pack_yaml": std::fs::read_to_string(pack_dir.join("pack.yaml")).unwrap_or_default(),
        "policy_yaml": std::fs::read_to_string(pack_dir.join("policy.yaml")).unwrap_or_default(),
        "pack_files": pack_reveal_files(&pack_dir),
        "added_recognizer": benefit_recognizer(&pack_dir),
        "note": note,
        "before_count": before.len(),
        "after_redactions": after.iter().map(|s| json!({"text": s.text, "entity": s.entity, "layer": s.layer})).collect::<Vec<_>>(),
        "scrubbed_text": scrubbed,
        "gate_pass": gate_pass,
        "gates": [
            {"name":"pack-blindness","pass": Pack::validate_blindness(&pack_path()).is_ok()},
            {"name":"reward-lint","pass": reward},
            {"name":"scope-boundary","pass": scope},
            {"name":"pack eval smoke","pass": gate_pass && after.iter().any(|s| s.entity == "BENEFIT_ID")}
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
    let addr = std::env::var("AIRPLANE_WEB_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let server = tiny_http::Server::http(&addr).map_err(|e| anyhow::anyhow!("bind {addr}: {e}"))?;
    let port = addr.rsplit_once(':').map(|(_, p)| p).unwrap_or("8088");
    println!("Airplane Mode — web shell");
    println!("  local:   http://localhost:{port}");
    for ip in local_ips() {
        println!("  phone:   http://{ip}:{port}   (same Wi-Fi / hotspot)");
    }
    println!("  needs the model:  ./scripts/serve-model.sh");
    match std::env::var("SLACK_WEBHOOK_URL") {
        Ok(_) => println!("  slack:   SLACK_WEBHOOK_URL set — records post for real"),
        Err(_) if std::env::var("SLACK_BOT_TOKEN").is_ok() => {
            let route = load_sink_config()
                .map(|c| slack_channel(&c))
                .unwrap_or_else(|_| "#coach-records".to_string());
            println!("  slack:   SLACK_BOT_TOKEN set — records post to {route}")
        }
        Err(_) => println!(
            "  slack:   NOT set — export SLACK_WEBHOOK_URL or SLACK_BOT_TOKEN to post for real"
        ),
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
            ("GET", "/api/status") => {
                let payload = json!({"ok": true, "slack": slack_status()}).to_string();
                let _ =
                    req.respond(tiny_http::Response::from_string(payload).with_header(json_header));
            }
            ("GET", "/favicon.ico") => {
                let _ = req.respond(tiny_http::Response::empty(204));
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
            ("POST", "/api/trajectory") => {
                let mut body = String::new();
                let _ = req.as_reader().read_to_string(&mut body);
                let payload = handle_trajectory(&body).to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TRAJECTORY_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn fallback_themes_are_grounded_for_sample_note() {
        let rec = json!({"themes": ["You are a coaching scribe: produce JSON"], "commitments": []});
        let out = sanitize_record(
            &rec,
            "[PERSON] is the one whose daughter just started college. Committed to a 10-min morning walk daily.",
        );
        let themes = out["themes"].as_array().unwrap();
        assert_eq!(themes[0], "family transition");
        assert!(themes.iter().any(|t| t == "daily movement"));
    }

    #[test]
    fn follow_up_uses_autonomy_signals_only() {
        let rec = json!({"commitments": [{"text": "10-min morning walk", "status": "open"}]});
        let out = follow_up_record(&rec, &[]);
        let encoded = out.to_string();
        assert!(encoded.contains("self_initiated"));
        assert!(encoded.contains("commitment_completed"));
        assert!(!encoded.contains("retention"));
        assert!(!encoded.contains("session_count"));
        assert_eq!(
            out["follow_ups"][0],
            "Before the next touch, try this once on your own: 10-min morning walk."
        );
    }

    #[test]
    fn risk_language_surfaces_escalation_instead_of_nudge() {
        let flags = clinical_risk_flags("Client said they might hurt myself tonight.");
        let rec = json!({"commitments": [{"text": "10-min morning walk", "status": "open"}]});
        let out = follow_up_record(&rec, &flags);
        assert_eq!(flags, vec!["self_harm_risk"]);
        assert!(out["follow_ups"][0]
            .as_str()
            .unwrap()
            .contains("escalation"));
        assert_eq!(
            out["autonomy_delta"]["signals"][0],
            "surface_human_escalation"
        );
    }

    #[test]
    fn sink_config_routes_default_channel() {
        let sink: SinkConfig = serde_yaml::from_str(
            r##"
kind: slack
channelMap:
  default: "#coach-records"
credentials:
  source: keychain
  ref: slack-bot-token
"##,
        )
        .unwrap();
        assert_eq!(slack_channel(&sink), "#coach-records");
        assert_eq!(sink.credentials.keychain_ref(), "slack-bot-token");
    }

    #[test]
    fn slack_status_reports_preview_when_unconfigured() {
        let status = slack_status();
        assert_eq!(status["configured"], false, "{status}");
        assert_eq!(status["route"], "preview", "{status}");
        assert_eq!(status["channel"], "#coach-records", "{status}");
    }

    #[test]
    fn pack_demo_reveals_five_files_and_catches_new_identifier() {
        let out = handle_pack_demo();
        let files = out["pack_files"].as_array().unwrap();
        let roles: Vec<_> = files.iter().filter_map(|f| f["role"].as_str()).collect();
        assert_eq!(
            roles,
            vec!["recognizers", "schema", "policy", "sink", "eval"]
        );
        assert_eq!(out["before_count"], 0, "{out}");
        assert_eq!(out["gate_pass"], true, "{out}");
        assert!(out["scrubbed_text"]
            .as_str()
            .unwrap()
            .contains("[BENEFIT_ID]"));
        assert!(out["gates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["name"] == "pack eval smoke" && g["pass"] == true));
    }

    #[test]
    fn trajectory_gate_blocks_residual_identifier_without_increment() {
        let _guard = TRAJECTORY_TEST_LOCK.lock().unwrap();
        let before = TRAJECTORY_COUNT.load(Ordering::SeqCst);
        let blocked = handle_trajectory(
            r#"{"record":{"themes":["member CM-204815"],"commitments":[],"follow_ups":[]}}"#,
        );
        assert_eq!(blocked["ok"], false, "{blocked}");
        assert_eq!(blocked["residual_count"], 1, "{blocked}");
        assert_eq!(TRAJECTORY_COUNT.load(Ordering::SeqCst), before);
    }

    #[test]
    fn slack_gate_blocks_residual_identifier_before_sink() {
        let blocked = handle_send(
            r#"{"record":{"themes":["member CM-204815"],"commitments":[],"follow_ups":[]}}"#,
        );
        assert_eq!(blocked["ok"], false, "{blocked}");
        assert_eq!(blocked["residual_count"], 1, "{blocked}");
        assert!(
            blocked["error"]
                .as_str()
                .unwrap()
                .contains("Slack gate blocked"),
            "{blocked}"
        );
    }

    #[test]
    fn trajectory_gate_increments_for_clean_record() {
        let _guard = TRAJECTORY_TEST_LOCK.lock().unwrap();
        let before = TRAJECTORY_COUNT.load(Ordering::SeqCst);
        let accepted = handle_trajectory(
            r#"{"record":{"themes":["daily movement"],"commitments":[{"text":"morning walk"}],"follow_ups":["Try this once on your own."]}}"#,
        );
        assert_eq!(accepted["ok"], true, "{accepted}");
        assert_eq!(accepted["count"].as_u64().unwrap(), (before + 1) as u64);
    }

    #[test]
    fn trajectory_gate_rejects_empty_or_invalid_records() {
        let _guard = TRAJECTORY_TEST_LOCK.lock().unwrap();
        let before = TRAJECTORY_COUNT.load(Ordering::SeqCst);
        let invalid = handle_trajectory(r#"{"#);
        let empty =
            handle_trajectory(r#"{"record":{"themes":[],"commitments":[],"follow_ups":[]}}"#);
        assert_eq!(invalid["ok"], false, "{invalid}");
        assert_eq!(empty["ok"], false, "{empty}");
        assert_eq!(TRAJECTORY_COUNT.load(Ordering::SeqCst), before);
    }
}
