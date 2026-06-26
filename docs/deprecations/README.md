# Deprecations And Decision Records

This folder keeps the demo clean without erasing useful ecosystem history.

Airplane Mode now has a core demo path:

```text
phone browser -> first-party edge laptop -> verifier gate -> Slack
```

Browser GPU is the preferred phone-local inference spike path. The laptop edge
path remains the reliable demo path. Native iOS/MLX, literal radio-off airplane
mode, public tunnels for the scrub app, and older clinic/HCL pack language are
kept as historical patterns or future work, not as the mainline demo.

## What Belongs Here

- Decision records for why a pattern is current, deferred, deprecated, or kept
  only as reference material.
- Cleanup proposals that say what the public demo should foreground.
- GitOps revival instructions so builders can recover an old pattern for their
  own use case without rewriting current history.

## Status Words

| Status | Meaning |
| --- | --- |
| Current | Supported by the core demo path and normal verification. |
| Deferred | Valuable, but not proven or not needed for the current demo. |
| Deprecated | Superseded by a safer/simpler path; keep for archaeology only. |
| Reference | Useful background, not an implementation promise. |

## Start Here

- [Demo cleanup proposal](demo-cleanup-proposal.md)
- [GitOps revival runbook](gitops-revival-runbook.md)
- [Decision records](decision-records/README.md)

## Rule For Agents

Do not delete or move old patterns just because they are deprecated. First add or
update a decision record, point to the replacement, and include a revival path.
Then make a small cleanup PR/commit that can be reviewed independently.
