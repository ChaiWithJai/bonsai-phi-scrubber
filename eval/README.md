# eval/ — the reproduction target

`golden-run.txt` is the committed expected eval output a stranger matches with
`./run.sh eval`. Refreshing it is explicit: run `./run.sh eval --update` only after
reviewing an intentional scoring change. The golden *notes* themselves live in the pack:
`packs/coach-session/eval/golden/` + `expected/` (authored at M0).

Determinism: fixed seeded model-card sampling, pinned model hash, pinned commit.
Matching this file is the proof.
