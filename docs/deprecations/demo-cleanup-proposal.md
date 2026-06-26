# Proposed Demo Cleanup

The demo should be easy for two audiences to read:

- **Adopters** need the shortest path to reproduce the safe workflow.
- **Ecosystem builders** need enough historical context to fork old patterns,
  compare alternatives, and contribute runtime ports.

The cleanup strategy is therefore not "delete the past." It is "make the current
path obvious, then put older patterns behind explicit decision records."

## Keep In The Main Demo Path

| Area | Keep | Why |
| --- | --- | --- |
| Runtime | `crates/airplane-core/`, `shells/web/`, `shells/cli/`, `shells/mcp/` | These prove the current trust core and demo loop. |
| Phone setup | `./run.sh web`, `./run.sh https-proxy`, phone observer scripts | These are needed to demo Browser GPU and first-party network behavior. |
| Verification | `./run.sh gates-fast`, `./run.sh browser-span-smoke`, `./run.sh gates` | These substantiate the claims. |
| Pack extension | `packs/coach-session/`, `docs/extending.md`, `docs/contracts/` | This is the hackathon extension unit. |
| Demo docs | `README.md`, `docs/demo/onboarding.md`, `docs/demo/how-the-demo-works.md`, `docs/demo/reference-architecture.md` | These teach the end-user and builder story. |
| Sovereign network docs | `docs/sovereign-network-pattern.md`, `docs/hipaa-cloudflare-boundary.md` | These explain why local HTTPS/VPN beats third-party tunneling for the scrub workflow. |

## Keep, But Mark As Deferred

| Area | Current interpretation |
| --- | --- |
| `shells/ios/` | Simulator-safe scaffold and contract exercise, not a proven real-device MLX path. |
| Native MLX Swift text inference | Future measurement gate. Do not use as a present-tense demo claim. |
| Literal airplane-mode/radio-off proof | Deferred until the model and scrubber run without any network dependency. |
| Encrypted trajectory storage | Valuable production hardening, not needed for the synthetic demo. |
| Full `./run.sh gates` during iteration | Release proof only; use `gates-fast` for normal work. |

## Deprecate From The Main Pitch

| Pattern | Replacement | Why |
| --- | --- | --- |
| Public tunnel for the full scrub app | Local HTTPS or IT-managed VPN | Public tunnels put the scrub workflow on a third-party transport and confuse the BAA story. |
| "HIPAA compliant because HTTPS" | "Synthetic demo; first-party network; BAA required for real PHI services" | HTTPS is transport security, not compliance. |
| "Raw note never leaves the phone" for the web demo | "Raw note never reaches the cloud; laptop is the current edge node" | The web path sends the note to the laptop edge. |
| Old clinic/HCL pack examples as current docs | YAML/JSON `coach-session` pack | The accepted pack contract is code-free YAML/JSON. |
| Splash/landing-page demo framing | Actual capture workflow as first screen | The demo should start where the user does the work. |

## Proposed Public Repo Shape

Keep the root narrow:

```text
README.md
AGENTS.md
CANON.md
run.sh
crates/
shells/
packs/
gates/
eval/
scripts/
docs/
```

Inside `docs/`, make the current path obvious:

```text
docs/demo/                 runbook, architecture, incidents
docs/deprecations/         old-pattern decisions and revival
docs/positioning/          audience-specific framing
docs/contracts/            API/schema fixtures
docs/seed/                 Bonsai ecosystem facts
```

The older `files/` canon is still useful, but it should be treated as historical
design context. The current public path should link readers to the smaller demo
docs first, then let ecosystem builders dig into the canon when needed.

## Cleanup Sequence

1. Add this deprecations folder and decision records.
2. Add pointers from `README.md` and `CANON.md`.
3. Stamp superseded docs with one-line status notes instead of editing their
   substance.
4. Only after review, consider moving stale docs into a `docs/deprecations/old-canon/`
   folder in a separate commit.
5. For any move, record the pre-move commit SHA and old path in a decision
   record so `git show <sha>:<old-path>` still works.

## Done-When For Cleanup

- A newcomer can find the core demo in under two minutes.
- A builder can find the old iOS/MLX, tunnel, or literal airplane-mode pattern
  without asking a maintainer.
- No current docs overclaim phone-local inference, HIPAA compliance, or radio-off
  proof.
- Every deprecated path has a replacement and a GitOps revival recipe.
