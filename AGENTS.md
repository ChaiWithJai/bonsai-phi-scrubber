# AGENTS.md — Operating manual for the Airplane Mode build loop

This repo is built by a **harnessed loop**, not one big session. You (a coding agent in Codex or Claude Code) pull the next unblocked task from `backlog/`, do exactly that task, run the gates, commit, and stop. The filesystem and git are the memory and the system of record. A human (Jai) reviews at milestone boundaries.

> **New here? Read in this order:** `README.md` → `CANON.md` (current demo contract) → `docs/demo/onboarding.md` → `docs/demo/how-the-demo-works.md` → this file → `backlog/README.md`. Historical canon and superseded patterns live under `docs/deprecations/`.

---

## What we're building (one paragraph)

An edge PHI scrubber for a mental-health **coaching** scribe demo. A phone browser captures a synthetic session note; the current reliable demo runs the scrubber on the first-party laptop edge, with Browser GPU as the preferred phone-local runtime spike. A **verifier gate** blocks egress until the output is provably clean; only a scrubbed, structured record reaches Slack. The de-id+gate logic is a portable **Rust core** (`airplane-core`) with **ports**. The demo role-models value migration: pull a sensitive workload off the datacenter, run it at the edge, make it yours via a declarative **pack**, reach it from any runtime.

## The architecture in one breath

`airplane-core` (Rust) owns rules executor · verifier gate · pipeline · pack loader. It depends only on four port traits: `InferenceProvider` · `SecureStore` · `Capture` · `Sink`. The **model is a port**, never in the core. Shells: **web** (live phone/laptop demo), **CLI** (reproduction front door), **MCP** (agent-native proof), and **iOS** (deferred/simulator-safe scaffold). See ADR-014 (`files/adr-014-portable-rust-core.md`).

---

## The loop (one iteration)

```
1. PICK    open backlog/ ; take the lowest-numbered task whose deps are ✅ and status is `ready`.
2. PLAN    restate its goal + done-when in one line. If it needs a design decision
           not already recorded in the canon, STOP and ask Jai (HITL).
3. BUILD   do exactly that task. Nothing more (no owl-drawing — see Hard Rules).
4. VERIFY  run `./run.sh gates` (and the task's own done-when check). Everything must pass.
5. RECORD  check the task off in its backlog file; note what changed. Commit on a
           feature branch with a message referencing the task id (e.g. `M1-T03`).
6. STOP    one task per iteration. Report status. Let the loop re-enter or Jai review.
```

Never batch milestones. Never skip VERIFY. A red gate means fix or revert — never disable the gate.

---

## Hard rules (non-negotiable — these are the trust boundary)

1. **Default-deny egress.** Nothing crosses any boundary until the verifier gate proves zero residual identifiers. The gate guards **two** exits: the Slack sink **and** the RL-environment/trajectory store. (Constitution IX.)
2. **The model is never trusted raw.** Bonsai output is "text that should be JSON" — strip `<think>`, extract, validate against the pack schema, clamp. No generate-and-ship. (Constitution II.)
3. **Packs are PHI-blind and code-free.** A pack is five declarative files (`recognizers/ schema/ policy/ sink/ eval/`). It can redefine identifiers/schema/policy/sink — it can **never** see raw PHI, read the redaction map, alter the verifier, or ship executable code. (ADR-005.)
4. **Raw input + redaction map never leave the device.** They live only in `SecureStore` (enclave/keychain). No sink, log, or telemetry ever receives them.
5. **The core stays thin and owned.** Rules executor + gate + pipeline + pack loader are Rust we own and review every line of. Inference/storage/capture/sink are ports. Do not pull a runtime into the core.
6. **Synthetic data only.** Never a real session note — in the repo, the eval set, or a demo.
7. **No owl-drawing.** Build the task in front of you. M1 (core + CLI + eval) lands **before** M3 (iOS). Don't scaffold speculative shells, sinks, or the control plane.
8. **Reproducibility is a feature.** Pin model **by hash**, greedy/temp-0 for eval, pinned deps. A stranger running `./run.sh eval` must match the committed `eval/golden-run.txt`.

## The gates (the harness — `./run.sh gates`)

Every gate encodes a constitution clause or a legal control and **blocks** on failure. Two (★) are our novel contribution — no k8s/CNCF analog:

`recall` · `leakage` · `pack-blindness` · **`reward-lint` ★** (reward references autonomy signals only, never engagement terms) · **`scope-boundary` ★** (escalation path present; coach ≠ therapist) · `signature/provenance` · `manifest/revocation` · `PHI-free-telemetry`.

A requirement that can't yet be a gate stays **prose**, and we say so. The three current prose-only items: autonomy-delta measurement, trajectory re-identification floor, "is the follow-up actually good?" Do not pretend these are enforced.

---

## HITL checkpoints (stop and ask Jai)

- At every **milestone boundary** (end of M0, M1, …).
- Before anything **irreversible or outward-facing**: publishing, signing a release, touching a git **remote**, or running the **public-repo gitops loop via `gh`**.
- Whenever a task needs a **design decision** not already in the canon (don't invent one — record-then-build).
- If a **Hard Rule** would have to bend to make a task pass. (It doesn't bend; surface the conflict.)

## Bonsai footguns (read before touching inference)

See `docs/seed/bonsai-ecosystem-brief.md`. The short version: it's `prism-ml/Bonsai-*` (not `deepgrove`); stock tooling runs **F16** (Metal) or **TQ2_0** (CPU only) — `Q1_0`/8B need the `PrismML-Eng/llama.cpp` fork and run ~1000× slow on stock; force JSON with `response_format: json_schema`; set sampling explicitly (`temp 0.5, top_k 20, top_p 0.85`); **iPhone-11-text is unproven (R1)** — M3 is a measurement gate.

---

*The terminal state of the current phase is Jai's review, then readying the public repo and running the gitops loop via `gh`. Until then, no remotes are touched.*
