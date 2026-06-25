#!/usr/bin/env python3
"""
PHI Scrubber Feasibility Spike
==============================

THROWAWAY harness. Python 3 standard library ONLY (urllib, no pip).

Purpose
-------
Measure whether `rules UNION Bonsai-1.7B` catches PHI in synthetic coaching
notes -- especially the HARD cases (names in prose, dates in words) that the
regex rules layer alone misses. This validates the core product claim before
we invest in the real Rust core.

Layers
------
1. Rules layer   -- deterministic regex for structured identifiers.
2. Bonsai layer  -- LLM extraction via a local OpenAI-compatible llama-server.
3. Score         -- union the two, compare against gold labels, report recall,
                    leakage, an over-redaction (precision proxy) count,
                    per-entity recall, and hard-case recall w/ which layer caught.

Data
----
Reads pairs from spike/notes/:  note-NN.txt + note-NN.labels.json
labels schema:
  {"id", "clean": bool,
   "redactions": [{"text","entity","hard": bool,"why"}]}

CLI
---
  python3 spike.py --rules-only                  # baseline: rules only
  python3 spike.py --full                         # rules UNION bonsai (needs server)
  python3 spike.py --determinism note-03 [--n 5]  # repeat-call stability probe
  python3 spike.py                                # prints usage

Output: a readable table to stdout AND a machine-readable spike/last-run.json.
"""

import json
import os
import re
import sys
import urllib.request
import urllib.error

# --------------------------------------------------------------------------- #
# Paths / config
# --------------------------------------------------------------------------- #
HERE = os.path.dirname(os.path.abspath(__file__))
NOTES_DIR = os.path.join(HERE, "notes")
LAST_RUN_PATH = os.path.join(HERE, "last-run.json")

BONSAI_URL = "http://127.0.0.1:8080/v1/chat/completions"
BONSAI_MODEL = "ternary-bonsai-1.7b"
SERVE_HINT = "start llama-server first: ~/projects/bonsai/scripts/serve.sh"

SYSTEM_PROMPT = (
    "You are a PHI de-identification engine. Extract every identifier -- "
    "names, dates, locations, relationships, ids, contact info -- from the "
    "note. Respond with JSON only."
)

# response_format json_schema. ASSUMPTION about shape: the model returns an
# object with a single "spans" array, each element {text, entity}. We keep the
# schema strict so a compliant server is forced into exactly that shape.
RESPONSE_FORMAT = {
    "type": "json_schema",
    "json_schema": {
        "name": "redactions",
        "strict": True,
        "schema": {
            "type": "object",
            "properties": {
                "spans": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "text": {"type": "string"},
                            "entity": {"type": "string"},
                        },
                        "required": ["text", "entity"],
                        "additionalProperties": False,
                    },
                }
            },
            "required": ["spans"],
            "additionalProperties": False,
        },
    },
}


# --------------------------------------------------------------------------- #
# Layer 1: Rules (regex)
# --------------------------------------------------------------------------- #
# Each pattern -> entity label. Order matters only for readability; we collect
# all matches. MRN-ish is handled separately because it needs context gating to
# avoid swallowing every 6-10 digit run (which would tank precision).
_RULES = [
    # SSN: 123-45-6789  (also tolerate spaces)
    (r"\b\d{3}[-\s]\d{2}[-\s]\d{4}\b", "SSN"),
    # US phone: (123) 456-7890 / 123-456-7890 / 123.456.7890 / +1 ...
    (r"(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]\d{3}[-.\s]\d{4}\b", "PHONE"),
    # Email
    (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "EMAIL"),
    # Member id  CM-204815
    (r"\bCM-\d{6}\b", "MEMBER_ID"),
    # ZIP (5 or ZIP+4)
    (r"\b\d{5}(?:-\d{4})?\b", "ZIP"),
    # URL
    (r"\bhttps?://[^\s)>\]]+", "URL"),
    # IPv4
    (r"\b(?:\d{1,3}\.){3}\d{1,3}\b", "IPV4"),
    # Numeric date: 1/2/24, 01/02/2024
    (r"\b\d{1,2}/\d{1,2}/\d{2,4}\b", "DATE"),
    # ISO date: 2024-01-02
    (r"\b\d{4}-\d{2}-\d{2}\b", "DATE"),
]

# MRN-ish: a 6-10 digit run, but only when nearby context says "MRN"/"record".
_MRN_CONTEXT = re.compile(
    r"(?:\bMRN\b|medical record|record (?:no|number|#))[^\n]{0,20}?(\b\d{6,10}\b)",
    re.IGNORECASE,
)


def rules_layer(text):
    """Return list of {text, entity, layer:'rules'} for regex hits."""
    out = []
    seen = set()  # (lowered_text, entity) to avoid intra-layer dupes

    def add(matched, entity):
        key = (matched.lower(), entity)
        if key not in seen:
            seen.add(key)
            out.append({"text": matched, "entity": entity, "layer": "rules"})

    for pattern, entity in _RULES:
        for m in re.finditer(pattern, text):
            add(m.group(0), entity)

    for m in _MRN_CONTEXT.finditer(text):
        add(m.group(1), "MRN")

    return out


# --------------------------------------------------------------------------- #
# Layer 2: Bonsai (LLM via local llama-server)
# --------------------------------------------------------------------------- #
def _strip_model_wrapping(content):
    """Strip a leading <think>...</think> block and ``` code fences."""
    if content is None:
        return ""
    s = content.strip()
    # Drop a leading <think>...</think> reasoning block if present.
    s = re.sub(r"^\s*<think>.*?</think>\s*", "", s, flags=re.DOTALL | re.IGNORECASE)
    # Strip ``` / ```json fences anywhere they bracket the payload.
    s = re.sub(r"^\s*```(?:json)?\s*", "", s)
    s = re.sub(r"\s*```\s*$", "", s)
    return s.strip()


def bonsai_layer(text, timeout=120):
    """
    POST the note to the local OpenAI-compatible server and parse identifiers.

    Returns (spans, error) where:
      spans  -> list of {text, entity, layer:'bonsai'}
      error  -> None on success, else a short error string (never raises).
    """
    payload = {
        "model": BONSAI_MODEL,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": text},
        ],
        "temperature": 0.5,
        "top_k": 20,
        "top_p": 0.85,
        "max_tokens": 512,
        "response_format": RESPONSE_FORMAT,
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        BONSAI_URL,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            body = resp.read().decode("utf-8")
    except urllib.error.URLError as e:
        # Connection refused -> server almost certainly not running.
        reason = getattr(e, "reason", e)
        return [], "server unreachable (%s) -- %s" % (reason, SERVE_HINT)
    except Exception as e:  # noqa: BLE001 - spike must never crash
        return [], "request failed: %r" % (e,)

    try:
        envelope = json.loads(body)
        content = envelope["choices"][0]["message"]["content"]
        cleaned = _strip_model_wrapping(content)
        parsed = json.loads(cleaned)
        spans_raw = parsed.get("spans", []) if isinstance(parsed, dict) else []
    except Exception as e:  # noqa: BLE001
        return [], "parse error: %r" % (e,)

    spans = []
    for sp in spans_raw:
        if not isinstance(sp, dict):
            continue
        t = sp.get("text")
        if not t:
            continue
        spans.append(
            {"text": t, "entity": sp.get("entity", "UNKNOWN"), "layer": "bonsai"}
        )
    return spans, None


# --------------------------------------------------------------------------- #
# Layer 3: Score
# --------------------------------------------------------------------------- #
def _union_dedupe(*span_lists):
    """Union spans across layers; dedupe by lowercased text.

    Keeps a record of which layers contributed each surviving span so we can
    later attribute a hard-case catch to rules / bonsai / both.
    """
    by_text = {}  # lowered text -> {text, entity, layers:set}
    for spans in span_lists:
        for sp in spans:
            key = sp["text"].lower().strip()
            if not key:
                continue
            if key not in by_text:
                by_text[key] = {
                    "text": sp["text"],
                    "entity": sp["entity"],
                    "layers": set(),
                }
            by_text[key]["layers"].add(sp["layer"])
    return list(by_text.values())


def _substr_match(label_text, pred_text):
    """Case-insensitive substring match in EITHER direction."""
    a = label_text.lower().strip()
    b = pred_text.lower().strip()
    if not a or not b:
        return False
    return a in b or b in a


def _layers_catching(label_text, predicted):
    """Return the set of layers among predicted spans that match label_text."""
    layers = set()
    for sp in predicted:
        if _substr_match(label_text, sp["text"]):
            layers |= sp["layers"]
    return layers


def score(records, use_bonsai):
    """
    records: list of dicts with keys:
        id, labels (the parsed labels json), text, rules (spans), bonsai (spans),
        bonsai_error (str|None)

    Returns a results dict (also what we serialize to last-run.json).
    """
    total_labels = 0
    caught_labels = 0
    leakage = []  # missed labels
    over_redactions = 0  # predicted spans overlapping NO label (precision proxy)

    # per-entity tallies: entity -> [caught, total]
    per_entity = {}
    # hard-case tallies
    hard_total = 0
    hard_caught = 0
    hard_layer_breakdown = {"rules": 0, "bonsai": 0, "both": 0}
    hard_caught_detail = []
    hard_missed_detail = []

    any_bonsai_error = None

    for rec in records:
        if rec.get("bonsai_error"):
            any_bonsai_error = rec["bonsai_error"]

        predicted = _union_dedupe(
            rec["rules"], rec["bonsai"] if use_bonsai else []
        )
        labels = rec["labels"].get("redactions", [])

        # --- recall + leakage + per-entity + hard cases ---
        matched_pred_keys = set()
        for lab in labels:
            total_labels += 1
            ent = lab.get("entity", "UNKNOWN")
            per_entity.setdefault(ent, [0, 0])
            per_entity[ent][1] += 1

            is_hard = bool(lab.get("hard"))
            if is_hard:
                hard_total += 1

            catching = _layers_catching(lab["text"], predicted)
            if catching:
                caught_labels += 1
                per_entity[ent][0] += 1
                # record which predicted spans were "used"
                for sp in predicted:
                    if _substr_match(lab["text"], sp["text"]):
                        matched_pred_keys.add(sp["text"].lower().strip())
                if is_hard:
                    hard_caught += 1
                    if "rules" in catching and "bonsai" in catching:
                        bucket = "both"
                    elif "rules" in catching:
                        bucket = "rules"
                    elif "bonsai" in catching:
                        bucket = "bonsai"
                    else:
                        bucket = "both"  # shouldn't happen
                    hard_layer_breakdown[bucket] += 1
                    hard_caught_detail.append(
                        {
                            "id": rec["id"],
                            "text": lab["text"],
                            "entity": ent,
                            "caught_by": bucket,
                        }
                    )
            else:
                leakage.append(
                    {
                        "id": rec["id"],
                        "text": lab["text"],
                        "entity": ent,
                        "hard": is_hard,
                        "why": lab.get("why", ""),
                    }
                )
                if is_hard:
                    hard_missed_detail.append(
                        {"id": rec["id"], "text": lab["text"], "entity": ent}
                    )

        # --- over-redaction: predicted spans that matched no label ---
        for sp in predicted:
            key = sp["text"].lower().strip()
            if key in matched_pred_keys:
                continue
            matched_any = any(
                _substr_match(lab["text"], sp["text"]) for lab in labels
            )
            if not matched_any:
                over_redactions += 1

    recall = (caught_labels / total_labels) if total_labels else 0.0
    hard_recall = (hard_caught / hard_total) if hard_total else 0.0

    per_entity_recall = {
        ent: {
            "caught": c,
            "total": t,
            "recall": round(c / t, 4) if t else 0.0,
        }
        for ent, (c, t) in sorted(per_entity.items())
    }

    return {
        "mode": "full" if use_bonsai else "rules-only",
        "notes_scored": len(records),
        "total_labels": total_labels,
        "caught_labels": caught_labels,
        "recall": round(recall, 4),
        "leakage_count": len(leakage),
        "leakage": leakage,
        "over_redactions": over_redactions,
        "per_entity_recall": per_entity_recall,
        "hard_cases": {
            "total": hard_total,
            "caught": hard_caught,
            "recall": round(hard_recall, 4),
            "layer_breakdown": hard_layer_breakdown,
            "caught_detail": hard_caught_detail,
            "missed_detail": hard_missed_detail,
        },
        "bonsai_error": any_bonsai_error,
    }


# --------------------------------------------------------------------------- #
# Data loading
# --------------------------------------------------------------------------- #
def load_notes():
    """Load all note-NN.txt / note-NN.labels.json pairs, sorted by id."""
    pairs = []
    if not os.path.isdir(NOTES_DIR):
        return pairs
    for fn in sorted(os.listdir(NOTES_DIR)):
        if not fn.endswith(".labels.json"):
            continue
        note_id = fn[: -len(".labels.json")]
        txt_path = os.path.join(NOTES_DIR, note_id + ".txt")
        lab_path = os.path.join(NOTES_DIR, fn)
        if not os.path.exists(txt_path):
            continue
        with open(txt_path, "r", encoding="utf-8") as f:
            text = f.read()
        with open(lab_path, "r", encoding="utf-8") as f:
            labels = json.load(f)
        pairs.append({"id": note_id, "text": text, "labels": labels})
    return pairs


def build_records(notes, use_bonsai):
    """Run the enabled layers over each note and assemble scoring records."""
    records = []
    for n in notes:
        rules = rules_layer(n["text"])
        bonsai, berr = ([], None)
        if use_bonsai:
            bonsai, berr = bonsai_layer(n["text"])
        records.append(
            {
                "id": n["id"],
                "text": n["text"],
                "labels": n["labels"],
                "rules": rules,
                "bonsai": bonsai,
                "bonsai_error": berr,
            }
        )
    return records


# --------------------------------------------------------------------------- #
# Output / reporting
# --------------------------------------------------------------------------- #
def print_report(results):
    R = results
    print("=" * 64)
    print("PHI SCRUBBER SPIKE  --  mode: %s" % R["mode"])
    print("=" * 64)
    print("notes scored      : %d" % R["notes_scored"])
    print(
        "RECALL            : %.1f%%  (%d/%d labels caught)"
        % (R["recall"] * 100, R["caught_labels"], R["total_labels"])
    )
    print("leakage (missed)  : %d" % R["leakage_count"])
    print(
        "over-redactions   : %d  (precision proxy -- not over-weighted)"
        % R["over_redactions"]
    )
    print()

    print("per-entity recall:")
    print("  %-14s %6s %6s %8s" % ("ENTITY", "caught", "total", "recall"))
    for ent, d in R["per_entity_recall"].items():
        print(
            "  %-14s %6d %6d %7.1f%%"
            % (ent, d["caught"], d["total"], d["recall"] * 100)
        )
    print()

    hc = R["hard_cases"]
    print(
        "HARD-CASE recall  : %.1f%%  (%d/%d)"
        % (hc["recall"] * 100, hc["caught"], hc["total"])
    )
    lb = hc["layer_breakdown"]
    print(
        "  caught by -> rules: %d   bonsai: %d   both: %d"
        % (lb["rules"], lb["bonsai"], lb["both"])
    )
    if hc["caught_detail"]:
        print("  hard caught:")
        for d in hc["caught_detail"]:
            print(
                "    [%s] %-32s (%s) <- %s"
                % (d["id"], _trunc(d["text"], 32), d["entity"], d["caught_by"])
            )
    if hc["missed_detail"]:
        print("  hard MISSED:")
        for d in hc["missed_detail"]:
            print("    [%s] %-32s (%s)" % (d["id"], _trunc(d["text"], 32), d["entity"]))
    print()

    if R["leakage"]:
        print("LEAKAGE detail (missed labels):")
        for lk in R["leakage"]:
            flag = "HARD" if lk["hard"] else "soft"
            print(
                "  [%s] %-30s (%s, %s)  %s"
                % (lk["id"], _trunc(lk["text"], 30), lk["entity"], flag, lk["why"])
            )
        print()

    if R.get("bonsai_error"):
        print("!! bonsai layer error: %s" % R["bonsai_error"])
        print()


def _trunc(s, n):
    s = s.replace("\n", " ")
    return s if len(s) <= n else s[: n - 1] + "…"


def write_last_run(results):
    with open(LAST_RUN_PATH, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print("wrote %s" % LAST_RUN_PATH)


# --------------------------------------------------------------------------- #
# Determinism probe
# --------------------------------------------------------------------------- #
def determinism_probe(note_id, n):
    """Call the bonsai layer n times on one note; report span-set stability."""
    notes = load_notes()
    match = [x for x in notes if x["id"] == note_id]
    if not match:
        print("no such note: %s" % note_id)
        print("available: %s" % ", ".join(x["id"] for x in notes))
        return 2
    note = match[0]

    print("determinism probe: %s  (n=%d, sampling temp=0.5/top_k=20/top_p=0.85)"
          % (note_id, n))
    signatures = []
    for i in range(n):
        spans, err = bonsai_layer(note["text"])
        if err:
            print("  run %d: ERROR -- %s" % (i + 1, err))
            print("\nVERDICT: INCONCLUSIVE (server error)")
            print(SERVE_HINT)
            return 1
        # canonical signature: sorted lowercased text|entity tuples
        sig = sorted("%s|%s" % (s["text"].lower().strip(), s["entity"]) for s in spans)
        signatures.append(sig)
        print("  run %d: %d spans" % (i + 1, len(spans)))

    identical = all(s == signatures[0] for s in signatures)
    verdict = "STABLE" if identical else "VARIES"
    print("\nVERDICT: %s  (span sets %s across %d runs)"
          % (verdict, "byte-identical" if identical else "DIFFER", n))

    out = {
        "mode": "determinism",
        "note_id": note_id,
        "runs": n,
        "verdict": verdict,
        "signatures": signatures,
    }
    write_last_run(out)
    return 0


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
USAGE = """\
PHI scrubber feasibility spike

usage:
  python3 spike.py --rules-only
      Score with the rules (regex) layer only -- the baseline.

  python3 spike.py --full
      Score with rules UNION bonsai. Requires the local llama-server:
      %s

  python3 spike.py --determinism note-03 [--n 5]
      Call the bonsai layer N times on one note at the configured sampling
      and report whether the returned span sets are byte-identical
      (verdict: STABLE / VARIES).
""" % SERVE_HINT


def run_scoring(use_bonsai):
    notes = load_notes()
    if not notes:
        print("no notes found in %s" % NOTES_DIR)
        return 2
    records = build_records(notes, use_bonsai)
    results = score(records, use_bonsai)
    print_report(results)
    write_last_run(results)
    # Non-zero exit if full mode but the server never answered.
    if use_bonsai and results.get("bonsai_error"):
        return 1
    return 0


def main(argv):
    args = argv[1:]
    if not args:
        print(USAGE)
        return 0

    if "--rules-only" in args:
        return run_scoring(use_bonsai=False)

    if "--full" in args:
        return run_scoring(use_bonsai=True)

    if "--determinism" in args:
        idx = args.index("--determinism")
        if idx + 1 >= len(args):
            print("--determinism requires a note id, e.g. --determinism note-03")
            return 2
        note_id = args[idx + 1]
        n = 5
        if "--n" in args:
            try:
                n = int(args[args.index("--n") + 1])
            except (ValueError, IndexError):
                print("--n requires an integer")
                return 2
        return determinism_probe(note_id, n)

    print(USAGE)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
