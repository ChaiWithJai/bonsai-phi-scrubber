# Technical Survey: The PHI Scrubber
### End-user stack + ecosystem extension surface
*Research output. Pairs with "Build the Harness, Not the Demo." Applies that constitution to a buildable bill of materials.*

---

## 0. The decision the research forced

The constitution says the scrubber must run on-device (Article IX, default-deny trust boundary). The two best de-id libraries — Philter and Presidio — are both Python. **Python does not run natively on iOS.** Rather than fight that, we split the scrubber into two layers that live in two different places:

- **Authoring layer (off-device, Python):** where recognizers are *defined and tested* — Presidio's framework. This is where the ecosystem works.
- **Runtime executor (on-device, native Swift):** a lightweight engine that *executes* exported recognizer definitions (regex + context + checksum + deny/allow lists) plus the Bonsai model. This is what the end user ships.

The recognizer **definitions are portable config** (regex, word lists, checksums) — they cross the boundary; the Python runtime does not. This is "codify, don't fork" applied to de-identification: Presidio is the compiler, the on-device engine is the interpreter.

---

## 1. The de-identification landscape

| Tool | License | Technique | Clinical recall | Role in our stack |
|---|---|---|---|---|
| **Philter** (UCSF) | BSD-2 | Rule-based regex + include/exclude lists | 99.46% UCSF, 99.92% i2b2 | **Rule-pack source** for the on-device executor; the certified reference pipeline |
| **Presidio** (Microsoft) | MIT | NER + regex + checksum + context; fully pluggable | Varies (MedicalNER add-on) | **Authoring/extension framework** (off-device); ecosystem SDK |
| **NLM Scrubber** (NLM) | Free, proprietary | Rule-based, targets all 18 Safe Harbor identifiers | ~85–95% | Safe Harbor coverage reference; not extensible |
| **PhysioNet deid** (MIT/MIMIC) | GPL-2 | Rule-based + surrogate replacement | ~85% | Surrogate-generation reference |
| **MITRE MIST** | GPL | ML, requires labeled training data | Varies | Skip — training burden too high for ecosystem extenders |

**Read:** Philter gives us the highest-recall clinical rule set and the only real certification precedent. Presidio gives us the extension architecture. Neither alone is the answer; the rules come from Philter's lineage, the plugin model comes from Presidio, and the contextual layer comes from Bonsai.

---

## 2. The three-layer scrubber (the core architecture)

A token is redacted if **any** layer flags it. Recall is the priority; false positives (over-redaction) are an accepted cost, consistent with the privacy-centric posture clinical de-id already adopts. False negatives are unacceptable — a single leaked identifier fails the whole trust boundary.

```
raw_note
   │
   ├─► LAYER 1  Rules executor (native Swift, on-device)         [the harness — Article VI]
   │     regex + context + checksum + deny/allow lists,
   │     compiled from Philter/Presidio definitions.
   │     Catches structured identifiers: SSN, MRN, phone, fax,
   │     email, dates, ZIP, account/license/device numbers, URLs, IPs.
   │
   ├─► LAYER 2  Bonsai (on-device LLM)                            [the ambiguous layer — the AI stage]
   │     contextual / narrative PHI the rules miss: names in odd
   │     positions, relationships ("patient's daughter Maria"),
   │     free-text locations, paraphrased identifiers.
   │
   │   ── union of layer 1 + layer 2 → scrubbed_text + redaction_map ──
   │
   └─► LAYER 3  Verifier gate (native, on-device)                [verification is the feature — Article II]
         re-scan scrubbed_text against the full rule set.
         If ANY identifier pattern survives → BLOCK egress.      [default deny — Article IX]
         Only a clean pass advances to the structurer/sink.
```

The **redaction_map** (PHI → token table) is the radioactive artifact. It is written only to the device secure enclave / Keychain and **never crosses the network boundary** under any configuration. No sink, including Slack, ever receives it.

**Policy baseline — the HIPAA Safe Harbor 18.** The default config targets all 18: names; geographic units smaller than a state; all date elements except year (and ages over 89); phone; fax; email; SSN; medical record numbers; health-plan beneficiary numbers; account numbers; certificate/license numbers; vehicle identifiers; device identifiers/serials; URLs; IP addresses; biometric identifiers; full-face images; and any other unique identifying code. Config can switch the policy to Expert Determination, which changes which identifiers are redacted and whether reversible pseudonymization is allowed.

---

## 3. What the END USER needs (Cityblock) — bill of materials

Everything required to turn the demo into a workload an org runs for its own care teams. Organized by the four seams plus cross-cutting concerns.

### Per-seam

**Source (capture)**
- On-device ASR (commodity) for voice notes; text and form-photo capture paths.
- Confidence thresholds; low-confidence transcription flagged, not silently passed.

**Scrubber (de-id)** — the load-bearing component
- Bonsai 1.7B (iPhone 11 budget) via mlx-swift / Locally AI runtime.
- Native rules executor + compiled Philter/Presidio rule pack.
- Verifier gate.
- Secure-enclave store for the redaction map.

**Structurer (note → typed record)**
- Note-archetype classifier (Bonsai, plan-then-execute).
- Schema definitions per archetype (CHP home visit, clinical follow-up, …) as codified config.
- Deterministic schema validator (not AI — Article IV).

**Sink (Slack reference)**
- Slack app: bot token, OAuth scopes (`chat:write`, `channels:read`), Block Kit renderer.
- Channel-mapping config (care team → channel).
- Offline queue: if no network, scrub + structure locally and queue the sink for later.

### Cross-cutting (the part that separates a demo from a workload)

- **Trust-boundary enforcement** — egress is structurally gated on a clean verifier pass; not a runtime check that can be skipped.
- **Audit log** — append-only record of every scrub decision (what entity types were redacted, layer that caught each, verifier result). Auditable decisions are a stated de-id requirement, not a nice-to-have.
- **Config** — one declarative block (model, scrub policy, schema, sink) per the codify-don't-fork principle. Extending = edit config + drop in one interface impl.
- **Identity / consent** — operator auth on-device; member consent state gating capture.
- **Key management** — enclave keys for the redaction map; rotation policy.

### Operational reality (surface honestly)

- **Certification.** A demo does not need it; a production workload does. Philter's precedent: the only certified de-id is institution-and-version-specific, audited by a forensics firm, and re-certified every 2 years. Any change to the rule pack or model triggers re-validation. Build the eval harness (§4) so re-cert is a button, not a project.
- **The BAA punchline — and its asterisk.** Running inference on-device removes the need for a BAA with any cloud LLM provider: no third party ever processes raw PHI. *However*, data still lands in Slack. If the scrubbed record is certified de-identified to Safe Harbor, it is no longer PHI and Slack needs no BAA. If you use Expert Determination with reversible pseudonymization, or if quasi-identifiers remain in the structured record, put Slack on a HIPAA-eligible configuration (Enterprise Grid + BAA) as defense in depth. **The architecture's value proposition is exactly this:** scrub upstream to a certified-complete state, and every downstream surface — Slack, FHIR, CSV — is de-risked by construction.

---

## 4. What we GIVE the ECOSYSTEM extender — the SDK

The CNCF "ecosystem" persona: developers, integrators, and vendors who extend the scrubber for a new org without touching the core. The PHI scrubber's extension surface is concrete and battle-tested because it rides Presidio's recognizer model.

### The contract

A `Scrubber` extension implements/produces three things:

1. **Custom recognizers** — the primary extension point. Add org-specific identifiers (Cityblock member-ID formats, partner names, internal codes) as Presidio `PatternRecognizer`s: regex + context words + optional checksum validator, or a custom NER recognizer. Authored and tested in Python against Presidio; **exported as a portable rule pack** (JSON: patterns, context, deny/allow lists) the on-device executor consumes.
2. **A policy declaration** — which identifier set (Safe Harbor vs. Expert Determination), redaction vs. reversible pseudonymization, per-entity operators.
3. **Conformance with the invariant** — the extension never receives, logs, or transmits the raw text or redaction map outside the seam. Enforced by the interface, not by trust.

### What we ship them

- **Interface signatures** for all four seams (Source / Scrubber / Structurer / Sink), with the Scrubber's `raw_text → (scrubbed_text, redaction_map)` contract and the hard rule that the map never leaves the seam.
- **The reference scrubber** (Bonsai + rules executor + verifier) as the worked example to copy — the way a reference Terraform provider teaches the contract.
- **A recognizer authoring kit** — Presidio-based templates for regex/context/checksum recognizers, plus the exporter that compiles them to the on-device rule-pack format.
- **An eval harness** — a set of golden, synthetic-PHI notes and a scorer that reports recall/precision per entity type. Extenders must prove a new recognizer **does not lower recall** before it ships. This is the generalized Philter re-cert lesson: any scrubber change is gated on the eval harness. (Presidio ships a PII data generator and an evaluation repo we build on.)
- **Conformance tests** — automated checks that a contributed Source/Sink never touches PHI and that a Scrubber's output passes the verifier gate.
- **Policy config schema** — declarative Safe Harbor / Expert Determination selection.

### Versioning posture

Because de-id certification is version-locked, the rule pack and model are versioned artifacts. An ecosystem extension declares the scrubber version it targets; the eval harness result is the gate for promoting any new version. Forking the core to extend is a smell and a certification hazard.

---

## 5. Build sequence (next)

1. **Earn the map (Article III):** hand-scrub ~20 real-shaped CHP notes to derive the identifier taxonomy and confirm which Safe Harbor categories actually appear in field notes. This is the spec for both the rule pack and the schema.
2. **Slam-dunk to a coding agent:** the native rules executor (port a Philter-style regex set), the Slack Block Kit renderer, the schema validator, the rule-pack exporter.
3. **Own by hand:** the verifier-gate egress logic and the trust-boundary architecture.
4. **Stand up the eval harness early** — it is simultaneously the dev safety net, the ecosystem gate, and the path to eventual certification.

---

## One-line test
> The scrubber is done when a clean note flows end-to-end with the network off, the redaction map never leaves the enclave, and an ecosystem-contributed recognizer can raise recall on a new identifier without anyone forking the core or re-running anything but the eval harness.
