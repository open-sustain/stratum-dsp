//! Determinism checks for public analysis output.

use rayon::prelude::*;
use std::path::PathBuf;
use stratum_dsp::{analyze_audio, AnalysisConfig, Key};

fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

fn load_wav(path: &str) -> Result<(Vec<f32>, u32), Box<dyn std::error::Error>> {
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

fn analyze_fixture_key(filename: &str) -> Key {
    let path = fixture_path(filename);
    let (samples, sample_rate) = load_wav(path.to_str().unwrap()).unwrap();
    analyze_audio(&samples, sample_rate, AnalysisConfig::default())
        .unwrap()
        .key
}

#[test]
fn analyze_audio_key_is_stable_across_repeated_runs() {
    for fixture in ["120bpm_4bar.wav", "cmajor_scale.wav"] {
        let first = analyze_fixture_key(fixture);
        for _ in 0..8 {
            assert_eq!(analyze_fixture_key(fixture), first, "{fixture}");
        }
    }
}

#[test]
fn analyze_audio_key_is_stable_across_parallel_runs() {
    for fixture in ["120bpm_4bar.wav", "cmajor_scale.wav"] {
        let keys = (0..8)
            .into_par_iter()
            .map(|_| analyze_fixture_key(fixture))
            .collect::<Vec<_>>();

        assert!(
            keys.iter().all(|key| *key == keys[0]),
            "{fixture}: keys were {keys:?}"
        );
    }
}
