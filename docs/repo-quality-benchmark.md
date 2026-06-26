# Repo Quality Benchmark

Date: 2026-06-26

This audit compares Airplane Mode against four canonical open-source reference
points:

- [HashiCorp Nomad](https://github.com/hashicorp/nomad) at `b86eb4e`
- [Ghostty](https://github.com/ghostty-org/ghostty) at `9f62873`
- [Uber Zap](https://github.com/uber-go/zap) at `5b81b37`
- [Cadence](https://github.com/cadence-workflow/cadence) at `c2276c2`

The goal is not to copy their size. Nomad and Cadence are mature infrastructure
systems; Ghostty is a serious application; Zap is a focused library. The bar to
copy is their discipline: obvious contributor path, bounded scope, reproducible
builds, visible maintenance surface, and clear separation between maintained
code and exploratory work.

## Current Measurement

Airplane Mode after the public-context cleanup:

| Measure | Current |
| --- | ---: |
| Tracked files | 163 |
| Markdown files | 66 |
| Markdown files under `docs/` | 38 |
| Rust files | 12 |
| Swift files | 4 |
| Shell scripts | 13 |
| `.github` files | 0 |

Root comparison:

| Repo | Tracked files | Workflows | Issue templates | Contributor docs | License | Maintainers |
| --- | ---: | ---: | ---: | --- | --- | --- |
| Airplane Mode | 163 | 0 | 0 | no `CONTRIBUTING.md` | missing | missing |
| Nomad | 5972 | 18 | 3 | `contributing/` | yes | CODEOWNERS |
| Ghostty | 5742 | 15 | 2 | `CONTRIBUTING.md` | yes | vouch system |
| Uber Zap | 177 | 1 | 3 | `CONTRIBUTING.md` | yes | implicit |
| Cadence | 2800 | 10 | 5 | `CONTRIBUTING.md` | yes | `MAINTAINERS.md` |

Read: our size is closer to Zap, but our governance surface is below Zap. That
is the strongest warning sign. We do not need Nomad-scale automation; we do need
Zap-level clarity.

## Scorecard

| Dimension | Grade | Evidence | Target |
| --- | --- | --- | --- |
| Product focus | B+ | README, CANON, intro doc, FSM, and deprecations now point to one current demo path. | Keep one primary path: phone browser -> first-party edge -> verifier -> Slack. |
| Context graph hygiene | B | Raw spike trees and superseded design specs are ignored/local-only. | Add a CI check that fails if ignored sausage paths are tracked again. |
| Reproducible build | B | `Cargo.lock`, `run.sh`, `gates-fast`, `gates`, model hash flow. | Add GitHub Actions for fast gates and Rust tests. |
| Contributor onboarding | C | README is strong, but there is no `CONTRIBUTING.md`, PR template, issue templates, or CODEOWNERS. | Add minimal `.github/` and contribution contract. |
| Legal/OSS hygiene | D | No `LICENSE`, no `SECURITY.md`, no code of conduct. | Choose a license before public push; add security disclosure policy. |
| Maintainer boundaries | C | AGENTS and DRs are good, but ownership is implicit. | Add `MAINTAINERS.md` or CODEOWNERS for core, web shell, docs, packs. |
| Dead-code control | B- | Deprecated paths are documented and ignored, but not enforced. | Add `scripts/check-public-graph.sh` and run it in CI. |
| Test posture | B | Fast gates and structural gates exist; full eval is documented as release proof. | CI should run fast gates on every PR; full eval remains manual/release. |
| API/extension clarity | B | Pack docs and reference architecture explain extension without forking core. | Add one small `packs/example-*` template only if it reduces friction. |
| Ambition management | B- | iOS/MLX claims are mostly honest, but simulator scaffold still looks bigger than the maintained path. | Keep iOS labeled deferred until real hardware measurement exists. |

Overall: **B- for a private build loop, C+ for a public OSS repo.** The core idea
is coherent. The repo is not yet at canonical OSS quality because the
contribution and maintenance surface is underbuilt.

## What The Benchmarks Teach

Nomad's lesson: large infrastructure repos make maintenance legible. They expose
CODEOWNERS, issue templates, pull request templates, CI, changelog flow, release
metadata, and security scans. We should copy the legibility, not the volume.

Ghostty's lesson: ambitious product repos can still keep contribution gated and
opinionated. Its vouching and workflow surface make clear that not every idea is
accepted by default. We need the same posture for runtime spikes: local until
promoted.

Uber Zap's lesson: a small repo can feel mature with a short README,
CONTRIBUTING, CHANGELOG, LICENSE, code of conduct, one CI workflow, and issue
templates. This is the closest near-term bar for Airplane Mode.

Cadence's lesson: ecosystem projects need explicit maintainer and PR process.
That matters here because our audience includes both adopters and builders.

## Slop Risks Still Present

1. **No public contribution contract.** A contributor cannot tell what kind of
   PR is welcome, what gates must pass, or how to propose a pack/runtime change.
2. **No license.** This blocks serious reuse.
3. **No security disclosure path.** Bad fit for a privacy-boundary demo.
4. **No CI.** The gates exist locally but are not visible as a public repo norm.
5. **No GitHub templates.** Issues can easily become ambition sprawl.
6. **Too much deferred iOS surface.** It is honest, but still visually prominent.
   Keep it clearly marked as simulator-only until measured.
7. **Dead-code policy is manual.** `.gitignore` helps, but future tracked
   sausage needs an automated check.

## Recommended Fix Order

1. **Add OSS table stakes.**
   - `LICENSE` after Jai chooses the license.
   - `CONTRIBUTING.md`
   - `SECURITY.md`
   - `CODE_OF_CONDUCT.md`
   - `MAINTAINERS.md`

2. **Add minimal GitHub hygiene.**
   - `.github/pull_request_template.md`
   - `.github/ISSUE_TEMPLATE/bug_report.md`
   - `.github/ISSUE_TEMPLATE/pack_request.md`
   - `.github/ISSUE_TEMPLATE/runtime_spike.md`
   - `.github/CODEOWNERS`

3. **Add one CI workflow.**
   - `cargo test`
   - `./run.sh gates-fast`
   - public markdown link check
   - public graph check

4. **Automate dead-code containment.**
   - `scripts/check-public-graph.sh`
   - Fail if `spike/`, `docs/superpowers/`, `docs/deprecations/spikes/`,
     `full-ctx.md`, `.airplane/`, generated logs, or model artifacts become
     tracked.

5. **Keep current ambition boundary.**
   - Browser GPU is the next runtime path.
   - iOS remains simulator-safe/deferred until real device measurement.
   - Old experiments are revived through GitOps, not copied into main docs.

## Definition Of "Ready Enough"

Airplane Mode reaches the intended OSS bar when a new contributor can answer
these questions in under five minutes:

1. What does the demo do?
2. What claim is allowed, and what claim is not allowed?
3. How do I run it?
4. How do I verify it?
5. How do I extend it without weakening the gate?
6. What kind of issue or PR should I open?
7. Who owns review?
8. How do I revive old experiments without polluting the main path?

We currently answer 1-5 and 8 well. We do not yet answer 6-7 at the level of
Nomad, Ghostty, Zap, or Cadence.
