# Battletest — Are We Doing Harness Engineering?
### Against Trivedy (LangChain), Horthy (HumanLayer), Hashimoto — and the build-vs-reuse call
*Jai named three practitioners and asked whether to package a LangChain workload or build hyper-performant Rust, reusing existing systems to cut load. Answer: we are doing harness engineering — specifically the hard, auditable branch — and the build is both, at different layers.*

---

## The standard (what the three actually say)

**Vivek Trivedy, LangChain** — *Agent = Model + Harness; "if you're not the model, you're the harness."* The harness is all the code, config, and execution logic that isn't the model — it gives a stateless predictor state, tools, feedback loops, and enforceable constraints. LangChain moved their coding agent from Top 30 to Top 5 on Terminal-Bench 2.0 **by changing only the harness, model fixed** — via self-verification and trace analysis. He names the core tension explicitly: a *lightest-possible* harness that lets the model cook, versus a *deterministic, replay-clean, auditable* harness you can trust at scale — and bets that **"whoever ends up with the best environments ends up with the best agents."**

**Dex Horthy, HumanLayer** — *12-Factor Agents.* The headline: most production "agents" are mostly deterministic code with LLM steps placed at the right points. **Don't use prompts for control flow — use actual control flow.** Own your prompts, own your control flow, stateless reducer (agent as a pure input→output function), small focused steps (<40 instructions), and **contact humans with tool calls** (human-in-the-loop). HumanLayer's product *is* the human-approval pattern; they also ship an Agent Control Plane.

**Mitchell Hashimoto** — every mistake becomes a permanent harness; the harness compounds; give the agent a way to verify; always have an agent running.

---

## Verdict: yes — and we're on the auditable branch

By Trivedy's definition, our **entire system is a harness** wrapped around a tiny 1-bit model: rules executor, verifier gate, structurer, sink, pack config, eval, gates — everything that isn't Bonsai. And we sit deliberately on Trivedy's *second* branch — deterministic, replay-clean, auditable (golden-run reproduction, temp 0, the FSM system) — not the "let the model cook" branch. For a PHI domain you **must** be on that branch; you cannot let a model improvise over patient data. We're a reference implementation of the auditable harness for a regulated domain.

---

## Principle-by-principle mapping

| Their principle | Source | Our analog | Result |
|---|---|---|---|
| Agent = Model + Harness | Trivedy | whole system wraps Bonsai 1.7B | ✅ canonical |
| Deterministic, replay-clean, auditable | Trivedy | golden-run, temp 0, FSM | ✅ (the hard branch) |
| Self-verification + tracing to improve the harness | Trivedy | eval harness + the harnessed loop's scar→harness | ✅ |
| Best environments win; efficient verifiers | Trivedy | de-identified trajectory env + verifier gate | ✅ (and defensible — below) |
| Control flow, not prompts | Horthy | the FSM system; model is one node, not the loop | ✅ (more 12-factor than most) |
| Stateless reducer (input→output) | Horthy | deterministic eval; FSM transitions | ✅ |
| Small, focused steps | Horthy | the pack is small + declarative; scrubber is a narrow tool | ✅ |
| Mostly deterministic code, LLM at key points | Horthy | rules + gate deterministic; Bonsai at one point | ✅ |
| Contact humans with tool calls | Horthy | scope-boundary escalation (`surface_human_escalation`) | ✅ (literally their pattern) |
| Own your context / dumb zone | Horthy | narrow pipeline barely uses context | ✅ (sidestepped by design) |
| Every scar → a permanent harness | Hashimoto | the harnessed loop; gates compound | ✅ |

We pass on every axis the three define. We are not using the term loosely.

---

## Where we extend the field: ethical harnesses

All three build **reliability** harnesses — optimizing task performance, token efficiency, latency, auditability, human approval. None build **ethical** harnesses. Our reward-lint (no engagement term in the reward) and scope-boundary (escalation required) are harnesses whose objective isn't performance but **the user's autonomy and safety** — the system refusing to harm the user even when it could. That is harness engineering pushed up the Meadows ladder from reliability (rungs 5–8) to goals (rung 3). It's the same move the field is making, aimed one level higher. **That's the contribution.**

---

## The build decision (reuse to cut load — honestly)

Not LangChain *or* Rust. Both, at different layers, divided by the trust boundary.

| Layer | Decision | Tool | Why |
|---|---|---|---|
| Off-device eval / orchestration / trace | **REUSE** | LangGraph / deepagents + tracing (or our golden-run) | LangGraph is a durable state-machine runtime — it *is* our FSM, off-device; Trivedy's trace-analysis-as-skill is our harness-improvement loop. Don't reinvent the loop or the tracing. |
| On-device hot path: rules executor + verifier gate | **BUILD** | one **Rust** core → CLI (laptop) + UniFFI→Swift (iOS) | native-portable, fast, memory-safe, *small and auditable*; Python/LangChain can't run on iOS, and the gate is the legal control — it must be owned, not a dependency. |
| Inference | **REUSE** | Bonsai mlx-swift (device) / llama.cpp (laptop) | existing 1-bit runtimes; never reinvent inference. |
| Human-in-the-loop escalation | **ADOPT PATTERN** | Horthy/HumanLayer contact-human-as-tool-call | our scope-boundary escalation is exactly this. |
| Recognizer authoring | **REUSE** | Presidio (off-device) | already decided; portable rule export. |
| The trust path & device runtime | **NEVER a framework** | — | auditability + the iOS constraint; the gate stays thin, owned, Rust. |

**The line:** reuse rich frameworks where it's safe (off-device orchestration and eval); own a thin native core where it's critical (the executor + gate). Trivedy's own tension makes the call: you cannot put a heavy "let-the-model-cook" framework on a PHI trust path and still be auditable — and it won't run on the phone regardless. This is the authoring/runtime split (Constitution IV) again: LangChain authors and evaluates off-device; Rust runs the hot path everywhere.

---

## The strategic kicker

Trivedy bets the best **environments** produce the best agents, and that efficient **verifiers** make RL cheap. Our de-identified trajectory store is a domain-specific environment, and the verifier gate is the verifier — **in a domain (mental-health coaching) nobody else can easily build an environment for, because of privacy.** The on-device de-identification is what makes the environment buildable at all. So the privacy architecture isn't a tax on the harness — it's what unlocks a *defensible* environment no competitor can replicate. The constraint is the moat.

---

## One-line test
> By the field's own definitions we're doing harness engineering on its hardest branch — deterministic and auditable — and extending it from reliability to ethics; so we reuse LangGraph and Bonsai's runtimes to cut load off-device and at inference, and we build one small Rust core for the executor and gate, because the trust anchor is the one thing we must own.
