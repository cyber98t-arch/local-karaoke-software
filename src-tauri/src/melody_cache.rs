use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::crepe_signal::{PitchPoint, CREPE_HOP, CREPE_SAMPLE_RATE};

const SCHEMA_VERSION: u32 = 1;
const DECODER: &str = "crepe-local-average";
const DECODER_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MelodyManifest {
    schema_version: u32,
    sample_rate: u32,
    hop_samples: u32,
    model_artifact_id: String,
    model_sha256: String,
    decoder: String,
    decoder_version: u32,
    points: Vec<PitchPoint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MelodyModelIdentity {
    pub artifact_id: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MelodyCacheArtifact {
    pub sha256: String,
    pub point_count: usize,
}

pub fn commit_melody(
    cache_directory: &Path,
    model: &MelodyModelIdentity,
    points: &[PitchPoint],
) -> io::Result<MelodyCacheArtifact> {
    validate_model(model)?;
    let final_path = cache_directory.join("melody.json");
    let temporary_path = cache_directory.join("melody.json.tmp");
    if final_path.exists() {
        let artifact = validate_existing(&final_path, model)?;
        remove_if_present(&temporary_path)?;
        return Ok(artifact);
    }
    validate_points(points)?;

    let manifest = MelodyManifest {
        schema_version: SCHEMA_VERSION,
        sample_rate: CREPE_SAMPLE_RATE,
        hop_samples: CREPE_HOP as u32,
        model_artifact_id: model.artifact_id.clone(),
        model_sha256: model.sha256.clone(),
        decoder: DECODER.to_owned(),
        decoder_version: DECODER_VERSION,
        points: points.to_vec(),
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

    Ok(MelodyCacheArtifact {
        sha256: sha256(&encoded),
        point_count: points.len(),
    })
}

fn validate_existing(
    path: &Path,
    expected_model: &MelodyModelIdentity,
) -> io::Result<MelodyCacheArtifact> {
    let bytes = fs::read(path)?;
    let manifest: MelodyManifest = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    if manifest.schema_version != SCHEMA_VERSION
        || manifest.sample_rate != CREPE_SAMPLE_RATE
        || manifest.hop_samples != CREPE_HOP as u32
        || manifest.model_artifact_id != expected_model.artifact_id
        || manifest.model_sha256 != expected_model.sha256
        || manifest.decoder != DECODER
        || manifest.decoder_version != DECODER_VERSION
    {
        return Err(invalid_data(
            "existing melody cache has an unsupported identity or format",
        ));
    }
    validate_points(&manifest.points)?;
    Ok(MelodyCacheArtifact {
        sha256: sha256(&bytes),
        point_count: manifest.points.len(),
    })
}

fn validate_model(model: &MelodyModelIdentity) -> io::Result<()> {
    if model.artifact_id.is_empty()
        || matches!(model.artifact_id.as_str(), "." | "..")
        || !model
            .artifact_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        || model.sha256.len() != 64
        || !model
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid melody model identity",
        ));
    }
    Ok(())
}

fn validate_points(points: &[PitchPoint]) -> io::Result<()> {
    if points.is_empty() {
        return Err(invalid_data("melody contour must contain points"));
    }
    let mut previous_time = None;
    for point in points {
        if !point.time_seconds.is_finite()
            || point.time_seconds < 0.0
            || !point.frequency_hz.is_finite()
            || point.frequency_hz < 0.0
            || !point.confidence.is_finite()
            || !(0.0..=1.0).contains(&point.confidence)
            || previous_time.is_some_and(|previous| point.time_seconds <= previous)
        {
            return Err(invalid_data(
                "melody points must be finite, ordered, and within valid ranges",
            ));
        }
        previous_time = Some(point.time_seconds);
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
    use super::{commit_melody, MelodyModelIdentity};
    use crate::crepe_signal::PitchPoint;
    use serde_json::Value;
    use std::{
        fs, io,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should follow the Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibiki-melody-cache-{}-{timestamp}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("test directory should be removable");
        }
    }

    fn model() -> MelodyModelIdentity {
        MelodyModelIdentity {
            artifact_id: "crepe-tiny-onnx-research-v1".to_owned(),
            sha256: "a".repeat(64),
        }
    }

    fn points() -> Vec<PitchPoint> {
        vec![
            PitchPoint {
                time_seconds: 0.0,
                frequency_hz: 220.0,
                confidence: 0.9,
            },
            PitchPoint {
                time_seconds: 0.01,
                frequency_hz: 221.0,
                confidence: 0.8,
            },
        ]
    }

    #[test]
    fn commits_versioned_contour_and_provenance_inside_cache() {
        let directory = TestDirectory::new();

        let artifact =
            commit_melody(&directory.0, &model(), &points()).expect("valid contour should commit");

        let path = directory.0.join("melody.json");
        let json: Value =
            serde_json::from_slice(&fs::read(path).expect("cache should be readable"))
                .expect("cache should contain JSON");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["sampleRate"], 16_000);
        assert_eq!(json["hopSamples"], 160);
        assert_eq!(json["modelArtifactId"], model().artifact_id);
        assert_eq!(json["modelSha256"], model().sha256);
        assert_eq!(json["decoder"], "crepe-local-average");
        assert_eq!(json["decoderVersion"], 1);
        assert_eq!(
            json["points"]
                .as_array()
                .expect("points should be an array")
                .len(),
            2
        );
        assert_eq!(artifact.point_count, 2);
        assert_eq!(artifact.sha256.len(), 64);
        assert!(!directory.0.join("melody.json.tmp").exists());
    }

    #[test]
    fn preserves_an_existing_valid_contour() {
        let directory = TestDirectory::new();
        let first =
            commit_melody(&directory.0, &model(), &points()).expect("first contour should commit");
        let before = fs::read(directory.0.join("melody.json")).expect("cache should be readable");
        let replacement = vec![PitchPoint {
            time_seconds: 0.0,
            frequency_hz: 440.0,
            confidence: 1.0,
        }];

        let second = commit_melody(&directory.0, &model(), &replacement)
            .expect("valid existing contour should be reused");

        assert_eq!(second, first);
        assert_eq!(fs::read(directory.0.join("melody.json")).unwrap(), before);
    }

    #[test]
    fn rejects_invalid_points_without_committing() {
        let directory = TestDirectory::new();
        let invalid = vec![
            PitchPoint {
                time_seconds: 0.01,
                frequency_hz: 220.0,
                confidence: 0.9,
            },
            PitchPoint {
                time_seconds: 0.0,
                frequency_hz: f32::NAN,
                confidence: 1.1,
            },
        ];

        let error = commit_melody(&directory.0, &model(), &invalid)
            .expect_err("invalid contour must not commit");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 0);
    }
}
