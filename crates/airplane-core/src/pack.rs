//! Pack loader — the declarative, PHI-blind extension unit (ADR-005).
//!
//! A pack is five files (`pack.yaml`, recognizers, `schema.yaml`, `policy.yaml`,
//! `sink.yaml`) and a golden eval set. It can redefine identifiers / schema / policy /
//! sink — never see PHI, read the redaction map, or ship code. [`Pack::validate_blindness`]
//! enforces the "no code, creds sourced" half structurally.

use crate::rules::RulesExecutor;
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct PackFile {
    metadata: Meta,
    spec: Spec,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Meta {
    name: String,
    version: String,
    #[serde(rename = "targetCore", default)]
    target_core: String,
}

#[derive(Debug, Deserialize)]
struct Spec {
    recognizers: Vec<String>,
    #[serde(default)]
    policy: String,
}

#[derive(Debug, Deserialize)]
pub struct Policy {
    pub deidentification: Deid,
    #[serde(default)]
    pub followup: Option<Followup>,
}

#[derive(Debug, Deserialize)]
pub struct Deid {
    pub profile: String,
    #[serde(rename = "recallThreshold")]
    pub recall_threshold: f64,
    #[serde(rename = "onResidual")]
    pub on_residual: String,
}

#[derive(Debug, Deserialize)]
pub struct Followup {
    #[serde(default)]
    pub reward: Option<Reward>,
    #[serde(rename = "scopeBoundary", default)]
    pub scope_boundary: Option<ScopeBoundary>,
}

#[derive(Debug, Deserialize)]
pub struct Reward {
    #[serde(rename = "autonomySignals", default)]
    pub autonomy_signals: Vec<String>,
    #[serde(rename = "usedSignals", default)]
    pub used_signals: Vec<String>,
    #[serde(rename = "forbiddenTerms", default)]
    pub forbidden_terms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeBoundary {
    #[serde(rename = "escalationRequired", default)]
    pub escalation_required: bool,
    #[serde(rename = "onClinicalRisk", default)]
    pub on_clinical_risk: String,
}

/// A loaded pack: its rules executor (built-ins + recognizers) and its policy.
pub struct Pack {
    pub name: String,
    pub dir: PathBuf,
    pub rules: RulesExecutor,
    pub policy: Policy,
    policy_raw: serde_yaml::Value,
}

impl Pack {
    pub fn load(dir: &Path) -> Result<Self> {
        let pf: PackFile = serde_yaml::from_str(
            &std::fs::read_to_string(dir.join("pack.yaml")).context("read pack.yaml")?,
        )
        .context("parse pack.yaml")?;

        let mut rules = RulesExecutor::new();
        for rel in &pf.spec.recognizers {
            rules.load_recognizer_file(&dir.join(rel))?;
        }

        let policy_rel = if pf.spec.policy.is_empty() {
            "policy.yaml".to_string()
        } else {
            pf.spec.policy.clone()
        };
        let policy_text = std::fs::read_to_string(dir.join(&policy_rel)).context("read policy")?;
        let policy_raw: serde_yaml::Value =
            serde_yaml::from_str(&policy_text).context("parse policy")?;
        let policy: Policy = serde_yaml::from_str(&policy_text).context("parse policy")?;

        Ok(Self {
            name: pf.metadata.name,
            dir: dir.to_path_buf(),
            rules,
            policy,
            policy_raw,
        })
    }

    /// Structural pack-blindness check (the `pack-blindness` gate):
    /// credentials must be sourced (not literal secrets), and the pack carries no code.
    pub fn validate_blindness(dir: &Path) -> Result<()> {
        let sink_path = dir.join("sink.yaml");
        let sink = std::fs::read_to_string(&sink_path).context("read sink.yaml")?;
        if !sink.contains("source:") {
            bail!("sink.yaml has no credential `source:` — secrets must be sourced at runtime, never stored in a pack");
        }
        for path in walk(dir) {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(
                    ext,
                    "sh" | "py" | "js" | "ts" | "rb" | "exe" | "dylib" | "so" | "wasm"
                ) {
                    bail!(
                        "pack contains executable file {} — packs are declaration only",
                        path.display()
                    );
                }
            }
        }
        Ok(())
    }

    /// `reward-lint`: reward references may use autonomy signals only, never the
    /// configured engagement/dependence terms. `forbiddenTerms` itself is the deny-list
    /// declaration, so it is not treated as a violation.
    pub fn validate_reward_lint(&self) -> Result<()> {
        let reward = self
            .policy
            .followup
            .as_ref()
            .and_then(|f| f.reward.as_ref())
            .context("policy.followup.reward is required")?;
        if reward.forbidden_terms.is_empty() {
            bail!("policy.followup.reward.forbiddenTerms is required");
        }

        let forbidden: BTreeSet<String> = reward
            .forbidden_terms
            .iter()
            .map(|s| normalize_term(s))
            .collect();
        let references = if reward.used_signals.is_empty() {
            &reward.autonomy_signals
        } else {
            &reward.used_signals
        };
        if references.is_empty() {
            bail!("policy.followup.reward must reference at least one autonomy signal");
        }
        for signal in references {
            if forbidden.contains(&normalize_term(signal)) {
                bail!("reward references forbidden engagement term `{signal}`");
            }
        }
        Ok(())
    }

    /// `scope-boundary`: coach packs must include a human escalation path and must
    /// not claim clinical/therapy behavior in policy text.
    pub fn validate_scope_boundary(&self) -> Result<()> {
        let boundary = self
            .policy
            .followup
            .as_ref()
            .and_then(|f| f.scope_boundary.as_ref())
            .context("policy.followup.scopeBoundary is required")?;
        if !boundary.escalation_required {
            bail!("policy.followup.scopeBoundary.escalationRequired must be true");
        }
        if boundary.on_clinical_risk.trim().is_empty() {
            bail!("policy.followup.scopeBoundary.onClinicalRisk must name the escalation action");
        }

        let mut claims = Vec::new();
        collect_clinical_claims(&self.policy_raw, &mut Vec::new(), &mut claims);
        if !claims.is_empty() {
            bail!(
                "clinical-claim language is not allowed in a coach pack: {}",
                claims.join(", ")
            );
        }
        Ok(())
    }
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.extend(walk(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}

fn normalize_term(s: &str) -> String {
    s.trim().to_ascii_lowercase().replace('-', "_")
}

fn collect_clinical_claims(
    value: &serde_yaml::Value,
    path: &mut Vec<String>,
    claims: &mut Vec<String>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let key = k.as_str().unwrap_or("<key>").to_string();
                path.push(key);
                collect_clinical_claims(v, path, claims);
                path.pop();
            }
        }
        serde_yaml::Value::Sequence(items) => {
            for item in items {
                collect_clinical_claims(item, path, claims);
            }
        }
        serde_yaml::Value::String(s) => {
            let field = path.join(".");
            if field.ends_with("forbiddenTerms") {
                return;
            }
            let lower = s.to_ascii_lowercase();
            const FORBIDDEN: &[&str] = &[
                "diagnose",
                "diagnosis",
                "treat",
                "treatment",
                "therapy",
                "therapist",
                "psychotherapy",
                "prescribe",
                "medication",
                "patient",
                "clinical intervention",
            ];
            for term in FORBIDDEN {
                if lower.contains(term) {
                    claims.push(format!("{field} contains `{term}`"));
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack_from_policy(policy_text: &str) -> Pack {
        let policy_raw: serde_yaml::Value = serde_yaml::from_str(policy_text).unwrap();
        let policy: Policy = serde_yaml::from_str(policy_text).unwrap();
        Pack {
            name: "test".into(),
            dir: PathBuf::new(),
            rules: RulesExecutor::new(),
            policy,
            policy_raw,
        }
    }

    #[test]
    fn reward_lint_rejects_engagement_signal() {
        let pack = pack_from_policy(
            r#"
deidentification: { profile: safe-harbor, recallThreshold: 0.99, onResidual: block }
followup:
  reward:
    autonomySignals: [commitment_completed]
    usedSignals: [engagement]
    forbiddenTerms: [engagement, retention, session_count, app_opens]
  scopeBoundary: { escalationRequired: true, onClinicalRisk: surface_human_escalation }
"#,
        );
        assert!(pack.validate_reward_lint().is_err());
    }

    #[test]
    fn scope_boundary_requires_escalation() {
        let pack = pack_from_policy(
            r#"
deidentification: { profile: safe-harbor, recallThreshold: 0.99, onResidual: block }
followup:
  reward:
    autonomySignals: [commitment_completed]
    forbiddenTerms: [engagement, retention, session_count, app_opens]
  scopeBoundary: { escalationRequired: false, onClinicalRisk: surface_human_escalation }
"#,
        );
        assert!(pack.validate_scope_boundary().is_err());
    }

    #[test]
    fn policy_with_autonomy_reward_and_escalation_passes() {
        let pack = pack_from_policy(
            r#"
deidentification: { profile: safe-harbor, recallThreshold: 0.99, onResidual: block }
followup:
  reward:
    autonomySignals: [commitment_completed, self_initiated]
    forbiddenTerms: [engagement, retention, session_count, app_opens]
  scopeBoundary: { escalationRequired: true, onClinicalRisk: surface_human_escalation }
"#,
        );
        assert!(pack.validate_reward_lint().is_ok());
        assert!(pack.validate_scope_boundary().is_ok());
    }
}
