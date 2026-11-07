use serde::Deserialize;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    analysis_worker::{
        AnalysisWorkerBackend, TranscriptionResult, TranscriptionSegment, WorkerOperation,
        WorkerRequest, WorkerResponse, ANALYSIS_WORKER_PROTOCOL_VERSION,
    },
    cache::content_sha256,
};

pub struct WhisperCppBackend {
    cli_path: PathBuf,
    expected_cli_sha256: String,
    expected_cli_bytes: u64,
    timeout: Duration,
}

impl WhisperCppBackend {
    pub fn new(
        cli_path: PathBuf,
        expected_cli_sha256: String,
        expected_cli_bytes: u64,
        timeout: Duration,
    ) -> Self {
        Self {
            cli_path,
            expected_cli_sha256,
            expected_cli_bytes,
            timeout,
        }
    }

    fn verify_cli(&self) -> Result<(), String> {
        let metadata = fs::metadata(&self.cli_path)
            .map_err(|_| "whisper.cpp CLI is not available".to_owned())?;
        if !metadata.is_file() || metadata.len() != self.expected_cli_bytes {
            return Err("whisper.cpp CLI size does not match the pinned artifact".to_owned());
        }
        if content_sha256(&self.cli_path).map_err(|e| e.to_string())? != self.expected_cli_sha256 {
            return Err("whisper.cpp CLI SHA-256 does not match the pinned artifact".to_owned());
        }
        Ok(())
    }
}

impl AnalysisWorkerBackend for WhisperCppBackend {
    fn execute(&self, request: &WorkerRequest) -> Result<WorkerResponse, String> {
        self.verify_cli()?;
        let WorkerOperation::TranscribeJapanese {
            input_audio_path,
            model_path,
            expected_source_sha256,
            expected_model_sha256,
            language,
        } = &request.operation
        else {
            return Err("whisper.cpp backend supports only Japanese transcription".to_owned());
        };
        if language != "ja" || self.timeout.is_zero() {
            return Err("invalid whisper.cpp execution settings".to_owned());
        }
        verify_file(
            input_audio_path,
            expected_source_sha256,
            "transcription input",
        )?;
        verify_file(model_path, expected_model_sha256, "Whisper model")?;
        fs::create_dir_all(&request.output_directory).map_err(|e| e.to_string())?;
        let cli_input = cli_path(input_audio_path, "transcription input")?;
        let cli_model = cli_path(model_path, "Whisper model")?;
        let cli_output_directory = cli_path(&request.output_directory, "transcription output")?;
        let prefix = cli_output_directory.join("whisper-result");
        let stderr_path = request.output_directory.join("whisper-stderr.txt");
        let stderr = File::create(&stderr_path).map_err(|e| e.to_string())?;
        let mut child = Command::new(&self.cli_path)
            .args([
                "-m",
                &cli_model.to_string_lossy(),
                "-f",
                &cli_input.to_string_lossy(),
            ])
            .args(["-l", "ja", "-oj", "-of"])
            .arg(&prefix)
            .arg("--no-prints")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|e| format!("whisper.cpp could not start: {e}"))?;
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err("whisper.cpp timed out".to_owned());
            }
            thread::sleep(Duration::from_millis(20));
        };
        if !status.success() {
            let detail = fs::read_to_string(stderr_path).unwrap_or_default();
            return Err(format!("whisper.cpp failed: {}", failure_detail(&detail)));
        }
        let transcription = parse_whisper_json(&prefix.with_extension("json"))?;
        Ok(WorkerResponse {
            protocol_version: ANALYSIS_WORKER_PROTOCOL_VERSION,
            request_id: request.request_id.clone(),
            success: true,
            metrics: None,
            separation: None,
            transcription: Some(transcription),
            error: None,
        })
    }
}

fn cli_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| format!("{label} path could not be normalized: {error}"))
}

fn verify_file(path: &Path, expected: &str, label: &str) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("{label} is not available"));
    }
    if content_sha256(path).map_err(|e| e.to_string())? != expected {
        return Err(format!("{label} SHA-256 mismatch"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct WhisperDocument {
    transcription: Vec<WhisperEntry>,
}
#[derive(Deserialize)]
struct WhisperEntry {
    offsets: WhisperOffsets,
    text: String,
}
#[derive(Deserialize)]
struct WhisperOffsets {
    from: u64,
    to: u64,
}

fn parse_whisper_json(path: &Path) -> Result<TranscriptionResult, String> {
    let document: WhisperDocument =
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("whisper.cpp JSON could not be decoded: {e}"))?;
    let segments = document
        .transcription
        .into_iter()
        .map(|entry| TranscriptionSegment {
            start_ms: entry.offsets.from,
            end_ms: entry.offsets.to,
            text: entry.text.trim().to_owned(),
        })
        .collect::<Vec<_>>();
    let duration_ms = segments.iter().map(|s| s.end_ms).max().unwrap_or(0);
    if segments.is_empty() {
        return Err("whisper.cpp JSON contains no segments".to_owned());
    }
    Ok(TranscriptionResult {
        language: "ja".into(),
        duration_ms,
        segments,
    })
}

fn failure_detail(value: &str) -> String {
    let errors = value
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("error:"))
        .take(4)
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return errors.join(" ");
    }
    tail(value, 2000)
}

fn tail(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .rev()
        .take(max_chars)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{failure_detail, parse_whisper_json, WhisperCppBackend};
    use crate::analysis_worker::{AnalysisWorkerBackend, WorkerOperation, WorkerRequest};
    use std::fs;
    use std::time::Duration;

    #[test]
    fn parses_pinned_whisper_json_shape_into_millisecond_segments() {
        let path =
            std::env::temp_dir().join(format!("hibiki-whisper-json-{}.json", std::process::id()));
        fs::write(
            &path,
            r#"{"transcription":[{"offsets":{"from":120,"to":980},"text":" 今日は "}]}"#.as_bytes(),
        )
        .unwrap();
        let result = parse_whisper_json(&path).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(result.language, "ja");
        assert_eq!(result.duration_ms, 980);
        assert_eq!(result.segments[0].text, "今日は");
    }

    #[test]
    fn rejects_an_unverified_cli_before_starting_a_process() {
        let directory =
            std::env::temp_dir().join(format!("hibiki-whisper-backend-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir(&directory).unwrap();
        let cli = directory.join("whisper-cli.exe");
        fs::write(&cli, b"not the pinned executable").unwrap();
        let backend =
            WhisperCppBackend::new(cli, "a".repeat(64), 2_115_584, Duration::from_secs(1));
        let request = WorkerRequest {
            protocol_version: 2,
            request_id: "test".into(),
            operation: WorkerOperation::TranscribeJapanese {
                input_audio_path: directory.join("input.wav"),
                model_path: directory.join("model.bin"),
                expected_source_sha256: "b".repeat(64),
                expected_model_sha256: "c".repeat(64),
                language: "ja".into(),
            },
            output_directory: directory.join("output"),
        };

        assert_eq!(
            backend.execute(&request).unwrap_err(),
            "whisper.cpp CLI size does not match the pinned artifact"
        );
        assert!(!request.output_directory.exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reports_actionable_cli_errors_instead_of_the_usage_tail() {
        let stderr = "error: input file not found 'long.wav'\nerror: no input files specified\nusage: whisper-cli [options]\nvery long help";
        assert_eq!(
            failure_detail(stderr),
            "error: input file not found 'long.wav' error: no input files specified"
        );
    }
}
