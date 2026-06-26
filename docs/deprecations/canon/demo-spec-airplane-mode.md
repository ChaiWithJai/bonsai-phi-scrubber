# Demo Specification — "Airplane Mode"
### On-device PHI scrub → clean record to Slack
*v1. One phone. One channel. Two beats. Everything else deferred.*

---

## Goal

Make a healthcare end user believe three things in under five minutes:
1. Patient data never leaves the device.
2. The system catches PHI — including a hard case — and blocks anything unclean from being sent.
3. Any clinic can make it their own by editing five files, never touching the core.

The demo *is* the verifier gate and the trust boundary, made watchable.

---

## Thesis (one line)

> Talk to a 2019 phone in a patient's home, with the radios off. It cleans the note on the device. Only a de-identified, structured care record ever reaches Slack — and the real name never leaves the phone.

---

## Preconditions / setup

**Hardware**
- One iPhone 11 (the age is the point — show it on the title card).
- Presenter screen mirrors the phone (so the room sees the airplane icon and the scrub happen live).

**Software on device**
- Bonsai 1.7B running locally (Locally AI or mlx-swift build).
- The Core Runtime: pipeline runner, native rules executor, Bonsai scrub, **verifier gate + egress control**, enclave redaction-map handler.
- The `cityblock-chp` clinic pack loaded (recognizers, schema, policy, Slack sink).

**Off device**
- One Slack workspace + one care-team channel, Block Kit sink configured (bot token sourced from device keychain).

**Data**
- A scripted visit note containing **synthetic** PHI only — including one deliberately hard identifier (a name embedded in prose, or a date written in words).

---

## The run

Total ~4 minutes. Title card first: *"iPhone 11. 2019. Airplane mode."*

### Beat 1 — The airplane-mode loop  (~2 min) — the no-leak proof

| # | Presenter action | On screen | The line |
|---|---|---|---|
| 1 | Toggle **airplane mode ON**, hold the phone up | Airplane icon visible; radios off | "Radios are off. This phone physically cannot transmit." |
| 2 | Dictate the visit note (synthetic PHI, incl. the hard case) | Raw transcript appears | "A nurse just captured a note in someone's home." |
| 3 | Tap **Scrub** | Redactions render live; the hard-case identifier gets caught and highlighted | "It's cleaning on the device. Watch it catch the name buried in that sentence." |
| 4 | Show the structured card forming | Typed care record (member_pseudonym, visit_type, sdoh_flags, follow_ups, risk_signals, next_touch) | "The note became a structured record. Still offline." |
| 5 | Tap **Send** — *still in airplane mode* | Sink **blocks**: "no network" → queued | "It can't send. Good. Nothing leaves while the radios are off." |
| 6 | Toggle **airplane mode OFF** | Queue flushes; one post lands in Slack | "Now — and only now — the clean record posts." |
| 7 | Open Slack on the big screen | Block Kit card in the channel; **no name, no number** | "The real name never left the phone. Only this did." |

The conversion moment is steps 5–7: they watched it refuse to send, then send only the clean thing.

### Beat 2 — The five-file reveal  (~2 min) — the extensibility proof

| # | Presenter action | On screen | The line |
|---|---|---|---|
| 1 | Open the `cityblock-chp` pack | Five files: recognizers / schema / policy / sink / eval | "This clinic is just these five files. No code." |
| 2 | Add one recognizer (a new clinic's member-ID format) | One line of declarative config | "A new clinic adds their own ID format here." |
| 3 | Run the **eval harness** | Recall gate: PASS | "The pack has to prove it doesn't lower recall before it ships." |
| 4 | Re-scrub a note containing that new ID | The new ID is now caught | "Same core. Their identifiers. They never forked anything." |

Close: "Cleaned on the phone, gated before it sends, and yours in five files. That's the whole thing."

---

## What must be built for v1 (minimal)

- On-device Bonsai 1.7B inference.
- Native rules executor that loads the pack's recognizers.
- Verifier gate that re-scans scrubbed output and **blocks egress** on any residual hit.
- Enclave write for the redaction map.
- Structurer → the home-visit schema.
- Slack Block Kit sink + offline queue.
- The `cityblock-chp` pack, including ~20 hand-labeled golden eval notes.
- Eval harness CLI (recall/precision against the golden set).

That is the entire build. Nothing here is optional; nothing else is in scope.

---

## Success criteria

The end user, unprompted, says some version of:
- "So the patient's data genuinely never went anywhere?" — yes, they watched it.
- "And it caught the one in the sentence?" — yes, the hard case.
- "We could just write our own pack?" — yes, five files, no fork.

If they ask those three questions, the demo converted.

---

## Hard rules (non-negotiable)

1. **No Wizard-of-Oz.** Real model, real scrub, real airplane mode. The gate must actually block — faking it makes it a chatbot pretending to be an agent.
2. **Airplane mode is the proof, not a software meter.** Radios off is tamper-evident to the room; it cannot transmit, full stop.
3. **Show one hard case the gate catches.** A too-clean demo reads as cherry-picked. One visible near-miss caught is more convincing than ten easy hits.
4. **Synthetic PHI only.** Never a real patient note, on stage or in the eval set.
5. **Title card shows the phone's age.** The 2019 device carries the democratization argument.

---

## Out of scope for v1 (defer, explicitly)

Fleet, Mac minis, registry/OCI distribution, control plane, MDM, multiple sinks, certification, second live adopter. None of it ships for the demo. One phone, one channel, two beats — that is the converting unit.
