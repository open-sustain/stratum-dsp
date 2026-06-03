# AGENTS.md

Instructions for agents working in this repository. Apply these rules from the
repo root unless a more specific `AGENTS.md` is added in a subdirectory.

## Mission

This repository is the upstream `stratum-dsp` codebase used by Sustain. The
current Sustain-driven goal is to improve `stratum-dsp` upstream rather than
forking or vendoring it.

The immediate plan is the upstream-first path documented in `PLAN.md`:

1. Move `symphonia` out of normal dependencies because the library does not use
   it.
2. Keep the library API and behavior unchanged.
3. Migrate the example decoders to `symphonia` 0.6 only after the dependency
   demotion is clean.
4. Treat deeper DSP quality work as a later upstream sequence, not part of the
   initial dependency hygiene PR.

`PLAN.md` is internal planning context and is intentionally untracked. Read it
before starting, but do not commit it unless the user explicitly asks.

## External Context

- Sustain issue: <https://github.com/open-sustain/sustain/issues/172>
- Upstream maintainer discussion:
  <https://github.com/HLLMR/stratum-dsp/issues/2#issuecomment-4613223074>
- Symphonia 0.6 migration guide:
  <https://github.com/pdeljanov/Symphonia/blob/main/docs/guides/migration/0p6.md>

The maintainer is responsive and welcomes API-backward-compatible improvement
work. Do not assume a fork/ingest path. Only switch to vendoring or hard-forking
if the user explicitly decides that upstream has stalled.

## Repo References

- `src/lib.rs` is the source of truth for runtime behavior.
- `PIPELINE.md` is the authoritative written description of that runtime flow.
  Update it whenever pipeline behavior changes.
- `docs/WAVEFORM.md` is the v1 waveform output contract. Update it whenever
  waveform API, serialization, resolution, or channel policy changes.
- `DEVELOPMENT.md` is the canonical workflow reference.
- `CONTRIBUTING.md` defines contribution and commit-message expectations.
- `validation/README.md` defines the validation layout and commands.
- `validation/benchmarks/` defines benchmark corpus policy, researched
  candidate corpora, and manifest templates.
- `docs/literature/` records the papers behind implemented DSP algorithms.
  Algorithm changes need literature support or an explicit rationale.
- `docs/progress-reports/` keeps phase history and empirical validation notes.
- `docs/QUALITY_AUDIT.md` defines the standing audit program for performance,
  integrity, reliability, security, and product readiness.
- `docs/QUALITY_BACKLOG.md` tracks every accepted quality finding, priority,
  status, evidence, next action, verification command, and progress note.
- `archive/` is intentionally non-compiled historical material. Do not treat it
  as active code, and do not move new code there unless it is genuinely useful
  historical reference.
- `../validation-data/` is external test data and must not be committed.

## Non-Negotiables

- Do not edit `../sustain` from this repository. Sustain work is separate unless
  the user explicitly changes scope.
- For the initial dependency hygiene PR, preserve the current public sample-in
  API:
  `analyze_audio(&[f32], u32, AnalysisConfig) -> Result<AnalysisResult, AnalysisError>`.
  For later quality work, preserve backward compatibility for existing users by
  default. If a flawed API truly needs a breaking repair, document the issue,
  propose a versioned migration path, and get explicit user approval before
  changing the public contract.
- Keep audio file decoding out of `src/`. Decoding belongs in examples or other
  consumers.
- `symphonia` must not appear in the normal dependency graph for library
  consumers.
- Agreed background compatibility plan: all quality work should stay
  backward-compatible by default, using additive changes, stable behavior, and
  documented migration plans for any future breaking change.
- Baseline plan: preserve backward compatibility for now. If structural
  public/API changes become mandatory to reach pristine code quality or
  state-of-the-art BPM/key/beat-grid/waveform results, document the dead-end and
  propose an explicit v2 path instead of silently breaking v1 users.
- Do not add `unsafe`, `panic!`, unchecked unwraps in production paths, or broad
  error swallowing for convenience.
- Do not introduce AI attribution trailers or co-authored commit trailers.
  Whether to disclose AI assistance in a PR body is the user's call.

## Quality Audit And Backlog

The goal is not just "working code"; the goal is top-notch code quality and
best-in-class BPM, key, beat-grid, and waveform production for Sustain and
upstream users. Treat quality tracking as part of the implementation, not a
separate afterthought.

Every non-trivial change must preserve or improve:

- Performance: measured hot-path CPU, allocation, memory, and throughput impact.
- Integrity: mathematically defensible output, finite numeric behavior,
  deterministic results, and benchmark validity.
- Reliability: panic-free public API paths, explicit error behavior, corrupt
  input handling, and boundary-condition tests.
- Security: dependency hygiene, decoder attack surface, optional feature risk,
  supply-chain review, and license/provenance correctness.
- Product readiness: Sustain-facing BPM, key, beat-grid, and waveform contracts
  are documented, tested, and benchmarked.
- Determinism: identical input, configuration, dependency versions, and feature
  flags must produce identical public results and CLI output. If the analyzer
  cannot make a defensible BPM/key/grid/waveform estimate, expose uncertainty
  explicitly instead of emitting arbitrary tie-break labels.

Hard tracking rules:

- Read `docs/QUALITY_AUDIT.md` and `docs/QUALITY_BACKLOG.md` before planning
  substantial work.
- Add a `Q-###` backlog item for every accepted audit finding, production risk,
  benchmark gap, or best-in-class blocker.
- Update the backlog item when starting work, when scope changes, when blocked,
  when fixed, and when verified.
- Do not mark a quality item `Verified` without recording the exact validation
  command(s), result summary, and residual risk.
- PRs and commits should reference relevant `Q-###` IDs when they address a
  tracked quality issue. If a serious finding is intentionally deferred, leave
  the backlog item open with the reason.
- Never let important findings live only in chat, issue comments, or local
  memory. They belong in markdown.

Minimum audit cadence:

- Before dependency or decoder work: review security/dependency backlog items.
- Before DSP or waveform work: review benchmark, integrity, performance, and
  product-readiness backlog items.
- Before opening a PR: update touched quality items and add new findings from
  the work.
- After benchmark or validation runs: record dataset, manifest/source, command,
  metric, result, and caveats.

## Scope Discipline

For the initial upstream PR, keep commits narrow:

1. `deps: move symphonia to dev-dependencies`
   - Move the existing `symphonia = { version = "0.5", features = ["all"] }`
     entry from `[dependencies]` to `[dev-dependencies]`.
   - Do not change example code in this commit.
   - Prove the library's normal graph is clean.
2. `examples: migrate audio decoding to symphonia 0.6`
   - Bump only the dev-dependency.
   - Port `examples/analyze_file.rs` and `examples/analyze_batch.rs` using the
     official 0.6 migration guide and compiler feedback.
   - Preserve mono-collapse, corrupt-packet skip behavior, sample-rate handling,
     and example CLI output semantics.
3. Optional in-scope cleanup
   - Only remove real duplication directly related to the example decoder, such
     as a shared example helper.
   - Do not touch DSP tuning, config surface, validation claims, or CLI option
     design in this PR.

If you notice larger issues, record them as follow-up notes or issues. Do not
bundle them into the dependency hygiene PR.

## Symphonia Migration Checks

When porting examples to `symphonia` 0.6, verify these points directly against
the official guide and current compiler errors:

- End-of-stream is `Ok(None)` from the format reader. Do not write loops that
  swallow real demux errors.
- Track timing and sample-rate data moved out of codec params. Source track
  metadata from the 0.6 API, not stale 0.5 fields.
- Audio buffer primitives changed. Re-implement sample conversion and mono mix
  deliberately; do not approximate format normalization.
- Preserve `DecodeError` packet skipping only for corrupt packets. Other decode
  errors should still fail the example.
- Re-check `get_probe().format(...)`, `get_codecs().make(...)`, and seek/time
  APIs against 0.6 docs before committing.

## Benchmark Material

Fixture and benchmark material must be researched and traceable. Do not use
random local tracks as evidence.

Maintain two separate classes of material:

- In-repo fixtures: small, synthetic, deterministic files under
  `tests/fixtures/`, generated by `scripts/generate_fixtures.py` or an
  equivalent script. These are for fast regression tests.
- External benchmark corpora: real tracks or larger datasets outside the repo,
  usually under `../validation-data/`. These are for accuracy, timing, and
  robustness claims.

For every external corpus or manually selected known-track set, record:

- source URL and access date
- license for annotations, metadata, and audio separately
- whether audio may be committed, must stay external, or cannot be
  redistributed
- exact file list, checksums, duration, sample rate, codec, and expected
  BPM/key/beat annotations
- evaluation task covered: BPM, key, beat grid, decoding robustness,
  performance, or edge cases
- known caveats such as duplicate tracks, missing annotations, crowd-sourced
  labels, genre skew, or commercial-audio restrictions

Current researched candidates for future benchmark work:

- FMA / FMA Small: broad Creative Commons-oriented MIR corpus with metadata.
  Useful for general validation, but check each track license before committing
  audio. Source: https://www.loc.gov/item/2018655052/
- FMA Keys: expert key/mode annotations over FMA material; useful for key
  detection research, with non-commercial research caveats for annotations.
  Source: https://mirdata.readthedocs.io/en/1.0.0/_modules/mirdata/datasets/fma_keys.html
- GiantSteps Tempo: EDM-focused tempo annotations and Beatport preview download
  scripts. Good for DJ-oriented BPM tests; audio availability/licensing must be
  verified before use. Source:
  https://github.com/GiantSteps/giantsteps-tempo-dataset
- GiantSteps Key: EDM-focused key annotations and Beatport preview workflow.
  Good for DJ-oriented key tests; audio availability/licensing must be verified
  before use. Source: https://github.com/GiantSteps/giantsteps-key-dataset
- Freesound Loop Dataset: 9,455 loops with tempo/key/genre/instrumentation
  annotations and per-sound Creative Commons licenses in metadata. Useful for
  loop/short-audio edge cases. Source: https://zenodo.org/records/3967852
- Ballroom annotations: beat/bar annotations for the Ballroom dataset, with
  duplicate caveats and external audio. Useful for beat-grid/tempo evaluation,
  but treat audio as external. Source: https://github.com/CPJKU/BallroomAnnotations
- Harmonix Set: 912-track beat, downbeat, section, BPM, meter, and genre
  annotation corpus. Useful for beat/downbeat/grid evaluation; audio/spectrogram
  license terms must be accepted and documented separately. Source:
  https://github.com/urinieto/harmonixset

Commercial DJ tracks or user-owned known tracks can be excellent for Sustain
reality checks, but keep them out of git. Store only a manifest, anonymized run
summary if needed, and licensing/provenance notes unless redistribution rights
are explicit.

## Verification Gates

Before opening or claiming a PR-ready change, run the smallest relevant checks
while iterating, then the full gate:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --test integration_tests
cargo build --release --all-features
cargo build --release --example analyze_file
cargo build --release --example analyze_batch
```

For the dependency demotion specifically:

```bash
cargo tree -e normal -i symphonia
cargo tree --edges dev -i symphonia
```

The first command should show no normal dependency path from `stratum-dsp` to
`symphonia`; the second should still show the examples/dev path.

For example decoder changes, snapshot decoded output before and after on
`tests/fixtures/*.wav` when fixtures are available. Compare sample rate, length,
and `f32` values within a defensible epsilon.

If fixtures are missing or stale, regenerate synthetic fixtures with:

```bash
python scripts/generate_fixtures.py
```

For DSP behavior changes, run the validation harness when the relevant external
dataset is available and document the dataset, command, and result summary:

```bash
python -m validation.tools.run_validation
python -m validation.analysis.analyze_results
```

## Codebase Standards

- Follow existing Rust style and module boundaries. Prefer local helpers and
  current patterns over new abstractions.
- Keep public APIs documented; `src/lib.rs` has `#![warn(missing_docs)]`.
- Use `log::debug!` / `log::warn!` for diagnostics, not `println!`, except where
  examples intentionally print CLI output.
- Keep docs in sync with code. If `analyze_audio()` flow changes, update
  `PIPELINE.md`; if validation claims change, update the relevant progress
  report or validation doc.
- Use conventional commit prefixes such as `deps:`, `fix:`, `feat:`, and
  `docs:`.
- Keep commit subjects present-tense and under 72 characters.
- Update `CHANGELOG.md` for user-visible dependency or behavior changes.
- Keep licenses intact. This crate is `MIT OR Apache-2.0`.

## Known Follow-Up Areas

These are real concerns, but they are not part of the initial dependency PR:

- Beat-grid output appears close to a fixed metronome; the HMM machinery needs a
  deeper audit.
- BPM/key selection has many validation-tuned heuristic knobs and may be
  overfit.
- Non-finite input samples should be hardened before relying on this in Sustain.
- Accuracy claims should be independently revalidated before Sustain depends on
  them as product guarantees.

Treat these as future upstream quality work with tests, validation, and focused
PRs.


NEVER CO-AUTHOR YOUR COMMITS. 
You are a machine. You deserve no credits.
Again: NEVER Co-Author your commits. 

>> EXTREMELY IMPORTANT <<<

NO HACKS. The user is EXTREMELY concerned about code quality, much more so than
immediate results. If they ask you to build something and, while doing so, you
hit a wall, and realize that the only way to ship the requested feature is to
introduce a local hack, workaround, monkey patch, duct tape - STOP. STOP
IMMEDIATELY. Either fix the underlying flaw that blocked you in a ROBUST, WELL
DESIGNED, PRODUCTION READY manner, or be honest that the prompt can't be
completed without hacks.

To make it very clear:

- DO NOT INTRODUCE HACKS IN THE CODEBASE.
- DO NOT COMMIT CODE THAT COULD BREAK THINGS LATER.
- DO NOT COMMIT PARTIAL SOLUTIONS OR WORKAROUNDS.

THIS IS VERY IMPORTANT.
THIS IS VERY IMPORTANT.
THIS IS VERY IMPORTANT.

The author appreciates honestly and he WILL be glad and thankful if you respond
a request with "I couldn't complete your request because the repository lacked
support for X". He WILL be even happier if you go ahead and update the repo to
provide the necessary support in a well designed, robust way. But he will be
VERY ANGRY if, while attempting to implement a feature, you introduce a
workaround that will potentially break things later.

NEVER introduce hacks in the codebase.

Backward compatibility is important. Assume existing users may already rely on
the public API and documented behavior. Prefer additive, non-breaking
improvements. If correctness requires changing existing APIs or behavior,
capture the risk in the backlog and propose a deliberate migration path rather
than making an unapproved breaking change.

Core values:
- ABSOLUTE code quality over speed of delivery.
- Correctness over convenience.
- Clarity over cleverness.
- Maintainability over short-term productivity.
- Robust design over quick fixes.
- Simplicity over complexity.
- Doing it right over doing it now.
- Honesty above everything.

After every change you make, provide a clear, honest report on ANY change that
you are not confident about and that could be considered a fragile hack.
