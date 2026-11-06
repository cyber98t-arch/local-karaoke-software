use rubato::{audioadapter_buffers::direct::InterleavedSlice, Fft, FixedSync, Resampler};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use symphonia::{
    core::{
        codecs::audio::AudioDecoderOptions,
        errors::Error as SymphoniaError,
        formats::{probe::Hint, FormatOptions, TrackType},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::{get_codecs, get_probe},
};

#[derive(Debug, PartialEq)]
pub struct DecodedAudio {
    pub sample_rate: u32,
    pub stereo_frames: Vec<[f32; 2]>,
}

pub fn write_pcm16_wav(
    output_path: &Path,
    sample_rate: u32,
    frames: &[[f32; 2]],
) -> io::Result<PathBuf> {
    if sample_rate == 0 || frames.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "WAV input must be nonempty",
        ));
    }
    let data_bytes = frames
        .len()
        .checked_mul(4)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "WAV input is too large"))?;
    let riff_bytes = 36u32
        .checked_add(data_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "WAV input is too large"))?;
    let temporary = output_path.with_extension("wav.tmp");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::remove_file(&temporary) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(b"RIFF")?;
    file.write_all(&riff_bytes.to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * 4).to_le_bytes())?;
    file.write_all(&4u16.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    for sample in frames.iter().flatten() {
        if !sample.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "WAV samples must be finite",
            ));
        }
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        file.write_all(&value.to_le_bytes())?;
    }
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, output_path)?;
    Ok(output_path.to_path_buf())
}

pub const SPLEETER_SAMPLE_RATE: u32 = 44_100;

pub fn resample_for_spleeter(audio: DecodedAudio) -> Result<DecodedAudio, String> {
    if audio.sample_rate == SPLEETER_SAMPLE_RATE {
        return Ok(audio);
    }
    if audio.sample_rate == 0 || audio.stereo_frames.is_empty() {
        return Err("resampling requires nonempty audio with a sample rate".to_owned());
    }

    let input_frames = audio.stereo_frames.len();
    let interleaved = audio
        .stereo_frames
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let input = InterleavedSlice::new(&interleaved, 2, input_frames)
        .map_err(|error| format!("invalid stereo input for resampling: {error}"))?;
    let mut resampler = Fft::<f32>::new(
        audio.sample_rate as usize,
        SPLEETER_SAMPLE_RATE as usize,
        1024,
        2,
        FixedSync::Both,
    )
    .map_err(|error| format!("failed to configure analysis resampling: {error}"))?;
    let output = resampler
        .process_all(&input, input_frames, None)
        .map_err(|error| format!("analysis resampling failed: {error}"))?
        .take_data();
    let stereo_frames = output
        .chunks_exact(2)
        .map(|frame| [frame[0], frame[1]])
        .collect();

    Ok(DecodedAudio {
        sample_rate: SPLEETER_SAMPLE_RATE,
        stereo_frames,
    })
}

pub fn decode_for_analysis(source_path: &Path) -> io::Result<DecodedAudio> {
    let source = File::open(source_path)?;
    let stream = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = source_path.extension().and_then(|value| value.to_str()) {
        hint.with_extension(extension);
    }

    let mut format = get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|error| invalid_data(format!("audio format probe failed: {error}")))?;
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| invalid_data("source has no audio track"))?;
    let track_id = track.id;
    let codec_params = track
        .codec_params
        .as_ref()
        .and_then(|params| params.audio())
        .ok_or_else(|| invalid_data("audio track has no codec parameters"))?;
    let mut decoder = get_codecs()
        .make_audio_decoder(codec_params, &AudioDecoderOptions::default())
        .map_err(invalid_audio)?;

    let mut sample_rate = None;
    let mut stereo_frames = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break,
            Err(SymphoniaError::IoError(error))
                if error.kind() == io::ErrorKind::UnexpectedEof && !stereo_frames.is_empty() =>
            {
                break;
            }
            Err(error) => {
                return Err(invalid_data(format!(
                    "audio packet reading failed: {error}"
                )))
            }
        };
        if packet.track_id != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::IoError(error))
                if error.kind() == io::ErrorKind::UnexpectedEof && !stereo_frames.is_empty() =>
            {
                break;
            }
            Err(error) => {
                return Err(invalid_data(format!(
                    "audio packet decoding failed: {error}"
                )))
            }
        };
        let spec = decoded.spec();
        let channels = spec.channels().count();
        if !(1..=2).contains(&channels) {
            return Err(invalid_data(format!(
                "analysis supports mono or stereo sources, found {channels} channels"
            )));
        }
        match sample_rate {
            Some(previous) if previous != spec.rate() => {
                return Err(invalid_data("sample rate changed during decoding"));
            }
            None => sample_rate = Some(spec.rate()),
            _ => {}
        }

        let mut interleaved = vec![0.0_f32; decoded.samples_interleaved()];
        decoded.copy_to_slice_interleaved(&mut interleaved);
        if channels == 1 {
            stereo_frames.extend(interleaved.into_iter().map(|sample| [sample, sample]));
        } else {
            stereo_frames.extend(
                interleaved
                    .chunks_exact(2)
                    .map(|frame| [frame[0], frame[1]]),
            );
        }
    }

    if stereo_frames.is_empty() {
        return Err(invalid_data("audio track decoded no samples"));
    }

    Ok(DecodedAudio {
        sample_rate: sample_rate.ok_or_else(|| invalid_data("audio has no sample rate"))?,
        stereo_frames,
    })
}

fn invalid_audio(error: SymphoniaError) -> io::Error {
    invalid_data(error)
}

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::{decode_for_analysis, resample_for_spleeter, write_pcm16_wav, DecodedAudio};
    use std::{fs, path::PathBuf};

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn decodes_supported_sources_to_finite_stereo_at_native_rate() {
        for name in ["metadata.mp3", "metadata.flac", "metadata.wav"] {
            let decoded = decode_for_analysis(&fixture(name)).expect("audio should decode");

            assert_eq!(decoded.sample_rate, 44_100, "{name} should retain its rate");
            assert!(
                !decoded.stereo_frames.is_empty(),
                "{name} should have frames"
            );
            assert!(
                decoded
                    .stereo_frames
                    .iter()
                    .all(|frame| frame[0] == frame[1]),
                "{name} mono samples should be duplicated to stereo"
            );
            assert!(
                decoded
                    .stereo_frames
                    .iter()
                    .flatten()
                    .all(|sample| sample.is_finite()),
                "{name} samples should be finite"
            );
        }
    }

    #[test]
    fn decoding_does_not_change_source_or_its_directory() {
        for name in ["metadata.mp3", "metadata.flac", "metadata.wav"] {
            let path = fixture(name);
            let parent = path.parent().expect("fixture should have a parent");
            let before_bytes = fs::read(&path).expect("fixture should be readable");
            let mut before_entries = fs::read_dir(parent)
                .expect("fixture directory should be readable")
                .map(|entry| entry.expect("entry should be readable").file_name())
                .collect::<Vec<_>>();
            before_entries.sort();

            decode_for_analysis(&path).expect("audio should decode");

            let mut after_entries = fs::read_dir(parent)
                .expect("fixture directory should remain readable")
                .map(|entry| entry.expect("entry should remain readable").file_name())
                .collect::<Vec<_>>();
            after_entries.sort();
            assert_eq!(
                fs::read(&path).expect("source should remain readable"),
                before_bytes
            );
            assert_eq!(after_entries, before_entries);
        }
    }

    #[test]
    fn writes_a_cache_owned_pcm_wav_that_round_trips() {
        let directory =
            std::env::temp_dir().join(format!("hibiki-whisper-wav-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir(&directory).unwrap();
        let path = directory.join("vocals.wav");
        let frames = vec![[0.25, -0.25]; 800];

        write_pcm16_wav(&path, 16_000, &frames).unwrap();
        let decoded = decode_for_analysis(&path).unwrap();

        assert_eq!(decoded.sample_rate, 16_000);
        assert_eq!(decoded.stereo_frames.len(), frames.len());
        assert!(!path.with_extension("wav.tmp").exists());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn resamples_96khz_stereo_to_spleeter_rate() {
        let frames = (0..96_000)
            .map(|index| {
                let phase = 2.0 * std::f32::consts::PI * 440.0 * index as f32 / 96_000.0;
                [0.2 * phase.sin(), 0.1 * phase.cos()]
            })
            .collect();

        let converted = resample_for_spleeter(DecodedAudio {
            sample_rate: 96_000,
            stereo_frames: frames,
        })
        .expect("96 kHz audio should resample");

        assert_eq!(converted.sample_rate, 44_100);
        assert_eq!(converted.stereo_frames.len(), 44_100);
        assert!(converted
            .stereo_frames
            .iter()
            .flatten()
            .all(|v| v.is_finite()));
        assert!(converted
            .stereo_frames
            .iter()
            .any(|frame| (frame[0] - frame[1]).abs() > 0.01));
    }
}
