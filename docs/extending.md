# Extending — make it yours in five files (no fork)

The correctness- and legally-critical core (`airplane-core`) is built and gated **once**. To run it for *your* practice, vertical, or use case, you write a **pack** — five declarative files and a set of golden notes. **No code, no fork.** A pack can't see raw input, the redaction map, or the verifier — so it's safe to write and safe to share.

This is the second conversion beat: *same signed core, your identifiers, no fork.*

## The five files

Copy the reference pack and edit it:

```bash
cp -r packs/coach-session packs/my-pack
```

| File | You change | Example |
|---|---|---|
| `recognizers/*.json` | the identifiers specific to you (member-ID formats, partner orgs, internal codes) | a `CM-\d{6}` member ID; a deny-list of partner clinics |
| `schema.yaml` | the shape of the record you want | fields: `client_pseudonym`, `themes[]`, `commitments[]`, `next_touch` |
| `policy.yaml` | what to redact + the recall bar + the reward rules | `profile: safe-harbor`, `recallThreshold: 0.99`, forbidden reward terms |
| `sink.yaml` | where clean records go (credential **sourced**, never stored) | Slack channel + `source: keychain` |
| `eval/golden/*.txt` + `eval/expected/*.json` | ~20 of **your** note shapes, hand-labeled | a note in your domain + the identifiers that must be caught |

### 1. Recognizers — your identifiers

Add a structured identifier as a Presidio-style recognizer (`recognizers/members.json`):

```json
{
  "name": "my_member_id",
  "supported_entity": "MEMBER_ID",
  "patterns": [{ "name": "id", "regex": "\\bMX-[0-9]{5}\\b", "score": 0.9 }],
  "context": ["member", "client id"]
}
```

Contextual identifiers (names in prose, relationships, dates-in-words) are caught by the **Bonsai** layer, not regex — you don't list those; the model handles them. Recognizers are only for the structured, practice-known formats.

### 2–4. Schema / policy / sink

These are YAML. Start from `packs/coach-session/{schema,policy,sink}.yaml` (the canonical shapes are in `files/rfc-002-final-ship.md` §4) and change field names, the recall threshold, and the destination.

### 5. Golden notes — prove it doesn't leak

Write ~20 synthetic notes in **your** domain under `eval/golden/note-NN.txt`, and the expected redactions under `eval/expected/note-NN.json`:

```json
{ "id": "note-01", "clean": false,
  "redactions": [ { "text": "Jordan Lee", "entity": "PERSON", "hard": true },
                  { "text": "MX-40231", "entity": "MEMBER_ID", "hard": false } ] }
```

`hard: true` marks identifiers a regex would miss (names in prose, dates in words) — the ones that prove the model layer earns its place. **Synthetic only — never a real note.**

## Ship it through the gate

A pack is only as trustworthy as its proven recall on its own golden notes. The gate enforces that automatically:

```bash
PACK=packs/my-pack ./run.sh eval     # recall / leakage on your golden set
PACK=packs/my-pack ./run.sh gates    # recall · leakage · pack-blindness
```

If recall clears your threshold and nothing leaks, you're running the identical signed core with your identifiers — and you never touched a line of the code that matters. `PACK=` is honored by the CLI, web shell, MCP shell, and parity smoke; if the path is missing or invalid, pack loading fails closed.
For custom packs, `PACK=... ./run.sh eval` prints that pack's metrics without overwriting the repo-level reference target. `./run.sh eval --update` is reserved for intentionally refreshing `eval/golden-run.txt` for the default pack.

## Why this is safe to open up

The core is reviewed and gated once; the pack is declarative and PHI-blind. A pack can redefine *what an identifier is* and *where clean records go* — it can never see raw input, read the redaction map, or change the verifier. So anyone may write a pack, and the trust boundary still holds. (ADR-005; `files/clinic-pack-pattern.md`.)
