# Literature Refresh Audit - 2026-06-03

Purpose: triple-check the existing literature base and identify additional
papers, datasets, and engineering references that can improve Stratum DSP's BPM,
key, beat-grid, waveform, confidence, and evaluation quality.

This is an audit matrix, not an implementation claim. Any algorithmic change
still needs code review, benchmark manifests, and before/after validation.

## Standing Principles

- Keep v1 backward-compatible by default. Prefer additive metrics, optional
  config, new result fields, and new validation tools.
- Treat ML/runtime additions as Phase 2 or v2 candidates unless they can be
  introduced without forcing heavy dependencies on existing users.
- Do not tune against one corpus. A change must be neutral or positive across a
  documented benchmark matrix, or it must be scoped to an opt-in mode.
- Record uncertainty explicitly. Arbitrary labels for ambiguous BPM/key/grid
  estimates are product risk.

## New Or Rechecked Sources

| Area | Source | Why It Matters | Implementation Implication |
| --- | --- | --- | --- |
| Evaluation | `mir_eval`: Raffel et al., "MIR_EVAL: A Transparent Implementation of Common MIR Metrics", ISMIR 2014. Source: <https://ir.webis.de/anthology/2014.ismir_conference-2014.60/> | Gives established evaluation metrics for MIR tasks instead of ad hoc result summaries. | Add a validation-methodology layer that reports standard BPM/key/beat metrics where possible and keeps Stratum-specific metrics separate. |
| Annotation format | JAMS: "A JSON Annotated Music Specification for Reproducible MIR Research". Source: <https://jams.readthedocs.io/en/stable/> | Standard schema for beats, chords, keys, tempo, metadata, and mir_eval interop. | Prefer JAMS ingestion/export for corpora that already provide it, especially GiantSteps, Harmonix, and Beat This materials. |
| Dataset reproducibility | `mirdata`: Bittner et al., "mirdata: Software for Reproducible Usage of Datasets", ISMIR 2019. Source: <https://github.com/mir-dataset-loaders/mirdata> | Provides dataset download/validation/load patterns and explicit dataset citation/version handling. | Use mirdata conventions as the model for Stratum corpus manifests even if the Rust repo does not depend on Python. |
| Beat/downbeat SOTA | Foscarin, Schlueter, Widmer, "Beat this! Accurate beat tracking without DBN postprocessing", ISMIR 2024. Source: <https://arxiv.org/abs/2407.21658>, code: <https://github.com/CPJKU/beat_this> | Modern beat/downbeat system with transformer/convolution architecture, multi-dataset training, and no DBN postprocessing. | Treat as Phase 2/v2 reference and as a benchmark comparison target. Do not pull ML runtime into v1 without explicit policy. |
| Beat/downbeat data | Beat This spectrogram/annotation collection. Source: <https://zenodo.org/records/13922116> | Aggregates mel spectrograms and annotations from 16 beat/downbeat datasets used in current research, including datasets where audio is not public. | Use for literature comparison and possible ML experiments; not a replacement for sample-in audio validation. |
| Key/chroma foundation | Gomez, "Tonal Description of Polyphonic Audio for Music Content Processing", INFORMS Journal on Computing, 2006. Source: <https://pubsonline.informs.org/doi/10.1287/ijoc.1040.0126> | Canonical tonal-description/HPCP work for key estimation and tonal similarity. | Audit current HPCP-style options against spectral peak selection, tuning, harmonic weighting, whitening, and temporal aggregation. |
| HPCP implementation reference | MTG HPCP Vamp plug-in documentation. Source: <https://www.upf.edu/web/mtg/hpcp> | Practical HPCP parameterization: bins per octave, frequency limits, reference tuning, peak threshold, harmonics per peak, whitening, non-linear compression, and two-band mode. | Convert these into explicit testable hypotheses for key detection rather than more opaque tuning knobs. |
| EDM key estimation | Faraldo, Jorda, Herrera, "A Multi-Profile Method for Key Estimation in EDM", Semantic Audio 2017. Source: <https://zenodo.org/records/3855499> | Directly targets DJ/EDM key detection, difficult minor tracks, amodal tracks, and multi-profile scoring. | High-priority key roadmap item for GiantSteps/Sustain validation. Preserve default behavior until benchmarked. |
| Key dataset | FMAK: Wong and Hernandez, "FMAK: A Dataset of Key and Mode Annotations for the Free Music Archive", ISMIR 2023 LBD / Zenodo 2024. Source: <https://zenodo.org/records/10719860> | Expert-labeled key/mode annotations for 5,489 FMA songs across 17 genres. | Add FMAK adapter before making stronger key claims beyond private datasets. |
| DJ/EDM datasets | Knees et al., "Two data sets for tempo estimation and key detection in electronic dance music annotated from user corrections", ISMIR 2015. Sources: <https://github.com/GiantSteps/giantsteps-tempo-dataset>, <https://github.com/GiantSteps/giantsteps-key-dataset> | Direct EDM/DJ benchmark material for tempo and key. | Add GiantSteps Tempo/Key adapters and keep audio licensing external. |
| Beat/grid datasets | Ballroom Annotations and Harmonix Set. Sources: <https://github.com/CPJKU/BallroomAnnotations>, <https://github.com/urinieto/harmonixset> | Beat, bar, downbeat, BPM, meter, genre, and segment annotations with known caveats. | Required before any beat-grid quality claim beyond synthetic fixtures. |
| Loop/short audio | Ramires et al., "The Freesound Loop Dataset and Annotation Tool", ISMIR 2020. Source: <https://zenodo.org/records/3967852> | 9,455 loops with tempo, key, genre, and instrumentation annotations. | Add short-loop and edge-case validation; score cautiously because loops differ from full tracks. |
| Waveform contract | BBC `audiowaveform`. Source: <https://github.com/bbc/audiowaveform> | Production precedent for min/max peak data, zoom levels, JSON/binary formats, mono vs split-channel output. | Use as an engineering baseline for `Q-006`; do not copy GPL code. |
| Waveform envelope | MathWorks `audioEnvelope`. Source: <https://www.mathworks.com/help/audio/ref/audioenvelope.html> | Documents min/max envelope over non-overlapping frames and frame locations as a common waveform abstraction. | Define Stratum waveform output as deterministic min/max plus optional RMS/peak metadata and multi-resolution levels. |

## Gaps Found

- The literature directory lacks a current audit index that distinguishes
  implemented foundations from candidates requiring validation.
- Current validation tooling is FMA-style and result-CSV oriented. It does not
  yet expose a standard metric layer for JAMS, mir_eval-compatible beat metrics,
  MIREX-style key categories, or corpus-specific caveats.
- Key detection has relevant EDM-specific and HPCP literature that should be
  audited against the current code before further tuning.
- Beat-grid validation needs real beat/downbeat corpora before production
  claims. HMM implementation tests alone are not enough.
- Waveform work needs a product contract first: min/max envelope, RMS envelope,
  channel policy, multi-resolution storage, normalization, serialization, and
  golden fixtures.
- ML-based SOTA beat/tempo systems are relevant but conflict with the current
  dependency hygiene goal if introduced casually. They belong behind explicit
  Phase 2/v2 gates.

## Immediate Backlog Implications

- `Q-007`: create corpus manifests and adapters for FMA/FMAK, GiantSteps,
  Freesound Loop, Ballroom, Harmonix, and private Sustain known tracks.
- `Q-012`: continue this literature refresh with paper-level notes for accepted
  candidates before algorithmic code changes.
- `Q-013`: standardize metrics and annotation ingestion around
  JAMS/mir_eval/MIREX conventions.
- `Q-006`: define waveform output as an additive public API with deterministic
  fixtures before optimizing rendering or storage.

## Proposed Near-Term Reading Order

1. `mir_eval`, JAMS, and mirdata, because they define how evidence should be
   measured and reproduced.
2. FMAK and GiantSteps papers/loaders, because they unlock immediate key/BPM
   validation material.
3. Gomez/HPCP and Faraldo EDM key estimation, because they are likely to affect
   key quality without requiring an ML runtime.
4. Ballroom/Harmonix/Beat This materials, because beat-grid work needs real
   beat/downbeat metrics before code changes.
5. Waveform production references, because Sustain needs a stable output
   contract before implementation.
