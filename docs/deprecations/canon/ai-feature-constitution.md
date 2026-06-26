# Build the Harness, Not the Demo
### A constitution for AI feature development + a routing tree
*Synthesized from Mitchell Hashimoto's "My AI Adoption Journey" (Feb 2026) and applied to the on-device PHI-scrubbing AI scribe.*

---

## The thesis

Mitchell's six-step framework is written about *using agents to build software*. But every principle has a twin meaning at *runtime*, when the thing you ship is itself AI-powered. "Give the agent a way to verify its work" (build) and "wrap the model in a verifier the product depends on" (runtime) are the same instinct. This constitution reads every article both ways. The same is true of the negative space — the hardest-won knowledge is *which parts should not be AI at all*, whether you're delegating to a coding agent or designing a pipeline stage.

---

## The Constitution — ten articles

**I. Agent, not oracle.**
A chat box that emits text you have to babysit is not a feature; it's a liability. An AI component earns its place only when it acts in a loop with tools and verification.
*Build:* use agents that read files, run programs, make HTTP requests — not a chatbot you paste into.
*Runtime:* the model stage is wrapped in tools that check it, never raw generate-and-pray.

**II. Verification is the feature, not the QA step.**
The fastest path to a good AI result is a cheap, fast, automatic check sitting directly behind the output. If a stage has no verifier, it is not shippable.
*Build:* when the agent can verify itself, it fixes its own mistakes and stops regressing.
*Runtime:* the PHI-leak detector behind the scrubber is part of the product, not a test you run later.

**III. Earn the map before you delegate the territory.**
Do the hard version by hand once before letting the model own it. You cannot review, route, or harness what you don't understand from first principles.
*Build:* reproduce your own work manually, then make an agent match it blind.
*Runtime:* hand-scrub real-shaped notes and hand-write the schema before the model touches either.

**IV. Respect the negative space.**
The most valuable knowledge is which stages must *not* be AI. Routing, schema validation, transport, secret storage, redaction-map handling — deterministic code. Reserve the model for genuinely ambiguous language or perception. Over-applying AI is the most common failure mode.

**V. Split planning from execution.**
Never "draw the owl" in one shot. Vague or multi-step work becomes a plan (approved at a gate) and then an execution pass.
*Build:* separate planning sessions from execution sessions.
*Runtime:* the scribe classifies *what kind of note this is* before it extracts fields.

**VI. Harness every scar.**
Every mistake becomes a permanent guardrail — a rule line or a programmed check — never a one-off correction. A system's reliability is the running integral of its scars turned into harnesses.
*Build:* each agent failure → an AGENTS.md line or a validation script it can call.
*Runtime:* each missed identifier class → a new detector rule the pipeline runs forever.

**VII. Declare the quality bar per surface, up front.**
Decide review and verification intensity per component *before* building. The trust boundary gets every line reviewed and a formal check; cosmetic surfaces ship and iterate. Applying one bar everywhere either wastes effort or ships risk.

**VIII. The human controls the interrupt.**
Asynchrony is the goal, but the human owns the context-switch. Systems report; they do not ping.
*Build:* turn off agent notifications — check in at natural breaks.
*Runtime:* the field operator is interrupted only by events that matter (a flagged risk signal), never chattered at.

**IX. Default deny at the trust boundary.**
Inputs, and especially AI *outputs*, are untrusted until proven safe. The boundary enforces this by construction, not by discipline.
*Build:* treat agent-generated contributions as default-deny, not default-trust.
*Runtime:* nothing crosses the network until the verifier passes; a sink structurally cannot receive raw PHI.

**X. Goal-directed systems break things outside their scope.**
A model optimizing to "succeed" at its local task will hallucinate, leak, or damage state to get there. Test coverage and guardrails must exceed what a human-only system would need. Assume the locally-helpful, globally-wrong move *will* happen.

---

## The Decision Tree — routing the core process

```
1. SHOULD THIS STAGE BE AI AT ALL?  (Articles IV, IX)
   ├─ Rule-expressible / deterministic?        → NO AI. Write code.
   ├─ Correctness-critical, no verifier exists? → NO AI, or AI + mandatory human gate.
   └─ Genuinely ambiguous language/perception
      AND verifiable?                           → AI candidate → go to 2.

2. ONE-SHOT OR PLAN-THEN-EXECUTE?  (Article V)
   ├─ Vague / multi-step?           → Split: plan (gate approves) → execute.
   └─ Well-scoped single transform? → One-shot, but only with a verifier (Article II).

3. WHAT IS THE QUALITY BAR / WHO REVIEWS?  (Article VII)
   ├─ Touches trust boundary (PHI, money, irreversible action)?
   │                                → Highest bar: verify every output + review, default-deny.
   └─ Cosmetic / easily reversible? → Ship-and-iterate, spot check.

4. CONFIDENCE ROUTING  (Mitchell's slam-dunk vs. hard task)
   ├─ High confidence the model nails it? → Outsource / automate it (slam dunk).
   └─ Low confidence, genuinely hard?     → Competitive parallel runs (two models,
                                            pick the better) OR do it by hand and
                                            reproduce with an agent later (Article III).

5. WHEN IT FAILS — FIX OR HARNESS?  (Article VI)
   ├─ One-off, won't recur?     → Fix it.
   └─ A *class* of mistake?      → Harness it. (Default to harness.)

6. EDGE OR CLOUD?  (our Khosla-thesis extension to Mitchell)
   ├─ Sensitive data / offline / latency- or energy-bound? → On-device (Bonsai).
   └─ Needs frontier reasoning, non-sensitive, online?     → Cloud.
```

---

## Applied to our process — the AI scribe

Our pipeline has four seams plus one invariant: **Source → Scrubber → Structurer → Sink**, with raw PHI and the redaction map never crossing the network. Routed through the tree:

| Stage | AI? (Tree 1) | Mode (Tree 2) | Quality bar (Tree 3) | Edge/cloud (Tree 6) | Harness (Article VI) |
|---|---|---|---|---|---|
| **Source** (capture/ASR) | Commodity model, treat as solved | One-shot | Medium | Edge | Confidence thresholds on transcription |
| **Scrubber** (de-id) | **Yes — the one true AI stage** | One-shot **+ mandatory verifier gate** | **Highest — trust boundary** | **Edge (Bonsai 1.7B)** | Every missed identifier class → new rule |
| **Structurer** (note → typed record) | Yes | **Plan-then-execute** (classify note type → extract) | High | Edge | Schema-validation failures → prompt + rule updates |
| **Sink** (Slack blocks/fields) | **No — deterministic rendering** (Article IV) | n/a | Ship-and-iterate | Either | Block Kit contract tests |

### The load-bearing decisions

**The scrubber is the only stage that is *really* AI**, and it sits on the trust boundary, so it inherits the maximum of every rule: highest quality bar (VII), default-deny output (IX), verification as part of the product (II). Concretely that means **belt-and-suspenders de-id** — the Bonsai model does ambiguous, context-dependent redaction, and a deterministic rules engine (Microsoft Presidio / Philter-style) runs as the harness backstop. The verifier gate is non-negotiable: if either layer flags residual PHI, the record does not advance to *any* sink. That is Article II made physical.

**The structurer is the one stage that earns plan-then-execute** (V): classify the note archetype first (CHP home visit vs. clinical follow-up), *then* extract against that archetype's schema. Its verifier is *not* the model — schema validation is deterministic code (IV, negative space). The model proposes; the schema disposes.

**The sink is not AI** and saying so out loud is the point. Rendering a typed record into Block Kit fields is a pure function. Keeping it dumb is what lets ecosystem users write new sinks safely (IX): a sink that never sees a model and never sees PHI cannot leak either.

### Earn the map first (Article III)

Before any of this is delegated to a coding agent or a model: **hand-scrub ~20 real-shaped notes** to derive the identifier taxonomy and the note-archetype schema by hand. That manual pass *is* the spec for both the scrubber harness and the structurer schema. Skipping it means you can't review or harness what you ship.

### Build-time routing (Mitchell, literally)

- **Slam dunks to hand a coding agent** (Tree 4, high confidence): the Block Kit renderer, the schema validator, the CLI scaffolding, the OpenAI-compatible local server wiring. Well-scoped, verifiable, boring — outsource them.
- **Do by hand / reproduce later** (III): the trust-boundary architecture and the de-id taxonomy. Low-confidence, high-stakes — own them.
- **End-of-day / warm-start research task** (Mitchell step 3): survey existing clinical de-identification libraries (Presidio, Philter, and the HIPAA Safe Harbor 18-identifier set) and report back pros/cons as the rules-backstop candidate set. Read-only; no shipping.

---

## One-line test for any change to this demo

> If extending it requires forking the core instead of editing config and adding one interface implementation, the seams are wrong — and if any stage emits AI output with no verifier behind it, it isn't done.
