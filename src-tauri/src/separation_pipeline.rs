use std::path::Path;

use crate::{
    spleeter_inference::{separate_source, SpectrogramInference},
    stem_cache::{commit_stems, StemCacheArtifacts},
};

pub fn separate_source_to_cache<I>(
    inference: &mut I,
    source_path: &Path,
    cache_directory: &Path,
) -> Result<StemCacheArtifacts, String>
where
    I: SpectrogramInference,
{
    let stems = separate_source(inference, source_path)?;
    commit_stems(cache_directory, &stems)
        .map_err(|error| format!("stem cache commit failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::separate_source_to_cache;
    use crate::spleeter_inference::{SpectrogramInference, StemEstimates};
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

        let artifacts = separate_source_to_cache(&mut VocalOnlyInference, &source, &cache.0)
            .expect("injected pipeline should commit stems");

        assert_eq!(artifacts.vocals_sha256.len(), 64);
        assert!(cache.0.join("vocals.flac").is_file());
        assert!(cache.0.join("accompaniment.flac").is_file());
        assert!(cache.0.join("stems.json").is_file());
        assert_eq!(
            fs::read(source).expect("source should remain readable"),
            before
        );
    }
}
