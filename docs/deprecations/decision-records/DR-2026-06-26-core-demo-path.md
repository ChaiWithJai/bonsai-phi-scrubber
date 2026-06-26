# DR-2026-06-26: Core Demo Path

Status: Accepted

## Decision

The supported demo path is:

```text
phone browser -> first-party edge laptop -> verifier gate -> Slack
```

Browser GPU is the default attempt in the UI and the preferred next phone-local
runtime spike. The Mac/laptop edge fallback remains part of the supported demo
because it keeps the scrubber and verifier reliable for live presentation.

## Context

The repo accumulated several plausible paths:

- Native iOS + MLX Swift.
- Browser WebGPU.
- Laptop edge through phone browser.
- Literal radio-off airplane mode.
- Public tunnel capability probes.

The live demo now needs a stable, teachable path for hackathon users. It also
needs honest limits for ecosystem builders who care about ports and runtime
experiments.

## Current Path

- `./run.sh web`
- `./run.sh https-proxy`
- `https://<mac-lan-ip>:8443/`
- `AIRPLANE_WEB_URL=https://127.0.0.1:8443 ./run.sh browser-span-smoke`

Allowed claim:

> The raw synthetic note stays on the first-party phone/laptop edge and only a
> verifier-clean record reaches Slack.

Disallowed claim for the web demo:

> The raw note never leaves the phone.

## Deferred Paths

- Real iPhone local inference through MLX Swift.
- Literal radio-off airplane-mode proof.
- Production HIPAA posture.

These require separate measurement, certification, and deployment controls.

## Revival

Use the GitOps revival runbook when bringing back native iOS, literal airplane
mode, or older runtime experiments:

```bash
git switch -c revive/<pattern>
git show <commit>:<path>
git restore --source <commit> -- <path>
```

Do not make a revived path the default unless it passes the current verifier and
demo gates.

## Verification

Current demo verification:

```bash
cargo test -p airplane-web
./run.sh gates-fast
AIRPLANE_WEB_URL=https://127.0.0.1:8443 ./run.sh browser-span-smoke
```
