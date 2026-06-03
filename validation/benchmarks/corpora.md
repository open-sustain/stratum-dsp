# Benchmark Corpora

Initial candidate registry for best-in-class BPM, key, beat-grid, waveform, and
robustness validation. This is not a validation result. A corpus is only usable
for claims after a file-level manifest and validation run are recorded.

Access date: 2026-06-03.

## Corpus Matrix

| Corpus | Primary Tasks | Source | License / Redistribution | Status | Caveats / Next Action |
| --- | --- | --- | --- | --- | --- |
| FMA / FMA Small | BPM sanity, genre breadth, performance, decoder robustness | <https://github.com/mdeff/fma>, <https://www.loc.gov/item/2018655052/> | Metadata is CC BY 4.0; code is MIT; audio follows each artist's chosen license and must be checked per track. | Candidate; existing validation scripts already support FMA-style metadata. | Existing FMA Small tempo labels come from Echonest metadata and are useful for development/tuning, not enough alone for DJ-grade claims. Build file-level manifest and separate tuning from final validation. |
| FMAK / FMA Keys | Key and mode validation over FMA material | <https://zenodo.org/records/10719860>, <https://mirdata.readthedocs.io/en/stable/_modules/mirdata/datasets/fma_keys.html> | Zenodo record and mirdata loader list CC BY 4.0 for annotations/metadata; audio is FMA audio with per-track licenses. | Candidate; high priority for key validation. | Expert-labeled song-level key/mode annotations for 5,489 songs. Need adapter from FMAK metadata to Stratum validation CSV and per-track audio license checks. |
| GiantSteps Tempo | DJ/EDM BPM validation | <https://github.com/GiantSteps/giantsteps-tempo-dataset> | Annotations are in the repo; audio previews are Beatport/lofi previews and must stay external unless rights are verified. | Candidate; high priority. | Contains 664 mostly two-minute EDM previews and tempo/genre annotations, including v2 annotations. Audio download links and availability must be rechecked before use. |
| GiantSteps Key | DJ/EDM key validation | <https://github.com/GiantSteps/giantsteps-key-dataset>, <https://mirdata.readthedocs.io/en/stable/_modules/mirdata/datasets/giantsteps_key.html> | mirdata records CC BY-SA 4.0 for dataset data; original audio snippets are Beatport previews. | Candidate; high priority. | Strong fit for Sustain DJ workflows. Need decide whether to use GitHub, Zenodo, or mirdata as the canonical retrieval path and record checksums. |
| Freesound Loop Dataset | Short-loop BPM, key, decoder robustness, edge cases | <https://zenodo.org/records/3967852> | Sounds have Creative Commons licenses listed in `FSL10K/metadata.json`; annotations are distributed by Zenodo. | Candidate. | 9,455 loops with tempo, key, genre, and instrumentation annotations. Good for short material, but user/tag-derived BPM and loop-specific labels need caveat-aware scoring. |
| Ballroom Annotations | Beat, bar, downbeat, tempo/grid | <https://github.com/CPJKU/BallroomAnnotations> | Annotation repo does not include audio; audio archive is external. | Candidate. | Beat/bar annotations for 698 Ballroom files, with documented duplicate and replica caveats. Need pin repository tag and archive checksum. |
| Harmonix Set | Beats, downbeats, sections, BPM, meter, genre | <https://github.com/urinieto/harmonixset> | Annotation repo is MIT; mel spectrogram use requires accepting the included license; audio access is indirect/external. | Candidate. | 912 Western pop tracks with beat/downbeat/segment annotations and JAMS files. Useful for beat-grid validation but audio alignment/provenance must be documented. |
| Beat This Spectrograms / Annotations | Beat/downbeat benchmarking and state-of-art comparison | <https://zenodo.org/records/13922116>, <https://github.com/CPJKU/beat_this> | Spectrogram/annotation package is on Zenodo; audio is unavailable for many included datasets. | Candidate for research, not direct audio validation. | Contains mel spectrograms and annotations for 16 beat/downbeat datasets used by the ISMIR 2024 Beat This system. Useful for literature comparison and possible ML/v2 exploration, but not a direct replacement for sample-in audio validation. |
| Sustain/private known tracks | Real-world DJ regression and product sanity | Local user-owned library only | Do not commit audio. Store anonymized manifest and aggregate metrics only unless rights are explicit. | Candidate; requires user-provided local data. | Best reality check for Sustain, but must be separated from public benchmark claims and protected from accidental redistribution. |

## Immediate Adoption Order

1. Convert current FMA/FMA Small validation setup into a file-level manifest
   with checksums, source archives, filters, and result provenance.
2. Add FMAK key metadata ingestion so key validation no longer depends on weak
   tag fallback when expert annotations are available.
3. Add GiantSteps Tempo and GiantSteps Key adapters for DJ/EDM validation.
4. Add beat-grid validation scaffolding for Ballroom and Harmonix before
   claiming beat-grid quality beyond synthetic fixtures.
5. Add Freesound Loop Dataset as a short-audio/loop robustness suite.
6. Add a private known-track manifest template for Sustain-owned tracks.

## Evaluation Policy

- BPM headline metric: accuracy within `+/- 2 BPM`, plus mean absolute error and
  metrical-level confusion buckets.
- Key headline metrics: exact match plus MIREX-style related-key categories
  where annotation format supports it.
- Beat-grid metrics: beat F-measure, downbeat F-measure, continuity-aware
  metrics when available, and explicit metrical-level caveats.
- Waveform metrics: deterministic min/max envelope, RMS envelope, peak
  preservation, multi-resolution consistency, serialization stability, and
  fixture-level golden outputs.
- Performance metrics: wall-clock throughput, allocations where measurable,
  input duration, codec, sample rate, channel count, CPU model, feature flags,
  build profile, and command.
