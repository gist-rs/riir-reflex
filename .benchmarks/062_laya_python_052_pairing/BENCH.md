# Bench 062 — the T5 data repair: py reference re-run at the 052 protocol

**Issue 040 T5.** The py laya reference re-run on the SAME stratified-052
sample as the rust lanes, in ONE run — every rust↔py pair now shares its
case population by construction (suite-level `cases_digest` covers both
lanes; one run, one tree). Numbered 062: 061 was dual-allocated in-flight
by the concurrent Issue-038 session (`061_cascade_lane`, same worktree,
same `.highwater` read — the same-box blind spot of
`dual_allocation_gate`, which compares checkout-vs-origin only); this
side moved, the cheaper direction, per the collision law.

## Verdict

**All 15 suites: rust laya accuracy == laya-python accuracy EXACTLY**
(macro-F1 identical to 15 decimals on every suite; brier differs only at
the 8th decimal — the py lane's own rounded-4 probability law). The three
suites the site counted as parity failures were sample difficulty, exactly
as Issue 040 predicted — NOT a port regression:

| suite | site showed (mismatched samples) | this run (same sample) |
|---|---|---|
| massive_intent_en | rust 0.6933 vs py 0.7500 | **0.6933 == 0.6933** |
| banking77 | rust 0.4220 vs py 0.4980 | **0.4220 == 0.4220** |
| code_fixtures | rust 0.6667 vs py 0.5833 | **0.6667 == 0.6667** |

Even the predicted "G5-class argmax wobble on the many-class suites" did
not materialize — exact equality everywhere, G5 parity is that good.

## Provenance

```
PROVENANCE: power=AC Power load=5.20 swap=1890.19M canary=138.9us/best5 powermode=2(high)
✓ preflight PASSED at the default MAX_LOAD=6.0 ceiling (bench_preflight.sh)
```

- Tree: ISOLATED WORKTREE at `6199e5e8` (`git worktree add --detach`, the
  develop HEAD of run time) — the sibling Issue-038 session was mid-edit on
  `src/harness/` at launch; the worktree excludes in-flight WIP by
  construction, and `code_fixtures` (whose cases are this repo's own source
  spans) reads the pinned tree. Built in an isolated
  `CARGO_TARGET_DIR=/tmp/reflex_t5_target`
  (`--features laya-riir,laya-riir-metal`), 36 s.
- Sibling path deps (`../katgpt-rs`, `../riir-infer`) resolved through a
  `.t5-parent/` symlink parent (worktree-sibling layout, removed after).
- Datasets: `.raw/datasets_t20k` (symlinked into the worktree, same bytes
  the 052 rust run served — no re-fetch since).
- Weights: `~/.cache/riir-reflex/laya` (the same flat cache both lanes
  verify); py interpreter system `python3`, torch 2.11.0, MPS; the lane's
  known temperature-clamp RuntimeWarning fired (disclosed upstream behavior,
  confidence-only, does not move argmax).

## What ran

```
LAYA_DEVICE=metal LAYA_PYTHON=python3 ./target/release/harness \
  --head-select --nb-select --laya-python \
  --datasets-dir .raw/datasets_t20k \
  --out /Users/katopz/git/riir-reflex/.benchmarks/061_laya_python_052_pairing
```

15 suites, no absences, `PASSED`. `determinism_ok` true on both laya lanes
on every suite. Wall: rust laya ≈ 297 s, py lane ≈ 363 s.

## Latency (secondary — the claim here is pairing)

py p50 runs ~1.5–2× the riir Metal p50 across suites (ag_news 34 vs 23 ms,
banking77 76 vs 48 ms, typed_decisions 362 vs 406 ms — the one suite where
py leads). Consistent with the lane's historical relationship (subprocess
IPC + torch overhead vs the riir Metal kernel ladder); the numbers were
taken at preflight-passed box state, ceiling 6.0, load 5.20 at launch.
Latency columns refresh; the pairing claim does not depend on them.

## Aftermath

Republish refreshes the modelless + laya(+py) classes site-wide from this
run; every rust↔py TL;DR pair carries the same `cases_digest` → the
`check_lane_pairing.py` gate passes with NO `PUBLISH_ALLOW_SAMPLE_MISMATCH`
ack — the 09-27 ack is retired, and the TL;DR reads `identical on 14/14`
(was `11/14` with 3 cross-sample pairs miscounted as parity failures).
