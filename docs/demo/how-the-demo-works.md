# How the Demo Works

This demo shows a coaching-session note moving from phone capture to Slack without
letting raw identifiers leave the edge node. It is intentionally small: one phone UI,
one local Rust web shell, one local model server, one declarative pack, one Slack sink,
and a verifier gate in front of every egress path.

For the diagram-only version, see
[`system-network-data-flows.md`](system-network-data-flows.md).

## The Short Version

1. The phone opens the local web UI at `http://<mac-lan-ip>:8099`.
2. The user dictates or types a synthetic coaching note.
3. `airplane-web` sends the note to `airplane-core` on the Mac.
4. `airplane-core` applies pack recognizers and asks local Bonsai for structured output.
5. The model output is parsed, clamped, and re-scanned by the verifier.
6. If residual identifiers remain, egress is blocked.
7. If the verifier sees zero residual identifiers, the clean care record can post to Slack.
8. Only after Slack accepts the clean post does the local trajectory store append a
   gate-clean `(s, a, r, s')` tuple.

The raw note and redaction map never go to Slack, logs, telemetry, or the trajectory
store.

## Why It Is Built This Way

The design is intentionally boring at the trust boundary:

- **Rust core owns the trust logic.** `airplane-core` owns rules, pack loading,
  verifier behavior, and pipeline semantics. That keeps the reviewed surface small.
- **The model is a port.** Bonsai is useful, but never trusted raw. Its output is
  treated as text that should become JSON, then parsed and validated.
- **The pack is declarative.** A pack can add recognizers, schemas, policies, evals,
  and sink routing. It cannot run code or see raw PHI/redaction maps.
- **Egress is default-deny.** Slack and trajectory storage are both behind verifier
  checks. A failed Slack post does not create a trajectory.
- **The phone is capture, not compute.** The phone gives a realistic first touch,
  but the current demo uses the Mac as the edge node. That avoids pretending the iOS
  model path is complete before hardware measurement.
- **No public tunnel for dictation.** The phone-to-Mac path stays on local Wi-Fi or
  Personal Hotspot. The only intended internet egress is the verifier-approved Slack
  payload.

## Runtime Workload Profile

| Workload | Where it runs | Network used | Sensitive data? | Cost driver |
|---|---|---|---|---|
| First screen status | Mac web shell | local browser -> `:8099`, Mac loopback to model status | no | tiny HTTP checks |
| Dictation / typing | phone browser | local LAN only | synthetic raw note in transit to Mac | browser input |
| Scrub | Mac `airplane-core` + local Bonsai | Mac loopback `127.0.0.1:8080` | raw note enters core; model sees scrubbed/prompted text only | model inference |
| Verifier gate | Mac `airplane-core` | none | clean candidate only | string/rule scan |
| Slack send | Mac web shell | HTTPS to Slack webhook | verifier-approved care record only | one network POST |
| Trajectory append | Mac filesystem | none | de-identified tuple only | local JSONL append |
| Full eval/gates | Mac CLI + local Bonsai | Mac loopback `127.0.0.1:8080` | synthetic eval notes only | 21 notes x 5 seeded passes |

Current eval profile from `eval/golden-run.txt`:

- Notes: `21`
- Model passes per note: `5`
- Local model calls in full eval: `105`
- Recall: `100.0%` (`71/71`)
- Hard-case recall: `100.0%` (`55/55`)
- Leakage: `0`
- Precision: `51.1%`
- Over-redactions: `68`

Interpretation: the harness is recall-first. It accepts conservative over-redaction
because the demo promise is "no residual identifiers leave," not "maximal text
retention."

## How We Verify It

Run these from the repo root.

### Server and phone readiness

```bash
./run.sh web
curl http://127.0.0.1:8099/api/status | jq '{slack:.slack, model:.model}'
curl http://<mac-lan-ip>:8099/api/health
```

Expected on this machine:

```json
{
  "slack": {
    "channel": "#coach-records",
    "configured": true,
    "route": "webhook"
  },
  "model": {
    "reachable": true,
    "route": "llama-server",
    "url": "http://127.0.0.1:8080/v1/chat/completions"
  }
}
```

### Slack sink proof

```bash
AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke
```

Expected:

```text
Slack route: webhook -> #coach-records
Slack smoke: app-originated gated send accepted
```

This proves app-originated Slack delivery through `/api/send`. It is different from
posting with a Slack connector or manually typing in Slack.

### Full gate proof

```bash
./run.sh gates
```

The harness checks:

- pack blindness
- reward-lint
- scope-boundary
- signature/provenance
- manifest/revocation
- recall
- leakage
- negative ethical fixtures

The negative fixtures create temporary bad packs and prove the same gate entrypoint
rejects:

- `usedSignals: [engagement]`
- `escalationRequired: false`

For iteration work that does not touch recall or leakage, use the fast lane:

```bash
./run.sh gates-fast
```

`gates-fast` runs the structural, policy, provenance, manifest, and negative
ethical fixture checks without making model calls. It is the right guard while
changing docs, Slack wiring, UI copy, or network setup. It is not a release
substitute for `./run.sh gates`, because it intentionally skips the 21-note
recall/leakage eval.

### Why full eval can take hours

The full eval is deliberately expensive: `21` synthetic notes x `5` seeded
passes = `105` serial Bonsai requests over `127.0.0.1:8080`. That is where the
time goes. The Rust rules, verifier, pack loading, and Slack fixtures are small;
model inference dominates.

The CLI now prints the eval plan, per-note timing, and total runtime before it
compares against `eval/golden-run.txt`. Model requests are bounded with
`AIRPLANE_MODEL_TIMEOUT_SECS` (default `120` seconds) so a dead local server
fails instead of blocking forever:

```bash
AIRPLANE_MODEL_TIMEOUT_SECS=120 ./run.sh gates
```

For a quick model-health smoke during development, run a reduced-pass eval
explicitly and do not treat it as the release bar:

```bash
AIRPLANE_EVAL_PASSES=1 ./run.sh eval
```

Slack egress in the web shell is also bounded with `AIRPLANE_SLACK_TIMEOUT_SECS`
(default `15` seconds). A failed Slack send does not append a trajectory.

### Mobile "Not Posted" State

The phone reaches **Not posted** through this state path:

```text
ready -> flush -> POST /api/send -> no ok:true response -> delivered(posted=false)
```

That state means the system did not receive a confirmed Slack acknowledgement.
The local trajectory store is skipped. The raw note still does not leave through
Slack because `/api/send` re-runs the verifier over the exact outbound Slack
text before using credentials.

There are three different failure classes:

- `Slack gate blocked residual identifiers`: the edge received the send request
  and refused egress before Slack.
- `Slack webhook/bot post failed...`: the edge received the request and Slack
  rejected or timed out.
- `phone lost connection to edge server...`: the phone did not complete the
  local LAN request to `airplane-web`.

The third class is where the incident was observed: on the phone UI after the
flush animation. Local status can still be green on the Mac because the Mac can
reach itself over `127.0.0.1`; the relevant check for the phone is the LAN URL,
for example:

```bash
curl http://192.168.1.88:8099/api/status | jq '{slack:.slack, model:.model}'
```

## Worked Example 1: Sample Note to Slack

Input note:

```text
Met with Maria Alvarez (CM-204815) at her place Tuesday. She's the one whose daughter just started college. Committed to a 10-min morning walk daily.
```

Scrubbed text:

```text
Met with [PERSON] ([MEMBER_ID]) at her place [DATE]. She's the one whose [FAMILY_DETAIL]. Committed to a 10-min morning walk daily.
```

Care record:

```text
Client: client ready circle
Themes: family transition · daily movement · routine building
Commitment: 10-min morning walk daily · open
Next touch: scheduled
Risk flags: none
```

Slack receives only the clean record plus the gate-clean footer. It does not receive
`Maria Alvarez`, `CM-204815`, the exact date, or the family detail.

## Worked Example 2: Adding a Pack Recognizer

Issue #1 is the pack reveal: show that extensibility comes from pack data, not a fork.

The coach-session pack includes `packs/coach-session/recognizers/benefits.json`, which
catches benefit IDs like:

```text
BEN-MH-7741
```

The eval set includes `note-21` for this format. The UI's **Pack reveal** action shows:

- the five pack surfaces
- the added recognizer
- the re-scrubbed note
- the new identifier caught as `BENEFIT_ID`

The point is not that benefit IDs are special. The point is that a user can redefine
identifier vocabulary declaratively while the verifier and core trust boundary stay
owned and reviewed.

## Worked Example 3: Clinical-Risk Language

The app is a coaching scribe, not a therapist. When the scrubbed text implies clinical
risk, the follow-up generator should surface escalation rather than a nudge.

Expected behavior:

```text
Risk flag: escalation
Follow-up: surface human escalation / crisis-support path
Reward signal: surface_human_escalation
```

This is enforced two ways:

- pack policy requires `scopeBoundary.escalationRequired: true`
- web tests verify clinical-risk language produces escalation instead of engagement

## Worked Example 4: Bad Reward Pack

A pack that rewards engagement is not allowed:

```yaml
followup:
  reward:
    autonomySignals: [commitment_completed]
    usedSignals: [engagement]
    forbiddenTerms: [engagement, retention, session_count, app_opens]
```

Expected gate result:

```text
gate reward-lint    : FAIL
```

This protects the demo from optimizing for dependence, retention, session count, or
app opens.

## Worked Example 5: Missing Escalation Boundary

A pack that removes the human escalation path is not allowed:

```yaml
followup:
  scopeBoundary:
    escalationRequired: false
    onClinicalRisk: surface_human_escalation
```

Expected gate result:

```text
gate scope-boundary : FAIL
```

This keeps the product claim bounded: coach support is allowed; therapist substitution
is not.

## Worked Example 6: Phone Access

Current URL on this network:

```text
http://192.168.1.88:8099/
```

If the phone cannot load it:

1. Confirm the Mac prints `phone: http://<ip>:8099` from `./run.sh web`.
2. Put phone and Mac on the same Wi-Fi.
3. If the network blocks peer devices, turn on iPhone Personal Hotspot and connect the
   Mac to it.
4. Use the `172.20.10.x:8099` URL printed by the server.

Do not use real phone airplane mode; that drops the local network path the phone needs
to reach the Mac.

## What This Demo Does Not Claim

- It does not claim the iPhone runs Bonsai yet. The current measured edge node is the Mac.
- It does not claim production-grade encrypted trajectory storage. The local JSONL store
  is gate-clean but not enclave-backed.
- It does not claim Sigstore/Rekor proof verification is complete. The local provenance
  gate catches manifest and digest tampering, but full Fulcio/Rekor validation remains
  future work.
- It does not use real patient/client data. Synthetic data only.
