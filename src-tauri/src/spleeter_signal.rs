use crate::spleeter_inference::StemEstimates;
use ndarray::Array4;
use realfft::{num_complex::Complex32, RealFftPlanner};

pub const FRAME_LENGTH: usize = 4096;
pub const FRAME_STEP: usize = 1024;
pub const FREQUENCY_BINS: usize = FRAME_LENGTH / 2 + 1;
pub const MODEL_FREQUENCIES: usize = 1024;
pub const SEGMENT_FRAMES: usize = 512;
const WINDOW_COMPENSATION_FACTOR: f32 = 2.0 / 3.0;
const MASK_EPSILON: f32 = 1.0e-10;

#[derive(Clone, Debug)]
pub struct StereoSpectrum {
    pub frames: usize,
    pub bins: Vec<Complex32>,
}

pub fn stft(waveform: &[[f32; 2]]) -> Result<StereoSpectrum, String> {
    if waveform.is_empty() {
        return Err("Spleeter STFT requires at least one stereo sample".to_owned());
    }
    let frames = waveform.len().div_ceil(FRAME_STEP);
    let window = periodic_hann();
    let mut planner = RealFftPlanner::<f32>::new();
    let transform = planner.plan_fft_forward(FRAME_LENGTH);
    let mut bins = vec![Complex32::new(0.0, 0.0); frames * FREQUENCY_BINS * 2];

    for frame in 0..frames {
        let frame_start = frame * FRAME_STEP;
        for channel in 0..2 {
            let mut input = transform.make_input_vec();
            for (offset, sample) in input.iter_mut().enumerate() {
                *sample = waveform
                    .get(frame_start + offset)
                    .map_or(0.0, |stereo| stereo[channel])
                    * window[offset];
            }
            let mut output = transform.make_output_vec();
            transform
                .process(&mut input, &mut output)
                .map_err(|error| error.to_string())?;
            for (frequency, value) in output.into_iter().enumerate() {
                bins[spectrum_index(frame, frequency, channel)] = value;
            }
        }
    }
    Ok(StereoSpectrum { frames, bins })
}

pub fn inverse_stft(
    spectrum: &StereoSpectrum,
    output_samples: usize,
) -> Result<Vec<[f32; 2]>, String> {
    if spectrum.frames == 0 || spectrum.bins.len() != spectrum.frames * FREQUENCY_BINS * 2 {
        return Err("invalid Spleeter stereo spectrum shape".to_owned());
    }
    let overlap_samples = (spectrum.frames - 1) * FRAME_STEP + FRAME_LENGTH;
    let mut waveform = vec![[0.0_f32; 2]; overlap_samples.max(output_samples)];
    let window = periodic_hann();
    let scale = WINDOW_COMPENSATION_FACTOR / FRAME_LENGTH as f32;
    let mut planner = RealFftPlanner::<f32>::new();
    let transform = planner.plan_fft_inverse(FRAME_LENGTH);

    for frame in 0..spectrum.frames {
        let frame_start = frame * FRAME_STEP;
        for channel in 0..2 {
            let mut input = transform.make_input_vec();
            for (frequency, value) in input.iter_mut().enumerate() {
                *value = spectrum.bins[spectrum_index(frame, frequency, channel)];
            }
            let mut output = transform.make_output_vec();
            transform
                .process(&mut input, &mut output)
                .map_err(|error| error.to_string())?;
            for (offset, sample) in output.into_iter().enumerate() {
                waveform[frame_start + offset][channel] += sample * window[offset] * scale;
            }
        }
    }
    waveform.truncate(output_samples);
    Ok(waveform)
}

pub fn model_input(spectrum: &StereoSpectrum) -> Result<Array4<f32>, String> {
    if spectrum.frames == 0 || spectrum.bins.len() != spectrum.frames * FREQUENCY_BINS * 2 {
        return Err("invalid Spleeter stereo spectrum shape".to_owned());
    }
    let segments = spectrum.frames.div_ceil(SEGMENT_FRAMES);
    let mut input = Array4::zeros((segments, SEGMENT_FRAMES, MODEL_FREQUENCIES, 2));
    for frame in 0..spectrum.frames {
        let segment = frame / SEGMENT_FRAMES;
        let segment_frame = frame % SEGMENT_FRAMES;
        for frequency in 0..MODEL_FREQUENCIES {
            for channel in 0..2 {
                input[[segment, segment_frame, frequency, channel]] =
                    spectrum.bins[spectrum_index(frame, frequency, channel)].norm();
            }
        }
    }
    Ok(input)
}

pub fn masked_spectra(
    mixture: &StereoSpectrum,
    estimates: &StemEstimates,
) -> Result<(StereoSpectrum, StereoSpectrum), String> {
    let expected_segments = mixture.frames.div_ceil(SEGMENT_FRAMES);
    let expected_shape = [expected_segments, SEGMENT_FRAMES, MODEL_FREQUENCIES, 2];
    if estimates.vocals.shape() != expected_shape
        || estimates.accompaniment.shape() != expected_shape
    {
        return Err("Spleeter estimates do not match the mixture shape".to_owned());
    }
    let mut vocals = vec![Complex32::new(0.0, 0.0); mixture.bins.len()];
    let mut accompaniment = vec![Complex32::new(0.0, 0.0); mixture.bins.len()];
    for frame in 0..mixture.frames {
        let segment = frame / SEGMENT_FRAMES;
        let segment_frame = frame % SEGMENT_FRAMES;
        for frequency in 0..MODEL_FREQUENCIES {
            for channel in 0..2 {
                let vocal_power =
                    estimates.vocals[[segment, segment_frame, frequency, channel]].powi(2);
                let accompaniment_power =
                    estimates.accompaniment[[segment, segment_frame, frequency, channel]].powi(2);
                let denominator = vocal_power + accompaniment_power + MASK_EPSILON;
                let vocal_mask = (vocal_power + MASK_EPSILON / 2.0) / denominator;
                let accompaniment_mask = (accompaniment_power + MASK_EPSILON / 2.0) / denominator;
                let index = spectrum_index(frame, frequency, channel);
                vocals[index] = mixture.bins[index] * vocal_mask;
                accompaniment[index] = mixture.bins[index] * accompaniment_mask;
            }
        }
    }
    Ok((
        StereoSpectrum {
            frames: mixture.frames,
            bins: vocals,
        },
        StereoSpectrum {
            frames: mixture.frames,
            bins: accompaniment,
        },
    ))
}

fn spectrum_index(frame: usize, frequency: usize, channel: usize) -> usize {
    (frame * FREQUENCY_BINS + frequency) * 2 + channel
}

fn periodic_hann() -> Vec<f32> {
    (0..FRAME_LENGTH)
        .map(|index| {
            0.5 - 0.5 * (2.0 * std::f32::consts::PI * index as f32 / FRAME_LENGTH as f32).cos()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        inverse_stft, masked_spectra, model_input, spectrum_index, stft, FRAME_LENGTH,
        SEGMENT_FRAMES,
    };
    use crate::spleeter_inference::StemEstimates;
    use ndarray::Array4;

    #[test]
    fn generated_stereo_round_trips_in_the_fully_overlapped_region() {
        let sample_rate = 44_100.0_f32;
        let waveform = (0..44_100)
            .map(|index| {
                let time = index as f32 / sample_rate;
                [
                    0.1 * (2.0 * std::f32::consts::PI * 220.0 * time).sin(),
                    0.1 * (2.0 * std::f32::consts::PI * 330.0 * time).sin(),
                ]
            })
            .collect::<Vec<_>>();

        let spectrum = stft(&waveform).expect("STFT should succeed");
        let reconstructed =
            inverse_stft(&spectrum, waveform.len()).expect("inverse STFT should succeed");

        assert_eq!(reconstructed.len(), waveform.len());
        for (actual, expected) in reconstructed[FRAME_LENGTH..waveform.len() - FRAME_LENGTH]
            .iter()
            .zip(&waveform[FRAME_LENGTH..waveform.len() - FRAME_LENGTH])
        {
            for channel in 0..2 {
                assert!((actual[channel] - expected[channel]).abs() < 2.0e-5);
            }
        }
    }

    #[test]
    fn partitions_magnitudes_into_zero_padded_model_segments() {
        let waveform = vec![[0.0_f32; 2]; 44_100];
        let spectrum = stft(&waveform).expect("STFT should succeed");

        let input = model_input(&spectrum).expect("model input should be built");

        assert_eq!(input.shape(), &[1, SEGMENT_FRAMES, 1024, 2]);
        assert!(input.iter().all(|value| *value == 0.0));
    }

    #[test]
    fn ratio_masks_reconstruct_the_processed_mixture_band() {
        let waveform = vec![[0.25_f32, -0.25_f32]; 4096];
        let spectrum = stft(&waveform).expect("STFT should succeed");
        let shape = (1, 512, 1024, 2);
        let estimates = StemEstimates {
            vocals: Array4::from_elem(shape, 3.0),
            accompaniment: Array4::from_elem(shape, 4.0),
        };

        let (vocals, accompaniment) =
            masked_spectra(&spectrum, &estimates).expect("masks should be constructed");

        for frame in 0..spectrum.frames {
            for frequency in 0..1024 {
                for channel in 0..2 {
                    let index = spectrum_index(frame, frequency, channel);
                    let tolerance = 1.0e-5_f32.max(spectrum.bins[index].norm() * 1.0e-6);
                    assert!(
                        (vocals.bins[index] + accompaniment.bins[index] - spectrum.bins[index])
                            .norm()
                            < tolerance
                    );
                }
            }
        }
    }
}
