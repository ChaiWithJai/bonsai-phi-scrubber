# Implementation Plan — Airplane Mode
### How to build it, and how to make a stranger reproduce it
*New workstream. The acceptance criterion is not "it runs for us" — it's "a stranger clones it and reproduces our numbers."*

---

## The acceptance test

> A stranger clones the repo, runs **one command** on their own laptop, and the eval harness prints the same recall and leakage numbers the README claims — against the golden notes that ship in the repo. If they also have an iPhone 11+, they build the app and watch the airplane-mode loop run offline.

Belief is not earned by a video or a pitch. It's earned by **self-verifying reproduction**: the same eval harness that gates our build is the one the stranger runs to check us.

---

## The reproduction ladder

Belief should scale down to whatever friction the person will accept. Three rungs, decreasing friction, decreasing strength:

| Tier | Who | What they do | What it proves | Friction |
|---|---|---|---|---|
| **2 — Run on device** | Mac + iPhone 11+ + Xcode | Build the app, toggle airplane mode, run the loop | The full thesis: useful AI, offline, 2019 phone | High |
| **1 — Run the eval** (primary) | Anyone with a laptop | `./run.sh eval` | The scrub is **real** and the numbers **reproduce** | Low |
| **0 — Watch** | Anyone | Play the recorded demo | Belief, not reproduction | None |

**The strategy is to make Tier 1 the front door.** It decouples "the scrub genuinely works" (reproducible on any machine, no phone) from "it runs offline on a phone" (the device upgrade). Most people will never build an iOS app; almost anyone will run one command.

---

## The belief engine: the eval harness

The eval harness is the reproducibility backbone. It:
- runs the scrubber (rules executor ∪ Bonsai) over the ~20 golden notes in the repo,
- scores recall / precision / **leakage rate** per entity type against the committed expected redactions,
- runs the verifier gate and reports its block decisions,
- prints a result that **matches a committed `golden-run.txt`**.

A stranger runs it and sees their output equal ours. That equality *is* the proof. Constitution II — verification is the feature — turned into a trust mechanism for outsiders.

To make the equality hold deterministically: **greedy decoding (temperature 0)** for the eval, a **pinned model hash**, a **pinned runtime fork commit**, and a **pinned pack version**. Document the tiny acceptable variance (if any) so a near-match still reads as success.

---

## Repo layout (what's in the box)

```
airplane-mode/
├── README.md                 # the contract + the ladder
├── run.sh                    # one command:  eval | demo | scrub "<note>"
├── Dockerfile                # hermetic Tier-1 path (OS-independent)
├── models/                   # fetched by hash (not committed); checksum-verified
├── core/                     # rules executor · verifier gate · scrubber · structurer
├── packs/
│   └── coach-session/        # the reference pack: recognizers/ schema/ policy/ sink/
├── eval/
│   ├── golden/               # ~20 synthetic notes (real content, fake people)
│   ├── expected/             # expected redactions per note
│   └── golden-run.txt        # committed expected eval output — the self-verify target
├── follow-up/                # the generator (heuristic v1)
├── ios/                      # the on-device app (Tier 2)
└── docs/                     # constitution · PRD · RFC-001 · component design
```

Synthetic-only notes are not just a safety rule — they're a **reproducibility feature**: anyone can run the whole thing with zero data-access requests.

---

## Build track — milestones, each with a reproduction criterion

Order falls out of the component-design tiers and dependencies. Each milestone is defined by *what a stranger can reproduce at it*.

**M0 — Content (the spec).**
Author the `coach-session` pack + ~20 hand-labeled golden notes.
→ *Reproduces:* nothing yet runnable, but the truth set exists. This is the gating dependency.

**M1 — The reproducibility backbone (ship this first).**
Rules executor + Bonsai scrubber + verifier gate + eval harness, as a laptop CLI (`./run.sh eval`, `./run.sh scrub`). Dockerized.
→ *Reproduces:* a stranger runs `./run.sh eval`, matches `golden-run.txt`. **This is the front door; it must land before anything iOS.**

**M2 — The full loop on desktop.**
Structurer + follow-up generator + Slack sink. `./run.sh demo` runs end-to-end (text in → clean record to Slack → follow-up note).
→ *Reproduces:* the whole loop, minus the phone — note reaches a stranger's own Slack channel.

**M3 — On-device (the airplane-mode proof).**
The iOS app: Bonsai via mlx-swift, the loop, the visible gate, airplane-mode toggle.
→ *Reproduces:* Tier 2 — build on an iPhone 11+, watch it run offline.

**M4 — Belief for non-builders.**
Record the demo; finalize README and the five-file reveal as a scripted exercise.
→ *Reproduces:* Tier 0 — watch and believe; then drop to Tier 1.

**M5 — (gated) The environment grows.**
Trajectory recorder + the counter, riding the existing gate. No policy training.
→ *Reproduces:* the counter ticks up, gate-clean, on a stranger's run.

The critical insight in the order: **M1 before M3.** The correctness claim must be reproducible on a laptop before the on-device claim is even attempted, or reproduction stays locked behind a device most people don't have.

---

## Reproducibility primitives (the engineering that makes "it works on my machine" impossible)

- **Pin everything:** model hash, runtime fork commit, pack version, dependency lockfile. (This is the in-toto/SLSA/cosign control-plane work, now serving reproduction.)
- **Fetch the model by hash**, verify checksum in `run.sh` before running.
- **Hermetic Tier-1 path:** a Dockerfile so the laptop reproduction is OS-independent (CPU Bonsai on 20 notes is slow but fine).
- **Determinism:** temperature 0 / greedy for the eval; document expected variance.
- **One command, three verbs:** `eval` (reproduce numbers), `demo` (full loop), `scrub "<note>"` (try your own text).
- **Committed golden run:** the expected output sits in the repo; matching it is the success signal.

---

## Risks to reproducibility → mitigations

| Risk | Mitigation |
|---|---|
| Model inference nondeterminism | Greedy decode + pinned hash; document acceptable variance |
| "Works on my machine" drift | Lockfiles + pinned fork commit + Dockerized Tier-1 path |
| iOS build friction blocks reproduction | Tier 1 (laptop) is the front door; iOS is Tier 2; video is Tier 0 |
| Model download size/host flakiness | Fetch by hash with checksum verify; document fallback mirror |
| No real data to test against | Synthetic golden notes ship in-repo — a feature, not a gap |

---

## One-line test
> A stranger with only a laptop runs one command, matches the committed golden run, and believes — and if they have a 2019 phone, they watch it happen offline. Reproduction, not persuasion, is what earns the bet.
