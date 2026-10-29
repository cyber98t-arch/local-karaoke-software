use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Sample, SampleFormat, SizedSample, StreamConfig, SupportedStreamConfig,
    I24, U24,
};
use serde::Serialize;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

const PROBE_DURATION_MS: u64 = 500;
const PROBE_FREQUENCY_HZ: f32 = 440.0;
const PROBE_AMPLITUDE: f32 = 0.15;

struct SineWave {
    phase: f32,
    phase_step: f32,
}

impl SineWave {
    fn new(sample_rate: u32, frequency_hz: f32) -> Self {
        Self {
            phase: 0.0,
            phase_step: frequency_hz * std::f32::consts::TAU / sample_rate as f32,
        }
    }

    fn next(&mut self) -> f32 {
        let sample = self.phase.sin() * PROBE_AMPLITUDE;
        self.phase = (self.phase + self.phase_step) % std::f32::consts::TAU;
        sample
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackProbeResult {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
}

#[tauri::command]
pub fn play_output_probe(output_device_id: String) -> Result<PlaybackProbeResult, String> {
    let host = cpal::default_host();
    let device_id = output_device_id.parse::<cpal::DeviceId>().map_err(|_| {
        "The selected output device is invalid. Refresh the device list and retry.".to_owned()
    })?;
    let device = host.device_by_id(&device_id).ok_or_else(|| {
        "The selected output device is no longer available. Reconnect it and retry.".to_owned()
    })?;
    let device_name = device.to_string();
    let supported = device.default_output_config().map_err(|_| {
        "The output configuration could not be read. Choose another device and retry.".to_owned()
    })?;
    let result = PlaybackProbeResult {
        device_name,
        sample_rate: supported.sample_rate(),
        channels: supported.channels(),
        duration_ms: PROBE_DURATION_MS,
    };

    play_tone(&device, supported)?;
    Ok(result)
}

fn play_tone(device: &Device, supported: SupportedStreamConfig) -> Result<(), String> {
    match supported.sample_format() {
        SampleFormat::I8 => play_tone_as::<i8>(device, supported.into()),
        SampleFormat::I16 => play_tone_as::<i16>(device, supported.into()),
        SampleFormat::I24 => play_tone_as::<I24>(device, supported.into()),
        SampleFormat::I32 => play_tone_as::<i32>(device, supported.into()),
        SampleFormat::I64 => play_tone_as::<i64>(device, supported.into()),
        SampleFormat::U8 => play_tone_as::<u8>(device, supported.into()),
        SampleFormat::U16 => play_tone_as::<u16>(device, supported.into()),
        SampleFormat::U24 => play_tone_as::<U24>(device, supported.into()),
        SampleFormat::U32 => play_tone_as::<u32>(device, supported.into()),
        SampleFormat::U64 => play_tone_as::<u64>(device, supported.into()),
        SampleFormat::F32 => play_tone_as::<f32>(device, supported.into()),
        SampleFormat::F64 => play_tone_as::<f64>(device, supported.into()),
        _ => Err(
            "The output sample format is unsupported. Choose another device and retry.".to_owned(),
        ),
    }
}

fn play_tone_as<T>(device: &Device, config: StreamConfig) -> Result<(), String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let mut wave = SineWave::new(config.sample_rate, PROBE_FREQUENCY_HZ);
    let stream_error = Arc::new(Mutex::new(None));
    let error_for_callback = Arc::clone(&stream_error);
    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [T], _| write_tone(output, channels, &mut wave),
            move |error| {
                if let Ok(mut stored) = error_for_callback.lock() {
                    *stored = Some(error.to_string());
                }
            },
            None,
        )
        .map_err(|_| {
            "The output stream could not be created. Choose another device and retry.".to_owned()
        })?;
    stream.play().map_err(|_| {
        "The test tone could not start. Check the device volume and retry.".to_owned()
    })?;
    std::thread::sleep(Duration::from_millis(PROBE_DURATION_MS));
    if stream_error
        .lock()
        .map_err(|_| "The output probe state is unavailable.".to_owned())?
        .is_some()
    {
        return Err(
            "The output stream failed during the test tone. Reconnect the device and retry."
                .to_owned(),
        );
    }
    Ok(())
}

fn write_tone<T>(output: &mut [T], channels: usize, wave: &mut SineWave)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = T::from_sample(wave.next());
        for channel in frame {
            *channel = sample;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SineWave, PROBE_AMPLITUDE};

    #[test]
    fn generated_probe_samples_stay_within_safe_amplitude() {
        let mut wave = SineWave::new(48_000, 440.0);
        let samples = (0..48_000).map(|_| wave.next()).collect::<Vec<_>>();

        assert!(samples.iter().all(|sample| sample.abs() <= PROBE_AMPLITUDE));
        assert!(samples.iter().any(|sample| *sample > 0.1));
        assert!(samples.iter().any(|sample| *sample < -0.1));
    }
}
