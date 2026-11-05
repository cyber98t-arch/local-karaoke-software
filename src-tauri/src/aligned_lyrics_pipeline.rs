use std::path::Path;

use crate::{
    aligned_lyrics_cache::{commit_aligned_lyrics, AlignedLyricsArtifact},
    analysis_job::AnalysisIdentity,
    analysis_worker::TranscriptionSegment,
    lyric_alignment::align_japanese_lyrics,
};

pub fn align_lyrics_to_cache(
    authoritative_lyrics: &str,
    transcript: &[TranscriptionSegment],
    cache_root: &Path,
    identity: &AnalysisIdentity,
) -> Result<AlignedLyricsArtifact, String> {
    let alignment = align_japanese_lyrics(authoritative_lyrics, transcript)?;
    let directory = identity.cache_directory(cache_root)?;
    commit_aligned_lyrics(&directory, &alignment)
        .map_err(|error| format!("aligned lyrics cache commit failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::align_lyrics_to_cache;
    use crate::{analysis_job::AnalysisIdentity, analysis_worker::TranscriptionSegment};
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
                "hibiki-aligned-pipeline-{}-{}",
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

    #[test]
    fn aligns_and_commits_only_below_identity_cache() {
        let cache = TestDirectory::new();
        let identity = AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "alignment-v1".into(),
            model_ids: vec!["whisper-base".into()],
            parameters_sha256: "b".repeat(64),
        };
        let transcript = [TranscriptionSegment {
            start_ms: 0,
            end_ms: 500,
            text: "今日は".into(),
        }];
        let output = identity.cache_directory(&cache.0).unwrap();
        let artifact = align_lyrics_to_cache("今日は。", &transcript, &cache.0, &identity).unwrap();
        assert_eq!(artifact.matched_count, 3);
        assert!(output.join("aligned-lyrics.json").is_file());
        assert_eq!(fs::read_dir(&cache.0).unwrap().count(), 1);
    }
}
