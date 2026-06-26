# Bonsai Ecosystem Brief (scoped)
*Distilled 2026-06-25 from isolated archeology over local sources: `full-ctx.md`, `~/projects/bonsai/` (scripts/serve.sh, docs, models), `~/projects/gemini-music` (branch `bonsai-local`), `~/projects/the-night-library/BONSAI_101.md`, and memory `prismml-bonsai-local-footguns`. Kept deliberately tight so it scopes the build without re-poisoning context.*

> **Disambiguation first:** PrismML's model is **`prism-ml/Bonsai-*`** (1.7B/4B/8B family, Caltech spinout, Apache 2.0, April 2026). It is **unrelated** to `deepgrove/Bonsai` (0.5B, 2025). AI summaries blend their specs — always check the HF org prefix.

## A. Runtime reality (drives the `InferenceProvider` port)

**CLI / server path (proven, working):**
- Stock Homebrew **`llama-server`**, OpenAI-compatible API on `127.0.0.1:8080`.
- `POST /v1/chat/completions`, chat messages (system + user).
- **Structured output is forced** via `response_format: {type: "json_schema", json_schema: {name, strict: true, schema}}` → llama.cpp compiles the schema to a GBNF grammar and constrains decoding. **The schema is the contract.**
- Parse `choices[0].message.content` → `json.loads`; defensively strip ` ``` ` fences and a leading `<think>` block; **re-validate/clamp host-side** regardless.
- Sampling (PrismML model card): `temperature 0.5, top_k 20, top_p 0.85` — set explicitly; defaults drift.
- `gemini-music`'s `services/bonsai_client.py:chat_json` is a **working reference integration** for this exact contract.

**Model artifacts (`~/projects/bonsai/models/`):**
| File | Size | Runs on | Notes |
|---|---|---|---|
| `Ternary-Bonsai-1.7B-F16.gguf` | 3.2GB | Metal (`-ngl 99`), ~125 t/s | default, stock tooling |
| `Ternary-Bonsai-1.7B-TQ2_0.gguf` | 590MB (2.88 BPW) | **CPU only** (`-ngl 0`), ~111 t/s | Metal aborts ("type 35") |
| `Bonsai-8B-Q1_0.gguf` | 1.1GB | **fork only** | stock llama.cpp loads it then runs ~1000× slow |
- **Footgun:** PrismML's `Q1_0` / custom `Q2_0` need the **`PrismML-Eng/llama.cpp` fork** (custom Metal/CUDA kernels). v1 uses stock paths (F16 / mainline-requant TQ2_0). The HTTP port means a fork-built quant slots in later with **zero core change**.
- HF repos: `prism-ml/Ternary-Bonsai-1.7B-gguf`, `prism-ml/Bonsai-8B-gguf`.

**iOS / Apple path:**
- **MLX is the Apple story.** Bonsai **Image 4B** is MLX-native, runs **512² in 9.4s on iPhone 17 Pro Max via mlx-swift** (`prism-ml/bonsai-image-ternary-4B-mlx-2bit`). GitHub `PrismML-Eng/` ships an mlx-swift fork + image demo; a **Locally AI** partnership targets iPhone.
- The **text** family ships **GGUF + MLX** on HF (`prism-ml/`), so MLX text weights exist — **but no local code runs the text model on iOS** (on-device demos here are all HTTP to llama-server).
- **No local evidence** of Core ML / Apple Neural Engine / "Locally AI" wiring for text. iOS text = mlx-swift loading MLX weights (proven for image, not yet wired for text).
- ⚠️ **Headline risk (R1 in the design spec):** 1.7B-text on **iPhone 11 / A13** is unproven. The proven device run is a *different* (image, 4B) model on a *2023* phone. Treat M3 as a measurement gate.

**Browser / WebGPU path (new evidence):**
- Hugging Face hosts `webml-community/bonsai-webgpu`, a running Space for a
  Bonsai 1-bit WebGPU demo.
- Its source uses `@huggingface/transformers` and loads
  `onnx-community/Bonsai-1.7B-ONNX` with `device: "webgpu"` and `dtype: "q1"`.
- This is the best first phone capability test because it checks WebGPU +
  Bonsai-family ONNX execution before we write any code.
- It still does not prove our workflow: no PHI scrub pack, no verifier gate, no
  structured span contract, no offline/cache guarantee, and no Keychain/Secure
  Enclave story.

**Port implications:** method ≈ `complete(messages, json_schema, sampling) -> String`; schema/GBNF is first-class; carry sampling explicitly; return raw string and let the core strip `<think>`, extract JSON, validate, clamp. CLI adapter enforces grammar **server-side**; iOS adapter must enforce it **client-side** (or validate-and-retry).

## B. Positioning (drives the README role-modeling)

- **Thesis:** *intelligence density* — useful intelligence per unit of size and power, not parameter count. Native 1-bit/ternary weights as the design target.
- **Honest mechanics:** "1-bit" = sign-only weights {−1,+1} with a shared per-group scale factor (lineage: BitNet 2017, 1.58-bit LLM 2024). 8B ≈ 1.15GB; claims 14× smaller / 8× faster / 5× more energy-efficient at the edge vs full precision.
- **Self-coined metric caveat:** "intelligence density" = −log(avg error rate) ÷ size. Qwen3-8B **beats** Bonsai-8B on raw MMLU Redux / GSM8K; Bonsai wins only on the ratio. **Do not present as an industry standard.**
- **Strategy:** Apache-2.0 free weights (8B/4B/1.7B), trained on Google v4 TPUs — give away the artifact, own the vocabulary, monetize later (enterprise licensing, hardware co-design, managed inference).
- **People/Caltech:** CEO Babak Hassibi (Caltech); Sahin Lale + Omead Pooladzandi (research); Reza Sadri (strategy). "1-bit not as an endpoint but a starting point."
- **Investors' bets:** Khosla = **value migration** ("most intelligence per unit of energy and cost"); Tracker/Cerberus (Amir Salek, ex-Google TPU) = **silicon co-design**; Google + Caltech compute grants; Ion Stoica endorsement.

**The 2026 inflection (3 forcing functions):** (1) datacenter **power ceiling** — "power is the ultimate bottleneck for scaling AI datacenters"; (2) **edge silicon maturity** — NPUs crossed the line (130 t/s on iPhone 17 Pro Max @ 0.24GB); (3) **privacy/latency pull** — on-device dissolves cloud constraints.

**Phrases the README may echo:** "intelligence per unit of energy and cost" · "intelligence density" · "ship the compute to the data, not the data to the compute" · "value migrating off the datacenter" · "radios-off, provable-by-airplane-mode privacy" · "free under Apache 2.0" · "the six-year-old phone already in a billion pockets" · "density models win *situations*, not benchmarks."

**Do NOT overclaim:** intelligence-density is self-coined and flatters Bonsai; "1-bit" is sign-only weights with grouped scale factors; frontier cloud still wins peak quality. Honesty about this *is* the credibility.

## C. Who the ecosystem is (the demo's audience)
Application developers; edge/on-device builders; the **Rust + AI inference community** (llama.cpp lineage); enterprise **repatriation** buyers; hardware co-design partners. Channels: HF `prism-ml/`, GitHub `PrismML-Eng/`, Apache-2.0 weights, Locally AI (iPhone). The mlx-swift 1-bit path lagging is itself a real adoption footgun — and an opening for a clean reference integration.
