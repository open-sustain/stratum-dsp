//! Deterministic waveform envelope generation.
//!
//! This module exposes an additive sample-in waveform contract for applications
//! that need visual waveform data without taking on audio decoding dependencies.

use crate::{validate_audio_input, AnalysisError};
use serde::{Deserialize, Serialize};

/// Configuration for waveform generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaveformConfig {
    /// Number of input samples represented by one point in the first level.
    pub samples_per_point: usize,

    /// Number of resolution levels to generate.
    ///
    /// Each subsequent level doubles `samples_per_point`.
    pub levels: usize,
}

impl Default for WaveformConfig {
    fn default() -> Self {
        Self {
            samples_per_point: 512,
            levels: 4,
        }
    }
}

impl WaveformConfig {
    /// Validate waveform generation parameters.
    pub fn validate(&self) -> Result<(), AnalysisError> {
        if self.samples_per_point == 0 {
            return Err(AnalysisError::InvalidInput(
                "Waveform samples_per_point must be greater than zero".to_string(),
            ));
        }
        if self.levels == 0 {
            return Err(AnalysisError::InvalidInput(
                "Waveform levels must be greater than zero".to_string(),
            ));
        }
        Ok(())
    }
}

/// One waveform envelope point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaveformPoint {
    /// Minimum signed sample value in the represented sample window.
    pub min: f32,

    /// Maximum signed sample value in the represented sample window.
    pub max: f32,

    /// Root-mean-square amplitude in the represented sample window.
    pub rms: f32,

    /// Maximum absolute sample value in the represented sample window.
    pub peak: f32,
}

/// One waveform resolution level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaveformLevel {
    /// Number of source samples represented by each point.
    pub samples_per_point: usize,

    /// Envelope points for this resolution level.
    pub points: Vec<WaveformPoint>,
}

/// Multi-resolution waveform envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waveform {
    /// Source sample rate in Hz.
    pub sample_rate: u32,

    /// Number of source channels represented by this waveform.
    ///
    /// The public library API accepts mono samples, so this is currently always
    /// `1`. Stereo or split-channel callers should generate one waveform per
    /// channel or mix before calling this function.
    pub channels: u16,

    /// Total number of source samples.
    pub total_samples: usize,

    /// Source duration in seconds.
    pub duration_seconds: f32,

    /// Resolution levels ordered from finest to coarsest.
    pub levels: Vec<WaveformLevel>,
}

/// Generate a deterministic multi-resolution waveform envelope from mono samples.
///
/// The function does not normalize, trim, or decode audio. It preserves the
/// signed min/max sample bounds for visual drawing and also records RMS and peak
/// amplitude per point for loudness-aware rendering.
///
/// # Errors
///
/// Returns `AnalysisError` for empty input, zero sample rate, non-finite sample
/// values, invalid config, or resolution overflow.
///
/// # Example
///
/// ```
/// use stratum_dsp::{generate_waveform, WaveformConfig};
///
/// let samples = vec![0.0f32; 44_100];
/// let waveform = generate_waveform(&samples, 44_100, WaveformConfig::default())?;
/// assert_eq!(waveform.sample_rate, 44_100);
/// # Ok::<(), stratum_dsp::AnalysisError>(())
/// ```
pub fn generate_waveform(
    samples: &[f32],
    sample_rate: u32,
    config: WaveformConfig,
) -> Result<Waveform, AnalysisError> {
    validate_audio_input(samples, sample_rate)?;
    config.validate()?;

    let mut levels = Vec::with_capacity(config.levels);
    let mut samples_per_point = config.samples_per_point;

    for _ in 0..config.levels {
        levels.push(WaveformLevel {
            samples_per_point,
            points: compute_level(samples, samples_per_point),
        });
        samples_per_point = samples_per_point.checked_mul(2).ok_or_else(|| {
            AnalysisError::InvalidInput(
                "Waveform samples_per_point overflowed while generating levels".to_string(),
            )
        })?;
    }

    Ok(Waveform {
        sample_rate,
        channels: 1,
        total_samples: samples.len(),
        duration_seconds: samples.len() as f32 / sample_rate as f32,
        levels,
    })
}

fn compute_level(samples: &[f32], samples_per_point: usize) -> Vec<WaveformPoint> {
    samples
        .chunks(samples_per_point)
        .map(|chunk| {
            let mut min = f32::INFINITY;
            let mut max = f32::NEG_INFINITY;
            let mut peak = 0.0f32;
            let mut sum_squares = 0.0f64;

            for &sample in chunk {
                min = min.min(sample);
                max = max.max(sample);
                peak = peak.max(sample.abs());
                let sample = sample as f64;
                sum_squares += sample * sample;
            }

            let rms = (sum_squares / chunk.len() as f64).sqrt() as f32;
            WaveformPoint {
                min,
                max,
                rms,
                peak,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 1e-6,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn generate_waveform_computes_min_max_rms_and_peak() {
        let samples = [-1.0, 0.5, 0.25, -0.25];
        let waveform = generate_waveform(
            &samples,
            4,
            WaveformConfig {
                samples_per_point: 2,
                levels: 1,
            },
        )
        .unwrap();

        assert_eq!(waveform.sample_rate, 4);
        assert_eq!(waveform.channels, 1);
        assert_eq!(waveform.total_samples, 4);
        assert_close(waveform.duration_seconds, 1.0);
        assert_eq!(waveform.levels.len(), 1);
        assert_eq!(waveform.levels[0].points.len(), 2);

        let first = waveform.levels[0].points[0];
        assert_close(first.min, -1.0);
        assert_close(first.max, 0.5);
        assert_close(first.peak, 1.0);
        assert_close(first.rms, 0.7905694);

        let second = waveform.levels[0].points[1];
        assert_close(second.min, -0.25);
        assert_close(second.max, 0.25);
        assert_close(second.peak, 0.25);
        assert_close(second.rms, 0.25);
    }

    #[test]
    fn generate_waveform_preserves_partial_final_point() {
        let waveform = generate_waveform(
            &[0.1, 0.2, -0.8],
            3,
            WaveformConfig {
                samples_per_point: 2,
                levels: 1,
            },
        )
        .unwrap();

        assert_eq!(waveform.levels[0].points.len(), 2);
        let last = waveform.levels[0].points[1];
        assert_close(last.min, -0.8);
        assert_close(last.max, -0.8);
        assert_close(last.rms, 0.8);
        assert_close(last.peak, 0.8);
    }

    #[test]
    fn generate_waveform_doubles_resolution_levels() {
        let waveform = generate_waveform(
            &[-0.5, 0.3, 0.9, -0.2, -1.0],
            10,
            WaveformConfig {
                samples_per_point: 2,
                levels: 3,
            },
        )
        .unwrap();

        let samples_per_point: Vec<usize> = waveform
            .levels
            .iter()
            .map(|level| level.samples_per_point)
            .collect();
        assert_eq!(samples_per_point, vec![2, 4, 8]);
        assert_eq!(waveform.levels[0].points.len(), 3);
        assert_eq!(waveform.levels[1].points.len(), 2);
        assert_eq!(waveform.levels[2].points.len(), 1);
        assert_close(waveform.levels[2].points[0].peak, 1.0);
    }

    #[test]
    fn generate_waveform_rejects_invalid_input_and_config() {
        assert!(generate_waveform(&[], 44_100, WaveformConfig::default()).is_err());
        assert!(generate_waveform(&[0.0], 0, WaveformConfig::default()).is_err());
        assert!(generate_waveform(&[f32::NAN], 44_100, WaveformConfig::default()).is_err());
        assert!(generate_waveform(
            &[0.0],
            44_100,
            WaveformConfig {
                samples_per_point: 0,
                levels: 1,
            },
        )
        .is_err());
        assert!(generate_waveform(
            &[0.0],
            44_100,
            WaveformConfig {
                samples_per_point: 1,
                levels: 0,
            },
        )
        .is_err());
    }
}
