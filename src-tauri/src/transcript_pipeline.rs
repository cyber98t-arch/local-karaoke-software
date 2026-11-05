use std::path::Path;

use crate::{
    analysis_job::AnalysisIdentity,
    analysis_worker::{AnalysisWorkerBackend, AnalysisWorkerService},
    transcript_cache::{commit_transcript, TranscriptCacheArtifact, TranscriptIdentity},
};

pub struct TranscriptionInput<'a> {
    pub request_id: &'a str,
    pub audio_path: &'a Path,
    pub model_path: &'a Path,
    pub model: &'a TranscriptIdentity,
}

pub fn transcribe_japanese_to_cache<B: AnalysisWorkerBackend>(
    worker: &AnalysisWorkerService<B>,
    cache_root: &Path,
    identity: &AnalysisIdentity,
    input: TranscriptionInput<'_>,
) -> Result<TranscriptCacheArtifact, String> {
    let result = worker.run_japanese_transcription(
        cache_root,
        identity,
        input.request_id,
        input.audio_path,
        input.model_path,
        &input.model.model_sha256,
    )?;
    let directory = identity.cache_directory(cache_root)?;
    commit_transcript(
        &directory,
        input.model,
        result.duration_ms,
        &result.segments,
    )
    .map_err(|error| format!("transcript cache commit failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{transcribe_japanese_to_cache, TranscriptionInput};
    use crate::{
        analysis_job::AnalysisIdentity,
        analysis_worker::{
            AnalysisWorkerBackend, AnalysisWorkerService, TranscriptionResult,
            TranscriptionSegment, WorkerRequest, WorkerResponse, ANALYSIS_WORKER_PROTOCOL_VERSION,
        },
        transcript_cache::TranscriptIdentity,
    };
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    struct TranscriptWorker;
    impl AnalysisWorkerBackend for TranscriptWorker {
        fn execute(&self, request: &WorkerRequest) -> Result<WorkerResponse, String> {
            Ok(WorkerResponse {
                protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
                request_id: request.request_id.clone(),
                success: true,
                metrics: None,
                separation: None,
                transcription: Some(TranscriptionResult {
                    language: "ja".into(),
                    duration_ms: 1200,
                    segments: vec![TranscriptionSegment {
                        start_ms: 100,
                        end_ms: 900,
                        text: "テスト".into(),
                    }],
                }),
                error: None,
            })
        }
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "hibiki-transcript-pipeline-{}-{}",
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
    fn commits_worker_result_only_below_identity_cache() {
        let cache = TestDirectory::new();
        let files = TestDirectory::new();
        let audio = files.0.join("audio.wav");
        let model_path = files.0.join("model.bin");
        fs::write(&audio, b"read-only fixture").unwrap();
        fs::write(&model_path, b"model").unwrap();
        let before = fs::read(&audio).unwrap();
        let identity = AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "transcription-v1".into(),
            model_ids: vec!["whisper-base-v1".into()],
            parameters_sha256: "b".repeat(64),
        };
        let model = TranscriptIdentity {
            model_artifact_id: "whisper-base-v1".into(),
            model_sha256: "c".repeat(64),
            worker_protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
        };
        let output = identity.cache_directory(&cache.0).unwrap();
        let artifact = transcribe_japanese_to_cache(
            &AnalysisWorkerService::new(TranscriptWorker),
            &cache.0,
            &identity,
            TranscriptionInput {
                request_id: "transcribe-1",
                audio_path: &audio,
                model_path: &model_path,
                model: &model,
            },
        )
        .unwrap();
        assert_eq!(artifact.segment_count, 1);
        assert!(output.join("transcript.json").is_file());
        assert_eq!(fs::read(&audio).unwrap(), before);
    }
}
