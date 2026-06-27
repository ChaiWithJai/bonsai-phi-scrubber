"""Systematic error-discovery analysis of all 21 golden notes.

For each note, this script:
1. Loads the original text and expected labels
2. Identifies PHI that IS labeled (known catches)
3. Identifies PHI that is NOT labeled (gaps in the expected set)
4. Categorizes failure modes
5. Produces a failure-modes.yaml

This is the ANALYZE phase of the Shreya framework — define mistakes,
find mistakes, create failure modes.
"""

import json
import re
import yaml
from pathlib import Path
from collections import defaultdict

REPO = Path(__file__).resolve().parent.parent.parent
GOLDEN_DIR = REPO / "packs" / "coach-session" / "eval" / "golden"
EXPECTED_DIR = REPO / "packs" / "coach-session" / "eval" / "expected"


def load_note(note_id):
    return (GOLDEN_DIR / f"{note_id}.txt").read_text().strip()


def load_expected(note_id):
    return json.loads((EXPECTED_DIR / f"{note_id}.json").read_text())


# --- PHI detection heuristics (rule-based scan for unlabeled PHI) ---

# Names that appear in notes but might not be in expected labels
# We'll check each note for pronoun-based identifying patterns
RELATIONSHIP_PATTERNS = [
    (r'\b(?:his|her|their)\s+(?:mother|father|brother|sister|son|daughter|wife|husband|partner|ex-husband|ex-wife|spouse|grandmother|grandfather|aunt|uncle|cousin)\b',
     'RELATIONSHIP_REFERENCE', 'Possessive pronoun + family role — could identify in context'),
    (r'\b(?:his|her|their)\s+(?:mom|dad)\b',
     'RELATIONSHIP_REFERENCE', 'Informal possessive + parent — could identify in context'),
]

TEMPORAL_PATTERNS = [
    (r'\b(?:last\s+week|yesterday|today|this\s+morning|this\s+afternoon|this\s+evening|tonight)\b',
     'COMMON_TEMPORAL', 'Common temporal — usually NOT identifying, but check'),
    (r'\b(?:this\s+past\s+\w+|the\s+week\s+before|the\s+day\s+after|the\s+Monday\s+before|the\s+first\s+weekend\s+after|the\s+second\s+Tuesday\s+of|the\s+last\s+day\s+of)\b',
     'RELATIVE_TEMPORAL', 'Relative temporal anchor — identifying if combined with context'),
    (r'\b(?:in\s+the\s+spring|this\s+fall|over\s+the\s+holidays|before\s+summer|this\s+year|each\s+year)\b',
     'SEASONAL_TEMPORAL', 'Seasonal reference — low identifying power alone'),
]

LOCATION_PATTERNS = [
    (r'\b(?:near\s+the\s+corner\s+of|just\s+off|on\s+\w+\s+Street|on\s+\w+\s+Road|on\s+\w+\s+Avenue|in\s+\w+dale|in\s+\w+ville)\b',
     'LOCATION_DESCRIPTION', 'Location description — check if labeled'),
    (r'\b(?:at\s+home|around\s+the\s+block)\b',
     'GENERIC_LOCATION', 'Generic location — usually NOT identifying'),
]

CLINICAL_PATTERNS = [
    (r'\b(?:custody\s+hearing|divorce|postpartum|panic\s+(?:episodes|attack)|burnout|grief|imposter|caregiving|retirement)\b',
     'CLINICAL_DETAIL', 'Clinical detail — sensitive but not PHI per se'),
    (r'\b(?:sliding-scale\s+rate|insurance\s+change|billing\s+cycle|new\s+program\s+code)\b',
     'ADMINISTRATIVE_DETAIL', 'Administrative detail — could be contextually identifying'),
]


def analyze_note(note_id):
    """Analyze a single note for labeled and unlabeled PHI."""
    text = load_note(note_id)
    expected = load_expected(note_id)

    findings = {
        'id': note_id,
        'clean': expected.get('clean', False),
        'text_length': len(text),
        'labeled_phi': [],
        'unlabeled_observations': [],
        'failure_mode_instances': [],
    }

    # Record labeled PHI
    for r in expected.get('redactions', []):
        findings['labeled_phi'].append({
            'text': r['text'],
            'entity': r['entity'],
            'hard': r.get('hard', False),
            'found_in_text': r['text'].lower() in text.lower(),
        })

    if expected.get('clean', False):
        # For clean notes, check if there really IS no PHI
        # Look for any names, emails, phones, etc.
        name_pattern = re.findall(r'\b[A-Z][a-z]+\b', text)
        common_words = {'The', 'We', 'Most', 'There', 'By', 'I'}
        unusual_caps = [w for w in name_pattern if w not in common_words
                       and len(w) > 2
                       and w not in ('She', 'He', 'Her', 'His', 'They')]
        if unusual_caps:
            findings['unlabeled_observations'].append({
                'observation': f'Clean note has capitalized words that could be names: {unusual_caps[:5]}',
                'severity': 'low',
                'type': 'clean_note_review',
            })
        return findings

    # --- Check for unlabeled PHI patterns ---

    labeled_texts = {r['text'].lower() for r in expected.get('redactions', [])}

    # 1. First-name-only analysis
    for r in expected.get('redactions', []):
        if r['entity'] == 'PERSON' and ' ' not in r['text']:
            findings['failure_mode_instances'].append({
                'mode': 'first_name_only',
                'text': r['text'],
                'note': note_id,
                'observation': f"Single first name '{r['text']}' — common names may be treated as regular words by the model",
            })

    # 2. Relationship references that might not be labeled
    for pattern, ptype, desc in RELATIONSHIP_PATTERNS:
        matches = re.finditer(pattern, text, re.IGNORECASE)
        for m in matches:
            matched = m.group(0)
            # Check if the broader context around this is labeled
            # Get surrounding context (20 chars before and after)
            start = max(0, m.start() - 5)
            end = min(len(text), m.end() + 40)
            context = text[start:end]
            is_labeled = any(label in context.lower() for label in labeled_texts
                           if len(label) > 5)
            if not is_labeled:
                findings['unlabeled_observations'].append({
                    'observation': f"Relationship reference '{matched}' in context: '...{context}...'",
                    'severity': 'medium',
                    'type': 'unlabeled_relationship',
                })

    # 3. Temporal references — check which are labeled vs not
    for pattern, ptype, desc in TEMPORAL_PATTERNS:
        matches = re.finditer(pattern, text, re.IGNORECASE)
        for m in matches:
            matched = m.group(0)
            is_labeled = any(matched.lower() in label for label in labeled_texts)
            if not is_labeled and ptype == 'RELATIVE_TEMPORAL':
                findings['failure_mode_instances'].append({
                    'mode': 'unlabeled_relative_temporal',
                    'text': matched,
                    'note': note_id,
                    'observation': f"Relative temporal '{matched}' not in expected labels — {desc}",
                })
            elif ptype == 'COMMON_TEMPORAL':
                findings['unlabeled_observations'].append({
                    'observation': f"Common temporal '{matched}' — should NOT be redacted (over-redaction risk)",
                    'severity': 'info',
                    'type': 'over_redaction_risk',
                })

    # 4. Check for emails/phones that might not be labeled
    emails = re.findall(r'[\w.+-]+@[\w-]+\.[\w.]+', text)
    for email in emails:
        email = email.rstrip('.')  # strip trailing period
        if email.lower() not in labeled_texts:
            findings['failure_mode_instances'].append({
                'mode': 'unlabeled_email',
                'text': email,
                'note': note_id,
                'observation': f"Email '{email}' not in expected labels",
            })

    phones = re.findall(r'(?:\(\d{3}\)\s*|\d{3}[-.])\d{3}[-.]?\d{4}', text)
    for phone in phones:
        if phone not in labeled_texts and phone.replace(' ', '') not in labeled_texts:
            findings['failure_mode_instances'].append({
                'mode': 'unlabeled_phone',
                'text': phone,
                'note': note_id,
                'observation': f"Phone '{phone}' not in expected labels",
            })

    # 5. Compound identifiers (name + institution + relationship)
    # Check if any FAMILY_DETAIL spans cover the full compound
    family_details = [r for r in expected.get('redactions', [])
                     if r['entity'] == 'FAMILY_DETAIL']
    if family_details:
        for fd in family_details:
            findings['failure_mode_instances'].append({
                'mode': 'compound_identifier',
                'text': fd['text'],
                'note': note_id,
                'observation': f"Compound identifier: '{fd['text']}' — combines relationship + context. Model must catch the FULL phrase, not just the name within it.",
            })

    # 6. Check for member IDs and benefit codes
    member_ids = re.findall(r'\b(?:CM|RC|BEN)[-\s]?\w+[-\s]?\w*\b', text)
    for mid in member_ids:
        mid_clean = mid.strip()
        if mid_clean.lower() not in labeled_texts and len(mid_clean) > 4:
            # Check more carefully
            if not any(mid_clean.lower() in label for label in labeled_texts):
                findings['unlabeled_observations'].append({
                    'observation': f"Possible unlabeled ID pattern: '{mid_clean}'",
                    'severity': 'high',
                    'type': 'unlabeled_id',
                })

    # 7. Nickname / alias patterns
    nickname_patterns = re.findall(
        r'(?:goes\s+by|prefers\s+(?:to\s+be\s+called|I\s+use))\s+(?:her\s+nickname\s+)?(\w+)',
        text, re.IGNORECASE
    )
    for nick in nickname_patterns:
        if nick.lower() not in labeled_texts:
            findings['failure_mode_instances'].append({
                'mode': 'unlabeled_nickname',
                'text': nick,
                'note': note_id,
                'observation': f"Nickname/alias '{nick}' not in expected labels",
            })

    # 8. Implicit identifiers — unique descriptors
    implicit_patterns = [
        (r'(?:the\s+only|the\s+first)\s+\w+\s+\w+\s+(?:at|in|who)', 'IMPLICIT_UNIQUE'),
        (r'who\s+(?:recently|just)\s+\w+\s+(?:to|into|from|at)', 'IMPLICIT_RECENT_ACTION'),
    ]
    for pattern, itype in implicit_patterns:
        matches = re.finditer(pattern, text, re.IGNORECASE)
        for m in matches:
            context = text[max(0, m.start()-10):min(len(text), m.end()+20)]
            findings['unlabeled_observations'].append({
                'observation': f"Implicit identifier pattern ({itype}): '...{context}...'",
                'severity': 'medium',
                'type': 'implicit_identifier',
            })

    return findings


def build_failure_mode_taxonomy(all_findings):
    """Build the failure mode taxonomy from all note analyses."""
    modes = defaultdict(lambda: {
        'description': '',
        'severity': '',
        'count': 0,
        'notes': [],
        'examples': [],
    })

    # Collect all failure mode instances
    for f in all_findings:
        for inst in f.get('failure_mode_instances', []):
            mode = inst['mode']
            modes[mode]['count'] += 1
            if f['id'] not in modes[mode]['notes']:
                modes[mode]['notes'].append(f['id'])
            modes[mode]['examples'].append(f"{inst['text']} ({inst['note']})")

    # Define descriptions and severities
    mode_defs = {
        'first_name_only': {
            'description': 'Single first name without surname — common names (Marcus, Kit, Cal, Devon) may be treated as regular words by the model. The model must learn that in a coaching note context, a capitalized word following a possessive or used as a subject is likely a person\'s name.',
            'severity': 'critical',
            'category': 'false_negative_risk',
        },
        'compound_identifier': {
            'description': 'Relationship + institution + timing combined into one identifying phrase (e.g., "his daughter who just started at Northwestern", "Her wife, who returned to work as a trauma surgeon at Gulfport General"). The model must catch the FULL compound, not just the name within it. Partial detection leaves re-identification possible.',
            'severity': 'critical',
            'category': 'detection_completeness',
        },
        'unlabeled_relative_temporal': {
            'description': 'Date expressed relative to an anchor ("this past Thanksgiving", "the Monday before Memorial Day", "the first weekend after New Year\'s", "the day after the autumn equinox"). These are identifying when combined with other context — if you know someone had a breakthrough "this past Thanksgiving" and they live on Maple Crest Avenue, the combination narrows identification significantly.',
            'severity': 'high',
            'category': 'contextual_identifier',
        },
        'unlabeled_email': {
            'description': 'Email address present in note text but not in expected labels. Emails are structured identifiers that the rules layer should catch deterministically.',
            'severity': 'critical',
            'category': 'label_gap',
        },
        'unlabeled_phone': {
            'description': 'Phone number present in note text but not in expected labels. Phones are structured identifiers that the rules layer should catch deterministically.',
            'severity': 'critical',
            'category': 'label_gap',
        },
        'unlabeled_nickname': {
            'description': 'Nickname or alias explicitly mentioned ("goes by Jamal", "prefers to be called Sunny", "prefers I use her nickname Kit") but not in expected labels as a separate PERSON entity.',
            'severity': 'high',
            'category': 'label_gap',
        },
    }

    taxonomy = {}
    for mode_name, data in sorted(modes.items(), key=lambda x: -x[1]['count']):
        defs = mode_defs.get(mode_name, {})
        taxonomy[mode_name] = {
            'description': defs.get('description', f'Auto-discovered failure mode: {mode_name}'),
            'severity': defs.get('severity', 'medium'),
            'category': defs.get('category', 'unknown'),
            'count': data['count'],
            'note_count': len(data['notes']),
            'notes': data['notes'],
            'examples': data['examples'][:10],
        }

    return taxonomy


def build_observed_patterns(all_findings):
    """Build observed patterns from unlabeled observations."""
    patterns = defaultdict(list)
    for f in all_findings:
        for obs in f.get('unlabeled_observations', []):
            patterns[obs['type']].append({
                'note': f['id'],
                'observation': obs['observation'],
                'severity': obs['severity'],
            })
    return dict(patterns)


def main():
    print("=" * 70)
    print("PHI SCRUBBER ERROR DISCOVERY — SYSTEMATIC ANALYSIS")
    print("=" * 70)
    print()

    all_findings = []

    for i in range(1, 22):
        note_id = f"note-{i:02d}"
        findings = analyze_note(note_id)
        all_findings.append(findings)

        labeled_count = len(findings['labeled_phi'])
        fm_count = len(findings['failure_mode_instances'])
        obs_count = len(findings['unlabeled_observations'])

        status = "CLEAN" if findings['clean'] else f"{labeled_count} labels"
        flags = []
        if fm_count > 0:
            flags.append(f"{fm_count} failure modes")
        if obs_count > 0:
            flags.append(f"{obs_count} observations")

        flag_str = f" | {', '.join(flags)}" if flags else ""
        print(f"  {note_id}: {status}{flag_str}")

        # Print detail for non-trivial findings
        for inst in findings['failure_mode_instances']:
            print(f"    [{inst['mode']}] {inst['observation']}")
        for obs in findings['unlabeled_observations']:
            if obs['severity'] in ('high', 'critical', 'medium'):
                print(f"    [{obs['type']}] {obs['observation']}")

    print()
    print("=" * 70)
    print("FAILURE MODE TAXONOMY")
    print("=" * 70)

    taxonomy = build_failure_mode_taxonomy(all_findings)
    for mode_name, data in taxonomy.items():
        print(f"\n  {mode_name} [{data['severity']}] — {data['count']} instances in {data['note_count']} notes")
        print(f"    {data['description'][:120]}...")
        for ex in data['examples'][:3]:
            print(f"    - {ex}")

    print()
    print("=" * 70)
    print("OBSERVED PATTERNS (unlabeled)")
    print("=" * 70)

    patterns = build_observed_patterns(all_findings)
    for ptype, items in patterns.items():
        print(f"\n  {ptype}: {len(items)} observations")
        for item in items[:3]:
            print(f"    [{item['note']}] {item['observation'][:100]}")

    # --- Now add manually-identified failure modes that the heuristics miss ---
    # These come from reading the notes carefully

    manual_modes = {
        'over_redaction_common_temporal': {
            'description': 'Common temporal phrases ("today", "last week", "this morning", "next week") that should NOT be redacted because they carry minimal identifying power. Over-redacting these reduces clinical utility of the scrubbed record. The model must learn the difference between "this past Thanksgiving" (identifying — anchors to a specific week) and "today" (not identifying — every note has a "today").',
            'severity': 'low',
            'category': 'over_redaction',
            'count': 14,
            'note_count': 12,
            'notes': ['note-01', 'note-02', 'note-03', 'note-04', 'note-05', 'note-06',
                      'note-07', 'note-08', 'note-09', 'note-11', 'note-17', 'note-18'],
            'examples': [
                'today (note-01, 02, 06, 08, 09)',
                'last week (note-01, 17)',
                'this fall (note-07)',
                'next week (note-05)',
                'this week (note-12, 14, 16)',
                'each morning (note-04)',
            ],
        },
        'possessive_relationship_without_name': {
            'description': 'Possessive pronoun + family role without an explicit name ("his mom", "her in-laws", "the kids", "his sister", "her mother", "aging parents"). These are identifying to different degrees — "his mom" is very low risk, but "her mother who recently moved into the memory-care wing at Lakeshore Manor" is high risk because the institution narrows the search. The expected labels handle some of these via FAMILY_DETAIL but not consistently.',
            'severity': 'high',
            'category': 'contextual_identifier',
            'count': 10,
            'note_count': 8,
            'notes': ['note-01', 'note-03', 'note-04', 'note-07', 'note-08',
                      'note-12', 'note-16', 'note-17'],
            'examples': [
                '"his mom" (note-01) — low risk, no context to narrow',
                '"her in-laws" (note-03) — medium risk with address on same note',
                '"the kids" (note-08) — low risk alone, higher with "divorce" context',
                '"her mother, who recently moved into the memory-care wing at Lakeshore Manor" (note-12) — HIGH, labeled as FAMILY_DETAIL',
                '"aging parents" (note-16) — medium risk with Asheville destination',
                '"the baby" (note-17) — low risk alone, but "postpartum" + "wife is trauma surgeon" narrows',
            ],
        },
        'clinical_detail_as_quasi_identifier': {
            'description': 'Clinical details that are not PHI per se but become quasi-identifiers in combination: "custody hearing" (note-13), "divorce" (note-08), "postpartum" (note-17), "panic episodes before shifts" (note-02). Under MHMDA/42 CFR Part 2, the fact that someone is receiving mental health treatment is itself protected. These are not in scope for the current scrubber (which focuses on direct identifiers), but a clinical expert should flag which combinations cross the re-identification threshold.',
            'severity': 'medium',
            'category': 'scope_boundary',
            'count': 8,
            'note_count': 7,
            'notes': ['note-02', 'note-04', 'note-08', 'note-09', 'note-13', 'note-15', 'note-17'],
            'examples': [
                '"custody hearing" (note-13) — legal proceeding is identifying in small communities',
                '"divorce" + "pottery class on Hawthorne Street" (note-08) — combination narrows',
                '"postpartum" + "wife is trauma surgeon at Gulfport General" (note-17) — high combination risk',
                '"imposter feelings back to her childhood in Cleveland" (note-09) — biographical detail',
            ],
        },
        'email_in_username_leaks_name': {
            'description': 'Email addresses that contain the person\'s real name in the username part: devon.k.harper@fastmail.com (leaks "Devon Harper"), carlos.mendez88@zoho.com (leaks "Carlos Mendez"), kit.delgado@gmail.com (leaks "Kit Delgado"), e.whitfield@protonmail.com (leaks "E. Whitfield"). Even if the scrubber catches the email as a unit, the name WITHIN the email must also be independently caught as a PERSON. If the email is caught but the name parsing fails, the name still appears elsewhere in the note.',
            'severity': 'high',
            'category': 'detection_completeness',
            'count': 4,
            'note_count': 4,
            'notes': ['note-02', 'note-05', 'note-09', 'note-13'],
            'examples': [
                'devon.k.harper@fastmail.com → "Devon Harper" (note-02)',
                'e.whitfield@protonmail.com → "E. Whitfield" (note-05)',
                'kit.delgado@gmail.com → "Kit Delgado" (note-09)',
                'carlos.mendez88@zoho.com → "Carlos Mendez" (note-13)',
            ],
        },
        'location_as_address_vs_region': {
            'description': 'The expected labels use both ADDRESS (specific street: "Maple Crest Avenue", "Hawthorne Street", "Juniper Bend Road", "corner of 4th and Birch in Glenndale") and LOCATION (city/region: "Cleveland", "Tacoma", "Asheville"). The distinction matters for severity — a street address is much more identifying than a city name. The model must learn this distinction, and the over-redaction risk is higher for common city names.',
            'severity': 'medium',
            'category': 'entity_granularity',
            'count': 7,
            'note_count': 7,
            'notes': ['note-03', 'note-04', 'note-07', 'note-08', 'note-09', 'note-15', 'note-16'],
            'examples': [
                'ADDRESS: "Maple Crest Avenue" (note-03) — street-level, high risk',
                'ADDRESS: "corner of 4th and Birch in Glenndale" (note-07) — intersection + city, very high',
                'LOCATION: "Cleveland" (note-09) — city, medium risk',
                'LOCATION: "Tacoma" (note-04) — city in FAMILY_DETAIL context, higher risk',
            ],
        },
    }

    taxonomy.update(manual_modes)

    # Write the full taxonomy to YAML
    out_path = Path(__file__).resolve().parent.parent.parent / "docs" / "eval-plan" / "failure-modes.yaml"

    yaml_data = {
        'metadata': {
            'generated': '2026-06-27',
            'source': 'error-discovery analysis of 21 golden coaching notes',
            'methodology': 'Systematic note-by-note review + heuristic pattern matching + manual clinical analysis',
            'total_notes': 21,
            'clean_notes': 3,
            'total_expected_labels': 71,
            'hard_labels': 55,
        },
        'failure_modes': {},
    }

    # Sort by severity (critical > high > medium > low)
    severity_order = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3}
    sorted_modes = sorted(taxonomy.items(),
                         key=lambda x: (severity_order.get(x[1]['severity'], 99), -x[1]['count']))

    for mode_name, data in sorted_modes:
        yaml_data['failure_modes'][mode_name] = {
            'description': data['description'],
            'severity': data['severity'],
            'category': data['category'],
            'prevalence': {
                'instance_count': data['count'],
                'note_count': data['note_count'],
                'notes': data['notes'],
            },
            'examples': data['examples'],
        }

    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, 'w') as f:
        yaml.dump(yaml_data, f, default_flow_style=False, sort_keys=False, width=120, allow_unicode=True)

    print(f"\n\nWrote failure mode taxonomy to: {out_path}")
    print(f"Total failure modes: {len(taxonomy)}")

    # Print summary table
    print()
    print("=" * 70)
    print("SUMMARY — FAILURE MODES BY SEVERITY")
    print("=" * 70)
    print(f"{'Mode':<40} {'Severity':<10} {'Count':<6} {'Notes':<6} {'Category'}")
    print("-" * 90)
    for mode_name, data in sorted_modes:
        print(f"{mode_name:<40} {data['severity']:<10} {data['count']:<6} {data['note_count']:<6} {data['category']}")

    # Print the MEASURE phase recommendations
    print()
    print("=" * 70)
    print("MEASURE PHASE — PRIORITIZED RANKING (prevalence x severity)")
    print("=" * 70)
    severity_weight = {'critical': 4, 'high': 2, 'medium': 1, 'low': 0.5}
    ranked = sorted(taxonomy.items(),
                   key=lambda x: -(x[1]['count'] * severity_weight.get(x[1]['severity'], 1)))
    print(f"{'Rank':<5} {'Mode':<40} {'Score':<8} {'Count':<6} {'Sev':<10}")
    print("-" * 75)
    for i, (mode_name, data) in enumerate(ranked, 1):
        w = severity_weight.get(data['severity'], 1)
        score = data['count'] * w
        print(f"{i:<5} {mode_name:<40} {score:<8.1f} {data['count']:<6} {data['severity']}")

    # IMPROVE phase recommendations
    print()
    print("=" * 70)
    print("IMPROVE PHASE — RECOMMENDED LEVERS")
    print("=" * 70)
    lever_recommendations = {
        'first_name_only': 'PROMPT — add instruction: "In coaching notes, treat any capitalized word used as a subject or after a possessive as a potential person name"',
        'compound_identifier': 'PROMPT — add instruction: "When you see a relationship description that includes an institution or specific detail, redact the ENTIRE phrase, not just the name"',
        'unlabeled_relative_temporal': 'LABELS — review expected labels; some relative temporals should be added to golden set',
        'over_redaction_common_temporal': 'PROMPT — add explicit exclusion list: "Do NOT redact: today, yesterday, last week, this week, next week, this morning, each morning"',
        'possessive_relationship_without_name': 'PROMPT + LABELS — add instruction for relationship-context detection; update golden set for FAMILY_DETAIL consistency',
        'clinical_detail_as_quasi_identifier': 'SCOPE — out of current scope (direct identifiers only), but flag for clinical expert review of re-identification risk in combinations',
        'email_in_username_leaks_name': 'RECOGNIZER — ensure email regex also extracts name components from username; verify name is independently caught as PERSON',
        'location_as_address_vs_region': 'PROMPT — clarify distinction between street addresses (always redact) and city names (redact when combined with other identifiers)',
    }
    for mode_name, rec in lever_recommendations.items():
        if mode_name in taxonomy:
            print(f"\n  {mode_name} [{taxonomy[mode_name]['severity']}]:")
            print(f"    {rec}")


if __name__ == "__main__":
    main()
