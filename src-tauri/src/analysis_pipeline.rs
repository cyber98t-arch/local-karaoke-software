use std::path::Path;

use crate::analysis_job::{
    append_job_state_in_directory, latest_job_state_in_directory, AnalysisIdentity,
    AnalysisJobState, AnalysisStage,
};

const STAGES: [AnalysisStage; 4] = [
    AnalysisStage::Separation,
    AnalysisStage::Melody,
    AnalysisStage::Transcription,
    AnalysisStage::Alignment,
];

pub trait AnalysisStageRunner {
    fn run_stage(
        &mut self,
        stage: AnalysisStage,
        cancellation_requested: &mut dyn FnMut() -> bool,
    ) -> Result<StageOutcome, String>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageOutcome {
    Completed,
    Cancelled,
}

pub fn run_analysis_pipeline<R, C>(
    cache_root: &Path,
    identity: &AnalysisIdentity,
    runner: &mut R,
    cancellation_requested: C,
) -> Result<AnalysisJobState, String>
where
    R: AnalysisStageRunner,
    C: FnMut() -> bool,
{
    let directory = identity.cache_directory(cache_root)?;
    run_analysis_pipeline_in_directory(&directory, runner, cancellation_requested)
}

pub fn run_analysis_pipeline_in_directory<R, C>(
    state_directory: &Path,
    runner: &mut R,
    mut cancellation_requested: C,
) -> Result<AnalysisJobState, String>
where
    R: AnalysisStageRunner,
    C: FnMut() -> bool,
{
    let latest = latest_job_state_in_directory(state_directory)?;
    let first_stage = match latest {
        None => {
            append_job_state_in_directory(state_directory, &AnalysisJobState::Queued)?;
            AnalysisStage::Separation
        }
        Some(AnalysisJobState::Queued) => AnalysisStage::Separation,
        Some(AnalysisJobState::Running { stage }) => stage,
        Some(AnalysisJobState::Failed {
            stage,
            retryable: true,
            ..
        }) => stage,
        Some(AnalysisJobState::Completed) => return Ok(AnalysisJobState::Completed),
        Some(AnalysisJobState::Cancelled) => {
            return Err("cancelled analysis jobs cannot be resumed".to_owned())
        }
        Some(AnalysisJobState::Failed {
            retryable: false, ..
        }) => return Err("non-retryable analysis jobs cannot be resumed".to_owned()),
    };
    let first_index = STAGES
        .iter()
        .position(|stage| *stage == first_stage)
        .expect("all analysis stages must be in the pipeline");

    for stage in STAGES.into_iter().skip(first_index) {
        if cancellation_requested() {
            let state = AnalysisJobState::Cancelled;
            append_job_state_in_directory(state_directory, &state)?;
            return Ok(state);
        }
        let running = AnalysisJobState::Running { stage };
        append_job_state_in_directory(state_directory, &running)?;
        match runner.run_stage(stage, &mut cancellation_requested) {
            Ok(StageOutcome::Completed) => {}
            Ok(StageOutcome::Cancelled) => {
                let cancelled = AnalysisJobState::Cancelled;
                append_job_state_in_directory(state_directory, &cancelled)?;
                return Ok(cancelled);
            }
            Err(message) => {
                let mut failed = running;
                failed.fail(message, true)?;
                append_job_state_in_directory(state_directory, &failed)?;
                return Ok(failed);
            }
        }
    }
    let completed = AnalysisJobState::Completed;
    append_job_state_in_directory(state_directory, &completed)?;
    Ok(completed)
}

#[cfg(test)]
mod tests {
    use super::{run_analysis_pipeline, AnalysisStageRunner, StageOutcome};
    use crate::analysis_job::{
        latest_job_state, AnalysisIdentity, AnalysisJobState, AnalysisStage,
    };
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
                "hibiki-analysis-pipeline-{}-{}",
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
    fn identity() -> AnalysisIdentity {
        AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "phase4-v1".into(),
            model_ids: vec!["spleeter".into(), "crepe".into(), "whisper".into()],
            parameters_sha256: "b".repeat(64),
        }
    }

    struct RecordingRunner {
        stages: Vec<AnalysisStage>,
        fail_once: Option<AnalysisStage>,
    }
    impl AnalysisStageRunner for RecordingRunner {
        fn run_stage(
            &mut self,
            stage: AnalysisStage,
            _cancellation_requested: &mut dyn FnMut() -> bool,
        ) -> Result<StageOutcome, String> {
            self.stages.push(stage);
            if self.fail_once == Some(stage) {
                self.fail_once = None;
                Err("generated stage failure; retry is safe".into())
            } else {
                Ok(StageOutcome::Completed)
            }
        }
    }

    #[test]
    fn runs_all_local_stages_in_order_and_completes() {
        let cache = TestDirectory::new();
        let mut runner = RecordingRunner {
            stages: Vec::new(),
            fail_once: None,
        };
        let state = run_analysis_pipeline(&cache.0, &identity(), &mut runner, || false).unwrap();
        assert_eq!(state, AnalysisJobState::Completed);
        assert_eq!(
            runner.stages,
            vec![
                AnalysisStage::Separation,
                AnalysisStage::Melody,
                AnalysisStage::Transcription,
                AnalysisStage::Alignment
            ]
        );
        assert_eq!(
            latest_job_state(&cache.0, &identity()).unwrap(),
            Some(AnalysisJobState::Completed)
        );
    }

    #[test]
    fn resumes_from_the_retryable_failed_stage() {
        let cache = TestDirectory::new();
        let mut first = RecordingRunner {
            stages: Vec::new(),
            fail_once: Some(AnalysisStage::Transcription),
        };
        let failed = run_analysis_pipeline(&cache.0, &identity(), &mut first, || false).unwrap();
        assert!(matches!(
            failed,
            AnalysisJobState::Failed {
                stage: AnalysisStage::Transcription,
                retryable: true,
                ..
            }
        ));
        let mut retry = RecordingRunner {
            stages: Vec::new(),
            fail_once: None,
        };
        let completed = run_analysis_pipeline(&cache.0, &identity(), &mut retry, || false).unwrap();
        assert_eq!(completed, AnalysisJobState::Completed);
        assert_eq!(
            retry.stages,
            vec![AnalysisStage::Transcription, AnalysisStage::Alignment]
        );
    }

    #[test]
    fn records_cancellation_before_starting_the_next_stage() {
        let cache = TestDirectory::new();
        let mut runner = RecordingRunner {
            stages: Vec::new(),
            fail_once: None,
        };
        let mut checks = 0;
        let state = run_analysis_pipeline(&cache.0, &identity(), &mut runner, || {
            checks += 1;
            checks == 2
        })
        .unwrap();
        assert_eq!(state, AnalysisJobState::Cancelled);
        assert_eq!(runner.stages, vec![AnalysisStage::Separation]);
    }

    #[test]
    fn accepts_cooperative_cancellation_from_a_running_stage() {
        struct CancellingRunner;
        impl AnalysisStageRunner for CancellingRunner {
            fn run_stage(
                &mut self,
                _stage: AnalysisStage,
                _cancellation_requested: &mut dyn FnMut() -> bool,
            ) -> Result<StageOutcome, String> {
                Ok(StageOutcome::Cancelled)
            }
        }
        let cache = TestDirectory::new();
        let state =
            run_analysis_pipeline(&cache.0, &identity(), &mut CancellingRunner, || false).unwrap();
        assert_eq!(state, AnalysisJobState::Cancelled);
    }
}
