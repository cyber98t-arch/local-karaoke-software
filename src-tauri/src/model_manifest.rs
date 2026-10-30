use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelFamily {
    Spleeter,
    Crepe,
    Whisper,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelArtifactManifest {
    pub schema_version: u32,
    pub id: String,
    pub family: ModelFamily,
    pub version: String,
    pub source_url: String,
    pub sha256: String,
    pub byte_size: u64,
    pub license_spdx: String,
    pub license_url: String,
}

impl ModelArtifactManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported model manifest schema version".to_owned());
        }
        if !is_safe_token(&self.id) {
            return Err("model id must be a nonempty safe token".to_owned());
        }
        if !is_safe_token(&self.version) {
            return Err("model version must be a nonempty safe token".to_owned());
        }
        if !is_https_url(&self.source_url) {
            return Err("model source must use HTTPS".to_owned());
        }
        if self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("model SHA-256 must be 64 lowercase hexadecimal characters".to_owned());
        }
        if self.byte_size == 0 {
            return Err("model byte size must be known and nonzero".to_owned());
        }
        if self.license_spdx.trim().is_empty() {
            return Err("model license SPDX identifier is required".to_owned());
        }
        if !is_https_url(&self.license_url) {
            return Err("model license evidence must use HTTPS".to_owned());
        }
        Ok(())
    }
}

fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn is_https_url(value: &str) -> bool {
    value
        .strip_prefix("https://")
        .is_some_and(|remainder| !remainder.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{ModelArtifactManifest, ModelFamily};

    fn valid_manifest() -> ModelArtifactManifest {
        ModelArtifactManifest {
            schema_version: 1,
            id: "whisper-base-multilingual".to_owned(),
            family: ModelFamily::Whisper,
            version: "upstream-revision".to_owned(),
            source_url: "https://example.invalid/model.bin".to_owned(),
            sha256: "a".repeat(64),
            byte_size: 148_000_000,
            license_spdx: "MIT".to_owned(),
            license_url: "https://example.invalid/LICENSE".to_owned(),
        }
    }

    #[test]
    fn accepts_complete_pinned_artifact_metadata() {
        valid_manifest()
            .validate()
            .expect("complete pinned metadata should be accepted");
    }

    #[test]
    fn rejects_untrusted_or_incomplete_artifact_metadata() {
        let mut manifest = valid_manifest();
        manifest.id = "../outside".to_owned();
        assert!(manifest.validate().is_err());

        let mut manifest = valid_manifest();
        manifest.source_url = "http://example.invalid/model.bin".to_owned();
        assert!(manifest.validate().is_err());

        let mut manifest = valid_manifest();
        manifest.sha256 = "not-a-sha256".to_owned();
        assert!(manifest.validate().is_err());

        let mut manifest = valid_manifest();
        manifest.byte_size = 0;
        assert!(manifest.validate().is_err());

        let mut manifest = valid_manifest();
        manifest.license_spdx.clear();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn serializes_a_stable_camel_case_boundary() {
        let value = serde_json::to_value(valid_manifest()).expect("manifest should serialize");

        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["family"], "whisper");
        assert_eq!(value["byteSize"], 148_000_000_u64);
        assert!(value.get("source_url").is_none());
    }

    #[test]
    fn bundled_evaluation_manifests_are_complete_and_valid() {
        let encoded = [
            include_str!("../model-manifests/spleeter-2stems-v1.4.0.json"),
            include_str!("../model-manifests/crepe-tiny-bb29b8d.json"),
            include_str!("../model-manifests/whisper-base-5359861.json"),
        ];

        let manifests: Vec<ModelArtifactManifest> = encoded
            .into_iter()
            .map(|value| serde_json::from_str(value).expect("bundled manifest should decode"))
            .collect();

        for manifest in &manifests {
            manifest
                .validate()
                .expect("bundled manifest should pass provenance validation");
            assert!(manifest.byte_size < 500_000_000);
        }
        assert_eq!(
            manifests
                .iter()
                .map(|manifest| &manifest.family)
                .collect::<Vec<_>>(),
            vec![
                &ModelFamily::Spleeter,
                &ModelFamily::Crepe,
                &ModelFamily::Whisper,
            ]
        );
    }
}
