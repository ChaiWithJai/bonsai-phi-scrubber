# Review Readiness

This is the current review map for the Beat 1 / issue #1-#12 branch. It separates
verified local behavior from work that still needs credentials, hardware, or a stronger
production implementation.

## Current Live Demo

- Web shell: `http://127.0.0.1:8099`
- Status preflight: `curl http://127.0.0.1:8099/api/status | jq .`
- Current local Slack state: `route: webhook`, `configured: true`
- Reason: the Slack app manifest was installed in Jai's A+ Active Services
  workspace and the generated incoming webhook is stored in macOS Keychain as
  `slack-webhook-url`.
- Slack channel: `#coach-records` exists in Jai's Slack workspace and has received
  app-originated posts from the `Airplane Mode` Slack app.

The app now reports Slack and local model readiness on screen 1 before dictation starts.
On this machine, the webhook route is live; other machines still need one runtime
credential as described in `docs/demo/onboarding.md`.

## Verified Commands

These have passed on this branch:

```bash
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p airplane-web
cargo clippy -p airplane-web --all-targets -- -D warnings
./run.sh eval --check
./run.sh gates
AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke   # after Slack credential install
./scripts/smoke-mcp-cli-parity.sh
PACK=packs/coach-session MCP_PARITY_LIMIT=1 ./scripts/smoke-mcp-cli-parity.sh
(cd shells/ios && swift test)
```

The current committed eval target is `eval/golden-run.txt`:

- notes: 21
- recall: 100.0% (71/71)
- hard-case recall: 100.0% (55/55)
- leakage: 0
- precision: 51.1% (71/139 predicted)
- over-redactions: 68

## Issue Coverage

| Issue | Review state | Evidence |
|---|---|---|
| #1 Pack reveal | Covered for demo | Real `recognizers/benefits.json` is wired in `pack.yaml`; `note-21` eval catches `BEN-MH-7741`; `/api/pack-demo` shows five pack surfaces, baseline miss, real-pack catch, and pack eval smoke. `PACK=` is honored by CLI, web, MCP, and parity smoke; invalid pack paths fail closed at load. |
| #2 Reward-lint | Covered | `Pack::validate_reward_lint`; `./run.sh gates`; web/CLI tests; `scripts/smoke-ethical-gate-fixtures.sh` proves a temporary pack with `usedSignals: [engagement]` fails through the gate entrypoint. |
| #3 Scope-boundary | Covered | `Pack::validate_scope_boundary`; `./run.sh gates`; escalation policy in `policy.yaml`; `scripts/smoke-ethical-gate-fixtures.sh` proves a temporary pack with `escalationRequired: false` fails through the gate entrypoint. |
| #4 Follow-up/autonomy | MVP covered | Web structurer emits client-paced follow-up and autonomy signals; clinical-risk language surfaces escalation. Not a trained policy. |
| #5 Trajectory recorder | Local durable MVP; encryption still open | `/api/trajectory` builds a de-identified `(s,a,r,s')` JSON tuple, gates the exact serialized payload with the same verifier, appends gate-clean records to local JSONL, and returns a count recovered from the store. The web loop records a trajectory only after Slack send succeeds, so preview/failed sends do not count as completed loops. No policy training. Not enclave-encrypted at rest yet. |
| #6 Themes quality | Covered for sample/demo | Themes are grounded, junk-filtered, and have deterministic fallback tests. |
| #7 Precision tuning | Improved, still recall-first | Precision tracked; over-redactions reduced while keeping 100% recall / 0 leakage. Remaining extras are privacy-conservative. |
| #8 MCP shell | Covered for smoke/parity | `shells/mcp`; `./run.sh mcp`; `scripts/smoke-mcp-cli-parity.sh` compares CLI/MCP scrubbed text on golden notes. |
| #9 iOS/R1 | Simulator artifact only; hardware blocked | `shells/ios` Swift package proves choreography only; `docs/ios-shell-scaffold.md` documents non-claims. Real mlx-swift/R1 measurement still requires physical device. |
| #10 Slack bot-token routing | Live webhook covered; bot-token fallback code covered | Web sink supports webhook, bot token, channel map, and Keychain lookup; `/api/send` gates the exact outbound Slack content before posting; `/api/scrub` returns redaction entity/layer summaries to the browser without raw matched text; `#coach-records` exists; `AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke` proves app-originated posting through the installed Slack app webhook. |
| #11 Manifest/provenance | Stronger local gate; still not full Sigstore | Manifest/provenance gates now require the trusted GitHub Actions release identity, UUID-shaped Rekor references, coherent provenance source/ref, and SHA-256 digests for declared pack files. This catches local pack tampering but still does not perform Fulcio certificate validation or Rekor inclusion proof. |
| #12 Determinism | Covered locally | `./run.sh eval` / `--check` compares the current report to committed `eval/golden-run.txt`; `--update` is the explicit mutation path. Golden report includes precision and 21 notes. |

## Reviewer Notes

- Do not treat the simulator scaffold as M3 completion. It proves only UI choreography.
- The current local Slack state is a successful app-originated webhook post to
  `#coach-records`. Re-run `AIRPLANE_WEB_URL=http://127.0.0.1:8099 ./run.sh slack-smoke`
  before final review if the Slack app or Keychain credential changes.
