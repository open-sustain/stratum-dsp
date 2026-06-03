# PLAN — stratum-dsp symphonia hygiene / 0.6 (Path A now → Path B fallback)

> Internal planning doc (untracked — do **not** commit to any upstream PR).
> Repo cloned from `https://github.com/HLLMR/stratum-dsp` @ `main`.
> Consumer that motivated this: `../sustain` (do not touch; another agent works there).

---

## STATUS — verification result (2026-06-03)

- **The maintainer is NOT gone.** `GET /users/HLLMR` → 200, live `User` account
  (id 147449512, 6 public repos), `updated_at` = **today, 2026-06-03**. Repo resolves,
  not archived/disabled. Commits authored `HLLMR <github@dah.mm.st>`. A user-side
  inability to load the profile page is most likely transient, not a deleted account.
- ⇒ The "author is gone, we must ingest" trigger did **not** fire. Both paths below are
  live; this is now a deliberate choice, not a forced fallback.

## DECISION (resolved) — A first, B as fallback

- **Path A — upstream PR — is THE PLAN. Execute it now.** Maintainer is active; the core
  fix is trivial and non-breaking, so it should be easy to merge.
- **Path B — ingest into sustain — is the FALLBACK, triggered only if the PR is not
  merged in a reasonable timeframe.** Do not start B preemptively.

**Trigger for falling back to B (confirm the exact window with the user):** open the PR,
send one polite nudge after ~1–2 weeks of silence, and if there's no merge or substantive
engagement by ~3–4 weeks (≈ early July 2026 given a June 2026 open date), switch to
Path B. Until that trigger fires, Path B stays untouched.

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

## Path A — upstream PR (PRIMARY — do this now)

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

## Path B — ingest into sustain & tailor (FALLBACK — only if the PR stalls)

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
