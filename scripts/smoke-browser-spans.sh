#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

URL="${AIRPLANE_WEB_URL:-http://127.0.0.1:8099}"

python3 - "$URL" <<'PY'
import json
import ssl
import sys
import time
import urllib.request
from urllib.parse import urlsplit

base = sys.argv[1].rstrip("/")
ctx = ssl._create_unverified_context() if urlsplit(base).scheme == "https" else None
note = "Met with Maria Alvarez (CM-204815) Tuesday. Her daughter just started college. Committed to a morning walk."
spans = [
    {"text": "Maria Alvarez", "entity": "PERSON"},
    {"text": "Tuesday", "entity": "DATE"},
    {"text": "daughter just started college", "entity": "FAMILY_DETAIL"},
]

def request(path, payload=None):
    started = time.perf_counter()
    if payload is None:
        req = urllib.request.Request(base + path, method="GET")
    else:
        req = urllib.request.Request(
            base + path,
            data=json.dumps(payload).encode(),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
    with urllib.request.urlopen(req, context=ctx, timeout=180) as resp:
        body = resp.read().decode()
    try:
        parsed = json.loads(body)
    except Exception:
        parsed = body[:200]
    return round(time.perf_counter() - started, 3), parsed

status_s, status = request("/api/status")
span_s, result = request("/api/browser-spans", {"text": note, "spans": spans})
send_s, send = request(
    "/api/send",
    {"send_id": f"browser-span-smoke-{time.time()}", "record": result["record"]},
)
traj_s, trajectory = request("/api/trajectory", {"record": result["record"]})

print(json.dumps({
    "url": base,
    "status_seconds": status_s,
    "status": {
        "model_reachable": status.get("model", {}).get("reachable"),
        "slack_configured": status.get("slack", {}).get("configured"),
        "client_capability": status.get("client_capability"),
    },
    "browser_spans_seconds": span_s,
    "backend": result.get("backend"),
    "gate_pass": result.get("gate_pass"),
    "residual_count": result.get("residual_count"),
    "redactions": result.get("redactions"),
    "send_seconds": send_s,
    "send": send,
    "trajectory_seconds": traj_s,
    "trajectory": trajectory,
}, indent=2))
PY
