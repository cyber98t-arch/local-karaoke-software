use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Eq, PartialEq)]
pub struct AudioSource {
    path: PathBuf,
}

impl AudioSource {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub fn scan_supported_audio(root: &Path) -> io::Result<Vec<AudioSource>> {
    let mut sources = Vec::new();
    scan_directory(root, &mut sources)?;
    sources.sort_unstable_by(|left, right| left.path.cmp(&right.path));
    Ok(sources)
}

fn scan_directory(directory: &Path, sources: &mut Vec<AudioSource>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            scan_directory(&path, sources)?;
        } else if file_type.is_file() && is_supported_audio(&path) {
            sources.push(AudioSource { path });
        }
    }

    Ok(())
}

fn is_supported_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["mp3", "flac", "wav"]
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
}

#[cfg(test)]
mod tests {
    use super::{scan_supported_audio, AudioSource};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibiki-library-scan-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).expect("test directory should be removable");
        }
    }

    fn write_fixture(root: &Path, relative_path: &str, contents: &[u8]) {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent should be created");
        }
        fs::write(path, contents).expect("fixture should be written");
    }

    fn relative_paths(root: &Path, sources: &[AudioSource]) -> Vec<PathBuf> {
        sources
            .iter()
            .map(|source| {
                source
                    .path()
                    .strip_prefix(root)
                    .expect("source should remain below scan root")
                    .to_path_buf()
            })
            .collect()
    }

    #[test]
    fn discovers_supported_audio_recursively_and_ignores_other_files() {
        let directory = TestDirectory::new();
        write_fixture(&directory.path, "alpha.MP3", b"mp3 fixture");
        write_fixture(&directory.path, "nested/beta.flac", b"flac fixture");
        write_fixture(&directory.path, "nested/deeper/gamma.WaV", b"wav fixture");
        write_fixture(&directory.path, "cover.jpg", b"image fixture");
        write_fixture(&directory.path, "notes.txt", b"text fixture");

        let sources = scan_supported_audio(&directory.path).expect("scan should succeed");

        assert_eq!(
            relative_paths(&directory.path, &sources),
            vec![
                PathBuf::from("alpha.MP3"),
                PathBuf::from("nested/beta.flac"),
                PathBuf::from("nested/deeper/gamma.WaV"),
            ]
        );
    }

    #[test]
    fn scanning_does_not_change_source_files() {
        let directory = TestDirectory::new();
        write_fixture(&directory.path, "song.mp3", b"original audio bytes");
        let song = directory.path.join("song.mp3");
        let before = fs::read(&song).expect("fixture should be readable");

        scan_supported_audio(&directory.path).expect("scan should succeed");

        assert_eq!(fs::read(&song).expect("source should still exist"), before);
        assert_eq!(
            fs::read_dir(&directory.path)
                .expect("source directory should remain readable")
                .count(),
            1,
            "scan must not add files beside the source"
        );
    }
}
