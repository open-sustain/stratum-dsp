# Quality Backlog

Stable backlog for performance, integrity, reliability, security, and product
readiness work. Keep this file current as findings are discovered, fixed, or
verified.

## Status Model

- Open: finding accepted, no fix in progress.
- In Progress: implementation or research is underway.
- Blocked: cannot proceed without an explicit external decision or missing data.
- Fixed: implementation exists, but verification or documentation is incomplete.
- Verified: fix landed locally and relevant verification evidence is recorded.

## Priority Model

- P0: blocks production readiness or can panic/corrupt results on realistic
  user input.
- P1: high-value quality, security, or performance issue needed for
  best-in-class goals.
- P2: important cleanup or optimization, but not blocking immediate safety.
- P3: nice-to-have or exploratory work.

## Backlog

| ID | Area | Priority | Status | Finding | Evidence | Next Action | Verification |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Q-001 | Reliability / Integrity | P0 | Verified | Public API did not reject or sanitize non-finite samples before DSP work. NaN could propagate into production-path `partial_cmp(...).unwrap()` sorts and panic or produce invalid analysis. | `src/lib.rs` now calls `validate_audio_input` before cloning, normalization, silence trimming, STFT, or sort/comparison-heavy DSP paths. Unit tests cover empty input, zero sample rate, NaN, `+Inf`, `-Inf`, and the public `analyze_audio` early-rejection path. | Completed for non-finite samples. Separate finite-amplitude and config validation policy is tracked as `Q-011`. | Passed on 2026-06-03: `cargo test validate_audio_input` (`3` tests); `cargo test analyze_audio_rejects_non_finite_before_processing` (`1` test); `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`227` unit, `2` decoder integration, `5` fixture integration, `46` doctests); `cargo test --test integration_tests` (`5` tests); `cargo test --test example_decoder_tests` (`2` tests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`. |
| Q-002 | Security / Dependencies | P0 | Verified | `symphonia` 0.5 with `features = ["all"]` was a normal dependency even though the library never uses it. This expanded downstream dependency and decoder surface for Sustain. | `symphonia` is now declared under `[dev-dependencies]`; `cargo tree -e normal -i symphonia` prints nothing; `cargo tree --edges dev -i symphonia` shows only the dev-dependency path to `stratum-dsp`. | Completed; example migration follow-up is verified separately as `Q-008`. | Passed on 2026-06-03: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`223` unit, `5` integration, `46` doctests); `cargo test --test integration_tests` (`5` tests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`. |
| Q-003 | Security / Dependencies | P1 | Verified | Optional `ml` feature pulled `ort` release-candidate dependency and network/TLS stack under `--all-features`, while ONNX inference is not implemented. | `Cargo.toml` now keeps `ml` as a dependency-free Phase 2 feature and defines `ort = []` as a backward-compatible alias for callers that enabled the former implicit optional-dependency feature. `src/ml/mod.rs` documents the ML module as a dependency-free stub while Phase 1 remains pure DSP. | Completed. Reintroduce an ONNX/runtime dependency only with an implemented ML path, explicit runtime/download policy, supply-chain review, and benchmark evidence. | Passed on 2026-06-03: `cargo tree --features ml -i ort` reported no matching `ort` package; `cargo tree --all-features --prefix depth | rg -n "(ort|ureq|native-tls|openssl)"` returned no matches; `cargo build --features ml`; `cargo build --features ort`; `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`232` unit, `2` determinism integration, `2` decoder integration, `5` fixture integration, `46` doctests); `cargo build --release --all-features`. |
| Q-004 | Performance | P1 | In Progress | STFT and pipeline hot paths allocate and copy heavily. This may limit waveform/BPM/key throughput on large Sustain libraries. | `compute_stft` now reuses one FFT input buffer across frames and validates zero frame/hop sizes directly. `analyze_audio` now borrows `processed_samples` when silence trimming is disabled instead of cloning the whole buffer. `benches/audio_analysis_bench.rs` now includes direct STFT, waveform, and no-trim full-analysis benchmark coverage. `docs/PERFORMANCE_AUDIT_2026.md` records local Criterion baselines. Remaining evidence: `src/lib.rs` still copies input samples before normalization; `compute_stft` still allocates one magnitude vector per frame because the public return shape is `Vec<Vec<f32>>`; multi-resolution paths still recompute STFT. | Run allocator profiling and real-manifest throughput before deeper ownership/spectrogram changes. Optimize copies or storage only where profiles show meaningful impact, preserving the current API or documenting a v2-only dead end. | Partial verification passed on 2026-06-03: `cargo test compute_stft` (`2` tests); `cargo test analyze_audio_rejects` (`2` tests); `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`238` unit, `2` determinism integration, `2` decoder integration, `5` fixture integration, `47` doctests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`; Criterion `compute_stft_30s_2048_512` local baseline `5.8776 ms` - `6.1480 ms`; Criterion `generate_waveform_30s_default` `8.5504 ms` - `8.7375 ms`; Criterion `generate_waveform_3min_default` `51.516 ms` - `51.693 ms`; Criterion `analyze_audio_30s_no_silence_trim` `240.93 ms` - `243.00 ms`. |
| Q-005 | Performance | P2 | Open | HPSS decomposition clones full spectrograms during iterative refinement. It is gated, but expensive when enabled. | `src/features/onset/hpss.rs:107-115` clones harmonic and percussive matrices each iteration. | Profile HPSS-enabled paths and consider convergence checks or in-place/ref-buffer strategy. | HPSS benchmarks and validation on tracks where HPSS changes BPM/key. |
| Q-006 | Product Readiness / Waveform | P1 | Verified | Library did not expose a clearly documented production waveform contract for Sustain. | `src/waveform.rs` now exposes an additive `generate_waveform` API with `WaveformConfig`, multi-resolution `WaveformLevel`s, and signed min/max/RMS/peak `WaveformPoint`s. `src/lib.rs` re-exports the API without changing `analyze_audio`. `docs/WAVEFORM.md` defines the v1 contract, including mono sample-in policy, partial-window behavior, determinism, validation, and serialization shape. | Completed for the v1 backward-compatible waveform contract. Future work remains for split-channel policy if Sustain needs it, external golden waveform fixtures, and high-volume profiling under `Q-004` / `Q-007`. | Passed on 2026-06-03: `cargo test waveform` (`4` tests); `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`236` unit, `2` determinism integration, `2` decoder integration, `5` fixture integration, `47` doctests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`. |
| Q-007 | Integrity / Validation | P1 | In Progress | Accuracy and "best-in-class" claims need reproducible benchmark manifests, not only prose reports or private track references. | README cites real-world DJ validation and FMA results. `validation/benchmarks/README.md`, `validation/benchmarks/corpora.md`, and `validation/benchmarks/manifest_template.md` now define source-level corpus policy, researched candidate corpora, and required manifest fields. | Convert each selected corpus into a file-level manifest with checksums and adapters, starting with FMA/FMAK and GiantSteps Tempo/Key. Keep private Sustain tracks manifest-only. | Validation command, dataset manifest, checksums, metrics, and caveats recorded for each result. |
| Q-008 | Reliability / Decoder Examples | P1 | Verified | Example decoders duplicated logic, used `symphonia` 0.5 EOS/error semantics, and allocated per packet. | `examples/common/mod.rs` now contains the shared `symphonia` 0.6 decoder; `examples/analyze_file.rs` and `examples/analyze_batch.rs` call the helper. `tests/example_decoder_tests.rs` compares decoded WAV output against `hound` on all tracked fixtures and checks non-audio input returns an error. | No decoder follow-up for this migration. Separate output determinism issue is tracked as `Q-010`. | Passed on 2026-06-03: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`223` unit, `2` decoder integration, `5` fixture integration, `46` doctests); `cargo test --test integration_tests` (`5` tests); `cargo test --test example_decoder_tests` (`2` tests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`; `cargo tree -e normal -i symphonia` showed no normal graph path; `cargo tree --edges dev -i symphonia` showed `symphonia v0.6.0` only under `[dev-dependencies]`; fixture smoke: `analyze_file --json` on `120bpm_4bar.wav` and `cmajor_scale.wav`, plus `analyze_batch --jobs 2 --json` on all four WAV fixtures, all decoded and analyzed successfully. |
| Q-009 | Integrity / Algorithm Governance | P1 | Open | BPM/key selection contains many heuristic knobs and trap-zone corrections; risk of validation-set overfitting remains. | `src/config.rs` exposes a large tuning surface; README and PLAN note validation-tuned behavior. | Freeze current behavior with benchmark manifests, then change one algorithmic concern per PR with before/after metrics. | Benchmark matrix showing improved or neutral performance across datasets, not only one private corpus. |
| Q-010 | Integrity / Determinism | P1 | Verified | Low-confidence key outputs were not stable enough for production semantics. Repeated fixture smoke runs and single-file vs batch CLIs produced different key labels with `key_confidence` reported as `0.0000`, despite identical audio fixtures and default config. | Key score sorting now uses deterministic total-order tie handling in `src/features/key/mod.rs`; detector, ensemble, median, segment-voting, and key-change aggregation paths use stable key tie-breaks instead of score-only ordering or `HashMap` iteration order. `tests/determinism_tests.rs` covers repeated and parallel `analyze_audio` key stability on fixtures. | Completed without public API changes. Future uncertain-key semantics remain a product/API design question and should be handled as a backward-compatible additive feature. | Passed on 2026-06-03: `cargo test --test determinism_tests` (`2` tests); `cargo clippy --all-targets --all-features -- -D warnings`; release smoke repeated `analyze_file --json` on `120bpm_4bar.wav` and `cmajor_scale.wav` and compared with `analyze_batch --jobs 2 --json` over all fixtures, with matching keys for checked files; `cargo test --all-features` (`227` unit, `2` determinism integration, `2` decoder integration, `5` fixture integration, `46` doctests); `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`. |
| Q-011 | Reliability / Integrity | P1 | Verified | The public input contract lacked an explicit finite-amplitude and configuration validation policy. The docs say samples are normalized to `[-1.0, 1.0]`, but the implementation only enforced non-empty, non-zero sample rate, and finite sample values. Many `AnalysisConfig` fields are floats that could be set to NaN or nonsensical values by callers. | `AnalysisConfig::validate()` now rejects non-finite public float fields, non-finite vector/array config values, zero frame/hop sizes, invalid BPM range/resolution, invalid onset percentile, and invalid pitch mapping basics before DSP work. `analyze_audio` calls it after input validation. It intentionally does not reject loud finite samples, preserving backward compatibility while leaving amplitude policy as a product contract. | Completed for compatibility-safe config validation. Additive documentation/API work can still define recommended finite amplitude/headroom policy later. | Passed on 2026-06-03: `cargo test default_config_is_valid`; `cargo test config_rejects`; `cargo test analyze_audio_rejects_invalid_config_before_processing`; `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`232` unit, `2` determinism integration, `2` decoder integration, `5` fixture integration, `46` doctests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`. |
| Q-012 | Integrity / Literature | P1 | In Progress | Existing algorithm literature review must be triple-checked and extended where current BPM, key, beat-grid, and waveform research can improve quality. Prior work exists under `docs/literature/`, but best-in-class claims need a refreshed literature pass before deeper DSP changes. | `docs/literature/REFRESH_AUDIT_2026.md` now records a current audit matrix covering mir_eval, JAMS, mirdata, Beat This, FMAK, GiantSteps, Freesound Loop, Ballroom, Harmonix, Gomez/HPCP, Faraldo EDM key estimation, and waveform envelope references. | Promote accepted candidates into paper-level literature notes and convert concrete implementation implications into focused backlog items before algorithmic code changes. | Literature audit document with source links/DOIs/access dates, algorithm relevance, licensing/implementation notes, and resulting backlog items. |
| Q-013 | Integrity / Evaluation | P1 | In Progress | Validation tooling does not yet expose a standardized MIR evaluation layer for JAMS/mir_eval-compatible beat metrics, MIREX-style key categories, corpus-specific annotation adapters, or uncertainty reporting. | `validation/_metrics.py` now centralizes dependency-free BPM error/ratio-bucket helpers, MIREX-style key scoring, and mir_eval-compatible beat/downbeat precision/recall/F-measure helpers with the default `0.07` second tolerance and optional early-event trimming. `run_validation.py` emits additive `bpm_ratio`, `bpm_ratio_bucket`, `key_mirex_category`, and `key_mirex_score` columns while preserving existing CSV fields. Analysis scripts can compute the same key/ratio metrics from old CSVs. | Continue with corpus manifest ingestion, JAMS adapters where datasets provide JAMS, wiring beat/downbeat metrics into manifest-backed runs, uncertainty reporting, and dry-runs on prepared external corpora. | Partial verification passed on 2026-06-03: `python3 -m unittest tests/test_validation_metrics.py` (`9` tests); `python3 -m compileall validation tests/test_validation_metrics.py`; `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-features` (`238` unit, `2` determinism integration, `2` decoder integration, `5` fixture integration, `47` doctests); `cargo build --release --all-features`; `cargo build --release --example analyze_file`; `cargo build --release --example analyze_batch`. |

## Progress Log

### 2026-06-03

- Verified `Q-001`: `analyze_audio` now rejects non-finite samples before
  preprocessing or DSP work. Remaining finite-amplitude/config validation policy
  is split into `Q-011`.
- Recorded `Q-010`: fixture smoke testing exposed unstable low-confidence key
  labels across single-file and batch CLI runs. This is separate from decoder
  correctness and must be fixed under determinism/product semantics.
- Verified `Q-010`: key-score tie handling now uses deterministic ordering and
  repeated/parallel fixture regressions pass without public API changes.
- Recorded `Q-012`: refresh and extend the scientific literature review before
  deeper BPM/key/beat-grid/waveform algorithm changes.
- Started `Q-007`: added `validation/benchmarks/` with corpus policy,
  researched candidate corpora, and a file-level manifest template.
- Started `Q-012`: added the 2026 literature refresh audit with current source
  links and implementation implications.
- Recorded `Q-013`: standard MIR evaluation and annotation ingestion need their
  own additive validation layer before stronger benchmark claims.
- Started `Q-013`: added shared BPM ratio-bucket and MIREX-style key scoring
  helpers, additive validation CSV columns, analysis-script summaries, and
  focused Python unit tests.
- Continued `Q-013`: added dependency-free beat/downbeat F-measure helpers with
  mir_eval-compatible default tolerance and optional early-event trimming.
- Verified `Q-011`: public config validation now rejects non-finite and
  structurally invalid config values without changing public API shape or
  rejecting loud finite samples.
- Verified `Q-003`: the unimplemented ML feature is now dependency-free, the
  former implicit `ort` feature is preserved as a no-op compatibility alias,
  and ORT/network/TLS crates no longer appear under `--all-features`.
- Verified `Q-006`: added the backward-compatible v1 waveform API and contract
  with deterministic multi-resolution min/max/RMS/peak envelopes and known
  signal tests. Remaining Sustain waveform work is split into external golden
  fixtures, split-channel policy, and high-volume profiling.
- Started `Q-004`: `compute_stft` now reuses its FFT input buffer and returns
  errors for direct zero-size public calls; direct STFT and waveform Criterion
  benchmarks plus `docs/PERFORMANCE_AUDIT_2026.md` record local baselines.
- Continued `Q-004`: no-trim `analyze_audio` now borrows the normalized sample
  buffer instead of cloning it, with a focused no-trim Criterion baseline.
- Verified `Q-008`: example CLIs now use a shared `symphonia` 0.6 decoder,
  release example builds pass, decoded WAV samples match the `hound` fixture
  loader, invalid non-audio input returns an error, and all four synthetic WAV
  fixtures smoke-run through `analyze_file` / `analyze_batch`.
- Started `Q-008`: migrating example-only decoding to `symphonia` 0.6 after
  the verified dev-dependency demotion.
- Verified `Q-002`: `symphonia` is no longer in the normal dependency graph and
  remains available only for examples/dev targets. Full documented gate passed.
- Started `Q-002`: reclassified `symphonia` as a dev-dependency so the
  sample-based library no longer forces audio decoder dependencies on consumers.
- Created the quality audit/backlog tracking system.
- Baseline checks passed: `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --all-features`.
- Initial P0/P1 findings recorded for non-finite input hardening, dependency
  hygiene, benchmark manifests, and waveform product contract.
