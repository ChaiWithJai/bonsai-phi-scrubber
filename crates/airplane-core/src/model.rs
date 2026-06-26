//! Core data types shared across the pipeline.

use serde::{Deserialize, Serialize};

/// A detected identifier span. `layer` records which stage caught it
/// (`"rules"`, `"bonsai"`, or `"both"`) — useful for the eval breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    pub entity: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub layer: String,
}

impl Span {
    pub fn new(
        text: impl Into<String>,
        entity: impl Into<String>,
        layer: impl Into<String>,
    ) -> Self {
        Self {
            text: text.into(),
            entity: entity.into(),
            layer: layer.into(),
        }
    }
    /// Dedup key — identifiers are matched case-insensitively.
    pub fn key(&self) -> String {
        self.text.to_lowercase()
    }
}

/// Decoding parameters passed to an [`crate::InferenceProvider`].
#[derive(Debug, Clone, Copy)]
pub struct Sampling {
    pub temperature: f32,
    pub top_k: u32,
    pub top_p: f32,
    pub max_tokens: u32,
    /// Fixed RNG seed. Eval keeps the sampling path the model extracts best under and
    /// reports deterministic score/gate outputs rather than raw model span churn.
    pub seed: u64,
}

impl Sampling {
    /// Pure greedy (argmax). Deterministic, but for this ternary model it extracts
    /// *fewer* identifiers than seeded sampling (see eval), so it's not the eval path.
    pub fn greedy() -> Self {
        Self {
            temperature: 0.0,
            top_k: 1,
            top_p: 1.0,
            max_tokens: 512,
            seed: 0,
        }
    }
    /// PrismML Bonsai model-card sampling — interactive use.
    pub fn model_card() -> Self {
        Self {
            temperature: 0.5,
            top_k: 20,
            top_p: 0.85,
            max_tokens: 512,
            seed: 0,
        }
    }
    /// Eval decoding: model-card sampling with a **fixed seed** — recall-best *and*
    /// reproducible. This is what `airplane eval` uses.
    pub fn eval() -> Self {
        Self {
            seed: 42,
            ..Self::model_card()
        }
    }
}

impl Default for Sampling {
    fn default() -> Self {
        Self::greedy()
    }
}

/// Ground-truth expected redactions for one golden note (`eval/expected/*.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct Expected {
    pub id: String,
    #[serde(default)]
    pub clean: bool,
    #[serde(default)]
    pub redactions: Vec<ExpectedSpan>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedSpan {
    pub text: String,
    pub entity: String,
    #[serde(default)]
    pub hard: bool,
}
