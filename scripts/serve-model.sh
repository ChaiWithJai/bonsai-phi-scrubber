#!/usr/bin/env bash
set -euo pipefail
# serve-model.sh — get + serve PrismML Bonsai locally for the demo / eval.
# Stock llama.cpp, OpenAI-compatible API on 127.0.0.1:8080.
# Override with env: MODEL_PATH, MODEL_SHA256, HF_REPO, MODEL_REV, MODEL_FILE, NGL.
cd "$(dirname "$0")/.."

MODEL_DIR="${MODEL_DIR:-models}"
MODEL_FILE="${MODEL_FILE:-Ternary-Bonsai-1.7B-F16.gguf}"   # F16, ~3.2GB, runs on Metal
MODEL_PATH="${MODEL_PATH:-$MODEL_DIR/$MODEL_FILE}"
HF_REPO="${HF_REPO:-prism-ml/Ternary-Bonsai-1.7B-gguf}"     # Apache-2.0
MODEL_REV="${MODEL_REV:-983b5dec2ff16aab79990711ba0f828a499a7e6a}"
MODEL_SHA256="${MODEL_SHA256:-00be231b1ba8ab8b45db35d288f897fa9b5836bb6ad41762b759b6bc990b4fea}"
NGL="${NGL:-99}"                                            # GPU layers; set NGL=0 for CPU-only quants

command -v llama-server >/dev/null 2>&1 || {
  echo "ERROR: llama-server not found. Install llama.cpp:  brew install llama.cpp"
  echo "(see docs/model-setup.md)"; exit 1; }

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

verify_model() {
  local path="$1"
  local actual
  actual="$(sha256_file "$path")"
  if [ "$actual" != "$MODEL_SHA256" ]; then
    echo "ERROR: model hash mismatch for $path"
    echo "  expected: $MODEL_SHA256"
    echo "  actual:   $actual"
    echo "Refusing to serve an unpinned or tampered model."
    exit 1
  fi
}

if [ ! -f "$MODEL_PATH" ]; then
  echo "Model not found at $MODEL_PATH — downloading $MODEL_FILE (~3.2GB, Apache-2.0) from $HF_REPO@$MODEL_REV"
  mkdir -p "$(dirname "$MODEL_PATH")"
  URL="https://huggingface.co/$HF_REPO/resolve/$MODEL_REV/$MODEL_FILE?download=true"
  tmp="$MODEL_PATH.partial"
  rm -f "$tmp"
  curl -L --fail --progress-bar -o "$tmp" "$URL" || {
    echo
    echo "Download failed. Get the GGUF manually from https://huggingface.co/$HF_REPO"
    echo "and place it at $MODEL_PATH (or set MODEL_PATH). See docs/model-setup.md."
    rm -f "$tmp"; exit 1; }
  verify_model "$tmp"
  mv "$tmp" "$MODEL_PATH"
fi

verify_model "$MODEL_PATH"

echo "Serving $MODEL_PATH on http://127.0.0.1:8080  (ngl=$NGL, sha256=${MODEL_SHA256:0:12}...)"
exec llama-server -m "$MODEL_PATH" --port 8080 --host 127.0.0.1 -ngl "$NGL" --alias ternary-bonsai-1.7b
