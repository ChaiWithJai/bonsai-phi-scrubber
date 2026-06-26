# airplane-ios — simulator-safe shell scaffold

This is a minimal Swift Package scaffold for the future native iOS shell. It is intentionally
honest: it simulates the Beat 1 Airplane Mode flow and UI states, but it does **not** claim the
M3 hardware proof.

## What it proves

- The native shell can present the expected loop: Idle -> Capturing -> Scrubbing -> Gated ->
  Structured -> Send held -> Flush -> Delivered.
- The UI keeps egress blocked until a simulated verifier reports zero residual identifiers.
- Raw synthetic input and the redaction map stay behind a simulator-only store API; only count
  metadata and the de-identified record are surfaced.
- The package can be opened by Xcode and tested locally with SwiftPM.

## What remains hardware-blocked

- No `mlx-swift` integration or in-process Bonsai text model runs here.
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
preview or host app, and run it in a simulator. The package itself is deliberately just the shell
surface and deterministic simulator flow, not a signed app target.
