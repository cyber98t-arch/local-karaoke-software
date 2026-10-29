use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheManifest {
    pub schema_version: u32,
    pub source_sha256: String,
    pub pipeline_version: String,
}

impl CacheManifest {
    pub fn new(source_sha256: String, pipeline_version: String) -> Self {
        Self {
            schema_version: 1,
            source_sha256,
            pipeline_version,
        }
    }
}

pub fn content_sha256(source_path: &Path) -> io::Result<String> {
    let mut source = File::open(source_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = source.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn manifest_path(
    cache_root: &Path,
    source_sha256: &str,
    pipeline_version: &str,
) -> io::Result<PathBuf> {
    if source_sha256.len() != 64
        || !source_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid_identity("source SHA-256"));
    }

    if pipeline_version.is_empty()
        || matches!(pipeline_version, "." | "..")
        || !pipeline_version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(invalid_identity("pipeline version"));
    }

    Ok(cache_root
        .join(source_sha256)
        .join(pipeline_version)
        .join("manifest.json"))
}

pub fn write_manifest_atomic(cache_root: &Path, manifest: &CacheManifest) -> io::Result<PathBuf> {
    if manifest.schema_version != 1 {
        return Err(invalid_manifest("unsupported schema version"));
    }

    let final_path = manifest_path(
        cache_root,
        &manifest.source_sha256,
        &manifest.pipeline_version,
    )?;
    let temporary_path = final_path.with_extension("json.tmp");

    if final_path.exists() {
        validate_existing_manifest(&final_path, manifest)?;
        remove_if_present(&temporary_path)?;
        return Ok(final_path);
    }

    let parent = final_path
        .parent()
        .ok_or_else(|| invalid_manifest("manifest path has no parent"))?;
    fs::create_dir_all(parent)?;

    let encoded = serde_json::to_vec_pretty(manifest)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut temporary = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary_path)?;
    temporary.write_all(&encoded)?;
    temporary.write_all(b"\n")?;
    temporary.sync_all()?;
    drop(temporary);

    fs::rename(&temporary_path, &final_path)?;
    Ok(final_path)
}

fn validate_existing_manifest(path: &Path, expected: &CacheManifest) -> io::Result<()> {
    let existing: CacheManifest = serde_json::from_reader(File::open(path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if existing != *expected {
        return Err(invalid_manifest(
            "existing manifest does not match cache identity",
        ));
    }
    Ok(())
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn invalid_identity(component: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid cache identity component: {component}"),
    )
}

fn invalid_manifest(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::{content_sha256, manifest_path, write_manifest_atomic, CacheManifest};
    use std::{
        fs,
        io::ErrorKind,
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
            let path =
                std::env::temp_dir().join(format!("hibiki-cache-{}-{unique}", std::process::id()));
            fs::create_dir(&path).expect("test directory should be created");
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).expect("test directory should be removable");
        }
    }

    #[test]
    fn hashes_source_content_without_changing_it() {
        let directory = TestDirectory::new();
        let source_path = directory.path.join("song.wav");
        fs::write(&source_path, b"abc").expect("fixture should be written");

        let hash = content_sha256(&source_path).expect("source should be hashed");

        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            fs::read(&source_path).expect("source should remain readable"),
            b"abc"
        );
    }

    #[test]
    fn builds_manifest_path_below_injected_cache_root() {
        let cache_root = PathBuf::from("application-cache");
        let hash = "a".repeat(64);

        let path = manifest_path(&cache_root, &hash, "phase2-v1")
            .expect("valid identity should produce a path");

        assert_eq!(
            path,
            cache_root
                .join(hash)
                .join("phase2-v1")
                .join("manifest.json")
        );
    }

    #[test]
    fn rejects_identity_components_that_could_escape_cache_root() {
        let cache_root = PathBuf::from("application-cache");

        let error = manifest_path(&cache_root, &"a".repeat(64), "../outside")
            .expect_err("path traversal must be rejected");

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn atomically_writes_a_versioned_json_manifest() {
        let directory = TestDirectory::new();
        let manifest = CacheManifest::new("a".repeat(64), "phase2-v1".to_owned());

        let path = write_manifest_atomic(&directory.path, &manifest)
            .expect("valid manifest should be committed");

        let decoded: CacheManifest = serde_json::from_slice(
            &fs::read(&path).expect("committed manifest should be readable"),
        )
        .expect("committed manifest should be valid JSON");
        assert_eq!(decoded, manifest);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn replaces_a_stale_temporary_file_before_committing() {
        let directory = TestDirectory::new();
        let manifest = CacheManifest::new("b".repeat(64), "phase2-v1".to_owned());
        let final_path = manifest_path(
            &directory.path,
            &manifest.source_sha256,
            &manifest.pipeline_version,
        )
        .expect("manifest path should be valid");
        fs::create_dir_all(final_path.parent().expect("manifest should have a parent"))
            .expect("cache directory should be created");
        fs::write(final_path.with_extension("json.tmp"), b"interrupted")
            .expect("stale temporary file should be created");

        write_manifest_atomic(&directory.path, &manifest)
            .expect("retry should replace the stale temporary file");

        assert!(final_path.is_file());
        assert!(!final_path.with_extension("json.tmp").exists());
    }

    #[test]
    fn preserves_an_existing_valid_manifest() {
        let directory = TestDirectory::new();
        let manifest = CacheManifest::new("c".repeat(64), "phase2-v1".to_owned());
        let final_path = manifest_path(
            &directory.path,
            &manifest.source_sha256,
            &manifest.pipeline_version,
        )
        .expect("manifest path should be valid");
        fs::create_dir_all(final_path.parent().expect("manifest should have a parent"))
            .expect("cache directory should be created");
        let existing = format!(
            "{{\"schemaVersion\":1,\"sourceSha256\":\"{}\",\"pipelineVersion\":\"phase2-v1\",\"marker\":\"keep\"}}",
            "c".repeat(64)
        );
        fs::write(&final_path, &existing).expect("existing manifest should be written");

        write_manifest_atomic(&directory.path, &manifest)
            .expect("existing valid manifest should be accepted");

        assert_eq!(
            fs::read_to_string(final_path).expect("manifest should remain readable"),
            existing
        );
    }
}
