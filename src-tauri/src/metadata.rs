use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
    tag::Accessor,
};
use std::{io, path::Path};

#[derive(Debug, Eq, PartialEq)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration_ms: u64,
}

pub fn read_track_metadata(source_path: &Path) -> io::Result<TrackMetadata> {
    let tagged_file = Probe::open(source_path)
        .and_then(Probe::read)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    let duration_ms = tagged_file.properties().duration().as_millis();

    Ok(TrackMetadata {
        title: tag.and_then(|tag| tag.title().map(|value| value.into_owned())),
        artist: tag.and_then(|tag| tag.artist().map(|value| value.into_owned())),
        duration_ms: duration_ms.try_into().unwrap_or(u64::MAX),
    })
}

#[cfg(test)]
mod tests {
    use super::read_track_metadata;
    use std::{fs, path::PathBuf};

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn reads_metadata_from_supported_audio_formats() {
        for (name, expected_title) in [
            ("metadata.mp3", "Hibiki MP3 Fixture"),
            ("metadata.flac", "Hibiki FLAC Fixture"),
            ("metadata.wav", "Hibiki WAV Fixture"),
        ] {
            let metadata = read_track_metadata(&fixture(name)).expect("metadata should be read");

            assert_eq!(metadata.title.as_deref(), Some(expected_title));
            assert_eq!(metadata.artist.as_deref(), Some("Hibiki Test Artist"));
            assert!(metadata.duration_ms > 0, "{name} should report a duration");
        }
    }

    #[test]
    fn reading_metadata_does_not_change_source_bytes() {
        for name in ["metadata.mp3", "metadata.flac", "metadata.wav"] {
            let path = fixture(name);
            let before = fs::read(&path).expect("fixture should be readable");

            read_track_metadata(&path).expect("metadata should be read");

            assert_eq!(
                fs::read(path).expect("fixture should remain readable"),
                before,
                "{name} must remain byte-for-byte unchanged"
            );
        }
    }
}
