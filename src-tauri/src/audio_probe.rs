use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Sample, SampleFormat, SizedSample, StreamConfig, SupportedStreamConfig,
    I24, U24,
};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

const PROBE_DURATION_MS: u64 = 500;
const PROBE_FREQUENCY_HZ: f32 = 440.0;
const PROBE_AMPLITUDE: f32 = 0.15;
const INPUT_PROBE_DURATION_MS: u64 = 1_500;

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputProbeResult {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
    pub callback_count: u64,
    pub average_callback_interval_ms: f64,
    pub peak_level: f32,
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

#[tauri::command]
pub fn probe_input_level(input_device_id: String) -> Result<InputProbeResult, String> {
    let host = cpal::default_host();
    let selected_id = input_device_id.parse::<cpal::DeviceId>().map_err(|_| {
        "The selected input device is invalid. Refresh the device list and retry.".to_owned()
    })?;
    let device = host
        .input_devices()
        .map_err(|_| {
            "Input audio devices could not be listed. Reconnect the device and retry.".to_owned()
        })?
        .find(|device| device.id().ok().as_ref() == Some(&selected_id))
        .ok_or_else(|| {
            "The selected input device is no longer available. Reconnect it and retry.".to_owned()
        })?;
    let device_name = device.to_string();
    let supported = device.default_input_config().map_err(|_| {
        "The input configuration could not be read. Choose another microphone and retry.".to_owned()
    })?;
    let sample_rate = supported.sample_rate();
    let channels = supported.channels();
    let measurement = measure_input(&device, supported)?;

    Ok(InputProbeResult {
        device_name,
        sample_rate,
        channels,
        duration_ms: INPUT_PROBE_DURATION_MS,
        callback_count: measurement.callback_count,
        average_callback_interval_ms: INPUT_PROBE_DURATION_MS as f64
            / measurement.callback_count as f64,
        peak_level: measurement.peak_level,
    })
}

struct InputMeasurement {
    callback_count: u64,
    peak_level: f32,
}

fn measure_input(
    device: &Device,
    supported: SupportedStreamConfig,
) -> Result<InputMeasurement, String> {
    match supported.sample_format() {
        SampleFormat::I8 => measure_input_as::<i8>(device, supported.into()),
        SampleFormat::I16 => measure_input_as::<i16>(device, supported.into()),
        SampleFormat::I24 => measure_input_as::<I24>(device, supported.into()),
        SampleFormat::I32 => measure_input_as::<i32>(device, supported.into()),
        SampleFormat::I64 => measure_input_as::<i64>(device, supported.into()),
        SampleFormat::U8 => measure_input_as::<u8>(device, supported.into()),
        SampleFormat::U16 => measure_input_as::<u16>(device, supported.into()),
        SampleFormat::U24 => measure_input_as::<U24>(device, supported.into()),
        SampleFormat::U32 => measure_input_as::<u32>(device, supported.into()),
        SampleFormat::U64 => measure_input_as::<u64>(device, supported.into()),
        SampleFormat::F32 => measure_input_as::<f32>(device, supported.into()),
        SampleFormat::F64 => measure_input_as::<f64>(device, supported.into()),
        _ => Err(
            "The input sample format is unsupported. Choose another microphone and retry."
                .to_owned(),
        ),
    }
}

fn measure_input_as<T>(device: &Device, config: StreamConfig) -> Result<InputMeasurement, String>
where
    T: SizedSample + Copy,
    f32: FromSample<T>,
{
    let callback_count = Arc::new(AtomicU64::new(0));
    let peak_bits = Arc::new(AtomicU32::new(0.0f32.to_bits()));
    let stream_error = Arc::new(Mutex::new(None));
    let count_for_callback = Arc::clone(&callback_count);
    let peak_for_callback = Arc::clone(&peak_bits);
    let error_for_callback = Arc::clone(&stream_error);
    let stream = device
        .build_input_stream(
            config,
            move |input: &[T], _| {
                count_for_callback.fetch_add(1, Ordering::Relaxed);
                let peak = peak_level(input);
                peak_for_callback.fetch_max(peak.to_bits(), Ordering::Relaxed);
            },
            move |error| {
                if let Ok(mut stored) = error_for_callback.lock() {
                    *stored = Some(error.to_string());
                }
            },
            None,
        )
        .map_err(|_| {
            "The microphone stream could not be created. Check Windows microphone permissions and retry."
                .to_owned()
        })?;
    stream.play().map_err(|_| {
        "The microphone probe could not start. Check Windows microphone permissions and retry."
            .to_owned()
    })?;
    std::thread::sleep(Duration::from_millis(INPUT_PROBE_DURATION_MS));
    drop(stream);
    if stream_error
        .lock()
        .map_err(|_| "The microphone probe state is unavailable.".to_owned())?
        .is_some()
    {
        return Err(
            "The microphone stream failed during the probe. Reconnect it and retry.".to_owned(),
        );
    }
    let callback_count = callback_count.load(Ordering::Relaxed);
    if callback_count == 0 {
        return Err(
            "The microphone returned no audio callbacks. Check Windows microphone permissions and retry."
                .to_owned(),
        );
    }
    Ok(InputMeasurement {
        callback_count,
        peak_level: f32::from_bits(peak_bits.load(Ordering::Relaxed)),
    })
}

fn peak_level<T>(input: &[T]) -> f32
where
    T: Copy,
    f32: FromSample<T>,
{
    input
        .iter()
        .map(|sample| f32::from_sample(*sample).abs().min(1.0))
        .fold(0.0, f32::max)
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
    use super::{peak_level, SineWave, PROBE_AMPLITUDE};

    #[test]
    fn generated_probe_samples_stay_within_safe_amplitude() {
        let mut wave = SineWave::new(48_000, 440.0);
        let samples = (0..48_000).map(|_| wave.next()).collect::<Vec<_>>();

        assert!(samples.iter().all(|sample| sample.abs() <= PROBE_AMPLITUDE));
        assert!(samples.iter().any(|sample| *sample > 0.1));
        assert!(samples.iter().any(|sample| *sample < -0.1));
    }

    #[test]
    fn input_peak_is_bounded_and_does_not_retain_samples() {
        assert_eq!(peak_level(&[0.0f32, -0.25, 0.75]), 0.75);
        assert_eq!(peak_level(&[2.0f32, -3.0]), 1.0);
        assert_eq!(peak_level::<f32>(&[]), 0.0);
    }
}
