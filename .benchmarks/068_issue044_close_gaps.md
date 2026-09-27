# Bench 068 — issue 044: close the accuracy gaps to laya (T1–T5)

**Status:** DONE 2026-09-27 · every task carries a measured verdict; the two
real gaps (ag_news, xnli) stand with recorded reopen paths · code_fixtures
resolved-by-pin (republished) · routing/sensitivity resolved as non-gaps.

M3, AC, `datasets_t20k`, release binaries at `fe7c123` (the 5-suite run) —
latency cells quotable in the code_fixtures lanes run (box_state reads
quotable at BOTH ends, load 5.54 → 5.42, powermode 2(high), swap 1882 MB
warned; PROVENANCE quoted per run in its own results.json `box_state`).

## The verdict table (published baseline → this bench)

| suite | reflex (published) | laya (published) | T2 95% Wilson CI | verdict |
|---|---|---|---|---|
| code_fixtures | 0.2500 (n=24, old tree) | 0.6667 (n=24, old tree) | significant | **resolved-by-pin** — the population was the defect; on the frozen population reflex 0.3750 vs laya 0.4062 @ n=32, laya INSIDE reflex's CI [0.229, 0.548] → NOT significant |
| xnli_en | 0.5233 | 0.8600 | significant (+28.2 pt, 101 questions) | **real gap** — pair-feature signal EXISTS (+5.67 pt harvestable) but the blend FAILS G1; see T3 |
| ag_news | 0.8825 | 0.9500 | significant (+6.8 pt, 27 questions) | **real gap** — no shipped lever moves it (T1); encoder-owned word-order remains the live hypothesis |
| harness_routing | 0.4375 | 0.6250 | **NOT significant** (gap = 3 questions; laya inside CI [0.231, 0.668]) | resolved as non-gap — no accuracy work owed at this n |
| harness_sensitivity | 0.4000 | 0.6000 | **NOT significant** (gap = 3 questions; laya inside CI [0.198, 0.643]) | resolved as non-gap |

## T1 — ag_news: the existing levers, measured

The published posture IS the lever surface: head/nb/oc/ridge selection all
run in the published row (its own `*_selection` blocks). What the shipped
levers do to the residual gap, measured:

- **corpus volume** — Bench 065: the 120k full pull +0.25 pt, the corpus-cap
  axis exactly nothing, the genome walk over the full-volume posture HELD.
- **joint blend-genome** — Bench 064 round 1 (cap 64): HELD; round 3 of 065
  (cap 1000): HELD. The composed posture sits at the slice's plateau.
- **this bench re-confirms the posture**: the 5-suite run at the published
  flags reproduces ag_news **0.8825** exactly (pick-identical), and the
  xnli/en route through the same run reads 0.5233 — the lane is where
  Benches 052/063/064/065 left it.

Verdict: **no shipped lever moves ag_news.** The residual −6.8 pt is not
volume-, cap-, posture-, or selection-bound on any measured axis. The live
hypothesis stays 065's fallback: word-order/disambiguation the encoder
owns. Closing it needs a NEW modelless mechanism (not a lever), and any such
mechanism must clear the same gates this bench's T3 head just demonstrated
(G1 first).

## T2 — the Wilson screen (before spending effort)

One question moves: code_fixtures 4.17 pt (n=24 old / 3.125 pt n=32 new),
routing 6.25 pt, sensitivity 6.67 pt, xnli 0.33 pt, ag_news 0.25 pt. The
95% intervals above put **routing and sensitivity's gaps inside noise** —
3 questions each, and laya's published reading sits inside reflex's CI on
both. The two families owe no accuracy work at this n; if they ever matter,
the lever is MORE AUTHORED QUESTIONS (the families are synthetic —
`families.rs` — so growing the eval slice is the protocol fix), not a
mechanism.

## T3 — xnli_en: the modelless pair-feature head (the round's real finding)

`--nli-feature-ab` (`src/harness/runner/nli_lane.rs`, report-only): a
closed-form diagonal-LDA over 10 lexical pair features (jaccard, containment,
length ratio, hyp/prem negation, negation diff, shared/unique numbers,
negation-prefix cross, antonym cross) fitted on the CAL slice (n=200), one
test read (n=300), under three postures:

| posture | cal | test | Δ vs engine |
|---|---|---|---|
| engine baseline | — | 0.5233 | — |
| head ALONE | 0.585 | **0.5333** | +1.0 pt |
| additive blend, λ=0.5 (cal-selected from [0.5,1,2,4,8]) | 0.610 | **0.5800** | **+5.67 pt** |
| override gate, τ=2.0 (cal-selected) | 0.615 | 0.5133 | −1.0 pt |
| oracle (head vs engine decorrelation) | — | 56 unique wins / 53 unique losses | — |

The head alone BEATS the engine, the additive blend harvests **+5.67 pt**
(0.5233 → 0.5800, above the house 5 pt arming bar), and the oracle row shows
the errors are decorrelated (56/53) — modelless pair reasoning EXISTS on
xnli. **But the posture is NOT promotable:** the blended readout
`q ∝ p_engine + λ·p_head` carries ECE **0.1596 against the conformal-naive
floor 0.1351 → G1 FAIL** (the engine's own calibrated readout sits at
0.0067). This is the Bench-064 massive-row law binding a second time: a
pick-level gain that breaks the calibration floor is a refused promotion —
the accuracy column alone would have called this the round's win.

Recorded reopen path: a PRE-REGISTERED cal-side G1-constrained λ selection
(pick λ on the cal ladder subject to the cal-side blend ECE ≤ the floor's
cal-side analogue), or a calibrated blend readout (sigmoid-gate refit under
the blend posture). Either lands as its own bench with its own G1 verdict —
never as a silent posture bump. The lane's published xnli row is UNCHANGED
(0.5233); the record rides `results.json` (`nli_feature_ab`).

En-route catch: the module test caught the fit's class counter never
incrementing (`counts.get_mut(y)?` only borrowed) — the head would have been
dead-on-arrival (every fit `None` → loud skip) without it.

## T4 — code_fixtures: the population pinned

The suite was commit-relative (harvested from `src/` at every build): the
`laya/agent.rs` + `laya/router.rs` moves left **4 of 8 option labels without
corpus docs** (the issue-039 guard's self-doc fallback) and the published
accuracy moved with the tree — 0.2500 published vs 0.2917 measured the same
week on the same flags. The population is now a **BLAKE3-pinned committed
fixture** (`src/harness/code_fixtures_frozen.json`, digest
`034774df…b44362`; 8 healthy modules × full 2-eval/8-cal/6-docs slices; the
builders read the frozen bytes; `cargo run --features modelless --example
gen_code_frozen` regenerates — a deliberate fixture change, never a build
side effect; hand edits fail the parse-time digest check).

On the frozen population (n=32 questions — the old suite only fielded 24,
four modules having lost their second eval span):

| lane | acc | notes |
|---|---|---|
| modelless | **0.3750** | no corpus-fallback labels; posture = the shipped lane |
| laya (english, metal) | **0.4062** | G5 parity green at this posture (11.6 s) immediately before the run |

**The +41.7 pt published gap was mostly a population artifact.** Reflex's
95% CI at n=32 is [0.229, 0.548] — laya's 0.4062 is INSIDE it. The residual
+3.1 pt is not significant. The suite's task also got harder for BOTH lanes
(laya 0.6667 → 0.4062): the frozen module set is genuinely confusable
(three `harness::*` internals among the eight options), which is what a
non-degenerate fixture should look like.

Republished lane-scoped at reflex-site (both lanes, one run, one
`cases_digest`; box_state quotable both ends — latency cells publish).

## T5 — the publish wall

The code_fixtures lane update went out with `latency_quotable: true` from
its own run's box_state — the wall is satisfied, nothing to waive. The
5-suite record run's own box_state (load 5.9 class) also read quotable; its
latency cells are same-class, not comparison-grade (a sibling build was
churning earlier in the session and the 1m average had not fully settled)
— the record's claims are the ACCURACY columns, which are pick-counts and
load-immune by construction.

## What this issue leaves open

- **ag_news −6.8 pt**: real, measured-unreachable by the shipped surface.
  A new modelless mechanism would have to exist and clear G1 first — the
  T3 head is the existence proof that such mechanisms are findable, and the
  G1 failure is the existence proof that they gate.
- **xnli −28.2 pt**: the pair-feature head's +5.67 pt is REAL and
  decorrelated; the promotion path is the pre-registered G1-constrained λ
  (or a calibrated blend readout). Even promoted, laya stays far ahead —
  the gap is structural (an encoder vs lexical features), and this issue's
  honest contribution is the measured ceiling of the lexical layer.
- routing/sensitivity: not gaps at this n (Wilson); revisiting requires
  growing the authored eval slices first.

Artifacts: `.benchmarks/044_close_gaps/` (the 5-suite record run at
`fe7c123`, `--nli-feature-ab` on) and `.benchmarks/044_code_fixtures_lanes/`
(the frozen-population both-lanes run; quotable box_state). The generator's
harvest log + digest are in the fixture commit (`fe7c123`).
