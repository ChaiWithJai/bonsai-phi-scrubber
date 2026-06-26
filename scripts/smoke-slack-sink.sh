#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${AIRPLANE_WEB_URL:-http://127.0.0.1:8099}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "ERROR: missing required command: $1" >&2
    exit 1
  }
}

need curl
need python3

status="$(curl -fsS "$BASE_URL/api/status")"
python3 - "$status" <<'PY'
import json
import sys

status = json.loads(sys.argv[1])
slack = status.get("slack", {})
model = status.get("model", {})
if not model.get("reachable"):
    raise SystemExit("ERROR: model is not reachable; run ./scripts/serve-model.sh")
if not slack.get("configured"):
    route = slack.get("route", "unknown")
    err = slack.get("error", "missing Slack credential")
    raise SystemExit(f"ERROR: Slack route is {route}; {err}")
print(f"Slack route: {slack.get('route')} -> {slack.get('channel')}")
PY

payload='{
  "record": {
    "client_pseudonym": "CLIENT-7F3A",
    "themes": ["sleep routine", "boundary practice"],
    "commitments": [{"text": "try a 10-minute wind-down routine", "status": "open"}],
    "follow_ups": ["Would you like to keep the 10-minute wind-down routine for two nights and decide what felt useful?"],
    "risk_flags": [],
    "next_touch": "scheduled"
  }
}'

response="$(curl -fsS \
  -H 'Content-Type: application/json' \
  -d "$payload" \
  "$BASE_URL/api/send")"

python3 - "$response" <<'PY'
import json
import sys

response = json.loads(sys.argv[1])
if response.get("ok") is not True:
    raise SystemExit(f"ERROR: Slack send failed: {response.get('error', response)}")
print("Slack smoke: app-originated gated send accepted")
PY
