use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::lyric_alignment::{AlignedLyricCharacter, AlignmentStatus, LyricAlignment};

const SCHEMA_VERSION: u32 = 1;
const ALGORITHM: &str = "japanese-character-edit-distance";
const ALGORITHM_VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AlignedLyricsManifest {
    schema_version: u32,
    algorithm: String,
    algorithm_version: u32,
    authoritative_lyrics_sha256: String,
    original_text: String,
    characters: Vec<AlignedLyricCharacter>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlignedLyricsArtifact {
    pub sha256: String,
    pub character_count: usize,
    pub matched_count: usize,
}

pub fn commit_aligned_lyrics(
    cache_directory: &Path,
    alignment: &LyricAlignment,
) -> io::Result<AlignedLyricsArtifact> {
    validate_alignment(alignment)?;
    let lyrics_sha256 = sha256(alignment.original_text.as_bytes());
    let final_path = cache_directory.join("aligned-lyrics.json");
    let temporary_path = cache_directory.join("aligned-lyrics.json.tmp");
    if final_path.exists() {
        let artifact = validate_existing(&final_path, &lyrics_sha256)?;
        remove_if_present(&temporary_path)?;
        return Ok(artifact);
    }
    let manifest = AlignedLyricsManifest {
        schema_version: SCHEMA_VERSION,
        algorithm: ALGORITHM.to_owned(),
        algorithm_version: ALGORITHM_VERSION,
        authoritative_lyrics_sha256: lyrics_sha256,
        original_text: alignment.original_text.clone(),
        characters: alignment.characters.clone(),
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
    Ok(artifact(&encoded, &manifest.characters))
}

fn validate_existing(
    path: &Path,
    expected_lyrics_sha256: &str,
) -> io::Result<AlignedLyricsArtifact> {
    let bytes = fs::read(path)?;
    let manifest: AlignedLyricsManifest = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    if manifest.schema_version != SCHEMA_VERSION
        || manifest.algorithm != ALGORITHM
        || manifest.algorithm_version != ALGORITHM_VERSION
        || manifest.authoritative_lyrics_sha256 != expected_lyrics_sha256
        || sha256(manifest.original_text.as_bytes()) != manifest.authoritative_lyrics_sha256
    {
        return Err(invalid_data(
            "existing aligned lyrics have an unsupported identity or format",
        ));
    }
    validate_alignment(&LyricAlignment {
        original_text: manifest.original_text,
        characters: manifest.characters.clone(),
    })?;
    Ok(artifact(&bytes, &manifest.characters))
}

fn validate_alignment(alignment: &LyricAlignment) -> io::Result<()> {
    if alignment.original_text.is_empty() || alignment.characters.is_empty() {
        return Err(invalid_data(
            "aligned lyrics must contain original text and characters",
        ));
    }
    let mut expected_start = 0;
    let mut previous_timed_end = 0;
    for character in &alignment.characters {
        let valid_bounds = character.byte_start == expected_start
            && character.byte_end > character.byte_start
            && alignment
                .original_text
                .is_char_boundary(character.byte_start)
            && alignment.original_text.is_char_boundary(character.byte_end)
            && alignment
                .original_text
                .get(character.byte_start..character.byte_end)
                == Some(character.text.as_str());
        let valid_timing = match character.status {
            AlignmentStatus::Matched => {
                matches!((character.start_ms, character.end_ms), (Some(start), Some(end)) if start < end && start >= previous_timed_end)
            }
            AlignmentStatus::Unmatched | AlignmentStatus::Ignored => {
                character.start_ms.is_none() && character.end_ms.is_none()
            }
        };
        if !valid_bounds || !valid_timing {
            return Err(invalid_data("aligned lyric characters must reproduce the original text with valid ordered timing"));
        }
        if let Some(end) = character.end_ms {
            previous_timed_end = end;
        }
        expected_start = character.byte_end;
    }
    if expected_start != alignment.original_text.len() {
        return Err(invalid_data(
            "aligned lyric characters must cover the complete original text",
        ));
    }
    Ok(())
}

fn artifact(bytes: &[u8], characters: &[AlignedLyricCharacter]) -> AlignedLyricsArtifact {
    AlignedLyricsArtifact {
        sha256: sha256(bytes),
        character_count: characters.len(),
        matched_count: characters
            .iter()
            .filter(|item| item.status == AlignmentStatus::Matched)
            .count(),
    }
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
    use super::commit_aligned_lyrics;
    use crate::{analysis_worker::TranscriptionSegment, lyric_alignment::align_japanese_lyrics};
    use serde_json::Value;
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
                "hibiki-aligned-lyrics-{}-{}",
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
    fn alignment(text: &str) -> crate::lyric_alignment::LyricAlignment {
        align_japanese_lyrics(
            text,
            &[TranscriptionSegment {
                start_ms: 0,
                end_ms: 500,
                text: "今日は".into(),
            }],
        )
        .unwrap()
    }

    #[test]
    fn commits_versioned_alignment_and_exact_lyrics_identity() {
        let directory = TestDirectory::new();
        let artifact = commit_aligned_lyrics(&directory.0, &alignment("今日は。")).unwrap();
        let json: Value =
            serde_json::from_slice(&fs::read(directory.0.join("aligned-lyrics.json")).unwrap())
                .unwrap();
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["algorithm"], "japanese-character-edit-distance");
        assert_eq!(json["algorithmVersion"], 1);
        assert_eq!(json["originalText"], "今日は。");
        assert_eq!(artifact.matched_count, 3);
        assert_eq!(artifact.character_count, 4);
        assert!(!directory.0.join("aligned-lyrics.json.tmp").exists());
    }

    #[test]
    fn preserves_an_existing_compatible_alignment() {
        let directory = TestDirectory::new();
        let first = commit_aligned_lyrics(&directory.0, &alignment("今日は。")).unwrap();
        let second = commit_aligned_lyrics(&directory.0, &alignment("今日は。")).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn rejects_invalid_character_boundaries_without_committing() {
        let directory = TestDirectory::new();
        let mut invalid = alignment("今日は。");
        invalid.characters[0].byte_end = 1;
        assert!(commit_aligned_lyrics(&directory.0, &invalid).is_err());
        assert!(!directory.0.join("aligned-lyrics.json").exists());
    }
}
