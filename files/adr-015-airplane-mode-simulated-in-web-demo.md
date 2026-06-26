# ADR-015 — Airplane mode is simulated in the web demo; the splash screen is removed
**Status:** Accepted (2026-06-25). Follows the Beat-1 web pivot (see `backlog/m3.md`) and builds on ADR-014 (the web shell is a shell over `airplane-core`).

## Context
The original demo (`docs/deprecations/canon/demo-spec-airplane-mode.md`, RFC-002) made **literal airplane mode the no-leak proof**: an iPhone with its radios physically off is *tamper-evidently* incapable of transmitting, it scrubs on-device, and only after the radios are switched back on does the clean record post. The demo spec's hard rule was explicit: *"Airplane mode is the proof, not a software meter."*

Beat 1 now ships as a **web shell**, not native iOS — TestFlight/native distribution was unavailable, so the phone runs a browser that reaches the scrub server on the **laptop** over Wi-Fi / Personal Hotspot (the "laptop as the edge node" framing in `shells/web/README.md`).

That breaks the literal proof: **if the phone's radios were actually off, its browser could not reach the laptop and the demo could not run.** Real airplane mode and a web-over-LAN demo are mutually exclusive. We also found the browser blocks the mic on plain `http://` (non-secure context), reinforcing that the web path is a different deployment with different physics.

## Decision
Two related "kills":

1. **Airplane mode becomes a *simulated* in-app egress toggle, not the phone's radios.** It gates the *send* (offline → "held / refuses to send" → toggle off → "flush → posted"), which demonstrates the egress-hold logic; the scrub always runs locally regardless of the toggle. The phone stays on the network the whole time; the toggle is a software control and is **explicitly not tamper-evident**. The honest claim shifts from *"the raw note never leaves the phone"* to **"the raw note never leaves the local link / never reaches the cloud"** (the laptop is the edge node). The UI copy and docs were updated accordingly ("never reached the cloud").

2. **The "Airplane Mode" splash / idle screen (screen 01) is removed.** It set the toggle and the brand framing but added a step before the actual work. The demo now opens directly on capture; airplane state defaults to "offline," and the single toggle lives on the send-held screen where it does real work.

## Rejected alternatives
- **Bonsai in the phone browser (WASM/WebGPU) over HTTPS** — would restore true on-device inference *and* real airplane mode, but the WASM path for Bonsai is unproven and heavy, and secure-context/mic constraints make it a research project, not a demo.
- **Public tunnel (ngrok / cloudflared)** — works over cellular, but routes the raw note over the internet, directly contradicting the privacy thesis. Rejected on principle.
- **Native iOS via TestFlight** — the tamper-evident path, but distribution is unavailable right now.

## Consequences
- The literal *"radios physically off, tamper-evident"* proof is **deferred, not abandoned** — it returns with the native-iOS / on-device (WASM) path. The web demo is honestly a **simulation** of the egress logic, with the **scrub and verifier gate fully real** (100% recall / 0 leakage on the golden set; see `eval/golden-run.txt`).
- Beat 1 **runs on any phone today**, no TestFlight, no app install.
- This is recorded so the reframing isn't mistaken for a weakening of the thesis: the bet — intelligence comes to the data, egress is gated — holds; only the *tamper-evidence* of the radio-off proof is traded for runnable-today. Anyone restoring the native path should re-read the demo spec's "airplane mode is the proof" rule and treat it as the bar to clear.

**Refs:** `docs/deprecations/canon/demo-spec-airplane-mode.md` (the original proof), `files/adr-014-portable-rust-core.md`, `shells/web/README.md`, `docs/demo/onboarding.md` (the "don't use real airplane mode" warning).
