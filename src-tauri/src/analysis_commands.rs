use crate::analysis_job::AnalysisJobState;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisStatus {
    pub song_id: String,
    pub state: AnalysisJobState,
}

struct ActiveAnalysis {
    state: AnalysisJobState,
    cancellation: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
pub struct AnalysisCommandState {
    jobs: Arc<Mutex<HashMap<String, ActiveAnalysis>>>,
}

#[derive(Clone, Debug)]
pub struct AnalysisStorage {
    pub cache_root: PathBuf,
}

impl AnalysisStorage {
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }
}

impl AnalysisCommandState {
    pub fn begin(&self, song_id: &str) -> Result<Arc<AtomicBool>, String> {
        validate_song_id(song_id)?;
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| "The analysis state is unavailable.".to_owned())?;
        if jobs.get(song_id).is_some_and(|job| {
            matches!(
                job.state,
                AnalysisJobState::Queued | AnalysisJobState::Running { .. }
            )
        }) {
            return Err("Analysis is already running for this song.".to_owned());
        }
        let cancellation = Arc::new(AtomicBool::new(false));
        jobs.insert(
            song_id.to_owned(),
            ActiveAnalysis {
                state: AnalysisJobState::Queued,
                cancellation: cancellation.clone(),
            },
        );
        Ok(cancellation)
    }

    pub fn update(&self, song_id: &str, state: AnalysisJobState) -> Result<(), String> {
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| "The analysis state is unavailable.".to_owned())?;
        let job = jobs
            .get_mut(song_id)
            .ok_or_else(|| "No analysis job exists for this song.".to_owned())?;
        job.state = state;
        Ok(())
    }

    pub fn status(&self, song_id: &str) -> Result<Option<AnalysisStatus>, String> {
        validate_song_id(song_id)?;
        let jobs = self
            .jobs
            .lock()
            .map_err(|_| "The analysis state is unavailable.".to_owned())?;
        Ok(jobs.get(song_id).map(|job| AnalysisStatus {
            song_id: song_id.to_owned(),
            state: job.state.clone(),
        }))
    }

    pub fn cancel(&self, song_id: &str) -> Result<AnalysisStatus, String> {
        validate_song_id(song_id)?;
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| "The analysis state is unavailable.".to_owned())?;
        let job = jobs
            .get_mut(song_id)
            .ok_or_else(|| "No analysis job exists for this song.".to_owned())?;
        if !matches!(
            job.state,
            AnalysisJobState::Queued | AnalysisJobState::Running { .. }
        ) {
            return Err("Only an active analysis job can be cancelled.".to_owned());
        }
        job.cancellation.store(true, Ordering::Release);
        Ok(AnalysisStatus {
            song_id: song_id.to_owned(),
            state: job.state.clone(),
        })
    }

    #[cfg(feature = "native-onnx")]
    fn fail_current(&self, song_id: &str, message: String) -> Result<(), String> {
        let stage = self
            .status(song_id)?
            .and_then(|status| match status.state {
                AnalysisJobState::Running { stage } => Some(stage),
                _ => None,
            })
            .unwrap_or(crate::analysis_job::AnalysisStage::Separation);
        self.update(
            song_id,
            AnalysisJobState::Failed {
                stage,
                message,
                retryable: true,
            },
        )
    }
}

#[tauri::command]
pub fn get_analysis_status(
    song_id: String,
    state: tauri::State<'_, AnalysisCommandState>,
) -> Result<Option<AnalysisStatus>, String> {
    state.status(&song_id)
}

#[tauri::command]
pub fn cancel_analysis(
    song_id: String,
    state: tauri::State<'_, AnalysisCommandState>,
) -> Result<AnalysisStatus, String> {
    state.cancel(&song_id)
}

#[tauri::command]
pub async fn start_analysis(
    song_id: String,
    library: tauri::State<'_, crate::library_commands::LibraryState>,
    lyrics: tauri::State<'_, crate::lyrics_commands::LyricsState>,
    jobs: tauri::State<'_, AnalysisCommandState>,
    storage: tauri::State<'_, AnalysisStorage>,
) -> Result<AnalysisStatus, String> {
    #[cfg(not(feature = "native-onnx"))]
    {
        let _ = (&song_id, &library, &lyrics, &jobs, &storage);
        return Err("Native analysis support is not enabled in this build.".to_owned());
    }

    #[cfg(feature = "native-onnx")]
    {
        let source_path = library.resolve_source_for_analysis(&song_id)?;
        let lyrics = lyrics
            .lyrics_for_song(&song_id)?
            .ok_or_else(|| "Add authoritative lyrics before starting analysis.".to_owned())?;
        let config = tauri::async_runtime::spawn_blocking(
            crate::analysis_runtime::NativeAnalysisConfig::from_development_environment,
        )
        .await
        .map_err(|error| format!("analysis preflight stopped unexpectedly: {error}"))??;
        let cancellation = jobs.begin(&song_id)?;
        let registry = jobs.inner().clone();
        let cache_root = storage.cache_root.clone();
        let task_song_id = song_id.clone();
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(message) = run_native_analysis(
                &task_song_id,
                &source_path,
                &cache_root,
                &lyrics.text,
                &config,
                &registry,
                &cancellation,
            ) {
                let _ = registry.fail_current(&task_song_id, message);
            }
        });
        Ok(AnalysisStatus {
            song_id,
            state: AnalysisJobState::Queued,
        })
    }
}

#[cfg(feature = "native-onnx")]
fn run_native_analysis(
    song_id: &str,
    source_path: &std::path::Path,
    cache_root: &std::path::Path,
    authoritative_lyrics: &str,
    config: &crate::analysis_runtime::NativeAnalysisConfig,
    registry: &AnalysisCommandState,
    cancellation: &AtomicBool,
) -> Result<(), String> {
    use crate::{
        analysis_job::{AnalysisIdentity, AnalysisStage},
        analysis_pipeline::{run_analysis_pipeline, AnalysisStageRunner, StageOutcome},
        analysis_worker::{AnalysisWorkerService, ANALYSIS_WORKER_PROTOCOL_VERSION},
        crepe_inference::NativeCrepeSession,
        local_analysis_runner::LocalAnalysisRunner,
        melody_cache::MelodyModelIdentity,
        spleeter_inference::NativeOrtSession,
        transcript_cache::TranscriptIdentity,
        whisper_cpp_backend::WhisperCppBackend,
    };
    use std::time::Duration;

    let identity = AnalysisIdentity {
        source_sha256: song_id.to_owned(),
        pipeline_version: "phase4-v1".into(),
        model_ids: vec![
            "spleeter-2stems-onnx-dev-v1".into(),
            "crepe-tiny-onnx-dev-v1".into(),
            "whisper-base-5359861".into(),
        ],
        parameters_sha256: "9ea689566c7c6a6803323a6131377046dd09d06affc295f412524deed0844eb0"
            .into(),
    };
    let melody_model = MelodyModelIdentity {
        artifact_id: "crepe-tiny-onnx-dev-v1".into(),
        sha256: crate::analysis_runtime::CREPE_SHA256.into(),
    };
    let transcript_model = TranscriptIdentity {
        model_artifact_id: "whisper-base-5359861".into(),
        model_sha256: crate::analysis_runtime::WHISPER_MODEL_SHA256.into(),
        worker_protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
    };
    let mut separation = NativeOrtSession::from_file(&config.spleeter_model)?;
    let mut pitch = NativeCrepeSession::from_file(&config.crepe_model)?;
    let transcription = AnalysisWorkerService::new(WhisperCppBackend::new(
        config.whisper_cli.clone(),
        crate::analysis_runtime::WHISPER_CLI_SHA256.into(),
        crate::analysis_runtime::WHISPER_CLI_BYTES,
        Duration::from_secs(60 * 60),
    ));
    let runner = LocalAnalysisRunner {
        separation: &mut separation,
        pitch: &mut pitch,
        transcription: &transcription,
        source_path,
        cache_root,
        identity: &identity,
        melody_model: &melody_model,
        transcript_model: &transcript_model,
        whisper_model_path: &config.whisper_model,
        authoritative_lyrics,
        request_id: "song-analysis",
    };
    struct ReportingRunner<'a, R> {
        inner: R,
        registry: &'a AnalysisCommandState,
        song_id: &'a str,
    }
    impl<R: AnalysisStageRunner> AnalysisStageRunner for ReportingRunner<'_, R> {
        fn run_stage(
            &mut self,
            stage: AnalysisStage,
            cancel: &mut dyn FnMut() -> bool,
        ) -> Result<StageOutcome, String> {
            self.registry
                .update(self.song_id, AnalysisJobState::Running { stage })?;
            self.inner.run_stage(stage, cancel)
        }
    }
    let mut reporting = ReportingRunner {
        inner: runner,
        registry,
        song_id,
    };
    let final_state = run_analysis_pipeline(cache_root, &identity, &mut reporting, || {
        cancellation.load(Ordering::Acquire)
    })?;
    registry.update(song_id, final_state)
}

fn validate_song_id(song_id: &str) -> Result<(), String> {
    if song_id.len() == 64
        && song_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err("song id must be a lowercase SHA-256".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::AnalysisCommandState;
    use crate::analysis_job::{AnalysisJobState, AnalysisStage};
    use std::sync::atomic::Ordering;

    fn song_id() -> String {
        "a".repeat(64)
    }

    #[test]
    fn owns_path_free_job_state_and_rejects_duplicate_active_work() {
        let state = AnalysisCommandState::default();
        state.begin(&song_id()).unwrap();
        state
            .update(
                &song_id(),
                AnalysisJobState::Running {
                    stage: AnalysisStage::Separation,
                },
            )
            .unwrap();

        assert!(state.begin(&song_id()).is_err());
        let encoded = serde_json::to_string(&state.status(&song_id()).unwrap()).unwrap();
        assert!(encoded.contains("separation"));
        assert!(!encoded.contains('/') && !encoded.contains("\\\\"));
    }

    #[test]
    fn cancellation_is_cooperative_and_terminal_jobs_can_be_retried() {
        let state = AnalysisCommandState::default();
        let cancellation = state.begin(&song_id()).unwrap();
        state.cancel(&song_id()).unwrap();
        assert!(cancellation.load(Ordering::Acquire));

        state
            .update(&song_id(), AnalysisJobState::Cancelled)
            .unwrap();
        assert!(state.cancel(&song_id()).is_err());
        assert!(state.begin(&song_id()).is_ok());
    }
}
