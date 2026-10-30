use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const ANALYSIS_WORKER_PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkerOperation {
    SyntheticSeparationProbe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerRequest {
    pub protocol_version: u32,
    pub request_id: String,
    pub operation: WorkerOperation,
    pub output_directory: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerProbeMetrics {
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerResponse {
    pub protocol_version: u32,
    pub request_id: String,
    pub success: bool,
    pub metrics: Option<WorkerProbeMetrics>,
    pub error: Option<String>,
}

pub trait AnalysisWorkerBackend {
    fn execute(&self, request: &WorkerRequest) -> Result<WorkerResponse, String>;
}

pub struct AnalysisWorkerService<B> {
    backend: B,
}

impl<B> AnalysisWorkerService<B>
where
    B: AnalysisWorkerBackend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn run_synthetic_probe(
        &self,
        cache_root: &Path,
        request_id: &str,
    ) -> Result<WorkerProbeMetrics, String> {
        if !is_safe_token(request_id) {
            return Err("worker request id must be a nonempty safe token".to_owned());
        }
        let request = WorkerRequest {
            protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
            request_id: request_id.to_owned(),
            operation: WorkerOperation::SyntheticSeparationProbe,
            output_directory: cache_root.join("worker-probes").join(request_id),
        };
        let response = self.backend.execute(&request)?;
        if response.protocol_version != ANALYSIS_WORKER_PROTOCOL_VERSION {
            return Err("analysis worker protocol version mismatch".to_owned());
        }
        if response.request_id != request.request_id {
            return Err("analysis worker response id mismatch".to_owned());
        }
        if !response.success {
            return Err(response
                .error
                .filter(|message| !message.trim().is_empty())
                .unwrap_or_else(|| "analysis worker failed without an error message".to_owned()));
        }
        let metrics = response
            .metrics
            .ok_or_else(|| "analysis worker succeeded without probe metrics".to_owned())?;
        if metrics.sample_rate == 0 || metrics.channels == 0 || metrics.frames == 0 {
            return Err("analysis worker returned invalid probe metrics".to_owned());
        }
        Ok(metrics)
    }
}

fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::{
        AnalysisWorkerBackend, AnalysisWorkerService, WorkerProbeMetrics, WorkerRequest,
        WorkerResponse, ANALYSIS_WORKER_PROTOCOL_VERSION,
    };
    use std::{cell::RefCell, path::PathBuf};

    struct FakeWorker {
        observed: RefCell<Option<WorkerRequest>>,
        response_version: u32,
        response_request_id: String,
    }

    impl AnalysisWorkerBackend for FakeWorker {
        fn execute(&self, request: &WorkerRequest) -> Result<WorkerResponse, String> {
            self.observed.replace(Some(request.clone()));
            Ok(WorkerResponse {
                protocol_version: self.response_version,
                request_id: self.response_request_id.clone(),
                success: true,
                metrics: Some(WorkerProbeMetrics {
                    sample_rate: 16_000,
                    channels: 2,
                    frames: 16_000,
                }),
                error: None,
            })
        }
    }

    #[test]
    fn constructs_a_cache_owned_versioned_probe_request() {
        let worker = FakeWorker {
            observed: RefCell::new(None),
            response_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
            response_request_id: "probe-1".to_owned(),
        };
        let service = AnalysisWorkerService::new(worker);

        let metrics = service
            .run_synthetic_probe(PathBuf::from("application-cache").as_path(), "probe-1")
            .expect("matching worker response should succeed");

        assert_eq!(metrics.sample_rate, 16_000);
        let request = service
            .backend
            .observed
            .borrow()
            .clone()
            .expect("worker should observe a request");
        assert_eq!(request.protocol_version, ANALYSIS_WORKER_PROTOCOL_VERSION);
        assert_eq!(request.request_id, "probe-1");
        assert_eq!(
            request.output_directory,
            PathBuf::from("application-cache")
                .join("worker-probes")
                .join("probe-1")
        );
    }

    #[test]
    fn rejects_unsafe_ids_and_unmatched_responses() {
        let service = AnalysisWorkerService::new(FakeWorker {
            observed: RefCell::new(None),
            response_version: ANALYSIS_WORKER_PROTOCOL_VERSION + 1,
            response_request_id: "different".to_owned(),
        });

        assert!(service
            .run_synthetic_probe(PathBuf::from("cache").as_path(), "../outside")
            .is_err());
        assert!(service
            .run_synthetic_probe(PathBuf::from("cache").as_path(), "probe-1")
            .is_err());
    }
}
