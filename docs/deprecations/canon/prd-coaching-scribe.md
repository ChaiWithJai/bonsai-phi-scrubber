# PRD — On-Device Coaching Scribe (v1 demo → learning loop)
*Product requirements. Companion to RFC-001 (engineering). End state: note reaches Slack · a great follow-up note · a growing RL environment.*

---

## One-liner
A coaching scribe that de-identifies the session on the device, delivers a structured record and a genuinely useful follow-up, and gets better at follow-up over time by growing a reinforcement-learning environment of de-identified trajectories — without raw client data ever leaving the phone.

## Problem & why now
Coaches drown in post-session admin and follow-through is where coaching actually works or fails — yet the field's tools compete on note generation and ship a cloud-plus-deletion privacy model. Coaches aren't HIPAA covered entities, so they're governed by consumer-health-data law (MHMDA-class) with a private right of action and almost no margin for a leak. The opening: own the follow-up loop, keep the data on-device, and make the system compound in value with use — which nobody is doing.

## Users & jobs
- **Coach (primary user).** "Capture what matters and follow up well, without becoming a data clerk or a privacy liability."
- **Client (beneficiary).** "Help me do what I committed to between sessions — and eventually not need the help."
- **Practice / org (buyer).** "Offer privacy-grade coaching tooling I can trust and extend, without taking on cloud-health-data liability."

## The product — one loop
```
session → capture (consented, offline) → scrub (on-device) → verifier gate
       → structured record → Slack (coach) 
       → follow-up note (client, client-paced)
       → outcome (did they act? more autonomous?) 
       → de-identified trajectory → RL environment grows → better next follow-up
```

## Definition of done — the three end-state outcomes
1. **The note reaches Slack.** A de-identified, structured coaching record posts to the coach channel as Block Kit. Raw name/disclosures never left the device.
2. **A great follow-up note.** Between sessions, the client receives a genuinely useful, specific, client-paced follow-up tied to their own commitment — not a generic reminder.
3. **The RL environment grows.** Each completed loop deposits one de-identified trajectory — (situation, nudge, outcome, autonomy-delta) — into the environment. The trajectory count grows; the follow-up policy has more to learn from. This is the compounding-value mechanism.

## The RL environment as a product mechanism
- **State:** the client's de-identified situation and open commitments.
- **Action:** the follow-up the system/coach chooses (timing, framing, intensity).
- **Reward:** movement toward the client doing it themselves — the **autonomy delta**. Explicitly **not** engagement, session count, or app opens.
- **What grows:** only de-identified trajectories and the reward signal. Raw content never enters the environment; the verifier gate guards environment ingress exactly as it guards Slack egress.
- The trajectory store and the eval golden-note corpus are the same growing asset.

## Success metrics
- **No-leak (gate):** zero raw identifiers reach Slack or the environment. Hard gate.
- **Follow-through rate:** commitments acted on between sessions.
- **Autonomy metric (north star):** per-client reliance *declines* over time as commitments internalize. Rising per-client dependence is a failure even if engagement rises.
- **Environment growth:** de-identified trajectories accumulated; coverage across situation types.
- **Adoption:** coach still using it past week two.

**Anti-metric:** engagement / time-in-app / nudge volume. We do not optimize these. Optimizing them is the failure mode (the dependence trap).

## Scope
**In (v1 demo):** one phone, one coach channel; the full single loop; consent capture; the verifier gate guarding both boundaries; one real follow-up note; the trajectory counter incrementing; the five-file pack reveal.
**Out (v1):** policy *training* (we grow the environment, we don't yet train a policy in the demo); fleet/registry/control plane; multiple sinks; clinical features; second adopter.

## Risks
- **Reward misdesign → dependence engine.** Highest risk. Mitigated by the autonomy reward and the anti-metric stance; revisited in RFC open questions.
- **Trajectory re-identification.** Even de-identified trajectories can leak via linkage; mitigation (aggregation / minimum cohort) is an RFC open question.
- **Scope creep into clinical.** Hard boundary: on any clinical-risk signal, surface human escalation; never nudge through it.
- **Demo over-claims RL.** We show the environment *growing*, not an agent *trained*. Say so plainly.

## Non-goals
Replacing the coach. Maximizing engagement. Diagnosis or therapy. Cloud accumulation of client data.
