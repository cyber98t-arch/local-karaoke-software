use ndarray::Array4;
#[cfg(feature = "native-onnx")]
use ndarray::Ix4;
#[cfg(feature = "native-onnx")]
use ort::{environment::Environment, session::Session, value::TensorRef};
#[cfg(feature = "native-onnx")]
use std::path::Path;

use crate::spleeter_signal::{inverse_stft, masked_spectra, model_input, stft};

#[cfg(feature = "native-onnx")]
const INPUT_NAME: &str = "Abs:0";
#[cfg(feature = "native-onnx")]
const VOCALS_OUTPUT_NAME: &str = "vocals_spectrogram/mul:0";
#[cfg(feature = "native-onnx")]
const ACCOMPANIMENT_OUTPUT_NAME: &str = "accompaniment_spectrogram/mul:0";

#[derive(Debug)]
pub struct StemEstimates {
    pub vocals: Array4<f32>,
    pub accompaniment: Array4<f32>,
}

pub trait SpectrogramInference {
    fn infer(&mut self, input: &Array4<f32>) -> Result<StemEstimates, String>;
}

#[derive(Debug)]
pub struct SeparatedWaveforms {
    pub vocals: Vec<[f32; 2]>,
    pub accompaniment: Vec<[f32; 2]>,
}

pub fn separate_waveform<I>(
    inference: &mut I,
    waveform: &[[f32; 2]],
) -> Result<SeparatedWaveforms, String>
where
    I: SpectrogramInference,
{
    let spectrum = stft(waveform)?;
    let input = model_input(&spectrum)?;
    let estimates = inference.infer(&input)?;
    validate_outputs(&input, &estimates.vocals, &estimates.accompaniment)?;
    let (vocal_spectrum, accompaniment_spectrum) = masked_spectra(&spectrum, &estimates)?;
    Ok(SeparatedWaveforms {
        vocals: inverse_stft(&vocal_spectrum, waveform.len())?,
        accompaniment: inverse_stft(&accompaniment_spectrum, waveform.len())?,
    })
}

#[cfg(feature = "native-onnx")]
pub struct NativeOrtSession {
    _environment: Environment,
    session: Session,
}

#[cfg(feature = "native-onnx")]
impl NativeOrtSession {
    pub fn from_file(model_path: &Path) -> Result<Self, String> {
        if !model_path.is_file() {
            return Err("ONNX model must be an existing file".to_owned());
        }
        let environment = ort::init()
            .with_name("hibiki-cpu-separation")
            .build()
            .map_err(|error| error.to_string())?;
        let session = Session::builder(&environment)
            .map_err(|error| error.to_string())?
            .commit_from_file(model_path)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            _environment: environment,
            session,
        })
    }
}

#[cfg(feature = "native-onnx")]
impl SpectrogramInference for NativeOrtSession {
    fn infer(&mut self, input: &Array4<f32>) -> Result<StemEstimates, String> {
        validate_input(input)?;
        let tensor = TensorRef::from_array_view(input.view()).map_err(|error| error.to_string())?;
        let outputs = self
            .session
            .run(ort::inputs![INPUT_NAME => tensor])
            .map_err(|error| error.to_string())?;
        let vocals = outputs[VOCALS_OUTPUT_NAME]
            .try_extract_array::<f32>()
            .map_err(|error| error.to_string())?
            .into_dimensionality::<Ix4>()
            .map_err(|error| error.to_string())?
            .to_owned();
        let accompaniment = outputs[ACCOMPANIMENT_OUTPUT_NAME]
            .try_extract_array::<f32>()
            .map_err(|error| error.to_string())?
            .into_dimensionality::<Ix4>()
            .map_err(|error| error.to_string())?
            .to_owned();
        validate_outputs(input, &vocals, &accompaniment)?;
        Ok(StemEstimates {
            vocals,
            accompaniment,
        })
    }
}

#[cfg(any(test, feature = "native-onnx"))]
fn validate_input(input: &Array4<f32>) -> Result<(), String> {
    let shape = input.shape();
    if shape[0] == 0 || shape[1..] != [512, 1024, 2] {
        return Err("Spleeter ONNX input must have shape [segments, 512, 1024, 2]".to_owned());
    }
    Ok(())
}

fn validate_outputs(
    input: &Array4<f32>,
    vocals: &Array4<f32>,
    accompaniment: &Array4<f32>,
) -> Result<(), String> {
    if vocals.shape() != input.shape() || accompaniment.shape() != input.shape() {
        return Err("Spleeter ONNX outputs must match the input shape".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        separate_waveform, validate_input, validate_outputs, SpectrogramInference, StemEstimates,
    };
    use ndarray::Array4;

    struct VocalOnlyInference;

    impl SpectrogramInference for VocalOnlyInference {
        fn infer(&mut self, input: &Array4<f32>) -> Result<StemEstimates, String> {
            Ok(StemEstimates {
                vocals: input.clone(),
                accompaniment: Array4::zeros(input.raw_dim()),
            })
        }
    }

    #[test]
    fn accepts_only_the_pinned_spleeter_tensor_contract() {
        let valid = Array4::zeros((1, 512, 1024, 2));
        assert!(validate_input(&valid).is_ok());
        assert!(validate_outputs(&valid, &valid, &valid).is_ok());

        let wrong_input = Array4::zeros((1, 256, 1024, 2));
        assert!(validate_input(&wrong_input).is_err());
        let wrong_output = Array4::zeros((1, 512, 512, 2));
        assert!(validate_outputs(&valid, &wrong_output, &valid).is_err());
    }

    #[test]
    fn separates_generated_waveform_through_the_injected_runtime() {
        let sample_rate = 44_100.0_f32;
        let waveform = (0..44_100)
            .map(|index| {
                let time = index as f32 / sample_rate;
                let sample = 0.1 * (2.0 * std::f32::consts::PI * 220.0 * time).sin();
                [sample, sample]
            })
            .collect::<Vec<_>>();

        let separated = separate_waveform(&mut VocalOnlyInference, &waveform)
            .expect("injected separation should succeed");

        assert_eq!(separated.vocals.len(), waveform.len());
        assert_eq!(separated.accompaniment.len(), waveform.len());
        let vocal_energy = separated
            .vocals
            .iter()
            .flatten()
            .map(|sample| sample * sample)
            .sum::<f32>();
        let accompaniment_energy = separated
            .accompaniment
            .iter()
            .flatten()
            .map(|sample| sample * sample)
            .sum::<f32>();
        assert!(vocal_energy > 100.0);
        assert!(accompaniment_energy < vocal_energy * 1.0e-6);
    }
}
