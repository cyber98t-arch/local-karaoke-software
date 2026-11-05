use crate::lyrics_input::{import_utf8_lyrics, validate_pasted_lyrics, ValidatedLyrics};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

#[derive(Debug, Default)]
pub struct LyricsState {
    lyrics_by_song: Mutex<HashMap<String, ValidatedLyrics>>,
}

impl LyricsState {
    pub fn set_pasted(&self, song_id: &str, text: String) -> Result<LyricsAcceptance, String> {
        let lyrics = validate_pasted_lyrics(text)?;
        self.store(song_id, lyrics)
    }

    pub fn import_selected(
        &self,
        song_id: &str,
        selected: Option<PathBuf>,
    ) -> Result<LyricsSelectionResult, String> {
        validate_song_id(song_id)?;
        let Some(path) = selected else {
            return Ok(LyricsSelectionResult {
                selected: false,
                lyrics: None,
            });
        };
        let lyrics = import_utf8_lyrics(&path)?;
        Ok(LyricsSelectionResult {
            selected: true,
            lyrics: Some(self.store(song_id, lyrics)?),
        })
    }

    pub fn lyrics_for_song(&self, song_id: &str) -> Result<Option<ValidatedLyrics>, String> {
        validate_song_id(song_id)?;
        Ok(self
            .lyrics_by_song
            .lock()
            .map_err(|_| "The lyric state is unavailable.".to_owned())?
            .get(song_id)
            .cloned())
    }

    fn store(&self, song_id: &str, lyrics: ValidatedLyrics) -> Result<LyricsAcceptance, String> {
        validate_song_id(song_id)?;
        let acceptance = LyricsAcceptance {
            sha256: lyrics.sha256.clone(),
            byte_size: lyrics.text.len(),
        };
        self.lyrics_by_song
            .lock()
            .map_err(|_| "The lyric state is unavailable.".to_owned())?
            .insert(song_id.to_owned(), lyrics);
        Ok(acceptance)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsAcceptance {
    pub sha256: String,
    pub byte_size: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsSelectionResult {
    pub selected: bool,
    pub lyrics: Option<LyricsAcceptance>,
}

#[tauri::command]
pub fn set_pasted_lyrics(
    song_id: String,
    text: String,
    state: tauri::State<'_, LyricsState>,
) -> Result<LyricsAcceptance, String> {
    state.set_pasted(&song_id, text)
}

#[tauri::command]
pub async fn select_lyrics_file(
    song_id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, LyricsState>,
) -> Result<LyricsSelectionResult, String> {
    use tauri_plugin_dialog::DialogExt;

    let selected = app
        .dialog()
        .file()
        .set_title("歌詞テキストを選択")
        .add_filter("UTF-8 テキスト", &["txt"])
        .blocking_pick_file()
        .map(|path| {
            path.into_path()
                .map_err(|_| "The selected lyrics location is not a local file.".to_owned())
        })
        .transpose()?;
    state.import_selected(&song_id, selected)
}

fn validate_song_id(song_id: &str) -> Result<(), String> {
    if song_id.len() != 64
        || !song_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("song id must be a lowercase SHA-256".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::LyricsState;
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
                "hibiki-lyrics-command-{}-{}",
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
    fn song_id() -> String {
        "a".repeat(64)
    }

    #[test]
    fn stores_pasted_lyrics_without_returning_the_text() {
        let state = LyricsState::default();
        let accepted = state.set_pasted(&song_id(), "今日は".into()).unwrap();
        let encoded = serde_json::to_string(&accepted).unwrap();
        assert_eq!(accepted.byte_size, "今日は".len());
        assert!(!encoded.contains("今日は"));
        assert_eq!(
            state.lyrics_for_song(&song_id()).unwrap().unwrap().text,
            "今日は"
        );
    }

    #[test]
    fn native_selection_keeps_the_path_out_of_the_response() {
        let directory = TestDirectory::new();
        let path = directory.0.join("private-lyrics.txt");
        fs::write(&path, "歌詞です").unwrap();
        let state = LyricsState::default();
        let result = state
            .import_selected(&song_id(), Some(path.clone()))
            .unwrap();
        let encoded = serde_json::to_string(&result).unwrap();
        assert!(result.selected);
        assert!(!encoded.contains("private-lyrics"));
        assert!(!encoded.contains("歌詞です"));
        assert_eq!(fs::read_to_string(path).unwrap(), "歌詞です");
    }

    #[test]
    fn cancellation_and_invalid_song_ids_do_not_change_state() {
        let state = LyricsState::default();
        let cancelled = state.import_selected(&song_id(), None).unwrap();
        assert!(!cancelled.selected);
        assert!(state.lyrics_for_song(&song_id()).unwrap().is_none());
        assert!(state.set_pasted("../song", "歌詞".into()).is_err());
    }
}
