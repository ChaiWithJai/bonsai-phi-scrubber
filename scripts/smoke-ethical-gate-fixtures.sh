#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

bin="target/debug/airplane"
if [ "${1:-}" = "--bin" ]; then
  bin="${2:?missing binary path}"
else
  cargo build -q --bin airplane
fi

tmp="$(mktemp -d "${TMPDIR:-/tmp}/airplane-ethical-gates.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

copy_pack() {
  local name="$1"
  local dir="$tmp/$name"
  cp -R packs/coach-session "$dir"
  printf '%s\n' "$dir"
}

run_must_fail() {
  local name="$1"
  local pack="$2"
  local expected="$3"
  local out status

  set +e
  out="$(PACK="$pack" "$bin" gates-fast 2>&1)"
  status=$?
  set -e

  if [ "$status" -eq 0 ]; then
    printf 'ethical fixture %s: expected failure, got success\n%s\n' "$name" "$out" >&2
    exit 1
  fi
  if ! grep -Fq "$expected" <<<"$out"; then
    printf 'ethical fixture %s: missing expected output %q\n%s\n' "$name" "$expected" "$out" >&2
    exit 1
  fi
  printf 'ethical fixture %-15s: PASS\n' "$name"
}

reward_pack="$(copy_pack bad-reward)"
python3 - "$reward_pack/policy.yaml" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
needle = "    autonomySignals: [commitment_completed, self_initiated, reduced_prompt_need]\n"
replacement = needle + "    usedSignals: [engagement]\n"
if needle not in text:
    raise SystemExit("policy autonomySignals line not found")
path.write_text(text.replace(needle, replacement, 1))
PY

scope_pack="$(copy_pack bad-scope)"
python3 - "$scope_pack/policy.yaml" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
needle = "    escalationRequired: true\n"
replacement = "    escalationRequired: false\n"
if needle not in text:
    raise SystemExit("policy escalationRequired line not found")
path.write_text(text.replace(needle, replacement, 1))
PY

run_must_fail "reward-lint" "$reward_pack" "gate reward-lint    : FAIL"
run_must_fail "scope-boundary" "$scope_pack" "gate scope-boundary : FAIL"
