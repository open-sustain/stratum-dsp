# Quality Audit

This document tracks the standing audit program for getting `stratum-dsp` to
production-grade quality for Sustain and upstream users. It is intentionally
separate from phase progress reports: this file records audit scope, evidence,
and current quality posture.

## Audit Dimensions

- Performance: hot-path CPU, allocations, memory growth, parallelism,
  benchmarks, and regressions.
- Integrity: mathematical correctness, finite numeric behavior, deterministic
  output, benchmark validity, and validation provenance.
- Reliability: error handling, corrupt/hostile audio behavior, panic-free public
  API paths, boundary conditions, and reproducible tests.
- Security: dependency hygiene, decoder attack surface, optional feature risk,
  supply-chain review, and license/provenance correctness.
- Product readiness: BPM, key, beat grid, and waveform contracts fit Sustain
  production workflows and are backed by fixtures and real-track benchmarks.

## Tracking Rules

- Every audit finding gets a stable ID in `docs/QUALITY_BACKLOG.md`.
- Every fix must update the finding with status, evidence, validation commands,
  and residual risk.
- Do not close a finding based on intent. Close it only after the fix is merged
  locally and the relevant verification gate has passed.
- Any benchmark or accuracy claim must name the dataset, version/source, file
  list or manifest, metric, command, and result.
- Keep external audio outside git unless redistribution rights are explicit.

## Baseline Audit - 2026-06-03

Scope: first pass over local code, dependencies, tests, and repo docs after the
Sustain upstream-first plan was established.

Commands run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo tree --all-features --prefix depth
rg -n "unsafe|panic!|unwrap\(|expect\(|todo!|unimplemented!|unreachable!" src examples benches tests -S
rg -n "is_finite|is_nan|is_infinite|partial_cmp" src -S
```

Results:

- Formatting: passed.
- Clippy: passed with `--all-targets --all-features -D warnings`.
- Tests: passed. `223` unit tests, `5` integration tests, and `46` doctests.
- Dependency scan: confirms `symphonia` 0.5 is still a normal dependency with
  `features = ["all"]`; optional `ml` pulls `ort` and its network/TLS stack
  under `--all-features`.
- Static scan: no `unsafe` use found in active Rust paths. Production-path
  `partial_cmp(...).unwrap()` occurrences exist and are unsafe if NaN reaches
  internal vectors.

Post-`Q-002` update:

- `symphonia` is no longer in the normal dependency graph. It remains available
  only through `[dev-dependencies]` for example CLIs.
- `cargo tree -e normal -i symphonia` prints nothing.
- `cargo tree --edges dev -i symphonia` shows only the dev-dependency path.

Post-`Q-008` update:

- Example CLIs now use `symphonia` 0.6 through shared example decoder code.
- `tests/example_decoder_tests.rs` verifies decoded WAV sample equivalence
  against `hound` for all tracked fixtures and checks non-audio input returns an
  error.
- Fixture smoke testing of the release CLIs succeeded for all four tracked WAV
  fixtures.
- The smoke runs exposed a separate determinism/product-semantics finding:
  low-confidence key labels can vary across single-file and batch CLI runs.
  This is tracked as `Q-010`.

Post-`Q-001` update:

- `analyze_audio` now validates empty input, zero sample rate, and non-finite
  samples through one public-boundary helper before preprocessing or DSP work.
- Regression tests cover NaN, `+Inf`, `-Inf`, and the public API early-rejection
  path.
- Finite-amplitude range policy and public `AnalysisConfig` validation remain
  open as `Q-011`.

Post-`Q-010` update:

- Key-score tie handling is now deterministic in the key module and related
  aggregation paths.
- Repeated and parallel fixture tests pass for public `analyze_audio` key
  output.
- Release CLI smoke tests now agree between `analyze_file` and `analyze_batch`
  for the checked low-confidence fixtures.

Post-`Q-011` update:

- `AnalysisConfig::validate()` now rejects non-finite config values and core
  structurally invalid analysis parameters before DSP work.
- `analyze_audio` invokes config validation at the public boundary.
- Finite sample amplitude range is intentionally not rejected yet to preserve
  backward compatibility; a recommended headroom/input-contract policy can be
  added later without breaking callers.

Post-`Q-003` update:

- The unimplemented `ml` feature no longer pulls `ort` or its network/TLS
  transitive dependency stack.
- The `ml` feature remains available, and the former implicit `ort` feature is
  preserved as a no-op compatibility alias for existing Cargo feature lists.
- Reintroducing ONNX runtime dependencies requires an implemented ML path,
  explicit runtime/download policy, supply-chain review, and benchmark evidence.

Post-`Q-007` / `Q-012` update:

- `validation/benchmarks/` now defines benchmark corpus policy, a researched
  source-level corpus registry, and a file-level manifest template.
- `docs/literature/REFRESH_AUDIT_2026.md` records a current literature and
  source audit for evaluation methodology, BPM/tempo, key/HPCP, beat/downbeat,
  waveform envelopes, and benchmark datasets.
- These documents do not close the validation gap yet. File-level manifests,
  corpus adapters, checksums, metrics, and validation runs remain required.

Post-`Q-013` update:

- `validation/_metrics.py` now provides shared BPM ratio-bucket helpers and
  MIREX-style key scoring.
- `validation/_metrics.py` also includes dependency-free beat/downbeat
  precision, recall, and F-measure helpers using the mir_eval-compatible default
  `0.07` second tolerance and optional early-event trimming.
- `run_validation.py` writes additive metric columns without removing or
  renaming existing CSV fields.
- Existing analysis scripts can compute the shared metrics from old result
  files, but JAMS/corpus-manifest ingestion and beat/downbeat CSV wiring remain
  open.

Post-`Q-006` update:

- `src/waveform.rs` now provides an additive deterministic waveform API for
  mono sample slices, with multi-resolution min/max/RMS/peak envelope output.
- `docs/WAVEFORM.md` defines the v1 Sustain-facing waveform contract, including
  resolution, partial-window behavior, validation, determinism, and
  serialization shape.
- The implementation does not change `analyze_audio` or existing result
  structures. Future waveform work should focus on external golden fixtures,
  split-channel policy if Sustain needs it, and high-volume profiling.

Post-`Q-004` update:

- `compute_stft` now reuses one FFT input buffer across frames and validates
  direct zero frame/hop size calls.
- `analyze_audio` now avoids cloning the full processed buffer when silence
  trimming is disabled.
- `benches/audio_analysis_bench.rs` now includes direct `stft` and `waveform`
  benchmark groups plus no-trim full-analysis coverage.
- `docs/PERFORMANCE_AUDIT_2026.md` records local Criterion baselines for STFT,
  waveform generation, and no-trim full analysis. This does not close `Q-004`
  yet: allocator profiling, real-track manifest throughput, the initial
  normalization input copy, and multi-resolution spectrogram reuse remain open.

## Initial Findings Summary

The codebase has a strong foundation: CI-style checks are green, test volume is
healthy, public APIs are documented, and the active library has no `unsafe`.

The most important gaps before claiming production-grade Sustain readiness are:

- File-level benchmark corpus manifests for all BPM/key/beat/waveform claims.
- Standardized MIR evaluation tooling for JAMS/mir_eval/MIREX-style metrics.
- Allocator profiling and real-track throughput for full analysis, HPSS,
  multi-resolution tempogram, key paths, and avoidable copies.
- Security review before any future ML/ORT runtime dependency is reintroduced.
- External golden waveform fixtures, Sustain integration checks, and
  high-volume waveform profiling.
- Explicit uncertain-key product semantics, if Sustain needs more than stable
  low-confidence key labels.
- Public input-contract documentation for recommended finite amplitude/headroom
  policy.

Current actionable items are tracked in `docs/QUALITY_BACKLOG.md`.
