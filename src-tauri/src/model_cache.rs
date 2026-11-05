use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::model_manifest::ModelArtifactManifest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedModel {
    pub path: PathBuf,
    pub byte_size: u64,
    pub sha256: String,
}

pub fn commit_verified_model(
    staged_file: &Path,
    model_root: &Path,
    manifest: &ModelArtifactManifest,
) -> io::Result<CachedModel> {
    manifest
        .validate()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if !staged_file.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "staged model must be an existing file",
        ));
    }
    let directory = model_root.join(&manifest.id).join(&manifest.version);
    let final_path = directory.join("model.bin");
    let temporary_path = directory.join("model.bin.tmp");
    if final_path.exists() {
        return verify_file(&final_path, manifest);
    }
    let staged = verify_file(staged_file, manifest)?;
    fs::create_dir_all(&directory)?;
    remove_if_present(&temporary_path)?;
    let mut source = File::open(staged_file)?;
    let mut temporary = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary_path)?;
    io::copy(&mut source, &mut temporary)?;
    temporary.flush()?;
    temporary.sync_all()?;
    drop(temporary);
    verify_file(&temporary_path, manifest)?;
    fs::rename(&temporary_path, &final_path)?;
    Ok(CachedModel {
        path: final_path,
        byte_size: staged.byte_size,
        sha256: staged.sha256,
    })
}

fn verify_file(path: &Path, manifest: &ModelArtifactManifest) -> io::Result<CachedModel> {
    let metadata = fs::metadata(path)?;
    if metadata.len() != manifest.byte_size {
        return Err(invalid_data("model byte size does not match its manifest"));
    }
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let sha256 = format!("{:x}", digest.finalize());
    if sha256 != manifest.sha256 {
        return Err(invalid_data("model SHA-256 does not match its manifest"));
    }
    Ok(CachedModel {
        path: path.to_owned(),
        byte_size: metadata.len(),
        sha256,
    })
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::commit_verified_model;
    use crate::model_manifest::{ModelArtifactManifest, ModelFamily};
    use sha2::{Digest, Sha256};
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
                "hibiki-model-cache-{}-{}",
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
    fn manifest(bytes: &[u8]) -> ModelArtifactManifest {
        ModelArtifactManifest {
            schema_version: 1,
            id: "whisper-base-multilingual".into(),
            family: ModelFamily::Whisper,
            version: "test-revision".into(),
            source_url: "https://example.invalid/model.bin".into(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
            byte_size: bytes.len() as u64,
            license_spdx: "MIT".into(),
            license_url: "https://example.invalid/LICENSE".into(),
        }
    }

    #[test]
    fn verifies_then_atomically_commits_below_model_root() {
        let files = TestDirectory::new();
        let models = TestDirectory::new();
        let bytes = b"small generated model fixture";
        let staged = files.0.join("download.part");
        fs::write(&staged, bytes).unwrap();
        let result = commit_verified_model(&staged, &models.0, &manifest(bytes)).unwrap();
        assert_eq!(fs::read(&result.path).unwrap(), bytes);
        assert!(result.path.starts_with(&models.0));
        assert!(!result.path.with_file_name("model.bin.tmp").exists());
    }

    #[test]
    fn rejects_invalid_content_without_committing() {
        let files = TestDirectory::new();
        let models = TestDirectory::new();
        let staged = files.0.join("download.part");
        fs::write(&staged, b"wrong").unwrap();
        assert!(commit_verified_model(&staged, &models.0, &manifest(b"right")).is_err());
        assert_eq!(fs::read_dir(&models.0).unwrap().count(), 0);
    }

    #[test]
    fn preserves_an_existing_verified_model() {
        let files = TestDirectory::new();
        let models = TestDirectory::new();
        let bytes = b"fixture";
        let staged = files.0.join("first.part");
        fs::write(&staged, bytes).unwrap();
        let first = commit_verified_model(&staged, &models.0, &manifest(bytes)).unwrap();
        fs::write(&staged, bytes).unwrap();
        let second = commit_verified_model(&staged, &models.0, &manifest(bytes)).unwrap();
        assert_eq!(first, second);
    }
}
