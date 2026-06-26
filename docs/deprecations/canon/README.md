# Historical Canon Archive

This folder contains high-context design material that helped shape Airplane
Mode, but is no longer required to run, verify, or extend the core demo.

The archive exists to reduce cognitive load for hackathon users while keeping
ecosystem history available for builders.

## Read Current Docs First

For the current path, start with:

- [`README.md`](../../../README.md)
- [`docs/demo/onboarding.md`](../../demo/onboarding.md)
- [`docs/demo/how-the-demo-works.md`](../../demo/how-the-demo-works.md)
- [`docs/demo/reference-architecture.md`](../../demo/reference-architecture.md)
- [`docs/extending.md`](../../extending.md)

## What This Archive Contains

| File | Why archived |
| --- | --- |
| `CANON-legacy.md` | Long design-corpus map; replaced by the short current `CANON.md`. |
| `public-artifact.md` | Early public pitch; useful for messaging archaeology. |
| `demo-spec-airplane-mode.md` | Original literal airplane-mode proof; superseded for the web demo. |
| `engineering-spec-v1.md` | Early engineering spec; superseded by current README/demo docs and accepted ADRs. |
| `rfc-001-coaching-scribe-rl.md` | Earlier RL framing; current demo grows trajectories only. |
| `prd-coaching-scribe.md` | Product framing reference; not required for reproduction. |
| `clinic-pack-pattern.md` | Pre-current pack framing; current pack is YAML/JSON `coach-session`. |
| `phi-scrubber-technical-survey.md` | Technical survey and legal context; useful background, not a runbook. |
| `harnessed-loop.md` | Historical build-loop framing; active agent instructions live in `AGENTS.md`. |
| `state-machine-system.md` | FSM design context; current URL/state work should be driven from live code and issues. |
| `component-design.md` | Throughline matrix; useful for reviewers, too dense for first-run users. |
| `battletest-*.md`, `dual-map-*.md`, `devops-first-design.md` | Adoption lenses and strategy context. |

## Revival

Use the GitOps runbook before reviving anything from this archive:

```bash
git log -- docs/deprecations/canon/<file>
git show <commit>:docs/deprecations/canon/<file>
git switch -c revive/<pattern>
git restore --source <commit> -- docs/deprecations/canon/<file>
```

If a revived pattern changes runtime behavior, it must pass the current verifier
and demo gates before it is promoted out of this archive.
