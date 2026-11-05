use ndarray::Array2;

pub const CREPE_SAMPLE_RATE: u32 = 16_000;
pub const CREPE_WINDOW: usize = 1_024;
pub const CREPE_HOP: usize = 160;
pub const CREPE_BINS: usize = 360;

#[derive(Clone, Debug, PartialEq)]
pub struct PitchPoint {
    pub time_seconds: f32,
    pub frequency_hz: f32,
    pub confidence: f32,
}

pub fn centered_normalized_frames(mono: &[f32]) -> Result<Array2<f32>, String> {
    if mono.is_empty() {
        return Err("CREPE framing requires nonempty mono audio".to_owned());
    }
    if mono.iter().any(|sample| !sample.is_finite()) {
        return Err("CREPE input samples must be finite".to_owned());
    }

    let frame_count = 1 + mono.len() / CREPE_HOP;
    let mut frames = Array2::zeros((frame_count, CREPE_WINDOW));
    let half_window = CREPE_WINDOW / 2;
    for frame_index in 0..frame_count {
        let center = frame_index * CREPE_HOP;
        for sample_index in 0..CREPE_WINDOW {
            let source_index = center + sample_index;
            if source_index >= half_window {
                let source_index = source_index - half_window;
                if source_index < mono.len() {
                    frames[(frame_index, sample_index)] = mono[source_index];
                }
            }
        }

        let mut frame = frames.row_mut(frame_index);
        let mean = frame.iter().sum::<f32>() / CREPE_WINDOW as f32;
        let deviation = (frame
            .iter()
            .map(|sample| (sample - mean) * (sample - mean))
            .sum::<f32>()
            / CREPE_WINDOW as f32)
            .sqrt()
            .max(1.0e-8);
        for sample in &mut frame {
            *sample = (*sample - mean) / deviation;
        }
    }
    Ok(frames)
}

pub fn decode_local_average(salience: &Array2<f32>) -> Result<Vec<PitchPoint>, String> {
    if salience.ncols() != CREPE_BINS {
        return Err("CREPE salience must contain exactly 360 bins".to_owned());
    }
    if salience.iter().any(|value| !value.is_finite()) {
        return Err("CREPE salience values must be finite".to_owned());
    }

    let mut points = Vec::with_capacity(salience.nrows());
    for (frame_index, row) in salience.rows().into_iter().enumerate() {
        let (center, confidence) = row
            .iter()
            .copied()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .ok_or_else(|| "CREPE salience must contain frames".to_owned())?;
        let start = center.saturating_sub(4);
        let end = (center + 5).min(CREPE_BINS);
        let mut weighted_cents = 0.0_f32;
        let mut weight = 0.0_f32;
        for bin in start..end {
            let value = row[bin];
            weighted_cents += value * (1997.3794 + 20.0 * bin as f32);
            weight += value;
        }
        let frequency_hz = if weight > 0.0 {
            10.0 * 2.0_f32.powf((weighted_cents / weight) / 1200.0)
        } else {
            0.0
        };
        points.push(PitchPoint {
            time_seconds: frame_index as f32 * CREPE_HOP as f32 / CREPE_SAMPLE_RATE as f32,
            frequency_hz,
            confidence,
        });
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::{
        centered_normalized_frames, decode_local_average, CREPE_BINS, CREPE_HOP, CREPE_SAMPLE_RATE,
        CREPE_WINDOW,
    };
    use ndarray::Array2;

    #[test]
    fn creates_centered_normalized_frames_at_ten_millisecond_hops() {
        let audio = (0..CREPE_SAMPLE_RATE as usize * 2)
            .map(|index| {
                (2.0 * std::f32::consts::PI * 220.0 * index as f32 / CREPE_SAMPLE_RATE as f32).sin()
            })
            .collect::<Vec<_>>();

        let frames = centered_normalized_frames(&audio).expect("audio should frame");

        assert_eq!(frames.shape(), [201, CREPE_WINDOW]);
        assert_eq!(CREPE_HOP, 160);
        for frame in frames.rows().into_iter().skip(4).take(190) {
            let mean = frame.iter().sum::<f32>() / CREPE_WINDOW as f32;
            let variance = frame
                .iter()
                .map(|sample| (sample - mean) * (sample - mean))
                .sum::<f32>()
                / CREPE_WINDOW as f32;
            assert!(mean.abs() < 1.0e-5);
            assert!((variance.sqrt() - 1.0).abs() < 1.0e-4);
        }
    }

    #[test]
    fn decodes_peak_confidence_and_local_weighted_frequency() {
        let mut salience = Array2::zeros((2, CREPE_BINS));
        salience[(0, 180)] = 0.8;
        salience[(0, 181)] = 0.2;
        salience[(1, 120)] = 0.7;

        let points = decode_local_average(&salience).expect("salience should decode");

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].time_seconds, 0.0);
        assert!((points[1].time_seconds - 0.01).abs() < f32::EPSILON);
        assert_eq!(points[0].confidence, 0.8);
        assert_eq!(points[1].confidence, 0.7);
        assert!(points.iter().all(|point| point.frequency_hz.is_finite()));
        assert!(points.iter().all(|point| point.frequency_hz > 0.0));
    }

    #[test]
    fn rejects_invalid_audio_and_salience_boundaries() {
        assert!(centered_normalized_frames(&[]).is_err());
        assert!(centered_normalized_frames(&[f32::NAN]).is_err());
        assert!(decode_local_average(&Array2::zeros((1, CREPE_BINS - 1))).is_err());

        let mut nonfinite = Array2::zeros((1, CREPE_BINS));
        nonfinite[(0, 0)] = f32::INFINITY;
        assert!(decode_local_average(&nonfinite).is_err());
    }
}
