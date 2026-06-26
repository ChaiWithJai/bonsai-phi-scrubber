# iOS Shell Scaffold Boundary

This is the simulator-safe plan for issue #9. It is intentionally a planning artifact,
not a shipped iOS shell.

## What Simulator Can Prove

An iOS Simulator mock can prove interaction choreography:

1. consent
2. capture text manually or through the simulator keyboard
3. show scrub pending
4. show verifier gate pass/block state
5. hold send while the in-app airplane-mode control is on
6. flush a clean record when the control turns off

That is useful for product review of the first touch point and the proof beat. It does
not prove the privacy claim.

## What Simulator Cannot Prove

Do not claim any of these from Simulator:

- Bonsai text weights load through `mlx-swift`
- iPhone 11 / A13 memory or tokens/sec
- Secure Enclave or Keychain behavior for raw notes and redaction maps
- real radios-off airplane-mode isolation
- on-device eval parity with the CLI

## Gate Before Real iOS Work

`backlog/m3.md` starts with `M3-T00`: load Ternary-Bonsai-1.7B MLX text weights on
the oldest available physical iPhone, record tokens/sec and memory, then stop for Jai's
decision. That remains the blocker for a real `shells/ios/` implementation.

Until that measurement exists:

- do not add `./run.sh ios`
- do not mark M3 tasks complete
- do not list iOS as a built shell
- if a visual mock is needed, name it `shells/ios-sim-mock/` and label it as choreography-only

## Future Real File Map

These files are not created yet:

- `shells/ios/` SwiftUI app shell
- UniFFI binding configuration for `airplane-core`
- Swift `InferenceProvider` backed by `mlx-swift`
- Keychain-backed `SecureStore`
- ASR/text `Capture`
- Slack `Sink` that receives only verifier-clean records

The web shell remains the honest Beat 1 demo until the physical-device measurement gate
is cleared.
