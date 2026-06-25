#!/usr/bin/env bash
set -euo pipefail
# serve-model.sh — get + serve PrismML Bonsai locally for the demo / eval.
# Stock llama.cpp, OpenAI-compatible API on 127.0.0.1:8080.
# Override with env: MODEL_PATH, HF_REPO, MODEL_FILE, NGL (0 = CPU only).
cd "$(dirname "$0")/.."

MODEL_DIR="${MODEL_DIR:-models}"
MODEL_FILE="${MODEL_FILE:-Ternary-Bonsai-1.7B-F16.gguf}"   # F16, ~3.2GB, runs on Metal
MODEL_PATH="${MODEL_PATH:-$MODEL_DIR/$MODEL_FILE}"
HF_REPO="${HF_REPO:-prism-ml/Ternary-Bonsai-1.7B-gguf}"     # Apache-2.0
NGL="${NGL:-99}"                                            # GPU layers; set NGL=0 for CPU-only quants

command -v llama-server >/dev/null 2>&1 || {
  echo "ERROR: llama-server not found. Install llama.cpp:  brew install llama.cpp"
  echo "(see docs/model-setup.md)"; exit 1; }

if [ ! -f "$MODEL_PATH" ]; then
  echo "Model not found at $MODEL_PATH — downloading $MODEL_FILE (~3.2GB, Apache-2.0) from $HF_REPO"
  mkdir -p "$(dirname "$MODEL_PATH")"
  URL="https://huggingface.co/$HF_REPO/resolve/main/$MODEL_FILE?download=true"
  curl -L --fail --progress-bar -o "$MODEL_PATH" "$URL" || {
    echo
    echo "Download failed. Get the GGUF manually from https://huggingface.co/$HF_REPO"
    echo "and place it at $MODEL_PATH (or set MODEL_PATH). See docs/model-setup.md."
    rm -f "$MODEL_PATH"; exit 1; }
fi

echo "Serving $MODEL_PATH on http://127.0.0.1:8080  (ngl=$NGL)"
exec llama-server -m "$MODEL_PATH" --port 8080 --host 127.0.0.1 -ngl "$NGL" --alias ternary-bonsai-1.7b
