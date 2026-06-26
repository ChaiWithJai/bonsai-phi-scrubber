//! Pack loader — the declarative, PHI-blind extension unit (ADR-005).
//!
//! A pack is five files (`pack.yaml`, recognizers, `schema.yaml`, `policy.yaml`,
//! `sink.yaml`) and a golden eval set. It can redefine identifiers / schema / policy /
//! sink — never see PHI, read the redaction map, or ship code. [`Pack::validate_blindness`]
//! enforces the "no code, creds sourced" half structurally.

use crate::rules::RulesExecutor;
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

const TRUSTED_ISSUER: &str = "https://token.actions.githubusercontent.com";
const TRUSTED_IDENTITY: &str =
    "ChaiWithJai/airplane-mode/.github/workflows/release.yml@refs/heads/main";

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
    files: Vec<ProvenanceDigest>,
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
struct ProvenanceDigest {
    path: String,
    sha256: String,
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
        validate_same_release_identity(
            "pack.signature",
            &pf.signature,
            "provenance.signature",
            &prov.signature,
        )?;
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
        validate_provenance_source_matches_identity(&prov.source, &prov.signature)?;
        validate_provenance_digests(dir, &prov.files)?;
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
        validate_signature("pack.signature", &pf.signature)?;
        validate_same_release_identity(
            "pack.signature",
            &pf.signature,
            "manifest.signature",
            &manifest.signature,
        )?;
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
    if sig.issuer != TRUSTED_ISSUER {
        bail!(
            "{path}.issuer `{}` is not trusted; expected `{TRUSTED_ISSUER}`",
            sig.issuer
        );
    }
    if sig.identity != TRUSTED_IDENTITY {
        bail!(
            "{path}.identity `{}` is not trusted; expected `{TRUSTED_IDENTITY}`",
            sig.identity
        );
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
    if !looks_like_uuid(&sig.rekor_log) {
        bail!("{path}.rekorLog must be a UUID-shaped transparency-log reference");
    }
    Ok(())
}

fn validate_same_release_identity(
    left_path: &str,
    left: &Signature,
    right_path: &str,
    right: &Signature,
) -> Result<()> {
    if left.issuer != right.issuer || left.identity != right.identity {
        bail!("{left_path} and {right_path} must use the same release identity");
    }
    Ok(())
}

fn validate_provenance_source_matches_identity(
    source: &ProvenanceSource,
    sig: &Signature,
) -> Result<()> {
    let (repo, reference) = release_identity_parts(&sig.identity)?;
    let expected_repo = format!("https://github.com/{repo}");
    if source.repository != expected_repo {
        bail!(
            "provenance.source.repository `{}` does not match signature identity `{expected_repo}`",
            source.repository
        );
    }
    if source.ref_name != reference {
        bail!(
            "provenance.source.ref `{}` does not match signature identity ref `{reference}`",
            source.ref_name
        );
    }
    Ok(())
}

fn release_identity_parts(identity: &str) -> Result<(&str, &str)> {
    let (repo_and_workflow, reference) = identity
        .split_once('@')
        .context("signature identity must include @ref")?;
    let (repo, workflow) = repo_and_workflow
        .split_once("/.github/workflows/")
        .context("signature identity must name a GitHub Actions workflow")?;
    if workflow.trim().is_empty() || reference.trim().is_empty() {
        bail!("signature identity must include workflow and ref");
    }
    Ok((repo, reference))
}

fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (idx, byte) in bytes.iter().enumerate() {
        match idx {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

fn validate_provenance_digests(dir: &Path, files: &[ProvenanceDigest]) -> Result<()> {
    if files.is_empty() {
        bail!("provenance.files must list pack file sha256 digests");
    }
    let mut seen = BTreeSet::new();
    for file in files {
        let rel = safe_pack_rel_path(&file.path)?;
        if !seen.insert(rel.clone()) {
            bail!("provenance.files repeats `{}`", file.path);
        }
        let expected = file.sha256.trim().to_ascii_lowercase();
        if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("provenance.files `{}` has invalid sha256", file.path);
        }
        let actual = sha256_file(&dir.join(&rel))
            .with_context(|| format!("hash provenance file `{}`", file.path))?;
        if actual != expected {
            bail!(
                "provenance digest mismatch for `{}`: expected {}, got {}",
                file.path,
                expected,
                actual
            );
        }
    }
    Ok(())
}

fn safe_pack_rel_path(path: &str) -> Result<PathBuf> {
    let rel = Path::new(path);
    if rel.is_absolute() {
        bail!("provenance file path `{path}` must be relative");
    }
    let mut out = PathBuf::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => out.push(part),
            _ => bail!("provenance file path `{path}` must not escape the pack"),
        }
    }
    if out.as_os_str().is_empty() {
        bail!("provenance file path must not be empty");
    }
    Ok(out)
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(format!("{digest:x}"))
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
            issuer: TRUSTED_ISSUER.into(),
            identity: TRUSTED_IDENTITY.into(),
            rekor_log: "<uuid-after-signing>".into(),
        };
        assert!(validate_signature("pack.signature", &sig).is_err());
    }

    #[test]
    fn signature_accepts_keyless_metadata() {
        let sig = Signature {
            keyless: true,
            issuer: TRUSTED_ISSUER.into(),
            identity: TRUSTED_IDENTITY.into(),
            rekor_log: "00000000-0000-4000-8000-000000000001".into(),
        };
        assert!(validate_signature("pack.signature", &sig).is_ok());
    }

    #[test]
    fn signature_rejects_untrusted_release_identity() {
        let untrusted_issuer = Signature {
            keyless: true,
            issuer: "https://example.invalid".into(),
            identity: TRUSTED_IDENTITY.into(),
            rekor_log: "00000000-0000-4000-8000-000000000001".into(),
        };
        let untrusted_identity = Signature {
            keyless: true,
            issuer: TRUSTED_ISSUER.into(),
            identity: "Other/repo/.github/workflows/release.yml@refs/heads/main".into(),
            rekor_log: "00000000-0000-4000-8000-000000000001".into(),
        };
        let bad_rekor = Signature {
            keyless: true,
            issuer: TRUSTED_ISSUER.into(),
            identity: TRUSTED_IDENTITY.into(),
            rekor_log: "not-a-uuid".into(),
        };
        assert!(validate_signature("pack.signature", &untrusted_issuer).is_err());
        assert!(validate_signature("pack.signature", &untrusted_identity).is_err());
        assert!(validate_signature("pack.signature", &bad_rekor).is_err());
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
signature: { keyless: true, issuer: "https://token.actions.githubusercontent.com", identity: "ChaiWithJai/airplane-mode/.github/workflows/release.yml@refs/heads/main", rekorLog: "00000000-0000-4000-8000-000000000001" }
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
signature: { keyless: true, issuer: "https://token.actions.githubusercontent.com", identity: "ChaiWithJai/airplane-mode/.github/workflows/release.yml@refs/heads/main", rekorLog: "00000000-0000-4000-8000-000000000002" }
"#,
        )
        .unwrap();
        assert!(Pack::validate_manifest_revocation(&manifest, &dir).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn provenance_digest_rejects_tampered_pack_file() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("airplane-provenance-test-{nonce}"));
        std::fs::create_dir_all(&dir).unwrap();
        let pack_yaml = r#"
metadata: { name: coach-session, version: 1.0.0, targetCore: ">=1.0.0 <2.0.0" }
spec: { recognizers: [], policy: policy.yaml }
signature: { keyless: true, issuer: "https://token.actions.githubusercontent.com", identity: "ChaiWithJai/airplane-mode/.github/workflows/release.yml@refs/heads/main", rekorLog: "00000000-0000-4000-8000-000000000001" }
"#;
        std::fs::write(dir.join("pack.yaml"), pack_yaml).unwrap();
        std::fs::write(dir.join("policy.yaml"), "deidentification: {}\n").unwrap();
        let pack_digest = sha256_file(&dir.join("pack.yaml")).unwrap();
        let policy_digest = sha256_file(&dir.join("policy.yaml")).unwrap();
        std::fs::write(
            dir.join("provenance.yaml"),
            format!(
                r#"
subject: {{ kind: Pack, name: coach-session, version: 1.0.0 }}
builder: {{ id: github-actions, workflow: release.yml }}
source: {{ repository: https://github.com/ChaiWithJai/airplane-mode, ref: refs/heads/main }}
files:
  - {{ path: pack.yaml, sha256: "{pack_digest}" }}
  - {{ path: policy.yaml, sha256: "{policy_digest}" }}
signature: {{ keyless: true, issuer: "https://token.actions.githubusercontent.com", identity: "ChaiWithJai/airplane-mode/.github/workflows/release.yml@refs/heads/main", rekorLog: "00000000-0000-4000-8000-000000000001" }}
"#
            ),
        )
        .unwrap();

        assert!(Pack::validate_signature_provenance(&dir).is_ok());
        std::fs::write(
            dir.join("policy.yaml"),
            "deidentification: { profile: changed }\n",
        )
        .unwrap();
        let err = Pack::validate_signature_provenance(&dir)
            .unwrap_err()
            .to_string();
        assert!(err.contains("digest mismatch"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn provenance_source_must_match_release_identity() {
        let source = ProvenanceSource {
            repository: "https://github.com/ChaiWithJai/airplane-mode".into(),
            ref_name: "refs/tags/v1.0.0".into(),
        };
        let sig = Signature {
            keyless: true,
            issuer: TRUSTED_ISSUER.into(),
            identity: TRUSTED_IDENTITY.into(),
            rekor_log: "00000000-0000-4000-8000-000000000001".into(),
        };
        assert!(validate_provenance_source_matches_identity(&source, &sig).is_err());
    }

    #[test]
    fn provenance_digest_paths_must_stay_inside_pack() {
        let file = ProvenanceDigest {
            path: "../policy.yaml".into(),
            sha256: "0".repeat(64),
        };
        assert!(validate_provenance_digests(Path::new("."), &[file]).is_err());
    }
}
