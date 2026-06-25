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
    #[serde(default)]
    signature: Signature,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Signature {
    #[serde(default)]
    keyless: bool,
    #[serde(default)]
    issuer: String,
    #[serde(default)]
    identity: String,
    #[serde(default)]
    rekor_log: String,
}

#[derive(Debug, Deserialize)]
struct ProvenanceFile {
    subject: ProvenanceSubject,
    builder: ProvenanceBuilder,
    source: ProvenanceSource,
    #[serde(default)]
    signature: Signature,
}

#[derive(Debug, Deserialize)]
struct ProvenanceSubject {
    kind: String,
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct ProvenanceBuilder {
    id: String,
    workflow: String,
}

#[derive(Debug, Deserialize)]
struct ProvenanceSource {
    repository: String,
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile {
    sequence: u64,
    previous_sequence: u64,
    current: ManifestCurrent,
    #[serde(default)]
    revoked: Vec<RevokedArtifact>,
    #[serde(default)]
    signature: Signature,
}

#[derive(Debug, Deserialize)]
struct ManifestCurrent {
    core: String,
    model: String,
    pack: String,
}

#[derive(Debug, Deserialize)]
struct RevokedArtifact {
    #[serde(default)]
    pack: String,
    #[serde(default)]
    model: String,
    reason: String,
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

    /// `signature/provenance`: the pack must carry keyless signing metadata and a
    /// PHI-free provenance document whose subject matches the pack.
    pub fn validate_signature_provenance(dir: &Path) -> Result<()> {
        let pf: PackFile = serde_yaml::from_str(
            &std::fs::read_to_string(dir.join("pack.yaml")).context("read pack.yaml")?,
        )
        .context("parse pack.yaml")?;
        if pf.metadata.target_core.trim().is_empty() {
            bail!("pack.metadata.targetCore must be set for provenance validation");
        }
        validate_signature("pack.signature", &pf.signature)?;

        let prov: ProvenanceFile = serde_yaml::from_str(
            &std::fs::read_to_string(dir.join("provenance.yaml"))
                .context("read provenance.yaml")?,
        )
        .context("parse provenance.yaml")?;
        validate_signature("provenance.signature", &prov.signature)?;
        if prov.subject.kind != "Pack"
            || prov.subject.name != pf.metadata.name
            || prov.subject.version != pf.metadata.version
        {
            bail!("provenance subject does not match pack metadata");
        }
        if prov.builder.id.trim().is_empty() || prov.builder.workflow.trim().is_empty() {
            bail!("provenance.builder must identify the release workflow");
        }
        if prov.source.repository.trim().is_empty() || prov.source.ref_name.trim().is_empty() {
            bail!("provenance.source must identify repository and ref");
        }
        Ok(())
    }

    /// `manifest/revocation`: the manifest must be keyless-signed, monotonic, point
    /// at the current pack, and must not revoke the current pack/model.
    pub fn validate_manifest_revocation(manifest_path: &Path, pack_dir: &Path) -> Result<()> {
        let manifest: ManifestFile = serde_yaml::from_str(
            &std::fs::read_to_string(manifest_path).context("read manifest.yaml")?,
        )
        .context("parse manifest.yaml")?;
        validate_signature("manifest.signature", &manifest.signature)?;
        if manifest.sequence <= manifest.previous_sequence {
            bail!("manifest sequence must increase monotonically");
        }
        if manifest.current.core.trim().is_empty() || manifest.current.model.trim().is_empty() {
            bail!("manifest current core/model must be set");
        }

        let pf: PackFile = serde_yaml::from_str(
            &std::fs::read_to_string(pack_dir.join("pack.yaml")).context("read pack.yaml")?,
        )
        .context("parse pack.yaml")?;
        let current_pack = format!("{}@{}", pf.metadata.name, pf.metadata.version);
        if manifest.current.pack != current_pack {
            bail!(
                "manifest current pack `{}` does not match loaded pack `{current_pack}`",
                manifest.current.pack
            );
        }
        for revoked in &manifest.revoked {
            if revoked.reason.trim().is_empty() {
                bail!("revocation entries must include a reason");
            }
            if revoked.pack == current_pack {
                bail!("manifest revokes current pack `{current_pack}`");
            }
            if !revoked.model.trim().is_empty() && revoked.model == manifest.current.model {
                bail!(
                    "manifest revokes current model `{}`",
                    manifest.current.model
                );
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

fn validate_signature(path: &str, sig: &Signature) -> Result<()> {
    if !sig.keyless {
        bail!("{path}.keyless must be true");
    }
    for (field, value) in [
        ("issuer", sig.issuer.as_str()),
        ("identity", sig.identity.as_str()),
        ("rekorLog", sig.rekor_log.as_str()),
    ] {
        if value.trim().is_empty() || value.contains('<') || value.contains('>') {
            bail!("{path}.{field} must be populated with non-placeholder metadata");
        }
    }
    Ok(())
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
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn signature_rejects_placeholders() {
        let sig = Signature {
            keyless: true,
            issuer: "https://token.actions.githubusercontent.com".into(),
            identity: "repo/workflow".into(),
            rekor_log: "<uuid-after-signing>".into(),
        };
        assert!(validate_signature("pack.signature", &sig).is_err());
    }

    #[test]
    fn signature_accepts_keyless_metadata() {
        let sig = Signature {
            keyless: true,
            issuer: "https://token.actions.githubusercontent.com".into(),
            identity: "repo/workflow".into(),
            rekor_log: "00000000-0000-4000-8000-000000000001".into(),
        };
        assert!(validate_signature("pack.signature", &sig).is_ok());
    }

    #[test]
    fn manifest_rejects_revoked_current_pack() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("airplane-pack-test-{nonce}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("pack.yaml"),
            r#"
metadata: { name: coach-session, version: 1.0.0, targetCore: ">=1.0.0 <2.0.0" }
spec: { recognizers: [] }
signature: { keyless: true, issuer: test, identity: test, rekorLog: "00000000-0000-4000-8000-000000000001" }
"#,
        )
        .unwrap();
        let manifest = dir.join("manifest.yaml");
        std::fs::write(
            &manifest,
            r#"
sequence: 2
previousSequence: 1
current: { core: 0.1.0, model: "ternary-bonsai-1.7b@local-gguf", pack: "coach-session@1.0.0" }
revoked:
  - { pack: "coach-session@1.0.0", reason: "recall regression" }
signature: { keyless: true, issuer: test, identity: test, rekorLog: "00000000-0000-4000-8000-000000000002" }
"#,
        )
        .unwrap();
        assert!(Pack::validate_manifest_revocation(&manifest, &dir).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
