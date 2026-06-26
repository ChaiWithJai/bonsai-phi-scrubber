# CANON — Current Demo Contract

This file is the low-cognitive-load orientation for the current Airplane Mode
demo. Historical canon and superseded design material live in
[`docs/deprecations/canon/`](docs/deprecations/canon/).

The core theme is Jai's intro note:
[`docs/bonsai-ecosystem-plan.md`](docs/bonsai-ecosystem-plan.md). Read it as the
"who, what, and why" for the repo: dogfood Bonsai in public, prove local
inference with a hard healthcare coaching case study, and turn the result into a
reference other builders and adopters can reuse.

## Current Demo Path

```text
phone browser -> first-party edge laptop -> verifier gate -> Slack
```

The current web demo uses the phone as the capture surface and the laptop as the
edge node. Browser GPU is the default attempt and preferred next runtime spike;
the laptop edge fallback keeps the demo reliable.

Allowed claim:

> Raw synthetic notes stay on the first-party phone/laptop edge, and only a
> verifier-clean record reaches Slack.

Do not claim, for the web demo:

> Raw notes never leave the phone.

That literal proof is deferred until the model and scrubber run without a network
dependency.

## Current Read Order

1. [`docs/bonsai-ecosystem-plan.md`](docs/bonsai-ecosystem-plan.md) — who, what, why.
2. [`README.md`](README.md) — run and understand the demo.
3. [`docs/demo/onboarding.md`](docs/demo/onboarding.md) — phone, HTTPS, Slack.
4. [`docs/demo/fsm-service-map.md`](docs/demo/fsm-service-map.md) — service map, screens, addressable state.
5. [`docs/demo/how-the-demo-works.md`](docs/demo/how-the-demo-works.md) — architecture, topology, examples.
6. [`docs/extending.md`](docs/extending.md) — change the pack for a hackathon.
7. [`docs/deprecations/`](docs/deprecations/) — old patterns and revival.

## Load-Bearing Decisions

| Decision | Current record |
| --- | --- |
| Portable Rust core plus shell ports | [`files/adr-014-portable-rust-core.md`](files/adr-014-portable-rust-core.md) |
| Web demo simulates airplane mode; literal radio-off proof deferred | [`files/adr-015-airplane-mode-simulated-in-web-demo.md`](files/adr-015-airplane-mode-simulated-in-web-demo.md) |
| Build spec and trust boundary | [`files/rfc-002-final-ship.md`](files/rfc-002-final-ship.md) |
| Deprecated pattern index | [`docs/deprecations/decision-records/DR-2026-06-26-deprecated-pattern-index.md`](docs/deprecations/decision-records/DR-2026-06-26-deprecated-pattern-index.md) |
| Public context graph policy | [`docs/deprecations/decision-records/DR-2026-06-26-public-context-graph.md`](docs/deprecations/decision-records/DR-2026-06-26-public-context-graph.md) |

## What Stays In The Main Path

- `crates/airplane-core/`
- `shells/web/`
- `shells/cli/`
- `shells/mcp/`
- `packs/coach-session/`
- `docs/demo/`
- `docs/contracts/`
- `docs/extending.md`
- `docs/model-setup.md`
- `docs/sovereign-network-pattern.md`

## What Moved Out Of The Main Path

Historical design essays, early product framing, spike reports, native iOS
non-claims, and old pack/clinic framing moved under:

```text
docs/deprecations/canon/
docs/deprecations/deferred/
```

Revive them with GitOps, not copy/paste:

```bash
git log -- docs/deprecations/canon/<file>
git show <commit>:docs/deprecations/canon/<file>
git switch -c revive/<pattern>
```
