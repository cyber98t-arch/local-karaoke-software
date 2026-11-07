use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::analysis_job::AnalysisIdentity;

pub fn authoritative_lyrics_sha256(lyrics: &str) -> Result<String, String> {
    if lyrics.trim().is_empty() {
        return Err("authoritative lyrics must not be empty".to_owned());
    }
    Ok(format!("{:x}", Sha256::digest(lyrics.as_bytes())))
}

pub fn lyric_revision_directory(
    cache_root: &Path,
    identity: &AnalysisIdentity,
    lyrics: &str,
) -> Result<PathBuf, String> {
    Ok(identity
        .cache_directory(cache_root)?
        .join("lyric-revisions")
        .join(authoritative_lyrics_sha256(lyrics)?))
}

#[cfg(test)]
mod tests {
    use super::lyric_revision_directory;
    use crate::analysis_job::AnalysisIdentity;
    use std::path::Path;

    fn identity() -> AnalysisIdentity {
        AnalysisIdentity {
            source_sha256: "a".repeat(64),
            pipeline_version: "phase4-v1".into(),
            model_ids: vec!["model".into()],
            parameters_sha256: "b".repeat(64),
        }
    }

    #[test]
    fn keys_revision_below_the_audio_identity_by_exact_lyrics() {
        let first = lyric_revision_directory(Path::new("cache"), &identity(), "今日は").unwrap();
        let same = lyric_revision_directory(Path::new("cache"), &identity(), "今日は").unwrap();
        let spaced = lyric_revision_directory(Path::new("cache"), &identity(), "今日は\n").unwrap();

        assert_eq!(first, same);
        assert_ne!(first, spaced);
        assert_eq!(
            first.parent().unwrap().file_name().unwrap(),
            "lyric-revisions"
        );
        assert_eq!(first.file_name().unwrap().to_string_lossy().len(), 64);
    }

    #[test]
    fn rejects_empty_authoritative_lyrics() {
        assert!(lyric_revision_directory(Path::new("cache"), &identity(), "  \n").is_err());
    }
}
