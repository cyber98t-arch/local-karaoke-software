use crate::{
    cache::content_sha256,
    catalog::{Catalog, CatalogSource},
    library::scan_supported_audio,
    metadata::read_track_metadata,
};
use std::{io, path::Path};

pub fn index_library(root: &Path, catalog: &mut Catalog) -> io::Result<Vec<CatalogSource>> {
    let mut indexed = Vec::new();

    for source in scan_supported_audio(root)? {
        let metadata = read_track_metadata(source.path())?;
        indexed.push(CatalogSource {
            source_path: source.path().to_string_lossy().into_owned(),
            content_sha256: content_sha256(source.path())?,
            title: metadata.title,
            artist: metadata.artist,
            duration_ms: metadata.duration_ms,
        });
    }

    catalog
        .replace_sources(&indexed)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
    Ok(indexed)
}

#[cfg(test)]
mod tests {
    use super::index_library;
    use crate::catalog::Catalog;
    use std::{
        collections::BTreeMap,
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
                "hibiki-library-service-{}-{unique}",
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

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn copy_fixture(root: &Path, name: &str) {
        fs::copy(fixture(name), root.join(name)).expect("fixture should be copied");
    }

    fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
        fs::read_dir(root)
            .expect("library should be readable")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| {
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    fs::read(entry.path()).expect("source should be readable"),
                )
            })
            .collect()
    }

    #[test]
    fn indexes_supported_sources_without_changing_library_files() {
        let directory = TestDirectory::new();
        let library_root = directory.path.join("music");
        fs::create_dir(&library_root).expect("library should be created");
        for name in ["metadata.mp3", "metadata.flac", "metadata.wav"] {
            copy_fixture(&library_root, name);
        }
        fs::write(library_root.join("cover.jpg"), b"not audio")
            .expect("ignored fixture should be written");
        let before = snapshot(&library_root);
        let mut catalog =
            Catalog::open(&directory.path.join("catalog.sqlite3")).expect("catalog should open");

        let indexed = index_library(&library_root, &mut catalog).expect("index should succeed");

        assert_eq!(indexed.len(), 3);
        assert_eq!(
            catalog.sources().expect("catalog should be readable"),
            indexed
        );
        assert_eq!(snapshot(&library_root), before);
        assert!(indexed.iter().all(|source| source.duration_ms > 0));
        assert!(indexed
            .iter()
            .all(|source| source.content_sha256.len() == 64));
    }

    #[test]
    fn leaves_catalog_empty_when_any_supported_source_is_invalid() {
        let directory = TestDirectory::new();
        let library_root = directory.path.join("music");
        fs::create_dir(&library_root).expect("library should be created");
        copy_fixture(&library_root, "metadata.flac");
        fs::write(library_root.join("broken.mp3"), b"not an mp3")
            .expect("broken fixture should be written");
        let mut catalog =
            Catalog::open(&directory.path.join("catalog.sqlite3")).expect("catalog should open");

        assert!(index_library(&library_root, &mut catalog).is_err());

        assert!(catalog
            .sources()
            .expect("catalog should remain readable")
            .is_empty());
    }
}
