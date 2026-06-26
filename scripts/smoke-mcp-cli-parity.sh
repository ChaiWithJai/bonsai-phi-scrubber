#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

passes="${MCP_PARITY_PASSES:-3}"
limit="${MCP_PARITY_LIMIT:-3}"
pack_dir="${PACK:-packs/coach-session}"
golden_dir="$pack_dir/eval/golden"

cargo build -q --bin airplane --bin airplane-mcp

count=0
for note in "$golden_dir"/*.txt; do
  id="$(basename "$note" .txt)"
  text="$(cat "$note")"

  cli_scrubbed="$(
    target/debug/airplane scrub "$text" |
      sed -n 's/^scrubbed : //p'
  )"

  request="$(
    jq -cn \
      --arg text "$text" \
      --argjson passes "$passes" \
      '{jsonrpc:"2.0",id:1,method:"tools/call",params:{name:"airplane_scrub",arguments:{text:$text,passes:$passes}}}'
  )"
  mcp_scrubbed="$(
    printf '%s\n' "$request" |
      target/debug/airplane-mcp |
      jq -r '.result.content[0].text | fromjson | .scrubbed_text'
  )"

  if [[ "$cli_scrubbed" != "$mcp_scrubbed" ]]; then
    printf 'MCP parity FAIL %s\n' "$id" >&2
    printf 'CLI: %s\n' "$cli_scrubbed" >&2
    printf 'MCP: %s\n' "$mcp_scrubbed" >&2
    exit 1
  fi

  printf 'MCP parity PASS %s\n' "$id"
  count=$((count + 1))
  if [[ "$count" -ge "$limit" ]]; then
    break
  fi
done

printf 'MCP parity checked %s golden note(s) with %s pass(es)\n' "$count" "$passes"
