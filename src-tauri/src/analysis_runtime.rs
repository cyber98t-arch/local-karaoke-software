use std::{
    env,
    path::{Path, PathBuf},
};

use crate::cache::content_sha256;
use crate::onnx_runtime::ProviderRequest;

pub const SPLEETER_SHA256: &str =
    "8d58651b5979075c954bcbe96c6776b2dc41e7852fb6f2d054b83a696a8db26f";
pub const SPLEETER_BYTES: u64 = 78_652_254;
pub const CREPE_SHA256: &str = "03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4";
pub const CREPE_BYTES: u64 = 1_956_735;
pub const WHISPER_MODEL_SHA256: &str =
    "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe";
pub const WHISPER_MODEL_BYTES: u64 = 147_951_465;
pub const WHISPER_CLI_SHA256: &str =
    "e8c1a1390c22ec4bb48c5a32ccd65942303c8633b09d6f897e1a1562d19ba7f2";
pub const WHISPER_CLI_BYTES: u64 = 2_115_584;

#[derive(Clone, Debug)]
pub struct NativeAnalysisConfig {
    pub spleeter_model: PathBuf,
    pub crepe_model: PathBuf,
    pub whisper_model: PathBuf,
    pub whisper_cli: PathBuf,
    pub onnx_provider: ProviderRequest,
}

impl NativeAnalysisConfig {
    pub fn from_development_environment() -> Result<Self, String> {
        let config = Self {
            spleeter_model: required_path("HIBIKI_SPLEETER_ONNX_MODEL")?,
            crepe_model: required_path("HIBIKI_CREPE_ONNX_MODEL")?,
            whisper_model: required_path("HIBIKI_WHISPER_MODEL")?,
            whisper_cli: required_path("HIBIKI_WHISPER_CLI")?,
            onnx_provider: ProviderRequest::parse(
                env::var("HIBIKI_ONNX_PROVIDER").ok().as_deref(),
            )?,
        };
        config.verify()?;
        Ok(config)
    }

    pub fn verify(&self) -> Result<(), String> {
        verify_artifact(
            &self.spleeter_model,
            SPLEETER_BYTES,
            SPLEETER_SHA256,
            "Spleeter model",
        )?;
        verify_artifact(&self.crepe_model, CREPE_BYTES, CREPE_SHA256, "CREPE model")?;
        verify_artifact(
            &self.whisper_model,
            WHISPER_MODEL_BYTES,
            WHISPER_MODEL_SHA256,
            "Whisper model",
        )?;
        verify_artifact(
            &self.whisper_cli,
            WHISPER_CLI_BYTES,
            WHISPER_CLI_SHA256,
            "whisper.cpp CLI",
        )?;
        Ok(())
    }
}

fn required_path(name: &str) -> Result<PathBuf, String> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{name} is not configured for development analysis"))
}

fn verify_artifact(path: &Path, bytes: u64, sha256: &str, label: &str) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(|_| format!("{label} is not available"))?;
    if !metadata.is_file() || metadata.len() != bytes {
        return Err(format!(
            "{label} byte size does not match the pinned artifact"
        ));
    }
    if content_sha256(path).map_err(|error| format!("{label} could not be hashed: {error}"))?
        != sha256
    {
        return Err(format!(
            "{label} SHA-256 does not match the pinned artifact"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify_artifact;
    use crate::cache::content_sha256;
    use std::fs;

    #[test]
    fn accepts_only_the_exact_artifact_bytes_and_hash() {
        let path =
            std::env::temp_dir().join(format!("hibiki-runtime-artifact-{}", std::process::id()));
        fs::write(&path, b"pinned artifact").unwrap();
        let hash = content_sha256(&path).unwrap();
        verify_artifact(&path, 15, &hash, "test artifact").unwrap();
        assert!(verify_artifact(&path, 14, &hash, "test artifact").is_err());
        assert!(verify_artifact(&path, 15, &"0".repeat(64), "test artifact").is_err());
        fs::remove_file(path).unwrap();
    }
}
