use std::path::{Path, PathBuf};

use crate::{
    aligned_lyrics_pipeline::align_lyrics_to_directory,
    analysis_audio::write_pcm16_wav,
    analysis_job::{AnalysisIdentity, AnalysisStage},
    analysis_pipeline::{AnalysisStageRunner, StageOutcome},
    analysis_worker::{AnalysisWorkerBackend, AnalysisWorkerService},
    crepe_inference::PitchInference,
    melody_cache::{load_cached_melody, MelodyModelIdentity},
    melody_pipeline::extract_melody_to_cache,
    separation_pipeline::separate_source_to_cache,
    spleeter_inference::SpectrogramInference,
    stem_cache::load_cached_vocals,
    transcript_cache::{load_transcript_segments, TranscriptIdentity},
    transcript_pipeline::{transcribe_japanese_to_cache_cancellable, TranscriptionInput},
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
    pub lyric_revision_directory: &'a Path,
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
                let directory = self.cache_directory()?;
                if directory.join("stems.json").exists() {
                    load_cached_vocals(&directory)
                        .map_err(|error| format!("cached stems could not be loaded: {error}"))?;
                } else {
                    separate_source_to_cache(
                        self.separation,
                        self.source_path,
                        self.cache_root,
                        self.identity,
                    )?;
                }
            }
            AnalysisStage::Melody => {
                let directory = self.cache_directory()?;
                if directory.join("melody.json").exists() {
                    load_cached_melody(&directory, self.melody_model)
                        .map_err(|error| format!("cached melody could not be loaded: {error}"))?;
                } else {
                    let (sample_rate, vocals) = load_cached_vocals(&directory)
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
            }
            AnalysisStage::Transcription => {
                let directory = self.cache_directory()?;
                if directory.join("transcript.json").exists() {
                    load_transcript_segments(&directory, self.transcript_model).map_err(
                        |error| format!("cached transcript could not be loaded: {error}"),
                    )?;
                } else {
                    let (sample_rate, vocals) = load_cached_vocals(&directory)
                        .map_err(|error| format!("cached vocals could not be loaded: {error}"))?;
                    let vocals_path = write_pcm16_wav(
                        &directory.join("transcription-input.wav"),
                        sample_rate,
                        &vocals,
                    )
                    .map_err(|error| format!("transcription WAV could not be prepared: {error}"))?;
                    let artifact = transcribe_japanese_to_cache_cancellable(
                        self.transcription,
                        self.cache_root,
                        self.identity,
                        TranscriptionInput {
                            request_id: self.request_id,
                            audio_path: &vocals_path,
                            model_path: self.whisper_model_path,
                            model: self.transcript_model,
                        },
                        cancellation_requested,
                    )?;
                    if artifact.is_none() {
                        return Ok(StageOutcome::Cancelled);
                    }
                }
            }
            AnalysisStage::Alignment => {
                let transcript =
                    load_transcript_segments(&self.cache_directory()?, self.transcript_model)
                        .map_err(|error| {
                            format!("cached transcript could not be loaded: {error}")
                        })?;
                align_lyrics_to_directory(
                    self.authoritative_lyrics,
                    &transcript,
                    self.lyric_revision_directory,
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
        analysis_pipeline::run_analysis_pipeline_in_directory,
        analysis_worker::{
            AnalysisWorkerBackend, AnalysisWorkerService, TranscriptionResult,
            TranscriptionSegment, WorkerRequest, WorkerResponse, ANALYSIS_WORKER_PROTOCOL_VERSION,
        },
        crepe_inference::PitchInference,
        crepe_signal::CREPE_BINS,
        lyric_revision::lyric_revision_directory,
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
    struct UnexpectedSeparation;
    impl SpectrogramInference for UnexpectedSeparation {
        fn infer(&mut self, _input: &Array4<f32>) -> Result<StemEstimates, String> {
            panic!("valid cached stems must skip separation inference")
        }
    }
    struct UnexpectedPitch;
    impl PitchInference for UnexpectedPitch {
        fn infer(&mut self, _frames: &Array2<f32>) -> Result<Array2<f32>, String> {
            panic!("valid cached melody must skip pitch inference")
        }
    }
    struct UnexpectedTranscriptWorker;
    impl AnalysisWorkerBackend for UnexpectedTranscriptWorker {
        fn execute(&self, _request: &WorkerRequest) -> Result<WorkerResponse, String> {
            panic!("valid cached transcript must skip transcription")
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
        let first_revision = lyric_revision_directory(&directory.0, &identity, "今日は。").unwrap();
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
            lyric_revision_directory: &first_revision,
            request_id: "song-analysis",
        };

        assert_eq!(
            run_analysis_pipeline_in_directory(&first_revision, &mut runner, || false).unwrap(),
            AnalysisJobState::Completed
        );
        let output = identity.cache_directory(&directory.0).unwrap();
        for name in [
            "stems.json",
            "vocals.flac",
            "accompaniment.flac",
            "melody.json",
            "transcript.json",
        ] {
            assert!(output.join(name).is_file(), "missing {name}");
        }
        assert!(first_revision.join("aligned-lyrics.json").is_file());

        let second_revision =
            lyric_revision_directory(&directory.0, &identity, "今日は！").unwrap();
        let mut unexpected_separation = UnexpectedSeparation;
        let mut unexpected_pitch = UnexpectedPitch;
        let unexpected_worker = AnalysisWorkerService::new(UnexpectedTranscriptWorker);
        let mut corrected_runner = LocalAnalysisRunner {
            separation: &mut unexpected_separation,
            pitch: &mut unexpected_pitch,
            transcription: &unexpected_worker,
            source_path: &source,
            cache_root: &directory.0,
            identity: &identity,
            melody_model: &melody_model,
            transcript_model: &transcript_model,
            whisper_model_path: &model,
            authoritative_lyrics: "今日は！",
            lyric_revision_directory: &second_revision,
            request_id: "corrected-song-analysis",
        };
        assert_eq!(
            run_analysis_pipeline_in_directory(&second_revision, &mut corrected_runner, || false)
                .unwrap(),
            AnalysisJobState::Completed
        );
        assert!(second_revision.join("aligned-lyrics.json").is_file());
        assert_ne!(first_revision, second_revision);
        assert_eq!(fs::read(source).unwrap(), source_before);
    }
}
