use crate::analysis_job::AnalysisJobState;
use serde::Serialize;
use std::{
    collections::HashMap,
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

#[derive(Default)]
pub struct AnalysisCommandState {
    jobs: Mutex<HashMap<String, ActiveAnalysis>>,
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
