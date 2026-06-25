# airplane-web — Beat 1 demo (web shell)

A third adapter over `airplane-core` (alongside the CLI and the iOS shell): a small Rust
server + a single-phone web UI that walks the **Beat 1 "airplane-mode loop"** — Idle →
Capturing → **Scrubbing** → **Gated** → Structured → Send-held → **Flush** → Delivered,
then the Slack card. The scrub and verifier gate are **real** (same core the eval runs);
the structurer runs only on the already-de-identified text.

## Run it

```bash
./scripts/serve-model.sh    # the model layer (separate terminal)
./run.sh web                          # serves http://localhost:8088 and prints a LAN URL
```

- **On your Mac:** open `http://localhost:8088`.
- **On your phone (same Wi-Fi / hotspot):** open the printed `http://<laptop-ip>:8088`.

`POST /api/scrub {text}` → `{ scrubbed_text, redactions[], gate_pass, residual_count, record }`.

## The honest trust boundary (read this)

The demo's "device" is the **laptop as the edge node** — the thing running Bonsai locally
instead of the cloud. The phone is the touchscreen; its browser sends the raw note to the
laptop over a **local link** (Wi-Fi/hotspot), never to the cloud or any third party. So the
honest claim here is **"the raw note never leaves the local machine/network,"** and the
verifier gate proves **only a de-identified record is ever eligible to leave** it. The
stronger "never leaves the *phone*" claim would require running Bonsai in the phone browser
(WASM) — a future stretch. The "airplane mode" toggle in the UI gates *egress* (the send),
demonstrating the no-leak logic; the scrub always runs locally.

## Engineering notes (why the scrub looks the way it does)

Driving the real 1.7B model surfaced three failure modes, each fixed in `airplane-core`
(`pipeline.rs`) the **recall-safe** way — precision is never bought by lowering recall,
because this is a PHI boundary:

- **Field-swap** (name lands in the category slot) → the JSON schema constrains `entity`
  to a fixed **enum**, so the grammar can't put a name there.
- **Over-redaction of activities** ("morning walk" → [PERSON]) → a **few-shot** prompt plus
  a deterministic **shape validator** (`plausible`) that rejects only things that *cannot*
  be an identifier (a 7-word "PERSON", a multi-word "MEMBER_ID", common verbs).
- **Structurer hallucination** → commitments are **grounded** to words present in the
  de-identified note.

Detection is **union across seeded passes** (recall-first); the eval gate enforces recall
≥ the pack's threshold. Voting/agreement was tried and rejected — it leaks the hardest
short names, which a PHI boundary cannot accept.

## Connecting a phone (troubleshooting)

Same-Wi-Fi is the primary path: open the printed `http://<laptop-ip>:8088` on a phone
joined to the same network. If the phone **"can't connect"** despite being on the same
Wi-Fi, the network most likely enforces **client/AP isolation** (common on corporate/guest
Wi-Fi), which blocks device-to-device traffic. The reliable fix is the **iPhone Personal
Hotspot**: turn it on, connect the Mac to it, and the Mac's IP becomes `172.20.10.x` — open
`http://172.20.10.x:8088` on the phone. No server restart is needed, since the server binds
all interfaces. See `../../docs/demo/onboarding.md` for the full runbook.
