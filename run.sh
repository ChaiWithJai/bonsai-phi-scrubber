#!/usr/bin/env bash
set -euo pipefail
# run.sh — the one entrypoint for local shells over airplane-core.
cd "$(dirname "$0")"

verb="${1:-help}"

build() { cargo build -q --bin airplane; }
BIN="target/debug/airplane"

case "$verb" in
  eval)   build; "$BIN" eval ;;
  scrub)  build; shift; "$BIN" scrub "${1:-}" ;;
  gates)  build; "$BIN" gates ;;
  web)    cargo build -q --bin airplane-web; target/debug/airplane-web ;;
  mcp)    cargo build -q --bin airplane-mcp; target/debug/airplane-mcp ;;
  help|*) cat <<'EOF'
Airplane Mode — run.sh   (on-device PHI scrubber; CLI shell over airplane-core)
  ./run.sh eval            reproduce recall/leakage over the golden set  (the front door)
  ./run.sh scrub "<text>"  scrub arbitrary text
  ./run.sh gates           run the harness gates
  ./run.sh web             serve the Beat 1 demo UI (http://localhost:8088, LAN-accessible)
  ./run.sh mcp             start the stdio MCP shell (agent-callable scrub tool)

Needs the model layer running:  ./scripts/serve-model.sh
Tune contextual passes:         AIRPLANE_EVAL_PASSES=5 ./run.sh eval
Use a different pack:           PACK=packs/my-pack ./run.sh gates
EOF
  ;;
esac
