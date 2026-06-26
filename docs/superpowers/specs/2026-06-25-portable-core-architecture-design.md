# Design — Portable Core Architecture for "Airplane Mode"
*Date: 2026-06-25 · Status: Seeding (awaiting Jai's review) · Resolves canon drift item D1*
*Companion to: `files/rfc-002-final-ship.md` (build spec), `CANON.md` (canon index), `docs/seed/bonsai-ecosystem-brief.md` (scoped research)*

---

## 0. Why this doc exists

The canon (`files/`) converged on a demo but left one decision unresolved (CANON.md **D1**: build it in Swift, or Rust?). This doc resolves it — but more importantly, it reframes the question. The choice isn't a language preference; it's **what the unit of adoption is**, because the demo's strategic job is to be a *reusable pattern*, not a one-off iPhone app.

Three decisions were made with Jai (2026-06-25), each laid out with alternatives and tradeoffs:

1. **Adoption unit:** *Portable core, iPhone is instance #1.* Ship the iPhone demo, but the scrub+gate core is a runtime-agnostic library; portability is **architected and shown** (≥2 shells from one core), not built across N runtimes.
2. **Core seam:** *Thin auditable Rust core + ports.* The core owns the deterministic, legally-critical logic; platform-divergent concerns are ports each shell implements. The model is **outside** the core.
3. **Agent surface:** *Ship an agent-callable shell in v1.* An MCP/service adapter, alongside iOS + CLI — cheap because it's a thin transport over the same core API.

---

## 1. The throughline: the architecture *is* the pitch

The audience is **application developers** who want to build a business case for PrismML/Bonsai in their own vertical — Vinod Khosla's value-migration thesis ("intelligence per unit of energy and cost," not datacenter scale) applied to *their* workload. The demo **role-models** that move.

So the demo must not just *assert* repatriation is possible — it must *be shaped like* a repatriation. The thing an enterprise would pull out of a datacenter (a sensitive inference workload) becomes **one portable, owned, auditable core**; everything platform-specific becomes a swappable adapter. An app developer reads the structure and sees the template for their own migration:

> Take the workload that "has to" live in a datacenter → run it on a 1-bit model at the edge → make it yours through declarative config (a *pack*) → reach it from any runtime (phone, server, agent).

That is hexagonal (ports-and-adapters) architecture, and it maps 1:1 onto the value-migration story.

---

## 2. The architecture

```
                    ┌─────────────────────────────────────────────┐
                    │   airplane-core  (Rust crate)                │
                    │   portable · signed · the "repatriated       │
                    │   workload" · the one thing we OWN           │
                    │                                              │
                    │   • rules executor   (regex/context/checksum)│
                    │   • verifier gate    (default-deny egress)   │  ← the legal control
                    │   • pipeline orchestration                   │
                    │   • pack loader + schema validation          │
                    │   • <think>-strip / JSON extract / clamp     │
                    │                                              │
                    │   PORTS (traits the core depends on):        │
                    │   InferenceProvider · SecureStore            │
                    │   Capture · Sink                             │
                    └─────────────────────────────────────────────┘
                       ▲                ▲                 ▲
          ┌────────────┘                │                 └────────────┐
   ADAPTER: iOS shell           ADAPTER: CLI shell          ADAPTER: MCP/agent shell
   (Swift, via UniFFI)          (Rust bin)                  (Rust bin)
   • mlx-swift (MLX weights)    • llama-server HTTP         • llama-server HTTP
   • Secure Enclave             • keychain / file           • ephemeral / file
   • ASR + airplane-mode UI     • stdin / arg               • MCP tool-call input
   • Slack sink                 • Slack / mock sink          • Slack / tool-result sink
   ── the conversion proof ──   ── reproduction (Tier 1) ── ── agent-native proof ──
   "the data never leaves"       "the numbers reproduce"     "an agent can do it too"
```

**Dependency inversion is the literal mechanism.** `airplane-core` depends only on the four port *traits* — never on iOS, llama.cpp, MLX, or Slack. Each shell injects concrete implementations. Swap `InferenceProvider` from mlx-swift to llama-server and the **identical recall-critical rules+gate logic runs** — which is exactly what makes a reproduced recall number meaningful, and exactly what an adopter needs to trust the pattern in their own runtime.

Swift stops being "the risk" and becomes one thin adapter. Rust owns the core because it must be fast, memory-safe, portable across runtimes, and **owned** (the verifier gate is a legal control, not a dependency you import). The core ships as a Rust crate — which also plants the flag in the Rust+AI inference community PrismML already courts.

### 2.1 The ports (the contract)

| Port | Core depends on (abstraction) | iOS adapter | CLI / MCP adapter |
|---|---|---|---|
| `InferenceProvider` | `complete(messages, json_schema, sampling) -> String` | mlx-swift, MLX weights, **client-side** constrained decode | HTTP → llama-server `/v1/chat/completions`, `response_format: json_schema` (grammar **server-side**) |
| `SecureStore` | put/get the redaction map; never leaves device | Secure Enclave / Keychain | keychain (mac) / encrypted file |
| `Capture` | yield raw note text | on-device ASR + text | stdin / argument |
| `Sink` | accept **scrubbed record only** | Slack Block Kit | Slack / mock / MCP tool-result |

**Why inference is a port, not in the core** (from `docs/seed/bonsai-ecosystem-brief.md`): Bonsai runs two genuinely different ways — `llama-server` (OpenAI-compatible HTTP, GGUF, grammar enforced server-side) on a laptop, vs. in-process `mlx-swift` (MLX weights) on iOS. Pulling either into the core would either bloat it or force a single runtime. Keeping it behind a trait keeps the core thin and auditable and lets a fork-built quant slot in with zero core change.

**The reliability mechanism the port must carry:** ternary Bonsai cannot freehand JSON; structured output requires **JSON-schema-constrained decoding**. So the schema is a first-class parameter of `complete(...)`. The CLI adapter gets this free (`response_format: json_schema` → llama.cpp GBNF). The **iOS/MLX adapter has no server to enforce it** and must either apply a GBNF client-side or schema-validate-and-retry. The core always treats output as "text that should be JSON," then strips any `<think>` block, extracts, validates against the pack schema, and clamps — never trusting raw model output (Constitution IX).

---

## 3. ADR-014 (the decision, recorded)

See `files/adr-014-portable-rust-core.md` for the terse record. In short:

- **Decision:** Build `airplane-core` as a thin Rust crate (rules executor + verifier gate + pipeline + pack loader) exposing four port traits. Ship three adapters in v1: iOS (UniFFI→Swift), CLI, MCP. The model is an `InferenceProvider` port, never in the core.
- **Supersedes:** every "native Swift regex engine" assertion in `engineering-spec-v1`, `phi-scrubber-technical-survey`, `clinic-pack-pattern`, `component-design`, and the stack table in `rfc-002-final-ship` §6. Confirms and generalizes the build-vs-reuse call in `battletest-harness-engineering`.
- **Consequence:** one auditable trust core, reproducible on a laptop (Tier 1) and provable on a phone (Tier 2), embeddable by agents (MCP) — the same artifact serves all three adoption audiences.

---

## 4. What v1 builds (scope / YAGNI)

**In:** `airplane-core` crate with the four ports; CLI adapter (the reproduction front door); iOS adapter (the airplane-mode proof); MCP adapter (the agent-native proof); the `coach-session` reference pack + ~20 golden notes; the eval harness as the gate; `run.sh` (eval | demo | scrub | gates).

**Architected but not built across N (north-star, documented in README):** server/WASM shells; additional sinks; additional packs; fleet/control-plane. The README shows *how* you'd target them — the port traits are the proof you could.

**Explicitly out (per RFC-002):** RL policy *training* (we grow the environment, train nothing — ADR-013); k8s control-plane build; certification; second live adopter; voice on non-iOS.

---

## 5. Risks (honest, with where they're resolved)

| # | Risk | Severity | Resolution / gate |
|---|---|---|---|
| **R1** | **iPhone 11 (A13) may not run 1.7B *text* acceptably.** The only locally-proven on-device run is the **4B image** model on an **iPhone 17 Pro Max** via mlx-swift. 1.7B-text-on-A13 is unproven. | **High — it's the headline** | **M3 is a validation gate, not an assumption.** Build the CLI (Tier 1) front door first so the *correctness* claim never depends on the phone. If A13 won't perform, options: (a) accept a newer-but-still-old phone and soften "2019," (b) ship F16/Metal vs a smaller quant, (c) keep the device tier as "iPhone-class" and let Tier 1 carry reproduction. Decide with a real on-device measurement, not now. |
| **R2** | iOS adapter must implement **constrained decoding client-side** (no `response_format` in mlx-swift). | Medium | Port accepts a JSON-schema/GBNF; iOS adapter does constrained sampling or validate-and-retry; core clamps regardless. Prove on CLI (grammar server-side) first; port the constraint to iOS at M3. |
| **R3** | PrismML's best quants (Q1_0, 8B) are **fork-gated** (`PrismML-Eng/llama.cpp`); stock tooling silently runs them ~1000× slow. | Medium | v1 uses stock-tooling paths only: `Ternary-Bonsai-1.7B-F16` (Metal) or `TQ2_0` (CPU). The HTTP port means a fork-built quant slots in later with zero core change. Pin model **by hash**; verify in `run.sh`. |
| **R4** | "Drawing the owl" — three shells before the first demo converts. | Medium | Build order enforces M1 (core + CLI + eval) **before** M3 (iOS); the MCP shell is a thin transport over the same core API and lands after the loop is end-to-end. The architecture makes shells cheap; the core is the only hard part. |
| **R5** | Overclaiming PrismML's framing. | Low | README echoes their language but flags the honest caveats: "intelligence density" is self-coined and loses on raw benchmarks; "1-bit" is sign-only weights with grouped scale factors; frontier cloud still wins peak quality. |

---

## 6. How this gets built: loop engineering

This repo is organized so the build runs as a **harnessed loop** (per `docs/deprecations/canon/harnessed-loop.md`), not as one big session. A champion drops the repo into Codex or Claude Code and runs the loop; the filesystem + git are the memory and system of record.

- **The harness / operating manual:** `AGENTS.md` (and `CLAUDE.md` → it). The non-negotiable gates, the trust-boundary hard rules, and how to run one iteration.
- **The work queue:** `backlog/` — milestones M0–M5 as task files, each with a goal, a reproduction criterion, the gate that admits it, and a "done-when." The loop pulls the next unblocked task, does it, runs the gates, and checks it off.
- **The gates:** `gates/` — every ethical/legal/security requirement as an automated check the loop cannot bypass (recall, leakage, pack-blindness, reward-lint ★, scope-boundary ★, signature, manifest, PHI-free-telemetry).
- **HITL checkpoints:** the loop pauses for Jai's review at milestone boundaries and before anything irreversible (publishing, signing, the public-repo gitops loop via `gh`).

The terminal state of *this* seeding pass is: **Jai reviews → we ready the public repo and run the gitops loop via `gh` CLI.** No remotes are touched now.

---

## 7. Spec self-review

- Placeholders: none — risks and unknowns are named as risks (R1–R5), not TBDs.
- Consistency: ADR-014 (§3) matches the architecture (§2) and scope (§4); supersession targets match CANON.md D1.
- Scope: single coherent architecture decision + its build organization; the *implementation* is decomposed into the M0–M5 backlog, not this doc.
- Ambiguity: the inference-port split (CLI=server-side grammar, iOS=client-side) is made explicit in §2.1 and R2.
