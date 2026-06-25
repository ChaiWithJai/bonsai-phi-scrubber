# Airplane Mode — Beat 1 Demo Runbook (Phone)

Run the on-device PHI-scrub demo on a phone in about five minutes: a clean web shell over the Rust `airplane-core`, with all the model work happening locally on your Mac.

## Prerequisites

- The repo checked out at `~/projects/iphone-phi-scrubber-demo`. (Architecture context: `CANON.md`, `shells/web/README.md`.)
- The PrismML Bonsai model + llama.cpp (one-time setup in **docs/model-setup.md**; `./scripts/serve-model.sh` fetches and serves it).
- A Mac and an iPhone with Safari.
- Keep the Mac awake and plugged in for the whole demo.

## Start the two servers

You need **both** running at the same time, in two terminals.

1. **Model server** (PrismML Bonsai via stock `llama-server`, OpenAI-compatible, default F16 on Metal):
   ```sh
   ./scripts/serve-model.sh
   ```
   This listens on `127.0.0.1:8080`.

2. **Web UI** (`airplane-web`), from the repo root:
   ```sh
   ./run.sh web
   ```
   This builds + runs the web shell, binds `0.0.0.0:8088`, and prints `http://localhost:8088` plus any LAN URLs.

### Pre-checks (run on the Mac)

- **Is the web server listening?**
  ```sh
  lsof -nP -iTCP:8088 -sTCP:LISTEN
  ```
  Expect to see `*:8088`.
- **Is the macOS firewall on?**
  ```sh
  /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
  ```
  If it reports "enabled", it may block incoming connections — allow `airplane-web` or disable the firewall temporarily.
- **Can the Mac reach its own server?**
  ```sh
  curl http://<lan-ip>:8088/api/health
  ```
  Should return `{"ok":true}`.

## Get on the demo (connect your phone)

First, find the URL: the `./run.sh web` output prints it, or get the Mac's IP with:
```sh
ipconfig getifaddr en0
```
Then use `http://<that-ip>:8088`.

### Primary: same Wi-Fi

1. Put the phone on the **same Wi-Fi** as the Mac.
2. Open `http://<mac-lan-ip>:8088` in **Safari**.

That's it — you should see the demo.

### Fallback: iPhone Personal Hotspot

If the phone **"can't connect" even on the same Wi-Fi**, the network has **client/AP isolation** — devices on it can't talk to each other. This is common on corporate, university, guest, and café Wi-Fi. It's a network setting, **not a bug in the app**.

The fix is to put both devices on the iPhone's own local network:

1. On the iPhone: **Settings → Personal Hotspot → "Allow Others to Join" ON**.
2. On the Mac: **Wi-Fi menu → connect to the iPhone's hotspot**.
3. The Mac's IP becomes a `172.20.10.x` address — confirm with:
   ```sh
   ipconfig getifaddr en0
   ```
4. On the phone, open `http://172.20.10.x:8088`.

**No server restart needed** — `airplane-web` already listens on all interfaces. All traffic stays on the phone's own local network (no internet), which preserves the demo's "stays local" story.

## Run the demo (8 steps)

1. **Start session.**
2. Note the **pre-filled** raw note: *"Met with Maria Alvarez (CM-204815)…"*
3. Tap **"Scrub on device."** Takes ~10–15s — it really runs Bonsai 5×, and the **"Scrubbing"** animation plays.
4. Tap **"continue →"** to the green **Verifier gate** (residual identifiers: **0**).
5. Review the clean **Care record** (with the commitment).
6. Tap **"Send to #coach-records."**
7. With the in-app airplane toggle **ON**, the send is **HELD** ("refuses to send").
8. Flip the **in-app airplane toggle OFF** → the queue flushes → **"Posted"** → tap **"View in #coach-records"** to see the de-identified Slack card.

## Wire Slack — the real post (the payoff)

By default the record shows as a **preview**. To make it actually land in a Slack channel — so your audience can open Slack and evaluate the clean record — set up an **incoming webhook** (2 minutes, no OAuth):

1. Go to **https://api.slack.com/apps** → **Create New App** → **From scratch** → name it "Airplane Mode", pick your workspace.
2. **Incoming Webhooks** → toggle **On** → **Add New Webhook to Workspace** → choose the channel (e.g. `#coach-records`) → **Allow** → copy the URL (`https://hooks.slack.com/services/...`).
3. Restart the web server with the URL in the environment:
   ```bash
   SLACK_WEBHOOK_URL='https://hooks.slack.com/services/XXX/YYY/ZZZ' ./run.sh web
   ```
   On startup it prints `slack: SLACK_WEBHOOK_URL set — records post for real`.
4. Run the demo. When the queue flushes, the **de-identified record posts to that Slack channel for real** — no name, no member ID. Open Slack on the big screen and evaluate it.

The webhook is a secret — it's read from the environment and never committed. Without it the demo still runs (the delivered screen shows "Held · set SLACK_WEBHOOK_URL").

## Warnings

- **Do NOT put the phone in REAL airplane mode.** That drops Wi-Fi/hotspot and the phone can't reach the Mac. The **"Airplane mode" toggle IN THE APP** is the demo control — that's the one you flip.
- **The "device" is honestly the laptop as the edge node** — it runs Bonsai locally. The phone is just the touchscreen. The raw note crosses only the local link, never the cloud.
- **Keep the Mac awake** and keep **both** servers running for the whole demo.
- **Don't run `./run.sh eval`** at the same time — it contends with the model and makes the scrub crawl.

## Troubleshooting

| Symptom | Likely cause & fix |
| --- | --- |
| Can't connect on same Wi-Fi | Client/AP isolation on the network. Use the **iPhone Personal Hotspot** fallback above. |
| Scrub spins forever / errors | Model server not running — start `serve.sh`. Or `eval` is running — stop it. |
| Page won't load at all | Wrong IP (re-run `ipconfig getifaddr en0`) or web server down (`./run.sh web`). |
| Frame/bezel showing on phone | Hard-refresh — the UI is full-bleed on phones. |
