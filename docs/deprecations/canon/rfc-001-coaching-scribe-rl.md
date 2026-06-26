# RFC-001 — On-Device Coaching Scribe with a Growing RL Environment
*Status: Draft / for review. Converts Engineering Spec v1 into proposal form and extends it with the learning loop.*
*Companion: PRD (product). Builds on ADR-001…010 in Engineering Spec v1.*

---

## Summary
Add a follow-up loop and a reinforcement-learning *environment* to the on-device coaching scribe such that the system improves at follow-up over time — while preserving the invariant that raw client data never leaves the device. The environment grows on **de-identified trajectories** with a reward aimed at **client autonomy**. The verifier gate, already guarding network egress, is extended to also guard environment ingress.

## Motivation
The scrubber demo proves safety but is static. For adoption and for the broader learning thesis (cf. the Ember Witness/Facilitator loop), the system must compound in value. The naive way to do that — accumulate session data centrally — violates the trust boundary and the consumer-health-data (MHMDA-class) posture. This RFC resolves the apparent collision: a system can learn from outcomes without centralizing raw data, by reducing each loop to a de-identified trajectory before it is ever stored or learned from.

## Guide-level design

### The loop
`session → capture (consented, offline) → scrub → verifier gate → structured record → Slack; → follow-up note (client-paced) → outcome → de-identified trajectory → RL environment`

### The RL environment
A formal environment over the follow-up decision:
- **State** `s`: de-identified client situation + open commitments (themes, commitment type, progress band, time-since-session). No names, no narrative.
- **Action** `a`: the follow-up choice (timing, framing, intensity, or none).
- **Reward** `r`: the **autonomy delta** — change in the client's demonstrated capacity to act without prompting. Positive for internalization; **negative for induced dependence**.
- **Transition**: next session/outcome updates the state.
- **Trajectory** `(s, a, r, s')`: the only thing persisted. De-identified by construction.

"Growing the environment" in v1 = accumulating trajectories and the reward signal — the experience buffer a policy could later be trained or evaluated against. **Policy optimization is explicitly out of v1 scope.**

### The dual-boundary gate
The verifier gate now guards two egress points with the same rule (default-deny on residual identifiers):
1. **→ Sink** (Slack / coach): already specified.
2. **→ RL environment**: a trajectory may be persisted only if it passes the gate. The trajectory store inherits the trust boundary.

The trajectory store and the eval golden-note corpus are unified: every accepted trajectory is also an eval example.

## Reference to existing decisions
ADR-001 (on-device scrub), 002 (no data-path orchestrator), 003 (authoring/runtime split), 004 (three-layer scrubber + gate), 005 (signed core + packs), 006 (data/control plane split + CNCF reuse), 007 (Bonsai 1.7B), 008 (coach market), 009 (follow-up loop + anti-dependence), 010 (Slack reference sink) stand unchanged. This RFC adds:

### ADR-011 — The RL environment grows on de-identified trajectories only
**Decision:** Persist and learn from `(s,a,r,s')` tuples that contain no raw identifiers; the verifier gate guards environment ingress.
**Rejected:** Storing raw transcripts/notes for richer learning; cloud trajectory aggregation of raw data.
**Consequence:** Learning is compatible with the trust boundary and MHMDA posture; some signal is sacrificed for safety, accepted.

### ADR-012 — Reward targets client autonomy, not engagement
**Decision:** The reward is the autonomy delta; engagement/usage are anti-metrics, never reward terms.
**Rejected:** Engagement-, retention-, or session-count-based reward.
**Consequence:** Structurally prevents the system from learning a dependence policy (the "shifting the burden" trap). This is the single highest-stakes design choice.

### ADR-013 — Policy optimization deferred; v1 grows the environment only
**Decision:** v1 accumulates trajectories + reward signal; no policy is trained or shipped in the demo.
**Rejected:** Training/serving an RL policy in v1.
**Consequence:** Avoids drawing the owl; keeps the demo honest ("environment growing," not "agent trained"); defers the hardest safety questions to a gated later milestone.

## Alternatives considered
- **Cloud RLHF on session data** — rejected: violates the trust boundary; makes us a full regulated entity.
- **No learning loop (static product)** — rejected: no compounding value; weak against the adoption and Ember theses.
- **Learn from raw on-device, never persist** — viable but thin: a single device's trajectories are too sparse to learn a good policy; the de-identified-trajectory approach allows safe aggregation later.

## Open questions
1. **Reward measurement.** How is the autonomy delta observed without intrusive instrumentation? Self-report? Commitment-completion trend? Coach rating? Needs a concrete, low-burden proxy.
2. **Trajectory re-identification.** Even de-identified `(s,a,r,s')` can leak via linkage. Do we need minimum-cohort aggregation, k-anonymity, or differential privacy before any off-device pooling?
3. **Where does the policy improve?** On-device personal fine-tune, federated learning across devices, or central training on pooled de-identified trajectories? Each has a different privacy and ops profile.
4. **Cold start.** With zero trajectories, follow-up is heuristic/coach-authored. What is the bootstrap policy?
5. **Cohort vs. individual.** Does the autonomy reward optimize per-client or across clients (risking "success to the successful" attention skew)?
6. **Gate semantics for trajectories.** Is the structured state already gate-clean, or does the abstraction itself need a residual-linkage check distinct from identifier scrubbing?

## Rollout / milestones
- **M0 (demo):** one full loop on one phone; note→Slack; one real follow-up; trajectory counter increments through the gate. (This RFC's shippable target.)
- **M1:** trajectory store + eval-corpus unification; autonomy-proxy v1; cohort coverage report.
- **M2 (gated):** offline policy evaluation against the grown environment; reward-design review before any policy is *served*.
- **M3 (gated):** federated/aggregation decision (open question 3) with a privacy review.

## Drawbacks / risks
- Reward misdesign is catastrophic, not cosmetic (ADR-012 mitigates but does not eliminate — open question 1).
- De-identified trajectories may still carry re-identification risk (open question 2).
- The honest framing ("growing the environment," not "trained agent") must hold against the temptation to over-claim RL in the demo.

## One-line proposal test
> Each loop deposits exactly one gate-clean trajectory with an autonomy-aimed reward; the environment grows; the trust boundary holds at both egress points; and nothing about learning required raw client data to leave the device.
