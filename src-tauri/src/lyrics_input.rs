use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

pub const MAX_LYRICS_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLyrics {
    pub text: String,
    pub sha256: String,
}

pub fn validate_pasted_lyrics(text: String) -> Result<ValidatedLyrics, String> {
    validate_text(text)
}

pub fn import_utf8_lyrics(path: &Path) -> Result<ValidatedLyrics, String> {
    if !path.is_file() {
        return Err("lyrics import must be an existing file".to_owned());
    }
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("txt"))
    {
        return Err("lyrics import must use a .txt extension".to_owned());
    }
    let file =
        File::open(path).map_err(|error| format!("lyrics file could not be opened: {error}"))?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if metadata.len() > MAX_LYRICS_BYTES as u64 {
        return Err("lyrics text exceeds the one MiB limit".to_owned());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((MAX_LYRICS_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("lyrics file could not be read: {error}"))?;
    if bytes.len() > MAX_LYRICS_BYTES {
        return Err("lyrics text exceeds the one MiB limit".to_owned());
    }
    let text =
        String::from_utf8(bytes).map_err(|_| "lyrics file must contain UTF-8 text".to_owned())?;
    validate_text(text.strip_prefix('\u{feff}').unwrap_or(&text).to_owned())
}

fn validate_text(text: String) -> Result<ValidatedLyrics, String> {
    if text.len() > MAX_LYRICS_BYTES {
        return Err("lyrics text exceeds the one MiB limit".to_owned());
    }
    if text.trim().is_empty() {
        return Err("lyrics text must not be empty".to_owned());
    }
    if text.contains('\0') {
        return Err("lyrics text must not contain NUL characters".to_owned());
    }
    let sha256 = format!("{:x}", Sha256::digest(text.as_bytes()));
    Ok(ValidatedLyrics { text, sha256 })
}

#[cfg(test)]
mod tests {
    use super::{import_utf8_lyrics, validate_pasted_lyrics, MAX_LYRICS_BYTES};
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
                "hibiki-lyrics-input-{}-{}",
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
    fn accepts_exact_pasted_text_and_hashes_its_utf8_bytes() {
        let lyrics = validate_pasted_lyrics("今日は、晴れ。\n".to_owned()).unwrap();
        assert_eq!(lyrics.text, "今日は、晴れ。\n");
        assert_eq!(lyrics.sha256.len(), 64);
        assert_eq!(
            lyrics.sha256,
            validate_pasted_lyrics(lyrics.text.clone()).unwrap().sha256
        );
    }

    #[test]
    fn imports_utf8_txt_without_changing_source_bytes() {
        let directory = TestDirectory::new();
        let path = directory.0.join("lyrics.txt");
        let bytes = "\u{feff}天気です。\r\n".as_bytes();
        fs::write(&path, bytes).unwrap();
        let before = fs::read(&path).unwrap();
        let lyrics = import_utf8_lyrics(&path).unwrap();
        assert_eq!(lyrics.text, "天気です。\r\n");
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    #[test]
    fn rejects_invalid_or_unsafe_inputs() {
        assert!(validate_pasted_lyrics(" \n".to_owned()).is_err());
        assert!(validate_pasted_lyrics("歌\0詞".to_owned()).is_err());
        assert!(validate_pasted_lyrics("あ".repeat(MAX_LYRICS_BYTES)).is_err());

        let directory = TestDirectory::new();
        let wrong_extension = directory.0.join("lyrics.lrc");
        fs::write(&wrong_extension, "歌詞").unwrap();
        assert!(import_utf8_lyrics(&wrong_extension).is_err());
        let invalid_utf8 = directory.0.join("invalid.txt");
        fs::write(&invalid_utf8, [0xff, 0xfe]).unwrap();
        assert!(import_utf8_lyrics(&invalid_utf8).is_err());
    }
}
