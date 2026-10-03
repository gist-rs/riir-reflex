# Bench 120 — the Issue-064 density pilot: the ascent leg HAS headroom (kill gate PASS on both suites)

**Status:** COMPLETE — pilot measured 2026-10-04 (m3, host `m3`, datasets `.raw/datasets_t20k`).
Instrument: `harness --synth-density-pilot` (new exclusive early-exit mode, report-only —
no artifact, no teacher, no acceptance-path change). Files: `density_pilot.json` /
`DENSITY_PILOT.md` (massive_intent_en, the V5 board suite + the published 0.7800 anchor's
pool) and `banking77_*` (the generalization control).

## The question (Issue 064 task 1)

Does a minimal-deviation density gate — accept a synth candidate iff its corpus-density
deviation from its source frame sits within the corpus's own natural variation — have
headroom (≥ 5% accept) AND-ed with the existing teacher veto? Below 5% the ascent leg
dies (no headroom), per the issue's own kill gate.

## The instrument

- **Density proxy** (the issue's "seat-corpus density proxy" arm — modelless, zero new
  deps): a vMF kernel `ℓ(x) = logsumexp_j(cos(x,c_j)/τ) − log n` over the ENGINE'S OWN
  256-dim hashed-bag embedding (`crate::embed::Embedder` — the projection the serving
  seat actually scores against).
- **τ per label, data-derived**: the mean same-label cosine deficit — the kernel is
  e⁻¹-scaled at the label's own natural separation. No free knob.
- **LOO for pool rows** (the self-cosine-1 term excluded — otherwise every pool row's
  own density is inflated and Δ biased); full-sum for candidates.
- **Δ = ℓ(cand) − ℓ_loo(src row)** — `SynthCand.src` is LABEL-LOCAL (mining iterates the
  label's rows; the pilot's first run mapped it as a global index and produced garbage —
  205,604 misses — caught by the src-misses counter before any number was recorded; the
  `SynthCand` doc now says label-local).
- **The control (ε_nat)**: per label, each pool row vs its nearest same-label neighbour
  (ties → lowest index) — the real-data analog of a new row joining its closest frame.
  The control |Δ| quantiles are the ε ladder.
- Deterministic: BTreeMap order, fixed float summation order, no RNG — two runs
  bit-identical (verified; only the `seconds` telemetry field differs).

## The readings

| suite | candidates | labels | accept@ε_nat(p50) | accept@p75 | accept@p90 | ascent (Δ>0) | dev ratio | src-misses | verdict |
|---|---|---|---|---|---|---|---|---|---|
| massive_intent_en | 212,912 | 59 | **38.3%** | 63.5% | 82.1% | 40.9% | 0.66× | 0 | **PASS** |
| banking77 | 135,249 | 77 | **41.5%** | 67.2% | 84.0% | 45.3% | 0.30× | 0 | **PASS** |

- **Kill gate: PASS on both** — 7.7× / 8.3× the 5% bar at the natural-variation scale.
- **The minimal-deviation signature holds**: the median candidate's |Δ| sits at 0.66×
  (massive) / 0.30× (banking77) of its label's natural-neighbour |Δ| — transplants are
  *inside* the corpus's own variation, exactly the property projection sampling wants
  (the candidate does not degrade the frozen consumer's density).
- Weakest label cells still clear the bar where it matters: massive's tightest label
  (audio_volume_other, 15 pool rows, 229 cands) reads 14.0%@nat; datetime_convert (16
  cands) reads 6.2% — small cells, both above 5%.
- Ascent ≈ 41–45% (Δ > 0) — the optional ascent arm has material to work with; never
  gated on here.

Wall: 33.8 s (massive) / 24.9 s-class (banking77) on the m3 — the pilot scores the FULL
bucket set (no teacher calls, no cap).

## What this UNBLOCKS (and does not)

- UNBLOCKS Issue 064 task 2: wire the ascent leg behind a flag — accept iff
  |Δ| ≤ ε_nat(label) (AND-ed with the teacher veto, never replacing it), then the
  corpus-ab V5 + OOD rig + abstention-entropy gates the issue names. The ε rung choice
  (p50 vs p75) is a real A/B question the flag makes cheap.
- Does NOT claim any accuracy gain — this bench is a HEADROOM measurement only. The
  learned-quality question belongs to the corpus-ab lane, as the issue specifies.
- The task-3 DRY coordination (shared scorer with riir-train Plan 438 Phase 2) stays
  deferred until that second consumer exists — unchanged.

## Provenance

- Binary: worktree HEAD (feature `modelless`, release), `m3` host, 2026-10-03T23:00Z /
  2026-10-04 runs.
- No latency claim is quoted from this run (accept-rate fractions + wall seconds only)
  — the Issue-021 box-state law applies to timing tables, not this record's fractions.
