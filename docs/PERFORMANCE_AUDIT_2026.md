# Performance Audit - 2026-06-03

This note tracks the current performance pass for `Q-004`. It is intentionally
limited to local, repeatable evidence. External corpus throughput must be added
through `validation/benchmarks/` manifests before using these numbers for
product claims.

## Scope

- STFT allocation behavior in `src/features/chroma/extractor.rs`.
- Full-analysis hot-path risk around cloning/copying in `src/lib.rs`.
- New waveform generation cost for Sustain-facing waveform output.
- Criterion baselines for the new STFT and waveform benchmarks.

## Code Changes

- `compute_stft` now validates zero `frame_size` and zero `hop_size` directly.
  The higher-level chroma API already validated those fields, but `compute_stft`
  is public and should be panic-free for invalid public input.
- `compute_stft` now reuses one FFT input buffer across frames. It still returns
  the same `Vec<Vec<f32>>` magnitude spectrogram shape, so callers and public API
  behavior remain compatible.
- `benches/audio_analysis_bench.rs` now includes direct `stft` and `waveform`
  benchmark groups.
- `analyze_audio` now borrows the normalized sample buffer when silence trimming
  is disabled instead of cloning the whole buffer.
- `benches/audio_analysis_bench.rs` now includes
  `analyze_audio_30s_no_silence_trim` for the no-trim ownership path.

## Local Criterion Baselines

Commands:

```bash
cargo bench --bench audio_analysis_bench compute_stft_30s_2048_512 -- --sample-size 10 --measurement-time 2 --warm-up-time 1
cargo bench --bench audio_analysis_bench generate_waveform -- --sample-size 10 --measurement-time 2 --warm-up-time 1
cargo bench --bench audio_analysis_bench analyze_audio_30s_no_silence_trim -- --sample-size 10 --measurement-time 2 --warm-up-time 1
```

Results on this machine:

| Benchmark | Input | Time |
| --- | --- | --- |
| `stft/compute_stft_30s_2048_512` | 30s synthetic 440 Hz sine, 44.1 kHz, frame 2048, hop 512 | `5.8776 ms` - `6.1480 ms` |
| `waveform/generate_waveform_30s_default` | 30s synthetic 440 Hz sine, default waveform config | `8.5504 ms` - `8.7375 ms` |
| `waveform/generate_waveform_3min_default` | 3 minute synthetic 440 Hz sine, default waveform config | `51.516 ms` - `51.693 ms` |
| `analyze_audio_30s_no_silence_trim` | 30s synthetic 440 Hz sine, default config except `enable_silence_trimming=false` | `240.93 ms` - `243.00 ms` |

Gnuplot was not installed, so Criterion used the plotters backend.

## Residual Risks

- `analyze_audio` still copies the full input into `processed_samples` before
  normalization. Avoiding that copy needs careful ownership/API analysis because
  normalization mutates its input buffer.
- `compute_stft` still allocates one magnitude vector per frame because the
  public return type is `Vec<Vec<f32>>`. Any flatter spectrogram storage would
  need a compatibility-preserving adapter or a v2 API path.
- Multi-resolution tempo paths still recompute STFTs at multiple hop sizes.
  Shared spectrogram reuse should be benchmarked against accuracy impact before
  changing defaults.
- No allocation profiler has been run yet. Use `heaptrack`, `dhat`, `valgrind
  massif`, or an equivalent allocator profiler before marking `Q-004` verified.
- Benchmarks are synthetic only. Real-track throughput and outlier analysis must
  use `validation/benchmarks/` manifests.

## Next Steps

- Run allocation profiling on default `analyze_audio` with a representative
  3-minute file and an external manifest-backed batch.
- Benchmark `analyze_audio_30s` before and after any ownership or spectrogram
  reuse changes.
- Keep HPSS-specific work under `Q-005`; it remains opt-in but can dominate
  runtime when enabled.
