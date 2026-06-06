# PLAN — focused upstream work now, minimal Sustain DSP ingest later

> Internal planning doc committed on the Sustain working branch for continuity.
> Do **not** include it in any upstream PR.
> Repo cloned from `https://github.com/HLLMR/stratum-dsp` @ `main`.
> Consumer that motivated this: `../sustain` (do not touch; another agent works there).
> **Superseded as the live tracker by `open-sustain/sustain` issue #192** — this
> doc is decision history; track the ingest there.

---

## DIRECTION DECISION (resolved 2026-06-04)

The approved direction is a deliberate split between generally useful upstream
work and Sustain-owned product architecture.

### Upstream track

- Finish and submit focused changes that bring independent value to
  `stratum-dsp` users, beginning with the narrow `symphonia` dependency-hygiene
  and example-decoder work.
- Submit other universal improvements separately when their scope, evidence,
  and backward compatibility justify an upstream PR.
- Preserve the existing v1 API by default.
- Do **not** add a Sustain-shaped standardized primitives interface or
  capability-driven orchestration API merely to avoid ingestion. That would
  commit upstream to a larger semver surface designed primarily around one
  consumer without being the value proposition of the PR.

### Sustain track

- Later ingest the minimal transitive DSP core Sustain actually needs for BPM,
  key, onsets, loudness, chroma, and STFT-based features.
- Shape the ingested code around Sustain's capability-driven analysis contract
  and deeper DSP rework.
- Keep Sustain-specific decoding, centered-window policy, Smart Shuffle
  features, and Pioneer waveform behavior in Sustain.
- Exclude unused upstream orchestration, beat-grid, waveform, ML, legacy BPM,
  and configuration surfaces unless the measured transitive call graph proves
  they are required.
- Retain upstream license notices and provenance. Treat validation,
  correctness, divergence, and bug ownership as Sustain responsibilities.
- Do not use a long-term git-pinned fork as the destination architecture. It
  carries divergence cost without providing the clean ownership boundary that
  justifies ingestion.

### Before deeper DSP work resumes

1. Finish the narrow upstream dependency-hygiene PR.
2. Write the exact Sustain analysis-output contract.
3. Measure the transitive call graph from the required DSP functions.
4. Establish benchmark baselines and manifests before restructuring or tuning
   algorithms.

This decision explicitly authorizes ingestion even though upstream is active.
It does not authorize doing Sustain work from this repository or bundling
Sustain-specific interfaces into upstream PRs.

---

## Decision Context

**Why this is the priority.** This rework exists to benefit **Sustain**
(`../sustain`), its primary consumer. A cross-repo survey on 2026-06-03 found
that the current orchestration-centric improvements largely do **not** serve
Sustain, because of how Sustain actually integrates.

**What we discovered (Sustain `crates/analysis`, pins `stratum-dsp = "1.0"`):**

- Sustain consumes stratum-dsp as a **low-level primitives toolkit, not via
  `analyze_audio`**. It imports directly: `estimate_bpm_tempogram`, `detect_key`
  + `KeyTemplates`, `chroma::extractor`, `spectral_flux` onsets, and
  `normalization` (ITU-R BS.1770-4 LUFS).
- It does **not** use `analyze_audio`, the beat-grid HMM, or the waveform module
  (it computes waveforms/band features itself; the beat-grid slot is reserved
  `None` and it synthesizes a constant-tempo grid for Rekordbox export).
- Bypassing orchestration costs Sustain **~7% BPM accuracy (~85% vs ~92%)** per
  its own comments — accepted today because `analyze_audio` is all-or-nothing.
- **Smart Shuffle** widens the consumed surface beyond BPM/key: LUFS triplet
  (integrated / short-term max / range), onset rate, low/mid/high band ratios,
  low-band variation, and tonalness — all built on stratum's LUFS / onsets /
  STFT primitives.
- **Coupling risk:** Sustain pins **public-but-low-level module paths** without a
  deliberate stable contract; an upstream rename could break it. Tracked as
  Q-014.

**Options considered for the stratum-dsp ↔ Sustain boundary:**

- **A. Composable full-quality primitives** — documented, semver-stable,
  individually-callable `bpm/key/onsets/loudness/chroma`, each full-quality
  standalone; `analyze_audio` becomes optional. Recovers the 7% and kills the
  coupling.
- **B. Bless + document the exact internals Sustain already imports** — smallest
  surface change; weaker long-term contract.
- **C. Capability-flagged `analyze_audio`** — centralizes quality but reverses
  Sustain's deliberate low-level integration.

**Hard constraints on any choice:**

- **Capability-gated performance is mandatory** — asking for BPM must not pay for
  key/chroma compute. (Favors A; achievable in C only with disciplined gating of
  shared stages such as the key-grade STFT.)
- **No genre profiles.** A `DjEdm`/`General` profile is rejected. The bar is
  genuine whole-tempo-range quality. The only acceptable knob is an honest
  `tempo_range` / octave-anchor search param (e.g. 81–160, a sub-2:1 span that
  removes octave ambiguity), replacing the magic 60–180 prior + the
  "ratio < 2.5" fold heuristic in `tempogram.rs`. The wide-open default must
  still earn whole-range quality (Q-009, benchmark-gated).
- Stay additive / backward-compatible per AGENTS.md; no unapproved public breaks.

**Two separate axes — do not conflate them:**

1. **API shape** — A / B / C above: how the analysis surface is exposed.
2. **Consumption model** — how Sustain obtains the code:
   - *published crate* (upstream PR → crates.io → version bump) — slowest loop,
     least control;
   - *git-pinned dependency* on `open-sustain/stratum-dsp` (pin a commit, carry
     Sustain patches on a branch, upstream selectively) — most control without
     owning the source;
   - *full ingestion / vendoring* into Sustain (Path B below) — max control +
     tailoring freedom, max ownership.

   These compose, e.g. ingest **and** shape as clean composable primitives
   (A-shaped, vendored).

**Updated analysis (2026-06-03) — re-weighting ingestion. This revises Path B's
cost framing below:**

- **Footprint: the ingest size is not the whole crate.** Library is ~20k LOC,
  but Sustain uses none of: `beat_tracking` (2,338), `analysis`/confidence
  (927), most of `lib.rs` orchestration (~1,500), `waveform.rs` (289), `ml`
  (105) — **~5k LOC droppable immediately** — plus the legacy onset-based BPM
  paths in `period`, the non-`spectral_flux` onset detectors, and most of
  `config.rs`'s knobs. A **clean, segmented ingest is realistically ~half the
  crate or less.** (Exact floor = transitive call graph of the five imported
  functions; can be measured.)
- **LLM-era dev cost ≈ free, but that lowers *every* option equally — including
  fixing it cleanly upstream.** So free labor does not by itself favor
  ingestion; it removes implementation effort from the decision and shifts the
  weight onto the costs LLMs do **not** make free: (a) **DSP validation /
  correctness** — proving best-in-class on real annotated corpora
  (Q-007/Q-009/Q-013); more generated code = more regression surface, so this
  becomes *more* necessary, not less; ingestion concentrates 100% of it on
  Sustain, upstreaming shares it with a responsive maintainer + community;
  (b) **divergence** from a live upstream — each future upstream improvement
  becomes a merge-or-forgo judgment; (c) **bug ownership** — wrong outputs
  become Sustain's, in Sustain's tree.
- **Ingestion also removes the v1 backward-compat handcuff.** The "stay
  additive / no public breaks" constraint exists to protect upstream's published
  `analyze_audio` API — which Sustain does not use — and Sustain's own policy is
  "backward-compat not important (pre-release)." Vendored, Sustain owns the
  surface, so a **clean re-architecture and deep DSP rework** (not patching
  inherited heuristics) becomes possible. This is the "opens interesting things"
  angle.

**User's lean (updated 2026-06-03):** moved from "A or C, upstream" toward
**ingestion + deep, clean, segmented rework** — re-architect what we ingest and
improve the DSP, rather than inheriting and going along with the existing
heuristic-heavy design. Still wants whole-range quality, capability-gated
performance, and no genre profiles.

**Mission tension resolved 2026-06-04.** The user explicitly approved the
dual-track strategy: bring focused, independently valuable improvements
upstream, and later ingest a minimal DSP core for Sustain's product-specific
architecture. `AGENTS.md` now reflects that decision. Licensing/provenance work
remains mandatory before ingestion.

**Resolved API/consumption decision:** do not standardize a Sustain-shaped
interface upstream. Sustain will own its clean capability-driven interface over
the minimal ingested DSP core. Upstream remains the destination for focused,
general-purpose fixes that stand on their own.

---

## STATUS — verification result (2026-06-03)

- **The maintainer is NOT gone.** `GET /users/HLLMR` → 200, live `User` account
  (id 147449512, 6 public repos), `updated_at` = **today, 2026-06-03**. Repo resolves,
  not archived/disabled. Commits authored `HLLMR <github@dah.mm.st>`. A user-side
  inability to load the profile page is most likely transient, not a deleted account.
- ⇒ The "author is gone, we must ingest" trigger did **not** fire. Both paths below are
  live; this is now a deliberate choice, not a forced fallback.

## DECISION (superseded 2026-06-04) — A first, B as fallback

The earlier fallback-only framing is superseded. Focused upstream work and the
later minimal Sustain ingest are now separate approved tracks. The ingest does
not depend on upstream stalling.

License is clear for either path (`MIT OR Apache-2.0` → GPL-3.0-or-later OK with
attribution; vendored files keep their MIT/Apache notices, not sustain's GPL header).

---

## The shared technical finding (true under both paths)

`stratum-dsp`'s **library never uses symphonia.** Verified: `grep -rn symphonia src`
→ zero hits; symphonia appears only in `examples/analyze_file.rs` and
`examples/analyze_batch.rs` (identical `decode_audio_file` helpers). `archive/src/io/`
holds the *removed* decoder — decoding was deliberately pulled out of the library and
the dependency declaration was never demoted. Yet `Cargo.toml` has:

```toml
[dependencies]
symphonia = { version = "0.5", features = ["all"] }   # wrong section → forced on all consumers
```

This single misplaced line is why sustain is forced to compile symphonia 0.5 "all".

Public API is sample-in: `analyze_audio(&[f32], u32, AnalysisConfig) -> AnalysisResult`
(+ `compute_confidence`, config/result types). No symphonia type is exposed, so removing
the dependency is **not** a breaking change.

---

## Path A — focused upstream PR (PRIMARY — do this now)

**Goal:** one immaculate, tightly-scoped, humble PR that (1) moves `symphonia` to
`[dev-dependencies]` — the load-bearing fix that removes it from every downstream graph,
sustain included — and (2) modernizes the two examples to symphonia 0.6. Library API and
behavior unchanged (guaranteed — it never touched symphonia).

**Non-breaking / patch-release argument** (put in PR): symphonia isn't in the public API,
so demoting it + bumping a dev-dep ships in a **patch (1.0.1)**. MSRV bonus: any
symphonia-0.6 MSRV bump applies only to dev/example builds, never to library consumers.

**Phases**

- **Phase 0 — baseline.** In `./stratum-dsp` on `main`: `cargo build`, `cargo test`,
  `cargo test --test integration_tests`, build both `--release` examples, `cargo fmt
  --check`, `cargo clippy --all-targets -- -D warnings`. Snapshot the examples' decoded
  samples on `tests/fixtures/*.wav` as the "before" for the equivalence check.
- **Phase 1 — the fix (own commit).** Move the `symphonia` line into
  `[dev-dependencies]`, keep `0.5` for a pure reclassification. Verify lib graph clean:
  `cargo tree -e normal -i symphonia` → empty; `cargo tree --edges dev` still shows it;
  examples still build; full gate green. Commit:
  `deps: move symphonia to dev-dependencies (library has no audio I/O)`.
- **Phase 2 — examples → 0.6 (own commit).** Bump dev-dep to `0.6` (consider narrowing
  `features` from `"all"` to `mp3,aac,flac,wav,ogg,vorbis`). Port both files against the
  official guide `https://github.com/pdeljanov/Symphonia/blob/main/docs/guides/migration/0p6.md`.
  Surface to remap (verbatim from current code): imports
  `core::audio::{AudioBufferRef, Signal}`, `core::codecs::{DecoderOptions,
  CODEC_TYPE_NULL}`, `core::formats::FormatOptions`, `core::io::MediaSourceStream`,
  `core::meta::MetadataOptions`, `core::probe::Hint`, `core::sample::i24`,
  `default::{get_probe,get_codecs}`, `core::errors::Error::DecodeError`. Known 0.6
  changes that bite this code: **audio primitives redesigned** (the
  `match decoded { AudioBufferRef::F32(buf) => buf.chan(0)… }` + per-format mono mix +
  `i24.inner()` block); **EOS flipped to `Ok(None)`** so
  `while let Ok(packet) = format.next_packet()` must become
  `while let Ok(Some(packet)) = …` without swallowing real errors; **track info moved out
  of `codec_params`** (`.codec` / `.sample_rate`); recheck `get_probe().format(...)` and
  `get_codecs().make(...)`. Commit: `examples: migrate audio decoding to symphonia 0.6`.
- **Phase 3 — in-scope cleanups only.** e.g. the duplicated `decode_audio_file` across
  both examples (optional `examples/common/mod.rs`). **Do not** touch DSP/config/CLI —
  out of scope; note unrelated findings in the PR or as upstream issues, don't bundle.

**Verification:** library tests identical before/after; example decode-equivalence on the
fixtures (Vec<f32> length + values within epsilon); full gate mirroring their
`.github/workflows/ci.yml`; ideally their Python validation harness shows BPM/key
unchanged.

**Codex audit gate (mandatory, before opening the PR).** Implement on a branch →
self-verify → **Codex audits** → fix all findings → only then open the PR. Codex checks:
0.6 port correctness (EOS loop, sample-format re-expression, no swallowed errors), scope
discipline, backward-compat proof (not just claim), cleanliness/no-hacks, dependency
hygiene (symphonia gone from normal graph; features + MSRV justified), and commit/PR
hygiene. Findings are blocking.

**House style** (their `CONTRIBUTING.md`): conventional-commit prefixes
(`deps:`/`feat:`/`fix:`/`docs:`), present tense, fork→branch→PR, update `CHANGELOG.md`,
`cargo fmt` + `clippy` clean. **No co-authored commits / no AI attribution trailers.**
Whether to disclose AI assistance in the PR body is the **user's call** — ask first.

**Adoption (no git-pin):** sustain benefits only after upstream merges **and publishes**
a release; then sustain bumps the version and lands its own symphonia-0.6 migration
(sustain #172) as the sole copy. If upstream goes unresponsive, escalate per the user's
decision (ping/offer release help; else hard-fork-to-crates.io or vendor) — out of scope
here.

---

## Path B — minimal ingest into Sustain & tailor (APPROVED LATER TRACK)

**Rationale:** full control; drop the unused symphonia dependency outright (sustain feeds
samples, so the vendored crate needs **no** decoder and **no** symphonia at all → clean
single-version 0.6 tree immediately, no upstream-release dependency); freedom to improve
the DSP for sustain's needs (note: upstream's own CONTRIBUTING cites key-detection exact
match at ~17.6% — real room to tailor). **Cost:** absorbing ~19k LOC of third-party DSP
as a permanent sustain maintenance burden; tension with sustain's "boring, maintainable
dependencies" value — weigh honestly.

**Licensing (must do it right):**
- Elect MIT (or keep both) — `MIT OR Apache-2.0` permits incorporation into GPL-3.0+.
- Vendor as its own workspace crate (e.g. `crates/<name>/`), retaining upstream
  `LICENSE-MIT` / `LICENSE-APACHE` and copyright. Vendored `.rs` files **keep their
  MIT/Apache identity** — do **not** add sustain's `GPL-3.0-or-later` SPDX header to them.
- Add a `PROVENANCE`/README note: source repo, commit hash, version, date, and what was
  changed. Record any third-party licenses in sustain's existing license tooling
  (`cargo-about` / THIRD-PARTY-LICENSES).
- Drop symphonia + the examples entirely from the vendored copy; keep only the
  sample-in analysis library. Strip `ort`/`ml` unless sustain wants it.

**If Path B is chosen, this PLAN.md is superseded.** Break the work into sustain issues
(vendoring + licensing/provenance; symphonia removal; wiring sustain-analysis to the
vendored crate; sustain #172 symphonia-0.6 as sole copy; any DSP tailoring) and track
them the normal sustain way. Apply sustain's `CLAUDE.md` standards to sustain-authored
glue, not to vendored files.

---

## Honest risks

- **Path A:** depends on a low-activity maintainer merging **and releasing**; timeline
  out of our control. Single-maintainer hobby-crate dependency risk persists.
- **Path B:** permanent maintenance of a large foreign DSP codebase; must get licensing/
  attribution exactly right; bigger near-term effort.
- **0.6 audio-buffer API** specifics must be learned from the migration guide + compiler;
  this plan deliberately does not assert the new symbol names.
