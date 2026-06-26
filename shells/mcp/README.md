# airplane-mcp — agent-callable shell

Thin stdio MCP-compatible adapter over the same `airplane-core` scrub + verifier gate
used by the CLI. It exposes one tool:

- `airplane_scrub` with input `{ "text": "...", "passes": 3 }`

The tool returns only gate-clean de-identified output. It never returns the raw
redaction-map text; redaction metadata is entity/layer only. If the verifier gate blocks,
the tool response is an error and no scrubbed payload is emitted.

Run:

```bash
cargo run -q --bin airplane-mcp
```

For deterministic local protocol smoke tests of the protocol surface:

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' |
  cargo run -q --bin airplane-mcp
```

Calling `airplane_scrub` needs the same local model server as the CLI:
`./scripts/serve-model.sh`.

To prove the MCP shell matches the CLI on the same golden notes:

```bash
./scripts/smoke-mcp-cli-parity.sh
```

The smoke defaults to the first three golden notes so it stays quick during demos.
Run all notes with `MCP_PARITY_LIMIT=20 ./scripts/smoke-mcp-cli-parity.sh`.
