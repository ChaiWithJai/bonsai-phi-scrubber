#!/usr/bin/env bash
set -euo pipefail
# run.sh — the one entrypoint for local shells over airplane-core.
cd "$(dirname "$0")"

verb="${1:-help}"

build() { cargo build -q --bin airplane; }
BIN="target/debug/airplane"

case "$verb" in
  eval)   build; shift; "$BIN" eval "$@" ;;
  scrub)  build; shift; "$BIN" scrub "${1:-}" ;;
  gates)  build; "$BIN" gates ;;
  web)    cargo build -q --bin airplane-web; target/debug/airplane-web ;;
  mcp)    cargo build -q --bin airplane-mcp; target/debug/airplane-mcp ;;
  slack-smoke) shift; ./scripts/smoke-slack-sink.sh "$@" ;;
  help|*) cat <<'EOF'
Airplane Mode — run.sh   (on-device PHI scrubber; CLI shell over airplane-core)
  ./run.sh eval            check recall/leakage against eval/golden-run.txt
  ./run.sh eval --update   intentionally refresh eval/golden-run.txt
  ./run.sh scrub "<text>"  scrub arbitrary text
  ./run.sh gates           run the harness gates
  ./run.sh web             serve the Beat 1 demo UI (default http://localhost:8088, LAN-accessible)
  ./run.sh mcp             start the stdio MCP shell (agent-callable scrub tool)
  ./run.sh slack-smoke     post one synthetic gate-clean record through the Slack sink

Needs the model layer running:  ./scripts/serve-model.sh
Tune contextual passes:         AIRPLANE_EVAL_PASSES=5 ./run.sh eval --update
Use a different pack:           PACK=packs/my-pack ./run.sh gates
Phone demo server:              AIRPLANE_WEB_ADDR=0.0.0.0:8099 ./run.sh web
Slack smoke target:             AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke
EOF
  ;;
esac
