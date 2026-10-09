# Bench 136 — Issue 081 T2c: the fitted-token head EXECUTED (the pairwise functional) — and the T3 target refuted for the reflex arm

**Status:** EXECUTED — split verdict. The named re-open's ENGINE SURFACE is built, tested, and
works perfectly end-to-end (live 123/123 correct both depths). The T3 GATE TARGET is refuted:
the "zero-risk depth-1 stop" premise was a mock-arm artifact — on the reflex arm the gold lives
in beam waves 2-3 and stopping at depth 1 costs −15.6pt recall. The v5 content label is closed
on the count-table substrate too. Four durable mechanism laws recorded.

Run date: 2026-10-09 · reflex `develop` (post `9adc908` + this bench's commits) · serve:
`/tmp/reflex-t2c/release/reflex` (default features incl `nb_scope`) · box: m3, load 19-21 all
session (three sibling agents; latency NOT claimed anywhere in this record — recall/edges/confusion
cells are load-independent; determinism is byte-level).

## What was built (the engine surface — the "effort-gated new surface" from Bench 135)

1. **`nb_pair_scale`** (`EngineConfig`, opt-in `nb_scope`, default 0.0 = byte-identical): a
   by-name-resolved choice question's option `i` gains `nb_pair_scale · σ(in_{d_i} −
   max_{j≠i} in_{d_j})` — the option-set-scoped table margin in BITS, NO per-token
   normalization, no cross-domain max_other. Serve arming: `RIIR_REFLEX_NB_PAIR_SCALE`
   (with `RIIR_REFLEX_NB_SCALE` / `RIIR_REFLEX_RIDGE_SCALE` siblings; bad values refuse the
   boot; a feature-less build refuses loud — the flag-does-nothing law).
2. **Corpus loader verbatim directory names** (`corpus.rs`): a domain named after a wire
   option string (prose ending in `.`) survives byte-intact for by-name routing (`file_stem`
   silently stripped the trailing separator).
3. **`RIIR_REFLEX_SYSTEMONE_SPACED`** (opt-in, OnceLock — the G4 env law): the request state
   renders with Python-default separators (`", "` / `": "`), recursively, key order preserved.
4. **Corpus variants v7-v11** (`scripts/jevmem_corpus_build.py`): the by-name description
   domains (v7), paired-doc cancellation (v8), + neutral tails (v9), + spaced dialect
   everywhere (v10), + the v5 supervision labels unpaired (v11). `eval --samples fit|eval`
   enforces the selection discipline (Bench-127 law: scale selection on s0-4, the honest
   read on s5-9 fired once).

## The four measured laws

**L1 — the averaging law.** The shipped `σ(margin/n_tokens)` family (nb choice terms, noul
polarity `p = σ(margin/n)`, option-cond) cannot express O(1)-token evidence on O(1000)-token
states: a single discriminative token contributes O(10) bits to the table margins (measured:
`in_yes − in_stop` = **+11.9 bits (depth-0 requests) vs −56.8 bits (depth-1)** — cleanly
signed on every state), which the per-token average reduces to ±0.005-0.02 in σ-space. The
ridge is doubly blind: its top-2048-by-df feature selection drops class-exclusive low-df
buckets outright (`feats_binary_search(depth buckets) = None`) and its margins pin-steal
(−11 bits). Under L1-normalized option scores the family is ALSO scale-self-limiting: the
common σ component grows with the scale and cancels — measured **byte-identical p at
nb_scale 4096 vs 65536**. The pairwise form has no common component (a 2-option pair is
complementary — the terms sum to the scale), so its leverage is `2σ(Δ)−1`: full range at any
scale ≥ ~20.

**L2 — the collision law.** Paired docs (identical states in both domains, only the depth
byte differs) cancel query/evidence table mass exactly — but each class-EXCLUSIVE bucket
(the depth token forms + ~48 description-tail tokens/bigrams) carries ±22 bits, and a
~2500-event request hashes onto a wrong-side exclusive bucket with λ ≈ n·exclusive/VOCAB ≈
1.1: P(≥2 collisions) ≈ 31% — matching the measured 38% premature depth-0 stops on fit
states (v8). Neutral identical tails drop the exclusive set to the ~6 depth-form buckets
(λ ≈ 0.13): **v9 fit premature 38% → 1.7%** (13/760, the predicted Poisson tail).

**L3 — the burial law.** Their client serializes the stopping state with ALPHABETICAL keys
(`{"depth":0,"evidence":[…`) and compact separators — the depth digit is glued into the
first whitespace token, which ends inside the first evidence item's opening utterance text:
a per-state-unique chunk (`{"depth":0,"evidence":[{"content":"[Audrey]:`), unseen for
held-out states. Measured: v9 held-out collapsed to a coin flip (38% premature / 46% stop)
while fit states (whose chunks ARE the docs) read 98%. The spaced rendering un-buries the
digit (the chunk `0,` trims to the clean token `0`): **v10 held-out 776/776 + 776/776 —
100%/100%** (p_cont means 0.979 vs 0.020).

**L4 — the depth-limit law (the T3 refutation).** Their controller's loop is bounded by
`depth_limit = ceil(maximum_depth · routing.multi_hop_need)` — for the reflex arm's routing
values (multi_hop ≈ 0.22) that is 2-3 rounds, and the beam waves 2-3 carry the gold:
stopping at depth 1 (the head working PERFECTLY — live audit 123/123 depth-0 no-stop at
0.979, 123/123 depth-1 stop at 0.020) measured **s5 recall 0.7229 → 0.5673 (−15.6pt) at
200 edges/QA (base ~1700-2200, nodes 40 → 10)**. The Bench-134 "scan-2 never adds nodes —
frontier_exhausted 1540/1540" premise was measured on the MOCK arm's regime (mock routing
0.5 → different depth_limit; mock stopping 0.9/0.2) and does not transfer: routing answers
were byte-identical between the v6 and v10 runs (verified in the audits) — the recall loss
is purely the missing later beam waves. The T3 letter (base recall @ ≤1200 edges) is
unreachable by ANY stopping policy: the base is depth-limited by routing, not by stopping.

## The v5-label arm (the original T2b target) — closed

v11 (v5 supervision labels — stop-ok 92.6% / expand 7.4% — unpaired, neutral tails, spaced,
pair=50): held-out p_cont = **0.020 for every depth-0 state** — the class prior dominates
and no content signal separates the 7.4%. This is the stop-at-0 regime (Bench 134: 0.7417 @
~0 edges) — the count-table substrate CONFIRMS v5's inseparability finding through a
completely different estimator. The stopping lever on this controller has no reachable
Pareto win beyond the known corners.

## The honest Pareto (s5, reflex arm, per-QA means)

| arm | recall | edges | nodes | note |
|---|---|---|---|---|
| base (never-stop; routing-bounded) | 0.7229 | ~2000 | 40 | the standing posture |
| v10 stop@1 (the head, perfect execution) | 0.5673 | 200 | 10 | −15.6pt — L4 |
| stop@0 (v11 / t3_stop0 class) | ~0.74-class | ~0 | 10 | the v5 prior corner |

## Posture notes

- The pairwise head + the three serve knobs are **opt-in, default-off, byte-identical when
  unset** — pinned by tests (`by_name_nb_reads_a_discriminative_state_token`,
  `pairwise_count_table_reads_a_single_token_on_large_states` — the averaged-family negative
  arm is asserted IN the test, `spaced_state_rendering_separates_structural_scalars`,
  `directory_names_load_verbatim_including_dots`, the `scale_*` parse gates).
- The spaced rendering is opt-in (Benches 133-135's measured posture is compact). Its
  collateral on the routing read is small (mh_acc 0.818 held-out vs 0.817 compact) and the
  traversal pin re-calibrates under the same dialect — but full adoption is a re-baseline
  decision, not a default.
- Determinism: the v10 s5 run reproduced bit-identically across two full pipeline passes
  (incl. the `--max-latency 60` re-run — identical recall to 16 decimals, 492 audit
  decisions identical).
- The 7331 serve is left DOWN (the lane's rig restarts it: `RIIR_REFLEX_CORPUS=<corpus>
  RIIR_REFLEX_NB_PAIR_SCALE=50 RIIR_REFLEX_SYSTEMONE_SPACED=1 … reflex`).
- Rig preserved: `.raw/locomo/corpus_pack_v{7..11}`, `t2c_v10/s5`, all `*_eval.json` files,
  captures s0-9, the t2b/t3 run dirs.

## What remains (named)

- The stopping lane is CLOSED on this substrate (L4 + the v5 closure). The head machinery
  stays available for any future O(1)-token policy surface (the engine surface ask of the
  re-open is DISCHARGED).
- T4 (LoCoMo answer-model spend) stays owner-gated. The 623 T2 teacher capture stays
  owner-gated.
