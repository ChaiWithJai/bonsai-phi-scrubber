//! Eval scoring — recall / leakage / over-redaction against a golden set.
//!
//! Recall is the priority (a false negative leaks; a false positive over-redacts).
//! Matching is case-insensitive substring, the same rule used in the M0.5 spike, so
//! the Rust numbers are comparable to the spike's.

use crate::model::{Expected, Span};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize)]
pub struct Score {
    pub notes: usize,
    pub total_labels: usize,
    pub caught: usize,
    pub recall: f64,
    pub precision: f64,
    pub leakage: usize,
    pub over_redactions: usize,
    pub hard_total: usize,
    pub hard_caught: usize,
    pub hard_recall: f64,
    pub per_entity: BTreeMap<String, (usize, usize)>, // entity -> (caught, total)
    pub missed: Vec<Missed>,
}

#[derive(Debug, Serialize)]
pub struct Missed {
    pub note: String,
    pub text: String,
    pub entity: String,
    pub hard: bool,
}

fn matches(pred: &str, label: &str) -> bool {
    let p = pred.to_lowercase();
    let l = label.to_lowercase();
    p.contains(&l) || l.contains(&p)
}

/// Accumulate one note's predictions against its expected labels.
pub fn score_note(predicted: &[Span], expected: &Expected, acc: &mut Score) {
    acc.notes += 1;
    for exp in &expected.redactions {
        acc.total_labels += 1;
        if exp.hard {
            acc.hard_total += 1;
        }
        let e = acc.per_entity.entry(exp.entity.clone()).or_insert((0, 0));
        e.1 += 1;
        let caught = predicted.iter().any(|p| matches(&p.text, &exp.text));
        if caught {
            acc.caught += 1;
            acc.per_entity.get_mut(&exp.entity).unwrap().0 += 1;
            if exp.hard {
                acc.hard_caught += 1;
            }
        } else {
            acc.missed.push(Missed {
                note: expected.id.clone(),
                text: exp.text.clone(),
                entity: exp.entity.clone(),
                hard: exp.hard,
            });
        }
    }
    // Over-redaction: a predicted span overlapping no label (recall-first: reported, not gated).
    for p in predicted {
        if !expected
            .redactions
            .iter()
            .any(|e| matches(&p.text, &e.text))
        {
            acc.over_redactions += 1;
        }
    }
}

/// Compute the derived rates after all notes are accumulated.
pub fn finalize(acc: &mut Score) {
    acc.leakage = acc.missed.len();
    acc.recall = if acc.total_labels > 0 {
        acc.caught as f64 / acc.total_labels as f64
    } else {
        1.0
    };
    let predicted_total = acc.caught + acc.over_redactions;
    acc.precision = if predicted_total > 0 {
        acc.caught as f64 / predicted_total as f64
    } else {
        1.0
    };
    acc.hard_recall = if acc.hard_total > 0 {
        acc.hard_caught as f64 / acc.hard_total as f64
    } else {
        1.0
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ExpectedSpan;

    #[test]
    fn scores_recall_and_leakage() {
        let predicted = vec![Span::new("Maria Alvarez", "PERSON", "bonsai")];
        let expected = Expected {
            id: "note-01".into(),
            clean: false,
            redactions: vec![
                ExpectedSpan {
                    text: "Maria Alvarez".into(),
                    entity: "PERSON".into(),
                    hard: true,
                },
                ExpectedSpan {
                    text: "CM-204815".into(),
                    entity: "MEMBER_ID".into(),
                    hard: false,
                },
            ],
        };
        let mut acc = Score::default();
        score_note(&predicted, &expected, &mut acc);
        finalize(&mut acc);
        assert_eq!(acc.caught, 1);
        assert_eq!(acc.leakage, 1); // CM-204815 missed
        assert_eq!(acc.hard_caught, 1);
    }
}
