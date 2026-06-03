# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> Detailed phase-by-phase implementation history and tuning logs live in `docs/progress-reports/`.

## [1.0.0] - 2025-12-18

### Production Ready
- **BPM Detection**: 87.7% accuracy (±2 BPM) on 155 verified Beatport/ZipDJ tracks
- **Key Detection**: 72.1% accuracy (exact match vs ground truth, matches Mixed In Key performance)
- **Beat Grid**: HMM-based tracking with stability scoring
- **Performance**: ~200ms per 3-minute track, 21 tracks/sec batch throughput (7.7× speedup)

### Core Features
- **Tempogram BPM** (Grosche et al. 2012): FFT + autocorrelation tempogram with multi-resolution escalation
- **Krumhansl-Kessler key detection**: Chroma-based analysis with HPSS preprocessing and circle-of-fifths weighting
- **Confidence scoring**: Comprehensive confidence metrics for all analysis components
- **Parallel batch processing**: CPU-1 workers default for high-throughput library scans

### Validation
- 155 real-world DJ tracks (Beatport, ZipDJ) with verified ground truth
- Full validation results documented in `docs/progress-reports/PHASE_1F_VALIDATION.md`
- Reference baseline: Mixed In Key achieves 98.1% ±2 BPM and 72.1% key accuracy on same dataset

### API
```rust
use stratum_dsp::{analyze_audio, AnalysisConfig};

let result = analyze_audio(&samples, 44100, AnalysisConfig::default())?;
println!("BPM: {:.1} | Key: {} ({})", 
    result.bpm, 
    result.key.name(), 
    result.key.numerical()
);
```

### Documentation
- Complete pipeline documentation (`PIPELINE.md`)
- Validation reports and benchmarks
- Literature reviews for all algorithms
- Development and contribution guides

---

## [Unreleased]

### Planned
- Phase 2 ML refinement (feature-gated `ml`)
- Key detection improvements (harmonic-only chroma, better aggregation)

### Added
- `examples/analyze_batch.rs`: parallel batch processing (CPU-1 workers default)
- `docs/progress-reports/PHASE_1F_BENCHMARKS.md`: batch throughput + outlier analysis
- `CONTRIBUTING.md`: contributor guidelines and development workflow
- `validation/benchmarks/`: benchmark corpus registry, manifest rules, and
  researched candidate corpora for BPM/key/beat-grid/waveform validation.
- `docs/literature/REFRESH_AUDIT_2026.md`: refreshed literature and source
  audit for evaluation methodology, datasets, BPM/key/beat-grid, and waveform
  production references.
- `validation/_metrics.py`: shared validation metric helpers for BPM ratio buckets
  and MIREX-style key categories.
- `validation/_metrics.py`: dependency-free beat/downbeat precision, recall,
  and F-measure helpers using the mir_eval-compatible default 70 ms tolerance.
- `generate_waveform`: additive public API for deterministic multi-resolution
  min/max/RMS/peak waveform envelopes.
- `docs/WAVEFORM.md`: v1 waveform output contract.
- `docs/PERFORMANCE_AUDIT_2026.md`: local performance audit notes, Criterion
  commands, STFT baseline, waveform baselines, and residual profiling risks.
- Criterion benchmark coverage for direct STFT timing and waveform generation.
- Validation tooling cleanup:
  - `validation/tools/` (run scripts) and `validation/analysis/` (post-run analysis)
  - `validation/_id3.py`, `validation/_keys.py`: shared ID3/key parsing utilities
  - `validation/tools/build_hllmr_metadata.py`: GT snapshot tool for real-world DJ tracks
- `archive/`: archived "construction debris" not compiled as part of the crate

### Changed
- Moved `symphonia` from normal dependencies to dev-dependencies because audio
  decoding is used only by example CLIs, not the sample-based library API.
- Migrated example CLI audio decoding to `symphonia` 0.6 with a shared decoder
  helper and narrower dev-only codec feature set.
- Kept the unimplemented `ml` integration point dependency-free while
  preserving `ml` and `ort` feature flags for backward-compatible Cargo feature
  lists.
- Validation results now include additive BPM ratio bucket and MIREX-style key
  scoring columns while preserving the existing CSV workflow.
- `compute_stft` now reuses one FFT input buffer across frames while preserving
  the existing `Vec<Vec<f32>>` magnitude spectrogram return shape.
- `analyze_audio` now borrows the processed sample buffer when silence trimming
  is disabled instead of cloning the whole buffer.
- **README.md**: Major update with validation results table, performance benchmarks, known limitations
- Documentation: top-level docs focus on the current pipeline and canonical workflows
- Defaults: HPSS percussive tempogram fallback is opt-in (avoids multi-second outliers)
- Key detection: Fixed Krumhansl-Kessler template alignment (canonical profiles + L2 normalization)
  - Minor keys now correctly detected (was previously biased toward major)
  - Key accuracy improved from 1.5% to 72.1% vs GT

### Fixed
- `analyze_audio` validates `AnalysisConfig` for finite values and core
  structural constraints before DSP work.
- `compute_stft` now rejects direct zero frame/hop size calls instead of
  allowing invalid public inputs to panic.
- Key-score tie handling is deterministic across repeated and parallel analysis
  runs.
- `analyze_audio` rejects NaN and infinite samples before preprocessing or DSP
  comparison paths.

### Removed
- Unused dependencies: `ndarray`, `ndarray-linalg`
- Inactive ORT/runtime dependency surface from `--all-features` while Phase 2 ML
  remains unimplemented.
- Unimplemented public IO stubs moved out of the crate (archived under `archive/`)
