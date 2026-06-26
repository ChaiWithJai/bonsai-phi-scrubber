//! airplane-mcp — agent-callable stdio shell over airplane-core.
//!
//! The MCP transport is intentionally thin: parse JSON-RPC lines, call the same
//! scrubber/gate core as the CLI, and emit only gate-clean de-identified payloads.

use airplane_core::{scrub, InferenceProvider, InferenceRequest, Pack, Sampling};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const DEFAULT_PACK_DIR: &str = "packs/coach-session";
const TOOL_NAME: &str = "airplane_scrub";

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
        let body = json!({
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
        let v: Value = resp.into_json().context("parse llama-server response")?;
        Ok(v["choices"][0]["message"]["content"]
            .as_str()
            .context("no message content in llama-server response")?
            .to_string())
    }
}

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
    let rel = std::env::var("PACK")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_PACK_DIR.to_string());
    repo_path(&rel)
}

fn tool_descriptor() -> Value {
    json!({
        "name": TOOL_NAME,
        "description": "Scrub a synthetic coaching note through airplane-core and return only gate-clean de-identified output.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["text"],
            "properties": {
                "text": {"type": "string"},
                "passes": {"type": "integer", "minimum": 1, "maximum": 8, "default": 3}
            }
        }
    })
}

fn scrub_payload(args: &Value, provider: &dyn InferenceProvider) -> Result<Value> {
    let text = args["text"]
        .as_str()
        .context("airplane_scrub requires string argument `text`")?;
    let passes = args["passes"].as_u64().unwrap_or(3).clamp(1, 8) as u32;

    let pack_path = pack_path();
    let pack =
        Pack::load(&pack_path).with_context(|| format!("load pack {}", pack_path.display()))?;
    let res = scrub(text, &pack.rules, Some(provider), Sampling::eval(), passes)?;

    let redactions: Vec<Value> = res
        .redaction_map
        .iter()
        .map(|s| json!({"entity": s.entity, "layer": s.layer}))
        .collect();
    if !res.gate.is_pass() {
        anyhow::bail!(
            "verifier gate blocked tool egress after {} redaction(s)",
            redactions.len()
        );
    }

    Ok(json!({
        "scrubbed_text": res.scrubbed_text,
        "gate": "PASS",
        "redaction_count": redactions.len(),
        "redactions": redactions
    }))
}

fn scrub_tool(args: &Value) -> Result<Value> {
    scrub_payload(args, &LlamaServer::default())
}

fn response(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error_response(id: Value, code: i64, message: impl ToString) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message.to_string()}})
}

fn tool_result(payload: Value) -> Value {
    json!({
        "content": [{"type": "text", "text": payload.to_string()}],
        "isError": false
    })
}

fn tool_error(message: impl ToString) -> Value {
    json!({
        "content": [{"type": "text", "text": message.to_string()}],
        "isError": true
    })
}

fn handle(request: Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request["method"].as_str().unwrap_or("");
    match method {
        "initialize" => response(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "airplane-mcp", "version": env!("CARGO_PKG_VERSION")}
            }),
        ),
        "tools/list" => response(id, json!({"tools": [tool_descriptor()]})),
        "tools/call" => {
            let params = &request["params"];
            if params["name"].as_str() != Some(TOOL_NAME) {
                return response(id, tool_error("unknown tool"));
            }
            match scrub_tool(&params["arguments"]) {
                Ok(payload) => response(id, tool_result(payload)),
                Err(e) => response(id, tool_error(format!("{e:#}"))),
            }
        }
        "notifications/initialized" => response(id, json!({})),
        _ => error_response(id, -32601, format!("method not found: {method}")),
    }
}

fn run() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let out = match serde_json::from_str::<Value>(&line) {
            Ok(v) => handle(v),
            Err(e) => error_response(Value::Null, -32700, format!("parse error: {e}")),
        };
        writeln!(stdout, "{out}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("airplane-mcp failed: {e:#}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeModel;

    impl InferenceProvider for FakeModel {
        fn complete(&self, _req: &InferenceRequest<'_>) -> Result<String> {
            Ok(r#"{"spans":[{"text":"Maria Alvarez","entity":"PERSON"}]}"#.to_string())
        }
    }

    #[test]
    fn lists_airplane_scrub_tool() {
        let out = handle(json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}));
        assert_eq!(out["result"]["tools"][0]["name"], TOOL_NAME);
    }

    #[test]
    fn tool_scrubs_without_raw_map_text() {
        let out = scrub_payload(
            &json!({"text": "Met Maria Alvarez (CM-204815)."}),
            &FakeModel,
        )
        .unwrap();
        assert!(out["scrubbed_text"].as_str().unwrap().contains("[PERSON]"));
        assert!(out["scrubbed_text"]
            .as_str()
            .unwrap()
            .contains("[MEMBER_ID]"));
        let encoded = out.to_string();
        assert!(!encoded.contains("CM-204815"));
        assert!(!encoded.contains("Maria Alvarez"));
    }
}
