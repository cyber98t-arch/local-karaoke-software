use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::analysis_worker::TranscriptionSegment;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptManifest {
    schema_version: u32,
    raw_draft: bool,
    language: String,
    duration_ms: u64,
    model_artifact_id: String,
    model_sha256: String,
    worker_protocol_version: u32,
    segments: Vec<TranscriptionSegment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptIdentity {
    pub model_artifact_id: String,
    pub model_sha256: String,
    pub worker_protocol_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptCacheArtifact {
    pub sha256: String,
    pub segment_count: usize,
}

pub fn load_transcript_segments(
    cache_directory: &Path,
    expected: &TranscriptIdentity,
) -> io::Result<Vec<TranscriptionSegment>> {
    validate_identity(expected)?;
    let bytes = fs::read(cache_directory.join("transcript.json"))?;
    let manifest: TranscriptManifest = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    if manifest.schema_version != SCHEMA_VERSION
        || !manifest.raw_draft
        || manifest.language != "ja"
        || manifest.model_artifact_id != expected.model_artifact_id
        || manifest.model_sha256 != expected.model_sha256
        || manifest.worker_protocol_version != expected.worker_protocol_version
    {
        return Err(invalid_data("transcript cache identity does not match"));
    }
    validate_segments(manifest.duration_ms, &manifest.segments)?;
    Ok(manifest.segments)
}

pub fn commit_transcript(
    cache_directory: &Path,
    identity: &TranscriptIdentity,
    duration_ms: u64,
    segments: &[TranscriptionSegment],
) -> io::Result<TranscriptCacheArtifact> {
    validate_identity(identity)?;
    validate_segments(duration_ms, segments)?;
    let final_path = cache_directory.join("transcript.json");
    let temporary_path = cache_directory.join("transcript.json.tmp");
    if final_path.exists() {
        let artifact = validate_existing(&final_path, identity)?;
        remove_if_present(&temporary_path)?;
        return Ok(artifact);
    }

    let manifest = TranscriptManifest {
        schema_version: SCHEMA_VERSION,
        raw_draft: true,
        language: "ja".to_owned(),
        duration_ms,
        model_artifact_id: identity.model_artifact_id.clone(),
        model_sha256: identity.model_sha256.clone(),
        worker_protocol_version: identity.worker_protocol_version,
        segments: segments.to_vec(),
    };
    let mut encoded = serde_json::to_vec_pretty(&manifest).map_err(invalid_data)?;
    encoded.push(b'\n');
    fs::create_dir_all(cache_directory)?;
    remove_if_present(&temporary_path)?;
    let mut temporary = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)?;
    temporary.write_all(&encoded)?;
    temporary.sync_all()?;
    drop(temporary);
    fs::rename(&temporary_path, &final_path)?;
    Ok(TranscriptCacheArtifact {
        sha256: sha256(&encoded),
        segment_count: segments.len(),
    })
}

fn validate_existing(
    path: &Path,
    expected: &TranscriptIdentity,
) -> io::Result<TranscriptCacheArtifact> {
    let bytes = fs::read(path)?;
    let manifest: TranscriptManifest = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    if manifest.schema_version != SCHEMA_VERSION
        || !manifest.raw_draft
        || manifest.language != "ja"
        || manifest.model_artifact_id != expected.model_artifact_id
        || manifest.model_sha256 != expected.model_sha256
        || manifest.worker_protocol_version != expected.worker_protocol_version
    {
        return Err(invalid_data(
            "existing transcript cache has an unsupported identity or format",
        ));
    }
    validate_segments(manifest.duration_ms, &manifest.segments)?;
    Ok(TranscriptCacheArtifact {
        sha256: sha256(&bytes),
        segment_count: manifest.segments.len(),
    })
}

fn validate_identity(identity: &TranscriptIdentity) -> io::Result<()> {
    let safe_id = !identity.model_artifact_id.is_empty()
        && !matches!(identity.model_artifact_id.as_str(), "." | "..")
        && identity
            .model_artifact_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'));
    let hash = identity.model_sha256.len() == 64
        && identity
            .model_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !safe_id || !hash || identity.worker_protocol_version == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid transcript identity",
        ));
    }
    Ok(())
}

fn validate_segments(duration_ms: u64, segments: &[TranscriptionSegment]) -> io::Result<()> {
    if duration_ms == 0 || segments.is_empty() {
        return Err(invalid_data("transcript must have a duration and segments"));
    }
    let mut previous_end = 0;
    for segment in segments {
        if segment.start_ms >= segment.end_ms
            || segment.end_ms > duration_ms
            || segment.start_ms < previous_end
            || segment.text.trim().is_empty()
        {
            return Err(invalid_data(
                "transcript segments must be ordered, bounded, and nonempty",
            ));
        }
        previous_end = segment.end_ms;
    }
    Ok(())
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::{commit_transcript, load_transcript_segments, TranscriptIdentity};
    use crate::analysis_worker::TranscriptionSegment;
    use serde_json::Value;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "hibiki-transcript-cache-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn reloads_validated_segments_for_the_alignment_stage() {
        let directory = TestDirectory::new();
        let identity = TranscriptIdentity {
            model_artifact_id: "whisper-base".into(),
            model_sha256: "a".repeat(64),
            worker_protocol_version: 2,
        };
        let segments = vec![TranscriptionSegment {
            start_ms: 0,
            end_ms: 500,
            text: "今日は".into(),
        }];
        commit_transcript(&directory.0, &identity, 600, &segments).unwrap();

        assert_eq!(
            load_transcript_segments(&directory.0, &identity).unwrap(),
            segments
        );
        let wrong = TranscriptIdentity {
            model_sha256: "b".repeat(64),
            ..identity
        };
        assert!(load_transcript_segments(&directory.0, &wrong).is_err());
    }
    fn identity() -> TranscriptIdentity {
        TranscriptIdentity {
            model_artifact_id: "whisper-base-v1".into(),
            model_sha256: "a".repeat(64),
            worker_protocol_version: 2,
        }
    }
    fn segments() -> Vec<TranscriptionSegment> {
        vec![
            TranscriptionSegment {
                start_ms: 0,
                end_ms: 700,
                text: "今日は".into(),
            },
            TranscriptionSegment {
                start_ms: 800,
                end_ms: 1500,
                text: "晴れです".into(),
            },
        ]
    }

    #[test]
    fn commits_versioned_raw_draft_inside_cache() {
        let directory = TestDirectory::new();
        let artifact = commit_transcript(&directory.0, &identity(), 1600, &segments()).unwrap();
        let json: Value =
            serde_json::from_slice(&fs::read(directory.0.join("transcript.json")).unwrap())
                .unwrap();
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["rawDraft"], true);
        assert_eq!(json["language"], "ja");
        assert_eq!(json["workerProtocolVersion"], 2);
        assert_eq!(artifact.segment_count, 2);
        assert!(!directory.0.join("transcript.json.tmp").exists());
    }

    #[test]
    fn preserves_an_existing_compatible_transcript() {
        let directory = TestDirectory::new();
        let first = commit_transcript(&directory.0, &identity(), 1600, &segments()).unwrap();
        let second = commit_transcript(
            &directory.0,
            &identity(),
            2000,
            &[TranscriptionSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "変更".into(),
            }],
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(second.segment_count, 2);
    }

    #[test]
    fn rejects_invalid_segments_without_committing() {
        let directory = TestDirectory::new();
        let invalid = [TranscriptionSegment {
            start_ms: 500,
            end_ms: 400,
            text: "".into(),
        }];
        assert!(commit_transcript(&directory.0, &identity(), 1000, &invalid).is_err());
        assert!(!directory.0.join("transcript.json").exists());
    }
}
