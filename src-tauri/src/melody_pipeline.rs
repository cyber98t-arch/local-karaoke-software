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

    #[cfg(feature = "native-onnx")]
    #[test]
    #[ignore = "requires HIBIKI_CREPE_ONNX_MODEL and performs native melody inference"]
    fn extracts_generated_vocals_with_verified_native_model() {
        use crate::{
            cache::content_sha256,
            crepe_inference::NativeCrepeSession,
            onnx_runtime::{ProviderRequest, ProviderRequest::*},
        };
        use std::time::Instant;

        const MODEL_SHA256: &str =
            "03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4";
        let model_path = PathBuf::from(
            std::env::var_os("HIBIKI_CREPE_ONNX_MODEL")
                .expect("HIBIKI_CREPE_ONNX_MODEL must name the converted ONNX file"),
        );
        assert_eq!(
            content_sha256(&model_path).expect("model should be hashable"),
            MODEL_SHA256
        );
        let cache = TestDirectory::new();
        let vocals = (0..88_200)
            .map(|index| {
                let phase = 2.0 * std::f32::consts::PI * 220.0 * index as f32 / 44_100.0;
                [0.2 * phase.sin(), 0.2 * phase.sin()]
            })
            .collect::<Vec<_>>();
        let identity = AnalysisIdentity {
            source_sha256: "d".repeat(64),
            pipeline_version: "melody-v1".to_owned(),
            model_ids: vec!["crepe-tiny-onnx-research-v1".to_owned()],
            parameters_sha256: "e".repeat(64),
        };
        let model = MelodyModelIdentity {
            artifact_id: "crepe-tiny-onnx-research-v1".to_owned(),
            sha256: MODEL_SHA256.to_owned(),
        };
        let output = identity.cache_directory(&cache.0).unwrap();
        let provider =
            ProviderRequest::parse(std::env::var("HIBIKI_ONNX_PROVIDER").ok().as_deref())
                .expect("provider request should be valid");
        let mut runtime = NativeCrepeSession::from_file_with_provider(&model_path, provider)
            .expect("verified research model should load");
        if provider == DirectMl {
            assert_eq!(runtime.provider_selection().active, DirectMl);
        }
        let selected_provider = runtime.provider_selection().clone();
        let started = Instant::now();

        let artifact =
            extract_melody_to_cache(&mut runtime, &vocals, 44_100, &cache.0, &identity, &model)
                .expect("native melody pipeline should commit");
        let elapsed = started.elapsed();

        assert_eq!(artifact.point_count, 201);
        assert!(output.join("melody.json").is_file());
        eprintln!("native melody provider: {:?}", selected_provider.active);
        eprintln!("native melody elapsed: {elapsed:.2?}");
        eprintln!("melody SHA-256: {}", artifact.sha256);
    }

    #[cfg(feature = "native-onnx")]
    #[test]
    #[ignore = "requires explicit model, vocal stem, and cache paths; performs native melody inference"]
    fn extracts_explicit_vocal_source_with_verified_native_model() {
        use crate::{
            analysis_audio::decode_for_analysis, cache::content_sha256,
            crepe_inference::NativeCrepeSession,
        };
        use std::time::Instant;

        const MODEL_SHA256: &str =
            "03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4";
        let model_path = required_path("HIBIKI_CREPE_ONNX_MODEL");
        let vocal_source = required_path("HIBIKI_MELODY_SOURCE");
        let cache_root = required_path("HIBIKI_MELODY_CACHE");
        assert!(
            vocal_source.is_file(),
            "melody source must be an existing audio file"
        );
        assert!(
            vocal_source
                .parent()
                .is_none_or(|source_directory| !cache_root.starts_with(source_directory)),
            "melody cache root must not be inside the source directory"
        );
        assert_eq!(
            content_sha256(&model_path).expect("model should be hashable"),
            MODEL_SHA256
        );
        let source_sha256 = content_sha256(&vocal_source).expect("source should be hashable");
        let decoded = decode_for_analysis(&vocal_source).expect("vocal source should decode");
        let identity = AnalysisIdentity {
            source_sha256: source_sha256.clone(),
            pipeline_version: "melody-v1".to_owned(),
            model_ids: vec!["crepe-tiny-onnx-dev-v1".to_owned()],
            parameters_sha256: "ae67e54203e71441a22dbce7869f583259f6c7f077e2f3941c12f04cb29f2412"
                .to_owned(),
        };
        let model = MelodyModelIdentity {
            artifact_id: "crepe-tiny-onnx-dev-v1".to_owned(),
            sha256: MODEL_SHA256.to_owned(),
        };
        let output = identity.cache_directory(&cache_root).unwrap();
        let mut runtime = NativeCrepeSession::from_file(&model_path)
            .expect("verified development model should load");
        let started = Instant::now();

        let artifact = extract_melody_to_cache(
            &mut runtime,
            &decoded.stereo_frames,
            decoded.sample_rate,
            &cache_root,
            &identity,
            &model,
        )
        .expect("native melody pipeline should commit representative contour");

        let elapsed = started.elapsed();
        assert!(artifact.point_count > 0);
        assert!(output.join("melody.json").is_file());
        assert_eq!(
            content_sha256(&vocal_source).expect("source should remain hashable"),
            source_sha256,
            "native melody inference must not change its input"
        );
        eprintln!("native melody elapsed: {elapsed:.2?}");
        eprintln!("cache directory: {}", output.display());
        eprintln!("melody SHA-256: {}", artifact.sha256);
        eprintln!("melody point count: {}", artifact.point_count);
    }

    #[cfg(feature = "native-onnx")]
    fn required_path(name: &str) -> PathBuf {
        PathBuf::from(
            std::env::var_os(name)
                .unwrap_or_else(|| panic!("{name} must be set to an explicit local path")),
        )
    }
}
