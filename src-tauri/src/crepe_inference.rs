use ndarray::Array2;
#[cfg(feature = "native-onnx")]
use ndarray::Ix2;
#[cfg(feature = "native-onnx")]
use ort::{environment::Environment, session::Session, value::TensorRef};
#[cfg(feature = "native-onnx")]
use std::path::Path;

use crate::crepe_signal::{decode_local_average, PitchPoint, CREPE_BINS, CREPE_WINDOW};

#[cfg(feature = "native-onnx")]
const INPUT_NAME: &str = "input";
#[cfg(feature = "native-onnx")]
const OUTPUT_NAME: &str = "classifier";

pub trait PitchInference {
    fn infer(&mut self, frames: &Array2<f32>) -> Result<Array2<f32>, String>;
}

pub fn infer_pitch<I>(inference: &mut I, frames: &Array2<f32>) -> Result<Vec<PitchPoint>, String>
where
    I: PitchInference,
{
    validate_frames(frames)?;
    let salience = inference.infer(frames)?;
    validate_salience(frames.nrows(), &salience)?;
    decode_local_average(&salience)
}

fn validate_frames(frames: &Array2<f32>) -> Result<(), String> {
    if frames.nrows() == 0 || frames.ncols() != CREPE_WINDOW {
        return Err("CREPE ONNX input must have shape [frames, 1024]".to_owned());
    }
    if frames.iter().any(|value| !value.is_finite()) {
        return Err("CREPE ONNX input values must be finite".to_owned());
    }
    Ok(())
}

fn validate_salience(frame_count: usize, salience: &Array2<f32>) -> Result<(), String> {
    if salience.shape() != [frame_count, CREPE_BINS] {
        return Err("CREPE ONNX output must have shape [frames, 360]".to_owned());
    }
    if salience.iter().any(|value| !value.is_finite()) {
        return Err("CREPE ONNX output values must be finite".to_owned());
    }
    Ok(())
}

#[cfg(feature = "native-onnx")]
pub struct NativeCrepeSession {
    _environment: Environment,
    session: Session,
}

#[cfg(feature = "native-onnx")]
impl NativeCrepeSession {
    pub fn from_file(model_path: &Path) -> Result<Self, String> {
        if !model_path.is_file() {
            return Err("CREPE ONNX model must be an existing file".to_owned());
        }
        let environment = ort::init()
            .with_name("hibiki-cpu-pitch")
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
impl PitchInference for NativeCrepeSession {
    fn infer(&mut self, frames: &Array2<f32>) -> Result<Array2<f32>, String> {
        validate_frames(frames)?;
        let tensor =
            TensorRef::from_array_view(frames.view()).map_err(|error| error.to_string())?;
        let outputs = self
            .session
            .run(ort::inputs![INPUT_NAME => tensor])
            .map_err(|error| error.to_string())?;
        let salience = outputs[OUTPUT_NAME]
            .try_extract_array::<f32>()
            .map_err(|error| error.to_string())?
            .into_dimensionality::<Ix2>()
            .map_err(|error| error.to_string())?
            .to_owned();
        validate_salience(frames.nrows(), &salience)?;
        Ok(salience)
    }
}

#[cfg(test)]
mod tests {
    use super::{infer_pitch, PitchInference};
    use crate::crepe_signal::{CREPE_BINS, CREPE_WINDOW};
    use ndarray::Array2;
    #[cfg(feature = "native-onnx")]
    use std::path::PathBuf;

    #[cfg(feature = "native-onnx")]
    const RESEARCH_MODEL_SHA256: &str =
        "03fb909a5618fa9f19942e20de749bc8e658a1ac0df4a778b354c15b6f7a59d4";

    struct KnownSalience;

    impl PitchInference for KnownSalience {
        fn infer(&mut self, frames: &Array2<f32>) -> Result<Array2<f32>, String> {
            let mut salience = Array2::zeros((frames.nrows(), CREPE_BINS));
            salience.column_mut(180).fill(0.9);
            Ok(salience)
        }
    }

    struct WrongShape;

    impl PitchInference for WrongShape {
        fn infer(&mut self, frames: &Array2<f32>) -> Result<Array2<f32>, String> {
            Ok(Array2::zeros((frames.nrows(), CREPE_BINS - 1)))
        }
    }

    #[test]
    fn runs_injected_inference_and_decodes_each_frame() {
        let frames = Array2::zeros((3, CREPE_WINDOW));

        let points = infer_pitch(&mut KnownSalience, &frames).expect("inference should succeed");

        assert_eq!(points.len(), 3);
        assert!(points.iter().all(|point| point.confidence == 0.9));
        assert!(points.iter().all(|point| point.frequency_hz > 0.0));
    }

    #[test]
    fn rejects_invalid_input_and_output_shapes() {
        assert!(infer_pitch(&mut KnownSalience, &Array2::zeros((0, CREPE_WINDOW))).is_err());
        assert!(infer_pitch(&mut KnownSalience, &Array2::zeros((1, CREPE_WINDOW - 1))).is_err());
        assert!(infer_pitch(&mut WrongShape, &Array2::zeros((1, CREPE_WINDOW))).is_err());
    }

    #[cfg(feature = "native-onnx")]
    #[test]
    #[ignore = "requires HIBIKI_CREPE_ONNX_MODEL and performs native inference"]
    fn loads_verified_research_model_and_runs_generated_input() {
        use super::NativeCrepeSession;
        use crate::cache::content_sha256;

        let model_path = PathBuf::from(
            std::env::var_os("HIBIKI_CREPE_ONNX_MODEL")
                .expect("HIBIKI_CREPE_ONNX_MODEL must name the converted ONNX file"),
        );
        assert_eq!(
            content_sha256(&model_path).expect("model should be hashable"),
            RESEARCH_MODEL_SHA256,
            "research model hash must match the verified local conversion"
        );
        let mut runtime = NativeCrepeSession::from_file(&model_path)
            .expect("verified research model should load");

        let points = infer_pitch(&mut runtime, &Array2::zeros((2, CREPE_WINDOW)))
            .expect("generated frames should infer");

        assert_eq!(points.len(), 2);
        assert!(points.iter().all(|point| point.frequency_hz.is_finite()));
        assert!(points.iter().all(|point| point.confidence.is_finite()));
    }
}
