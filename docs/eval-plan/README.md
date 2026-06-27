# PHI Scrubber Evaluation Plan

> **Purpose:** A shareable, extendable plan for evaluating the Bonsai PHI
> Scrubber's reliability, recruiting clinical domain experts, and building a
> systematic improvement lifecycle.
>
> **Audience:** Clinical experts, legal advisors, ethical reviewers, and
> builders who want to extend the eval surface.
>
> **Status:** Plan — not yet executed. This document defines what to build.
>
> **Last updated:** 2026-06-27

---

## Table of Contents

1. [Framework: Error Analysis → Measure → Improve](#framework)
2. [Current State Assessment](#current-state-assessment)
3. [Gap Analysis](#gap-analysis)
4. [The Error-Discovery Skill Applied to PHI Scrubbing](#error-discovery-skill)
5. [Evaluation Plan (30/60/90 days)](#evaluation-plan)
6. [Devin Runbooks](#devin-runbooks)
7. [Cloud vs Edge Workload Discernment](#cloud-vs-edge-workload-discernment)
8. [Clinical Expert Recruitment Plan](#clinical-expert-recruitment-plan)
9. [Legal & Ethical Framework](#legal--ethical-framework)
10. [Clinical Use Cases](#clinical-use-cases)

---

<a id="framework"></a>
## 1. Framework: Error Analysis → Measure → Improve

This plan applies the **Analyze → Measure → Improve** lifecycle (Shreya
Shankar, Jun 24 2026) to the PHI Scrubber. The key principles from that
framework that apply here:

| Principle | Application to PHI Scrubber |
|-----------|---------------------------|
| **AI can't automate error analysis** — it's taste-specific, discovery-driven | Clinical experts must do the error analysis; they know what PHI looks like in real coaching notes, what identifiers are clinically significant, and what redaction failures would cause harm |
| **Persist intermediates** — don't re-derive failure modes from scratch each run | Build a failure mode taxonomy that lives in the repo, grows with each review cycle, and is versioned |
| **Iterate over data** — don't one-shot; new failure modes emerge on re-review | Build the eval as an outer loop (new note batches) + inner loop (failure mode refinement within a batch) |
| **Breadth then depth** — cover all failure modes, then find many examples of each | First: what categories of PHI does the scrubber miss? Then: how many examples of each category can we generate? |
| **Build interactive review interfaces** — not just CLI metrics | Extend the eval harness to produce human-reviewable trace reports (what was scrubbed, what was missed, what was over-redacted, in context) |
| **Not all applications need the same accuracy bar** — reason about worst case | PHI scrubbing is **high stakes**: a false negative leaks a real identifier. This demands the highest accuracy bar and the most rigorous eval |

---

<a id="current-state-assessment"></a>
## 2. Current State Assessment

### What exists today

| Component | Status | Evidence |
|-----------|--------|----------|
| **Golden eval set** | 21 synthetic notes, 71 labels (55 hard cases) | `packs/coach-session/eval/golden/` |
| **Expected redactions** | Per-note JSON with entity type, text, hard/easy flag | `packs/coach-session/eval/expected/` |
| **Eval harness** | Rust scorer: recall, precision, leakage, hard-case recall, per-entity breakdown | `crates/airplane-core/src/eval.rs` |
| **Verifier gate** | Default-deny re-scan of scrubbed output; blocks on any residual structured identifier | `crates/airplane-core/src/gate.rs` |
| **Rules executor** | Deterministic regex-based recognizers for MEMBER_ID, PERSON, ORG, DATE, etc. | `crates/airplane-core/src/rules.rs` |
| **Gates harness** | `./run.sh gates` runs recall + leakage + pack-blindness + reward-lint + scope-boundary + ethical fixtures | `run.sh`, `scripts/smoke-ethical-gate-fixtures.sh` |
| **Committed golden run** | 100% recall, 51.1% precision, 0 leakage, 100% hard-case recall | `eval/golden-run.txt` |
| **Deterministic reproducibility** | Fixed seeds (42..), temp 0.5, 5 seeded passes, pinned model hash | `eval/README.md` |

### Current metrics

```
recall          : 100.0%  (71/71)
precision       : 51.1%  (71/139 predicted)
hard-case recall: 100.0%  (55/55)
leakage         : 0
over-redactions : 68
```

### What's strong

1. **The verifier gate is the novel contribution.** It re-scans scrubbed output
   with the same rules executor and blocks egress on any residual. This is the
   defense-in-depth layer that most scrubbing approaches lack.
2. **Recall-first design.** 100% recall means zero leakage on the golden set.
   The 51.1% precision (68 over-redactions) is explicitly accepted — false
   positives are safe; false negatives leak.
3. **Deterministic reproducibility.** Pinned model hash, fixed seeds, committed
   golden run. A stranger can run `./run.sh eval` and match.
4. **Ethical gates.** The reward-lint and scope-boundary gates are programmatic
   tests that catch policy violations (e.g., engagement metrics in rewards,
   missing escalation paths).

---

<a id="gap-analysis"></a>
## 3. Gap Analysis

Mapped against the Analyze → Measure → Improve lifecycle:

### Error Analysis gaps

| # | Gap | Why it matters | How to close |
|---|-----|---------------|-------------|
| EA-1 | **No clinical expert review of failure mode taxonomy** | The current recognizers and golden set were authored by builders, not clinicians. Real coaching notes contain identifier patterns that engineers won't anticipate (insurance group numbers, therapeutic technique names that double as identifiers, relationship descriptions that are de facto identifiers). | Recruit 2-3 clinical domain experts to do structured error analysis on synthetic notes. |
| EA-2 | **No adversarial/red-team notes** | All 21 golden notes are "normal" synthetic notes. No notes are designed to probe edge cases: embedded identifiers in quoted speech, identifiers split across sentences, identifiers disguised as common words, identifiers in non-English text. | Create an adversarial note set (10-20 notes) designed to probe known weakness categories. |
| EA-3 | **No interactive review interface** | The eval outputs metrics to stdout. There is no human-reviewable trace report showing scrubbed-vs-original side by side, highlighting what was caught, missed, and over-redacted in context. | Build a simple HTML review interface (per Shreya's skill pattern) or structured JSON trace report. |
| EA-4 | **No failure mode persistence** | Failure modes are implicit in the recognizer patterns. There is no explicit, versioned taxonomy of "ways the scrubber can fail" that grows with each review cycle. | Create `docs/eval-plan/failure-modes.yaml` as a living taxonomy. |

### Measurement gaps

| # | Gap | Why it matters | How to close |
|---|-----|---------------|-------------|
| M-1 | **No clinical significance weighting** | A leaked member ID and a leaked therapy technique name are equally weighted. In reality, a leaked name + date-of-birth combination is catastrophically worse than a leaked employer name. | Add a `severity` field to expected redactions (critical / high / medium / low). Weight recall scoring by severity. |
| M-2 | **No cross-note failure correlation** | The eval scores each note independently. It doesn't identify systematic patterns — e.g., "the model always misses names preceded by possessives" or "dates in relative form are never caught." | Add a failure-mode-prevalence report: group missed labels by entity type and syntactic context. |
| M-3 | **No confidence/stability scoring** | 5 seeded passes are unioned. But how stable is each detection? If a name is caught in 1/5 passes, that's fragile. If 5/5, it's robust. | Report per-label hit-rate across passes, flag fragile detections (<3/5). |
| M-4 | **No cross-model baseline** | The eval only runs against Bonsai-1.7B. There's no baseline showing what a rules-only pass catches (no model) or what a different model would catch. | Add a `--rules-only` mode and optionally a secondary model baseline. |

### Improvement gaps

| # | Gap | Why it matters | How to close |
|---|-----|---------------|-------------|
| I-1 | **No systematic prompt iteration** | The system prompt to Bonsai is fixed. There's no record of prompt variants tried, their effect on recall/precision, or why the current prompt was chosen. | Create a prompt changelog and A/B eval log. |
| I-2 | **No regression testing after recognizer changes** | Adding a new recognizer pattern could cause false positives in previously-clean notes. There's no automated regression check for this. | `./run.sh eval` already catches regressions if golden-run.txt changes; make this explicit in CI/Devin runbooks. |
| I-3 | **No clinical-expert-driven improvement loop** | The current improvement path is builder-driven. Clinical experts should be able to submit new synthetic notes, expected redactions, and failure reports — and see the effect on metrics. | Create a contribution guide for clinical experts. |

---

<a id="error-discovery-skill"></a>
## 4. The Error-Discovery Skill Applied to PHI Scrubbing

> **Prior art:** [`shreyashankar/error-discovery-skill`](https://github.com/shreyashankar/error-discovery-skill)
> (SKILL.md + review-loop.md). The skill makes an AI agent run interactive error
> analysis — build a review UI, select diverse samples, monitor annotations, and
> organize failure modes. This section maps that skill to the PHI Scrubber's
> three-phase eval loop.

### The Dataset: PHI Scrubber Eval Traces

The error-discovery-skill expects a dataset (JSONL/CSV/JSON). For the PHI
Scrubber, the dataset is the **eval trace set** — one record per golden note:

```json
{
  "id": "note-01",
  "original_text": "Maria Alvarez came in tense today...",
  "scrubbed_text": "[PERSON] came in tense today...",
  "predicted_redactions": [
    {"text": "Maria Alvarez", "entity": "PERSON", "layer": "bonsai"},
    {"text": "Brightwater Logistics", "entity": "ORG", "layer": "rules"},
    {"text": "CM-204815", "entity": "MEMBER_ID", "layer": "rules"}
  ],
  "expected_redactions": [
    {"text": "Maria Alvarez", "entity": "PERSON", "hard": true},
    {"text": "Brightwater Logistics", "entity": "ORG", "hard": true},
    {"text": "Marcus", "entity": "PERSON", "hard": true},
    {"text": "CM-204815", "entity": "MEMBER_ID", "hard": false},
    {"text": "the second Tuesday of next month", "entity": "DATE", "hard": true}
  ],
  "caught": ["Maria Alvarez", "Brightwater Logistics", "CM-204815"],
  "missed": ["Marcus", "the second Tuesday of next month"],
  "over_redacted": ["last week's"],
  "gate_decision": "PASS",
  "per_pass_hits": {
    "Maria Alvarez": [true, true, true, true, true],
    "Marcus": [false, true, false, true, false]
  }
}
```

**Content structure** (per SKILL.md Phase 1b): This is an **input/output pair
with structured annotations** — the original note (input), the scrubbed output,
and three annotation layers (predicted, expected, missed).

**Dimensions of variation** (per SKILL.md Phase 1c):

Between notes:
- Entity types present (PERSON, ORG, DATE, EMAIL, PHONE, ADDRESS, MEMBER_ID, FAMILY_DETAIL)
- Number of identifiers (2-5 per note)
- Hard vs easy ratio
- Note length and complexity
- Relationship structures (possessives, pronouns, cross-references)

Within each note:
- Which layer caught each identifier (rules vs bonsai vs both)
- Per-pass stability (caught in 1/5 vs 5/5 passes)
- Over-redaction locations (what non-PHI text was incorrectly flagged)

### Phase 1: ANALYZE — Define Mistakes, Find Mistakes, Create Failure Modes

This is the phase that **cannot be automated**. It requires human judgment —
specifically, clinical expert judgment for PHI-specific failure modes.

#### Step 1: Build the PHI Scrubber Review Interface

Adapt the error-discovery-skill's HTML review app (Phase 3 of SKILL.md) for
PHI scrubber traces:

**Article/Content View — side-by-side original vs scrubbed:**
```
┌─────────────────────────────────────┬──────────────────────────┐
│ ORIGINAL NOTE                       │ MARGIN NOTES             │
│                                     │                          │
│ [Maria Alvarez] came in tense       │ ✓ PERSON caught (5/5)    │
│ today, still carrying the weight    │   layer: bonsai          │
│ of [last week]'s layoff.            │                          │
│                                     │ ⚠ DATE over-redacted     │
│ We spent most of the hour           │   "last week" is not     │
│ unpacking how the conflict with     │   the scheduled date     │
│ her supervisor at [Brightwater      │                          │
│ Logistics] had been building...     │ ✓ ORG caught (5/5)       │
│                                     │   layer: rules           │
│ Near the end she softened, and      │                          │
│ [Marcus] finally called his mom     │ ✗ PERSON missed (2/5)    │
│ to apologize for missing the        │   FRAGILE — first name   │
│ funeral.                            │   only, no surname       │
│                                     │                          │
│ Her member id [CM-204815] was       │ ✓ MEMBER_ID caught (5/5) │
│ updated...                          │   layer: rules           │
│                                     │                          │
│ We agreed to meet again [the        │ ✗ DATE missed (0/5)      │
│ second Tuesday of next month].      │   relative temporal ref  │
└─────────────────────────────────────┴──────────────────────────┘
```

Visual encoding (per SKILL.md Phase 2):
- **Color hue:** green = caught, red = missed, yellow = over-redacted
- **Opacity:** full for identifier spans, muted for surrounding text
- **Border style:** solid = expected label, dashed = agent suggestion
- **Badge in header:** per-pass hit rate as stability indicator

**Map View:** 2D scatter of all 21 notes, clustered by entity-type
composition. Annotated notes in orange. Click to navigate.

**Progress View:**
- Failure mode treemap (sized by annotation count)
- Agent suggestions queue (accept/dismiss)
- Coverage: how many entity types, hard-case categories, and syntactic
  patterns have been reviewed

#### Step 2: The Human Reviews (Clinical Expert + Builder)

The reviewer (clinical expert or Jai) reads each note in the content view and
annotates using free-text inline notes. The skill's key principle applies:
**the human notices, the agent organizes.**

What to look for (seeded from existing eval data):

**Define mistakes** (what does "bad" mean for PHI scrubbing?):
1. **False negative (leakage)** — a real identifier survives into the scrubbed
   output. This is the critical failure. The verifier gate should catch
   structured identifiers, but contextual ones (first names, relative dates,
   relationship descriptions) may pass through.
2. **False positive (over-redaction)** — non-PHI text is redacted. This reduces
   clinical utility of the clean record. Currently 68 over-redactions across
   21 notes (51.1% precision).
3. **Fragile detection** — an identifier is caught in some passes but not others
   (per-pass hit rate < 3/5). This is a reliability risk even when the union
   catches it.
4. **Category error** — the identifier is caught but labeled as the wrong entity
   type (e.g., an employer name labeled PERSON instead of ORG).
5. **Partial detection** — only part of an identifier is caught (e.g., "Maria"
   caught but not "Alvarez," or "Maple Crest" caught but not "Avenue").
6. **Contextual identifier** — information that is not a named entity but could
   identify someone in combination (e.g., "his daughter who just started at
   Northwestern" — this is marked as FAMILY_DETAIL in the golden set).
7. **Implicit identifier** — information that implies identity without naming
   (e.g., "the only female partner at the firm" or "the coach who works
   Wednesdays at the Elm Street clinic").

**Find mistakes** (the interactive loop):
- Breadth first: review one note from each cluster (entity-type diversity)
- When a failure mode is found, depth: spawn subagent to scan all 21 notes for
  instances of that mode
- Re-review: after finding new modes, go back to earlier notes (criteria drift)

**Create failure modes** (the taxonomy):
The agent monitors annotations and builds a running taxonomy:
```yaml
failure_modes:
  first_name_only:
    description: "Single first name without surname — often common names that the model treats as generic words"
    severity: critical  # a real person's name leaked
    count: 3
    examples: ["Marcus (note-01)", "Devon (note-02)", "Priya (note-03)"]
    
  relative_temporal:
    description: "Date expressed relative to an anchor ('this past Thanksgiving', 'the second Tuesday of next month') rather than absolute"
    severity: high  # temporal + other context can identify
    count: 4
    examples: ["the second Tuesday of next month (note-01)", "this past Thanksgiving (note-03)"]
    
  over_redact_common_temporal:
    description: "Common temporal phrases ('last week', 'today') redacted unnecessarily"
    severity: low  # reduces utility but doesn't leak
    count: 12
    examples: ["last week (note-01)", "today (note-05)"]
    
  family_relationship_detail:
    description: "Relationship description that could identify ('his daughter who just started at Northwestern')"
    severity: critical  # combines relationship + institution + timing
    count: 2
    examples: ["his daughter who just started at Northwestern (note-02)"]
```

#### Step 3: Persist the Failure Mode Taxonomy

The taxonomy is the **persisted intermediate** — the artifact that survives
across review sessions. It lives at:

```
docs/eval-plan/failure-modes.yaml
```

This is versioned in git. Each review session updates it. The taxonomy grows
with each clinical expert's contribution.

### Phase 2: MEASURE — Observe Traces, Apply Pareto, Prioritize

Once the failure mode taxonomy exists, measurement begins. This phase CAN be
automated (Devin runbooks).

#### Step 1: Count Failure Mode Prevalence

For each failure mode in the taxonomy, count:
- How many notes contain at least one instance
- Total instances across all notes
- Which entity types are affected
- What percentage of all missed identifiers fall into this mode

#### Step 2: Apply the Pareto Principle

Shreya's key finding: **80% of issues are caused by 20% of failure modes.**

Rank failure modes by prevalence. The top 3-5 modes are where improvement
effort should concentrate.

Example ranking (hypothetical, based on current eval data):

| Rank | Failure Mode | Prevalence | % of all misses | Cumulative % |
|------|-------------|------------|-----------------|-------------|
| 1 | Over-redact common temporal | 17/21 notes | 38% of over-redactions | 38% |
| 2 | First name only (no surname) | 8/21 notes | 22% of misses | 60% |
| 3 | Relative temporal reference | 6/21 notes | 18% of misses | 78% |
| 4 | Family relationship detail | 3/21 notes | 9% of misses | 87% |
| 5 | Partial detection | 2/21 notes | 6% of misses | 93% |

#### Step 3: Apply Legal/Ethical Prioritization

The Pareto ranking alone is not sufficient. A failure mode that is rare but
catastrophic must be prioritized over one that is common but low-severity.

**Legal severity weighting:**

| Severity | Definition | Weight | Examples |
|----------|-----------|--------|---------|
| **Critical** | A real person can be identified from the leaked information alone or in trivial combination | 4x | Full name leaked, SSN leaked, name + DOB |
| **High** | Identification requires combining the leaked information with other context that a motivated actor could obtain | 2x | First name + employer + date, family relationship + institution |
| **Medium** | The leaked information is contextual but not individually identifying | 1x | Relative date, neighborhood description |
| **Low** | Over-redaction (no leak, but reduces clinical utility) | 0.5x | Common temporal phrases, generic pronouns |

**Prioritized ranking = prevalence x severity weight.**

This is where the legal and ethical frameworks shape the improvement loop:
- A legal advisor says: "Under MHMDA, the combination of name + substance
  abuse treatment detail is catastrophically worse than name alone."
- An ethicist says: "Over-redacting therapy technique names reduces the
  clinical utility of the record to the point where the coaching relationship
  suffers."
- A clinical expert says: "In real notes, the therapist often uses possessive
  pronouns ('his daughter') that are effectively identifiers in context."

These judgments convert into severity weights and change the prioritized ranking.

### Phase 3: IMPROVE — Decide on Levers

Once failure modes are measured and prioritized, the improvement phase decides
which levers to pull. There are two primary levers:

#### Lever 1: Change the Prompt

The system prompt to Bonsai tells it what to look for. Prompt changes are:
- **Fast** (minutes to implement, immediate eval)
- **Cheap** (no training cost)
- **Reversible** (just change the prompt back)
- **Limited** (the model's capacity to follow complex instructions is bounded,
  especially at 1.7B parameters)

**When to use prompt changes:**
- The failure mode is a pattern the model could recognize but wasn't told to
  look for (e.g., "also look for first names without surnames")
- The failure mode is about output format (e.g., "include the full phrase,
  not just the key word")
- The failure mode is about over-redaction (e.g., "do NOT redact common
  temporal words like 'today', 'yesterday', 'last week'")

**Prompt changelog requirement:** Every prompt variant is recorded with:
```yaml
prompt_versions:
  v1:
    date: 2026-06-01
    change: "baseline system prompt"
    recall: 100.0%
    precision: 51.1%
    failure_modes_addressed: []
  v2:
    date: 2026-07-15
    change: "added instruction to catch first names without surnames"
    recall: 100.0%
    precision: 48.2%
    failure_modes_addressed: [first_name_only]
    regression: "precision dropped 2.9pp (more over-redaction of common names)"
```

#### Lever 2: Fine-Tune the Model

Fine-tuning changes the model's weights based on training data. It is:
- **Slow** (hours to days of compute)
- **Expensive** (GPU/TPU time, data preparation)
- **Permanent** (changes the model; requires pinning a new hash)
- **Powerful** (can learn patterns that prompt engineering can't express)

**When to use fine-tuning:**
- The failure mode is systematic and the prompt can't fix it (e.g., the model
  doesn't understand that "his daughter who just started at Northwestern" is
  a compound identifier because it doesn't see it as one unit)
- The model consistently fails on a category despite clear prompt instructions
- You have enough labeled examples (50+) of the failure mode to fine-tune on
- The improvement justifies the cost and the need to re-pin the model hash

**Fine-tuning requirements for the PHI Scrubber:**
1. Training data must be **synthetic only** (Hard Rule 6)
2. The fine-tuned model must be **pinned by hash** (Hard Rule 8)
3. The fine-tuned model must pass the **full gate suite** including recall ≥ 99%
4. The old model hash and the new model hash must both be documented
5. The eval must run against both to confirm improvement without regression

#### Lever 3: Add Recognizer Patterns (Rules Layer)

The rules executor uses regex patterns from `packs/coach-session/recognizers/`.
Adding a new pattern is:
- **Fast** (minutes)
- **Deterministic** (regex matches are exact)
- **Limited** (only catches structured identifiers with predictable formats)

**When to use recognizer changes:**
- A new structured identifier format is discovered (e.g., a new member ID
  pattern, a benefits plan number format)
- A false positive is caused by an overly broad regex
- A new clinical vertical (pack) has its own identifier formats

#### Decision Framework

```
                Is the failure mode structured
                (predictable regex pattern)?
                     /           \
                   YES            NO
                    |              |
              Add recognizer   Is the prompt sufficient
              pattern          to describe the pattern?
                                  /          \
                                YES           NO
                                 |             |
                           Change prompt   Is the failure
                                          systematic (50+
                                          examples)?
                                             /       \
                                           YES        NO
                                            |          |
                                       Fine-tune   Change prompt
                                       the model   (accept residual
                                                   error, document
                                                   in failure modes)
```

### How This Connects to UVI

The three phases map to the UVI scale:

| Phase | UVI Position | What it proves |
|-------|-------------|----------------|
| **Analyze** | Usability → Value | "We can find our own mistakes systematically" |
| **Measure** | Value | "We know how prevalent each failure mode is and which ones matter most" |
| **Improve** | Value → Impact | "We can reduce failure rates using the right lever for each failure mode" |

We are at **Value**. The scrubber does useful work (data piping). The eval loop
is how we move toward Impact — but Impact requires clinical validation,
legal framework completion, and real-world workload testing that we haven't
done yet. We say this explicitly.

---

<a id="evaluation-plan"></a>
## 5. Evaluation Plan (30/60/90 days)

### Days 0-30: Build the error analysis layer

| Task | Owner | Deliverable |
|------|-------|-------------|
| Create failure mode taxonomy v1 | Builder (Jai) + 1 clinical expert | `docs/eval-plan/failure-modes.yaml` |
| Build adversarial note set (10-20 notes) | Builder + clinical expert | `packs/coach-session/eval/adversarial/` |
| Build HTML trace review interface | Builder (Devin runbook) | `./run.sh eval-review` → serves localhost HTML |
| Add per-label stability scoring (hit-rate across 5 passes) | Builder | `eval.rs` enhancement |
| Add severity field to expected redactions | Builder + clinical expert | `packs/coach-session/eval/expected/*.json` schema update |
| Run rules-only baseline | Builder | `./run.sh eval --rules-only` |
| Document prompt changelog | Builder | `docs/eval-plan/prompt-log.md` |

**Milestone:** Error analysis infrastructure exists. Clinical expert can review
traces in a human-readable interface and contribute failure modes.

### Days 30-60: Measurement and clinical expert loop

| Task | Owner | Deliverable |
|------|-------|-------------|
| Recruit 2-3 clinical domain experts (coaching, therapy, social work) | Jai | Named advisors with signed contribution agreements |
| Clinical expert error analysis session (Shreya-style: review traces, annotate failure modes, build taxonomy) | Clinical expert + Jai | Updated failure mode taxonomy, 5+ new failure categories |
| Expand golden set to 50+ notes based on clinical expert input | Clinical expert + builder | `packs/coach-session/eval/golden/` expansion |
| Add clinical significance weighting to scoring | Builder | Severity-weighted recall metric |
| Add failure-mode-prevalence report | Builder | `./run.sh eval --report` |
| Create Devin runbooks for automated eval cycles | Builder | `.agents/skills/` or Devin playbooks |
| Cross-model baseline (rules-only vs Bonsai-1.7B vs Bonsai-4B) | Builder | Comparison report |

**Milestone:** Clinical experts are contributing to the failure mode taxonomy.
Measurement is severity-weighted. Devin runbooks automate the eval cycle.

### Days 60-90: Improvement lifecycle and legal/ethical framework

| Task | Owner | Deliverable |
|------|-------|-------------|
| Systematic prompt iteration based on top failure modes | Builder | 3+ prompt variants evaluated, best promoted |
| Legal framework review (MHMDA, state mental health privacy laws) | Legal advisor | `docs/eval-plan/legal-framework.md` |
| Ethical framework review (clinical ethics of automated scrubbing) | Clinical expert + ethicist | `docs/eval-plan/ethical-framework.md` |
| New pack for a second clinical vertical (e.g., benefits navigation, intake) | Clinical expert + builder | `packs/benefits-nav/` |
| Regression testing automation via Devin | Builder | Devin runbook: on-PR eval check |
| Publish first clinical expert journey report | Jai | Shareable document with clear non-claims |

**Milestone:** Legal and ethical frameworks are documented. Two clinical
verticals have packs with eval sets. The improvement lifecycle is running.

---

<a id="devin-runbooks"></a>
## 6. Devin Runbooks

These are automatable workflows that Devin can execute repeatedly to evaluate
the PHI Scrubber's reliability. Each runbook is designed to be a Devin playbook
or skill.

### Runbook 1: Golden Set Eval (baseline check)

```
Name: phi-scrubber-golden-eval
Trigger: On PR, on schedule (weekly), or manual
Steps:
  1. Clone bonsai-phi-scrubber
  2. Build: cargo build --bin airplane
  3. Run: ./run.sh eval
  4. Compare output to eval/golden-run.txt
  5. If mismatch: report diff (which notes changed, which metrics shifted)
  6. If match: report "golden set stable"
Output: Pass/fail + metric diff
```

### Runbook 2: Adversarial Eval (stress test)

```
Name: phi-scrubber-adversarial-eval
Trigger: After recognizer or prompt changes, or manual
Steps:
  1. Clone bonsai-phi-scrubber
  2. Build: cargo build --bin airplane
  3. Run eval against adversarial note set (when it exists)
  4. Report: per-note pass/fail, per-entity recall, failure modes triggered
  5. Flag any new leakage (false negatives on adversarial notes)
Output: Adversarial recall report + failure mode breakdown
```

### Runbook 3: Stability Analysis (cross-seed variance)

```
Name: phi-scrubber-stability
Trigger: After model or prompt changes, or manual
Steps:
  1. Clone bonsai-phi-scrubber
  2. Run eval with seeds 42..46 (default) and report per-label hit-rate
  3. Run eval with seeds 100..104 (different seed set) and compare
  4. Flag any label with hit-rate < 3/5 (fragile detection)
  5. Report fragile detections by entity type and note
Output: Stability report + fragile detection list
```

### Runbook 4: Trace Review Report (for clinical experts)

```
Name: phi-scrubber-trace-review
Trigger: Before clinical expert review session, or manual
Steps:
  1. Clone bonsai-phi-scrubber
  2. Run eval and capture full trace output (per-note predictions + expected)
  3. Generate HTML report: side-by-side original vs scrubbed, highlighting
     caught (green), missed (red), and over-redacted (yellow)
  4. Serve report on localhost or export as static HTML
  5. Clinical expert reviews and annotates in the interface
Output: HTML trace report for human review
```

### Runbook 5: Regression Check (CI-style)

```
Name: phi-scrubber-regression
Trigger: On every PR that touches crates/, packs/, or scripts/
Steps:
  1. Build
  2. Run ./run.sh gates-fast (no model required)
  3. If model available: run ./run.sh gates (full)
  4. Compare eval/golden-run.txt to committed version
  5. Report any metric changes with context
Output: Pass/fail + metric delta
```

### Runbook 6: New Note Ingestion (clinical expert contribution)

```
Name: phi-scrubber-add-note
Trigger: When a clinical expert submits a new synthetic note
Steps:
  1. Validate note is synthetic (no real PHI — automated check + human confirm)
  2. Place note in packs/coach-session/eval/golden/note-{N}.txt
  3. Create expected redactions JSON: packs/coach-session/eval/expected/note-{N}.json
  4. Run eval with the new note included
  5. Report results: did the scrubber catch everything? What was missed?
  6. If new failure modes found: update failure-modes.yaml
  7. If eval passes: update golden-run.txt via ./run.sh eval --update
Output: New note evaluated + failure mode taxonomy updated
```

### Runbook 7: Error Discovery Session (Shreya's skill)

```
Name: phi-scrubber-error-discovery
Trigger: Before clinical expert review, after golden set changes, or manual
Prereq: error-discovery-skill cloned (https://github.com/shreyashankar/error-discovery-skill)
Steps:
  1. Clone bonsai-phi-scrubber
  2. Build and run eval to generate trace JSONL
     (one record per note: original, scrubbed, predicted, expected, caught,
      missed, over-redacted, gate decision, per-pass hits)
  3. Invoke the error-discovery skill against the trace JSONL:
     Phase 1: Understand the data (input/output pair with annotations)
     Phase 2: Design visual encoding (green/red/yellow for caught/missed/over)
     Phase 3: Build the review HTML app (Python stdlib server)
     Phase 4: Cluster notes by entity-type composition, select 15-21 samples
  4. Launch the review app on localhost
  5. Human (clinical expert or Jai) reviews and annotates
  6. Agent monitors annotations, builds failure mode taxonomy
     (review-loop.md: breadth → depth → re-review)
  7. Export: failure-modes.yaml + annotated trace report
  8. Measure: rank failure modes by prevalence x severity
  9. Recommend: which lever to pull (prompt / recognizer / fine-tune)
Output: Failure mode taxonomy + prioritized improvement recommendations
```

---

<a id="cloud-vs-edge-workload-discernment"></a>
## 7. Cloud vs Edge Workload Discernment

The PHI Scrubber's architecture creates a natural split between workloads that
**must** stay at the edge and workloads that **can** run in the cloud.

### Decision Framework

```
                    Contains real PHI?
                    /              \
                  YES               NO
                   |                 |
            MUST be edge        Can be cloud
               |                    |
         Contains raw          Contains only
         note text?            synthetic data?
          /        \            /          \
        YES        NO         YES          NO
         |          |          |            |
      Edge only  Edge only  Cloud OK    Evaluate
      (capture,  (redaction  (eval,      case by
       scrub)    map, gate)  CI, Devin   case
                             runbooks)
```

### Workload Classification

| Workload | Edge or Cloud | Rationale |
|----------|--------------|-----------|
| **Note capture** | Edge only | Raw notes contain PHI. Phone → laptop, never cloud. |
| **Bonsai inference** (scrubbing) | Edge only | Raw note is the input. The model sees PHI. |
| **Verifier gate** | Edge only | Re-scans scrubbed text. While the scrubbed text should be clean, the gate is the trust boundary — it runs where the data is. |
| **Redaction map storage** | Edge only | The mapping of original text to redacted text is itself PHI. |
| **Slack egress** (clean record) | Cloud OK | Only the verified-clean record crosses the boundary. The gate has already proved zero residual identifiers. |
| **Golden set eval** (`./run.sh eval`) | Cloud OK | Uses only synthetic notes and expected labels. No real PHI. Devin can run this. |
| **Gates harness** (`./run.sh gates`) | Cloud OK (model-free gates) | `gates-fast` runs without a model. Full `gates` needs the model running locally. |
| **Adversarial eval** | Cloud OK | Synthetic adversarial notes. No real PHI. |
| **Trace review report generation** | Cloud OK | Generated from synthetic data. |
| **CI regression checks** | Cloud OK | Synthetic data only. |
| **Failure mode taxonomy maintenance** | Cloud OK | Metadata about failure types, not PHI. |
| **Clinical expert annotation** (on synthetic data) | Cloud OK | Experts review synthetic notes only. Never real patient data. |
| **Prompt iteration / A/B testing** | Cloud OK for synthetic; Edge only if using real notes | For the eval set (synthetic), cloud is fine. |
| **Model training / fine-tuning** | Cloud OK | Training data must be synthetic or properly de-identified by a separate, validated process. PrismML's models are trained on public data. |
| **Trajectory / RL-environment storage** | Edge only | Constitution IX: trajectory store is behind the verifier gate. |

### How to run evals in the cloud (via Devin)

The eval harness can run on Devin's VM because:

1. **All test data is synthetic** — the golden notes in `packs/coach-session/eval/golden/`
   are authored synthetic notes, never real patient data.
2. **The model is not required for `gates-fast`** — policy, provenance, reward-lint,
   and scope-boundary gates run without a model.
3. **For full eval, the model needs to be accessible** — either:
   - Install `llama-server` + download the pinned Bonsai GGUF on the Devin VM
   - Or: run eval against a pre-computed prediction cache (not yet implemented)

**Recommended approach for Devin:**

- Run `./run.sh gates-fast` on every PR (no model needed, fast)
- Run full `./run.sh eval` on a schedule (weekly) on a machine with the model
- Generate trace review reports in Devin for clinical expert consumption
- Keep the model-dependent eval as a release gate, not a PR gate

---

<a id="clinical-expert-recruitment-plan"></a>
## 8. Clinical Expert Recruitment Plan

### Who to recruit

| Role | Why | Ideal profile |
|------|-----|---------------|
| **Clinical domain expert** (2-3 people) | Knows what real coaching/therapy notes look like; can identify identifier patterns engineers won't anticipate; understands clinical significance of different PHI types | Licensed clinical social worker (LCSW), licensed professional counselor (LPC), or clinical psychologist with coaching/therapy practice experience |
| **Legal advisor** (1 person) | Can evaluate the scrubber against MHMDA, state mental health privacy laws, HIPAA's de-identification safe harbor, and the distinction between "scrubbed" and "de-identified" | Health privacy attorney or compliance officer with MHMDA / 42 CFR Part 2 experience |
| **Ethical reviewer** (1 person) | Can evaluate the ethical implications of automated PHI scrubbing — consent, autonomy, power dynamics, equity | Clinical ethicist or bioethics researcher, ideally with experience in digital health ethics |

### What they'd do

#### Clinical domain experts

1. **Error analysis sessions** (Shreya-style):
   - Review synthetic coaching notes in the trace review interface
   - Identify failure modes the current scrubber misses
   - Annotate: "this is PHI because..." / "this is not PHI because..."
   - Build the failure mode taxonomy from clinical experience
   - Create new synthetic notes that probe identified gaps

2. **Severity classification:**
   - Rate each identifier type by clinical significance
   - Define what constitutes a "catastrophic" leak vs a "nuisance" over-redaction
   - Map identifier combinations that are worse together (name + DOB, name + employer + date)

3. **Clinical use case validation:**
   - Review the clean care record schema: is it clinically useful?
   - Identify what information must survive scrubbing for the record to have value
   - Validate that the escalation boundary (coach ≠ therapist) is clinically sound

#### Legal advisor

1. **Regulatory mapping:**
   - Map the scrubber's behavior against HIPAA Safe Harbor (18 identifiers)
   - Evaluate against MHMDA / 42 CFR Part 2 (substance abuse / mental health records)
   - Identify state-specific mental health privacy requirements
   - Clarify the distinction between "scrubbed" (our claim) and "de-identified" (we do NOT claim)

2. **Claims review:**
   - Validate every public-facing claim in the repo
   - Flag anything that could be construed as a compliance claim
   - Define safe harbor language for the demo and documentation

#### Ethical reviewer

1. **Ethics assessment:**
   - Evaluate consent model: who consents to automated scrubbing?
   - Assess power dynamics: who controls the scrubbing criteria?
   - Review equity implications: does the scrubber perform equally across names from different cultural backgrounds?
   - Evaluate the "synthetic only" boundary: how do we ensure real notes never enter the system?

### How to engage them

| Phase | Duration | Commitment | Compensation model |
|-------|----------|------------|-------------------|
| **Phase 1: Initial review** | 2-4 hours | Review docs, do one error analysis session, provide written feedback | Honorarium or advisory equity |
| **Phase 2: Ongoing advisory** | 2 hours/month | Monthly review of new failure modes, new notes, metric changes | Advisory board retainer |
| **Phase 3: Co-creation** | As needed | Create new packs, validate new verticals, co-author journey reports | Named co-author on publications |

### What they receive

- Access to the repo and trace review interface
- Contribution guide: how to submit synthetic notes and expected redactions
- Clear scope: synthetic data only, no real patient data ever
- Named credit in the repo and in any published reports
- Opportunity to shape the legal, ethical, and clinical frameworks from the ground up

---

<a id="legal-ethical-framework"></a>
## 9. Legal & Ethical Framework (Skeleton)

> This section is a skeleton for the legal and ethical framework. It must be
> completed by qualified legal and clinical ethics advisors.

### Legal Framework

```
Status: SKELETON — requires legal review

Key regulatory touchpoints:
1. HIPAA Safe Harbor method (18 identifier categories)
   - Current coverage: MEMBER_ID, PERSON, ORG, DATE — NOT all 18
   - Gap: geographic data, phone numbers, fax numbers, email, SSN, MRN,
     health plan beneficiary numbers, account numbers, certificate/license
     numbers, VINs, device IDs, URLs, IPs, biometric IDs, photos, "any other
     unique identifying number, characteristic, or code"

2. MHMDA / 42 CFR Part 2 (substance abuse records)
   - Heightened protections for mental health / substance abuse treatment records
   - Applies directly to coaching notes if coaching is part of a treatment program

3. State mental health privacy laws
   - Vary by state; some more restrictive than HIPAA

4. GDPR / international considerations
   - If any user is in the EU, GDPR's definition of personal data is broader

5. What we can claim:
   - "Scrubbed" — yes
   - "Redacted" — yes
   - "De-identified" per HIPAA Safe Harbor — NO (we don't cover all 18 categories)
   - "HIPAA-compliant" — NO (compliance is organizational, not per-tool)
   - "Synthetic data only" — yes (for the eval set and demo)
```

### Ethical Framework

```
Status: SKELETON — requires clinical ethics review

Key ethical considerations:
1. Consent and autonomy
   - Who consents to automated scrubbing of their notes?
   - Can a client opt out? What happens if they do?
   - Is the client informed that AI is processing their session notes?

2. Accuracy and harm
   - A false negative (leaked PHI) causes direct harm to the client
   - A false positive (over-redaction) reduces clinical utility of the record
   - How do we balance these risks?
   - What is the acceptable error rate for a clinical setting?

3. Equity
   - Does the scrubber perform equally across names from different
     cultural/linguistic backgrounds?
   - Are there systematic biases in what gets caught vs missed?

4. Power and control
   - Who defines the scrubbing criteria? (Pack author)
   - Who reviews the scrubbed output? (Clinician)
   - Who has access to the raw notes? (Only the edge device)
   - Is there a human-in-the-loop before the clean record is sent?

5. Synthetic-only boundary
   - How do we technically enforce that real notes never enter the eval set?
   - What controls prevent a user from accidentally using real data in the demo?
```

---

<a id="clinical-use-cases"></a>
## 10. Clinical Use Cases

These are the verticals where the PHI Scrubber pattern could be extended. Each
requires its own pack (recognizers, schema, policy, sink, eval set) and its own
clinical expert review.

### Use Case 1: Coaching Session Notes (current)

```
Pack: packs/coach-session/
Status: Reference implementation, eval passing
Clinical expert needed: Licensed coaching professional or LCSW
Key identifiers: client names, member IDs, employer names, dates, locations
Clean record schema: client_pseudonym, themes, commitments, follow_ups,
                     risk_flags, next_touch
Escalation boundary: coach ≠ therapist; crisis → escalation path required
```

### Use Case 2: Benefits Navigation

```
Pack: packs/benefits-nav/ (to be created)
Status: Planned
Clinical expert needed: Benefits navigator or social worker
Key identifiers: member IDs, insurance group numbers, SSNs, employer names,
                 plan names, provider names, dates, addresses, phone numbers
Clean record schema: issue_category, resolution_steps, referrals,
                     follow_up_date, barriers_identified
Escalation boundary: navigator ≠ case manager; complex cases → supervisor
```

### Use Case 3: Intake Forms

```
Pack: packs/intake/ (to be created)
Status: Planned
Clinical expert needed: Intake coordinator or triage nurse
Key identifiers: full demographic set (name, DOB, SSN, address, phone,
                 insurance, emergency contact, referring provider)
Clean record schema: presenting_concern, urgency_level, service_requested,
                     insurance_verification_status
Escalation boundary: administrative intake ≠ clinical assessment
```

### Use Case 4: Discharge Summaries

```
Pack: packs/discharge/ (to be created)
Status: Planned
Clinical expert needed: Discharge planner or case manager
Key identifiers: patient name, MRN, DOB, attending physician, facility name,
                 medications, diagnoses (may or may not be PHI depending on context)
Clean record schema: discharge_date, disposition, follow_up_plan,
                     medication_reconciliation, safety_plan
Escalation boundary: discharge planning ≠ continued treatment
```

### Use Case 5: Community Health Worker Field Notes

```
Pack: packs/chw-field/ (to be created)
Status: Planned
Clinical expert needed: Community health worker supervisor or CHW program manager
Key identifiers: client names, addresses, household members, school names,
                 community organization names, social service case numbers
Clean record schema: visit_type, sdoh_factors, referrals_made, barriers,
                     follow_up_plan, transportation_needs
Escalation boundary: CHW ≠ clinician; clinical concerns → referral required
```

---

## Appendix: How This Connects to the PrismML Story

This eval plan is directly relevant to the DevRel pitch:

1. **It proves the thesis honestly.** The eval infrastructure is the mechanism
   by which PrismML can claim "Bonsai is reliable enough for sensitive
   workloads" — with numbers, not marketing.

2. **Clinical experts make it credible.** Builder-authored evals prove the
   engineering. Clinical-expert-authored evals prove the domain applicability.

3. **It's a flywheel.** Each clinical use case generates a new pack → new eval
   set → new proof point → new reference for the next builder. This is the
   adoption flywheel from the DevRel plan, grounded in clinical reality.

4. **It's the cloud/edge story made concrete.** The eval can run in the cloud
   (Devin, CI, synthetic data). The scrubbing runs at the edge. This separation
   is itself a reference architecture for how to build trustworthy AI systems
   that touch sensitive data.
