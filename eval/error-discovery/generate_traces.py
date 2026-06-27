"""Generate eval trace JSONL from PHI Scrubber golden set.

Since we can't run the model on this VM, we reconstruct traces from:
- packs/coach-session/eval/golden/note-{01..21}.txt  (original notes)
- packs/coach-session/eval/expected/note-{01..21}.json (expected redactions)
- eval/golden-run.txt (100% recall — all 71 labels caught, 68 over-redactions)

The golden run tells us every expected label WAS caught. So we mark all expected
redactions as caught and leave over-redactions as unknown (we don't have the
model's raw predictions without running it). The review app will show the
original note with expected labels highlighted so a human can discover:
1. Labels that SHOULD be in the expected set but aren't (the error discovery)
2. Categories of PHI the current expected set doesn't cover
3. Patterns that would be fragile under model variation
"""

import json
import os
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
GOLDEN_DIR = REPO / "packs" / "coach-session" / "eval" / "golden"
EXPECTED_DIR = REPO / "packs" / "coach-session" / "eval" / "expected"
OUT = Path(__file__).resolve().parent / "traces.jsonl"


def load_note(note_id: str) -> str:
    path = GOLDEN_DIR / f"{note_id}.txt"
    return path.read_text().strip()


def load_expected(note_id: str) -> dict:
    path = EXPECTED_DIR / f"{note_id}.json"
    return json.loads(path.read_text())


def find_span_position(text: str, target: str) -> tuple[int, int]:
    lower_text = text.lower()
    lower_target = target.lower()
    start = lower_text.find(lower_target)
    if start == -1:
        return -1, -1
    return start, start + len(target)


def generate():
    records = []
    for i in range(1, 22):
        note_id = f"note-{i:02d}"
        original = load_note(note_id)
        expected = load_expected(note_id)

        redactions_with_positions = []
        for r in expected.get("redactions", []):
            start, end = find_span_position(original, r["text"])
            redactions_with_positions.append({
                "text": r["text"],
                "entity": r["entity"],
                "hard": r.get("hard", False),
                "start": start,
                "end": end,
            })

        entity_types = list(set(r["entity"] for r in expected.get("redactions", [])))
        hard_count = sum(1 for r in expected.get("redactions", []) if r.get("hard"))
        total_count = len(expected.get("redactions", []))

        record = {
            "id": note_id,
            "original_text": original,
            "clean": expected.get("clean", False),
            "expected_redactions": redactions_with_positions,
            "entity_types": entity_types,
            "hard_count": hard_count,
            "total_labels": total_count,
            "note_length": len(original),
            "gate_decision": "PASS",
            "golden_run_status": "all_caught" if not expected.get("clean") else "clean_note",
        }
        records.append(record)

    with open(OUT, "w") as f:
        for r in records:
            f.write(json.dumps(r) + "\n")

    print(f"Generated {len(records)} trace records -> {OUT}")

    total_labels = sum(r["total_labels"] for r in records)
    hard_labels = sum(r["hard_count"] for r in records)
    clean_notes = sum(1 for r in records if r["clean"])
    entity_types = set()
    for r in records:
        entity_types.update(r["entity_types"])

    print(f"  Total labels: {total_labels}")
    print(f"  Hard labels: {hard_labels}")
    print(f"  Clean notes (no PHI): {clean_notes}")
    print(f"  Entity types: {sorted(entity_types)}")


if __name__ == "__main__":
    generate()
