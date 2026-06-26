# Airplane Mode

Build a healthcare hackathon demo where a sensitive note is scrubbed at the edge,
verified before egress, and only a scrubbed care record reaches Slack.

This repo is for people who already run messy real-world workflows: automations,
scripts, Google Sheets glue, intake forms, Zapier-style handoffs, or EHR-adjacent
workarounds. You do not need to be a Rust expert. You do need to be willing to run
terminal commands and keep the demo synthetic.

The narrative anchor is Jai's intro note:
[`docs/bonsai-ecosystem-plan.md`](docs/bonsai-ecosystem-plan.md). It explains
who is building this, why healthcare is the first case study, and why the demo is
framed as Bonsai ecosystem work rather than a one-off app.

## What This Demo Proves

The core pattern is simple:

```text
synthetic note -> local scrubber -> verifier gate -> clean Slack record
```

The raw note is captured in a phone browser, but the compute runs on your laptop
as the local edge node. A PrismML Bonsai model helps identify sensitive text. The
model output is not trusted raw: the Rust core parses, clamps, redacts, and then
re-scans the exact outbound Slack payload before anything leaves.

This is not a production medical device, not HIPAA compliance in a box, and not
the final iPhone airplane-mode proof. It is a starter template for learning how
to structure healthcare AI workflows so sensitive data has a real boundary.

For adopters, the point is the job done: sensitive notes stop flowing into shared
tools before they are scrubbed. For builders, the point is the reference
architecture: model-as-port, deterministic harness, verifier gate, and
declarative packs.

This repo keeps those two registers separate. The docs are intentionally weighted
about 70/30 toward the end user: the runbook and pack workflow come first for
CNCF-style adopters who need a safe path, while the architecture, ports, and iOS
simulator backend selector are exposed for builders who want to extend the
inference path.

## Who Should Use This

Use this if you are:

- Building at a healthcare hackathon.
- Tired of EHR copy/paste work and want a safer automation pattern.
- Comfortable enough with scripts to follow a runbook.
- Managing a complex workflow in spreadsheets and wondering what should become
  software.
- Trying to understand how Codex or Claude Code can help you maintain a repo with
  tests, gates, docs, and demo workflows.

Do not put real patient data, real member IDs, or real session notes into this
demo. The repo and eval set are synthetic-only.

## Machine Requirements

The smooth path is an Apple Silicon Mac.

| Need | Recommendation | Why |
| --- | --- | --- |
| Computer | Apple Silicon Mac, 16 GB RAM or more | Runs the local model through Metal. |
| Disk | 8-10 GB free | The default Bonsai GGUF is about 3.2 GB; builds use more. |
| OS tools | Homebrew, Git, Rust, `jq` | Build, run, inspect status JSON. |
| Model runtime | `llama.cpp` with `llama-server` | Serves Bonsai on `127.0.0.1:8080`. |
| Browser | Safari/Chrome on laptop or phone | Runs the local web UI. |
| Phone | Optional, same Wi-Fi/hotspot as laptop | Makes the demo feel like capture at the edge. |
| Slack | Optional but recommended | Shows real, gated egress to `#coach-records`. |

Linux can work for CLI/model reproduction if you build/install `llama.cpp`, but
the Slack secret helper uses macOS Keychain. Windows is not the supported path
for this starter.

Install the basic tools:

```bash
brew install git rust llama.cpp jq gh
```

If you just created a GitHub account, install GitHub CLI, then authenticate:

```bash
gh auth login
```

You only need GitHub authentication if you plan to fork, push changes, or open
pull requests. Running the demo locally does not require it.

## Quick Start: Run The Core Demo

Clone the repo:

```bash
git clone https://github.com/ChaiWithJai/airplane-mode.git
cd airplane-mode
```

Start the local Bonsai model server in terminal 1:

```bash
./scripts/serve-model.sh
```

The first run downloads the pinned model and verifies its SHA-256 before serving.
Leave this terminal open. The model API will listen at:

```text
http://127.0.0.1:8080/v1/chat/completions
```

Start the web demo in terminal 2:

```bash
./run.sh web
```

The server prints URLs like:

```text
local: http://localhost:8099
phone: http://192.168.x.x:8099
```

For the phone Browser GPU path, start the local HTTPS proxy in terminal 3:

```bash
./run.sh https-proxy
```

Open `http://localhost:8099` on your laptop, or open
`https://<mac-lan-ip>:8443` on a phone connected to the same Wi-Fi. If the phone
is on the Mac's Personal Hotspot, use the hotspot IP printed by the server with
`:8443`. Use the plain `:8099` phone URL only for setup checks or fallback.

In the UI:

1. Tap **Use sample note** or dictate a synthetic note.
2. Tap **Scrub on device**.
3. Watch identifiers get removed.
4. Continue through the verifier gate.
5. Send the clean care record to Slack, or run in preview mode if Slack is not
   configured.

The phone is a touchscreen for this web build. The laptop is the current edge
node. The native iPhone shell is tracked separately.

The intended network shape is first-party and sovereign: phone to your own
laptop, hotspot, LAN, or IT-managed VPN. Do not use a third-party public tunnel
for the scrub workflow. See
[`docs/sovereign-network-pattern.md`](docs/sovereign-network-pattern.md).

To verify the simulator-safe iOS scaffold and backend selector, run:

```bash
./run.sh ios-sim
```

That command runs SwiftPM tests/build for the mock `mlx-swift` and edge-HTTP
backend modes. It does not prove real iPhone 11/A13 inference.

For the next real phone path, start with the browser GPU route. The ecosystem has
already proven a Bonsai WebGPU demo on Hugging Face, so the next dense move is to
wrap that runtime path in this repo's verifier gate. Keep research spikes local
until they are promoted into a decision record or current runbook; old runtime
experiments are revived through the GitOps runbook, not linked from the starter
path.

For a sovereign browser-GPU demo, warm the browser runtime and q1 Bonsai model
artifacts before opening the phone UI:

```bash
./run.sh vendor-browser-runtime
./run.sh vendor-browser-model
```

Those commands cache Transformers.js and the q1 ONNX artifacts under
`.airplane/`. They are served back to the phone from this laptop at `/vendor/...`
and `/models/...`; they are intentionally not committed to git.
The local server supports `HEAD` and byte ranges for those artifacts.
To watch for a real iPhone consuming the app/runtime/model routes, run:

```bash
AIRPLANE_WEB_URL=https://<mac-lan-ip>:8443 ./run.sh phone-request-observe
```

You can also open `https://<mac-lan-ip>:8443/proof` or the short
`https://<mac-lan-ip>:8443/p` on the laptop during a demo.
It refreshes `/api/status` and shows Slack/model health, active phone capability,
recent PHI-free browser requests, and the current phone URLs detected from the
edge server.

If you need HTTPS to test browser GPU behavior on a phone, do not tunnel the full
scrub app through Cloudflare. Use local HTTPS for the real app:

```bash
./run.sh web
./run.sh https-proxy
```

If you only need a non-PHI capability page, run the capability-only probe:

```bash
./run.sh gpu-probe
```

That probe does not accept notes or post to Slack. A public tunnel is acceptable
only for that capability-only surface, not for the scrub workflow. See
[`docs/hipaa-cloudflare-boundary.md`](docs/hipaa-cloudflare-boundary.md).

## Wire Slack For A Real Post

Without Slack credentials, the app still demos the scrub and verifier path, but
the final send stays in preview. To make the clean record post for real, use an
incoming webhook.

1. Create a Slack app from `slack-app-manifest.yaml`.
2. Enable Incoming Webhooks in Slack.
3. Add a webhook for `#coach-records`.
4. Store it locally:

```bash
scripts/setup-slack-secret.sh webhook
./run.sh web
```

The secret goes into macOS Keychain. Do not commit webhook URLs or bot tokens.

Check status:

```bash
curl http://127.0.0.1:8099/api/status | jq '{slack:.slack, model:.model}'
```

Expected for a live demo:

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

Smoke test the Slack sink:

```bash
AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke
```

## Reproduce The Trust Claim

The committed eval target is `eval/golden-run.txt`. It uses 21 synthetic notes
and 5 seeded Bonsai passes per note: 105 local model calls.

Run the full eval:

```bash
./run.sh eval
```

Expected headline:

```text
recall 100.0%
leakage 0
```

This can take a while because model inference dominates the runtime. For normal
UI/docs/Slack iteration, use the fast non-model gates:

```bash
./run.sh gates-fast
```

Use the full gates before release:

```bash
./run.sh gates
```

You can bound slow model requests:

```bash
AIRPLANE_MODEL_TIMEOUT_SECS=120 ./run.sh gates
```

## Extend The Template For Your Hackathon

Most healthcare hackathon ideas should start by changing the pack, not the Rust
core.

The reference pack lives here:

```text
packs/coach-session/
├── recognizers/   known identifier formats, like member IDs
├── schema.yaml    the clean care record shape
├── policy.yaml    redaction policy, reward rules, escalation boundary
├── sink.yaml      where clean records go
└── eval/          synthetic golden notes and expected redactions
```

Make your own pack:

```bash
cp -R packs/coach-session packs/my-hackathon-pack
```

Then edit:

- `recognizers/` for local identifier formats your workflow knows about.
- `schema.yaml` for the structured record you want after scrubbing.
- `policy.yaml` for recall threshold, escalation path, and autonomy-only reward
  rules.
- `sink.yaml` for the destination channel or route.
- `eval/golden/*.txt` and `eval/expected/*.json` with synthetic examples.

Run your pack:

```bash
PACK=packs/my-hackathon-pack ./run.sh eval
PACK=packs/my-hackathon-pack ./run.sh gates
```

If the gate fails, fix the pack or expected labels. Do not weaken the verifier
to make a demo pass.

## Common Hackathon Adaptations

Here are realistic starter ideas:

- Intake note to care-team Slack summary.
- Referral form scrubber before it enters a shared tracker.
- Scrubbed coaching recap for a community health worker.
- Benefits navigation note that strips member IDs before routing.
- Synthetic EHR discharge-summary exercise where only safe follow-ups leave.

For each one, ask:

1. What raw text is sensitive?
2. What identifiers must never leave?
3. What clean record is useful after redaction?
4. What sink receives only the clean record?
5. What synthetic eval notes prove it works?

## Where Codex And Claude Code Fit

This repo is designed for agent-assisted development. The useful pattern is not
"ask an AI to make an app." The useful pattern is:

- Keep the repo as the system of record.
- Put tasks in issues/backlog.
- Add gates that fail when trust boundaries break.
- Ask Codex or Claude Code to make small changes.
- Run the same checks every time.
- Commit and review the diff.

If you already maintain automations or spreadsheet workflows, think of this as
turning your gluework into a versioned workflow with tests.

## Architecture Map

```text
airplane-core
  Rust trust core
  rules executor · verifier gate · pipeline · pack loader
  depends on ports, not platforms

shells/web
  live phone/laptop demo

shells/cli
  reproducibility front door

shells/mcp
  agent-callable shell

shells/ios
  simulator-safe native scaffold with selectable backend mocks

packs/coach-session
  reference healthcare coaching pack
```

The model is a port. Bonsai is useful because it is small enough to make local
edge inference plausible, but the verifier gate is what decides whether anything
can leave. The iOS scaffold currently lets builders switch between an
`mlx-swift` text-path mock and an edge-HTTP mock that both emit the same
backend-shaped scrub response. That is interoperability scaffolding, not the
real iPhone 11 hardware proof.

For the screen-level state machine mapped to the services above, see
[`docs/demo/fsm-service-map.md`](docs/demo/fsm-service-map.md).

## Deprecated Patterns And Revival

The repo keeps old demo patterns as ecosystem history instead of deleting them.
If you want to study or revive native iOS, literal radio-off airplane mode,
public-tunnel capability probes, older clinic/HCL pack language, or other
superseded work, start here:

- [`docs/deprecations/demo-cleanup-proposal.md`](docs/deprecations/demo-cleanup-proposal.md)
- [`docs/deprecations/gitops-revival-runbook.md`](docs/deprecations/gitops-revival-runbook.md)
- [`docs/deprecations/decision-records/`](docs/deprecations/decision-records/)

The rule is: keep the current demo path clean, but make older patterns easy to
recover with Git when a builder has a real use case.

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| `llama-server not found` | Run `brew install llama.cpp`. |
| Model download fails | Check network access and rerun `./scripts/serve-model.sh`. |
| Web page loads but model says off | Start `./scripts/serve-model.sh` and leave it running. |
| Phone cannot load demo | Use the printed `phone:` URL, same Wi-Fi, or Mac Personal Hotspot. Do not use `127.0.0.1` on the phone. |
| Slack says preview | Configure webhook or bot token; check `/api/status`. |
| Send fails but edge is reachable | Retry; sends are idempotent by `send_id` to avoid duplicate Slack posts. |
| Eval takes too long | Use `./run.sh gates-fast` for non-model iteration; reserve full eval/gates for release proof. |

## Repository Guide

| Path | Purpose |
| --- | --- |
| `crates/airplane-core/` | Portable Rust trust core. |
| `shells/web/` | Phone-driven demo UI and local web shell. |
| `shells/cli/` | Eval, gates, and command-line scrub path. |
| `shells/mcp/` | Agent-callable interface over the same core. |
| `shells/ios/` | Simulator-safe native scaffold with selectable backend mocks. |
| `packs/coach-session/` | Reference pack and synthetic eval set. |
| `docs/bonsai-ecosystem-plan.md` | Jai's intro: who is building this, what he is doing, and why Bonsai ecosystem work is the theme. |
| `docs/model-setup.md` | Model download/runtime details. |
| `docs/deprecations/` | Cleanup proposal, deprecated-pattern decisions, and GitOps revival runbook. |
| `docs/hipaa-cloudflare-boundary.md` | Why Cloudflare HTTPS helps capability probes but is not HIPAA compliance. |
| `docs/sovereign-network-pattern.md` | First-party phone-to-edge network pattern for adopters. |
| `docs/contracts/` | Shared JSON contract fixtures for shell/backend interoperability. |
| `docs/demo/onboarding.md` | Phone demo and Slack runbook. |
| `docs/demo/fsm-service-map.md` | Current service map, screen FSM, addressable state, and service failure meanings. |
| `docs/demo/reference-architecture.md` | CNCF-style adopter/builder reference architecture. |
| `docs/extending.md` | Pack extension walkthrough. |
| `docs/demo/how-the-demo-works.md` | Architecture, topology, workload profile, worked examples. |
| `docs/positioning/cncf-end-user-and-inference-ecosystem.md` | Audience split: adopter outcome language vs. builder mechanism language. |
| `docs/positioning/prismml-partner-brief.md` | Respectful partner-facing brief for how Airplane Mode can help Bonsai adoption. |
| `AGENTS.md` | Harnessed build loop and hard rules. |
| `CANON.md` | Design canon index. |

## Safety Boundaries

- Synthetic data only.
- Raw note and redaction map never go to Slack.
- Slack and trajectory storage are behind the verifier gate.
- Packs are declarative and code-free.
- The current web demo uses the laptop as the edge node.
- Native iPhone, real airplane-mode proof, real `mlx-swift` text inference, and
  encrypted trajectory storage are tracked as future work.

Built to make sensitive workflows easier to inspect, reproduce, and improve.
