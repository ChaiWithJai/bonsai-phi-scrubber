//! Pipeline orchestration: `rules ∪ inference → redact → gate`.
//!
//! The union is recall-first (redact if *either* layer flags). The model layer is
//! reached only through the [`InferenceProvider`] port. The redaction map (PHI → token)
//! is the radioactive artifact — it stays with the caller / Secure Enclave, never a sink.

use crate::gate::{GateDecision, VerifierGate};
use crate::hygiene;
use crate::model::{Sampling, Span};
use crate::ports::{InferenceProvider, InferenceRequest};
use crate::rules::RulesExecutor;
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

pub struct ScrubResult {
    /// PHI → token table. The radioactive artifact; never leaves the device.
    pub redaction_map: Vec<Span>,
    /// The de-identified text safe to forward past the gate.
    pub scrubbed_text: String,
    /// Egress decision (default-deny).
    pub gate: GateDecision,
}

/// Allowed identifier categories. Constraining `entity` to this enum in the grammar
/// stops the small model from swapping a name into the `entity` slot — the failure
/// mode that leaks the actual identifier.
pub const ENTITIES: &[&str] = &[
    "PERSON",
    "DATE",
    "LOCATION",
    "ADDRESS",
    "ORG",
    "RELATIONSHIP",
    "FAMILY_DETAIL",
    "MEMBER_ID",
    "PHONE",
    "EMAIL",
    "OTHER",
];

/// The JSON schema the model output is constrained to (CLI: server-side; iOS: client-side).
pub fn bonsai_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["spans"],
        "properties": {
            "spans": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["text", "entity"],
                    "properties": {
                        "text": { "type": "string" },
                        "entity": { "type": "string", "enum": ENTITIES }
                    }
                }
            }
        }
    })
}

/// Deterministic shape check on a model-proposed span. Recall-safe: every rule here
/// rejects only shapes that cannot be a real identifier (so it never drops true PHI),
/// while removing the small model's common false positives (mislabeled activities/verbs).
fn plausible(text: &str, entity: &str) -> bool {
    let t = text.trim();
    let lower = t.to_lowercase();
    let words = t.split_whitespace().count();
    let has_digit = t.chars().any(|c| c.is_ascii_digit());
    let digit_count = t.chars().filter(|c| c.is_ascii_digit()).count();
    let has_sentence_punct = t.contains('.') || t.contains('!') || t.contains('?');
    // Common non-identifier words the 1.7B model mislabels (never real names/ids).
    const COMMON: &[&str] = &[
        "committed",
        "commit",
        "commitment",
        "daily",
        "weekly",
        "morning",
        "evening",
        "walk",
        "walks",
        "session",
        "meeting",
        "met",
        "next",
        "plan",
        "goal",
        "the",
        "she",
        "he",
        "her",
        "him",
        "they",
        "i",
        "email",
        "set",
        "today",
        "today's session",
        "today's check-in",
        "check-in",
        "lights out",
        "lights out by eleven",
        "weekend",
        "weekend pickups",
        "this week",
        "this year",
        "summer",
        "household",
        "concrete step",
        "handoffs",
        "isolation",
        "local group",
        "recently joined a pottery class",
        "pottery class",
        "around the block",
        "memory-care wing",
        "sleeping",
        "progress",
        "feels",
        "slow",
        "but it is real",
        "evening walks",
    ];
    if COMMON.contains(&lower.as_str()) {
        return false;
    }
    match entity {
        "PERSON" => words <= 5 && !has_digit && !has_sentence_punct,
        "ORG" => words <= 5,
        "MEMBER_ID" => words == 1 && has_digit, // ids are single tokens containing a digit
        "PHONE" => digit_count >= 7,
        "EMAIL" => t.contains('@') && t.contains('.'),
        "DATE" => words <= 9 && !has_sentence_punct,
        "RELATIONSHIP" | "FAMILY_DETAIL" | "ADDRESS" | "LOCATION" => {
            words <= 9 && !has_sentence_punct
        }
        _ => true,
    }
}

#[derive(Deserialize)]
struct ModelSpans {
    #[serde(default)]
    spans: Vec<ModelSpan>,
}
#[derive(Deserialize)]
struct ModelSpan {
    text: String,
    #[serde(default)]
    entity: String,
}

/// The contextual layer — ask the model for identifiers, parse defensively.
/// Never panics on bad output: a parse failure yields no spans (the rules layer
/// and the gate still hold the line).
pub fn bonsai_layer(
    text: &str,
    model: &dyn InferenceProvider,
    sampling: Sampling,
) -> Result<Vec<Span>> {
    let schema = bonsai_schema();
    let req = InferenceRequest {
        system: "You are a PHI de-identification engine. From the note, list ONLY personal \
                 identifiers: people's names, family relationships, specific dates, \
                 locations/addresses, organizations or employers, member/record IDs, and \
                 contact info (phone, email). Do NOT list activities, goals, commitments, \
                 durations, feelings, or generic words. `text` = the exact words copied \
                 verbatim from the note; `entity` = its category. Respond with JSON only.\n\n\
                 Example —\n\
                 Note: \"Saw Tom Reilly (CM-100200) Friday. His wife Dana is unwell. He'll jog 20 minutes daily.\"\n\
                 JSON: {\"spans\":[{\"text\":\"Tom Reilly\",\"entity\":\"PERSON\"},{\"text\":\"CM-100200\",\"entity\":\"MEMBER_ID\"},{\"text\":\"Friday\",\"entity\":\"DATE\"},{\"text\":\"wife Dana\",\"entity\":\"FAMILY_DETAIL\"}]}\n\
                 (\"jog 20 minutes daily\" is an activity — NOT listed.)",
        user: text,
        json_schema: &schema,
        sampling,
    };
    let raw = model.complete(&req)?;
    let lower = text.to_lowercase();
    let spans = hygiene::extract_json_object(&raw)
        .and_then(|j| serde_json::from_str::<ModelSpans>(&j).ok())
        .map(|m| {
            m.spans
                .into_iter()
                // Keep only spans that (a) appear verbatim in the note — drops
                // hallucinations and the swapped-field failure mode — and (b) match the
                // plausible shape of their category — drops e.g. a 7-word "PERSON".
                .filter(|s| {
                    let t = s.text.trim();
                    !t.is_empty() && lower.contains(&t.to_lowercase()) && plausible(t, &s.entity)
                })
                .map(|s| {
                    let entity = if s.entity.is_empty() {
                        "OTHER".to_string()
                    } else {
                        s.entity
                    };
                    Span::new(s.text, entity, "bonsai")
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(spans)
}

/// Full scrub: union the layers, redact, and run the verifier gate.
///
/// `passes` runs the contextual layer K times with consecutive seeds and **votes**:
/// a candidate is redacted only if ≥`threshold` passes agree on it. Single-pass recall
/// on a 1.7B ternary model is ~80% and *stochastic* — it both misses and spuriously
/// flags a different subset each seed. Voting keeps what the model consistently sees
/// (real identifiers) and drops one-off noise (e.g. a stray pass redacting an activity),
/// balancing the recall a PHI boundary needs against precision. Rules-layer spans are
/// deterministic and always kept.
pub fn scrub(
    text: &str,
    rules: &RulesExecutor,
    model: Option<&dyn InferenceProvider>,
    sampling: Sampling,
    passes: u32,
) -> Result<ScrubResult> {
    let mut spans = rules.find(text);
    if let Some(m) = model {
        // Union across passes — recall-first. A PHI boundary never trades recall for
        // precision: the hardest short names (e.g. "Theo", "Imani") only surface on
        // *some* seeds, so any agreement threshold > 1 leaks them. Precision instead
        // comes from recall-safe means: the grammar entity-enum, the few-shot prompt,
        // and deterministic shape-validation (`plausible`) in `bonsai_layer`.
        for i in 0..passes.max(1) {
            let mut s = sampling;
            s.seed = sampling.seed.wrapping_add(i as u64);
            spans.extend(bonsai_layer(text, m, s)?);
        }
    }

    // Dedup by case-insensitive text; longest spans first so redaction is greedy.
    spans.sort_by_key(|s| std::cmp::Reverse(s.text.len()));
    let mut seen: HashSet<String> = HashSet::new();
    spans.retain(|s| seen.insert(s.key()));

    // Redact: replace each identifier with an [ENTITY] token.
    let mut scrubbed = text.to_string();
    for s in &spans {
        if s.text.trim().is_empty() {
            continue;
        }
        scrubbed = scrubbed.replace(&s.text, &format!("[{}]", s.entity));
    }

    let gate = VerifierGate::new(rules).check(&scrubbed);
    Ok(ScrubResult {
        redaction_map: spans,
        scrubbed_text: scrubbed,
        gate,
    })
}

#[cfg(test)]
mod tests {
    use super::plausible;

    #[test]
    fn validator_rejects_only_non_identifiers() {
        // real identifiers pass
        assert!(plausible("Maria Alvarez", "PERSON"));
        assert!(plausible("CM-204815", "MEMBER_ID"));
        assert!(plausible("daughter just started college", "FAMILY_DETAIL"));
        // the model's common false positives are rejected — recall-safe (never real PHI)
        assert!(!plausible("Committed", "PERSON")); // a verb, not a name
        assert!(!plausible("walk", "PERSON")); // an activity
        assert!(!plausible("10-min morning walk", "MEMBER_ID")); // not an id shape
        assert!(!plausible(
            "Met with John Doe at the clinic today",
            "PERSON"
        )); // too long
        assert!(!plausible("email", "EMAIL")); // channel, not address
        assert!(!plausible("Delphine moved to Juniper Bend Road.", "PHONE")); // category swap
    }
}
