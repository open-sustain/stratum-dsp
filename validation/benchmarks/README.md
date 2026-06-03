# Benchmark Registry

This directory is the canonical registry for benchmark material used to support
Stratum DSP quality claims. It exists so BPM, key, beat-grid, waveform, decoder,
and performance results are reproducible instead of relying on private memory or
untracked local tracks.

Access date for the initial registry: 2026-06-03.

## Rules

- Keep benchmark audio outside git unless redistribution rights are explicit.
- Store external corpora under `../validation-data/` or another documented
  external path.
- Record source, version, license, local path, checksums, file list, and
  annotation policy before using any corpus result in docs, issues, or PRs.
- Separate annotation licensing from audio licensing. They are often different.
- Do not mix development/tuning results with final validation claims.
- Keep Sustain/private known tracks out of git. Store only an anonymized
  manifest and aggregate results unless redistribution rights are explicit.

## Required Manifest Fields

Every prepared corpus must have a manifest in markdown or CSV with:

- `corpus_id`: stable short name, for example `fma_small` or
  `giantsteps_tempo`.
- `source_url`: canonical source URL.
- `accessed_at`: date the source was checked.
- `version_or_commit`: release, DOI, Git commit, tag, or archive checksum.
- `local_path`: path outside git, usually under `../validation-data/`.
- `audio_license`: license/provenance for audio files.
- `annotation_license`: license/provenance for BPM/key/beat/grid annotations.
- `redistribution`: whether audio, annotations, and metadata may be committed.
- `file_count`: number of usable tracks after filters.
- `file_list`: exact files included.
- `checksums`: checksum algorithm and digest for each file or source archive.
- `codec_sample_rate_channels`: observed audio format details.
- `tasks`: BPM, key, beat grid, waveform, decoder robustness, performance, or
  edge cases.
- `metrics`: exact evaluation metrics and tolerances.
- `known_caveats`: duplicates, weak labels, genre skew, missing annotations,
  commercial-audio restrictions, or metrical-level ambiguity.
- `prepared_by` and `prepared_at`: who prepared the local copy and when.

## Minimum Validation Tiers

- Tier 0: in-repo synthetic fixtures for deterministic fast regression tests.
- Tier 1: broad open corpus sanity checks, currently FMA/FMA Small and FMAK.
- Tier 2: DJ/EDM BPM and key validation, currently GiantSteps Tempo and
  GiantSteps Key.
- Tier 3: beat/downbeat/grid validation, currently Ballroom, Harmonix Set, and
  Beat This annotation material.
- Tier 4: loop and short-audio edge cases, currently Freesound Loop Dataset.
- Tier 5: Sustain/private known tracks, manifest-only and never committed as
  audio unless rights are explicit.

See `corpora.md` for the initial researched corpus registry.
