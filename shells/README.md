# shells/ — the adapters (one core, many runtimes)

Each shell is a thin adapter that injects concrete implementations of the four `airplane-core` ports. The core is identical across all of them — that's what makes a reproduced recall number meaningful and the value-migration story honest.

| Shell | Built at | `InferenceProvider` | `SecureStore` | `Capture` | `Sink` | Proves |
|---|---|---|---|---|---|---|
| `cli/` | **M1** | llama-server HTTP (grammar server-side) | encrypted file | stdin/arg | mock/stdout/Slack | reproduction (Tier 1) |
| `ios/` | M3 | mlx-swift, MLX weights (grammar client-side) | Secure Enclave | ASR + UI | Slack Block Kit | planned hardware proof; current scaffold is simulator-only |
| `mcp/` | **M4** | llama-server HTTP (shared w/ CLI) | ephemeral/file | MCP tool-call | gate-clean tool-result | "an agent can do it too" |

**North-star (architected, not built in v1):** `server/`, `wasm/`. The port traits are the proof you could add them — the README shows how. Don't scaffold them speculatively (Hard Rule 7).

Build order: `cli` (M1) **before** `ios` (M3). `mcp` (M4) is a thin transport over the same core API the CLI already calls; it returns no raw redaction-map text and blocks tool output if the verifier gate fails.
