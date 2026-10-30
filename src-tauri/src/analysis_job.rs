use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

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

impl AnalysisJobState {
    pub fn start_stage(&mut self, stage: AnalysisStage) -> Result<(), String> {
        let can_start = matches!(self, Self::Queued | Self::Running { .. })
            || matches!(
                self,
                Self::Failed {
                    retryable: true,
                    ..
                }
            );
        if !can_start {
            return Err("analysis job is terminal or not retryable".to_owned());
        }
        *self = Self::Running { stage };
        Ok(())
    }

    pub fn fail(&mut self, message: String, retryable: bool) -> Result<(), String> {
        let stage = match self {
            Self::Running { stage } => *stage,
            _ => return Err("only a running analysis stage can fail".to_owned()),
        };
        if message.trim().is_empty() {
            return Err("analysis failure requires an actionable message".to_owned());
        }
        *self = Self::Failed {
            stage,
            message,
            retryable,
        };
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        if !matches!(self, Self::Queued | Self::Running { .. }) {
            return Err("only an active analysis job can be cancelled".to_owned());
        }
        *self = Self::Cancelled;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), String> {
        if !matches!(self, Self::Running { .. }) {
            return Err("only a running analysis job can complete".to_owned());
        }
        *self = Self::Completed;
        Ok(())
    }
}

pub fn append_job_state(
    cache_root: &Path,
    identity: &AnalysisIdentity,
    next_state: &AnalysisJobState,
) -> Result<PathBuf, String> {
    let directory = identity.cache_directory(cache_root)?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let latest = latest_state_snapshot(&directory)?;
    match &latest {
        None if !matches!(next_state, AnalysisJobState::Queued) => {
            return Err("the first analysis job state must be queued".to_owned());
        }
        Some((_, previous)) if !transition_allowed(previous, next_state) => {
            return Err("invalid analysis job state transition".to_owned());
        }
        _ => {}
    }

    let sequence = latest.map_or(1, |(sequence, _)| sequence + 1);
    let final_path = directory.join(format!("state-{sequence:06}.json"));
    let temporary_path = directory.join(format!("state-{sequence:06}.json.tmp"));
    match fs::remove_file(&temporary_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }

    let encoded = serde_json::to_vec_pretty(next_state).map_err(|error| error.to_string())?;
    let mut temporary = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)
        .map_err(|error| error.to_string())?;
    temporary
        .write_all(&encoded)
        .and_then(|()| temporary.write_all(b"\n"))
        .and_then(|()| temporary.sync_all())
        .map_err(|error| error.to_string())?;
    drop(temporary);
    fs::rename(&temporary_path, &final_path).map_err(|error| error.to_string())?;
    Ok(final_path)
}

fn latest_state_snapshot(directory: &Path) -> Result<Option<(u64, AnalysisJobState)>, String> {
    let mut latest: Option<(u64, PathBuf)> = None;
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let Some(sequence) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(snapshot_sequence)
        else {
            continue;
        };
        if latest
            .as_ref()
            .is_none_or(|(latest_sequence, _)| sequence > *latest_sequence)
        {
            latest = Some((sequence, path));
        }
    }

    latest
        .map(|(sequence, path)| {
            serde_json::from_reader(File::open(path).map_err(|error| error.to_string())?)
                .map(|state| (sequence, state))
                .map_err(|error| error.to_string())
        })
        .transpose()
}

fn snapshot_sequence(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("state-")?.strip_suffix(".json")?;
    (digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
}

fn transition_allowed(previous: &AnalysisJobState, next: &AnalysisJobState) -> bool {
    match previous {
        AnalysisJobState::Queued => matches!(
            next,
            AnalysisJobState::Running { .. } | AnalysisJobState::Cancelled
        ),
        AnalysisJobState::Running { .. } => matches!(
            next,
            AnalysisJobState::Running { .. }
                | AnalysisJobState::Completed
                | AnalysisJobState::Failed { .. }
                | AnalysisJobState::Cancelled
        ),
        AnalysisJobState::Failed {
            retryable: true, ..
        } => matches!(next, AnalysisJobState::Running { .. }),
        AnalysisJobState::Completed
        | AnalysisJobState::Cancelled
        | AnalysisJobState::Failed {
            retryable: false, ..
        } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{append_job_state, AnalysisIdentity, AnalysisJobState, AnalysisStage};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibiki-analysis-job-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).expect("test directory should be removable");
        }
    }

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

    #[test]
    fn allows_cancellation_but_keeps_terminal_states_terminal() {
        let mut state = AnalysisJobState::Queued;
        state
            .start_stage(AnalysisStage::Separation)
            .expect("a queued job should start");
        assert_eq!(
            state,
            AnalysisJobState::Running {
                stage: AnalysisStage::Separation
            }
        );

        state.cancel().expect("a running job should cancel");
        assert_eq!(state, AnalysisJobState::Cancelled);
        assert!(state.start_stage(AnalysisStage::Melody).is_err());
        assert!(state.complete().is_err());
    }

    #[test]
    fn records_actionable_stage_failure_and_allows_retry() {
        let mut state = AnalysisJobState::Running {
            stage: AnalysisStage::Melody,
        };
        state
            .fail("Pitch process stopped".to_owned(), true)
            .expect("a running stage should fail");

        assert_eq!(
            state,
            AnalysisJobState::Failed {
                stage: AnalysisStage::Melody,
                message: "Pitch process stopped".to_owned(),
                retryable: true,
            }
        );
        state
            .start_stage(AnalysisStage::Melody)
            .expect("a retryable failure should restart");
        state.complete().expect("a running job should complete");
        assert_eq!(state, AnalysisJobState::Completed);
    }

    #[test]
    fn appends_atomic_state_snapshots_and_removes_stale_temporary_files() {
        let directory = TestDirectory::new();
        let identity = identity();
        let job_directory = identity
            .cache_directory(&directory.path)
            .expect("identity should be valid");
        fs::create_dir_all(&job_directory).expect("job directory should be created");
        fs::write(job_directory.join("state-000001.json.tmp"), b"interrupted")
            .expect("stale temporary state should be created");

        let queued_path = append_job_state(&directory.path, &identity, &AnalysisJobState::Queued)
            .expect("initial queued state should commit");
        let running_path = append_job_state(
            &directory.path,
            &identity,
            &AnalysisJobState::Running {
                stage: AnalysisStage::Separation,
            },
        )
        .expect("running state should commit");

        assert!(queued_path.ends_with("state-000001.json"));
        assert!(running_path.ends_with("state-000002.json"));
        assert!(!job_directory.join("state-000001.json.tmp").exists());
    }

    #[test]
    fn preserves_completed_state_against_later_writes() {
        let directory = TestDirectory::new();
        let identity = identity();
        append_job_state(&directory.path, &identity, &AnalysisJobState::Queued)
            .expect("queued state should commit");
        append_job_state(
            &directory.path,
            &identity,
            &AnalysisJobState::Running {
                stage: AnalysisStage::Alignment,
            },
        )
        .expect("running state should commit");
        append_job_state(&directory.path, &identity, &AnalysisJobState::Completed)
            .expect("completed state should commit");

        let error = append_job_state(
            &directory.path,
            &identity,
            &AnalysisJobState::Running {
                stage: AnalysisStage::Separation,
            },
        )
        .expect_err("completed state must be terminal");

        assert!(error.contains("transition"));
        assert_eq!(
            fs::read_dir(
                identity
                    .cache_directory(&directory.path)
                    .expect("identity should remain valid")
            )
            .expect("job directory should remain readable")
            .filter_map(Result::ok)
            .filter(|entry| entry
                .path()
                .extension()
                .is_some_and(|value| value == "json"))
            .count(),
            3
        );
    }
}
