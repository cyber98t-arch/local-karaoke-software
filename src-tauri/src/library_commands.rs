use crate::{catalog::Catalog, library_service::index_library};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug)]
pub struct LibraryState {
    catalog_path: PathBuf,
    selected_root: Mutex<Option<PathBuf>>,
}

impl LibraryState {
    pub fn new(catalog_path: PathBuf) -> Self {
        Self {
            catalog_path,
            selected_root: Mutex::new(None),
        }
    }

    pub fn select_library(&self, root: &Path) -> Result<(), String> {
        let metadata = fs::metadata(root)
            .map_err(|_| "The selected library folder could not be opened.".to_owned())?;
        if !metadata.is_dir() {
            return Err("The selected library path is not a folder.".to_owned());
        }

        let root = root
            .canonicalize()
            .map_err(|_| "The selected library folder could not be opened.".to_owned())?;
        *self
            .selected_root
            .lock()
            .map_err(|_| "The library state is unavailable.".to_owned())? = Some(root);
        Ok(())
    }

    pub fn index_selected_library(&self) -> Result<LibraryIndexResult, String> {
        let root = self
            .selected_root
            .lock()
            .map_err(|_| "The library state is unavailable.".to_owned())?
            .clone()
            .ok_or_else(|| "No library folder has been selected.".to_owned())?;
        let mut catalog = Catalog::open(&self.catalog_path)
            .map_err(|_| "The local library catalog could not be opened.".to_owned())?;
        let indexed = index_library(&root, &mut catalog)
            .map_err(|_| "The selected library could not be indexed.".to_owned())?;
        let songs = indexed
            .into_iter()
            .map(|source| IndexedSong {
                id: source.content_sha256,
                title: source.title,
                artist: source.artist,
                duration_ms: source.duration_ms,
            })
            .collect::<Vec<_>>();

        Ok(LibraryIndexResult {
            song_count: songs.len(),
            songs,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryIndexResult {
    pub song_count: usize,
    pub songs: Vec<IndexedSong>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedSong {
    pub id: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration_ms: u64,
}

#[tauri::command]
pub fn index_selected_library(
    state: tauri::State<'_, LibraryState>,
) -> Result<LibraryIndexResult, String> {
    state.index_selected_library()
}

#[cfg(test)]
mod tests {
    use super::LibraryState;
    use std::{
        fs,
        path::PathBuf,
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
                "hibiki-library-commands-{}-{unique}",
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

    #[test]
    fn requires_a_rust_selected_library_before_indexing() {
        let directory = TestDirectory::new();
        let state = LibraryState::new(directory.path.join("catalog.sqlite3"));

        let error = state
            .index_selected_library()
            .expect_err("indexing without a selected root should fail");

        assert_eq!(error, "No library folder has been selected.");
        assert!(!directory.path.join("catalog.sqlite3").exists());
    }

    #[test]
    fn rejects_a_file_as_a_library_root() {
        let directory = TestDirectory::new();
        let file = directory.path.join("not-a-folder");
        fs::write(&file, b"fixture").expect("fixture should be written");
        let state = LibraryState::new(directory.path.join("catalog.sqlite3"));

        let error = state
            .select_library(&file)
            .expect_err("a file should not be selectable as a library");

        assert_eq!(error, "The selected library path is not a folder.");
    }

    #[test]
    fn indexes_selected_library_without_returning_source_paths() {
        let directory = TestDirectory::new();
        let library = directory.path.join("music");
        fs::create_dir(&library).expect("library should be created");
        fs::copy(fixture("metadata.flac"), library.join("song.flac"))
            .expect("fixture should be copied");
        let state = LibraryState::new(directory.path.join("catalog.sqlite3"));

        state
            .select_library(&library)
            .expect("library selection should succeed");
        let result = state
            .index_selected_library()
            .expect("selected library should be indexed");

        assert_eq!(result.song_count, 1);
        assert_eq!(result.songs.len(), 1);
        assert_eq!(result.songs[0].id.len(), 64);
        assert_eq!(
            result.songs[0].title.as_deref(),
            Some("Hibiki FLAC Fixture")
        );
        assert_eq!(
            result.songs[0].artist.as_deref(),
            Some("Hibiki Test Artist")
        );
        assert!(result.songs[0].duration_ms > 0);
        assert!(!format!("{result:?}").contains(&library.to_string_lossy().as_ref()));
    }
}
