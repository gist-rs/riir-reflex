# Bench 131 — the kn-route A/B (issue 079): typed's legacy `k == N` binding measured both ways, the perm acceptance, the gold-position leak, and one G3 finding (sst5)

**Status:** COMPLETE — cells 1–8 measured on `m3` at HEAD `03533fb` (worktree content later committed verbatim as `459d344` by a sibling; one instrument, one binary — see Provenance). Control RED expected-and-reproduced on the incumbent; content-bound PASS; G3 surfaced a still-on-the-legacy-path suite (sst5); G2/G4 PASS; laya/typed re-read RED 4.12 pt. No promotion claim — the owner verdict owns the call.
**Status (follow-up, `4ad25df`): the G3 finding is RESOLVED** — the coordinator's index-anchored route arm replaces the blind fill (Score + k == N + domains literally named "0".."N-1"), sst5 byte-preserved at the new default (0.3967 digit-exact; strict posture still collapses 0.1567), typed content-bound AT the default (0.5630 digit-exact; per_byte 0.5700), full-registry re-diff clean (typed the only mover), perm PASS byte-identical to the knob-off run. See `## Follow-up` below. No promotion claim — the owner verdict owns the call.

## What this is

typed_decisions rides the engine's legacy `k == N` route binding (bench 128: the only
remaining RED control — median 4.70 pt swing, 68.3% flips under option permutation). The
instrument (committed at HEAD): `EngineConfig::legacy_kn_route` (default `true` =
incumbent, byte-identical), harness `--no-kn-route` (content-bound posture: route terms
arm only via by-name resolution), and `--drafter-fix <off|per_byte|ncd|shared_prefix|key_only>`
(the content-bound tuning axis, `Off` = shipped scores). This bench measures both postures
per the owner verdict round 1, verbatim.

## Cell 1 — typed A/B, full suite (the delta cell)

Promoted posture (`--skip-laya --nb-select --oc-select --ridge-select`, gate-fit-calibrated
default, canonical pool `.raw/datasets`), n = 2000 questions / 400 cases:

| posture | hard acc | cal. abstain rate | sel. n | sel. acc | ECE (cal) | floor ECE | G1 | p50 ms | p99 ms |
|---|---|---|---|---|---|---|---|---|---|
| incumbent (`kn_route=on`) | **0.5725** | 0.7170 | 566 | 0.6537 | 0.0496 | 0.1818 | PASS | 1.023 | 2.706 |
| content-bound (`--no-kn-route`) | **0.5630** | 0.7445 | 511 | 0.5068 | 0.0192 | 0.2778 | PASS | 0.961 | 2.780 |

- Incumbent 0.5725 reproduces the published typed number digit-for-digit.
- Delta: **−0.95 pt hard** at content-bound, and the calibrated gate's selective set is
  weaker (0.6537 → 0.5068 at similar coverage) — the route term is load-bearing for both
  the hard pick and the gate's confidence ordering.
- Per-posture cal selection re-ran and picked differently, as designed: oc_scale 4.0
  (cal 0.544) incumbent vs **0.25** (cal 0.548) content-bound. The oc term is keyed
  `(qid, option)` — permutation-invariant by construction — and at content-bound it does
  the work the position-bound route used to do.
- G1 (Report-the-Floor) PASSES both ways: calibrated ECE 0.0496 / 0.0192 against the
  conformal-naive floor 0.1818 / 0.2778.
- ⚠ Latency columns are **provisional**: preflight REFUSED the box (load 11.8–19.3 all
  run; see Provenance). p99 tail support is 5 everywhere — never a strong read. The
  correctness columns are load-insensitive.

## Cell 2 — B-posture tuning sweep (verdict amendment b)

All at `--no-kn-route`, `--oc-select` on in every run (re-selects per posture; 0.25 in all
five), n = 2000:

| drafter-fix | hard acc | cal. abstain | sel. n | sel. acc | ECE (cal) | floor | p50 ms |
|---|---|---|---|---|---|---|---|
| **off** (the B default) | 0.5630 | 0.7445 | 511 | 0.5068 | 0.0192 | 0.2778 | 0.961 |
| **per_byte** | **0.5700** | 0.6320 | 736 | 0.5747 | 0.0946 | 0.2164 | 1.031 |
| ncd | 0.5670 | 0.6020 | 796 | 0.5892 | 0.0854 | 0.2157 | 1.785 |
| shared_prefix | 0.5630 | 0.7445 | 511 | 0.5068 | 0.0192 | 0.2778 | 1.043 |
| key_only | 0.5535 | 0.8105 | 379 | 0.5303 | 0.0374 | 0.2571 | 1.459 |

- **Chosen B posture: `per_byte` (0.5700)** — recovers 0.0070 of the 0.0095 gap; the
  residual incumbent-vs-best-B delta is **−0.25 pt**.
- Honest notes: `shared_prefix` is byte-identical to `off` on EVERY metric — verified
  structural no-op: 0 of 600 choice questions in the suite share any non-empty option
  prefix (typed keys: `continue/human_review/observe/stop`, `failure/harmful/partial/
  success`, … — no shared stem). `ncd` is the best selective-accuracy cell (0.5892 @ 796)
  but costs ~1.8× p50 (a second drafter pass per option) — provisional, loaded box.
  `key_only` is strictly worse (many typed keys are not `key: value` shapes; stripping
  bytes up to `: ` throws away the discriminating content).

## Cell 3 — perm-probe, typed, both postures (the acceptance cell)

Plain posture (the probe pins it and REFUSES if the fitted config arms oc/nb — measured
that way on purpose), K = 5 orderings, 40 sampled cases → **60 choice slots**
(40×4-option + 20×5-option; bench 128's "60" was slots, not cases):

| run | route binding | n | median pt | max pt | flips | rate | byte-identity | verdict |
|---|---|---|---|---|---|---|---|---|
| perm_incumbent | legacy k==N | 60 | **4.70** | 18.81 | 41 | 0.683 | false | RED (expected — the finding) |
| perm_content | content-bound | 60 | **0.00** | **0.00** | 31 | 0.517 | false | **PASS** |
| perm_content_per_byte | content-bound | 60 | 0.00 | 0.00 | 31 | 0.517 | false | PASS |

- **ACCEPTANCE: PASS** — median swing 0.00 pt ≤ 0.01 pt ceiling, and every one of the 31
  flips is an exact tie (the control verdict PROVES this: the probe reds any untied flip —
  `any_untied_flip` is a RED condition; a PASS with 31 flips ⟹ 31 tie-structure flips).
- Incumbent reproducibility: **4.70 / 18.81 / 41-of-60 / 0.683** — digit-identical to
  bench 128's pre-fix read (same pool, same seed 125239297, same stride). Small-diff
  disclosure not needed: there is no diff.
- Byte-identity false in BOTH = the disclosed bench-128 ULP class (the L1 sum reorders
  with the presentation); display-median 0.00 pt at content-bound. Not fixed, unchanged,
  bounded — the standing disclosure stands.
- ⚠ Honest expectation note (the tie pile): the 31 tie-flips are NOT noise — they are the
  content-bound posture's real shape. With route terms off, typed's drafter-only i32
  deltas tie EXACTLY across options on ~half the slots (10/60 are full all-way ties:
  p_ref == 1/k at EVERY ordering; the rest tie in the top-2). An exact tie breaks to the
  first-presented option, so the pick moves with the presentation — the tie STRUCTURE the
  contract deliberately excludes from the verdict. Consequence worth the owner's eyes:
  on tied slots the content-bound plain posture picks by coin-break, which is part of why
  cell 1's hard accuracy drops and why the gate's selective set thins.
- Instrument plumbing gap (disclosed, not worked around): `perm_modelless` hardcodes
  `DrafterFix::Off` (src/harness/runner/perm_probe.rs L534), so `--drafter-fix` does not
  reach the perm control. The third run is byte-identical to `perm_content` except meta
  (verified: only date_utc / git_sha / box load differ) — proof by execution, not
  inference. If the owner wants the per_byte perm cell, the flag needs plumbing (one-line
  `ModellessInput` forward) — NOT done here (no src/ edits per the measurement mandate).

## Cell 4 — leakage decomposition (verdict amendment a)

Script: `scripts/issue079_gold_position.py` (stdlib only; replicates
`build_typed_decisions`'s parse — JSON-string columns, insertion-ordered criteria keys,
`str(gold.label)` through `py_str`, the exact per-question skips — and the probe's
`stride_indices(400, 40)` sampling; artifacts: `gold_position.json`, `perm_summary.json`).
Sampling derivation stated: the modelless control uses `DEFAULT_PERM_MAX_CASES = 40`
(stride ceil(400/40) = 10 → 40 cases → 60 choice slots; bench 128's "60" was slots —
there is no separate modelless per-posture max; laya's 20 is an invocation flag, cell 8).

### (i/ii) Gold-position histogram over the probe's 60 slots (+ full suite as context)

| population | n_options | slots | pos 0 | pos 1 | pos 2 | pos 3 | pos 4 | chance |
|---|---|---|---|---|---|---|---|---|
| sampled (probe) | 4 | 40 | 9 (22.5%) | 7 (17.5%) | **16 (40.0%)** | 8 (20.0%) | — | 0.2500 |
| sampled (probe) | 5 | 20 | 3 (15.0%) | 2 (10.0%) | **9 (45.0%)** | **0 (0.0%)** | 6 (30.0%) | 0.2000 |
| full suite | 4 | 400 | 84 (21.0%) | 82 (20.5%) | **169 (42.2%)** | 65 (16.2%) | — | 0.2500 |
| full suite | 5 | 200 | 35 (17.5%) | 38 (19.0%) | **72 (36.0%)** | 11 (5.5%) | 44 (22.0%) | 0.2000 |

(iii) **Chance accuracy given the gold distribution: 0.2333** (sampled slots; same figure
full-suite) — against uniform 0.25/0.20. The gold labels pile on **position 2**
(40–45% vs 20–25% uniform) and avoid 5-option position 3 (0–5.5%). The incumbent binds
option-at-position-i → domain-i's route term, so any route term that scores index 2 high
harvests the pile: the legacy binding is a **gold-position leak by construction** — part
of the published hard accuracy is position prior, not content.

### (iv) Permutation-averaged baselines (the honest incumbent baseline)

From both perm runs' `perm_probe.json`, modelless control lane, gold joined by
`(case_id, qid)` (60/60 matched, 0 unmatched):

| run | identity acc | per-ordering acc | **perm-averaged** | all-way ties | mean p_ref (identity) |
|---|---|---|---|---|---|
| incumbent | 0.1833 | [0.183, 0.100, 0.217, 0.233, 0.183] | **0.1833** | 0/60 | — |
| content-bound | 0.1333 | [0.133, 0.067, 0.133, 0.050, 0.117] | **0.1000** | 10/60 | — |
| content-bound (+per_byte mode — see the plumbing gap) | 0.1333 | identical | 0.1000 | 10/60 | — |

- ⚠ Expectation note: these are PLAIN-posture probe numbers over 60 choice slots — a
  different population from the published 0.5725 (that is the promoted posture over 2000
  questions incl. score/noul with the calibrated gate). The probe slice reads BELOW the
  0.2333 positional chance on both postures: with no levers, typed choice questions are
  barely-above-coin material, and permuting options does not average the incumbent UP —
  ordering variance spans 0.10–0.23 across the five orderings.
- Per-ordering Brier: **NOT DERIVABLE** — the probe record carries `p_ref` (mass on the
  identity pick) only, never the full per-ordering vector. Stated, not imputed; the
  full-vector record would need a probe field (owner's call, not this bench).

## Cell 5 — G3 byte-identity (verdict amendment c)

Full-workspace modelless, default suites, laya skipped, levers on, both postures. Suite
set identical both sides (10 suites; `wanli_en` absent BY REGISTRY — `named_only: true`
suites never tax a default run, runner.rs L7452 — not an absence):

| suite | incumbent | content | verdict |
|---|---|---|---|
| ag_news, banking77, code_fixtures, emotion, massive_intent_en, prompt_injections, semantic_defects, xnli_en | — | — | **digit-identical** (hard + both abstain cells) |
| typed_decisions | 0.5725 | 0.5630 | the expected mover (matches cell 1 exactly — cross-run determinism) |
| **sst5** | 0.3967 | **0.1567** | ⛔ **FINDING — a still-on-the-legacy-path suite** |

### ⛔ The sst5 finding (reported loud, mechanism diagnosed, NOT fixed — no src/ edits)

sst5 moved **−24.0 pt** (cal. abstain 0.532→0.367, sel. acc 0.4057→0.1632). Mechanism,
verified in code and data:

1. `build_sst5`'s option strings are the WORD LEVELS `"very negative".."very positive"`
   (suites.rs `LEVELS` const) — not "0".."4".
2. sst5's engine domains are named by the train labels' INTEGER spellings
   ("0".."4"; the b3520a5 label rule deliberately keeps the integer).
3. By-name resolution requires byte-equality of every option string with a domain name —
   word vs integer can never match → `by_name` is FALSE on every sst5 question.
4. The legacy `k == N` branch (5 == 5) was therefore sst5's ONLY route binding — and it
   is identity (opt_dom[i] = i), so it has been silently load-bearing.
5. `--no-kn-route` leaves sst5 drafter-only → the documented constant-pick collapse
   (engine.rs: "drafter-only scoring made every option identical and the argmax tie broke
   to index 0 — a constant pick", the Issue-004 T7 lesson): 0.3967 → 0.1567.

This is the SAME class bench 128's content-binding repair fixed for ag_news/emotion/
xnli_en — that repair skipped sst5 because "its option keys ARE '0'..'4'" was believed
(bench 128 record). True for the LABEL side; never checked for the OPTION side. Fix shape
(coordinator's call, one suite late): map sst5's train labels through the `LEVELS` const
in `train_row_label`'s sst5 arm so by-name arms — the same one-line content-binding
repair, and `kn_route` becomes inert on sst5 by construction (`kn_route_is_inert_when_by_
name_resolution_arms` already pins that law). Until then **`--no-kn-route` is NOT
typed-only**: it demotes sst5, and any future knob-off promotion must carry the sst5
repair or scope the knob to `named_only = false` suites that pass a by-name feasibility
check.

## Cell 6 — G2 (the incumbent-path latency gate)

`cargo bench --bench decision_set_goat` (release):

| posture | p50 | p99 | G4 |
|---|---|---|---|
| default | 47 µs | 65 µs | solve_into × 200: **0 allocations** |
| nb_scope armed | 36 µs | 45 µs | 0 allocations |
| option_cond armed | 19 µs | 23 µs | 0 allocations |
| nb_ridge armed | 34 µs | 67 µs | 0 allocations |

**G2 PASS** (p99 65 µs ≤ 1000 µs — the gate holds with 15× margin) · determinism
bit-identical in all four postures. ⚠ Box-state: measured under the standing preflight
REFUSAL (load ~13); the margin is far wider than any measured load tax, and the
cross-posture knob cost (one bool check per question) is below measurement grain. The
typed p50s in cells 1–2 (0.96–1.79 ms) are the cross-posture latency read; they are
provisional (loaded box, tail support 5) and the ncd/key_only columns carry real
algorithmic cost (a second drafter pass), not knob cost.

## Cell 7 — G4 (the alloc-canary bookkeeping)

- The counting allocator covering `DecisionEngine::solve_into` is the BENCH
  `benches/decision_set_goat.rs` (binary-unique `Counting` GlobalAlloc + canary +
  200-rep zero-alloc assert): **PASS — 0 allocations after warmup, canary counted 1**
  (instrument live). Run in cell 6.
- Grep tests/ finding, disclosed: the only counting allocator under `tests/` is
  `slice_leak_oracle.rs`'s, which covers the slice-leak probe's classify path behind
  `required-features = ["slice_leak"]` — it does NOT run in a plain `cargo test --lib`
  and does not cover `solve_into`. There is NO lib-test alloc counter for `solve_into`;
  the G4 verdict rests on the bench (which runs green). `cargo test --lib`: **339
  passed, 0 failed**, including the five committed instrument tests
  (`kn_route_off_is_permutation_invariant_and_on_is_position_bound`,
  `kn_route_is_inert_when_by_name_resolution_arms`,
  `drafter_fix_off_keeps_the_shipped_scores_bit_identical`,
  `drafter_fixes_engage_on_the_drafter_only_path`,
  `drafter_fix_spellings_round_trip`).

## Cell 8 — laya/typed re-read (board honesty)

G5 parity FIRST: `cargo test --release --features laya-riir --test laya_riir_parity` →
**2/2 PASS** (49.98 s) — the lane is certified, the cell may publish.

`LAYA_DEVICE=metal cargo run --release --features laya-riir,laya-riir-metal --bin
harness -- --perm-probe --perm-max-cases 20 --suites typed_decisions`:

| lane | n | median pt | max pt | flips | rate | verdict |
|---|---|---|---|---|---|---|
| modelless (control, incumbent posture, 20-case slice) | 30 | 6.55 | 13.62 | 21 | 0.700 | RED (expected) |
| laya/typed (typed checkpoint, Metal) | 30 | **4.12** | 9.23 | 4 | 0.133 | **RED** (lane gate ≤ 2 pt) |

- The board-honesty point LANDS: our content-bound control holds 0.00 pt median, but the
  laya lane on typed is still **4.12-pt order-fragile** (4/30 flips, canary clean). The
  laya lane does not read the engine knob — this cell is the disclosure that the BOARD's
  typed row (either lane) must not be presented as order-fixed. A laya-side option-order
  fix (or a canonical-presentation law at the serve edge) is a separate lane's work.
- First attempt note: the task's command template omitted `--features laya-riir-metal`;
  with bare `--features laya-riir` the laya lane SKIPPED LOUD ("LAYA_DEVICE=metal needs
  --features laya-riir-metal on macOS — fail loud, never a silent CPU fallback") — the
  fail-loud law working as designed. Re-run with the feature; nothing faked.

## Provenance

```
./scripts/bench_preflight.sh   # start of matrix:
PROVENANCE: power=AC Power load=11.80 swap=6108.88M canary=147.9us/best5 powermode=2(high)
✗ preflight REFUSED — do not publish a latency number from this box now
# re-check mid-matrix (before cell 6):
PROVENANCE: power=AC Power load=13.21 swap=6108.88M canary=128.1us/best5 powermode=2(high)
✗ preflight REFUSED — load average 11.80/13.21 exceeds MAX_LOAD=6.0 — a sibling session is on the box
```

- REFUSAL cause: a sibling session's compute (`hyperthink_t1_delta_census` + a rustc
  build), load 11.8–19.3 across the matrix. Every latency number above is marked
  provisional under that refusal; correctness cells (hard/abstain/ECE/perm/gold) are
  load-insensitive and stand. powermode = 2 (High Power) — the preflight's
  latency-refusal trigger is LOAD only; no Low-Power confound.
- Host `m3` (REFLEX_BENCH_HOST=m3 on every harness invocation; the meta's `host` field
  reads it). Workspace `target/release`, ONE build (`cargo build --release --bin
  harness`, 48.6 s, green) used for the whole matrix — never switched mid-matrix.
- Worktree disclosure: the matrix was measured from the `03533fb` worktree WITH the
  sibling's `wanli_en` WIP already present in it (additive new-suite plumbing: builder +
  slice rows + fetch lane; inert for every cell — the suite is `named_only: true` and
  absent from all default/typed runs). The sibling committed that content verbatim as
  `459d344` mid-matrix, so records written after that point disclose `git_sha 459d344`.
  One binary, one tree content; the sha move is a HEAD move, not an instrument change.
- Datasets: canonical pool `.raw/datasets` (typed_decisions test split = 400 rows whole,
  per protocols §3.1). Perm seed 125239297 (the probe's PROBE_SEED) — same as bench 128.

## Reproduce

```sh
cd /Users/katopz/git/riir-reflex
cargo build --release --bin harness
./scripts/bench_preflight.sh   # expect a green PROVENANCE line before quoting latency

# cell 1 — typed A/B (promoted posture, both route postures)
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --suites typed_decisions --out .benchmarks/131_kn_route_ab/incumbent
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --no-kn-route --suites typed_decisions --out .benchmarks/131_kn_route_ab/content

# cell 2 — B-posture sweep (four more runs at --no-kn-route)
for m in per_byte ncd shared_prefix key_only; do
  REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --no-kn-route --drafter-fix $m --suites typed_decisions --out .benchmarks/131_kn_route_ab/content_$m
done

# cell 3 — perm acceptance (plain posture; the probe refuses lever-armed fits)
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --perm-probe --suites typed_decisions --out .benchmarks/131_kn_route_ab/perm_incumbent
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --perm-probe --no-kn-route --suites typed_decisions --out .benchmarks/131_kn_route_ab/perm_content
#   (+ the per_byte run: byte-identical to perm_content — DrafterFix::Off is hardcoded
#    in perm_modelless; disclosed plumbing gap, see cell 3)

# cell 4 — leakage decomposition
python3 scripts/issue079_gold_position.py

# cell 5 — G3 full-workspace byte-identity
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --out .benchmarks/131_kn_route_ab/full_incumbent
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --no-kn-route --out .benchmarks/131_kn_route_ab/full_content

# cell 6 — G2+G4
cargo bench --bench decision_set_goat

# cell 7 — lib gate
cargo test --lib

# cell 8 — laya re-read (G5 first)
cargo test --release --features laya-riir --test laya_riir_parity
LAYA_DEVICE=metal REFLEX_BENCH_HOST=m3 cargo run --release --features laya-riir,laya-riir-metal --bin harness -- --perm-probe --perm-max-cases 20 --suites typed_decisions --out .benchmarks/131_kn_route_ab/laya_typed
```

Artifacts: `.benchmarks/131_kn_route_ab/{incumbent,content,content_per_byte,content_ncd,
content_shared_prefix,content_key_only}/results.json` · `{perm_incumbent,perm_content,
perm_content_per_byte,laya_typed}/perm_probe.json` · `full_{incumbent,content}/results.json`
· `gold_position.json` · `perm_summary.json` · script `scripts/issue079_gold_position.py`.

## Follow-up — the index-anchored refinement (`4ad25df`)

The coordinator landed bench 131's G3 finding's fix: the blind `k == N` identity fill is
REPLACED by an index-anchored NAME check — the route arm binds option i → domain i ONLY
when the question is **Score**, k == N, and the domain names are exactly `"0".."N-1"`
(sst5's ordinal level corpora: the LEGAL, noul-precedent class). **Choice NEVER takes the
arm** (typed's defect). Verification cells below; every expected digit landed, no
surprises.

### Cell 1 — sst5 byte-identity (the fix's core claim) — PASS

| run | posture | hard acc | verdict |
|---|---|---|---|
| `sst5_refit` | new default (knob on, index-anchored arm) | **0.3967** | digit-exact vs the bench-096-era incumbent cell |
| `sst5_strict` | `--no-kn-route` | **0.1567** | the collapse, digit-exact vs bench 131's full_content sst5 |

Byte-exactness beyond hard acc — full core rows identical to the bench-130 runs:
`refit == full_incumbent/sst5` and `strict == full_content/sst5` on (acc, macro_f1,
brier, abstain, sel_n, sel_acc, ECE, floor) — TRUE both. The index-anchored arm
reproduces the old blind fill's bytes on the legal shape, and the knob still gates it.

### Cell 2 — typed carries over — PASS

| run | posture | hard acc | verdict |
|---|---|---|---|
| `typed_refit` | new default | **0.5630** | digit-exact vs bench 131's content cell |
| `typed_refit_pb` | + `--drafter-fix per_byte` | **0.5700** | digit-exact vs bench 131's chosen-B cell |

Core rows byte-identical to the old knob-off runs (acc, macro_f1, brier, abstain 0.7445,
sel 511/0.5068, ECE 0.0192, floor 0.2778, oc_scale 0.25, score/distance thresholds) —
TRUE both. The new rule produces literally the same bytes the `--no-kn-route` run did:
typed is content-bound AT the default now, with the tuning axis intact.

### Cell 3 — G3 re-diff, full registry — PASS (resolved)

ONE full default run (`full_refit`) vs the on-disk `full_incumbent/results.json`, all 10
suites, core-row diff:

- **typed_decisions 0.5725 → 0.5630 — the only mover** (the intended posture change).
- **sst5 does NOT move** (0.3967 identical) — the fix, proven at the registry level.
- ag_news, banking77, code_fixtures, emotion, massive_intent_en, prompt_injections,
  semantic_defects, xnli_en — all digit-identical. **No third mover.**
- Cross-run determinism: `typed_refit` (single-suite) == `full_refit`'s typed core — TRUE.

### Cell 4 — perm typed at the new default — PASS

`perm_refit`: **0 RED cells, verdict PASS** — median 0.00 pt, max 2.98e-06 pt (the
standing L1-sum ULP grain), flips 31/60 (0.517, ties-only), control_invariant false (the
standing disclosure). **Every verdict cell AND every per-case row is identical to
bench 131's `perm_content`** (lane/model/control/n_cases 60/n_orderings 5/median/max/
flips/flip_rate/control_invariant/verdict/cases — all equal). The new default's typed
perm posture is the old knob-off perm posture, case by case.

Cosmetic anachronism disclosed: the probe meta's `route_binding` string still reads
"legacy k==N index alignment (incumbent)" at the new default — the string predates the
`4ad25df` rewording of the arm; the actual posture is the Score-only index-anchored arm.
One-line string fix for whoever next touches perm_probe.rs (not done here — no src/edits
in the measurement lane).

### Cell 5 — wanli_en unaffected — CONFIRMED (no run, per plan)

`build_wanli_en` read (suites.rs): 3 fixed options in XNLI_KEYS order
(entailment/neutral/contradiction), k = 3; `train_row_label`'s wanli arm returns the gold
STRING → the domains are named by the same three key spellings; `prepare`'s expected-key
pin is XNLI_KEYS. So **by-name resolves on every question** and the index-anchored arm is
Score-only anyway — the old blind fill never armed for wanli_en and the new law never
will. The sibling's 0.3100 baseline (`130_wanli_en_baseline`) is unaffected by the engine
change. Same shape as xnli_en, which has held the perm gate since the b3520a5 label-key
fix.

### Cell 6 — unit tests — PASS

`cargo test --lib`: **339 passed, 0 failed** — including the reworked
`kn_route_choice_is_content_bound_and_score_index_names_arm` (choice inert across knob
values + permutation-invariant; score + index-named domains arms at the default and
disarms at `--no-kn-route`), `kn_route_is_inert_when_by_name_resolution_arms`, and the
two drafter-fix byte-identity tests. (The old
`kn_route_off_is_permutation_invariant_and_on_is_position_bound` was replaced by the new
law in `4ad25df` — the count stays 339.)

### Follow-up provenance

- HEAD `4ad25df` (`feat(079): the index-anchored route arm…`), one rebuild
  (`cargo build --release --bin harness`, 1 m 31 s, green) used for every cell.
- Box state: the sibling compute was still on the box (load 8.9–13.7 across the cells;
  the standing preflight REFUSAL carries over — every latency figure above remains
  provisional; the verdict cells are byte-identity claims, load-insensitive).
- Artifacts: `.benchmarks/131_kn_route_ab/{sst5_refit,sst5_strict,typed_refit,
  typed_refit_pb,full_refit}/results.json` · `perm_refit/perm_probe.json`.
- Renumber note: this record is bench 131 (the sibling's `130_wanli_en_baseline` took
  130 first in `459d344`; dual_allocation_gate green after the renumber; `.highwater`
  = 131). The body sections above keep their bench-130-era measurements verbatim —
  every `130_kn_route_ab` path reference was updated to `131_kn_route_ab` in the same
  renumber.

```sh
# Reproduce the follow-up cells (4ad25df)
cargo build --release --bin harness
# cell 1
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --suites sst5 --out .benchmarks/131_kn_route_ab/sst5_refit
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --no-kn-route --suites sst5 --out .benchmarks/131_kn_route_ab/sst5_strict
# cell 2
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --suites typed_decisions --out .benchmarks/131_kn_route_ab/typed_refit
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --drafter-fix per_byte --suites typed_decisions --out .benchmarks/131_kn_route_ab/typed_refit_pb
# cell 3
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --skip-laya --nb-select --oc-select --ridge-select --out .benchmarks/131_kn_route_ab/full_refit
# cell 4
REFLEX_BENCH_HOST=m3 cargo run --release --bin harness -- --perm-probe --suites typed_decisions --out .benchmarks/131_kn_route_ab/perm_refit
# cell 6
cargo test --lib
```

## 4090-windows cross-host cell

The publish's cross-host leg: the SAME B-posture board cell re-run on the 4090
Windows box and diffed against `full_refit_pb/results.json` (the M3 cell above)
under the bench-096/097 two-host law — every suite's modelless hard accuracy AND
calibrated abstain must be bit-identical.

- 4090 state: synced via the scheduled gitsync (SYNC DONE 2026-10-08 22:33:57
  local); develop read `70f2b2c` — `f068ae6` plus ONE docs-only commit (issue-081
  doc line; zero engine-code delta). The run's meta records `git_sha: 70f2b2c`;
  the engine code is the `f068ae6` posture the M3 cell measured. Build: warm
  release, 52.45 s, green. Datasets present (`.raw/datasets`) — no refusal.
- Box state: **UNJUDGED** (standing disclosure — the 4090 posture carries no
  preflight; `REFLEX_BENCH_HOST=4090-windows` labels the meta). Every latency
  figure below is host-comparative only; the verdict is a bit-identity claim,
  load-insensitive.

Command (PowerShell over ssh, cwd `E:\git\riir-reflex`):

```powershell
$env:REFLEX_BENCH_HOST = '4090-windows'
.\target\release\harness.exe --skip-laya --nb-select --oc-select --ridge-select --drafter-fix per_byte --out .benchmarks/131_kn_route_ab_4090/results
```

Per-suite m3 vs 4090-windows (modelless lane, B posture `--drafter-fix per_byte`;
both hosts' values are EXACTLY equal — printed to full precision from both
results.json):

| suite | hard acc (both) | cal abstain rate (both) | cal sel-acc (both) | sel n (both) | verdict |
|---|---|---|---|---|---|
| typed_decisions | 0.5700 | 0.6320 | 0.574728 | 736 | IDENTICAL |
| ag_news | 0.8825 | 0.4900 | 0.950980 | 204 | IDENTICAL |
| emotion | 0.8850 | 0.4625 | 0.934884 | 215 | IDENTICAL |
| sst5 | 0.3967 | 0.5317 | 0.405694 | 281 | IDENTICAL |
| banking77 | 0.8420 | 0.4780 | 0.934866 | 261 | IDENTICAL |
| massive_intent_en | 0.7800 | 0.3100 | 0.768116 | 207 | IDENTICAL |
| xnli_en | 0.5233 | 0.5400 | 0.608696 | 138 | IDENTICAL |
| prompt_injections | 0.7672 | 0.6810 | 0.945946 | 37 | IDENTICAL |
| code_fixtures | 0.3750 | 0.3438 | 0.285714 | 21 | IDENTICAL |
| semantic_defects | 0.4412 | 0.5000 | 0.431373 | 51 | IDENTICAL |

**Bit-identity verdict: PASS (10/10).** Every suite's hard accuracy,
calibrated_abstain triple (abstain_rate / selective_accuracy / selective_n) and
cases_digest compare exactly equal (`==` on parsed values, both files in hand).
Disclosures, neither of which touches the gate:

- Latency fields (p50/p99/seconds/extremes) differ by host as always — excluded
  by the law's terms.
- Auxiliary continuous metrics (ECE/Brier/NLL/mean_confidence, the raw-readout
  ECE) drift at ULP scale (~1e-10 relative, worst ~1e-8 absolute) — fp
  summation-order across hosts on identical decision sets (the identical hard
  accuracy + selective_n prove the decisions did not move). Same class the
  perm-probe control calls the L1 fp envelope.

Provenance: artifacts `.benchmarks/131_kn_route_ab_4090/results/{results.json,
TABLES.md}` (4090) · the fetched twin `.benchmarks/131_kn_route_ab/results_4090.json`
(M3, compared against `full_refit_pb/results.json`). Run completed 2026-10-08
15:40:17Z. No commit from this lane — the coordinator owns the push.
