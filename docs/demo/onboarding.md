# Airplane Mode — Beat 1 Demo Runbook (Phone)

Run the on-device PHI-scrub demo on a phone in about five minutes: a clean web shell over the Rust `airplane-core`, with all the model work happening locally on your Mac.

For architecture, network topology, and data-flow diagrams, see
[`docs/demo/system-network-data-flows.md`](system-network-data-flows.md).
For a narrative walkthrough, verification guide, workload profile, and worked examples,
see [`docs/demo/how-the-demo-works.md`](how-the-demo-works.md).

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
   This builds + runs the web shell, binds `0.0.0.0:8099`, and prints `http://localhost:8099` plus any LAN URLs.

### Pre-checks (run on the Mac)

- **Is the web server listening?**
  ```sh
  lsof -nP -iTCP:8099 -sTCP:LISTEN
  ```
  Expect to see `*:8099`.
- **Is the macOS firewall on?**
  ```sh
  /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
  ```
  If it reports "enabled", it may block incoming connections — allow `airplane-web` or disable the firewall temporarily.
- **Can the Mac reach its own server?**
  ```sh
  curl http://<lan-ip>:8099/api/health
  ```
  Should return `{"ok":true}`.

## Get on the demo (connect your phone)

First, find the URL: the `./run.sh web` output prints it, or get the Mac's IP with:
```sh
ipconfig getifaddr en0
```
Then use `http://<that-ip>:8099`.

### Primary: same Wi-Fi

1. Put the phone on the **same Wi-Fi** as the Mac.
2. Open `http://<mac-lan-ip>:8099` in **Safari**.

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
4. On the phone, open `http://172.20.10.x:8099`.

**No server restart needed** — `airplane-web` already listens on all interfaces. All traffic stays on the phone's own local network (no internet), which preserves the demo's "stays local" story.

## Run the demo (8 steps)

1. Start on **"Dictate the session"** and tap **"Start dictation"** to speak a synthetic note, or tap **"Use sample note"** for the scripted demo note.
2. Confirm the first screen says **model ready** and **Slack live** (or **Slack preview** if you intentionally have no credential).
3. Tap **"Scrub on device."** Takes ~10–15s — it really runs Bonsai 5×, and the **"Scrubbing"** animation plays.
4. Tap **"continue →"** to the green **Verifier gate** (residual identifiers: **0**).
5. Review the clean **Care record** (with the commitment).
6. Tap **"Send to #coach-records."**
7. The app re-runs the verifier over the exact Slack payload, then posts only if the Slack credential is configured.
8. On success, tap **"View Slack"** to see the de-identified Slack card; in preview mode the screen explains which credential is missing.

## Wire Slack — the real post (the payoff)

By default the record shows as a **preview**. To make it actually land in a Slack channel — so your audience can open Slack and evaluate the clean record — provide one runtime credential. Secrets are sourced from the environment or macOS Keychain and are never committed.

### Fast path: incoming webhook

1. Create the Slack app from `slack-app-manifest.yaml` in Slack's app config UI, then install it to your workspace. The manifest enables incoming webhooks and requests only `chat:write` for the bot-token fallback. Slack's manifest reference: <https://docs.slack.dev/reference/app-manifest>.
2. In the app settings, open **Incoming Webhooks** → **Add New Webhook to Workspace** → choose `#coach-records` → **Allow** → copy the URL (`https://hooks.slack.com/services/...`). Slack's docs: <https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks>.
3. Store the URL in Keychain:
   ```bash
   scripts/setup-slack-secret.sh webhook
   AIRPLANE_WEB_ADDR=0.0.0.0:8099 ./run.sh web
   ```
   Or restart the web server with the URL in the environment:
   ```bash
   SLACK_WEBHOOK_URL='https://hooks.slack.com/services/XXX/YYY/ZZZ' AIRPLANE_WEB_ADDR=0.0.0.0:8099 ./run.sh web
   ```
   On startup it prints `slack: webhook configured — records post for real`.
4. Run the demo. When the queue flushes, the **de-identified record posts to that Slack channel for real** — no name, no member ID. Open Slack on the big screen and evaluate it.

### Pack-routed path: bot token + channel map

Use this when you want the channel to come from `packs/coach-session/sink.yaml` (`channelMap.default`), or when incoming webhooks are not available.

1. Create or use a Slack app with `chat:write`. Slack's docs: <https://docs.slack.dev/messaging/sending-and-scheduling-messages> and <https://docs.slack.dev/reference/scopes/chat.write>.
2. Install it into the workspace and copy the bot token (`xoxb-...`).
3. Either pass it in the environment:
   ```bash
   SLACK_BOT_TOKEN='xoxb-...' SLACK_CHANNEL='#coach-records' AIRPLANE_WEB_ADDR=0.0.0.0:8099 ./run.sh web
   ```
   Or put it in Keychain under the pack's configured ref:
   ```bash
   scripts/setup-slack-secret.sh bot-token '#coach-records'
   AIRPLANE_WEB_ADDR=0.0.0.0:8099 ./run.sh web
   ```
   If `SLACK_CHANNEL` is absent, the sink routes to `channelMap.default` in `sink.yaml`.
4. On startup it prints `slack: SLACK_BOT_TOKEN set — records post to #coach-records`.

The Slack endpoint re-runs the verifier gate over the outgoing de-identified record before posting. A residual identifier blocks the send before Slack credentials are used. Without a webhook or bot token, the demo still runs and the delivered screen explains which credential to set.

Preflight the current sink before the demo:

```bash
curl http://localhost:8099/api/status | jq .
```

Expect `.model.reachable == true` before scrubbing. Expect `.slack.configured == true` and route `webhook` or `bot_token` for a real Slack post. If the Slack route is `preview`, the UI will still run but the clean card will not leave the app.

After Slack is configured, prove the app-originated sink without dictation:

```bash
AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke
```

That smoke posts one synthetic, gate-clean record through `/api/send`. It fails before posting if the model is unreachable, Slack is still in preview mode, or the verifier blocks the outbound record.

## Warnings

- **Do NOT put the phone in REAL airplane mode.** That drops Wi-Fi/hotspot and the phone can't reach the Mac. Use the app's send flow; the verifier and Slack sink decide whether anything can leave.
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
