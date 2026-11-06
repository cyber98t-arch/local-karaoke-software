use std::path::{Path, PathBuf};

use crate::{
    aligned_lyrics_pipeline::align_lyrics_to_cache,
    analysis_job::{AnalysisIdentity, AnalysisStage},
    analysis_pipeline::{AnalysisStageRunner, StageOutcome},
    analysis_worker::{AnalysisWorkerBackend, AnalysisWorkerService},
    crepe_inference::PitchInference,
    melody_cache::MelodyModelIdentity,
    melody_pipeline::extract_melody_to_cache,
    separation_pipeline::separate_source_to_cache,
    spleeter_inference::SpectrogramInference,
    stem_cache::load_cached_vocals,
    transcript_cache::{load_transcript_segments, TranscriptIdentity},
    transcript_pipeline::{transcribe_japanese_to_cache, TranscriptionInput},
};

pub struct LocalAnalysisRunner<'a, S, P, W> {
    pub separation: &'a mut S,
    pub pitch: &'a mut P,
    pub transcription: &'a AnalysisWorkerService<W>,
    pub source_path: &'a Path,
    pub cache_root: &'a Path,
    pub identity: &'a AnalysisIdentity,
    pub melody_model: &'a MelodyModelIdentity,
    pub transcript_model: &'a TranscriptIdentity,
    pub whisper_model_path: &'a Path,
    pub authoritative_lyrics: &'a str,
    pub request_id: &'a str,
}

impl<S, P, W> LocalAnalysisRunner<'_, S, P, W>
where
    S: SpectrogramInference,
    P: PitchInference,
    W: AnalysisWorkerBackend,
{
    fn cache_directory(&self) -> Result<PathBuf, String> {
        self.identity.cache_directory(self.cache_root)
    }
}

impl<S, P, W> AnalysisStageRunner for LocalAnalysisRunner<'_, S, P, W>
where
    S: SpectrogramInference,
    P: PitchInference,
    W: AnalysisWorkerBackend,
{
    fn run_stage(
        &mut self,
        stage: AnalysisStage,
        cancellation_requested: &mut dyn FnMut() -> bool,
    ) -> Result<StageOutcome, String> {
        if cancellation_requested() {
            return Ok(StageOutcome::Cancelled);
        }
        match stage {
            AnalysisStage::Separation => {
                separate_source_to_cache(
                    self.separation,
                    self.source_path,
                    self.cache_root,
                    self.identity,
                )?;
            }
            AnalysisStage::Melody => {
                let (sample_rate, vocals) = load_cached_vocals(&self.cache_directory()?)
                    .map_err(|error| format!("cached vocals could not be loaded: {error}"))?;
                extract_melody_to_cache(
                    self.pitch,
                    &vocals,
                    sample_rate,
                    self.cache_root,
                    self.identity,
                    self.melody_model,
                )?;
            }
            AnalysisStage::Transcription => {
                let vocals_path = self.cache_directory()?.join("vocals.flac");
                transcribe_japanese_to_cache(
                    self.transcription,
                    self.cache_root,
                    self.identity,
                    TranscriptionInput {
                        request_id: self.request_id,
                        audio_path: &vocals_path,
                        model_path: self.whisper_model_path,
                        model: self.transcript_model,
                    },
                )?;
            }
            AnalysisStage::Alignment => {
                let transcript =
                    load_transcript_segments(&self.cache_directory()?, self.transcript_model)
                        .map_err(|error| {
                            format!("cached transcript could not be loaded: {error}")
                        })?;
                align_lyrics_to_cache(
                    self.authoritative_lyrics,
                    &transcript,
                    self.cache_root,
                    self.identity,
                )?;
            }
        }
        Ok(StageOutcome::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::LocalAnalysisRunner;
    use crate::{
        analysis_job::{AnalysisIdentity, AnalysisJobState},
        analysis_pipeline::run_analysis_pipeline,
        analysis_worker::{
            AnalysisWorkerBackend, AnalysisWorkerService, TranscriptionResult,
            TranscriptionSegment, WorkerRequest, WorkerResponse, ANALYSIS_WORKER_PROTOCOL_VERSION,
        },
        crepe_inference::PitchInference,
        crepe_signal::CREPE_BINS,
        melody_cache::MelodyModelIdentity,
        spleeter_inference::{SpectrogramInference, StemEstimates},
        transcript_cache::TranscriptIdentity,
    };
    use ndarray::{Array2, Array4};
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
                "hibiki-local-runner-{}-{}",
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

    struct VocalOnly;
    impl SpectrogramInference for VocalOnly {
        fn infer(&mut self, input: &Array4<f32>) -> Result<StemEstimates, String> {
            Ok(StemEstimates {
                vocals: input.clone(),
                accompaniment: Array4::zeros(input.raw_dim()),
            })
        }
    }
    struct KnownPitch;
    impl PitchInference for KnownPitch {
        fn infer(&mut self, frames: &Array2<f32>) -> Result<Array2<f32>, String> {
            let mut output = Array2::zeros((frames.nrows(), CREPE_BINS));
            output.column_mut(180).fill(0.9);
            Ok(output)
        }
    }
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
                    duration_ms: 1000,
                    segments: vec![TranscriptionSegment {
                        start_ms: 0,
                        end_ms: 900,
                        text: "今日は".into(),
                    }],
                }),
                error: None,
            })
        }
    }

    #[test]
    fn runs_all_real_cache_handoffs_without_modifying_the_source() {
        let directory = TestDirectory::new();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/metadata.wav");
        let source_before = fs::read(&source).unwrap();
        let model = directory.0.join("whisper.bin");
        fs::write(&model, b"model").unwrap();
        let identity = AnalysisIdentity {
            source_sha256: crate::cache::content_sha256(&source).unwrap(),
            pipeline_version: "phase4-v1".into(),
            model_ids: vec!["spleeter".into(), "crepe".into(), "whisper".into()],
            parameters_sha256: "a".repeat(64),
        };
        let melody_model = MelodyModelIdentity {
            artifact_id: "crepe".into(),
            sha256: "b".repeat(64),
        };
        let transcript_model = TranscriptIdentity {
            model_artifact_id: "whisper".into(),
            model_sha256: "c".repeat(64),
            worker_protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
        };
        let mut separation = VocalOnly;
        let mut pitch = KnownPitch;
        let worker = AnalysisWorkerService::new(TranscriptWorker);
        let mut runner = LocalAnalysisRunner {
            separation: &mut separation,
            pitch: &mut pitch,
            transcription: &worker,
            source_path: &source,
            cache_root: &directory.0,
            identity: &identity,
            melody_model: &melody_model,
            transcript_model: &transcript_model,
            whisper_model_path: &model,
            authoritative_lyrics: "今日は。",
            request_id: "song-analysis",
        };

        assert_eq!(
            run_analysis_pipeline(&directory.0, &identity, &mut runner, || false).unwrap(),
            AnalysisJobState::Completed
        );
        let output = identity.cache_directory(&directory.0).unwrap();
        for name in [
            "stems.json",
            "vocals.flac",
            "accompaniment.flac",
            "melody.json",
            "transcript.json",
            "aligned-lyrics.json",
        ] {
            assert!(output.join(name).is_file(), "missing {name}");
        }
        assert_eq!(fs::read(source).unwrap(), source_before);
    }
}
