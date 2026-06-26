# airplane-ios — simulator-safe shell scaffold

This is a minimal Swift Package scaffold for the future native iOS shell. It is intentionally
honest: it simulates the Beat 1 Airplane Mode flow and UI states, but it does **not** claim the
M3 hardware proof.

## What it proves

- The native shell can present the expected loop: Idle -> Capturing -> Scrubbing -> Gated ->
  Structured -> Send held -> Flush -> Delivered.
- The native UI can select how the backend is run before capture:
  - `MLX Swift mock`: simulator stand-in for the future in-process `mlx-swift` text path on
    iPhone 11/A13-class hardware.
  - `Edge HTTP mock`: simulator stand-in for the laptop `/api/scrub` JSON contract.
- Both mocks return the same backend-shaped DTOs as the web shell uses conceptually:
  `scrubbed_text`, `redactions`, `gate_pass`, `residual_count`, and a scrubbed `record`.
  The shared contract fixture is `../../docs/contracts/scrub-response.sample.json`.
- The UI keeps egress blocked until a simulated verifier reports zero residual identifiers.
- Raw synthetic input and the redaction map stay behind a simulator-only store API; only count
  metadata and the scrubbed record are surfaced.
- The package can be opened by Xcode and tested locally with SwiftPM.

## What remains hardware-blocked

- No real `mlx-swift` integration or in-process Bonsai text model runs here.
- No UniFFI binding to `airplane-core` is wired yet.
- No Secure Enclave or Keychain implementation is claimed; storage is in-memory simulator state.
- No real Airplane Mode/radio-off proof is claimed.
- No R1 measurement has been performed on iPhone 11/A13 or any other device from this scaffold.
- No real Slack sink is used; delivery is a simulated clean payload state.

Those items remain the M3 path in `../../backlog/m3.md`, starting with the real mlx-swift device
measurement gate.

## Run

From this directory:

```bash
swift test
swift build
```

To inspect the UI scaffold, open `Package.swift` in Xcode, add `AirplaneDemoView()` to a local
preview or host app, and run it in a simulator. Use the segmented control at the top of the view
to switch between `MLX Swift mock` and `Edge HTTP mock` before capture.

The package itself is deliberately just the shell surface and deterministic simulator flow, not a
signed app target.

## Builder note

The open contribution path is the real `InferenceProvider` adapter: replace
`SimulatorMLXSwiftTextProvider` with an `mlx-swift` implementation that accepts the same
`BackendScrubRequest` shape and returns the same `BackendScrubResponse` shape, then run the M3-T00
measurement on the oldest available iPhone before making any 2019/iPhone 11 claim.
