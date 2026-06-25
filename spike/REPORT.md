# M0.5 Feasibility Spike — Report
*Run 2026-06-25 on an M4 Pro (24GB), stock Homebrew `llama-server`, `Ternary-Bonsai-1.7B-F16.gguf` (Metal, `-ngl 99`). Throwaway harness `spike/spike.py` (stdlib only) over 10 hand-labeled synthetic coaching notes (`spike/notes/`, 42 labeled identifiers). This validates feasibility — it is **not** a production recall claim.*

## Verdict: **GO for M1.** ✅

The core product claim holds on PrismML's actual 1.7B model: the three-layer scrubber catches contextual PHI a regex layer cannot, and the eval is deterministic enough to reproduce.

## Q1 — Recall (the product claim) ✅
| Configuration | Recall | Hard-case recall | Leakage |
|---|---|---|---|
| Rules only (regex) | **26.2%** (11/42) | 0/33 | 31 leaked |
| **Rules ∪ Bonsai-1.7B** | **100%** (42/42) | **all** | **0** |

- Bonsai-1.7B caught **every** hard case the rules missed — names buried in prose ("Marcus finally called his mom"), dates in words ("the second Tuesday of next month", "this past Thanksgiving"), and disclosed relationships ("his daughter who just started at Northwestern"). 31 hard catches were Bonsai-only; 2 were caught by both layers.
- Per-entity recall was 100% across PERSON, DATE, ADDRESS, EMAIL, FAMILY_DETAIL, LOCATION, MEMBER_ID, ORG, PHONE.
- Latency: ~4.7s/note (10 notes in ~47s), F16 on Metal.

**Honest caveat:** 22 **over-redactions** (predicted spans not in the label set). This is expected and acceptable under the recall-first posture (a false positive over-redacts; a false negative leaks). Precision is the **tuning knob for M1**, not a blocker — and it's exactly what the verifier gate + the eval harness are for.

## Q2 — Determinism (the reproduction ladder) ✅ (conditional)
- At **model-card sampling** (temp 0.5 / top_k 20 / top_p 0.85): span sets **VARY** across 5 runs — expected, it's sampling.
- At **temp-0 greedy** (top_k 1): output is **byte-identical** across runs (sha256 stable).
- **Implication:** `golden-run.txt` can be an exact-match target **if and only if** `./run.sh eval` pins **temp-0 greedy**. Bake that into the eval path (it's already the plan). Re-confirm on the real Rust pipeline at M1.

## Q3 — On-device (R1) — ⏸ UNPROVEN, needs hardware (Jai)
Not testable here — requires a physical device + mlx-swift. The only locally-proven on-device Bonsai run remains the *4B image* model on an *iPhone 17 Pro Max*. **1.7B text on iPhone 11 / A13 is still the open headline risk.** → M0.5-T05, blocked on Jai. Until measured, the CLI (Tier-1) carries the reproduction claim and the "2019 phone" line stays provisional.

## What this de-risks for the build
- The **three-layer scrubber** (rules ∪ Bonsai → verifier gate) is the right shape — the union is doing real work (74-point recall lift from the model layer).
- The **`InferenceProvider` port** contract is validated against the working llama-server path (`response_format: json_schema`, explicit sampling, `<think>`-strip). The iOS/MLX adapter inherits the same contract (R2) but must enforce the grammar client-side.
- The **recall gate** at ≥0.99 is plausible on 1.7B — pending a larger, non-author-labeled note set.

## Honest limits of this spike
- N=10, **synthetic and author-labeled** — proves feasibility and the qualitative gap, not a production recall number. M0 expands to ~20 notes; a real recall claim needs an independent label set.
- F16 (3.2GB) on a Mac, not the on-device quant (R1).
- The harness is throwaway; the real measurement is the M1 eval harness over the real Rust core.

*Raw machine-readable last run: `spike/last-run.json` (regenerated per run).*
