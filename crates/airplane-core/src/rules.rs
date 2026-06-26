//! The rules executor — deterministic, auditable detection of structured identifiers.
//!
//! Built-in Safe-Harbor-style patterns plus a pack's recognizers (regex + deny lists).
//! This is the "negative space" stage: no model, fast, reviewable (Constitution IV).
//! It is also the gate's re-scan engine.

use crate::model::Span;
use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RecognizerSpec {
    #[allow(dead_code)]
    name: String,
    supported_entity: String,
    #[serde(default)]
    patterns: Vec<PatternSpec>,
    #[serde(default)]
    deny_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PatternSpec {
    #[allow(dead_code)]
    name: String,
    regex: String,
    #[allow(dead_code)]
    #[serde(default)]
    score: f32,
}

struct CompiledPattern {
    entity: String,
    re: Regex,
}

/// Runs built-in + pack-supplied recognizers over text.
pub struct RulesExecutor {
    patterns: Vec<CompiledPattern>,
    deny: Vec<(String, String)>, // (entity, literal)
}

impl RulesExecutor {
    /// A new executor with the built-in Safe-Harbor-style structured patterns.
    pub fn new() -> Self {
        let builtins: &[(&str, &str)] = &[
            ("SSN", r"\b\d{3}-\d{2}-\d{4}\b"),
            (
                "PHONE",
                r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b",
            ),
            ("EMAIL", r"\b[\w.%+-]+@[\w.-]+\.[A-Za-z]{2,}\b"),
            ("DATE", r"\b\d{1,2}/\d{1,2}/\d{2,4}\b"),
            ("DATE", r"\b\d{4}-\d{2}-\d{2}\b"),
            ("URL", r"\bhttps?://\S+"),
            ("IP", r"\b\d{1,3}(?:\.\d{1,3}){3}\b"),
        ];
        let patterns = builtins
            .iter()
            .map(|(e, r)| CompiledPattern {
                entity: (*e).to_string(),
                re: Regex::new(r).expect("valid built-in regex"),
            })
            .collect();
        Self {
            patterns,
            deny: Vec::new(),
        }
    }

    /// Load a pack recognizer file (Presidio-style JSON) and compile its patterns.
    pub fn load_recognizer_file(&mut self, path: &Path) -> Result<()> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("read recognizer {}", path.display()))?;
        let spec: RecognizerSpec = serde_json::from_str(&raw)
            .with_context(|| format!("parse recognizer {}", path.display()))?;
        for p in &spec.patterns {
            let re = Regex::new(&p.regex)
                .with_context(|| format!("compile regex `{}` in {}", p.name, path.display()))?;
            self.patterns.push(CompiledPattern {
                entity: spec.supported_entity.clone(),
                re,
            });
        }
        for lit in &spec.deny_list {
            self.deny.push((spec.supported_entity.clone(), lit.clone()));
        }
        Ok(())
    }

    /// Add one regex recognizer at runtime. Shells use this for demos or adapters that
    /// stage declarative pack changes before writing/signing a pack artifact.
    pub fn add_regex(&mut self, entity: impl Into<String>, regex: &str) -> Result<()> {
        self.patterns.push(CompiledPattern {
            entity: entity.into(),
            re: Regex::new(regex).with_context(|| format!("compile staged regex `{regex}`"))?,
        });
        Ok(())
    }

    /// All structured-identifier spans found in `text`.
    pub fn find(&self, text: &str) -> Vec<Span> {
        let mut spans = Vec::new();
        for p in &self.patterns {
            for m in p.re.find_iter(text) {
                spans.push(Span::new(m.as_str(), &p.entity, "rules"));
            }
        }
        for (entity, lit) in &self.deny {
            if text.contains(lit.as_str()) {
                spans.push(Span::new(lit.clone(), entity.clone(), "rules"));
            }
        }
        spans
    }
}

impl Default for RulesExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_structured_identifiers() {
        let r = RulesExecutor::new();
        let spans = r.find("call me at 555-123-4567 or a@b.com on 03/14/2026");
        let entities: Vec<_> = spans.iter().map(|s| s.entity.as_str()).collect();
        assert!(entities.contains(&"PHONE"));
        assert!(entities.contains(&"EMAIL"));
        assert!(entities.contains(&"DATE"));
    }

    #[test]
    fn misses_names_in_prose() {
        // The whole point: rules cannot catch contextual names — that's the model's job.
        let r = RulesExecutor::new();
        let spans = r.find("Marcus finally called his mom this week");
        assert!(spans.is_empty());
    }
}
