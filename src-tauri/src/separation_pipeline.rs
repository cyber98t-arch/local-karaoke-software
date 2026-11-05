use std::path::Path;

use crate::{
    analysis_job::AnalysisIdentity,
    spleeter_inference::{separate_source, SpectrogramInference},
    stem_cache::{commit_stems, StemCacheArtifacts},
};

pub fn separate_source_to_cache<I>(
    inference: &mut I,
    source_path: &Path,
    cache_root: &Path,
    identity: &AnalysisIdentity,
) -> Result<StemCacheArtifacts, String>
where
    I: SpectrogramInference,
{
    let cache_directory = identity.cache_directory(cache_root)?;
    let stems = separate_source(inference, source_path)?;
    commit_stems(&cache_directory, &stems)
        .map_err(|error| format!("stem cache commit failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::separate_source_to_cache;
    use crate::{
        analysis_job::AnalysisIdentity,
        spleeter_inference::{SpectrogramInference, StemEstimates},
    };
    use ndarray::Array4;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct VocalOnlyInference;
    impl SpectrogramInference for VocalOnlyInference {
        fn infer(&mut self, input: &Array4<f32>) -> Result<StemEstimates, String> {
            Ok(StemEstimates {
                vocals: input.clone(),
                accompaniment: Array4::zeros(input.raw_dim()),
            })
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
                "hibiki-separation-pipeline-{}-{timestamp}-{unique}",
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
    fn separates_read_only_source_only_into_injected_cache() {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/metadata.wav");
        let before = fs::read(&source).expect("source should be readable");
        let cache = TestDirectory::new();
        let identity = AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "separation-v1".to_owned(),
            model_ids: vec!["spleeter-2stems-onnx".to_owned()],
            parameters_sha256: "b".repeat(64),
        };
        let output = identity
            .cache_directory(&cache.0)
            .expect("identity should derive a cache path");

        let artifacts =
            separate_source_to_cache(&mut VocalOnlyInference, &source, &cache.0, &identity)
                .expect("injected pipeline should commit stems");

        assert_eq!(artifacts.vocals_sha256.len(), 64);
        assert!(output.join("vocals.flac").is_file());
        assert!(output.join("accompaniment.flac").is_file());
        assert!(output.join("stems.json").is_file());
        assert_eq!(
            fs::read(source).expect("source should remain readable"),
            before
        );
    }

    #[cfg(feature = "native-onnx")]
    #[test]
    #[ignore = "requires HIBIKI_SPLEETER_ONNX_MODEL and performs native separation"]
    fn separates_fixture_with_verified_native_model_into_cache() {
        use crate::{cache::content_sha256, spleeter_inference::NativeOrtSession};

        const MODEL_SHA256: &str =
            "8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f";
        let model_path = PathBuf::from(
            std::env::var_os("HIBIKI_SPLEETER_ONNX_MODEL")
                .expect("HIBIKI_SPLEETER_ONNX_MODEL must name the downloaded ONNX file"),
        );
        assert_eq!(
            content_sha256(&model_path).expect("model should be hashable"),
            MODEL_SHA256
        );
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/metadata.wav");
        let source_sha256 = content_sha256(&source).expect("fixture should be hashable");
        let cache = TestDirectory::new();
        let identity = AnalysisIdentity {
            source_sha256: source_sha256.clone(),
            pipeline_version: "separation-v1".to_owned(),
            model_ids: vec!["spleeter-2stems-onnx-dev-v1".to_owned()],
            parameters_sha256: "9ea689566c7c6a6803323a6131377046dd09d06affc295f412524deed0844eb0"
                .to_owned(),
        };
        let output = identity
            .cache_directory(&cache.0)
            .expect("identity should derive a cache path");
        let mut runtime = NativeOrtSession::from_file(&model_path)
            .expect("verified development model should load");

        let artifacts = separate_source_to_cache(&mut runtime, &source, &cache.0, &identity)
            .expect("native separation should commit both stems");

        assert_eq!(artifacts.vocals_sha256.len(), 64);
        assert!(output.join("vocals.flac").is_file());
        assert!(output.join("accompaniment.flac").is_file());
        assert!(output.join("stems.json").is_file());
        assert_eq!(
            content_sha256(&source).expect("fixture should remain hashable"),
            source_sha256,
            "native separation must not change the source"
        );
    }
}
