# The Floor
### Evaluation · Ethics & Law · Stack selection · JTBD · Adoption checklist
*The layer beneath the architecture. What the system design assumed but never made explicit.*
*Note: the legal section is a design framework, not legal advice — confirm with counsel before production.*

---

## Part 1 — Evaluation framework

Most teams evaluate only the model. This system needs two levels, and a gate.

### A. Correctness eval — does the scrubber work?
Per-entity **recall** and **precision** against the golden synthetic-PHI notes, with recall prioritized (a false positive over-redacts; a false negative leaks).
- **Leakage rate** — the rate at which any identifier survives to the sink. Target: zero. This is the one metric that gates egress.
- **Verifier-gate pass rate** — how often the gate has to block. A rising block rate is an early signal the rule pack or model drifted.
- **Recall threshold** per policy (e.g. ≥0.99). A pack that drops below does not ship.

### B. Outcome eval — does the *system* work?
The follow-up loop is the product, so measure the loop, not the note (Meadows trap: "seeking the wrong goal").
- **Follow-through rate** — commitments acted on between sessions.
- **Autonomy metric** — the anti-dependence measure: do commitments graduate to "internalized" and exit nudging? Healthy usage should *decline per client over time*. Rising per-client dependence is a failure even if engagement looks good.
- **Adoption/retention** — did the coach keep using it past week two? (Field data: ~15% of clinicians never adopt scribes even with support; structure week one or it won't stick.)

### C. Eval as gate, not report
The harness runs in CI. Nothing — core change, model bump, or ecosystem pack — promotes without passing. This is the through-line from the constitution ("verification is the feature") to certification: re-cert is a button, not a project.

---

## Part 2 — Ethical & legal framework

### Ethical (carried from the constitution and systems work)
- **Privacy as substrate, not feature.** Honest disclosure is the precondition of coaching; privacy is what makes disclosure safe. It is the product's ground, not a setting.
- **Design to recede (anti-dependence).** The "shifting the burden to the intervenor" trap is the central ethical risk of any follow-up system. Success is measured partly by clients no longer needing it. Build toward autonomy, not retention.
- **Scope boundary: coach ≠ therapist.** Stay in the coaching frame (goals, commitments, behavior). On any signal of clinical risk, surface human/professional escalation — never nudge through it or simulate care.
- **The client controls the interrupt.** Follow-up cadence is the client's to set. Over-nudging provokes disengagement (the "fixes that fail" trap).

### Legal (the binding reality)
The governing fact: **a mental-health coach is generally not a HIPAA covered entity** — which is a trap, not a relief, because the consumer-health-data laws were written precisely for that gap.

- **The binding constraint is MHMDA-class law** (Washington's My Health My Data Act, RCW 19.373, and its spreading cousins). It covers consumer health data held by non-HIPAA entities, defines that data sweepingly to include **mental health status, behavioral/psychological interventions, and ML-inferred or extrapolated data**, and is **consent-driven**. It carries a **private right of action** (treble damages, capped ~$25K, plus per se Consumer Protection Act liability), a **sweeping deletion right** with almost no exceptions, and **no company-size floor**.
- **The architecture is the compliance strategy.** Keeping raw input on the consumer's own device, shipping only de-identified records, and never selling or sharing **minimizes regulated-entity exposure by construction**. Where cloud-plus-BAA incumbents are full regulated entities holding consumer health data, this design largely sidesteps the highest-risk provisions (sale authorization, third-party disclosure lists, server-side deletion at scale).
- **The verifier gate is a legal control.** Because inferred data is covered, the structured note is still consumer health data unless de-identification is *real* (Safe-Harbor-grade). The gate that blocks egress on residual identifiers is the technical enforcement of a legal requirement.
- **Recording consent is separate and additional.** All-party-consent states (Washington, California, and others) require the client's consent to record the session regardless of MHMDA. Bake consent capture into the flow.
- **Design to the highest standard.** HIPAA is the floor, not the ceiling. Even where neither HIPAA nor a given state law strictly applies, build to HIPAA-grade controls (encryption in transit and at rest, access control, audit logging, incident plan) — MHMDA requires a "reasonable standard of care."

**One-line compliance posture:** collect less, keep it local, de-identify before egress, never sell, make deletion trivial because there's almost nothing held to delete.

---

## Part 3 — Stack selection (decision · alternatives · rationale)

Each choice follows from the framework above, not from preference.

| Layer | Choice | Alternatives considered | Why this one |
|---|---|---|---|
| Inference model | **Bonsai 1.7B** (1-bit, on-device) | Cloud LLM; Bonsai 4B/8B; Phi/Gemma small | Fits iPhone 11; runs offline; 1-bit edge-native. Cloud rejected (trust boundary + MHMDA exposure). 8B rejected (memory). |
| Inference runtime | **mlx-swift / Locally AI** | llama.cpp (desktop/CUDA); Core ML (conversion burden) | Native Apple Silicon path; on-device on A-series. |
| Rules executor | **Native Swift regex engine** | Presidio on-device; Philter (both Python) | Python won't run on iOS. Executes portable rule packs; fast, auditable. |
| Recognizer authoring | **Presidio (Python, off-device)** | Roll-your-own | Battle-tested, pluggable, exports portable definitions. The ecosystem's authoring SDK. |
| Rule lineage | **Philter-derived + Safe Harbor 18** | NLM Scrubber; PhysioNet; MITRE MIST | Highest clinical recall; certification precedent. MIST rejected (training burden). |
| Verifier gate | **Native re-scan + egress block** | Trust-the-model | The legal + technical control; default-deny made real. |
| Secure store | **iOS Secure Enclave / Keychain** | App sandbox file | Raw input + redaction map never leave hardware. |
| Structurer | **Bonsai (plan→execute) + deterministic schema validation** | Model-only | Schema validation is code, not AI (negative space). |
| Sink | **Slack Block Kit (reference) + pluggable** | Direct EHR/FHIR first | Demo velocity + field/block resonance; sinks are swappable. |
| Distribution | **Signed OCI artifacts (cosign/SLSA) + manifest pull** | App-store-only; MDM-only | Ecosystem extensibility; GitOps for clinics. |
| Control plane | **PHI-free backend (containers; k8s only if scale)** | Nomad/k8s on data path | It moves versions, signatures, de-id metrics — never PHI. Schedulers on the data path rejected. |
| Extension unit | **Declarative coach pack** | Fork-per-clinic | Extend without forking; the self-organization leverage point; certification-safe. |

**The stack in one breath:** an on-device 1-bit model and a native rules executor scrub at the edge; a verifier gate enforces the boundary; the structurer and follow-up engine run locally; only de-identified records reach a pluggable sink; everything ecosystem-facing is a signed, declarative pack gated by the eval harness.

---

## Part 4 — The Jobs To Be Done the stack powers

Three jobs, three buyers, mapped to the layers that serve each.

**The coach's job (functional + emotional).**
*"When I finish a session, help me capture what matters and follow up well — without becoming a data-entry clerk or a privacy liability."*
→ Powered by: capture → scrub → structure → follow-up engine. The emotional payoff is relief from documentation burden *and* from liability fear.

**The client's job (the deep one).**
*"Help me actually do what I committed to between sessions — and eventually not need the help."*
→ Powered by: the follow-up loop + the autonomy exit. The job is progress that becomes self-sustaining; the stack's success is its own obsolescence per client.

**The adopter's job (the org / system buyer).**
*"Let me offer privacy-grade coaching tooling I can trust and extend, without taking on cloud-health-data liability."*
→ Powered by: on-device architecture (the MHMDA-minimizing design) + the coach pack (extend without forking) + the eval harness (trust a pack you didn't write).

The through-line: each job is served by the same property — intelligence stays where the data is — seen from a different seat.

---

## Part 5 — Adoption stages checklist

Staged on the CNCF radar (Assess → Trial → Adopt), with each stage gated on specific eval and legal artifacts.

**Stage 0 — Demo / Awareness**
- [ ] Two-beat "Airplane Mode" demo runs (real scrub, hard case caught, five-file reveal)
- [ ] Exit signal: prospect asks "did it really never leave the phone / catch the hard one / could we write our own pack?"

**Stage 1 — Assess**
- [ ] Separate consumer-health-data privacy policy drafted (MHMDA requires a distinct, homepage-linked policy)
- [ ] Eval results published (recall/precision, leakage rate)
- [ ] Threat model + scope-boundary (coach≠therapist) docs available
- [ ] Reference coach pack available to inspect
- [ ] Exit: prospect agrees to a bounded trial

**Stage 2 — Trial**
- [ ] Recording-consent flow in place (all-party-consent compliant)
- [ ] On-device operation independently verified (airplane-mode loop)
- [ ] Audit log running; escalation path tested
- [ ] Recall gate passed on the trial coach's own note shapes
- [ ] Exit: outcome signal (follow-through) + zero leak

**Stage 3 — Adopt**
- [ ] Their coach pack authored and eval-gated
- [ ] Deletion flow + MHMDA rights-request handling (45-day response)
- [ ] Incident-response plan; HIPAA-grade security controls in place
- [ ] A second adopter exists (kills the bus-factor objection)
- [ ] Certification path defined if the use creeps clinical-adjacent
- [ ] Exit: rolled out to their coaches

**Stage 4 — Extend / Ecosystem**
- [ ] Adopter authors and/or publishes a pack to the registry
- [ ] Conformance tests pass (PHI-blind, verifier-clean)
- [ ] Contributes recognizers/templates back
- [ ] Exit: the ecosystem evolves the system without you

---

## One-line floor test
> The system is ready when the eval harness gates every release, the architecture means there is almost no regulated data to govern, the stack choice traces to a requirement rather than a preference, every layer serves one of three jobs, and an adopter can move demo → adopt against a checklist instead of a sales call.
