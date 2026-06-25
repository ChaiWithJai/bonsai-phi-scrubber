# Model setup — get + serve Bonsai locally

The demo and the eval call a local `llama-server` (OpenAI-compatible API on `127.0.0.1:8080`) running PrismML's **Bonsai** 1-bit model. This is what makes the reproducibility claim real: with the model served, anyone can run `./run.sh eval` and get the same recall/leakage numbers.

## Quick start

```bash
brew install llama.cpp          # the runtime (macOS; Linux: build llama.cpp)
./scripts/serve-model.sh        # downloads the model (Apache-2.0) + serves on :8080
```

That's it — leave it running, then in another terminal: `./run.sh eval` or `./run.sh web`.

## The model

- **What:** `prism-ml/Ternary-Bonsai-1.7B-gguf` on Hugging Face — **free, Apache-2.0**. (Note: `prism-ml/Bonsai-*` is the Caltech-spinout family; it is **not** the unrelated `deepgrove/Bonsai`.)
- **Default:** `Ternary-Bonsai-1.7B-F16.gguf` (~3.2GB) — runs on Apple Metal (`-ngl 99`), ~125 tok/s. `serve-model.sh` downloads this by default.
- `scripts/serve-model.sh` is configurable via env: `MODEL_PATH` (point at an existing GGUF), `HF_REPO`, `MODEL_FILE`, `NGL` (`0` = CPU only).

## Footguns (verified)

- **Stock vs. fork-gated quants.** F16 and a mainline-requantized `TQ2_0` (~590MB, **CPU only** — Metal has no TQ2_0 kernels, so use `NGL=0`) run on **stock** llama.cpp. PrismML's `Q1_0` / custom `Q2_0` and the 8B model need PrismML's `PrismML-Eng/llama.cpp` **fork** — on stock tooling they load silently and run ~1000× slow. v1 uses the stock paths.
- **Structured output is grammar-constrained.** The scrubber forces JSON via `response_format: {type: json_schema}`; llama.cpp compiles the schema to a GBNF grammar. This is the reliability mechanism for a 1-bit model — don't remove it.
- **Sampling.** The eval pins seeded sampling (temp 0.5, fixed seeds) so it's both recall-best and reproducible; the model card's interactive sampling is temp 0.5 / top_k 20 / top_p 0.85.

Deeper notes: `docs/seed/bonsai-ecosystem-brief.md`.

## On-device (the vision)

For the native iPhone path, Bonsai ships **MLX** weights on `prism-ml/` for mlx-swift — that's the tamper-evident, radios-off shell tracked in [issue #9](../../issues/9). The GGUF + `llama-server` path here is the laptop edge node and the reproduction front door.
