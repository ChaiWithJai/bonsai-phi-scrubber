#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

blocked=(
  '^spike/'
  '^docs/deprecations/spikes/'
  '^docs/superpowers/'
  '^full-ctx\.md$'
  '^\.airplane/'
  '^\.playwright-mcp/'
  '^target/'
  '^output/'
  '^eval/last-run\.json$'
  '\.log$'
  '\.tmp$'
  '\.gguf$'
  '\.mlx$'
)

tracked="$(git ls-files)"
fail=0

for pattern in "${blocked[@]}"; do
  if printf '%s\n' "$tracked" | grep -Eq "$pattern"; then
    echo "public graph violation: tracked path matches $pattern" >&2
    printf '%s\n' "$tracked" | grep -E "$pattern" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  cat >&2 <<'EOF'

Raw research, spike scratch, generated run output, model weights, local certs,
and logs do not belong in the public starter graph. Promote the learning into a
current guide or decision record, or keep the artifact local/ignored.
EOF
  exit 1
fi

echo "public graph ok"
