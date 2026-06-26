# gates/ — the harness (ethics-as-code, law-as-code)

Every ethical, legal, and security requirement is an automated check that gates the loop. A principle that can't become a gate stays **prose**, and we say so (see `files/harnessed-loop.md`). Run via `./run.sh gates`. Each **blocks** on failure — never disable one to make a task pass (AGENTS.md Hard Rule).

| Gate | Encodes | Live at |
|---|---|---|
| `recall` | MHMDA de-id (Safe Harbor) · Const. II — recall ≥ threshold on golden notes | M1 |
| `leakage` | no-leak · Const. IX — zero residual identifiers post-gate (Slack **and** trajectory egress) | M1 |
| `pack-blindness` | trust boundary — pack has no code, no PHI access, creds sourced not stored | M1 |
| **`reward-lint` ★** | anti-dependence ethic · ADR-012 — reward references autonomy signals only, **no engagement terms** | M2 · live |
| **`scope-boundary` ★** | coach ≠ therapist — escalation path present, no clinical-claim language | M2 · live |
| `signature/provenance` | supply-chain trust — keyless metadata, provenance present | M4 · live |
| `manifest/revocation` | version integrity — signed monotonic manifest, current artifacts not revoked | M4 · live |
| `PHI-free-telemetry` | no backdoor — telemetry = version/health only, never content | M5+ |

★ = our contribution; no k8s/CNCF analog. The two ethical gates are the beat no other system can show.

`./run.sh gates` also runs `scripts/smoke-ethical-gate-fixtures.sh` after the
normal pack gates. The smoke creates temporary bad packs and proves the same gate
entrypoint rejects `usedSignals: [engagement]` and `escalationRequired: false`.

**Prose-only (NOT enforced — do not pretend otherwise):** autonomy-delta measurement, trajectory re-identification floor, "is the follow-up actually good?".
