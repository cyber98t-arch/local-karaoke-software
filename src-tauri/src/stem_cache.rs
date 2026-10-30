use flacenc::{
    bitsink::ByteSink, component::BitRepr, config::Encoder, encode_with_fixed_block_size,
    error::Verify, source::MemSource,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::{analysis_audio::decode_for_analysis, spleeter_inference::SeparatedWaveforms};

const SAMPLE_RATE: u32 = 44_100;
const CHANNELS: usize = 2;
const BITS_PER_SAMPLE: usize = 16;

#[derive(Debug, Eq, PartialEq)]
pub struct StemCacheArtifacts {
    pub vocals_sha256: String,
    pub accompaniment_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StemManifest {
    schema_version: u32,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    frame_count: u64,
    vocals_bytes: u64,
    vocals_sha256: String,
    accompaniment_bytes: u64,
    accompaniment_sha256: String,
}

pub fn commit_stems(
    cache_directory: &Path,
    stems: &SeparatedWaveforms,
) -> io::Result<StemCacheArtifacts> {
    fs::create_dir_all(cache_directory)?;
    let manifest_path = cache_directory.join("stems.json");
    if manifest_path.exists() {
        return validate_existing(cache_directory, &manifest_path);
    }
    if stems.vocals.is_empty() || stems.vocals.len() != stems.accompaniment.len() {
        return Err(invalid_data(
            "both stems must contain the same nonzero frame count",
        ));
    }

    let vocals = encode_flac(&stems.vocals)?;
    let accompaniment = encode_flac(&stems.accompaniment)?;
    let vocals_sha256 = sha256(&vocals);
    let accompaniment_sha256 = sha256(&accompaniment);
    let vocals_final = cache_directory.join("vocals.flac");
    let accompaniment_final = cache_directory.join("accompaniment.flac");
    let vocals_temporary = cache_directory.join("vocals.tmp.flac");
    let accompaniment_temporary = cache_directory.join("accompaniment.tmp.flac");
    let manifest_temporary = cache_directory.join("stems.json.tmp");

    for path in [
        &vocals_final,
        &accompaniment_final,
        &vocals_temporary,
        &accompaniment_temporary,
        &manifest_temporary,
    ] {
        remove_if_present(path)?;
    }
    write_new_synced(&vocals_temporary, &vocals)?;
    write_new_synced(&accompaniment_temporary, &accompaniment)?;
    validate_temporary(&vocals_temporary, stems.vocals.len())?;
    validate_temporary(&accompaniment_temporary, stems.accompaniment.len())?;

    fs::rename(&vocals_temporary, &vocals_final)?;
    fs::rename(&accompaniment_temporary, &accompaniment_final)?;
    let manifest = StemManifest {
        schema_version: 1,
        sample_rate: SAMPLE_RATE,
        channels: CHANNELS as u16,
        bits_per_sample: BITS_PER_SAMPLE as u16,
        frame_count: stems.vocals.len() as u64,
        vocals_bytes: vocals.len() as u64,
        vocals_sha256: vocals_sha256.clone(),
        accompaniment_bytes: accompaniment.len() as u64,
        accompaniment_sha256: accompaniment_sha256.clone(),
    };
    let mut encoded_manifest = serde_json::to_vec_pretty(&manifest).map_err(invalid_data)?;
    encoded_manifest.push(b'\n');
    write_new_synced(&manifest_temporary, &encoded_manifest)?;
    fs::rename(manifest_temporary, manifest_path)?;

    Ok(StemCacheArtifacts {
        vocals_sha256,
        accompaniment_sha256,
    })
}

fn encode_flac(frames: &[[f32; 2]]) -> io::Result<Vec<u8>> {
    let mut samples = Vec::with_capacity(frames.len() * CHANNELS);
    for sample in frames.iter().flatten() {
        if !sample.is_finite() {
            return Err(invalid_data("stem samples must be finite"));
        }
        let scaled = if *sample >= 0.0 {
            sample.clamp(0.0, 1.0) * i16::MAX as f32
        } else {
            sample.clamp(-1.0, 0.0) * -(i16::MIN as f32)
        };
        samples.push(scaled.round() as i32);
    }

    let config = Encoder::default()
        .into_verified()
        .map_err(|(_, error)| invalid_data(error.to_string()))?;
    let source = MemSource::from_samples(&samples, CHANNELS, BITS_PER_SAMPLE, SAMPLE_RATE as usize);
    let mut stream = encode_with_fixed_block_size(&config, source, config.block_size)
        .map_err(|error| invalid_data(error.to_string()))?;
    // Fixed-block FLAC excludes the shorter final frame from the STREAMINFO
    // block-size range. Keeping both values fixed is required by strict readers.
    stream
        .stream_info_mut()
        .set_block_sizes(config.block_size, config.block_size)
        .map_err(|error| invalid_data(error.to_string()))?;
    let mut sink = ByteSink::new();
    stream
        .write(&mut sink)
        .map_err(|error| invalid_data(error.to_string()))?;
    Ok(sink.into_inner())
}

fn validate_temporary(path: &Path, expected_frames: usize) -> io::Result<()> {
    let decoded = decode_for_analysis(path)?;
    if decoded.sample_rate != SAMPLE_RATE || decoded.stereo_frames.len() != expected_frames {
        return Err(invalid_data("encoded stem failed format validation"));
    }
    Ok(())
}

fn validate_existing(
    cache_directory: &Path,
    manifest_path: &Path,
) -> io::Result<StemCacheArtifacts> {
    let manifest: StemManifest =
        serde_json::from_slice(&fs::read(manifest_path)?).map_err(invalid_data)?;
    if manifest.schema_version != 1
        || manifest.sample_rate != SAMPLE_RATE
        || manifest.channels != CHANNELS as u16
        || manifest.bits_per_sample != BITS_PER_SAMPLE as u16
        || manifest.frame_count == 0
    {
        return Err(invalid_data(
            "existing stem manifest has an unsupported format",
        ));
    }
    let vocals = fs::read(cache_directory.join("vocals.flac"))?;
    let accompaniment = fs::read(cache_directory.join("accompaniment.flac"))?;
    if vocals.len() as u64 != manifest.vocals_bytes
        || accompaniment.len() as u64 != manifest.accompaniment_bytes
        || sha256(&vocals) != manifest.vocals_sha256
        || sha256(&accompaniment) != manifest.accompaniment_sha256
    {
        return Err(invalid_data("existing stem cache hash validation failed"));
    }
    Ok(StemCacheArtifacts {
        vocals_sha256: manifest.vocals_sha256,
        accompaniment_sha256: manifest.accompaniment_sha256,
    })
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
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
    use super::commit_stems;
    use crate::{analysis_audio::decode_for_analysis, spleeter_inference::SeparatedWaveforms};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should follow the Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibiki-stem-cache-{}-{timestamp}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("test directory should be removable");
        }
    }

    fn generated_stems() -> SeparatedWaveforms {
        let waveform = (0..44_100)
            .map(|index| {
                let phase = 2.0 * std::f32::consts::PI * 220.0 * index as f32 / 44_100.0;
                [0.2 * phase.sin(), 0.1 * phase.cos()]
            })
            .collect::<Vec<_>>();
        SeparatedWaveforms {
            vocals: waveform.clone(),
            accompaniment: waveform,
        }
    }

    #[test]
    fn commits_two_valid_flac_stems_and_manifest_inside_cache() {
        let directory = TestDirectory::new();
        let artifacts =
            commit_stems(&directory.0, &generated_stems()).expect("generated stems should commit");
        for name in ["vocals.flac", "accompaniment.flac"] {
            let decoded =
                decode_for_analysis(&directory.0.join(name)).expect("committed stem should decode");
            assert_eq!(decoded.sample_rate, 44_100);
            assert_eq!(decoded.stereo_frames.len(), 44_100);
        }
        assert_eq!(artifacts.vocals_sha256.len(), 64);
        assert_eq!(artifacts.accompaniment_sha256.len(), 64);
        assert!(directory.0.join("stems.json").is_file());
        assert!(!directory.0.join("vocals.tmp.flac").exists());
        assert!(!directory.0.join("accompaniment.tmp.flac").exists());
        assert!(!directory.0.join("stems.json.tmp").exists());
    }

    #[test]
    fn preserves_an_existing_valid_two_stem_cache() {
        let directory = TestDirectory::new();
        let first = commit_stems(&directory.0, &generated_stems())
            .expect("first cache commit should succeed");
        let before =
            fs::read(directory.0.join("vocals.flac")).expect("committed vocals should be readable");
        let replacement = SeparatedWaveforms {
            vocals: vec![[0.0, 0.0]; 44_100],
            accompaniment: vec![[0.0, 0.0]; 44_100],
        };

        let second =
            commit_stems(&directory.0, &replacement).expect("valid cache should be reused");

        assert_eq!(second, first);
        assert_eq!(
            fs::read(directory.0.join("vocals.flac"))
                .expect("committed vocals should remain readable"),
            before
        );
    }

    #[test]
    fn rejects_nonfinite_samples_without_committing_artifacts() {
        let directory = TestDirectory::new();
        let invalid = SeparatedWaveforms {
            vocals: vec![[f32::NAN, 0.0]],
            accompaniment: vec![[0.0, 0.0]],
        };

        let error =
            commit_stems(&directory.0, &invalid).expect_err("nonfinite audio must not be cached");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            fs::read_dir(&directory.0)
                .expect("cache directory should remain readable")
                .count(),
            0
        );
    }
}
