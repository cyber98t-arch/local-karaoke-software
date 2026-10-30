use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisIdentity {
    pub source_sha256: String,
    pub pipeline_version: String,
    pub model_ids: Vec<String>,
    pub parameters_sha256: String,
}

impl AnalysisIdentity {
    pub fn cache_directory(&self, cache_root: &Path) -> Result<PathBuf, String> {
        if !is_sha256(&self.source_sha256) {
            return Err("source identity must be a lowercase SHA-256".to_owned());
        }
        if !is_safe_token(&self.pipeline_version) {
            return Err("pipeline version must be a nonempty safe token".to_owned());
        }
        if self.model_ids.is_empty() || self.model_ids.iter().any(|id| !is_safe_token(id)) {
            return Err("at least one safe model id is required".to_owned());
        }
        if !is_sha256(&self.parameters_sha256) {
            return Err("analysis parameters must be identified by SHA-256".to_owned());
        }

        let mut model_hasher = Sha256::new();
        for model_id in &self.model_ids {
            model_hasher.update((model_id.len() as u64).to_be_bytes());
            model_hasher.update(model_id.as_bytes());
        }
        let model_set_sha256 = format!("{:x}", model_hasher.finalize());

        Ok(cache_root
            .join(&self.source_sha256)
            .join(&self.pipeline_version)
            .join(model_set_sha256)
            .join(&self.parameters_sha256))
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisStage {
    Separation,
    Melody,
    Transcription,
    Alignment,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum AnalysisJobState {
    Queued,
    Running {
        stage: AnalysisStage,
    },
    Completed,
    Failed {
        stage: AnalysisStage,
        message: String,
        retryable: bool,
    },
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::{AnalysisIdentity, AnalysisJobState, AnalysisStage};
    use std::path::PathBuf;

    fn identity() -> AnalysisIdentity {
        AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "phase4-v1".to_owned(),
            model_ids: vec![
                "spleeter-2stems".to_owned(),
                "crepe-full".to_owned(),
                "whisper-base-multilingual".to_owned(),
            ],
            parameters_sha256: "b".repeat(64),
        }
    }

    #[test]
    fn builds_a_cache_path_from_content_and_pipeline_identity_only() {
        let cache_root = PathBuf::from("application-cache");

        let path = identity()
            .cache_directory(&cache_root)
            .expect("valid identity should produce a cache path");

        assert!(path.starts_with(&cache_root));
        assert_eq!(
            path.components().count(),
            cache_root.components().count() + 4
        );
        assert!(!path.to_string_lossy().contains("song"));
        assert!(!path.to_string_lossy().contains("whisper"));

        let mut reordered = identity();
        reordered.model_ids.reverse();
        assert_ne!(
            reordered
                .cache_directory(&cache_root)
                .expect("reordered identity should remain valid"),
            path,
            "model order is part of the pipeline identity"
        );
    }

    #[test]
    fn rejects_identity_values_that_can_escape_or_collide() {
        let mut value = identity();
        value.pipeline_version = "../outside".to_owned();
        assert!(value
            .cache_directory(PathBuf::from("cache").as_path())
            .is_err());

        let mut value = identity();
        value.model_ids.clear();
        assert!(value
            .cache_directory(PathBuf::from("cache").as_path())
            .is_err());

        let mut value = identity();
        value.parameters_sha256 = "invalid".to_owned();
        assert!(value
            .cache_directory(PathBuf::from("cache").as_path())
            .is_err());
    }

    #[test]
    fn serializes_typed_progress_without_a_source_path() {
        let state = AnalysisJobState::Failed {
            stage: AnalysisStage::Transcription,
            message: "Model process stopped; retry from transcription.".to_owned(),
            retryable: true,
        };

        let value = serde_json::to_value(state).expect("state should serialize");

        assert_eq!(value["status"], "failed");
        assert_eq!(value["stage"], "transcription");
        assert_eq!(value["retryable"], true);
        assert!(value.get("sourcePath").is_none());
    }
}
