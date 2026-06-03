# Waveform Contract

This document defines the v1 waveform output contract for Stratum DSP.

The waveform API is additive and backward-compatible. It does not change
`analyze_audio` or `AnalysisResult`.

## Public API

```rust
use stratum_dsp::{generate_waveform, WaveformConfig};

let waveform = generate_waveform(&samples, sample_rate, WaveformConfig::default())?;
```

## Input Contract

- Input samples are mono `f32` samples.
- `sample_rate` must be greater than zero.
- Samples must be finite. `NaN`, `+Inf`, and `-Inf` are rejected.
- The function does not decode, normalize, trim silence, or mix channels.
- Stereo or split-channel callers should either mix before calling or call the
  function once per channel.

This matches the current sample-in library policy and keeps audio file decoding
out of `src/`.

## Output Contract

`Waveform` contains:

- `sample_rate`: source sample rate.
- `channels`: currently `1`, because the public library input is mono.
- `total_samples`: source sample count.
- `duration_seconds`: source duration.
- `levels`: finest-to-coarsest waveform levels.

Each `WaveformLevel` contains:

- `samples_per_point`: number of source samples represented by each point.
- `points`: deterministic envelope points.

Each `WaveformPoint` contains:

- `min`: minimum signed sample value in the window.
- `max`: maximum signed sample value in the window.
- `rms`: root-mean-square amplitude in the window.
- `peak`: maximum absolute sample value in the window.

Partial final windows are preserved. Values are recomputed from the original
source samples at each level, not recursively downsampled, so peaks are not lost
through intermediate rounding.

## Resolution Policy

`WaveformConfig::default()` starts at `512` samples per point and generates `4`
levels. Each subsequent level doubles `samples_per_point`.

The config is intentionally small:

- `samples_per_point`
- `levels`

More rendering choices belong in the consuming UI, not in the DSP contract.

## Serialization

All waveform public types derive `Serialize` and `Deserialize`. The field names
are part of the v1 compatibility surface. Additive fields may be introduced in
future versions, but existing fields should not be renamed or removed without a
v2 migration.

## Verification

Current tests cover:

- exact min/max/RMS/peak values on known samples
- partial final windows
- multi-resolution level spacing
- empty input, zero sample rate, non-finite samples, and invalid config

External waveform benchmarks should use the manifest policy in
`validation/benchmarks/` and record deterministic golden outputs for selected
fixtures.
