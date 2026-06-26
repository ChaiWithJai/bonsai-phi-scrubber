# DR-2026-06-26: Deprecated Pattern Index

Status: Accepted

## Decision

Deprecated patterns stay available as reference material, but they should not be
presented as the current demo path.

## Index

| Pattern | Status | Replacement | Revival use case |
| --- | --- | --- | --- |
| Public tunnel for scrub workflow | Deprecated | Local HTTPS or IT-managed VPN | Capability-only probes, never raw-note scrub traffic. |
| "HTTPS means HIPAA compliant" | Deprecated | Compliance requires scope, contracts, BAAs where applicable, and controls | Legal/compliance teaching example. |
| Literal radio-off airplane mode in web shell | Deferred | Simulated egress hold plus first-party network boundary | Native/on-device future proof. |
| Native iOS as the only core | Deprecated | Portable Rust core plus shell ports | Native product shell or MLX measurement spike. |
| HCL clinic pack | Deprecated | YAML/JSON `coach-session` pack | Historical comparison for pack DSL design. |
| Cityblock/clinic framing as default | Deprecated | Coaching/community-health synthetic workflow | Regulated-vertical adaptation after a new pack and legal review. |
| Full eval on every UI iteration | Deprecated | `gates-fast` during iteration; full gates before release | Release-hardening or model-change proof. |
| Landing/splash-first demo | Deprecated | Capture workflow first screen | Marketing site, not the product demo. |

## Revival Requirements

A deprecated pattern may be revived when:

- The revival has a branch or worktree.
- The decision record names the source commit and files.
- The revived path does not weaken the verifier gate.
- The README or UI copy makes the claim boundary explicit.
- Verification commands are recorded.

## Agent Instruction

When asked to "bring back" an old pattern, do not search randomly and rewrite the
current app. First use `git log -- <path>` or `git show <commit>:<path>`, then
create `revive/<pattern>` and make the smallest compatible patch.
