# GitOps Revival Runbook

Use this when you want to inspect, revive, or adapt an old Airplane Mode pattern
without disturbing the current demo path.

This runbook is for humans and agents. It deliberately avoids destructive Git
commands.

## Ground Rules

- Do not use `git reset --hard` to revive an old pattern.
- Do not overwrite current files unless you are on a new branch or worktree.
- Prefer `git show`, `git restore --source`, `git worktree`, and
  `git cherry-pick -n`.
- Run the relevant gates before claiming the revived path works.
- Keep secrets and `.airplane/` artifacts out of git.

## Find The Pattern

Search current docs and history:

```bash
rg -n "mlx|ios|airplane mode|Cloudflare|HCL|gpu|tunnel|pack" .
git log --oneline --all --decorate --max-count=80
git log --oneline --all -- <path>
```

Inspect a historical file without changing the worktree:

```bash
git show <commit>:<path>
git show <commit>:<path> | sed -n '1,160p'
```

Example:

```bash
git show 945760f:shells/ios/README.md
```

## Create A Safe Revival Branch

Use a branch when you want to experiment in the same checkout:

```bash
git switch -c revive/<pattern-name>
git restore --source <commit> -- <path>
git diff
```

Use a worktree when you want the old pattern side-by-side:

```bash
git worktree add ../airplane-revive-<pattern> <commit>
cd ../airplane-revive-<pattern>
```

Worktrees are best for comparing old UI/runtime code while the current demo keeps
running in the original checkout.

## Bring Back One File

Restore one historical file into your branch:

```bash
git restore --source <commit> -- files/demo-spec-airplane-mode.md
```

Rename it into the deprecations area if it is only reference material:

```bash
mkdir -p docs/deprecations/revived
git mv files/demo-spec-airplane-mode.md docs/deprecations/revived/demo-spec-airplane-mode.md
```

Then add a note at the top:

```text
Status: revived reference. Not the current demo claim.
Revived from: <commit>:files/demo-spec-airplane-mode.md
```

## Bring Back A Commit Without Committing It

Use `cherry-pick -n` to stage an old change for inspection:

```bash
git switch -c revive/<pattern-name>
git cherry-pick -n <commit>
git diff
```

If it is not the right patch:

```bash
git cherry-pick --abort
```

If conflicts appear, stop and decide whether the old pattern is still compatible
with the current trust boundary before resolving them.

## Verify The Revived Pattern

For docs-only revival:

```bash
git diff --check
```

For web/demo changes:

```bash
cargo test -p airplane-web
./run.sh gates-fast
AIRPLANE_WEB_URL=https://127.0.0.1:8443 ./run.sh browser-span-smoke
```

For core changes:

```bash
cargo test
./run.sh gates
```

Use full gates before release proof. Use `gates-fast` for iteration.

## Record The Revival

Add a short decision record:

```text
docs/deprecations/decision-records/YYYY-MM-DD-revive-<pattern>.md
```

Include:

- What was revived.
- Source commit and path.
- Why the current demo does not make this the default.
- What gates passed.
- What claims are allowed and disallowed.

## Useful Revival Anchors

| Pattern | Anchor | Notes |
| --- | --- | --- |
| iOS simulator backend selector | `945760f` | Useful for native-shell contract work; not real-device inference proof. |
| Browser GPU spike path | `2903800` through `4695492` | Useful for WebGPU/runtime work. Current main has the hardened version. |
| HTTPS phone setup | `7159093`, `21d376f` | Useful for local secure-context demos. |
| Proof/status page | `413379f`, `7a5134d` | Useful for demo telemetry and phone URL discovery. |
| Early positioning docs | `68ebe67`, `e30d58c` | Useful for audience/register work, not runtime behavior. |

These anchors are starting points, not permanent API contracts. Prefer
`git log -- <path>` when reviving a specific file.
