use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub const ANALYSIS_WORKER_PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WorkerOperation {
    SyntheticSeparationProbe,
    SeparateTwoStems {
        input_audio_path: PathBuf,
        model_path: PathBuf,
        expected_source_sha256: String,
        expected_model_sha256: String,
    },
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
pub struct SeparationArtifact {
    pub file_name: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeparationResult {
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: u64,
    pub vocals: SeparationArtifact,
    pub accompaniment: SeparationArtifact,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerResponse {
    pub protocol_version: u32,
    pub request_id: String,
    pub success: bool,
    pub metrics: Option<WorkerProbeMetrics>,
    pub separation: Option<SeparationResult>,
    pub error: Option<String>,
}

pub trait AnalysisWorkerBackend {
    fn execute(&self, request: &WorkerRequest) -> Result<WorkerResponse, String>;
}

pub struct LocalProcessWorker {
    executable: PathBuf,
    script: PathBuf,
    timeout: Duration,
    max_output_bytes: usize,
}

impl LocalProcessWorker {
    pub fn new(
        executable: PathBuf,
        script: PathBuf,
        timeout: Duration,
        max_output_bytes: usize,
    ) -> Self {
        Self {
            executable,
            script,
            timeout,
            max_output_bytes,
        }
    }
}

impl AnalysisWorkerBackend for LocalProcessWorker {
    fn execute(&self, request: &WorkerRequest) -> Result<WorkerResponse, String> {
        if self.timeout.is_zero() || self.max_output_bytes == 0 {
            return Err("analysis worker bounds must be nonzero".to_owned());
        }
        fs::create_dir_all(&request.output_directory).map_err(|error| {
            format!("analysis worker output directory could not be created: {error}")
        })?;

        let mut child = Command::new(&self.executable)
            .arg(&self.script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("analysis worker could not start: {error}"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "analysis worker stdout was not available".to_owned())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "analysis worker stderr was not available".to_owned())?;
        let stdout_reader = read_capped(stdout, self.max_output_bytes);
        let stderr_reader = read_capped(stderr, self.max_output_bytes);

        let encoded = serde_json::to_vec(request).map_err(|error| error.to_string())?;
        let write_result = child
            .stdin
            .take()
            .ok_or_else(|| "analysis worker stdin was not available".to_owned())?
            .write_all(&encoded);
        if let Err(error) = write_result {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(format!(
                "analysis worker request could not be written: {error}"
            ));
        }

        let started = Instant::now();
        let (status, timed_out) = loop {
            match child.try_wait() {
                Ok(Some(status)) => break (status, false),
                Ok(None) if started.elapsed() < self.timeout => {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(None) => {
                    child.kill().map_err(|error| {
                        format!("timed-out worker could not be stopped: {error}")
                    })?;
                    let status = child.wait().map_err(|error| error.to_string())?;
                    break (status, true);
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("analysis worker status could not be read: {error}"));
                }
            }
        };

        let (stdout, stdout_exceeded) = join_reader(stdout_reader)?;
        let (stderr, stderr_exceeded) = join_reader(stderr_reader)?;
        if timed_out {
            return Err("analysis worker timed out and was stopped".to_owned());
        }
        if stdout_exceeded || stderr_exceeded {
            return Err("analysis worker output exceeded the configured limit".to_owned());
        }
        if !status.success() {
            let detail = String::from_utf8_lossy(&stderr);
            return Err(format!("analysis worker exited unsuccessfully: {detail}"));
        }
        serde_json::from_slice(&stdout)
            .map_err(|error| format!("analysis worker returned invalid JSON: {error}"))
    }
}

fn read_capped<R>(
    mut reader: R,
    limit: usize,
) -> thread::JoinHandle<Result<(Vec<u8>, bool), String>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut retained = Vec::new();
        let mut exceeded = false;
        let mut buffer = [0_u8; 4096];
        loop {
            let count = reader
                .read(&mut buffer)
                .map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            let remaining = limit.saturating_sub(retained.len());
            retained.extend_from_slice(&buffer[..count.min(remaining)]);
            exceeded |= count > remaining;
        }
        Ok((retained, exceeded))
    })
}

fn join_reader(
    reader: thread::JoinHandle<Result<(Vec<u8>, bool), String>>,
) -> Result<(Vec<u8>, bool), String> {
    reader
        .join()
        .map_err(|_| "analysis worker output reader stopped unexpectedly".to_owned())?
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

    pub fn run_two_stem_separation(
        &self,
        cache_root: &Path,
        identity: &crate::analysis_job::AnalysisIdentity,
        request_id: &str,
        input_audio_path: &Path,
        model_path: &Path,
        expected_model_sha256: &str,
    ) -> Result<SeparationResult, String> {
        if !is_safe_token(request_id) {
            return Err("worker request id must be a nonempty safe token".to_owned());
        }
        if !input_audio_path.is_file() {
            return Err("separation input must be an existing file".to_owned());
        }
        if !model_path.is_file() {
            return Err("separation model must be an existing file".to_owned());
        }
        if !is_sha256(expected_model_sha256) {
            return Err("separation model identity must be a lowercase SHA-256".to_owned());
        }
        let output_directory = identity
            .cache_directory(cache_root)?
            .join("separation")
            .join(request_id);
        let request = WorkerRequest {
            protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
            request_id: request_id.to_owned(),
            operation: WorkerOperation::SeparateTwoStems {
                input_audio_path: input_audio_path.to_owned(),
                model_path: model_path.to_owned(),
                expected_source_sha256: identity.source_sha256.clone(),
                expected_model_sha256: expected_model_sha256.to_owned(),
            },
            output_directory,
        };
        let response = self.backend.execute(&request)?;
        validate_response_identity(&request, &response)?;
        if !response.success {
            return Err(response
                .error
                .filter(|message| !message.trim().is_empty())
                .unwrap_or_else(|| "analysis worker failed without an error message".to_owned()));
        }
        let result = response
            .separation
            .ok_or_else(|| "analysis worker succeeded without separation metadata".to_owned())?;
        if result.sample_rate == 0 || result.channels != 2 || result.frames == 0 {
            return Err("analysis worker returned invalid separation metrics".to_owned());
        }
        validate_artifact(&result.vocals, "vocals.wav")?;
        validate_artifact(&result.accompaniment, "accompaniment.wav")?;
        Ok(result)
    }
}

fn validate_response_identity(
    request: &WorkerRequest,
    response: &WorkerResponse,
) -> Result<(), String> {
    if response.protocol_version != ANALYSIS_WORKER_PROTOCOL_VERSION {
        return Err("analysis worker protocol version mismatch".to_owned());
    }
    if response.request_id != request.request_id {
        return Err("analysis worker response id mismatch".to_owned());
    }
    Ok(())
}

fn validate_artifact(artifact: &SeparationArtifact, expected_name: &str) -> Result<(), String> {
    if artifact.file_name != expected_name || artifact.bytes == 0 || !is_sha256(&artifact.sha256) {
        return Err(format!(
            "analysis worker returned invalid {expected_name} metadata"
        ));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::{
        AnalysisWorkerBackend, AnalysisWorkerService, LocalProcessWorker, SeparationArtifact,
        SeparationResult, WorkerProbeMetrics, WorkerRequest, WorkerResponse,
        ANALYSIS_WORKER_PROTOCOL_VERSION,
    };
    use crate::analysis_job::AnalysisIdentity;
    use std::{
        cell::RefCell,
        fs,
        path::PathBuf,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    struct FakeWorker {
        observed: RefCell<Option<WorkerRequest>>,
        response_version: u32,
        response_request_id: String,
        separation: Option<SeparationResult>,
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
                separation: self.separation.clone(),
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
            separation: None,
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
            separation: None,
        });

        assert!(service
            .run_synthetic_probe(PathBuf::from("cache").as_path(), "../outside")
            .is_err());
        assert!(service
            .run_synthetic_probe(PathBuf::from("cache").as_path(), "probe-1")
            .is_err());
    }

    #[test]
    fn constructs_a_cache_owned_two_stem_separation_request() {
        let directory = test_directory("separation-contract");
        let input = directory.join("input.wav");
        let model = directory.join("model.onnx");
        fs::write(&input, b"read-only input fixture").expect("input fixture should be written");
        fs::write(&model, b"model fixture").expect("model fixture should be written");
        let identity = AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "phase4-v1".to_owned(),
            model_ids: vec!["spleeter-2stems-onnx".to_owned()],
            parameters_sha256: "b".repeat(64),
        };
        let worker = FakeWorker {
            observed: RefCell::new(None),
            response_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
            response_request_id: "separate-1".to_owned(),
            separation: Some(SeparationResult {
                sample_rate: 44_100,
                channels: 2,
                frames: 44_100,
                vocals: SeparationArtifact {
                    file_name: "vocals.wav".to_owned(),
                    bytes: 176_444,
                    sha256: "c".repeat(64),
                },
                accompaniment: SeparationArtifact {
                    file_name: "accompaniment.wav".to_owned(),
                    bytes: 176_444,
                    sha256: "d".repeat(64),
                },
            }),
        };
        let service = AnalysisWorkerService::new(worker);
        let input_before = fs::read(&input).expect("input should be readable");

        let result = service
            .run_two_stem_separation(
                &directory.join("cache"),
                &identity,
                "separate-1",
                &input,
                &model,
                &"e".repeat(64),
            )
            .expect("valid separation response should succeed");

        assert_eq!(result.vocals.file_name, "vocals.wav");
        assert_eq!(
            fs::read(&input).expect("input should remain readable"),
            input_before
        );
        let request = service
            .backend
            .observed
            .borrow()
            .clone()
            .expect("worker should observe the separation request");
        assert!(request
            .output_directory
            .starts_with(directory.join("cache")));
        assert!(!request.output_directory.starts_with(&input));
        let encoded = serde_json::to_value(&request).expect("request should serialize");
        assert_eq!(
            encoded
                .pointer("/operation/separateTwoStems/inputAudioPath")
                .and_then(serde_json::Value::as_str),
            input.to_str()
        );
        assert_eq!(
            encoded
                .pointer("/operation/separateTwoStems/expectedModelSha256")
                .and_then(serde_json::Value::as_str),
            Some("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
        );
        fs::remove_dir_all(directory).expect("test directory should be removable");
    }

    #[cfg(unix)]
    #[test]
    fn launches_the_fake_worker_without_a_shell() {
        let directory = test_directory("worker-process");
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("workers")
            .join("fake_analysis_worker.py");
        let service = AnalysisWorkerService::new(LocalProcessWorker::new(
            PathBuf::from("python3"),
            script,
            Duration::from_secs(5),
            16 * 1024,
        ));

        let metrics = service
            .run_synthetic_probe(&directory, "process-probe")
            .expect("fake worker process should complete");

        assert_eq!(metrics.frames, 16_000);
        assert!(directory
            .join("worker-probes")
            .join("process-probe")
            .join("probe.json")
            .is_file());
        fs::remove_dir_all(directory).expect("test directory should be removable");
    }

    #[cfg(unix)]
    #[test]
    fn terminates_a_worker_that_exceeds_its_deadline() {
        let directory = test_directory("worker-timeout");
        let script = directory.join("hang.py");
        fs::write(&script, "import time\ntime.sleep(10)\n")
            .expect("hanging fixture should be written");
        let service = AnalysisWorkerService::new(LocalProcessWorker::new(
            PathBuf::from("python3"),
            script,
            Duration::from_millis(50),
            1024,
        ));

        let error = service
            .run_synthetic_probe(&directory, "timeout-probe")
            .expect_err("worker should time out");

        assert!(error.contains("timed out"));
        fs::remove_dir_all(directory).expect("test directory should be removable");
    }

    fn test_directory(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("hibiki-{label}-{}-{unique}", std::process::id()));
        fs::create_dir(&path).expect("test directory should be created");
        path
    }
}
