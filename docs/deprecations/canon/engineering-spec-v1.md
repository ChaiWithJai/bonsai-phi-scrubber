# Engineering Specification v1 — On-Device Coaching Scribe
### Refined demo spec · Stack selection · Architectural decisions (ADRs)
*Supersedes the scattered versions. Reconciled to the mental-health-coach domain and the consumer-health-data (MHMDA-class) legal floor.*

---

# Section 1 — Demo Specification (refined)

## Goal
Make a coach — or an org that equips coaches — believe three things in under five minutes:
1. The client's words never leave the device.
2. The system catches identifying detail, including a hard case, and blocks anything unclean from being sent.
3. Any practice can make it their own by editing five files, never touching the core.

The demo is the verifier gate and the trust boundary, made watchable. It is also the live proof of the legal posture: there is almost no regulated data to govern.

## Thesis (one line)
> Run a coaching session with a 2019 phone in airplane mode. It de-identifies on the device. Only a structured, de-identified record ever reaches the coach's workspace — the client's name and disclosures never leave the phone.

## Preconditions
- One iPhone 11; screen mirrored to the room.
- Bonsai 1.7B on-device (mlx-swift / Locally AI).
- Core Runtime loaded: pipeline runner, native rules executor, Bonsai scrub, verifier gate + egress control, enclave handler.
- `coach-session` pack loaded (recognizers, schema, policy, Slack sink, eval notes).
- One Slack workspace + coach channel.
- **Consent captured on-device** (all-party-consent compliant) before capture — shown as the first interaction.
- Script uses **synthetic** client content only, including one hard identifier (a name in prose, or a disclosed place/relationship).

## The run (~4 min). Title card: *"iPhone 11. 2019. Airplane mode. One session."*

### Beat 1 — The airplane-mode loop (the no-leak proof)
| # | Action | On screen | Line |
|---|---|---|---|
| 1 | Capture consent, then toggle **airplane mode ON** | Consent confirmed; airplane icon | "Consented and offline. The phone physically can't transmit." |
| 2 | Dictate the session note (synthetic, incl. hard case) | Raw transcript | "A coach just captured what happened in the room." |
| 3 | Tap **Scrub** | Redactions render live; the hard-case identifier is caught and highlighted | "Cleaning on the device. Watch it catch the name buried in that sentence." |
| 4 | Show the record form | Coaching record: themes, commitments, follow-ups, risk-flags | "It became a structured record. Still offline." |
| 5 | Tap **Send** — still in airplane mode | Sink **blocks** → queued | "It won't send. Good. Nothing leaves while offline." |
| 6 | Toggle **airplane mode OFF** | Queue flushes; one post lands | "Now — only now — the clean record posts." |
| 7 | Open Slack on the big screen | Block Kit card; no name, no disclosure | "The client's name never left the phone. Only this did." |

### Beat 2 — The five-file reveal (the extensibility proof)
| # | Action | On screen | Line |
|---|---|---|---|
| 1 | Open the `coach-session` pack | Five files | "This practice is just these five files. No code." |
| 2 | Add one recognizer (a new practice's ID format) | One line of config | "A new practice adds their identifiers here." |
| 3 | Run the **eval harness** | Recall gate: PASS | "It has to prove it doesn't lower recall before it ships." |
| 4 | Re-scrub a note with the new ID | Now caught | "Same core, their identifiers, no fork." |

Close: "Cleaned on the phone, gated before it sends, yours in five files — and almost nothing for the law to reach, because the regulated data never left the device."

### Beat 3 — deferred
The follow-up loop (commitment → client-paced nudge → autonomy exit) is the next demo surface, built only after the no-leak loop converts. It is where the anti-dependence design must be made visible.

## Hard rules
1. No Wizard-of-Oz. Real model, real scrub, real airplane mode; the gate must actually block.
2. Airplane mode is the proof, not a software meter.
3. Show one hard case the gate catches.
4. Synthetic content only. Consent shown, not skipped.
5. Title card shows the phone's age (the democratization argument).

## Success criteria
The viewer, unprompted, asks: "It really never left the phone?" / "It caught the one in the sentence?" / "We could write our own pack?" Three questions = converted. (Maps to CNCF Assess → Trial.)

## Out of scope for v1
Fleet, Mac minis, registry, control plane, MDM, multiple sinks, certification, second live adopter, the follow-up loop. One phone, one channel, two beats.

---

# Section 2 — Stack Selection (refined, with traceability)

Every choice traces to a requirement, not a preference. Two planes: **data plane** (on-device, holds regulated data) and **control plane** (PHI-free backend, reuses CNCF primitives).

## Data plane (on-device)
| Layer | Choice | Rejected | Requirement it serves |
|---|---|---|---|
| Inference model | **Bonsai 1.7B** (1-bit) | Cloud LLM; Bonsai 4B/8B | Runs offline on iPhone-class hardware; minimizes regulated-entity exposure |
| Runtime | **mlx-swift / Locally AI** | llama.cpp; Core ML | Native on-device on A-series |
| Rules executor | **Native Swift regex engine** | Presidio/Philter on-device (Python) | Python can't run on iOS; portable, auditable rule packs |
| Rule lineage | **Philter-derived + Safe Harbor 18** | NLM Scrubber; MITRE MIST | Highest recall; certification precedent |
| Verifier gate | **Native re-scan + egress block** | Trust-the-model | Legal control: de-identify before egress (MHMDA) |
| Secure store | **Secure Enclave / Keychain** | App sandbox file | Raw input + redaction map never leave hardware |
| Structurer | **Bonsai (plan→execute) + deterministic schema validation** | Model-only | Schema validation is code, not AI |
| Sink (reference) | **Slack Block Kit**, pluggable | Direct EHR/FHIR first | Demo velocity; sinks are swappable |

## Control plane (PHI-free backend — reuses CNCF supply-chain stack)
| Concern | Choice | CNCF lineage | Requirement it serves |
|---|---|---|---|
| Pack/model distribution | **Signed OCI artifacts** | cosign / Sigstore / Rekor / SLSA / in-toto | Tamper-evident, auditable artifact supply chain |
| Eval / conformance gate | **Policy-as-code on pack admission** | OPA (Rego) / Kyverno-JSON | No pack promotes without passing recall gate |
| Control mapping | **MHMDA + Safe-Harbor expressed in OSCAL** | NIST OSCAL / CNCF CCF pattern | Compliance posture is machine-checkable |
| Adopter posture | **Self-assessment artifact** | CNCF TSSA self-assessment format | Reviewable trust artifact for adoption |
| Orchestration | **Containers; scheduler only if scale demands** | (not k8s/Nomad on data path) | Control plane carries no PHI; tool choice is free |

**Stack in one breath:** an on-device 1-bit model and native rules executor scrub at the edge; a verifier gate enforces the legal boundary; only de-identified records reach a pluggable sink; everything ecosystem-facing is a signed, declarative pack gated by policy-as-code — the data plane is novel (edge-native enforcement), the control plane is standard CNCF supply chain.

---

# Section 3 — Architectural Decisions (ADRs)

Lightweight records. Status, context, decision, alternatives rejected, consequences.

### ADR-001 — Scrub on-device, never in the cloud
**Status:** Accepted.
**Context:** A coach is not a HIPAA covered entity, but MHMDA-class law governs consumer health data — including ML-inferred data — with a private right of action. Cloud inference makes us a regulated entity holding that data.
**Decision:** All de-identification runs on-device; raw input never crosses the network.
**Rejected:** Cloud LLM + BAA (the incumbent model); hybrid cloud-fallback.
**Consequences:** Minimizes regulated-entity exposure by construction; constrains model size to edge-capable; becomes the core differentiator and the legal posture simultaneously.

### ADR-002 — No orchestrator on the data path
**Status:** Accepted.
**Context:** Schedulers move work to machines. Here the work and data are permanently co-located on a phone; nothing is scheduled.
**Decision:** The data plane is the app's own pipeline runner plus an offline retry queue. No k8s, no Nomad.
**Rejected:** Kubernetes; Nomad; a LAN relay where phones ship raw input to a shared box.
**Consequences:** Drastically simpler edge; the LAN never carries raw input; schedulers are confined to the (PHI-free) control plane if needed at all.

### ADR-003 — Authoring/runtime split for de-identification
**Status:** Accepted.
**Context:** The best de-id frameworks (Presidio, Philter) are Python, which can't run on iOS.
**Decision:** Author and test recognizers off-device in Presidio; compile them to portable rule packs (regex + context + checksum + lists) that a native executor runs on-device.
**Rejected:** Full Presidio/Philter stack on-device; rolling our own authoring tool.
**Consequences:** "Codify, don't fork" applied to de-id; the authoring layer is the ecosystem SDK, the runtime layer is the end-user binary.

### ADR-004 — Three-layer scrubber with a verifier-gate egress control
**Status:** Accepted.
**Context:** No single detector catches everything (Presidio's own warning); de-identification is also a legal requirement, not just a quality target.
**Decision:** Redact if any layer flags — rules executor ∪ Bonsai — then a verifier gate re-scans and blocks egress on any residual hit (default-deny). Recall-first.
**Rejected:** Single-model scrub; trust-the-model with no gate.
**Consequences:** The gate is simultaneously a technical control and the enforcement point for the MHMDA de-identification exemption.

### ADR-005 — One signed core + declarative packs (no fork-per-tenant)
**Status:** Accepted.
**Context:** The correctness/legal-critical code must be certified once; every practice needs different identifiers, schemas, and sinks.
**Decision:** Build and sign one immutable Core Runtime; each practice authors a declarative, PHI-blind pack against a fixed contract.
**Rejected:** Forking the core per practice.
**Consequences:** Certify once; let anyone extend safely; a pack can never touch PHI, the redaction map, or the verifier, so the trust boundary holds for strangers.

### ADR-006 — Data plane / control plane split; reuse CNCF only on the control plane
**Status:** Accepted.
**Context:** The CNCF compliance stack (policy-as-code, attestation, OSCAL, scanning) assumes the cluster is the unit of governance. Our regulated data is on a phone, not a cluster.
**Decision:** Reuse CNCF supply-chain primitives for the PHI-free control plane (signed packs, policy-as-code eval gate, OSCAL control mapping, TSSA self-assessment). Enforce data-plane compliance on-device via the verifier gate — a gap CNCF does not cover.
**Rejected:** Trying to govern the data plane with cluster-oriented CNCF tools.
**Consequences:** Strong, standard prior art for the control plane; the on-device enforcement is the genuine contribution and has no CNCF analog.

### ADR-007 — Bonsai 1.7B as the device model
**Status:** Accepted.
**Context:** iPhone 11 has a 4GB memory budget; iOS jetsam kills oversized apps.
**Decision:** Ship 1.7B as the workhorse (4B where the device allows); never 8B on iPhone-11-class hardware.
**Rejected:** Bonsai 8B on-device.
**Consequences:** Slightly lower raw capability, fully offset by the demo's old-phone democratization story; comfortable memory headroom.

### ADR-008 — Target coaches, not licensed therapists
**Status:** Accepted.
**Context:** The behavioral-health scribe field is crowded around licensed clinicians and competes on cloud + BAA. Coaches are underserved, handle intimate content, and lack a HIPAA backstop — so on-device privacy carries more weight, and MHMDA-class law is the live constraint.
**Decision:** Enter via the coaching market.
**Rejected:** Licensed-therapist market entry.
**Consequences:** Less crowded, stronger thesis fit; imposes a hard scope-boundary obligation (ADR-009 risk handling).

### ADR-009 — The follow-up loop is the product; design against dependence; bound the scope
**Status:** Accepted.
**Context:** Incumbents compete on note generation (low systems leverage). The leverage is the between-session loop. A follow-up system also risks the "shifting the burden to the intervenor" trap, and a coach must not act as a therapist.
**Decision:** Own the follow-up loop with an explicit autonomy exit (commitments graduate out of nudging); the client controls cadence; on any clinical-risk signal the system surfaces human escalation and never nudges through it.
**Rejected:** Note generation as the core value; engagement-maximizing follow-up.
**Consequences:** Differentiation at high leverage points; success is partly measured by clients no longer needing it; a non-negotiable escalation rule.

### ADR-010 — Slack Block Kit as the reference sink, pluggable by contract
**Status:** Accepted.
**Context:** The demo needs a fast, legible destination; adopters will want other destinations.
**Decision:** Ship a Slack Block Kit sink as the worked reference behind a sink-adapter interface; a sink receives only de-identified records.
**Rejected:** EHR/FHIR-first; Slack as the only sink.
**Consequences:** Demo velocity and field/block resonance; a sink can never see PHI, so new sinks are safe to contribute.

---

## One-line spec test
> The demo converts on the airplane-mode loop and the five-file reveal; every stack choice traces to a requirement; every architectural decision has a recorded context, alternative, and consequence — and across all three, the regulated data never leaves the device.
