//! Tests for the example CLI decoder path.

use std::error::Error;
use std::path::{Path, PathBuf};

#[path = "../examples/common/mod.rs"]
mod example_common;

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

fn load_wav_with_hound(path: &Path) -> Result<(Vec<f32>, u32), Box<dyn Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let max_value = (1 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|s| s as f32 / max_value))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    let channels = spec.channels as usize;
    let mono_samples = if channels > 1 {
        samples
            .chunks_exact(channels)
            .map(|frame| frame.iter().copied().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };

    Ok((mono_samples, spec.sample_rate))
}

#[test]
fn example_decoder_matches_hound_for_wav_fixtures() -> Result<(), Box<dyn Error>> {
    for fixture in [
        "120bpm_4bar.wav",
        "128bpm_4bar.wav",
        "cmajor_scale.wav",
        "mixed_silence.wav",
    ] {
        let path = fixture_path(fixture);
        let path_str = path.to_str().ok_or("fixture path is not valid UTF-8")?;

        let (decoded, decoded_sample_rate) = example_common::decode_audio_file(path_str)?;
        let (expected, expected_sample_rate) = load_wav_with_hound(&path)?;

        assert_eq!(decoded_sample_rate, expected_sample_rate, "{fixture}");
        assert_eq!(decoded.len(), expected.len(), "{fixture}");

        let max_abs_diff = decoded
            .iter()
            .zip(expected.iter())
            .map(|(actual, expected)| (actual - expected).abs())
            .fold(0.0_f32, f32::max);

        assert!(
            max_abs_diff <= 1.0e-5,
            "{fixture}: max decoded sample diff was {max_abs_diff}"
        );
    }

    Ok(())
}

#[test]
fn example_decoder_rejects_non_audio_input() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!(
        "stratum_dsp_invalid_audio_{}.bin",
        std::process::id()
    ));

    std::fs::write(&path, b"not an audio file")?;
    let result = example_common::decode_audio_file(path.to_str().ok_or("temp path is not UTF-8")?);
    let _ = std::fs::remove_file(path);

    assert!(result.is_err());

    Ok(())
}
