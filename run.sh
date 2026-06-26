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
  gates)  build; "$BIN" gates; ./scripts/smoke-ethical-gate-fixtures.sh --bin "$BIN" ;;
  gates-fast) build; "$BIN" gates-fast; ./scripts/smoke-ethical-gate-fixtures.sh --bin "$BIN" ;;
  web)    cargo build -q --bin airplane-web; target/debug/airplane-web ;;
  gpu-probe) shift; ./scripts/serve-gpu-probe.sh "$@" ;;
  phone-observe) shift; ./scripts/wait-phone-capability.sh "$@" ;;
  phone-request-observe) shift; ./scripts/wait-phone-browser-request.sh "$@" ;;
  https-proxy) shift; ./scripts/serve-local-https-proxy.sh "$@" ;;
  vendor-browser-runtime) shift; ./scripts/vendor-browser-runtime.sh "$@" ;;
  vendor-browser-model) shift; ./scripts/vendor-browser-model.sh "$@" ;;
  public-graph) ./scripts/check-public-graph.sh ;;
  mcp)    cargo build -q --bin airplane-mcp; target/debug/airplane-mcp ;;
  ios-sim) (cd shells/ios && swift test && swift build) ;;
  slack-smoke) shift; ./scripts/smoke-slack-sink.sh "$@" ;;
  browser-span-smoke) shift; ./scripts/smoke-browser-spans.sh "$@" ;;
  help|*) cat <<'EOF'
Airplane Mode — run.sh   (on-device PHI scrubber; CLI shell over airplane-core)
  ./run.sh eval            check recall/leakage against eval/golden-run.txt
  ./run.sh eval --update   intentionally refresh eval/golden-run.txt
  ./run.sh scrub "<text>"  scrub arbitrary text
  ./run.sh gates           run the harness gates
  ./run.sh gates-fast      run no-model policy/provenance gates for fast iteration
  ./run.sh web             serve the Beat 1 demo UI (default http://localhost:8099, LAN-accessible)
  ./run.sh gpu-probe       serve capability-only GPU probe (safe to tunnel; no notes)
  ./run.sh phone-observe   wait for phone browser capability/model telemetry
  ./run.sh phone-request-observe
                            wait for passive iPhone browser request telemetry
  ./run.sh https-proxy     serve local HTTPS proxy for phone secure-context testing
  ./run.sh vendor-browser-runtime
                            download local Transformers.js runtime into .airplane/
  ./run.sh vendor-browser-model
                            download local Bonsai q1 browser model into .airplane/
  ./run.sh public-graph     fail if raw spikes/logs/local artifacts are tracked
  ./run.sh mcp             start the stdio MCP shell (agent-callable scrub tool)
  ./run.sh ios-sim         verify the simulator-safe iOS shell scaffold (no hardware proof)
  ./run.sh slack-smoke     post one synthetic gate-clean record through the Slack sink
  ./run.sh browser-span-smoke
                            profile browser-span finalizer -> Slack -> trajectory

Needs the model layer running:  ./scripts/serve-model.sh
Tune contextual passes:         AIRPLANE_EVAL_PASSES=5 ./run.sh eval --update
Bound model requests:           AIRPLANE_MODEL_TIMEOUT_SECS=120 ./run.sh gates
Bound Slack sends:              AIRPLANE_SLACK_TIMEOUT_SECS=15 ./run.sh web
Use a different pack:           PACK=packs/my-pack ./run.sh gates
Phone demo server:              ./run.sh web
Phone observation:              AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh phone-observe
Slack smoke target:             AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke
Browser span smoke target:      AIRPLANE_WEB_URL=https://127.0.0.1:8443 ./run.sh browser-span-smoke
EOF
  ;;
esac
