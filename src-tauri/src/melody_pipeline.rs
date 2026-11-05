use std::path::Path;

use crate::{
    analysis_job::AnalysisIdentity,
    crepe_inference::{infer_pitch, PitchInference},
    crepe_signal::{centered_normalized_frames, mono_for_crepe},
    melody_cache::{commit_melody, MelodyCacheArtifact, MelodyModelIdentity},
};

pub fn extract_melody_to_cache<I>(
    inference: &mut I,
    vocals: &[[f32; 2]],
    sample_rate: u32,
    cache_root: &Path,
    identity: &AnalysisIdentity,
    model: &MelodyModelIdentity,
) -> Result<MelodyCacheArtifact, String>
where
    I: PitchInference,
{
    let cache_directory = identity.cache_directory(cache_root)?;
    let mono = mono_for_crepe(vocals, sample_rate)?;
    let frames = centered_normalized_frames(&mono)?;
    let points = infer_pitch(inference, &frames)?;
    commit_melody(&cache_directory, model, &points)
        .map_err(|error| format!("melody cache commit failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::extract_melody_to_cache;
    use crate::{
        analysis_job::AnalysisIdentity, crepe_inference::PitchInference, crepe_signal::CREPE_BINS,
        melody_cache::MelodyModelIdentity,
    };
    use ndarray::Array2;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct KnownPitch;
    impl PitchInference for KnownPitch {
        fn infer(&mut self, frames: &Array2<f32>) -> Result<Array2<f32>, String> {
            let mut output = Array2::zeros((frames.nrows(), CREPE_BINS));
            output.column_mut(180).fill(0.9);
            Ok(output)
        }
    }

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should follow Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibiki-melody-pipeline-{}-{timestamp}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test cache should be created");
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("test cache should be removable");
        }
    }

    #[test]
    fn commits_inferred_contour_only_below_identity_derived_cache() {
        let cache = TestDirectory::new();
        let vocals = (0..44_100)
            .map(|index| {
                let phase = 2.0 * std::f32::consts::PI * 220.0 * index as f32 / 44_100.0;
                [0.2 * phase.sin(), 0.2 * phase.sin()]
            })
            .collect::<Vec<_>>();
        let identity = AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "melody-v1".to_owned(),
            model_ids: vec!["crepe-tiny-onnx-research-v1".to_owned()],
            parameters_sha256: "b".repeat(64),
        };
        let model = MelodyModelIdentity {
            artifact_id: "crepe-tiny-onnx-research-v1".to_owned(),
            sha256: "c".repeat(64),
        };
        let output = identity.cache_directory(&cache.0).unwrap();

        let artifact = extract_melody_to_cache(
            &mut KnownPitch,
            &vocals,
            44_100,
            &cache.0,
            &identity,
            &model,
        )
        .expect("injected melody pipeline should commit");

        assert_eq!(artifact.point_count, 101);
        assert!(output.join("melody.json").is_file());
        assert_eq!(
            fs::read_dir(&cache.0).unwrap().count(),
            1,
            "cache root should contain only the source identity directory"
        );
    }
}
