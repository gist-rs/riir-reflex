# HISTORY.md — riir-reflex

Durable records for closed issues (the noise-reduction rule: a resolved
issue file is removed from `.issues/`; its record lands here, hash-pinned).
A removed file's full life: `git log --follow -- .issues/<file>`. Open work
lives in `.issues/` and `.plans/`, never here.

## 2026-09-27

- **Issue 042 CLOSED — the gate rate axis measured (Bench 066,
  `842f89d`): the COMBINED posture (`--gate-fit-selection
  --gate-distance-only`, worthiness margin 0.16 unchanged) is the first
  that passes the issue's full T4′ acceptance.** Three full 15-suite runs
  (the 063 lane posture ± the lever flags; artifacts
  `.benchmarks/066_gate_rate_axis_levers/{lever2_only,lever1_only,both}/`).
  Lever 2 (distance-only: score threshold pinned 0.0) CONFIRMS the
  transfer finding — the corpus-distance axis lands 31.5–49.6% test-side
  abstain on every armed topical suite (the 061 90–99% pathology is a
  score-axis property) and xnli +0.1167 / ag_news +0.0325 recover at sane
  rates — but arms sst5 on a probe reading +0.2000, EXACTLY equal to
  ag_news's +0.2000 (no margin separates them), and the armed sst5 flips
  −0.0133 on test: the arm-side flip class now measured on BOTH probe
  families (fused +0.072 → −2.2; distance +0.20 → −1.33). Lever 1
  (selection-slice threshold fit) is NULL alone — every verdict repeats
  the shipped posture, xnli's gain halves — and is the combination's
  FIXER: at BOTH, sst5's distance threshold shifts, its probe reads
  +0.1341 < 0.16, the suite disarms to modelless-exact, and the loss is
  gone. BOTH rows: typed +0.1685 @ 49.6% escalation, ag_news +0.0300 @
  37.5%, xnli +0.0600 @ 15.7%, every other dataset suite reads modelless
  EXACTLY — cascade ≥ modelless everywhere, escalation inside [15%, 60%]
  (the shipped fused posture's typed 96.7% fails this window — cascade
  0.7430 vs laya-typed alone 0.7445 at near-full escalator cost, which is
  why the window exists), zero regressions, and the fixed 0.16 margin
  holds 8/8 on the shifted probe sets (second independent probe family
  where (0.150, 0.288] separates). The recommended lane command
  (AGENTS.md) carries the combined posture; library defaults stay OFF;
  the shipped fused command keeps its 063@0.16 record (higher headline
  +0.394 total). Latency PROVISIONAL (preflight REFUSED, load 7.93,
  sibling session); accuracy pick-count/load-immune — forced rows
  byte-match the published state (ag 0.8825 / xnli 0.5233 / emotion
  ridge@8 0.8850). sst5 is a twice-measured arm-side flip across both
  probe families: a third flip anywhere files a per-suite probe-size
  floor, not a margin change. T4′ promotion to any DEFAULT surface stays
  a separate owner call (the lane is opt-in). Issue file removed per the
  noise-reduction rule; this row + the bench doc are the record.

- **Issue 043 RESOLVED (measured, `37cb732d` + `44bb7cbd`) — the
  prompt_injections polarity gap does not exist as a slice problem, and
  the gain does not exist through the shipped term.** The issue asked for
  a discriminating cal slice; measurement split the premise: (a) the
  positional-prefix hypothesis is REFUTED as dangerous —
  `scripts/issue043_slice_probe.py` shows the train mirror is NOT
  label-grouped (injections from index 4) and positional prefixes FLIP
  the preferred polarity by region (prefix100 → nb1, prefix200/300/400 →
  nb0, the WRONG posture, mirror-test 0.32–0.34), so raising the
  positional cal_cap (candidate 1) would have been actively harmful; (b)
  the stratified lanes (`--nb-select`, `--cal-select-cap`) ALREADY
  select on `stratified_selection_slices` — on that slice the polarity
  candidates separate cleanly (off 0.5000 / Some(0) 0.3900 / **Some(1)
  0.6100**, stable across every scale — the noul pick is the margin's
  sign, scale-invariant — and across the α axis, observed-laplace 0.61
  vs fixed-1 0.51); selection picks `scale 1 α observed-laplace noul-yes
  Some(1)`, clearing the house margin; (c) the SINGLE TEST READ at that
  posture is **0.7672 — byte-identical to the shipped row** (run at
  `/tmp/reflex_issue043_run`, HEAD `767c577`, isolated worktree: the
  sibling's in-flight manifest edits blocked the main checkout); (d) the
  corpus-cap axis is FLAT (every cap 8..512 reads 0.5000 on the slice;
  `/tmp/reflex_issue043_run2`). The 0.8362 probe datum is unreachable
  through the shipped mechanism: it came from plain MNB argmax WITH
  priors over the full train corpus, while the engine's noul polarity is
  σ(margin/n_tokens) of one-vs-rest in-scope log2-odds with no prior
  term, pick = margin sign; at the engine's reachable corpus (≤64/label)
  even the mirror's MNB reads only 0.63–0.65 — below the shipped 0.7672.
  Reopen path recorded in the issue (removed per the noise-reduction
  rule; this row is the record): a `noul_full_posterior` term + an
  NB-table-only corpus rise, expected gain unproven and possibly
  negative. Probe-mirror validation: full-train MNB reads 0.8017 ≈ the
  038-POC's recorded 0.802.

- **Issue 038 CLOSED — every lever in the ranked plan has a verdict; the
  remaining gap belongs to the model-based lane (T4 → riir-instinct).
  Final probe verdicts (T7c/d, `e36b7e09`):
  `scripts/issue038_t7cd_probe.py`, cal-selected / test-once, the base
  probe's `src/embed.rs` tokenization.** (c) **BM25-kNN vote REFUTED** —
  Okapi k1/b × k × vote-γ ladders, cal-selected; test reads BELOW the
  shipped lane on every suite: banking77 0.7809 vs 0.8620, emotion 0.5375
  vs 0.8550, sst5 0.3217 vs 0.4017, massive 0.7226 vs 0.8267, ag_news
  0.8525 vs 0.8975. (It beats the bare-MNB probe reference on
  banking77/massive — the neighbourhood signal exists — but the shipped
  lane's genome + ridge arms sit far above it, so the lever premise fails
  at the shipped bar.) (d) **Hebbian premise×hypothesis co-occurrence
  REFUTED as a lever** — per-class unigram co-occurrence tables,
  contrastive log-odds weights, fused MNB + λ·bilinear, top-k df vocab,
  cal-selected: the term is REAL signal on the bare namespaced bag (test
  0.3867 → 0.4767, +9 pt) but on the SHIPPED pair-view shape the cal walk
  selects **λ = 0** (every λ > 0 posture below the reference: 0.5167 <
  0.5367, consistent 8/8). Probe-mirror validated: the shipped-shape MNB
  reads cal 0.5367 / test 0.5100 against the Rust hold-out .505 and
  published 0.520. The mechanism: the shipped novel-word mark IS a coarse
  premise×hypothesis interaction ("hypothesis word absent from premise")
  and the coverage/negation buckets harvest the refinement — the
  co-occurrence table adds nothing measurable. With the T5 genome having
  held xnli at every coordinate, the xnli residual is confirmed
  representational AND already-harvested: lexical NLI tops out near 0.52
  on this feature class (the issue's own honest ceiling); the rest of the
  gap to laya is the model-based lane's. Landed over the issue's life:
  T1/T1b/T2/T3/T6 (Bench 051, nb_scope default-on), T7(a)+(b) (Plan 004 /
  Bench 057: option_cond + nb_ridge default-on), T5 (Plan 005 / Bench 064
  `bc0a3fb`: the joint blend-genome lane; banking77 0.8260 → 0.8620, sst5
  0.3967 → 0.4017 ADOPTED; massive +5.33 REJECTED on G1; emotion
  REJECTED; RRF DECLINED), T4′ cascade lane (Bench 061 `337974d`, gate
  NEGATIVE → issue 042), ag_news volume lever (Bench 065, NEGATIVE).
  Scoreboard at close: modelless ≥ laya best on 5/9 dataset suites,
  banking77 margin +36.2; xnli 0.520 / typed 0.4655 recorded as the
  hashed-bag class's representational ceilings. Site republish still
  deferred to a gate-clean whole-run (`publish_bench` has no G1 filter).

- **Issue 041 RESOLVED — the ag_news full-pull volume lever measured
  NEGATIVE (Bench 065).** The full 120k train pull
  (`.raw/datasets_agnews_full/`, a measurement sidecar — the canonical
  t20k basis is UNCHANGED, no protocol column, no republish owed) moved
  ag_news modelless accuracy **+0.25 pt** (0.8825 → 0.8850) against the
  expected 0.90–0.92; the corpus-cap axis moved **exactly nothing** (cap
  64 == cap 1000 at full volume — identical accuracy, +17× p50 cost,
  provisional latency); and the T5 joint blend-genome walk over the
  full-volume posture ended **HELD** (no volume × posture interaction).
  The T2-POC volume law did not transfer because the engine changed —
  T7's uncapped count tables already dominate the pick at the t20k
  volume, and ~1000 docs/label saturates the unigram signal. The
  remaining −6.5 pt gap to laya (0.9500) is not volume-reachable on any
  measured axis; the issue's fallback (word-order/disambiguation the
  encoder owns) is the only live hypothesis on this suite. 041's
  non-lever note (prompt_injections' selection slice cannot see the
  pure-NB posture: 0.8362 probe vs 0.7672 shipped) filed forward as
  issue 043. Tooling rider: `scripts/fetch_datasets.sh` gained a
  `SUITES` filter (a 120k TRAIN_CAP without it would pointlessly pull
  ~120k xnli rows); datasets-server limiter measured at ~300 pages per
  burst sustained (4 burst→cooldown→resume cycles; the skip logic
  resumes exactly). Byte-identity law verified 204/204 shared pages;
  full per-file digest table + fold digest in the bench record.
  Record: `.benchmarks/065_agnews_full_volume.md` + the three run dirs.

- **Issue 038 T5 steps 2+ LANDED — the joint blend-genome lane
  (`--genome-select`, Plan 005 / Bench 064).** `{route, head, nb(+α,+view),
  oc, ridge}` as one genome line, coordinate descent on the cal slice with
  build-once-fitted + `DecisionEngine::set_blend_scales` (fail-closed
  `ScaleNotFitted`; a coordinate eval is one sel-slice scoring pass, not a
  rebuild), acceptance-vs-seed bar `--genome-accept-margin` defaulting to
  the house 0.05, test read once at the walk end. Round 1 (house bar): ALL
  HELD and every held row reproduced the published state byte-exactly (G3
  proven end to end); the walks still surfaced +3.0..+4.0 cal interactions
  the greedy ladders could not reach (ridge@16 included). Round 2
  (pre-registered refinement bar 0.03): **banking77 0.8260 → 0.8620**
  (ridge@1 + head off) and **sst5 0.3967 → 0.4017** (ridge@8) ADOPTED, both
  G1-passing; **massive REJECTED on G1** (+5.33 accuracy with ECE .2383 vs
  floor .1361 — the UQ-bearing law binding: an accuracy gain that breaks
  the calibration floor is a failed gate); **emotion REJECTED** (the walk's
  nb-off posture read 0.35 on test vs published 0.885 — the house bar would
  have refused it; hypothesis on record: the ridge fit overfits its own
  pool distribution, the count tables generalize). RRF DECLINED on measured
  rationale. Site republish deferred to the post-041 whole re-run
  (`publish_bench` has no G1 filter). Note (same-day): the 041 full-pull
  question was answered NEGATIVE (Bench 065 — the volume lever buys
  +0.25 pt), so that deferred whole re-run happens at the CANONICAL t20k
  basis with the genome winners, not at a new corpus volume. Two
  protocol costs disclosed: the
  0.03 refinement round spends 4 more test reads than the house bar, and
  its one catastrophe is the price of the 3 wins — the bar value is now a
  measured trade-off, not a convention.

- **Issue 042 T3 DECIDED (a) — fixed escalation-worthiness margin 0.16, the
  recommended cascade posture (acceptance re-run
  `.benchmarks/063_cascade_worthiness_margin016/`, 10/10 PASS).** Why (a)
  over the per-suite cal-fit (b): the failure being fixed is the cal→test
  SIGN FLIP on small positive probes — a cal-split fit re-estimates from
  more cal data (variance ↓) but cannot observe a cal→test shift (test
  never enters any fit), nothing measured separates ag_news's cal
  distribution from sst5/massive's, and MAGNITUDE is the one measured
  discriminator — the fixed bar implements exactly that; (b) also touches
  selection-adjacent code near the sibling's T5 genome work and its
  ag_news-retention clause is unguaranteeable from cal-only fitting. The
  re-run (exact 063 command + `--cascade-worthiness-margin 0.16`; preflight
  REFUSED on load 6.08 — sibling session — accuracy pick-counts load-immune
  as in 063): cascade ≥ modelless on every suite; typed·typed armed
  (+0.2881 → test +0.2770) and xnli armed (+0.4000 → +0.1167) keep their
  gains; every other suite — including sst5 (+0.0718) and massive
  (+0.1500), both under the bar — disarms and reads modelless EXACTLY (the
  two 063 FAILs gone); ag_news (+0.1316) disarms, its +6.75 the recorded
  price. All probe deltas byte-identical to 063 (G5 determinism end to
  end). Library default stays 0.0 (neutral arm-at-parity); the AGENTS.md
  lane command carries 0.16. Default-promotion trigger: a second
  independent lane run reproducing 10/10, ideally post-levers-1–2. (b)
  reopen trigger: a future lane run where a fixed-bar ARM-side row flips on
  test.

- **Issue 042 lever 3 LANDED — the per-suite cascade escalation-worthiness
  gate (`--cascade-worthiness`, Bench 063).** The `--cascade` lane's
  escalation was direction-blind (Bench 061: banking77 −14.0 pt while xnli
  +11.7 — the gate escalates every calibrated abstain regardless of whether
  the escalator reads better there). The lever: the checkpoint answers the
  suite's CAL-slice questions the calibrated gate abstained on (protocol-
  legal selection-slice work — cal never enters any fit here, test never
  enters the probe), and the suite's escalation stays armed only where the
  probe reads ≥ the forced modelless picks on that set
  (`--cascade-worthiness-margin <F64>`, default 0.0; negative probe →
  disarm, disclosed in the row + a TABLES.md sub-table; missing cal
  records / thin support < 16 stay armed with a named `unprobed_reason` —
  the probe never invents a disarm it cannot measure). Shape: pure
  composer (`src/harness/cascade.rs` WorthinessInput/Verdict), modelless
  half = the deployed calibrated gate re-evaled over cal in
  `run_modelless`, escalator half = untimed cal serving in
  `run_laya_checkpoint` (one rendering law shared via the extracted
  `laya_prob_pick`), 8 new module tests (lib 140/140), clippy clean at
  default/laya-riir/no-default/all-features, G3 flag-off rows carry no
  worthiness key (byte-shape = 061). Measured (Bench 063, 8 dataset
  suites at the 061 posture): all four 061 losers disarm to exactly the
  modelless row (emotion −28.3, banking77 −14.0, typed-en/multi −8.4/−9.2,
  prompt −2.6 → all 0.000); the two small-magnitude armed flips remain
  (sst5 −2.2, massive −3.0, the SAME amounts 061 had); arm-side sign
  reliability is magnitude-gated (≥ 0.288 → 2/2, 0.072–0.150 → 1/3),
  disarm-side is 5/5 — the measured arm-bar gap (0.150, 0.288] holds the
  margin call (issue 042 T3). Latencies provisional: preflight REFUSED
  (canary 195 µs vs 141, load 5.49, swap 1.9 GB — no GPU compute consumer
  found; accuracy gates are pick-counts, load-immune by G5
  determinism).

- **Issue 040 RESOLVED — the lane-pairing population guard (T1–T5) complete;
  the site's `identical on 11/14` was a sample mismatch, not a port
  regression (guards `448f652` + `b30f5b1`; data repair Bench 062).** Root
  cause (measured): the 09-27 republish refreshed the rust lanes to the
  052 STRATIFIED protocol while the python comparison lane survived on the
  old first-N sample (`9dbdca5`) — on 3 suites (massive_intent_en,
  banking77, code_fixtures) the case sets genuinely differed, and the
  TL;DR counted different question sets as parity failures. The guard:
  `cases_digest` (stable FNV-1a64 over the canonical serde_json of the
  served cases, dependency-free — harness must compile at
  `--no-default-features` where blake3 is absent) stamped per run, carried
  per lane cell by the publisher, paired on identity in the TL;DR (three
  states — same / different-sample-disclosed / excluded, never pooled),
  and `check_lane_pairing.py` fail-loud at publish with an opt-in
  `PUBLISH_ALLOW_SAMPLE_MISMATCH` ack that reds when stale. T5 (Bench 061,
  isolated worktree at `6199e5e8`, preflight PASSED at the default 6.0
  reds when stale. T5 (Bench 062, isolated worktree at `6199e5e8`,
  preflight PASSED at the default 6.0
  ceiling — load 5.20, canary 138.9 µs): py reference re-run at the 052
  protocol in the SAME run as the rust lanes — **all 15 suites read exact
  accuracy AND macro-F1 equality rust == py** (massive 0.6933==0.6933,
  banking77 0.4220==0.4220, code_fixtures 0.6667==0.6667); even the
  predicted G5-class argmax wobble did not materialize. Ack retired (no
  env set at republish — the stale-ack ratchet forces its removal now that
  every pair matches); TL;DR reads `identical on 14/14`. Latency note: py
  p50 ≈ 1.5–2× riir Metal across suites (subprocess IPC + torch vs the
  Metal kernel ladder) — consistent with the lane's historical
  relationship, pairing-independent.
  relationship, pairing-independent. ⛔ Numbering note: the run dir was
  allocated 061 and RENAMED 062 pre-commit — the concurrent Issue-038
  session had taken `061_cascade_lane` from the same `.highwater` in the
  same worktree (the same-box blind spot of `dual_allocation_gate`:
  checkout-vs-origin only, sibling untracked WIP invisible). Renumber +
  citation rewrite per the collision law; highwater bumped to 062.

- **Issue 038 T7(a)+(b) LANDED — the two biggest modelless gaps closed
  (Plan 004, Bench 057, commits `0abb222` T7b + `de67f50` T7a); the issue
  stays OPEN for T5/T4′/T7(c)/(d).** The option-conditioned count tables
  (`option_cond`, promoted default-on) key one contrastive table per
  (question id, gold option) from TRAIN gold events — the typed suite's
  state-side signal the domain-level tables can never arm on (k ≠ N on
  every question shape there): typed_decisions **0.33 → 0.4655** (+13.5 pt,
  cal-selected oc@2). The NBSVM closed-form ridge readout (`nb_ridge`,
  promoted default-on) — binary-presence features over the same hashed
  lexicon scaled by the per-class NB log-count ratios, one one-vs-rest
  `ridge_solve_direct_f32` per class — emotion **0.7375 → 0.8475** (+11.0
  pt, cal-selected ridge@2, calibrated ECE 0.0 vs conformal floor 0.204).
  Every other suite byte-identical (G3), p99 ≤ 50 µs and `solve_into`
  alloc-free with both levers armed (G2/G4), det ✓. The cross-host
  bit-identity claim was RE-PROVEN at the new posture before the site
  publish (the publish gate refused the one-host state, exactly as
  designed): 4090-windows re-ran the full protocol (datasets 974/974
  sha256-verified, `REFLEX_BENCH_HOST=4090-windows` — the unset label
  read "unknown", the Issue-033 class again) — **14/14 modelless rows
  identical to the M3 to full precision**, and the 4090's own re-run
  reproduced its numbers exactly. Site republished (typed + emotion move
  on both hosts; every other row unchanged). Measured lessons recorded in
  the module + bench: an unsorted `binary_search` vec silently misses
  (hits=1/33, 0.29 vs the numpy replica's 0.8925 — the probe's Python dict
  cannot have this bug class); the probe's `tot[lab]` was a Python
  missing-key read (0), not a total — the validated arithmetic has NO
  per-class totals and "fixing" it breaks the denominator (loud non-finite
  refusal fired); and `/n_tokens` is the NB convention, not the ridge's —
  damped margins are O(0.1) and do not grow with tokens, so the blend
  divides by a fit-time self-calibrated temperature (mean per-doc class
  spread, strided 512-doc sample) instead.

- **Issue 019 CLOSED — the CLM T4 T-Rex re-run under OUR protocol (Bench
  059, `.benchmarks/057_clm_trex_4090/`, this commit); issue file removed
  (noise-reduction) with all seven tasks done.** Upstream had force-pushed
  `main` and stripped `examples/t_rex/` from its tree — the pin commit
  `cca045ff` is no longer reachable from `main` but GitHub still serves it
  by SHA, so the pin held and the harness ran FROM THE PIN (re-verified:
  `git fetch origin cca045ff…` succeeds; their current `main` is 8 commits
  with no T-Rex). The window (4090-windows, serialized, 027 laws — util
  0.72, one fixed warmup first, same-box latency law): vLLM
  `0.29.1rc1.dev397+ga8d1aa9c9` (the SAME nightly image as Bench 034) over
  a fresh `Qwen/Qwen3-8B` snapshot + `clm-serve` over the head (sha256
  `b2b4a8c9…` pinned in the record). **Determinism pin GREEN 8/8
  byte-identical, twice** (boot + resume invocation). Cells (ours, never
  theirs): shield-on **5/5 survive / 0 deaths / 697 ×5** — the headline
  reproduces; decisions 2441.6 vs their 3341.8; agreement 0.749 vs 0.658;
  latency p50 median 48.9 ms vs their 16.5 — and the record publishes the
  COLD→WARM trajectory their table hides: seed 0 pays 66.6 ms, seed 4
  (arena warm) reads 16.7 ms p50 / 3.4 ms model-side ≈ their headline, so
  the vendor 16.5 is the WARM steady state of the VectorArena (post-run
  `/health` hit_rate 0.9749). The no-shield secondary: **0/5 survive, 23
  deaths, mean score 201.6** at warm-state 16.7 ms — the shield is
  load-bearing and the survival number measures the combined system, their
  own convention. T-Rex cells are bench-record-only by design (not `/bench`
  lane columns — those come from OUR harness). Two live specimens of the
  katgpt-rs console-encoding class on the way: their harness `log()` prints
  a U+00B7 detail string and dies on this cp874 console
  (`PYTHONIOENCODING=utf-8` for the child — no source changes to their
  tree), and then the window driver itself died the same death one layer up
  (its redirected stdout inherited cp874) — both fixed posture-side, the
  workspace rule about `subprocess` encoding now has a third measured
  surface. Stack torn down after the window (container removed, VRAM back
  to 517 MiB); `.raw/` scratch (weights, clone, venv) cleaned in this
  commit.

## 2026-09-26/27

- **Issue 033 — the PAW comparison lane CLOSED — issue file removed (noise-reduction);
  record hash-pinned here.** ProgramAsWeights (compile-a-classifier category) measured on
  the same harness instead of quoting their marketing tiers. Arc: filed from the PAW
  distill → Posture A hosted lane + specs + first cells `2e9f351` (Bench 049: default
  mapper compiler, hosted-anonymous; banking77 refusal-dominated 73% — the honest table
  WAS the finding) → ft-tier cells measured TWICE independently (Bench 055 M3 stratified
  hand-derived `4cef66b` + Bench 054 4090 full-pull; every direction agreeing, the tier
  claim TRUE on every suite) → Posture B local llama.cpp runtime `7f9c2f5` (Bench 056:
  accuracy-neutral vs hosted, determinism-positive 4/4) → **arena republish + close
  (this entry):** the site's publisher grew the `paw`/`paw_local` lane classes, the
  `shikuwa`→`4090-win` host alias, and the `PUBLISH_BENCH_LANES` lane-scoped update
  filter with `:acc-only` latency suppression (reflex-site `dffabe3`, deployed live) —
  because the PAW docs' by-product modelless controls are the UNCALIBRATED baseline
  posture and must never publish over the calibrated same-law rows (or trip the drift
  gate over a posture difference); a fresh same-law hosted ft run (Bench 058,
  `de67f50`, stratified 400/400/600/500) mechanized 055's hand-derived anchor — ag
  0.7900 / emotion 0.5000 / sst5 0.3950 / banking77 0.4200 @ 34.6% refusals, matching
  055 within noise — and 056's local cells published beside it. **Posture record (which
  cells published):** `paw (hosted)` = ft-bs48 tier, hosted-anonymous, Bench 058,
  accuracy-only (end-load box state made latency NOT QUOTABLE — the `:acc-only`
  disclosure); `paw (local)` = ft-bs48 tier via their llama.cpp runtime, Bench 056
  (stratified, det 4/4, latency cells published — measured on the idle 4090). The
  default-mapper hosted cells (049) and the first-N ft cells (054) stay bench records,
  never published (different sample law). Deferred: a latency-quotable hosted re-run
  (publishes by removing the `:acc-only` suffix). Full life:
  `git log --follow -- .issues/033_paw_comparison_lane.md`.
- **Issues 036 + 037 + 039 CLOSED — issue files removed (noise-reduction);
  records hash-pinned here.**
  - **036 — the drafter-only path is degenerate on dynamic option spaces.**
    T1 landed the disclosure (`Slot.drafter_only` + the `; drafter-only=N`
    routing-reason suffix — any wire caller can see the weakest-scorer
    posture); T2 measured all four `EngineConfig::drafter_fix` candidates
    on the cua VALIDATION split and ALL fail the constant-skip floor (best
    `ncd` 11.17% vs 52.22%) — nothing promotes and the test split stays
    unread per the gate's own read-once rule; T3 closed the gate NEGATIVE.
    `drafter_fix` stays an opt-in measurement knob with `ncd` the recorded
    best base; the disclosed-abstain path is the product mitigation.
    Closed at `b092869`. Full life:
    `git log --follow -- .issues/036_modelless_dynamic_option_length_prior.md`.
  - **037 — the e8 table converter arm (riir-infer Plan 612 reflex half).**
    T1–T4 landed at `6535b75`: converter `--table-precision e8` + manifest
    rows + conversion-log entries; 3 sidecars emitted local-only under the
    repo's artifact rule (+ the `.gitignore` row); determinism asserted
    in-run AND cross-run; the six `.mlpackage`s byte-identical throughout.
    The RUNTIME half (`gather_e8`, `LAYA_ANE_TABLE=e8`, the G5-ANE re-pass)
    stays riir-infer Plan 612 Phases 2–3. Full life:
    `git log --follow -- .issues/037_ane_e8_table_converter.md`.
  - **039 — the 4000-row train cap truncates the label universe on
    label-sorted mirrors** (banking77 32/77, massive 42/60; modelless rows
    inflated). Arc: filed `087856f` → Bench 051 full-pull posture
    `fd3f3d3` → the stratified split + corpus-fallback guard + the
    readout-selection lever demoted NEGATIVE at `a2353e2` (Bench 052:
    modelless ≥ laya 5/8 suites at the representative sample; the
    wide-label G1 gap closed via the stratified cal slice instead;
    emotion's 051 pass explained as a sampling artifact) → T5 both-hosts
    identity `d7c0d4f` (row below; the banking77 cuda repeat-check flag
    filed as riir-infer Issue 021 and RESOLVED same-day — riir-infer
    `c64d0b1` chain-cache stale-slot eviction fix, det true ×4 on the
    re-run, in-repo instrument + note `e6f7108`). Records:
    `.benchmarks/051_nb_count_tables/` + `.benchmarks/052_stratified_readout/`
    (+ `results_4090.json` / `TABLES_4090.md`). Full life:
    `git log --follow -- .issues/039_train_cap_truncates_label_universe.md`.
- **Issue 039 T5 — the 4090 re-run at the Bench-052 protocol: both-hosts
  identity LANDED** (2026-09-27; record: `.benchmarks/052_stratified_readout/`
  §"4090 re-run" + `results_4090.json` / `TABLES_4090.md`; comparator
  `scripts/compare_052_4090.py`). The queued "bytes already copied there"
  premise was STALE (the box had the old `.raw/datasets` pull; its own
  re-fetch had died on curl 429s) — `datasets_t20k` copied fresh and
  byte-VERIFIED (977/977 files, sorted SHA256-manifest diff empty) BEFORE
  the run. Harness `--features laya-riir,laya-riir-cuda` at `634093f` in an
  isolated worktree on the 4090 (the checkout carried sibling WIP —
  worktree exclusion, not stash), launched via the Bench-047 schtasks
  one-shot pattern; 15/15 suites exit 0. Verdict: **modelless decision-level
  bit-identity 14/14** (accuracy/macro-F1/confusion/head+nb selections/
  thresholds/G1 exact; `code_fixtures` re-verified at the SAME snapshot
  `634093f` on the M3 after its 052-record mismatch was explained as input
  drift — its cases are this repo's own source spans, and the M3 record's
  tree predates `b092869`); **one raw statistic at ulp, 5/14 suites**
  (`readout_ece_raw` Δ ≤ 4.5e-10, the NEON↔AVX reduction-order class — no
  pick moves, calibrated ECE exact everywhere; disclosed, not pooled into
  the claim); **laya accuracy identical metal↔cuda 8/8** with ONE flag:
  banking77 cuda repeat-check `determinism_ok = FALSE` (first-10
  double-answer render differs on ≥1 repeat; every other suite True;
  shape-scoped: the ~317-token 77-way suite) — filed to the lane owner as
  riir-infer Issue 021. Latency indicative (UNJUDGED box state, the
  macOS-only Issue-021 probe posture): banking77 cuda p50 23 ms vs M3 metal
  47 ms.
- **Issue 033 Posture B LANDED — [Bench 056](.benchmarks/056_paw_local_full4090/BENCH.md):
  the PAW LOCAL llama.cpp runtime cells, full-N, det 4/4 ✓** (`--paw-local`,
  `src/lanes/paw_local.rs` + `scripts/paw_local_lane.py` +
  `scripts/paw_preload.py`, 8 in-module tests): the SAME compiled programs
  the hosted cells measured, answered through their local runtime
  (`paw.function` over programasweights 0.4.10, llama.cpp auto-CUDA) as a
  Python subprocess oracle — the lane never compiles, the program id comes
  from the hosted lane's cache, so the posture delta is measured on
  identical artifacts. ag_news 0.8000 · emotion 0.4875 · sst5 0.4167 ·
  banking77 **0.4120** (refusals 35.4%, answered-acc 0.6378); accuracy-neutral
  vs hosted at the same law (−1.3..+2.3 pt vs [Bench
  055](.benchmarks/055_paw_ft_bs48_m3/BENCH.md)) and determinism-positive
  (4/4 byte-identical repeats — the hosted tier cannot promise this);
  p50 114–264 ms local vs ~930 ms hosted round-trip. Two measured traps on
  record: the **uv-venv trampoline stdin deadlock** (a uv venv
  `python.exe` spawns the real interpreter as a child while holding its own
  copy of the stdin write handle — close-then-wait deadlocks the reap;
  Drop kills-then-waits) and the **cache-key posture** (`PAW_COMPILER`
  unset + a single cached program auto-selects loud; ambiguity refuses
  naming the tiers). ⚠ Sample-law disclosure in the record: this run is
  Issue-039-T2 stratified; 054 ran pre-`a2353e2` (first-N law) — the
  same-law hosted anchor is 055, and this box's modelless control column is
  pull-limited (the 429-wall fetch carries fewer train rows than the M3's
  `datasets_t20k`), a different pull, not a regression.
 86a0c03 (feat(033): PAW Posture B lands - the LOCAL llama.cpp runtime cells, full-N, det 4/4 (Bench 056))
- **Issue 033 ft-cells: cross-box CONFIRMATION — [Bench
  054](.benchmarks/054_paw_ft_cells_win4090.md)** (the 4090 Windows box, full
  test pulls at the 049 caps, hosted anonymous, run in parallel with the M3
  session's [Bench 055](.benchmarks/055_paw_ft_bs48_m3/BENCH.md) — the Issue-825
  twin-repair, not a duplicate: different sample, every direction agreeing;
  ⚠ the M3 record was RENUMBERED 053→055 on 2026-09-27 — the two sessions
  dual-allocated 053 the same day (`053_e0_evidence_density` is the earlier
  allocation, `299a3af`, and keeps 053; the paw record's `5cf57b1` moves).
  ft-bs48 strictly beats the mapper tier on all 4 suites: ag_news
  0.7825→0.7925, emotion 0.4750→0.5025, sst5 0.3283→0.3933, banking77
  0.1400→**0.3940** (refusals 73%→42.2%, answered-acc 0.6817). Compile ~12
  s/suite (the issue's 2–5 min did not materialize on this tier/box); det
  4/4 ✓ this run (recorded per run-date). Datasets fetched fresh through the
  HF 429 walls (cooldown + re-run; mteb/banking77 test = 3,076 rows / 31
  pages — not the 10.4k `cap=all` suggests). Modelless baselines
  byte-identical to 049's (same pull, same caps), so the PAW columns are
  apples-to-apples with 049's mapper rows.

## 2026-09-26

- **Issue 020 CLOSED — the riir Metal lane beats the python torch MPS
  oracle on every published cell, p50 AND p99** (owner directive 2026-09-24:
  *"make rust faster as it should in all cost"*). Final evidence
  ([Bench 050](.benchmarks/050_t13_mps_gemm/BENCH.md)): the paired per-suite
  A/B reads **9/9 p50 + 9/9 p99 wins** (−26…−44% p50), and the same-run
  full-lane table **17/17 p50 + 17/17 p99** (was 8/17 · 14/17 at Bench 041),
  published at reflex-site `6074b2b` and deployed live.
  - **Class B (the first-forward cliff):** T1, warm-at-load, −64…−68%.
  - **Class A (steady-state p50), the ladder in riir-infer:** T2/T3 waste
    removal; T4 coalesced Wᵀ; T5 packed multi-question forward (`da30007`);
    T7 rung 1 (the dispatch band, −29…−36% suite p50); T10 flash_attn
    softmax rungs (`0ec88a9`, `14af99f`); T11 split-K + its measured rule +
    `ln_rows_wide` (`512477e`, `ed1cb74`, `9b0e55c`) and the split-K fold
    epilogues (`53334f9`); T12 the packed-head deferred read (`40d15dd`).
    Two live correctness bugs were found and fixed on the way (the
    download-resolution aliasing, `2ea0197` + the most-recently-touched
    rule).
  - **The closer, T13/T13b (`b0de034`, `5e18da4`):** the residual was the
    encoder GEMM at big m, an axis T7 + the roofline probe had closed among
    OUR kernels (five refutations). The reopen was the Metal-stack change
    the issue named: unsplit batch-1 GEMMs dispatch Apple's
    `MPSMatrixMultiplication`, the oracle's own kernel family. It is
    bit-identical to the narrow instance and runs the whole forward at
    0.575–0.741× for m 106–895. T13b then lets MPS take m 33–96 from
    split-K (0.73–0.81×). That is a reduction-order change, and G5 holds it
    at LOWER drift. Kill-switches `LAYA_METAL_MPS=0` /
    `LAYA_METAL_MPS_SPLIT=0`.
  - Measured-negative rungs kept on record: T6 (host pooling, 1–1.6% of
    wall), the T7 occupancy / BK48 / coalescing / f16-B axes, T10 rung 2
    rope hoist (opt-in, not promoted), T11 rung 1 packed-at-1q.
  - **Deferred, not owed by the bar:** the `.metallib` precompile (a
    process-start cost, not a request cost); rope-table caching.
    **Priced 2026-09-26 and declined** (riir-infer
    `examples/startup_rope_probe.rs`; M3 Max, AC, load 3.1–3.8, three
    fresh processes):
    - `Metal::new()` takes **205.9 ms** on the first launch after a
      rebuild and **22.7–26.0 ms** once Apple's shader cache is warm. So
      a precompiled metallib saves about 180 ms, once per binary per
      machine. The engine is a long-lived server, so that cost is spread
      over every request it serves. In exchange, the release build would
      need the Xcode Metal toolchain. (The ~180 ms is inferred to be the
      shader compile from the warm/cold gap; the probe does not split
      compile time from MPS load.)
    - The rope-table pair (both thetas, hd 64) costs **21.8 / 40.4 /
      112.9 / 203.4 µs** at seq 106 / 188 / 512 / 895. That is ~0.1% of
      the Bench 050 p50s (12–194 ms), far below the ±6% A/B noise, so
      no gate could show a gain.
    - Neither applies off macOS-native anyway. A metallib is Apple-only;
      the browser and edge-worker lanes are the 92 KB wasm heads, with
      no laya weights and no GPU. A vessel carries signed genomes, not
      GPU code, and loading shaders from an artifact would widen the
      attack surface the binary channel already covers.
    - Reopen if the engine becomes a short-lived process per request,
      or if a macOS update breaks or drifts the runtime MSL compile
      (a metallib pins the front-end compiler that G5 measured).
  - The lesson: five refutations among your own instances say nothing
    about the vendor's. Price the library call before a kernel rewrite.
  - Full life: `git log --follow -- .issues/020_riir_metal_latency_parity.md`.

- **Issue 008 CLOSED — the riir-infer consolidation (the public
  LLM-inference substrate).** Owner directive 2026-09-22: open-source the
  LLM-inference part (and reflex) without leaking anything else, and put
  the related Metal work in one layer consumed by both riir-ai and reflex.
  The verdict was FEASIBLE: riir-ai Proposal 041's `riir-infer-core` was
  already the clean leaf, and this issue was 041's Phase 2 trigger T-B
  firing. All seven tasks landed:
  - **T1 (contract):** riir-ai mirror, now `../riir-infer/.issues/998`;
    BOUNDARY rows; the Research 003 dated amendment.
  - **T2:** the riir-gpu module audit (171 modules, CLEAN 98 / SEAM 34 /
    STAYS 39).
  - **T3:** carve v1 at riir-infer `86a5986`, name-unchanged, no shim;
    registered as the 23rd contract repo (katgpt-rs `bd2ce3cca`).
  - **P3 (the GPU kernel migration):** slices S1–S7 per riir-infer plan
    610. The S6b full edge-drop is owner-gated on the training-families
    home (riir-infer `.issues/1003`).
  - **T4 (the encoder-lane move):** `src/laya` moved to
    `riir-infer-laya` (riir-infer `c6716a4`), with the metal 0.31 bump
    needing zero API fixes, and G5 88/88 at both postures from the new
    home.
  - **T5:** the fence gate.
  - **T6:** both repos opened PUBLIC 2026-09-23 with fresh sanitized
    histories. `publish = false` stays; crates.io is owner-gated.
  - **T7 (the op-layer unification, riir-infer plan 611):** a portable
    CubeCL backend of the laya `Backend` trait over riir-infer-gpu's op
    layer, plus the engine-side GAP kernels (mean-centered two-pass
    LayerNorm, batched row-softmax, offset/head-batched matmuls,
    rope/split/merge/gather).
    - Its G5 posture surfaced and fixed riir-infer 016 (`gather_rows`
      residency) and 018: a CubeCL row-softmax write-after-read race,
      fixed at `2a34bd3`, found by this session's isolated A/B (probe
      43/120 → 0/120). The G5 cubecl arm was re-armed at `ccb5bd0`.
    - **Bench 006** (riir-infer; pre-registered `fac0dfd`, results
      `ac85c8e`): CubeCL is 5.1–8.4× slower than the hand Metal lane
      (0/12 wins in every cell), so **the Metal lane stays the macOS
      default**. It beats the CPU lane on short sequences (0.54–0.72) and
      on 1q (0.75), so **the CubeCL arm is kept opt-in**
      (`laya-riir-cubecl`, never in `RELEASE_FEATURES`). **Nothing was
      deleted or promoted.**
  - Standing follow-ups:
    - The lane's weights cache path still names this repo
      (`~/.cache/riir-reflex/laya`; `LAYA_WEIGHTS_DIR`/`LAYA_HOME`
      override).
    - riir-infer 998 keeps its D4 re-narrowing residue.
    - The T4 sibling-WIP snapshot lives untracked at
      `.issues/020_wip_snapshot_t4move/` in the main checkout.
  - Citations of `.issues/008` (BOUNDARY.md, AGENTS.md, bench 006, issue
    020, riir-ai / riir-infer docs) resolve here; the full file is
    `git log --follow -- .issues/008_riir_infer_consolidation.md`.

- **Issue 035 CLOSED — the cua-s1-forms CoreML/ANE arena arm (the
  System-One family's fourth serving posture).** T1 lineage (`a3c69f3`): Cua's
  independent jev-like option scorer (MIT, 706,048 params), converted to FP16
  CoreML by FluidInference — the family tie is the one-pass option-scorer
  CONTRACT, not the weights. T2–T5 (`6fadb5d`): serving = THEIR stack
  (`scripts/cua_s1_lane.py`, coremltools 9.0 subprocess, CPU_AND_NE, their
  `preprocessing.py` imported verbatim; HF model @ `8e18ee41`, all 64 files
  checksum-verified), our Rust measures (`examples/cua_s1_forms_arena.rs` —
  macOS item cfg, loud SKIP per absent lane, exit 2 on absent fixture,
  `required-features = ["modelless"]`, never in the default run). Fixture =
  their published test split (`cua-ai/cua-s1-forms` @ `8273f347`, 24,370 rows,
  SHA-256 matches their card, BLAKE3-pinned in the example), one row → one
  choice question — our 15 suites are OOD for a form specialist. **Bench 048**
  (M3 Max, AC, preflight quoted): coreml **24,359/24,370 = 99.9549%**, their
  published result reproduced exactly (same 11 fill-for-skip errors), 2.20 ms
  p50 round-trip; laya-typed zero-shot 29.95% (N=2437 stride, metal, G5 green
  on that build); modelless 4.36% — a constant `check` pick (route terms
  inactive on request-time options) → **Issue 036** (closed 2026-09-26 —
  see the 09-26/27 section). Constant-skip
  floor 52.16%: neither of our lanes clears it. No ANE-vs-CPU latency edge on
  the M3 (sequential readings overlap). Research 002 landscape row landed with
  the posture labeled. Code comments citing `.issues/035` resolve here.
- **Issues 032 + 034 CLOSED — the head-select publish landed both hosts
  full-lane; the publisher wipe's real mechanism measured + walled.**
  - **Issue 034's finding CORRECTED by re-measurement (the session's
    central result):** the lane wipe lives on the FRESH-DOCS publish path
    (`publish_bench.py <doc1> <doc2> <site>` — 15→14 suites, 85→14 cells,
    123 slots dropped, reproduced against the live table), NOT on the
    update path the original issue blamed: `host_lane_entry`'s
    `setdefault` returns the EXISTING container for any known host — an
    update writes its slots in place and every undeclared lane survives
    (verified with live-as-primary + both docs: 15 suites / 3 hosts /
    every comparison + ane cell intact). The REAL both-docs-together
    constraint is the Issue 018 T7 drift gate (published m3 modelless was
    pre-head-select 0.69-class vs the fresh 0.7933 — measured refusal).
    The issue-as-filed code-quote mechanism was wrong; the owner's
    option-3 decision survived the correction (fresh re-runs still
    wanted) but the comparison servers were never needed for this
    publish — the published clm/gliner/agentjev cells ride along
    untouched.
  - **The wall landed** (reflex-site `1bc4bab`):
    `guard_wholesale_replace` + `lane_inventory` refuse a fresh-docs
    publish that would drop published slots (real-table refusal names
    the 123), `PUBLISH_BENCH_FULL_REPLACE=1` acknowledges a deliberate
    replacement with a disclosure, the update path never walls
    (resolved-path compare), self-test 17→21/21 (refuse / env-acknowledge
    / update-path-bypass / laya-checkpoint-inventory arms); README
    publish-shape law `cd08984`.
  - **Bench 045 — the M3 full-lane leg** (`--head-select --laya-python
    --features slice_leak,laya-riir,laya-riir-metal`, reflex `6939420` +
    riir-infer `53334f9`, preflight AC/high load 3.21→2.21, 15/15
    suites): head selections identical to 040/043/044 (four-run);
    banking77 0.6840 / massive 0.7933 bit-identical again; 7/7 leak
    blocks byte-identical to the 4090's Bench 044 (the scan is a
    dataset+cap property — two-host fact); typed/english 309 ms vs the
    published 367 (the fold promotion visible).
  - **Bench 047 — the 4090 full-lane leg** (`--head-select --features
    slice_leak,laya-riir-cuda` at d727196, launched detached via a
    schtasks one-shot after two Start-Process deaths — the ssh session
    kills its process tree; box-state UNJUDGED by design): 15/15
    suites, cross-host modelless BIT-IDENTICAL to 045, selections
    identical, third leak confirmation, typed/english CUDA 105 ms p50.
  - **The publish + deploy + live verify** (reflex-site `1e27741`, worker
    `b086cf74`): `republish_bench.sh` full chain (self-test → chart
    smoke → publish → mirror parity → bench-page smoke PASSED), live
    curl verified — 15 suites / 3 hosts, m3 lane_sources `6939420`,
    comparisons + ane rows alive, `harness_cache_reuse` present (the
    034 reappearance checkpoint confirmed).
  - **Issue 024 T5's host halves both landed** (045 M3 + 047 4090 with
    Bench 044's 4090 run); CLOSED later the same day — see the 024 close-out below.
  - **Issue 020 T11's fold rung PROMOTED default-on** (riir-infer
    `53334f9`): `LAYA_METAL_FOLD_RES`/`_GLU` flip to the `!= Ok("0")`
    kill-switch spelling on the paired A/B's evidence — 24/24 wins on
    every shape within the split rule's reach (medians −1.5…−5.9%), the
    no-op control flat at 1.002/1.001 above the crossover, bit gates +
    G5 both postures green. **Bench 046 — the paired per-suite A/B's
    first publication-grade run:** the stable band FLIPPED to wins
    (ag_news −4.0% / banking77 −5.2% / emotion −17.6% / sst5 −10.0% /
    xnli_en −17.4% p50; p99 8/9; banking77 reproduced byte-identical on
    a second paired sample); the every-cell bar's remaining deficit is
    the typed_decisions trio (english +22.3% p50 AND p99 — the only
    double loss) + massive_intent_en +2.8% (within-tol, sup 4 — rerun
    before calling). Instrument repairs found BY the run: per-suite dirs
    pre-created (the log redirect died pre-measurement) +
    provenance.txt carried the NOT-FOR-PUBLICATION default even when
    preflight PASSED (marker and evidence disagreeing).
- **Issue 024 CLOSED — the laya-lane de-leaked columns were already in
  the Bench 045 results; the write-up landed and the owed clippy
  postures discharged** (this session, `bfc19b0`-successor commit): the
  T3 plumbing (`served_flags` through `assemble_laya_lane_result`) had
  produced `acc_deleaked` on every laya checkpoint row of all 7
  leak-scope suites in 045's results.json — the issue's "still to take"
  note was stale, the data existed unwritten up. The 045 BENCH.md
  addendum renders the per-suite headline → de-leaked table: laya DROPS
  with the leaks removed on banking77 (−0.32 pt) / ag_news (−0.09 pt) /
  prompt_injections (−0.53 pt) — the shared leak inflates the laya lanes
  too, the disclosure was never a modelless-only concern — while
  massive_intent_en moves UP +1.34 pt (its 21 flagged rows were net
  unlucky for the lane; de-leaked is not mechanically lower), and
  emotion / xnli_en / sst5 are flat (0 / 0 / 1 exact-only flags).
  riir ≡ py to all printed digits per suite (same cases, same mask —
  the determinism cross-check). The T3-owed laya clippy verification
  ran in the same window and caught a REAL pre-existing breakage:
  `--no-default-features --features laya-riir --all-targets` failed
  E0433 — the parity files hash the frozen capture with blake3, which
  only `modelless` pulled; fixed by `laya-riir = [..., "dep:blake3"]`,
  all three laya clippy postures clean, both parity gates re-run green
  at the no-default posture. Issue file removed; this entry is the
  record.
- **Issue 020's typed-trio lever PRICED and RE-AIMED; T12 built opt-in**
  (riir-infer `be46033`, the `typed_case_split` probe `abbcbb3`, docs
  `bfc19b0`): typed_decisions measured 100% multi-q (all 400 cases =
  exactly 5 questions) — the packed-at-1q re-price cannot touch it; the
  stage decomposition (per-stage per-dispatch GPU profile at real 5-q
  cases) puts the ENCODER at 90.1% of case GPU (sgemm narrow 85.7% of
  that, flash_attn 7.5%) and head+copy at 9.9% — so the v2 packed head
  stays bounded at a few percent and the +22.3% typed·english deficit is
  the encoder GEMM at big-m vs MPS (the axis whose reopening conditions
  are owner-gated). The wall split alone misleads: the encoder's GPU
  work hides inside the FIRST question's drain (encoder enqueue 0.6% of
  wall). **T12** (`LAYA_HEAD_DEFER=1`, default-OFF) defers the packed
  case's head reads into two drain classes (15 → 2 per 5-q case);
  bit-identical both postures (raw-bit same-shape gate green under
  both), measured ~1–3% against a ±15–35% noise floor (Metal's enqueue
  already runs ahead inside a case) — opt-in pending a proven-quiet A/B,
  the rope-hoist precedent. The priced next rung: fold the residual-add
  + GLU epilogues into the NARROW (non-split) sgemm — the landed folds
  engage only on split-K calls, which typed·english's big-m shapes are
  not (≈5.7% of case GPU rides unfused: add 56 + glu 28
  dispatches/case). massive_intent_en's +2.8% ruled UNADJUDICABLE at ms
  quantization (1 ms of 63 ms = 1.6% sits inside the 2% tie band) — the
  rerun would be decorative. Issue 020 stays OPEN on the trio.

## 2026-09-25

- **Issue close-out sweep 2026-09-25 #2 (this commit) — Issue 007 removed;
  the `.issues/.highwater` counter repaired; open-issue status refresh.**
  - **Issue 007 CLOSED — harness families + corpus on the neuron-db
    substrate.** P1 LANDED `307a10b` (`corpus_db` opt-in, native-only,
    DEFAULT-OFF: subprocess store over the released `ndb` CLI —
    `--json`-only calls, stdin writes never argv, one-corpus-one-row,
    BLAKE3 digest-pinned corpus keys, `NDB_ASSUME_YES`,
    `NDB_BIN`→PATH→loud-refuse resolution; harness `--runs-kv` run-history
    rows + `--save-corpus` read-back-verified rows; consumer-side golden
    pin live-verified against ndb 0.1.0; BOUNDARY.md runtime-dep row —
    zero cargo dep on the storage leaf, the no-source-leak posture made
    structural). P2 DECIDED `e0557a2` (owner-delegated): NEITHER
    neuron-db route — the trigger was MEASURED as fired
    (`scripts/slice_leak_probe.py`: ag_news 6.8% / banking77 3.2% /
    massive 2.9% near-twins at ≥ 0.8 char-4-gram Jaccard, ~98% same-label)
    and the answer is the in-harness `slice_leak` report, landed as
    Issue 024 (still open there on its T5 write-up). **Deferred remainder
    carried here**: corpus LOAD-from-kv (`[-]`) — the write+verify side is
    live (`--save-corpus` read-back-verifies each row), no consumer
    exists yet; land it when the first kv-only run is actually wanted
    (e.g. CI without the dataset checkout). Reopen triggers: a third
    harness lane needing cross-process durability → P1's scope moves up;
    a corpus larger than memory → the `ndb shard` route reopens
    (neuron-db Proposal 002 Phase 4); the CLI proves burdensome in CI
    beyond `NDB_ASSUME_YES` → the `--no-identity` posture decision opens
    neuron-db-side. Code comments citing `.issues/007` resolve here
    (`AGENTS.md` corpus_db build-command row).

- **Issue close-out sweep 2026-09-25 (this commit) — six done issues removed
  from `.issues/`, their records here.** Live-verified before removal:
  `reflex.gist.rs/data/bench.json` is byte-identical to reflex-site HEAD
  (`cb0eaf7`) and carries the clm (15), gliner (15) and agentjev (14) lane rows
  — so the "site DEPLOY, M3-gated" remains in 025/027/029 were already
  discharged by the Bench 041 republish deploy.
  - **Issue 017 CLOSED — the ANE lane (laya-apple distill).** Gate 1 ratified
    at feasibility+bench scope (release scope: download-on-demand,
    digest-pinned); P0 six BC1S FP16 artifacts 100%-ANE / 0 transitions
    (`de3de3e`+`53df6e1`); P1 substrate runtime riir-infer `78a91c3`+`5c8ec2a`,
    G5-ANE 76/76 top-1 zero flips (`2169ed1`); P2 publish inside the 023 T5
    republish (site `3ab373f`, worker `cde063c2`, Plan 002 closed `957b87b`);
    serve wiring reflex `c2df053` + substrate fetch `1cbbb6e` (`5a71c61`).
    Defers carried as posture: (a) heterogeneous ANE+Metal two-queue serving
    (the 2.9–4.6× class) waits on Issue 015's parallel-instance root cause +
    a site posture plan; (b) the "defer the typed ANE conversion" row was in
    fact discharged by P0 (all three models, english/typed/multilingual,
    converted). Artifact HOSTING (who uploads the 1.3 GB where) is an owner
    ops act; the client side is ready.
  - **Issue 023 CLOSED — option-NAME route for the modelless centroid signal**
    (`d02f3a8`: massive 0.077 → 0.690, G1 FAIL → PASS; 12 suites
    byte-identical). T4 cap re-selection declined promotion; T5 the
    together-republish (site `3ab373f`, worker `cde063c2`, `3d4f7ce`); T6
    pair-head refutation stands (`e8ee043`); T7 stale-verdict sweep
    (`a21ed07`).
  - **Issue 025 CLOSED — AgentJev / System-One positioning.** Bench-doc rows
    verified at the pinned sha `a965ca8f`; the MEASURE row landed as Bench 039
    (`1e5847a`): gold-label typed_decisions **0.7715** vs laya-typed 0.7445
    (+2.7 pt, the published ranking survives the protocol change); 15-suite
    shape 3 agentjev / 7 laya / 4 gliner; site column `12ab4bb` (`3f80d95`),
    live. **DEFERRED (owner-gated verdict 2026-09-24, `#Verdict: AGREE`):**
    the lane-3 serving port of `aimeigaoshou/agent-jev` through riir-infer.
    Reopen triggers (checkable): riir-infer op-layer T7 AND the EXL3 trellis
    lane AND the ANE lane each closed, OR the first production consumer of
    `Lane::Hybrid` lands. Non-goal stands: no shared-prefix work without a
    causal decision lane to host it.
  - **Issue 027 CLOSED — the CLM T3 4090 window** (Bench 034, `b67eeb8`,
    renumbered `904a5c1`; Rust halves `967415c`): determinism pin GREEN (8/8
    byte-identical, head mtime unchanged), CLM cells on all 15 suites, leak
    columns rode the publish; the riding AgentJev row executed as Bench 039.
    **Window laws for the next 4090 CLM run (Issue 019 T4 inherits them):**
    never beside the perf-league cycles or a laya CUDA pass (the first
    combined attempt stuck ~13 min under VRAM contention and was discarded —
    run SERIALIZED passes); `--gpu-memory-utilization 0.72`, not 0.35 (bf16
    weights alone are 14.11 GiB on a 24 GB card); send one fixed warmup
    request first (the first request after a `clm-serve` boot reports
    `usage.input_tokens = 0`); same-box latency law (no CLM-4090 vs
    laya-M3 latency comparison); the "9× vs Jev" figure never enters our
    tables. Boot script: `scripts/clm_serve_4090.sh`. Code comments citing
    `.issues/027` resolve here.
  - **Issue 029 CLOSED — the GLiNER comparison lane** (`7ad656e` lane + Bench
    037 first cells, `a2c03aa` landscape row, `e04b586`/`fa86b46`): gliner
    beats the laya BASE checkpoints on 9/15 suites; site deploy verified live
    (above). The publisher-owner cosmetic question (the 4090-win per-host
    header keeps the ORIGINAL run's sha `afacc3a` beside newer lanes) —
    DECIDED (Claude verdict under the standing owner delegation): **keep the
    Issue 023 T5 law** — the header records the original run's facts; the
    per-lane truth is `lane_sources`; no publisher change.
  - **Issue 030 CLOSED — noul route anti-alignment + lever-4 fitted heads**
    (fix `36e4e0a` / Bench 038; heads `69a6eae` / Bench 040). Its last item —
    the arena republish at the promoted `--head-select` posture — had NOT
    happened (Bench 041 ran `head_posture: OFF` on both hosts), so it moves
    to the new **Issue 032** rather than being ticked as done.
  - Issue 019 T5 (`/bench` publish + README row) ticked in place — verified
    live (15 `clm (reference)` rows) + README lane rows; 019 stays OPEN on the
    optional T4 T-Rex re-run.

- **Issue 031 RESOLVED — the fixture_pins four-hash + the parse-precision
  note, LANDED (this commit): `fixture_pins()` (issue-884 hardening) hashes
  every EMBEDDED fixture's `include_str!` bytes against its pin in the serve
  tests (`every_embedded_fixture_hashes_to_its_pin`) — the three pins were
  length-only assertions before; the UNSERVED tetris v3/v4 oracle fixtures
  are pinned test-side from verbatim copies in `tests/fixtures/`
  (`unserved_tetris_v3_v4_fixtures_hash_to_their_pins`, pins `12035ebf…` /
  `18e6b260…`, a missing file FAILS never skips) — deliberately not embedded
  in the serve binary (v4 alone is 8.4 MB); the parse-precision note lives in
  the `game_heads` module doc (a fit landing on katgpt-rs head bytes needs
  the same `serde_json/float_roundtrip` feature — the default parser moved
  the tetris structured-head digest `65409c14…` vs `b3c91ee0…`, Bench 890
  §G3, agreement numbers unaffected). The crossed-head integration stays
  CLOSED on the Bench 890 G1 FAIL — two-line serving to the spot head
  remains refused by design. Serve suite 16/16 green (both new pins
  included), clippy -D clean. Full file life: `git log --follow --
  .issues/031_tetris_v4_phase2_handoff.md`.
- **Issue 030 RESOLVED — lever 4, the fitted-head feature-class arc, LANDED at
  bench 040 (this commit): banking77 +23.8 pt, massive +10.3 pt, every other
  suite bit-identical or ECE-better — the modelless lane takes BOTH rows past
  their laya-best opponents.** `src/label_heads.rs`: one-vs-all logistic heads
  (sigmoid, never softmax) over the UNTOUCHED 256-dim hashed-bag features,
  fitted at engine-build time from the same post-cal pool (deterministic:
  fixed doc order, 12 epochs, lr 0.5 linear decay, L2 1e-4, no RNG;
  bit-identical re-fit, tested). Blend term `0.5 + head_scale·(σ(logit) − 0.5)`
  wherever route terms are active; noul never takes it (issue 030's law).
  `EngineConfig.head_scale` default 0 = byte-identical baseline; `build()`
  REFUSES a non-zero scale it cannot fit (fail-closed — never a silent no-op
  head). Arena posture = harness `--head-select`: per eligible dataset suite,
  accuracy per ladder `[0, 0.25, 0.5, 1.0]` on the STRATIFIED selection slice
  (forced), 5 pt promotion bar over scale-0, ties → off, test read once;
  per-candidate rows disclosed in results.json + TABLES.md +
  `RunMeta.head_posture`. Three measured traps shaped the protocol: (1) the
  raw cal slice REVERSES banking77's scale signal (label-clustered cal,
  Bench-004/Issue-023 class re-measured on this axis — accs fall 0.265→0.200
  on cal while test rises +23.8 pt; the stratified slice ranks it right,
  0.49→0.715); (2) n=200 selection noise flips small/neutral suites both ways
  (emotion: sel +2.5 pt / test −5.5 pt) — the promotion bar makes the selection
  only move on strong evidence; (3) scale > 1 fails G1 BY CONSTRUCTION (the
  blend over-weights the model past its own calibrated confidence; banking77
  at pin-2.0: calibrated ECE 0.498 > floor 0.464) — the ladder caps at the
  fitted-model-verbatim 1.0, where G1 passes. GOAT: G1 PASS at every promoted
  posture (ag_news' floor FAIL is byte-identical baseline, selected 0,
  pre-existing); G2 decision_set_goat p99 45 µs + per-suite p50s unchanged;
  G3 bit-identical everywhere not promoted; G4 core alloc-free (stack dot per
  option). 152 tests green, clippy -D clean at both postures. Posture law: the
  ENGINE default stays 0 (the published baseline reproduces bit-for-bit); the
  ARENA protocol is `--head-select`. Open from the issue: the published-table
  republish (both laya lanes + reflex-site + manual CF deploy) — the last
  unchecked task.

- **Issue 030 (partial) — the noul route anti-alignment FIXED at `36e4e0a`
  (bench 038): prompt_injections 0.4397 → 0.4828.** The T7 option-rank blend reached
  noul questions through the legacy `k == N` index path (by-name already
  excluded noul); on prompt_injections (N = 2: `"0"` = benign,
  `"1"` = injection; internal option order `[yes, no]`, yes = injection)
  that mapped "yes, injection" onto the BENIGN centroid — an anti-signal
  by construction, the exact signature of the recorded below-chance 0.4397
  (chance 0.50; T7 Addendum 5's −4.3 pt). Fix: `route_active` requires a
  Choice/Score question — noul never takes route terms through either
  path. Blast radius: only `N == k == 2` noul engines (prompt_injections
  is the only dataset suite in that shape; the serve edge is 7-domain).
  A/B (modelless, deterministic lane, back-to-back same box): the changed
  suite improves on every axis (ECE 0.1070 → 0.0228, acc@50 0.3966 →
  0.6552, G1 PASS held); every other suite bit-identical;
  `code_fixtures` wiggles are its self-referential corpus tracking this
  repo's own source (disclosed in the bench). Regression pin:
  `noul_never_takes_route_terms_even_when_k_equals_n`. Remaining in the
  issue: lever-4 fitted heads (the feature-class arc), the residual
  emotion −1.3 pt (deferred), and the full-table/arena republish (the
  published modelless column is stale as of this fix).
- **Issue 029 — the GLiNER comparison lane LANDED + first cells measured
  + published** (same-day arc on the 4090 window): the external
  fastino/GLiNER2.5-Decide zero-shot classifier (Apache-2.0, 340M
  DeBERTa-v3-large) as a JSONL subprocess oracle over THEIR gliner2
  package — `scripts/gliner_lane.py` (the `laya_python_lane.py` protocol;
  one schema per case, labels-with-descriptions their native form, noul →
  `[no, yes]`, a DISCLOSED strip for their structural-token refusal, the
  loader banner kept off the protocol channel) + `run_gliner_lane` (the
  handshake-advertised model id, the trim law, the GPU pre-ramp, the
  observed-repeat det check, the same metrics tail via
  `assemble_laya_lane_result`; `--gliner`, NO feature gate — zero new
  deps). The measured verdict (host `4090-windows`, all 15 suites, no
  absences, gliner det ✓ everywhere, same run as a fresh laya-riir cuda
  pass at the SAME substrate sha `1afd4f8` as the published lanes):
  **gliner beats the laya BASE checkpoints on 9/15 suites** — banking77
  0.706 vs 0.498, typed_decisions base 0.528 vs 0.3575, massive_intent
  0.823 vs 0.750 — **loses classic NLU** (ag_news 0.70 vs 0.95, xnli 0.48
  vs 0.86, emotion 0.565 vs 0.593) and **does not touch the laya `typed`
  specialist on the headline** (0.528 vs 0.7445). Latency: p50 22–32 ms
  (subprocess IPC in, fp32 CUDA) — 3.5× FASTER than the in-process laya
  cuda lane on long-context typed_decisions (30 vs 107 ms), ~2× slower on
  short suites (22 vs 10–12 ms). Modelless drift gate: clean (13 suites
  bit-identical to m3; `code_fixtures` the designed exclusion). Site ride:
  the publisher carries the lane (the clm law), the bench page gained
  clm + gliner chart slots with extra-host hero fallback (host-tagged
  tooltips) + the **lane-filter checkbox bar** (the owner ask) governing
  EVERY section with localStorage persistence — validated by a headless
  playwright smoke (chips, rows, toggle-off everywhere, reload
  persistence, restore). Site deploy: the M3 discharges it (the 027
  no-CF-creds law on this box).
- **Issue 027 — the CLM T3 4090 window EXECUTED** (bench 034,
  renumbered from 033 in the same-window dual-allocation with the
  sibling's rope-hoist instrument `033_rope_hoist_ab` (landed at origin
  first — numbers are never reused, the renumber consumes 034 exactly
  like a fresh allocation); reflex `967415c`+`16c6589`+`b67eeb8`, site
  `ae79a87` — the site commit's "bench(033)" spelling predates the
  renumber, disambiguated HERE): the external Contrastive-LM reference's
  first measured cells on the arena, end-to-end in one window. The
  serving stack (their code, our docker): vLLM `--runner pooling` over
  Qwen3-8B (LAST-token, prefix cache, their `serve_qwen3_8b.sh` flags
  verbatim) + `clm-serve` over the mounted `CLM_v0.1-8B` head
  (`--no-download` provenance posture) — one container,
  `scripts/clm_serve_4090.sh`. **The determinism pin GREEN** (8/8
  byte-identical repeats + head mtime unchanged) after one measured
  cold-start law: the first request after a clm-serve boot answers
  correctly but reports `usage.input_tokens = 0` — answers byte-stable,
  accounting not; the lane and the pin warm with one fixed throwaway
  request (never a case's). **The serialization law bit as written**:
  the combined attempt (laya CUDA beside resident vLLM) hung 13 min on
  `laya[typed]` (the contention signature) and was killed — the
  published run is clm-with-vLLM only; the 4090 laya lanes carry over
  from bench 032 at the NEWER substrate (`lane_sources` disclose the
  split; re-running laya at this box's older checkout would regress the
  table). **util 0.35 does not fit a 24 GB card** (bf16 weights alone
  14.11 GiB; 8.6 GB budget dies at cache-block allocation) — the
  dedicated-window posture is 0.72. The harness grew the `clm` column
  (`--clm`, feature `clm-lane`, `SuiteResult.clm`): the parity law
  applied at the builder — THEIR `to_text` state prose (byte-pinned
  copy), choice candidates = criterion DESCRIPTIONS under their
  `candidates` law ("a candidate reaches the encoder exactly as the
  caller wrote it"), score = rubric levels, noul = their default law;
  latency = client round-trip; observed-repeat det check; `clm_request`
  unit-pinned. **Cells (honest rows, not wins)**: typed_decisions
  **0.3465** against laya-typed 0.7445 on the same split (modelless
  0.3190) · xnli 0.6167 · ag_news 0.4025 · prompt_injections 0.5345 ·
  banking77 **0.0100** · p50 ≈31 ms/case localhost (66 ms on typed's
  5-question cases), det ✓ every suite. **The 024 leak columns rode the
  same publish** (T4 closed): 7 dataset suites carry `leak` blocks +
  `acc_deleaked` per lane (ag_news exact 1/near 26; massive 4/17;
  banking77 0/17; emotion + xnli clean); `publish_bench.py` carries clm
  lanes + leak blocks through update docs and the page renders the
  disclosure line. Publisher tests 11/11. REMAINS: the optional AgentJev
  gold-label row (a different server — their `jev_service`; the next
  4090 window, 025's do-NOT-auto-start stands) + the site deploy
  (M3-gated — no CF creds on this box; `npx wrangler deploy` from the M3
  discharges it). Attribution: Contrastive-LM/CLM @ `cca045ff` +
  CLM-v0.1-8B, Apache-2.0, not affiliated — a comparison lane, never a
  product lane.

- **README CUDA-row ratio refresh — 14–17× the CPU row, measured at the
  post-ladder HEAD** (doc-sync; no code change). The v1-rung claim
  ("10–12× the CPU row", landed with 026 before the flash/float4/reg4
  rungs) was stale in the conservative direction. Same-binary env-flip
  pairs on this box (release, `laya-riir-cuda`, `LAYA_DEVICE=cpu` vs
  `=cuda`, 2–3 alternating pairs per fixture, row p50): english
  211.4/12.6 = **16.8×** (pair range 16.1–17.5) · typed 205.2/12.7 =
  **16.1×** · multilingual 93.1/6.65 = **14.0×**. CUDA row p50
  12.6/12.7/6.5 ms sits below the M3 Metal row (28.3/28.3/12.2) on every
  fixture. Box state: AC · GPU idle of compute (GUI apps only, the
  exempt class) · a sibling agent's CPU-only tokenization running
  throughout · the sibling's riir-infer WIP verified behavior-neutral
  for the laya lane (opt-in feature + visibility derives; the lane's
  dep closure untouched) — the binary measured is HEAD-equivalent.
  Build isolated at `CARGO_TARGET_DIR=/tmp/reflex_ratio_target`.

- **Bench 030 — the float4 sgemm rung: every suite improves, the packed
  zone's first win** (substrate: riir-infer `.issues/006`, commit
  `aadbc07`; reflex artifacts here + the probe's new packed-zone
  population). The `.issues/004` open question (multi-wave zone, split-K /
  occupancy-tuned instance) reframed on measurement: the wide instance ran
  at 15-20 % of the 4090's fp32 peak because the inner loop issues 6 smem
  loads per 8 FMAs with the four B-fragment loads CONTIGUOUS in staging.
  Every instance's B staging row pads to a 16 B multiple (65→68, 129→132)
  and the four loads collapse to ONE float4 — result-identical by
  construction, bit-identical on every probe shape. **Every suite −6.8..
  −16.7 % p50, the PACKED suites included** (typed_decisions 109→100 /
  59→55 / 109→99 ms, banking77 28→25, code_fixtures 31→28) — the
  multi-wave zone's first measured win. Forward rows: english 15.1→12.9
  (−14.6 %), multilingual 7.2→6.3 (−12.5 %), typed 15.1→12.8 (−15.2 %).
  Result identity: 13/16 bit-identical; the three typed_decisions
  wobblers (1-4 cases / 2000) stay in that lane's pre-existing
  `determinism_ok: false` class — false in 026/028/029/030. Gates at the
  landing: G5 cuda posture (top-1 1.0 ×3) + laya_batch_parity +
  packed_forward_equiv + cuda_ops_smoke + clippy, both repos. **Also this
  window:** CUDA graphs CLOSED NEGATIVE in the substrate (riir-infer
  `.issues/005`: `gpu == wall` on every fixture row — the CPU submit path
  is fully hidden; the `LAYA_CUDA_STATS` submit/wall/gpu instrument
  ships). Site: the 4090 lane row refresh is PREPARED (this record) but
  `wrangler deploy` remains the standing M3-side handoff — Node tooling is
  broken on this box too. Remaining rung on record: register blocking /
  double-buffered staging (the instances sit at ~25-30 % of fp32 peak
  after this rung).

- **Bench 029 — the CUDA sgemm tile ladder: the narrow instance's
  single-question win** (substrate: riir-infer `.issues/004`; reflex
  artifacts at `2a61a6d`+; site lane refresh reflex-site `1a59939`).
  THREE sgemm instances (narrow 32×64×64 / wide 64×64×32 / xwide
  64×128×32) picked per call by **BLOCK-FIT on the SM count** — the
  measured cliff: at m=106 narrow wins −15.7 % at n=2048 (=128 blocks,
  exactly one per SM) and LOSES +46 % at n=2560 (=160 blocks — static
  block scheduling strands 32 SMs at 2× work while 96 idle). The M3
  Metal lane's m<256 floor does NOT transfer to the 128-SM 4090; the
  block-fit arithmetic subsumes it. xwide keeps m≥256 ∧ n≥2048 PLUS the
  block-fit cap (the multi-wave gate/up zone reverts to the proven wide
  — its readings sat inside the new probe's measured ±8-10 % two-context
  artifact band). Kill-switch `LAYA_CUDA_LADDER=0`. A launch defect fixed
  in passing: the v1 form passed the staging footprint as DYNAMIC smem
  on top of the kernels' STATIC `__shared__` — 2×24 960 B crosses the
  48 KB default and dies `CUDA_ERROR_INVALID_VALUE`; dynamic smem is now
  0 (the footprints live on as compile-time bounds). **Result identity:**
  13/16 suite-lane rows bit-identical vs 028 (every single-question
  suite); typed_decisions wobbles 1 case in 2000 per checkpoint — that
  lane's `determinism_ok` has been false since the 026 v1 run
  (pre-existing, on record, independent of the ladder); G5 +
  `laya_batch_parity` + `packed_forward_equiv` + the boundary/ragged
  `cuda_ops_smoke` arms all green at the final floors. Forward A/B
  (fixture rows, ABAB): english −10.4 % · multilingual −14.1 % · typed
  −8.4 %. The published row: single-question suites −6..−14 % p50
  (emotion 14→12 · tool_fit 14→12 · routing/sensitivity/cache 16→14 ·
  massive 20→18 · ag_news 16→15 ms), packed suites flat by the
  conservative floor (typed 108→109 · banking77 27→28 · code 30→31 ms).
  The probe grew a CUDA arm (`sgemm_shape_timing` — same-process
  two-backend A/B + a `--control` artifact-band mode; the example's
  `required-features` row dropped — its posture arms are item-level cfgs
  with a loud fallback, so a whole-file row would green-zero the other
  posture's lane). Open after this: CUDA graphs; the packed multi-wave
  zone has NO measured win yet (split-K / occupancy-tuned instance is
  the question, not another tile size).
- **Issue 019 T1 + T2's M3-verifiable scope — the CLM comparison-lane
  adapter + the prose-rendering law, byte-pinned** (T3 remains the 4090
  window, `.issues/027`): `src/lanes/clm.rs` behind `clm-lane`
  (`src/lanes/clm.rs` behind `clm-lane`
  (implies `katgpt-core/decision_wire`; zero new packages — std HTTP
  client over the `serve.rs` hand-rolled posture + the in-tree
  serde_json/preserve_order). The law (`to_text`/`state_text`/
  `candidates`) is a byte-pinned port of their `schema.py` @ `cca045ff`:
  42 goldens generated by running THEIR Python at the pin
  (`scripts/clm_goldens.py`, the offline one-time carve-out) — Python float
  `repr` presentation, empty-container trailing spaces, noul
  false-first answer order, empty-option→key, unicode. The T2(b) stub
  pins the request wire server-side (method, path, body shape, option
  order via preserve_order — their `list(crit)` order IS the answer
  order) and maps the canned response through `validate_against`;
  fail-closed pins: missing answer, out-of-range probability, non-200,
  name-keyed mapping (JSON key order cannot misalign). Lane vocabulary
  grew upstream: `Lane::Clm` (katgpt-rs `0193ae92e`, purely additive —
  no exhaustive matches, wire goldens untouched). Deliberate divergences
  documented in-module: noul per-side descriptions inexpressible on our
  wire (default law always), criteria prose folds under the prompt,
  temperature pinned 1.0 (their `∈ (0,100]` logit divisor — never our τ),
  noul confidence recomputed by their own `top − mean(rest)` law (their
  server reports none). Gates: clippy `-D` at default / `clm-lane` /
  all-features; 12/12 lane lib tests; 56/56 default lib tests. The
  renumber 026→027 landed first (`7e62f96`): upstream dual-allocated 026
  while `.issues/026_clm_t3_4090_posture.md` was in flight (the 4090
  CUDA-lane issue, closed + removed at `6d6cd8c`); citations rewritten
  in 019 + 025, `.highwater` = 27, `ec65d4c` dropped as already-upstream
  (both sides had independently added the `DeviceKind::Cuda` arm to the
  harness runner — upstream's is canonical).

- **Issue 028 — the 026 bench CORRECTION: the CUDA packed-path zeros
  defect + the flash rung** CLOSED (substrate: riir-infer `.issues/003`
  `75ed138`; reflex artifacts at `6d6cd8c`+; site correction
  reflex-site `faf9b50`). The 026 run's trait-default attention SLICES
  host memory at the packed multi-question offsets — under the device
  backends' write-first discipline those host bytes are STALE
  (device-written only), and the chain-cache MISS uploads them, so
  **every multi-question case's attention ran on zeros**. Corrupted
  published rows (vs CPU 018 / m3 025, which agree): typed_decisions
  english 0.3575→0.2690 · multilingual 0.3490→0.2690 · typed
  **0.7445→0.2690** (−47.5 pt) · code_fixtures 0.5417→0.2917 (2 q/case);
  every 1-question suite was byte-identical and correct. The 026
  close-out's "accuracy byte-identical on every lane" claim was WRONG
  for the multi-question rows — corrected there. The consumer-side G5
  passed because its fixtures are single-question (offset zero);
  **`laya_batch_parity` — the multi-question gate — had never been run
  at the cuda posture**; it is now part of the cuda gate set and green
  (top-1 1.000000 ×3 checkpoints, drift ≤ 5.1e-5). The fix is the
  substrate flash kernel (the Metal one-pass online-softmax form ported
  to CUDA C, offsets bind at dispatch — Metal's design, immune by
  construction). The corrected bench
  (`.benchmarks/028_4090windows_cuda_flash/`, host `4090-windows`):
  typed_decisions restored to 0.357/0.350/0.7415, code_fixtures 0.5833,
  all 1-question suites unchanged; latencies improved −4..−12%
  (typed 113→108 · multiling 65→59 · banking77 29→27 · code_fixtures
  34→30 · ag_news 17→16 · emotion 15→14 ms). Honest note: 2000-question
  suites carry ±0.003 run-to-run variance (corpus-pool composition —
  pre-existing, affects every lane; the frozen-capture gates are the
  authority). The site correction landed BEFORE the pending M3-side
  `wrangler deploy` — **the corrupted numbers never went live** (the
  live site serves the pre-026 data; the deploy now publishes the
  corrected row directly).
  Renumber note (the dual-allocation class): this issue was filed as
  `.issues/027` in the same window a sibling session filed the CLM T3
  issue; theirs is the live 027 (`027_clm_t3_4090_posture.md`, landed
  first at `7e62f96`), so this record claims 028 — numbers never
  reused; the bench artifacts moved with it.

Created retroactively 2026-09-22: four issues (001, 002, 003, 005) had
already closed with records only in git history.

## 2026-09-24

- **Issue 026 — the 4090 CUDA lane: `laya-riir-cuda` green + the
  4090-windows bench row cpu → cuda** CLOSED (backend:
  riir-infer `.issues/002`, record `e99d767`/`1efc8b7`; reflex
  `0550820`; site data `c6d1ece`). The published row ran the laya lane on
  CPU (Metal is macOS-scoped; the GPU idle) — 11–20× the m3 metal row.
  The CUDA backend (cudarc 0.19 + nvrtc sm_89, the Metal architecture
  ported: permanent weight cache, epoch-keyed chain slots,
  `download_into` prefix barrier over `CudaView` slices — `CudaSlice::
  clone()` is a dtod COPY, the caches hold `Arc`) composes with the
  same-day packed-forward landing (`copy_at` added at the rebase).
  Gates green on this box: `cuda_ops_smoke` (every op vs CPU;
  bit-exact data movement), G5 at `LAYA_DEVICE=cuda` first run (english
  26/26 drift 1.863e-6 · typed 26/26 2.471e-6 · multilingual 36/36
  2.894e-6 — the metal drift class), `packed_forward_equiv` at the cuda
  posture. The refreshed row (`.benchmarks/026_4090windows_cuda/`, host
  `4090-windows`, sha `0550820`, accuracy byte-identical to the cpu row
  on every lane **⚠ CORRECTED 2026-09-25 (Issue 027): this claim was
  wrong for the MULTI-QUESTION suites — the packed path ran dead
  attention (typed 0.7445→0.2690, code_fixtures 0.5417→0.2917); the
  1-question suites were byte-identical and correct**): typed english 8127→114 ms · typed multilingual
  4354→65 · typed typed 8221→113 · ag_news 460→17 · banking77 1745→29 ·
  emotion 253→15 — **every suite below the m3 metal row** at the v1
  rung (default attention path; flash/tile-ladder/CUDA-graph rungs are
  follow-ups, each G5-gated at the cuda posture). Site publisher fixes
  riding the merge (10/10 tests): `code_fixtures` POPULATION-EXCLUDED
  from the cross-host drift gate (its population is commit-relative —
  real fn spans from the repo's own sources; the 018 close-out's
  recorded exclusion, mechanized), and the device posture is a LANE
  fact (the 025 T4 class — a laya update refreshes the host row's
  `laya_device`, never "cpu" beside cuda numbers). The site data commit
  is pushed; **the `wrangler deploy` remains the M3-side handoff** (no
  Node≥22/CF creds on this box — the 018 T6 precedent).

- **Issue 025 — the `laya (python)` lane back on reflex.gist.rs/bench**
  CLOSED. The page had shown `laya (python) — not run` on every suite. No
  issue owned it: the lane shipped as Issue 012, but it is opt-in and every
  run published since then left `--laya-python` off. Fixed by one M3
  `--laya-python` run (`fda5cf4`, `.benchmarks/025_m3_laya_python/`, a
  detached worktree at `b2fd694`, quiet-gate autofire). The run took
  17:51 to 18:12, and all 15 suites came back with python accuracy equal to
  rust on every suite and checkpoint. The start preflight read load 3.63;
  the end preflight was rc 1 at load 8.28, so treat the latencies as an
  upper bound for that window. It was published as an ordered lane-update
  (reflex-site `2129860`, worker `210a3437`), live-verified byte-identical,
  and the pairwise modelless drift gate passed. The publisher fix that came
  with it is reflex-site `20ba115`: a host row's `laya_python_lane` now
  flips from "off" when an update contributes python lanes. The python
  oracle stays absent on the 4090 (torch-MPS only) and on m3-ane.

- **Issue 021 CLOSED — the power axis nothing was recording (AC, plug- and
  thermal-gated re-bench of Issue 020).** Filed `e6aff52` on the owner flag
  *"beware thermal and unplug recently, rebench if need"*: `pmset -g log` showed
  the published `bench.json` (`77c408e`) was AC on both columns, but **every
  paired A/B in Bench 006 was taken on BATTERY** (unplugged 09:56 at 100% →
  48%). Landed: `scripts/bench_preflight.sh` (refuses on battery / Low Power /
  < `SETTLE_MIN` since plug-in / over `MAX_LOAD`; prints a `PROVENANCE:` line).
  ⛔ `pmset powermode` is a THREE-state enum (0 Automatic, 1 Low Power, 2 High
  Power) and the first gate refused High Power — fixed at `b819718`, only `1`
  refuses. No sudo-free throttle readout exists on this box, so the detector is
  a fixed-kernel canary (`317×1024×1024` `matmul_w`), judged as a **best-of-5
  minimum** pinned at `CANARY_REF_US=141` in powermode 2 (`122276b`: 30 single
  runs spread 18%, their best-of-5 minima 2.9% — contention adds time to some
  runs, a throttled clock raises the floor of all). AC re-bench = Bench 006
  Addendum 2 (`098b399`): narrow k=2624 GEMM −21.5% CONFIRMED; wide −9.5/−10%
  (larger than battery's −4.4 — the throttle compressed the delta); massive
  ≈ −5% p50 / −9% p99 (the battery "−8%" retired); §1's −64…−68% p99
  REPRODUCED; same-run `--laya-python` head-to-head showed the python oracle
  16–30% faster than its own published column, so rust wins p99 and loses p50
  ~10% (Class A open under Issue 020); no accumulated-state penalty at p50
  (Issue 020 T0 answered). T7 resolved advisory-stamped: `src/harness/box_state.rs`
  writes power / powermode / load / swap at run start+end into `results.json`
  `meta.box_state` with a `latency_quotable` verdict, never refusing a
  correctness run. The rule now names POWER SOURCE + POWER MODE (katgpt-rs
  AGENTS.md G2 `a12ae11`; this repo's AGENTS.md GOAT-gates bullet).
- **Issue 022 CLOSED — boundary: the `riir-infer` allowlist row now names the CRATE `riir-infer-laya`.**
  Filed at `fba613e` by the boundary-guard 145th run: `04531ae` (Issue 008 T4) landed the
  intended, pre-declared `riir-reflex → riir-infer-laya` edge, but the § May depend on
  row's Crate cell said `riir-infer` (the repo), and C3 matches the measured dep crate
  exactly, so the workspace gate read a declared edge as undeclared (exit 1). One-cell
  fix: Crate cell → `riir-infer-laya` (Location already named `../riir-infer` +
  `crates/riir-infer-laya`); drift row D1 removed in the same commit as the issue file.

- **CLM distilled — Research 001 filed + Issue 019 (the `clm` comparison lane).**
  Contrastive-LM/CLM (blog 2026-09-23, Kwok/Ré/Mirhoseini et al.; repo pinned
  @ `cca045f…`, Apache-2.0, clone deleted after pin) is the first open
  *contrastive* System One in the Jev lineage (katgpt-rs Research 562→573→576):
  frozen Qwen3-8B encoder + two ~20M projection heads, bidirectional InfoNCE,
  served behind the TypeSafe `systemone` wire `decision_wire` already speaks —
  the cheapest lane the arena can add. Vendor numbers (reference cells until
  OUR harness re-measures): Jev-parity zero-shot at up to 9× lower latency;
  DeepSWE 81.6% / Terminal-Bench 2.1 87.6% as best-of-N verifier where Jev
  sits below pass@1; T-Rex p50 16.5 vs 149.8 ms. Distill verdict YES (A−):
  the lane (019, Gate-0 owner-gated per the Issue-017 precedent) + the
  VectorArena disaggregation-cache pattern record + the head-only InfoNCE
  recipe (≈1 h on a single 4090 at 60M pairs; N* ∝ D^1.02) for the corpus
  flywheel's specialist path. Divergences held: their softmax-over-cosine and
  zero-abstention posture NOT adopted (sigmoid-then-L1 + first-class abstain
  stay); no runtime dep beyond the lane's HTTP adapter. Process note: the
  first allocation collided with the sibling session's `.issues/018` (filed
  on origin after this box's sync) — caught by the verdict reviewer pre-commit,
  renumbered 018→019 after the ff; the dual-allocation gate clears untracked
  allocations, so `ls .issues/ && git fetch && git log HEAD..origin/develop
  --name-only` is the real pre-commit check on this class.
- **Issue 014 — engine.rs local sigmoid delegates to `katgpt_core::exact_sigmoid`**
  CLOSED (substrate-first Mode 2 finding, fixed by the lane owner after the
  X-Reflex-Lane lane went quiet). The local single-branch fn deleted; the two
  call sites (route-term gate, per-option score normalization) call the
  substrate directly. Pin `sigmoid_delegation_matches_frozen_legacy_body`:
  bit-identical for x ≥ 0, measured max **3 ULPs at x=−16.68** on the negative
  band (−87, 0) — the same maximum the katgpt-rs Issue-870 pin measured on its
  domain — far tail (−96, −87] envelope-only (legacy saturates to exactly 0.0
  via 1/inf at x ≤ −88.73; the two-branch form stays representable; unreachable
  from the call sites). G2 p99 44 µs / G4 core alloc-free — neutral as
  predicted. Full guard PASSED (7/7 incl. G5 parity 27.8 s).
- **Wide-BK=48 rung measured NEGATIVE + the sgemm shape-timing probe**
  (uncommitted at write time; session record). The recorded rung ("BK=48
  for the wide instance — the largest k-chunk fitting 32 KB at 64×64")
  was executed: kernel edited, G5 green, then the new
  `examples/sgemm_shape_timing` probe (the forward's real `matmul_w`
  geometries: O k=1024, down k=2624 = the wide population; QKV/gate-up
  xwide + ag_news narrow controls) read the wide pair FLAT across 4
  position-balanced rounds — O ≈145–148 µs steady-state both sides,
  down dead flat ~399 µs, controls flat (the instrument discriminates).
  Mechanism: ~34% fewer staging barriers offset by +50% uncoalesced Wᵀ
  staging per iteration — barriers are not the wide instance's binding
  constraint. Constants REVERTED; the kernel is byte-identical to the
  pre-rung state; the negative is recorded in metal.rs docs + README +
  AGENTS.md so the rung isn't re-tried blind. What landed for real:
  the probe (with its three measured birth traps — the as_micros/1000
  ms-as-µs unit bug, the begin_pass-less stale-chain-slot aliasing that
  diverged shape 3 by exactly max|CPU − stale-b| on BOTH binaries, and
  the pipelined-block posture because per-op commit+wait measures
  submission overhead), the xwide smoke-arm re-aim in
  `tests/metal_ops_smoke.rs` (the (300,100,1500)/(512,64,1024) arms
  were orphaned by the XWIDE_N_MIN 1024→2048 floor — the kernel with
  ~70% of forward GEMM FLOPs had no tolerance gate), and Issue 015
  observation 7 (simultaneous cross-binary flake on unrelated ops with
  warm caches — kills the cold-shader-compile hypothesis; host-level
  transient). G5 parity green at the reverted state (2/2, 9.75 s);
  smoke 7/7.

- **v0.2.3 — the THREE-BOARD release cut** (`a386119` tag; dist release live).
  The deferred-until-Metal-landed tag, cut after the Metal sibling's
  `a51ea42`/`9ed1211`/`3ecab32` landed and the tree went clean. Release gates on
  the tagged tree: full default suite 108/0; clippy `-D warnings` at default /
  all-features / no-default postures; metal smoke 7/7 ×3 serialized + full
  laya-riir-metal suite **154/0** serialized; G5 parity green BOTH postures (cpu
  27.79 s / metal 9.92 s); leak scan PASS ×5; packaged-binary live smoke (stamp
  complete `default laya-riir laya-riir-metal modelless`, all three heads fitted
  at their published digests, lanes turn answered from the head over HTTP).
  Release surface: **dist repo `gist-rs/reflex`** release v0.2.3 (6 assets;
  SHA256SUMS regenerated v0.2.3-only after the cumulative-pkg-dir bug put five
  stale v0.2.2 rows in the first upload), tap `3054310` + bucket `5900c33`
  (both hash-verified against the SHA256SUMS), site version floor v0.2.3
  (`6db0cce`, deployed CF `ab5d8524`, prod curl-verified).
  ⛔ **The wrong-repo finding**: the first `gh release create` ran inside this
  checkout and created the release on **gist-rs/riir-reflex** (this repo) —
  the private source repo, where the tap's URLs 404. The dist surface is
  **gist-rs/reflex** (the install surface since v0.1.0; the AGENTS.md dist
  bullets say so — the release step must name `--repo gist-rs/reflex` or run
  from a dist checkout). The mistaken release was deleted (the TAG stays —
  v0.2.1/v0.2.2 both carry tags here; only the release object was wrong).
  ⚠ The `--clobber` SHA256SUMS half: `gh release upload --clobber` with a
  renamed file (`SHA256SUMS_v023`) created a SECOND asset instead of replacing;
  the canonical `SHA256SUMS` kept the stale cumulative content. Fix was
  delete-asset + re-upload + API-route verify (the download CDN served the
  pre-replacement bytes for minutes — `x-cache: HIT`; the API route returned
  the correct 548-byte file immediately).
  **Issue 015 observation 4** recorded in the same landing: the release
  pre-flight's FIRST serialized smoke run red 6/7 minutes after the sibling
  Metal session's parity runs ceased, then 3× serialized greens + 154/0 + a
  parallel full-suite pass with NO code change between — the run-to-run decay
  pattern (candidate (b), cross-process contention) is load-bearing, and the
  flake class can surface even serialized.
- **`04f9a4d`** — the laya posture's three `unused variable` warnings fixed at
  the root: `run_laya_checkpoint`'s dead `percentile_us` call deleted — the
  shared tail (`assemble_laya_lane_result`, both laya lanes) owns the latency
  metrics; both lanes' metrics must not diverge.
- **Laya-armed live coexistence validated** (the pre-release lane): engine
  booted with `RIIR_REFLEX_LAYA=1` (`laya lane: ready english, device cpu`),
  all three heads fitted alongside; one real lanes grammar turn answered by
  BOTH lanes — modelless routing `game-head/lanes` with per-lane scores
  (0.1553/0.1384/0.1553 — the head distinguishes train-blocked from
  rock-blocked), laya routing `requested lane=laya` from the real forward;
  off-grammar prompts decline to the honest abstain; raw skips the head try.
  The protocol change touches only the head-serving edge — laya's measured
  per-option shape untouched, now proven live.
- **Issue 011 — the lanes + flappy heads join the serve lane** CLOSED
  (`d1eda08`). Both boards left the abstain list at their published
  anchors, acceptance met in `tests/game_heads_serve.rs`:
  - **lanes** — katgpt-rs Bench 880's lossless decoded arm served at λ 0.01,
    in-corpus 84/100, head digest `7d3f1d8e…09d34` FULL (matches the
    published pin; the decoded arm is exactly lossless so the digest IS
    the structured arm's). Protocol: the JOINED-STATE path (011's option
    1) — one `/decide` per turn, `state` = the three lane sentences one
    per line (pinned left/middle/right), exactly three noul questions,
    answer i = lane i's P(safe); the lane-name fill must equal the line's
    position (a swapped turn refuses). Modelless lane only — the laya
    lane's measured per-option shape is site-side and untouched.
  - **flappy** — katgpt-rs Bench 882's v3 decoded arm served at λ 1, in-corpus
    96/100, FULL head digest `c93d36dc…e3c5` (the family's strongest
    anchor, exact match). Protocol: state = the state context sentence +
    the option sentence (two lines), one noul question, per-option
    requests — the head row needs the state's pre-rel/v/h beside the
    option's post band.
  - **Wire shape** (the protocol discovery of the lane): `noul` questions
    legally carry no options (`WireError::NoulCarriesOptions`), so the
    sentence sequence rides in the `state` field ONE PER LINE
    (closed-grammar sentences never contain newlines). 011's "joined with
    ; " sketch was refined to lines for exactly this reason — no case
    normalization, no punctuation surgery, one split rule for both games.
  - **Measured en route**: the fixtures' `features` column carries the
    STRUCTURED TRUE geometry; the reconstruction lawfully collapses the
    documented tails (post_rel at ±(h+1); |pre_rel| ≥ 2 clamped) — which
    is exactly why katgpt-rs Bench 882 pins TWO digests (`dc6bcf73…` structured,
    `c93d36dc…` decoded) at ONE 96/100 agreement. The lanes decoded ==
    structured per-row cross-check (881's losslessness) is kept in the
    parse; a flappy equality check would be wrong by design and is
    replaced by the full-digest pin.
  - **Site half** (reflex-site `edeb133`, deployed CF `a5f86875`): the
    live path speaks both shapes (modelless lane only); with an OLDER
    engine the new shapes fall through and abstain — the labelled random
    fallback the boards already render, no version gate needed.
    Verified live: arena_smoke PASS (flappy `flap 0.140 · coast 0.119`,
    lanes `left 0.124 · middle 0.138 · right 0.138` — real head scores
    over HTTP against the new engine); prod-page + local-engine smoke
    PASS; demo smoke + demo check + goldens 7/7 PASS.
  - `/healthz` now advertises `"heads":{"tetris":true,"lanes":true,
    "flappy":true}` (compile-time surfaces); `engine_gates`' body pin
    re-pinned.
  - Drive-by gate repair riding the same session (`c08419d`):
    `tests/harness_units.rs` carries its modelless gate now — the flag-OFF
    posture was red at import resolution since 67470be (the file imports
    the gated runner slice without the repo-birth pair); file-level
    `#![cfg]` + the paired `[[test]] required-features` row, the
    engine_gates shape.

- **Issue 014 — the engine lane-override knob `X-Reflex-Lane: raw`** CLOSED.
  The serve edge accepts `raw` alongside `laya`/`modelless`: it SKIPS the
  game-head try and answers from the raw modelless engine (the abstain IS
  the answer, never a head fallback) — the honest baseline the arena's
  third board renders (katgpt-rs Plan 607's ratified three-tier arena:
  laya teacher / fitted head / raw baseline). `/healthz` advertises
  `"raw":"ready"` (lane discovery); unknown lanes still 400 (fail-closed
  preserved); default posture byte-identical (no header = head-first).
  Per-lane claims hold by construction — the engine's response discloses
  itself (lane `modelless`, the engine's own routing reason), never the
  head's. Paired smoke pins BOTH directions on the request where the lanes
  diverge (fixture spot question: head-first by default, raw abstain under
  the override); flappy/lanes raw pins the abstain baseline (the WITHOUT-
  header side deliberately unpinned — it moves when `.issues/011`'s
  engine-side serving lands). Dispatch deduped: `engine_decide` shared by
  the raw path and the head's fall-through, `json_error` the one refusal
  shape. Unblocks reflex-site's 3-board arena layout (katgpt-rs Plan 607 roadmap).
  Files: `src/serve.rs`, `tests/serve_lanes.rs` (+2 pins),
  `tests/engine_gates.rs` (healthz body re-pinned).

- **Issue 015 — parallel-Metal smoke divergence RESOLVED** (`a3215da`; docs
  `00dff46`/`b4a5b10`; issue file removed per the noise-reduction rule —
  this row is the durable record). The root cause was NEVER GPU contention,
  shader-compile, or the driver: the smoke called ops STANDALONE without
  `begin_pass`, so the chain-cache epoch stayed 0 forever and
  `chain_buf`/`chain_slot_for`'s `(ptr, len, epoch)` keys silently HIT
  across arms — a recycled same-len host address served a stale DST slot
  holding a previous arm's device output (the `LAYA_METAL_TRACE=1` smoking
  gun: a red round's `matmul_kt` uploaded ONE of its two inputs; the green
  round uploaded both). Which arms collide is a PER-PROCESS HEAP-LAYOUT
  LOTTERY — one mechanism reproducing all seven observations (different op
  each run, deterministic values per binary+env, alone-pass/in-suite-red,
  serialized reds, obs 7's simultaneous cross-binary red). The quiet-window
  experiment REFUTED the cross-process reading — **the confounder WAS the
  finding**: 16/20 parallel + 5/5 serialized red on a QUIET GPU. Fix:
  `m.begin_pass()` at each arm boundary (the two composed chains begin ONE
  pass and compose device-side within it, exactly like a forward); post-fix
  **30/30 parallel + 15/15 serialized green** in the same window; G5 parity
  never flagged the class because forwards begin a pass per layer.
  **The containment decision rule is RETIRED**: "a parallel red after the
  gpu_lock is evidence of cross-process contention" was wrong — a red after
  that lock is evidence of an EPOCH-CONTRACT VIOLATION IN THE CALLER; check
  the trace for the missing-upload signature first.
  **Standing hazard recorded**: the `weights` cache keeps its permanent
  `(ptr, len)` map with NO epoch defense (deliberate — agent-lifetime
  weights), so any future op-level consumer that passes recycled same-len
  slices as weights re-opens this class one layer over; the module doc
  carries the contract note.
- **Issue 018 CLOSED — the 4090-windows bench lane is LIVE end-to-end (run,
  publish, deploy, verified).** Filed 2026-09-24 on the owner directive
  ("add issue to reflex bench on 4090 and update reflex.gist.rs/bench");
  executed by session `katgpt-rs-4090-b`. T1–T7 landed same-day: the full
  15-suite run @`afacc3a` (reflex `40828a0`, site `7508b7a` — the
  4090-windows host row; modelless accuracy bit-identical to m3 on all 14
  comparable suites, `code_fixtures` population-excluded; latencies the
  per-host story — typed laya english 421 ms metal vs 8127 ms cpu), plus the
  post-023 modelless-only update run @`8028a10` (reflex `10236c8`, quiet-box,
  massive_intent_en 0.0767 → 0.6900 confirmed on the second host, 13 other
  suites byte-identical). The ONE step left open at the last update — T6's
  `npx wrangler deploy` from a Node≥22 box with CF creds (a handoff, not
  this box) — was **discharged by the Issue-023-T5 together-republish
  deploy** (site `3ab373f`, worker `cde063c2`). **Live verification from
  the 4090 box 2026-09-24:** `https://reflex.gist.rs/data/bench.json`
  git-hashes `e5055982` — byte-identical to the repo file at `3ab373f`;
  three hosts served (m3 / m3-ane / 4090-windows) with `lane_sources`
  disclosing both post-023 modelless update runs.

## 2026-09-23

- **crates.io publication DEFERRED (owner-gates menu v2 row 2):** no
  `cargo publish` until an external adopter asks. The binary funnel already
  serves distribution (GitHub releases + brew tap + scoop bucket, all
  live-verified through v0.2.2), and the door stays open: `katgpt-core` — the
  one code-level dep — ships to crates.io, so a future publish needs no
  sibling-strip. Record-only; nothing to execute.
- **Issue 012 — the laya-python reference lane** CLOSED at `67470be`.
  The bench tables gain the ORIGINAL torch reference as a measurement-only
  subprocess oracle (`scripts/laya_python_lane.py`, harness
  `--laya-python`): SAME cases, ONE shared metrics tail
  (`assemble_laya_lane_result`), the reference's own rounded-4
  probabilities, 4 known-answer mapping tests. First measurement
  (prompt_injections, metal/mps): acc/ECE/F1 IDENTICAL to laya-riir
  (0.6983 / 0.2620) — the port is parity-true outside the fixture corpus;
  torch-MPS row batching leads our per-op dispatch ~2.7× at p50 (the
  Bench-001 optimization ladder's baseline, now published in the same
  table). Site spellings (`laya (rust)` / `laya (python)` /
  `modelless · none`) are display-only renames in reflex-site's
  `publish_bench.py` — the canonical results.json keeps machine fields.
  The "no Python anywhere" directive governs the shipped binary, not the
  bench reference (the `probe_orig_laya_latency.py` precedent).

- **Plan 001 — the fitted game head served over HTTP + the Metal-default
  laya lane + release v0.2.2** LANDED (`367766c` + `932a2a3`, tag
  `v0.2.2`). The arena's modelless board plays Tetris out of the box: the
  decoded Bench-881 head (λ=1, 44/120 in+LOO — katgpt-rs's published
  anchors, reproduced bit-identically by `tests/game_heads_serve.rs`;
  serving head digest `00aa6221…c6e`) is boot-fitted from the verbatim
  BLAKE3-pinned fixture copy and answers the pinned spot question on
  `/decide` before the cosine engine's honest abstain. Lanes + flappy
  stay abstains with measured reasons (cross-lane features 6–7 / v2
  render) — unblock paths in `.issues/011`. The laya lane defaults to
  Metal on macOS metal builds (`LAYA_DEVICE=cpu` opts out); G5 green at
  both postures; fresh interleaved 78.1 vs 176.8 ms row p50 (2.26×).
  Release: 6 assets, leak-scan PASS ×5, host + 4090 windows smoke
  (byte-identical head answer), brew/scoop bumped (audit clean),
  installers teach the `reflex` rename with a pre-v0.2.2 pin fallback
  (both paths live-verified), site copy deployed (b09bffb / e6ac2ce8).
  Plan: `.plans/001_game_head_serving.md`.

- **Issue 004 — the Jev harness decision-point map** CLOSED (T1/T2/T4/T5
  landed 2026-09-22 at `e4bf657`-window commits; T3 THIS COMMIT — the
  completion its deferral named). T3 (the cache-reuse `noul` lane,
  LLM-lane-only) was deferred on "the laya-lane numbers ride the next
  full harness run with weights present"; the weights
  (`~/.cache/riir-reflex/laya/{english,typed,multilingual}`) landed
  09-22 and this session ran the run: UNCAPPED
  (`laya_max_questions = 0`), 15 suites, PASSED no-absences, tables +
  results.json regenerated at HEAD `2aa2dda` — the sha the TABLES.md
  header carries next to the laya numbers, because `src/laya` moves to
  the riir-infer repo (008 T4) and the laya columns are labelled a
  PRE-MOVE BASELINE pinned at that sha (owner-verdict condition;
  hardcoded into the runner's header output, along with an explicit
  PARTIAL-run cap disclosure whenever `--laya-max-questions` is used
  and an honest "the laya lane answered below" blockquote for the
  LLM-only family). The T3 measurement itself:
  The T3 measurement itself:
  `harness_cache_reuse` laya·english acc **0.5000** on 12 binary
  fixtures — at chance, with the power stated (95% CI ≈ [0.21, 0.79]
  at n=12 — a weak refutation, one flip from 0.583) and the mechanism
  sharper than a coin flip: macro F1 0.3333 with mean confidence
  0.8618 (ece = 0.8618 − 0.5; brier 0.754) is a CONSTANT CONFIDENT
  one-class predictor over the 6/6-balanced fixtures (a constant
  `reuse` scores exactly 6/12), so the readout collapses to one class
  rather than hedging — the next attempt aims at the readout
  (per-class calibration / class prior / a cache-metrics feature), not
  at re-running the forward. p50 227 ms, det ✓. Full reading + power +
  box-state disclosure: Bench 001 Addendum 6,
  `.benchmarks/001_phase1_harness.md`. The six decision-point
  families live in `src/harness/families.rs`; gates in
  `tests/harness_families_gates.rs`; task-family framing per katgpt-rs
  Research 579 (nominative use). Full file life:
  `git log --follow -- .issues/004_jev_harness_decision_point_map.md`.

- **Issue 006 — remove candle from this repo at all cost (owner
  directive)** CLOSED (T1/T2/T4/T5/T6/T7 executed 2026-09-22; the
  release cut's T4 tail closed at `bf8ebe5` — tag `v0.2.0`, 5-target
  matrix, leak-scan cross-arch strip catch, tap `c30f0d6` on_linux fix,
  scoop `07cb8b6`; T4's own record:
  `git log --follow -- .issues/006_remove_candle_lane.md`).
  **T3 (tokenizers v1) remains DEFERRED on a measured upstream
  negative**, and per the owner verdict the reopen trigger lives HERE,
  in this row — a cited record must outlive the document it was parked
  in (it previously rode only 008's T4 task text, and 008 will itself
  be removed one day): **reopen trigger — tokenizers 1.0.0 STABLE (or
  an rc relaxing the byte-atom strictness): wire the same adaptation
  that measured the negative (facade `from_json` → `PipelineTokenizer`,
  `EncodeOptions::no_specials()`, the tk-convert v1→v2 in-memory
  canonicalize pass), run G5 at BOTH postures (CPU + Metal) before any
  publish — the GATE refuses the load today (`Byte atom 0xC0 not found
  in the vocabulary`: the english/typed GPT-2-family vocabs genuinely
  lack 14 ByteLevel byte atoms that 0.22 tolerates lazily and v1
  validates up front — upstream v1-rc incompatibility with
  legitimately-published tokenizers, not our wiring). The pin is back
  on 0.22 as a DIRECT dep (never via candle); its one-shared-build
  rationale died with T2 but the dep itself is load-bearing. The
  3-30× encode promise is unused headroom (encode is µs against a
  ~10-160 ms forward).**

- **The T2 disclosure is LIVE on the site + the committed tables — the
  landing gap closed (this commit + reflex-site `80d4fb1`).** The T2
  landing (`0dc2397`) had carried the PRE-swap baseline tables (run at
  ancestor `86e3727`): the validation outputs lived in the /tmp rig and
  were cleaned with it, so the committed results.json lacked
  `threshold_recommendation` and TABLES.md lacked the gate-fit column
  its own commit message promised. Full both-lane regeneration at
  `0dc2397` (15 suites, laya-riir, PASSED no-absences, ~62 min wall;
  modelless accuracy/ECE bit-match the published run). Box state: M3
  LOADED (loadavg 10.9-15.1, sibling cargo builds throughout) — the
  laya latencies carry that state; the modelless determinism lanes do
  not. Site half: `data/bench.json` republished via publish_bench.py +
  the bench page renders the per-suite Gate fit line (posture rho cited
  with the numbers, per-axis threshold/accuracy/support, thin ->
  defaults on a null axis; laya lanes and pre-T2 data degrade to no
  line). Live-verified on reflex.gist.rs (bench.json meta sha
  `0dc2397`, gateFit deployed, HTTP 200). Site README rider: the
  harness flag is `laya-riir` since `.issues/006` (the `--features
  laya` line was stale).

- **Issue 010 — the agent-skill section (the laya-page steal:
  "Give your coding agent a decision engine")** CLOSED at `177156d` +
  `c65bfcd`-followed close (filed `4e7cc6c`; verdict fixes `c65bfcd`).
  `.docs/04_agent_skill/SKILL.md` authored as the source of truth in-repo,
  versioned with the engine — every wire example captured against the real
  binary built from clean HEAD `2c6acb9` in the detached worktree
  `riir-reflex.w010`, not hand-written. Two traps measured live and taught:
  feedback-lies collapse (invented p values → one refit → confidence
  1e-3→1e-10) and the 64-observation refit floor (`method` `none`→
  `sigmoid-gate`, temperature 0.306). The ρ=30 threshold recipe cites
  `engine::threshold_recommendation` (`a732bcf`, Issue 009) as the
  normative surface with the quantile law reproduced over the wire.
  Numbers table is losses-published (G1 ag_news FAIL included, run
  `86e3727`). Site half LANDED live in gist-rs/reflex-site (`f2b9063`):
  the `#skill` section after the arena, both agent-target curl one-liners
  (`.claude/skills/` + `.agents/skills/`), the FAQ pair, nominative credit
  to the laya page; SKILL.md served at the stable URL
  `https://reflex.gist.rs/skills/reflex-integration/SKILL.md` (200
  `text/markdown`; the one-liner verified end-to-end — fetched file
  byte-identical to the source of truth). Rider fix `47247a8`: the site's
  assets root had been serving repo plumbing as public assets
  (`GET /.git/HEAD` → 200 with contents — pre-existing since the first
  deploy); `.assetsignore` closed it (`.git`, `scripts`, `wrangler.toml`,
  `README.md` now 404; site surfaces re-verified 200). The release-cut
  curl re-verification stays a stated process in the skill's Freshness
  section (mechanizing it into `build-release.sh` deferred — the sibling
  candle-cut WIP owns that file).

- **Issue 006 — candle removed ENTIRELY (owner directive "no candle at all
  cost") + the candle-free release cut v0.2.0** CLOSED (file removed
  2026-09-23 with this row; executed 2026-09-22, T1/T2/T4/T5/T6/T7
  done; the release cut's T4 tail closed at `bf8ebe5`; T3 deferred on a
  measured negative — **the reopen trigger is now THE row above**, this
  09-23 entry, which is where it lives durably). Deleted: the `laya`,
  `laya-metal`, `candle-metal` features, the `candle-core` dep,
  `src/laya/{agent,encoder,head}.rs`, `tests/laya_parity.rs` + its
  `[[test]]` row; the harness's laya lane re-pointed at `RiirAgent` (same
  `system_one` envelope); `RELEASE_FEATURES` flipped to
  `modelless+laya-riir`; about.toml/build-release/guard/workflow re-derived;
  BOUNDARY gained an explicit never-candle row; docs amended across
  AGENTS/README/pin-doc/sibling-layout. Verified end-to-end in the same
  cut: `grep candle Cargo.lock` = 0, clippy green at all five postures,
  full test suite green, **G5 green at BOTH postures post-removal** (CPU
  58+2 / Metal 2/2), guard [1-7] PASSED (full). **T3 negative on record:**
  tokenizers 1.0.0-rc.2 was wired end-to-end (facade API + the tk-convert
  v1→v2 in-memory pass) and REFUSED to load the pinned english/typed
  tokenizers — their GPT-2-family vocabs genuinely lack 14 ByteLevel byte
  atoms (`j`, `}`, `~`, `²`…), tolerated lazily by 0.22, validated up
  front by v1. Upstream incompatibility, not wiring; reverted to 0.22
  (always a DIRECT dep); reopen at 1.0.0 stable. Chart's candle column
  frozen as history (T6). The tokenizers/GEMM version-match rationales
  died with the lane (T5) — gemm/metal/objc2 float freely with a G5
  re-run on any bump; libm stays pinned on numerics grounds.
  **The release cut landed the same day:** tag `v0.2.0` at `c25e1c3`,
  5-target matrix all leak-scan PASS (the scan caught the zigbuild
  x86_64-apple-darwin binary keeping a symbol table — profile `strip`
  doesn't fully apply cross-arch; stripped before publish), live-smoked
  on mac arm64 + Rosetta, alpine musl ×2, windows on the 4090, decide
  byte-identical mac↔linux; released on gist-rs/reflex; homebrew tap
  bumped to 0.2.0 WITH the formula gaining its missing `on_linux` blocks
  (brew's arm64_linux audit rejects the macOS-only shape — the cargo-heal
  pattern, tap `c30f0d6`); scoop bumped (`07cb8b6`).

- **Issue 005 — the riir forward's own Metal backend (`laya-riir-metal`)**
  CLOSED at `4f22e9b` (filed `73ada82`; deps boundary-first `07039c1`;
  backend T2–T4 `a3e7c49`). 14 MSL kernels, per-op committed command
  buffers in candle's lazy-flush shape; G5 GREEN at the Metal posture
  (drift ≤ 5.981e-6 vs the 1e-3 gate) — the gate caught four real defects
  en route (weight-cache stale-serving recycled activation addresses,
  sync-count eviction vs forward-lifetime slots, untracked hazards across
  command buffers, layer-0's host copy reading stale residual bytes). The
  fair all-Metal three-way (Bench 001 addendum 6, same-session
  interleaved): torch MPS 25.7/25.5/16.2 · candle Metal 31.1/31.2/18.7 ·
  riir Metal 79.0/78.7/38.5 ms row p50 — the honest naive-v1 baseline the
  optimization ladder measures against. Full narrative: AGENTS.md
  §"The laya-riir lane".

- **Issues 002 + 003 — the `laya-riir` kernel deps (`gemm` 0.18, `libm` 0.2)**
  CLOSED at `e601295` (the lane landing; both filed, rowed in BOUNDARY.md,
  and closed per the boundary-gap-first pattern — issue, row, code, one
  commit). 003's standalone file was born with a stale OPEN line (the
  CLOSED record lived embedded in 002's file — a drifted duplicate); both
  files removed 2026-09-22 with this record. Both deps version-matched to
  candle's own CPU calls (one shared build); the A&S f64 gelu measured
  1.4e-2–3.5e-2 drift, ~1000× over the gate, before `libm::erff`.
  Acceptance: G5 top-1 1.000000 ×3 checkpoints, prob drift
  1.03e-6/1.83e-6/3.01e-6 against the 1e-3 gate. The `libm` pin SURVIVES
  candle's removal (Issue 006): its reason is bit-parity numerics, not
  build-sharing.

- **Issue 001 — undeclared crates.io deps (missing BOUNDARY rows)** CLOSED
  at `8e58cc5`; the rows landed in the same window as the katgpt-rs Plan 603 T1.4
  verdict-review closures (`1ef3e8e`).

- **Issue 009 — the threshold-recommendation surface (the jimothy steal)**
  CLOSED at `059794d` (T1+T3+T4 landed `a732bcf`; T2 this commit). The
  four-part CONTRACT stolen from jimothy (fit-time recommendation, return
  everything at run time, null on thin support, validate in the
  deployment environment) — the PATTERN, never the trained method. T2
  migrated the harness to the engine surface BYTE-IDENTICALLY: pure-swap
  diff over 14 suites (`--skip-laya`) NORMALIZED-IDENTICAL on
  results.json with the noise set {latency, seconds, date, git_sha}
  validated by a double-baseline run; TABLES.md identical beyond the
  environmental run-line sha; the disclosure step measured ADDITIVE-ONLY
  (exactly 14 new `threshold_recommendation` keys, zero changed values).
  The runner's old inline `quantile` is deleted; its law lives on as the
  frozen oracle in the 3 in-module migration gates
  (`harness::runner::threshold_migration_tests`). The LaneResult now
  carries jimothy's `thresholdRecommendation` shape (posture + per-axis
  threshold/status/targetAccuracy/support, camelCase, per-axis null on
  thin support) and TABLES.md gains the `gate-fit (rho=.30)` column.
  Residual (out-of-repo): arena-site rendering of the metadata rides the
  reflex-site repo's next data/bench.json regeneration — publish_bench is
  site-side.

- **Issue 012 remains + the G1 no-claim fix** LANDED 2026-09-24 (commits
  `bb2370a`, `fd3ae48`, docs commit; record: Bench 001 Addendum 7). The
  calibrator-never-fitted state serialized `g1_pass: Some(false)` — a FAIL
  for a claim that was never made, against the run meta's own
  calibration_protocol promise — now `G1Verdict::{Pass,Fail,NoClaim}` via
  the pure `g1_verdict_of` (31/31 units; g1_pass keeps its wire shape).
  Census 6 PASS/8 FAIL → 7 PASS/2 FAIL/5 NO CLAIM; code_fixtures
  FAIL→PASS is its mined-source questions moving with the day's edits.
  `route_scale` became `EngineConfig.route_scale` (default 8.0 unchanged);
  the synthetic-family sweep probe measured FLAT — promotion declined,
  evidence in issue 013. Latency refresh runs (17:21Z quiet; committed
  artifacts are the 18:09Z isolated-worktree re-run `83173e5` after a
  sibling's 2-suite re-run clobbered the first run's working-tree
  artifacts — tree restored, no data lost; typed rust p50 1164/1312 ms
  across the two readings), accuracy bit-identical on every lane across
  all three same-day runs. Issue 013 (accuracy levers) filed;
  levers 1/3 open, lever 2 recorded dead-on-this-evidence.

- **Issue 013 — modelless accuracy levers: all three levers measured, issue
  RESOLVED + removed** (2026-09-24; instruments + verdict `d5a704f`, fmt
  sweep `104da7a`; record: Bench 005). Lever 1: the cap is a per-suite
  tuning knob — ag_news default 64 selection-CONFIRMED, banking77 128
  promotion REFUSED by the stratified protocol (Bench 004). Lever 2:
  route_scale flat, declined. Lever 3: the dataset pair-head candidate —
  the confusion probe found real concentration (xnli 95% of errors →
  neutral, emotion 73% → joy; centroid collapse), but the A/B REFUTED the
  fix: diagonal-LDA heads fitted from the pair's own corpus docs, armed
  from CAL-slice confusion (the lever-1 protocol law), fired under BOTH
  gates, lose everywhere that matters — global net ≈ −19 questions, no
  GOAT cell. Mechanism: the engine is not at chance on the fired subsets
  (46–81% gold-in-pair); where it IS at chance the hashed-bag head is at
  chance too; where the engine is strong the head is strictly worse. The
  lane sits at its feature-class ceiling — lever 4 (a more expressive
  embedder) recorded in the issue's final state, unstarted, owner-gated.
  ⛔ CORRECTED by Issue 023 (Bench 007): the "ceiling" was read with the
  centroid signal OFF on massive (k ≠ N guard) and index-misaligned on the
  banking77 CAL slice — name-resolved routing moves massive 0.077 → 0.690.
  Instruments stay, report-only, default posture byte-identical: the
  always-on `modelless.confusion` readout + `harness --pair-head-ab`
  (both firing gates, gold-in-pair split, n_counted disclosure;
  deterministic byte-reproduced). Full guard PASS incl. G5 parity.
