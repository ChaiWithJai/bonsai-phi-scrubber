//! The four **ports** the core depends on (dependency inversion — ADR-014).
//!
//! The core never imports a platform. Each shell injects concrete implementations:
//! - CLI: llama-server HTTP · file store · stdin · Slack/mock
//! - iOS: mlx-swift · Secure Enclave · ASR · Slack Block Kit
//! - MCP: llama-server HTTP · ephemeral · tool-call · tool-result

use crate::model::Sampling;
use anyhow::Result;
use serde_json::Value;

/// A request to the model. The core builds it (prompt + schema + sampling);
/// the adapter owns transport and constrained decoding.
pub struct InferenceRequest<'a> {
    pub system: &'a str,
    pub user: &'a str,
    /// JSON Schema the output must conform to. CLI enforces it server-side
    /// (`response_format`); iOS must enforce it client-side (R2).
    pub json_schema: &'a Value,
    pub sampling: Sampling,
}

/// The model is a PORT, never in the core (ADR-014).
pub trait InferenceProvider {
    /// Return the raw completion string (text that should be JSON matching the schema).
    /// The core strips `<think>` / fences and parses — adapters return raw output.
    fn complete(&self, req: &InferenceRequest<'_>) -> Result<String>;
}

/// Holds the redaction map and raw input. On device this is the Secure Enclave /
/// Keychain; it must never cross the network.
pub trait SecureStore {
    fn put(&self, id: &str, blob: &str) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<String>>;
}

/// Source of raw notes (ASR on device, stdin on the CLI).
pub trait Capture {
    fn next_note(&mut self) -> Result<Option<String>>;
}

/// Receives a **de-identified record only** — structurally cannot see PHI.
pub trait Sink {
    fn deliver(&self, record: &str) -> Result<()>;
}
