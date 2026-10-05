# HISTORY.md — riir-reflex

Durable records for closed issues (the noise-reduction rule: a resolved
issue file is removed from `.issues/`; its record lands here, hash-pinned).
A removed file's full life: `git log --follow -- .issues/<file>`. Open work
work lives in `.issues/` and `.plans/`, never here.

## 2026-10-05 — Issue 068 CLOSED (the lane HTTP micro-client extraction: one `parse_http_response`, chunked handling for every lane)

The substrate-first mode-2 drift audit's finding — four diverged per-lane copies of
`parse_http_response` plus per-lane `request()` (openthai/agentjev/clef/clm, all four
hash-different; openthai's own module doc recorded the extraction as its DEFERRED
T4.1) — resolved by the extraction the issue proposed, on the owner's "fix this"
call rather than the recorded next-lane-change trigger: `src/lanes/http_mini.rs`,
the shared std-only HTTP/1.1 micro-client (fix `d12e5d6`).

- `request(host, port, method, path, body, timeout, extra_headers)` + one
  `parse_http_response`; the per-lane knobs are parameters and the per-lane error
  conversion stays at the call site — clm keeps `ClmError` via `From<LaneHttpError>`,
  clef keeps its connect-failure forwarder hint by matching `LaneHttpError::Connect`.
- **The clef variant's chunked decoding folded in for EVERY lane**: a content-length-
  only parser silently mis-truncates when a forwarder chunks (the clef law, now
  everyone's); `decode_chunked` moved verbatim.
- `LaneHttpError`'s Display preserves the historical message formats byte-for-byte
  (`connect {host}:{port}: {e}` / `set timeout` / `write` / `read` / the parser
  messages) — those strings flow into run logs and operator refusals.
- G-ISO-4 import law amended (openthai/agentjev/clef): imports now include the
  std-only `crate::lanes::http_mini` sibling — the law's intent (the lanes compile
  ungated with zero feature surface) is unaffected; a std-only sibling cannot pull
  a feature in. **paw stays deliberately NOT a consumer** (curl subprocess
  transport; its `-i` parser must fold redirect chains — a different shape).
- Regression gates: the parser goldens consolidated into http_mini's own battery
  (split-head, chunked+truncate, malformed arms incl. bad-chunk-size/truncated-
  chunk, connect-Display, a stub-listener pin of the REQUEST wire — extra-header
  knob, POST body headers, GET bare, chunked reply on the GET path); the lanes'
  stub round trips — openthai 10, agentjev 7, clef 10, clm 12 — all green THROUGH
  the shared client (the clm wire pin proves request line + body shape + the
  `X-CLM-Latency-Ms` header read survive the extraction).
- Validation: clippy `-D warnings` at default / `clm-lane` / `--no-default-features`
  / `--all-features`; full `cargo test` (16 targets) + `--features clm-lane` (302);
  wasm32 `--no-default-features` check clean (the ungated lanes + http_mini compile
  there; the modelless-on-wasm32 failures are pre-existing harness-runner breakage,
  why the site ships a separate `wasm-head` crate).
- Known unrelated red the guard still carries: layer 9 site-mirror drift on
  `.docs/05_resources/dev_flow.md` — PRE-EXISTING (reflex-site `3baffcc` edited the
  mirror side only); not touched by this change.

## 2026-10-05 — Issue 066 CLOSED (the fused-abstain density half: wired, measured, not certified — the many-label signal recorded)

**[Bench 124](.benchmarks/124_density_gate_ab/RECORD.md)** — the LSL App-E density half
(`arXiv:2610.02126`) landed as the fused gate's THIRD axis behind the `density_gate`
feature, over the katgpt-rs `gmm_support` substrate (Plan 618 `1d8da4862`, Bench 908):
per-domain `SupportGate<64,16>` (positive = the domain's JL-projected rows; negative = the
POOLED projected rows, shared) + the engine-level `JlProjector<256,64>`, fitted in
`build_specs_impl`, `AbstainCause::DensityGate` as the cause chain's LAST marginal arm (the
Issue-060 short-circuit law preserved), `EngineConfig {density_gate, density_threshold,
density_tau}` (default false — byte-identical unarmed, pinned), fail-closed
`DensityNeedsCorpora` on raw-expert builds, and the report-only `--density-gate` A/B
(`harness/density_ab.rs`): the density engine at the SAME fitted posture (calibrator re-fit
on the same cal pairs), the density threshold at the cal ρ=30 percentile, ONE frozen test
read, per-question pairing with `pick_disagreements` as the void tripwire (0 everywhere).
Five engine unit tests pin the pairing premise (picks/probs/domains never move), the
unarmed posture, the marginal-cause classification, the fail-closed refusal, and the
accessor's determinism.

**Verdict: NOT CERTIFIED — opt-in/report-only stays.** LCB95 < 0 on all six suites at the
ρ=30 per-axis posture (+8–18pp union abstention nowhere certifiably paid back). The
directional signal splits exactly along the PRE-CHECK's whole-corpus line:
massive_intent_en **+0.0408** / banking77 **+0.0344** selective accuracy (marginal slices
49%/37% correct vs base 61%/55% — the gate removes genuine errors) on the many-label
multimodal side; ag_news +0.0177; sst5 flat; emotion −0.0110 / xnli_en −0.0202 (the
few-label near-unimodal suites REMOVE better-than-average slices — per-label ≈ pooled ⇒
ratio ≈ noise there). Two en-route findings worth keeping: **(1)** the consumer var-floor
re-pin (`DENSITY_VAR_FLOOR = 1e-2`) — the substrate default let K=16 positives spike on
40–64-doc pools (question embeddings read ℓ ≈ −100s, >30% of cal confidences underflowed
sigmoid, the ρ=30 fit landed on a denormal — abstention without discrimination); K=16 +
floor 1e-2 beat K=4 and floor 2e-2 on the full board. **(2)** τ is PROVABLY inert under
percentile threshold fitting (`sigmoid(ℓ/τ)` strictly monotone ⇒ same rank set for every
τ>0 — the sweep task closed by argument). The wire-shape law held: the shipped rows'
`abstain_causes` keep the three-key closed taxonomy (the `density_gate` key serializes
only when nonzero — pinned both shapes). Re-open triggers in Bench 124 §Verdict (per-suite
arming with cal-side selection; the out-of-suite negative reference; a consumer at the
800-sample regime). The issue file is removed with this record per the noise-reduction
rule; its full life: `git log --follow -- .issues/066_fused_abstain_reference_density_threshold.md`.

## 2026-10-05 — Issue 065 CLOSED (the quotable-timing backlog: every comparison lane now plots)

**[Bench 123](.benchmarks/123_lanes_rerun_4090_boxstate/RECORD.md)** — the four 4090 lane
re-runs, one window, all self-judged quotable by the T3(b) Windows probes (`00bedc0`'s first
production proof: four real harness results carrying `BoxState` from the PowerShell
single-spawn, two honest load-refusals on the way — 7.44/8.16 end-captures against
sibling+staging load, both re-run clean):

| lane | cells | p50 geomean | notes |
|---|---|---|---|
| openthai @4090 | 11/11 | (extra-host rollup) | every cell == the published lane; the home row's 4-suite accuracy-pick drop resolves |
| clm @4090 | 8/8 | 44.46 ms | vLLM stack re-staged from the surviving docker image + HF cache (no 16 GB re-download) |
| agentjev @4090 | 8/8 | 45.17 ms | their repo @ `a965ca8f` + step-600 tensors re-wrapped; typed 0.7720 (the one-question bf16 wobble class) |
| gliner @4090 | 8/8 | 22.85 ms | 6 cells reproduce bench-037 exactly; massive 0.7267 / banking77 0.7100 are the corrected-decode cells (below) |

Every run's modelless control cells bit-identical to the published board (the drift guard's
premise, all four docs). Posture: the board-canonical `--nb-select --oc-select --ridge-select`
over the canonical `.raw/datasets`; `--skip-laya` (the laya@4090 cells were never the backlog).
Publish: `PUBLISH_BENCH_LANES="openthai,agentjev,gliner,clm"` update publish, publisher
self-test 84/84 first, all site gates green afterward (the `clm@4090-win` lane profile view
renders on /bench).

**The window's one real product find — the cp874 STDIN decode (fixed, landed with the
close):** the gliner oracle crashed on typed_decisions + prompt_injections with first an
`IndexError` inside gliner2's schema-embedding extraction and then, under the crash-capture
instrument, `UnicodeEncodeError: '\udc99' surrogates not allowed` — the JSONL lane scripts
(`gliner_lane.py`, `bekko_lane.py`) reconfigured stdout/stderr to UTF-8 but NOT stdin, so on
this box Python decoded the harness's UTF-8 JSON pipe with the ANSI codepage: case text
mojibake'd, some bytes surfacing as lone surrogates, and the mis-decoded text's token/schema
misalignment read upstream as the package's IndexError. Fix:
`sys.stdin.reconfigure(encoding="utf-8", errors="backslashreplace")` in all three
subprocess-lane scripts (gliner + bekko + laya_python — the family's missing half of the
cp874 law). **Consequence:** every gliner/bekko cell previously measured through a cp874
stdin pipe carried mojibake'd non-ASCII text — the ASCII-majority suites reproduce their old
cells exactly, and massive_intent_en (0.8233 → 0.7267) + banking77 (0.7060 → 0.7100) MOVED on
the corrected decode; the new numbers are the honest ones (the oracle finally reads the
dataset's actual text) and are what the publish landed.

Re-stage ledger (Bench 123's RECORD carries the full notes): fresh `uv venv`s on Windows pull
CPU-only torch — the CUDA build needs `--index-url https://download.pytorch.org/whl/cu124`
(measured: the agentjev boot died with "Torch not compiled with CUDA enabled" until the
index pin); the agentjev torch-wrap is `torch.save({'state_dict': load_file(...)})` (343
tensors, re-runnable); the CLM pin `cca045ffd…` resolves via the GitHub API when the
abbreviated ref fetch refuses.

## 2026-10-04 — Issue 064 closed (the projection-ascent density gate: works as designed, buys no accuracy)

- **Issue 064 (projection-ascent corpus synthesis)**: task 1 pilot — kill gate PASS with
  wide margin ([Bench 120](.benchmarks/120_synth_density_pilot/RECORD.md): 38.3%/41.5%
  accept at ε_nat, the minimal-deviation signature at 0.66×/0.30× natural-neighbour scale,
  src-misses 0). Task 2 — the flag + corpus-ab V5 + the pre-registered echo-gate rig
  (abstention-entropy KL + the OOD word-dropout ladder) landed, then the teacher-gated
  dose-response pair: **p50** ([Bench 121](.benchmarks/121_synth_density_p50_corpus_ab/RECORD.md),
  1,859 rows: V5 PASS marginal LB95 +0.0016, all echo gates green, ~7× weaker lift than
  ungated) and **p75** ([Bench 122](.benchmarks/122_synth_density_p75_dose_response/RECORD.md),
  1,980 rows: V5 PASS LB95 +0.0062, OOD gate-rung +0.0117 ≈ ungated's +0.0130, **zero
  gold-only regressions**). **The verdict: the lift tracks row count — p50's deficit was
  walk starvation (quantity), not selection; and UNGATED (0.8133 / +0.0110) beats every
  gate strength at every accuracy cell — the fusion's premise (density-gated selection
  improves corpus quality for the frozen consumer) measures NO gain where ungated
  synthesis already transfers cleanly. Corpus health is IDENTICAL at every strength
  (KL 0.0000, retention 1.0) — the gate neither helps nor hurts health, it only shrinks
  the corpus. A remedy without a disease on this suite.** The SEATED production corpus
  stays the ungated artifact. The code stays (`--synth-density-gate`, `DensityGate`, the
  echo-gate rig — zero-cost instruments for any future gate experiment). En-route
  production proof: **density-first ordering** (the gate before the teacher forward) —
  6,982 + 2,965 pre-forward rejects ≈ **11 h of teacher calls not spent** across the
  pair. Task 3 (share the operator with riir-train Plan 438 Phase 2's RIDT v2) stays
  deferred-DRY: the second consumer has not landed, and the measured negative weakens
  the pull — re-open only if the training side lands and wants the gate. Ops notes: the
  p75 run was interrupted at ~505/2048 by an owner silent-rig call and resumed same-day
  via the recorded protocol; the rerun reproduced the interrupted trajectory
  block-for-block (the determinism witness). The issue file is removed per the
  noise-reduction rule.

## 2026-10-04 — Issue 054 closed (all four parts terminal)

- **Issue 054 (openjev lane + prefix-state handoff)**

- **Issue 054 (openjev lane + prefix-state handoff)**: Part 1 lane candidate — owner call
  2026-10-02, TERMINAL (do NOT add to the arena; the 27B model is too huge for the fleet;
  re-arm would need a smaller Jev-family model, none exists). Part 2 prefix-state handoff —
  REFUTED for laya 2026-09-30 (measured: the laya encoder is ModernBERT, bidirectional, NOT
  GDN — no exact prefix state exists; the lead's SHAPE stays valid for causal decoders, a
  re-file should target the league's GDN surface, not laya). Part 3 serving-technique
  follow-through — LANDED 2026-10-02 (`.research/006`: prefix-tree root sharing implemented
  in the modelless engine `solve_sample_into`, bit-identical + regression-gated
  `per_question_slots_are_independent_of_their_neighbors`; GDN chunked prefill absorbed by
  riir-infer-gpu v3; the rest not transferable/already adjudicated). Part 4 perch wire intel
  — recorded (question-batching is a first-class System One wire contract; the
  `semantic_defects` authoring seed filed via issue 061). Full record: `.research/006` +
  this entry; the file is removed per the noise-reduction rule.

## 2026-10-03 — the v0.2.4 release cut

- **Shipped:** tag `v0.2.4` → `e616814` (a docs-only sibling commit landed in the shared
  worktree over the bump commit `4bea9fc`; `git diff 4bea9fc e616814` touches only
  `.benchmarks/` + `.plans/` — the shipped code is identical, and the published tag
  was deliberately NOT force-moved over it), GitHub release with 6 assets (5 target
  archives + SHA256SUMS),
  brew tap `813cb68` (fetch-verified 0.2.4) + scoop bucket `596e156`. The
  first-corpus lane (Issue 063: `RIIR_REFLEX_CORPUS`, the distance-gated
  first-corpus posture, the `/healthz` corpus disclosure) and the wire id fix
  (Issue 062) are the headline lanes; the game heads are MINT-ONLY now (the
  boot-fit retirement ships), the archives carry LICENSE (MIT).
- **Measured release posture (the wire diff v0.2.3 → v0.2.4):** the out-of-box
  tetris answer is GONE (heads abstain without a minted dir — the release
  notes' breaking change), `healthz` gained `"corpus":…`, the unknown-lane
  message lists `laya-ane`, question-level violations are 422, noul
  probability tails shifted (route-term polarity + option-name routing).
- **Gates:** full `ci_feature_guard.sh` PASSED (9/9 — including the flag-OFF
  layer this cut UNBLOCKED: `fixture_digest`'s blake3 touchpoint was ungated
  since the 2026-10-02 hygiene landing, red at HEAD, fixed modelless-gated);
  G2 PASS p99 44 µs ≤ 1000 µs / G4 PASS alloc-free with the box state quoted
  (`PROVENANCE: power=AC load=5.27 powermode=2(high) canary=113.9us/best5`,
  preflight PASSED); leak scan PASS ×5; no tag/PR collisions at the pre-push
  check. Disclosed: the darwin binaries embed two id-shaped strings inside
  Metal-kernel source COMMENTS (`Issue 020`, `Plan 616` — the laya lane's MSL
  strings; wire-inert, page-inert, same class shipped in v0.2.3; stripped in
  the laya substrate, not patched here).
- **Site half:** reflex-site `cce66a5`+`dfdd6d5` (CF `15bd631f`→`c345c993`):
  the capture mints throwaway demo vessels and boots a second engine on the
  vendored first-corpus sample; the walkthrough is live (`/first-corpus.tar.gz`,
  the home block, `/docs/api/#corpus`); `WIRE_ID_ALLOW` is EMPTY.

## 2026-10-02

- **thai_sib200 pool floor re-based onto the audited pool-after-cal (560 →
  400; `slice_guard::MIN_POOL_ROWS`)** (2026-10-02): the 560 was 80% of the
  RAW pull, but the audit measures the pool AFTER the cal front is removed —
  and the suite's SOURCE yields only 701 train rows total (re-fetch
  TRAIN_CAP=20000, proven by the riir-infer Plan-617 A5 board's slice
  refusal), so pool-after-cal maxes at 501 (701 − 200 cal) and 560 was
  unsatisfiable at ANY pull. The mispin survived because Bench 100 never
  exercised sib200 (the floor postdates the 084 run; no run measured the
  suite between the floor landing and the 617 board). 400 = 80% of 501; both
  shrink classes stay covered (the cal front is a fixed 200 rows, so
  pool = raw − 200 and a raw-pull loss >14% still refuses). The justification
  is the SOURCE MEASUREMENT, not a lane want — this is NOT a gate loosened to
  admit the 617 board: that lane's re-open is a NEW pre-registered decision
  (riir-infer issue 034) and stays decoupled from this fix. slice_guard tests
  16/16.

- **Bench 110 — the OpenThai-SystemOne EXL3 4.0-bpw convert board (riir-infer
  Plan 617 Phase A5) RECORDED: NO-GO by the pre-registered letter;
  PROVISIONAL (GPU-shared)** (2026-10-02): the 4090 session's board runs land
  here (`617_openthai_exl3_4090{,/retry,/thai}`, harness @ `f730497`,
  determinism pin green every row) — 6/11 suites clean, typed/sst5 exactly at
  the ±1.0 pt bar, ag_news +1.50 (improvement direction — still divergence on
  a lossy surface), code_fixtures −3.13 (one question of n=32), ECE bar PASS
  11/11, sib200 absent (the floor mispin above), semantic_defects unpinned.
  Verdict: NO-GO stands (owner-delegated Claude verdict, AGREE — even with
  both breaches excused GO is unreachable: 2 of 13 suites are unmeasurable
  today); re-open conditions recorded in the record (paired-discordance bar,
  sib200 measured post-fix, consumer-tied). Latency columns carry no box
  state (probes unavailable under the sibling training) — unquotable.

- **`.benchmarks/.highwater` normalized to a single-line counter (was
  `109\n144\n`); `scripts/thai_rerun.sh` reader hardened** (2026-10-02): the
  144 second line appeared in the f730497 highwater repair with no allocation
  behind it — nothing on disk or on origin is numbered 110–144 (dual-
  allocation gate clean at allocation time), so the line was corruption, not
  a reservation; the true max allocated was 109. The latent hazard was real:
  thai_rerun.sh read the counter with `tr -d '[:space:]'`, which CONCATENATES
  a multi-line file ("109144") and would have allocated garbage on the next
  thai rerun — now `head -n 1` first. Disclosed per the collision law: if a
  sibling session somewhere allocated 144 in flight, the citation-rewrite
  repair applies (the 062 precedent).

- **The six harness families RETIRED (owner call, this session):**
  `harness_visibility` · `harness_permissions` · `harness_tool_fit` ·
  `harness_routing` · `harness_sensitivity` · `harness_cache_reuse` are
  REMOVED from the harness so no one benches them anymore — the runner
  SUITES rows, the `families.rs` defs + `families_eval_wide.rs` wide
  evals, the gates' family arms (incl. the
  `cache_reuse_grounded_posture_discriminates` production-seat grounding
  gate and the wide-law/divergence pins), `benches/harness_families_goat.rs`
  (the families' G2/G4 lane — `decision_set_goat` remains THE G2/G4
  lane), the fixture-fleet cache_reuse history pin, and the
  `route_scale_probe` sweep rows all went in one change. Verdict that
  drove it: home-made synthetic evals (authored in this repo, Issue 004
  / Research 579) the modelless engine reads AT CHANCE on at the honest
  wide-eval populations — 0.3125/0.3125/0.302/0.281/0.22 vs
  ~0.2–0.33 chance — while the small-n template-shared reads (0.56–0.92,
  n=12–16) that had looked strong were the artifact (instinct issue 008
  T8's unfalsifiable-memorization class), and cache_reuse's 0.9167/0.5000
  pair was n=12 noise either way. The eval DISCRIMINATES (laya-rust read
  0.51–0.66 on 4/5 at the same populations), so the at-chance verdict is
  about the lane, not the fixtures. Consequences accepted and recorded:
  the seat-selection machinery the families exercised is name-agnostic
  (rides the `synthetic` flag) and stays covered by
  `harness_seat_gates` over `semantic_defects`; the wide-law gates +
  digest pin continue over `semantic_defects` (Issue 061, which STAYS);
  the BLAKE3-pinned `code_fixtures_frozen.json` still quotes the retired
  names inside its frozen source snapshot — history, untouched by law.
  `.benchmarks/` records survive as history. Downstream: reflex-site
  board rows removed + /families/ section retired (its HISTORY),
  instinct arena population 15 → 9 (its HISTORY). Gates: full `cargo
  test` green (15/15 binaries, 0 failures) + clippy `--all-targets`
  clean.

- **Issue 061 LANDED — the `semantic_defects` code-defect family (this session, shikuwa/4090):** the perch-derived defect taxonomy became a 7th harness family — 6-way Choice (`clean` + `off_by_one` / `inverted_condition` / `unwrapped_none` / `swapped_lookup` / `swallowed_error`, Rust-native spellings), **102 eval (6×17 exact balance) / 18 cal / 18 corpus** in `src/harness/families_semantic_defects{,_eval}.rs`, authored under the full wide law (label-token-free eval+cal with the 12-token ban row, ZERO trigram wall hits vs corpus∪cal, per-case overlap ≤ 0.92 / mean ~0.3x, digest pin `52573a21…cac78`), wired through `FAMILY_DEFS` + the runner SUITES + `MODELLESS_FAMILIES`/`WIDE_FAMILIES`/`LABEL_BANS`/`ACC_FLOORS` (floor 0.13 = 0.8× the 1/6 chance) + the fleet-hygiene coverage wall. **First modelless cell: acc 0.1863 vs 0.1667 chance** (macro-F1 0.1798, ECE 0.0287, abstain 61%, p50 0.021 ms, det ✓; `.scratch/sd_first_cells/` — untracked) — the family is the hardest of the seven for the hashed-bag lane, which is the honest answer to the issue's opening question. Gates: `harness_families_gates` 9/9 + `fixture_fleet_hygiene` GREEN + full `--features modelless` sweep 0 failures (one loopback flake in `lanes::openthai::stub_listener_round_trip` passed in isolation — the known class, not this change) + clippy `-D`-clean at the modelless posture. The two documented divergences from the issue text: the family is class-Choice rather than per-class booleans (the wide band cannot hold 10 classes), and four perch classes (`does_not_do_what_it_claims`, `bad_state_change`, `wrong_return_value`, `resource_leak`) wait for a deliberate widening. The issue's remaining open question — LLM-lane cells (agentjev / openthai / gliner / PAW) over the family — rides the standing comparison-lane cadence, not this file. Per the noise-reduction rule the issue file is removed with this record; its full life: `git log --follow -- .issues/061_semantic_defects_suite_family.md`.

- **The fixture-fleet hygiene wall (`3a307ee`) — the four-time recurrence closed mechanically; the
  `91e9246` en-route fix completes the plan-011 typed_case_split dispatch (E0603 under
  `--all-features` on macOS, the one posture its fd3c0d2 verification missed).** The owner asked why
  the corpus-overlap class keeps recurring "the 4th time" — the honest accounting: (1) the dataset
  suites' near-duplicate train/test leakage (Issue 024, the `slice_leak` probe); (2) the frozen t20k
  sst5 pool's cross-split duplicates (`slice_guard` KNOWN_DIRTY — published rows measured on dirty
  bytes, dedupe parked on instinct 013); (3) `code_fixtures`' template-shared eval (the instinct 008
  T8 unfalsifiable-memorization refusal); (4) the harness families — measured at the 059 landing:
  mean unigram overlap 0.68–0.78 against corpus∪cal, per-case maxes at 1.00, 2–7 trigram-hit cases
  each, and the published family accuracies (0.56–0.92) were that overlap's inflation. **Why the
  asserts never caught it:** every assert that existed was structurally blind to this class —
  `assert_ne!` on whole texts (exact-equality disjointness) passes two texts sharing 90% of their
  vocabulary; `ACC_FLOORS` gate DOWNWARD only (a systematically-wrong engine, never an inflated one);
  `slice_guard` audits row MEMBERSHIP across slices, not vocabulary sharing; the slice_leak probe and
  the code_frozen digest each lived private to the set that got burned. And the fixture-design law
  itself pushed toward overlap — "class-distinctive by VOCABULARY" with a corpus that teaches exactly
  that vocabulary, and no quantitative boundary between separable and contaminated. **The wall
  (`3a307ee`):** `src/harness/fixture_hygiene.rs` is the ONE shared checker (tokens/trigrams/
  audit_slices/label bans/fixture digest — the families gates now consume it instead of their private
  copies); `tests/fixture_fleet_hygiene.rs` derives its population from the registry via the new
  `runner::synthetic_suite_names()` — a new synthetic suite joins the audit BY EXISTING and reds until
  it carries hygiene facts, so the 5th instance cannot land silently; the wide families re-assert the
  overlap bounds; `harness_cache_reuse` is pinned at its exact measured history (0.8145/0.8750/11 —
  the documented divergence, a history pin never a ceiling); **`code_fixtures` measured and pinned at
  0.9494 mean / 1.0000 max / 15-of-16 trigram-hit cases — the fleet's WORST contamination, the
  measured basis of the 008 T8 refusal, RECORDED-not-gated** (its population is frozen + digest-pinned;
  a re-baseline is the same owner-visible decision the families' wide eval was, and any silent drift
  now reds). 379 tests / clippy `-D` at default AND all-features. Recorded follow-up (not landed): the
  hygiene facts could ride results.json for synthetic suites the way 058's corpus digests ride the
  dataset cells — the audit wall covers authored fixtures; the disclosure seam would make any future
  inflation self-identifying on the board itself.

- **Issue 059 CLOSED — the families wide eval + the quarantined web section EXECUTED; file removed
  per the noise-reduction rule; this row is the record.** Plan 009 REVISED-2's product shipped end to
  end. **reflex `e78c0e6`**: the five families' evals widened to 96–100-case class-balanced
  template-disjoint populations (`src/harness/families_eval_wide.rs`, assembled from the committed
  `.scratch/famwide/*.tsv` authoring sources) with the permanent authoring gates — per-family
  label-token bans (TOOL bans the six tool names, eval-only; SENS bans every digit), the zero-tolerance
  corpus/cal 3-gram wall, unigram-overlap ceilings (mean ≤ 0.78 / per-case ≤ 0.92 — actuals 0.35–0.45 /
  ≤ 0.73), exact class balance, and per-family BLAKE3 digest pins; `harness_cache_reuse` stays the
  frozen T3 12-fixture record (the DOCUMENTED DIVERGENCE, `CACHE_REUSE_NOTE` + the
  `cache_reuse_stays_the_documented_divergence` pin). The old evals' overlap stats are in Bench 105:
  the old populations shared 68–78% of their vocabulary with corpus/cal (per-case maxes at 1.00, 2–7
  3-gram-hit cases each) — **the old accuracy numbers were corpus-overlap inflation; the wide evals are
  the honest, un-inflated reads** (the finding that justifies the caveat). Frozen read
  `.benchmarks/105_families_wide_eval` (modelless run() posture + abstains, PROVENANCE quoted, latency
  QUOTABLE; acc 0.3125/0.3125/0.3021/0.2812/0.2200/0.5000, p99 ≤ 0.017 ms, determinism ✓). Full lane
  green: 359 tests / clippy `-D` at default AND all-features (the all-features posture needed the
  pre-existing clm-lane break fixed in passing — `b72f808`: `req` was never built per case + a shadowed
  dead `determinism_ok`; landed broken by the S1MB session). **instinct `0340f1f`**: Bench
  0051_families_wide_eval re-measures the A0 seat posture on the wide evals (0.4896 / 0.4896 /
  0.4896 / 0.4479 / 0.3400 / 0.9167 — the NB count-table lift over the raw engine is real; cache_reuse's
  noul polarity holds 0.9167 on its unchanged 12); the A0 pin's leg 1 (arena == reflex run()) HELD on
  all six, leg 2 (published site rows) is stale by the eval swap and documented under `--skip-pin-a0`
  (the board refreshes at the next full republish); `served_family_decisions_are_the_frozen_a0_picks`
  re-pinned off the invalidated 049 record and green ×3 (one transient first-run failure on the busy
  box, two clean full reruns). **reflex-site `38fbae4`** (CF `0da6b32e`, live-verified): the quarantined
  `/families/` section — own page + own `data/families.json` (generated by
  `scripts/publish_families.py` from the two records, nothing typed), our lanes only (Reflex modelless ·
  Rethink hybrid · Rethink encoder `not run`), the honesty caveat rendered VERBATIM, and the quarantine
  asserted two ways (`test_publish_families.py`: publish_bench never references families.json;
  `families_page_smoke.cjs`: the page never fetches bench.json). The encoder lane renders `not run` —
  the riir-train heads question is the recorded owner call and the issue's reopen trigger.

- **Issue 053 CLOSED (owner-gate pickup C8/C9/D6) — file removed per the noise-reduction
  rule; the ratification state lives in the master (riir-ai 1016).** C8 (typed_decisions
  cap 800→1200) was SUPERSEDED-BY-EXECUTION before the prep ran: landed as `2cbbce7` +
  Bench 078 (modelless 0.4655 → 0.5725 at the published posture), master 1016 C8 row
  EXECUTED, site republished `a2f1f9e` — the scratch-worktree prep the row asked for is
  moot. C9 RECORD-ONLY: the Thai lane stays closed; the reopen trigger lives on
  `.research/003_openthai_systemone_thai_lane.md`'s status line and master 1016 C9 reads
  "Keep closed; reopen trigger already documented". D6 RECORD-ONLY: native agentjev
  serving via riir-infer DECLINED for now — the HTTP comparison lane already yields the
  data (`.research/002_agentjev_system1_landscape.md:107`); master 1016 D6 reads
  TRIGGER/decline. The 2026-09-28 owner direction (4 repos deferred, all mainnet actions
  on hold) is recorded in the master, which remains the single ratification surface.

- **Issue 058 CLOSED (cal-selection ladder / board collapse) — every actionable task
  landed; file removed per the noise-reduction rule; the deferral triggers are preserved
  here.** Root cause (d): A/B-confirmed silent datasets-dir move (t20k → default 4k
  pools), no code to revert — fixed at the fetcher (`TRAIN_CAP` default 20000) + the
  `slice_guard` per-suite pool floors (overlaps hard-refuse; slice-identity digests in
  results/TABLES/run log; the sst5 dedupe + the frozen-t20k KNOWN_DIRTY pin). The
  canonical pool was REBUILT as clean full pools (deduped, provenance-manifested,
  `POOL_MANIFEST.json`), the board RESTORED on both hosts (Bench 100 m3 + Bench 101 4090
  — 15/15 accuracy bit-identity, zero drift, 1039 dataset files SHA256-verified identical
  pre-run) and PUBLISHED (reflex-site `d750175` + deployed, live-verified; the Issue-057
  CORPUS_RESET ack scoped to the six changed suites + the dated disclosure note). The
  executed canonicalization was the owner's option (i) (t20k-scale pools); the remaining
  OWNER ratification + the frozen-`datasets_t20k` sst5 dedupe decision live on **instinct
  `.issues/013`** (OPEN — the arena re-point, ready-to-flip). DEFERRALS with reopen
  triggers: **(a)** baseline-arm floor — reopen if the publisher ever ships a modelless
  cell below its pre-076 value without a loud disclosure row (the slice digests + floors
  closed the silent channel the floor was belt-and-braces against); **(b)** LCB-based
  selection — reopen when a future re-selection's ladder top-2 are cal-tied AND the pick
  flips the served posture (massive 0.715/0.72/0.72 was the recorded shape); **(c)**
  k-fold cal-front probe — reopen when any future selection's cal→test gap exceeds ~15 pt
  at the selected posture (the massive-076 shape). Published posture flags stay
  `--skip-laya --nb-select --oc-select --ridge-select` unless the owner re-baselines.
  Code comments citing "Issue 058" (slice_guard/runner) remain valid historical
  identifiers — the file's full life: `git log --follow --
  .issues/058_cal_selection_ladder_overfit.md`.

- **Issue 060 RESOLVED (a7475c7): the harness records the abstain CAUSE per case —
  reflex-site plan 001 task 6's unblock.** The engine classifies WHY the fused
  gate abstained at the decision site (`AbstainCause`: ScoreGate / DistanceGate /
  GrammarInvalid-reserved, score-gate precedence, the `||` short-circuit preserved
  — byte-identical decisions, identical evaluation order; `DistanceGate` is the
  marginal cause: score threshold passed, distance gate fired). The harness
  captures the per-question causes from the CALLER-OWNED scratch right after
  `decide_with` (zero wire changes — the slots survive the call and hold exactly
  the last request's verdicts), asserts the wire's `outcome.is_none()` agrees
  with `cause.abstained()` (debug), and publishes `abstain_causes` on the
  modelless LaneResult — the CALIBRATED gate's shares as the fixed 3-key
  snake_case object (`{score_gate, distance_gate, grammar_invalid}`; the shares
  sum to `calibrated_abstain`'s abstaining total by the precedence law). laya
  publishes `None` (cannot abstain — never a fake zero); pre-field cells publish
  as absent (the site renders "not recorded", never guessed). grammar_invalid
  is 0 on every harness run — the reserved serve-lane arm (game-heads
  fall-through), carried so the taxonomy's wire shape is closed. Gates:
  engine_gates classification test (score-only / distance-only / answered /
  both→ScoreGate), harness_units wire-shape pin + cause/flag agreement law;
  clippy `-D warnings` clean; lib 241 / engine_gates 9 / harness_units 35 /
  families 7 all green. The SITE half (publish + render) stays with reflex-site
  plan 001 task 6 — it lights up when a harness run carrying the field lands.

- **Issue 059 CLOSED WITHOUT EXECUTION (owner verdict 2026-10-02): the six harness
  families are INTERNAL TEST FIXTURES, not benchmarks — the wide template-disjoint
  eval is withdrawn before any build.** The plan (`009`) + the anchor issue were
  authored as filed at `72c1238` (no code, no evals, no bench record ever landed —
  verified: no `families_eval_wide.rs`, no landing commits, no `.benchmarks/104_*`).
  The verdict, in three parts: (a) authored data cannot become a benchmark by
  self-defined disjointness — the gates would reduce token leakage but question shapes,
  distractor styles and class balance remain ours, and with corpus/cal unchanged a
  specialist "win" would prove grammar-fit, not capability, uncitable beside an
  external index (Jev-style standardization lives on the dataset suites + bench.json,
  reflex-site Plan 001); (b) the lane had no consumers — instinct Issue 008 T8 already
  dropped the families from the specialist covered set, the arena law-excludes them,
  and the site card dropped their cells; (c) the recorded re-open condition ("a larger
  template-disjoint reflex-side eval") was necessary, not sufficient — it fixes
  statistical power, not external validity. The families KEEP everything that makes
  them load-bearing as fixtures: G2/G4 workloads, the engine regression pins
  (discrimination floor, determinism, anti-pathology, gold-agreement), the existing
  n = 12–16 evals, and `harness_cache_reuse`'s grounded-posture GATE (Bench 072).
  riir-train trainer extension: never filed. Issue file removed (full life:
  `git log --follow -- .issues/059_harness_families_wide_eval_reopen.md`); the
  revised plan with the full verdict is `.plans/009_families_wide_eval.md`.

- **Issue 059 REVIVED same day (owner call, hours after the closure): the families DO
  go on the web — scope REDEFINED from certification to a QUARANTINED our-lanes
  section.** The REVISED-1 verdict's reasoning is not reversed but ENCODED: a NEW
  reflex-site section (own page/block, own data file — never touching bench.json's
  areas/index) renders the six families with ONLY our lanes (Reflex modelless · Rethink
  hybrid · Rethink encoder — no external lanes, owner call "bench only us"), behind a
  MANDATORY honesty caveat rendered verbatim on the page (self-authored fixtures;
  engineering signal, not capability claims; not comparable to the dataset board or the
  Jev Decision Index). The wide eval is UN-WITHDRAWN as the data substrate for that
  section (publishable n, gates doubling as fixture hygiene); the specialist-certification
  purpose stays dead (instinct's covered-set law-exclusion stands, display-only);
  Rethink (encoder) cells need riir-train heads on the family corpora — owner call
  pending, absent cells render `not run`. Issue file restored as
  `.issues/059_harness_families_web_quarantine_bench.md`; plan at REVISED-2.

## 2026-10-01

- **Issue 057 CLOSED (Bench 102 + reflex-site `8d33e3f` + `2b699f0`, CF
  `30402050`): the LANE_CARRY corpus-axis gap — every host slot can now
  refresh published timing on a corpus change.** The m3 primary
  `typed_decisions` cell reads p50 **0.782 ms** live (was the carried 0.517
  from the pre-lift 800-row pool; the 4090 half refreshed at Bench 099).
  The close-out caught a residual publisher defect the 4090 half could not
  see: the Bench-100 board restore had carried the stale timing onto a cell
  already carrying the new-pool digest (`_carry_into` re-attaches ONLY
  timing — the merged cell read digest(new pool) + timing(old pool)), which
  made the m3 cell PERMANENTLY unrefreshable (equal-digest refuses the ack;
  no ack re-carries). The repair keys the ack's staleness on whether the
  incumbent's timing is its OWN — a carried incumbent's equal digest stamps
  its accuracy merge, not its timing; only an own-timing cell can make the
  ack stale (one-time property pinned both directions, self-test 67/67).
  Validated dry against a /tmp copy of the real board with Bench 100's doc
  BEFORE the live publish; the live publish printed both disclosure notes.
  Run gates: preflight PASSED (AC/high/settle 6m/canary 129.2µs; swap 3831M
  disclosed), acc 0.5725 bit-identical, slice + corpus digests byte-equal
  Bench 100's (pairing gate's own check), all publish gates green, live
  curl-verified. Full record: `.benchmarks/102_typed_corpus_republish_m3.md`.

## 2026-09-30

- **Plan-426-T6 seat half LANDED (`776e8db`): the SEAT consumes the synth
  corpus.** `seat::prepare_seat_with_synth` — the artifact is
  blake3-verified by [`load_synth_corpus`]'s own path (the digest rides
  the same read that verified it — no TOCTOU window), the header's suite
  is pinned against the seat's name (a cross-suite corpus is a loud
  refusal), rows outside the engine's label universe are dropped COUNTED,
  and the provenance (path + digest + row counts) rides the public
  `Seat.synth` — Issue 057's corpus-identity axis, now carried by the
  type. `build_seat_engine` applies the overlay at BUILD time
  (gold-cap-first + synth-beyond, in BOTH the drafter corpus and the
  count tables) — the exact V5 arm-B construction, extracted as
  `specs_corpus_extended` and SHARED with the corpus-ab lane (its
  `arm_specs` now delegates): the measured V5 build and the served seat
  are the same code by construction, they can no longer drift apart. An
  armed option-conditioned posture refuses the overlay (the (qid, option)
  tables are fit from gold events synth rows do not carry). `None` seats
  are byte-identical to the pre-overlay seat. Consumers: instinct
  `c7b7165` (arena `--synth-corpus/--synth-extra-cap`; serve
  `INSTINCT_SYNTH_CORPUS_DIR` — a present artifact seats it, absent =
  byte-identical boots). Validated by the instinct-side smoke: the
  emotion fixture seated 6/6, A0 0.8850 == the published armed number
  (unchanged path untouched at trivial corpus mass); the massive cert
  read rides the instinct side (Plan 426 T6, in flight).

- **The 056 close-out's owed site republish LANDED — Benches 096+097: the
  promoted gate posture serves on reflex.gist.rs (both hosts), and the
  deploy resolved the 09-29 typed-H2 deploy-pending note in the same
  push.** The 076+077 flow verbatim, replayed for the promotion: two
  coordinated runs at `d4051c8` (096 m3 — preflight PASSED, load
  3.92→4.16, quotable both spans; 097 4090-windows — UNJUDGED-standing,
  sibling tree synced + dataset pools MD5-verified across boxes first),
  canonical pool, nb/oc/ridge select, heads off. **Hard accuracy
  bit-identical to every incumbent cell on all 15 suites both hosts**
  (cross-host bit-identity re-proven, Issue 018 T7); the gate cells move
  to the promoted posture (calibrated_abstain == the old fitted raw
  target on every dataset suite; raw_abstain 1.0 by construction — the
  documented 095 lever artifact); source stamps `d4051c8` on both host
  rows. Timing carried per the Issue-032 law (both sides quotable →
  incumbent wins). **En-route gap filed as Issue 057** (owner-gated): the
  typed m3 p50 0.517 was carried from Bench 076 — measured on the pre-lift
  800-row corpus, while the accuracy cells measure the 1200-row corpus
  (quotable reads on the new pool: 0.764–0.815 ms); the corpus axis has
  no LANE_CARRY escape (the population reset covers only the question
  axis), so no lane-scoped publish can refresh it while the incumbent
  stays quotable. Publish gates green end-to-end (self-test 52/52, chart
  smoke, drift gate, pairing, mirror parity, bench-page smoke);
  reflex-site `0ce46a8` + CF deploy `6f5496e3`, live-verified.

- **Issue 054 Part 2 REFUTED — the prefix-state handoff serving lead does not transfer to laya (measured, `riir-infer` probe gate `crates/riir-infer-laya/tests/prefix_state_coupling.rs`).** The as-filed premise read "laya is GDN-family (riir-infer `deltanet` substrate)" — wrong architecture: the laya checkpoints are **ModernBERT-large / mmBERT-base**, a **bidirectional** encoder (no causal mask anywhere in the op stream; the only mask is the symmetric sliding-window band; the `deltanet` substrate is the ternary Bonsai/GDN lane, unrelated). GDN's causal recurrence is exactly what makes open-jev-fast's `fla_mode="state"` handoff lossless — ModernBERT has no per-position state a suffix could resume from. Three grounds, all structural: (1) bidirectional coupling — state-span rows are functions of the per-question head span; (2) the state sits AFTER the per-question head span in `build_sequence`'s render, at a different RoPE offset per question (measured: offsets 34 vs 35); (3) per-question truncation (`room = max_len − ids.len() − 1`) — the "shared prefix" is not even guaranteed identical tokens. **Measured on the real typed checkpoint**: two questions against one shared state — the shared span (**283 of 317/318 tokens; the state is ~89% of each sequence, so the as-filed ~5× prize was real**) drifts **5.1e2**, 5+ orders above the 1e-5 CPU GEMM reduction-order budget; synthetic arm **2.7e0** over a 16-row shared prefix with a bit-identical determinism control. The probe is TWO-SIDED (a reading at or below the budget FAILS the test — the record re-opens mechanically if the architecture ever changes). Consequence: the packed per-question pass (issue 020 T5) stays the exact floor for laya case serving; the lead's shape stays valid for causal serving models (GDN/KV decoders — the league lane), never laya. Issue stays OPEN for Part 1 (hardware-gated openjev lane candidate). Gate landed riir-infer `ff5de37` (an earlier draft cited `c1caa22` — the same commit rewritten by two rebases under concurrent dq614 landings).

- **Issue 056 CLOSED — the calibrated-confidence ranking regression: root-caused to a Platt-solver defect substrate-side (katgpt-rs Issues 909+910+911, three verdict rounds), and `--gate-fit-calibrated` PROMOTED to the default posture (verdict round 2 — the owner-delegated adjudication: "a units bug, not a matter of taste").** The full arc, one day: the 092 finding (cal key ranks worse than raw, 7/9 suites; banking77 below its own accuracy) → the mechanism probe (`788cf5b`: zero monotonicity violations, 1–3 distinct f32 values of 200–500 — f32 tie-collapse under a steep fit) → the severity elevation (`2fb2ae4`: thresholds fit on raw, applied to the saturated calibrated scale — Issue 042's 96.7% typed escalation root-caused) → reflex repair 0 (`2641477`, Bench 093) → **the substrate repairs**: the real cal windows dumped via `RIIR_DEBUG_CAL_WINDOW` + committed as katgpt-rs replay fixtures (replay reproduces the measured fits EXACTLY), the audit refuting the early-break-precision theory and finding Platt's undamped identity-init Newton stall (10.8×/23× above the achievable loss), the Lin–Lin–Weng 2007 solve landing (banking77's TRUE MLE recovered: w=8.674, T=0.115, loss 71.57 — the band carries real signal), the resolution-aware `w` floor + the zero-tolerance AUC guard (ties stopped at their source; distinct scores stay distinct in f32) → re-baselines Bench 094 (constant-map interim, framing corrected) + **095 (final: the AUC regression GONE 15/15 via real fits; the lever reproduces the raw gate's fitted target EXACTLY on every suite — percentile coherence)**. **The promotion**: `gate_fit_calibrated: true` at the harness CLI / the seat / the e0 lane (`--no-gate-fit-calibrated` restores the old posture); verified byte-for-byte — the promoted default reproduces the 095 lever cells on all 15 suites; gates green (lib 219/0, engine_gates 8/0, game_heads_serve 16/0, clippy `-D`). The katgpt-rs commits: `74e9d192d` (909) → `b0d80979d` (910) → `77eacda8f` (911). The old "−11.5pt selacc" cross-coverage framing is retired ("operating point shifted off the fitted target" — every posture sits on the same risk–coverage curve). File removed per the noise-reduction rule.

- **Issue 056 repair direction 0 LANDED — `--gate-fit-calibrated` (Bench 093): the gate's score-axis threshold now fitable on the scale the gate applies.** The 056 severity-elevation finding (thresholds fit on the RAW scale via the identity-calibrator probe, applied to the SATURATED calibrated conf → 90–99% abstain on six suites / a disarmed score axis on banking77 — the root of Issue 042's 96.7% typed escalation) gets its reflex-side remedy: the lever observes the probe's own per-question (conf, correct) pairs — byte-identical to the deployed calibrator's `cal_pairs` in the default posture — into a `SigmoidGateCalibrator` at the engine's window config and maps the score observations through the fitted `apply` BEFORE the ρ=30 percentile. Measured (deployed posture, t20k): the six saturate-at-0 suites' calibrated abstain collapses 90.5–98.8% → 31.5–47.5% (the fake "sel-acc 1.0000 on 3–7% kept" cells become real readings — ag_news 0.9053 at 61% kept); banking77 unchanged (already disarmed — a saturated scale carries no information); massive (sane-fit control) unchanged within 0.3 pt with a real 0.2008 threshold; forced accuracy unchanged everywhere. **Opt-in, default off — promotion is owner-gated WITH the substrate saturation guard (repair 2, the remaining work in Issue 056): the two compose, and promoting the reflex lever alone would change the deployed abstain semantics on six published rows.** The seat pins all three gate levers off (the arena's published face unchanged by construction). Clippy `-D` clean both postures; lib 230/0 (feature) / 219/0 (default). Commits: the lever + Bench 093.

- **Issue 055 CLOSED — the distributional decision layer PoC (Bench 092): the pre-registered null path FIRED, stronger than first recorded; the feature stays opt-in. Issue 056 SPUN OFF (the calibration-ranking regression the PoC's baseline columns exposed).** The seeded-MC wrapper (katgpt-core `perturbation_ensemble` + reflex `mc_ensemble`, Plan 008 T1–T4) gained its harness arm `--mc-ab` (T5: `src/harness/mc_ab.rs` — rejection-ranking A/B at matched coverage + the histogram's decision rules, p/λ selected cal-side with the tables in the record, test read once) and the 15-suite PoC ran at the DEPLOYED seat posture over the frozen Bench-005 pool. **Verdict (gate 6, dataset suites n ≥ 100): u_pair loses to the RAW readout confidence 8 of 8 (mean −0.091 AUC) and to the deployed calibrated key 7 of 8 (mean −0.041); LCB-λ is null everywhere (≤ +0.012)** — marginal lift ≈ 0 ⇒ the recorded negative, `set_rerank`/`differential_anchor` precedent, feature opt-in behind the kill switch + default-off knob. **The first draft's banking77 carve-out (+0.082 AUC) was WITHDRAWN on the verdict round-1 review** — the RAW key beats the MC layer there too (0.9350 vs 0.8945): the win was against a degraded baseline, not the engine's information; the "wide-label" re-open lead fell with it (post-hoc — massive_intent_en, 60 labels, is the WORST loss at −0.147). **The real finding the PoC surfaced: the calibrated readout confidence ranks strictly WORSE than raw as a rejection key on 7/9 suites (up to −0.122 AUC; banking77 0.8125 < its own accuracy 0.826 = worse than random) — a monotone calibrator cannot reorder at all, so the windowed calibrator is non-monotone by construction or mis-mapped → Issue 056** (the gate's threshold-wise abstain decisions are order-free and NOT implicated; every ranking consumer is). **The founding defect caught in-run:** the mean-score rule read ENGINE space against gold — `prompt_injections` mean-pick 0.2328 = exactly 1 − 0.7672, the complement signature — the noul `[yes,no]`→`[no,yes]` flip applied to majority/LCB but missed on mean; fixed + pinned by the N=1 point-mass equality test. Gates: G1 three-part green (unarmed == legacy via the full pre-existing suite incl. 16 frozen-pick game_heads; armed same-seed byte-identity via two independent runs' identical records); G2 latency PROVISIONAL-load-disclosed (639–3 985 µs p50/case at N=8; the heavy-suite breach is STRUCTURAL — banking77 8 × 0.338 ms base = 2.7 ms — adaptive-N stays the recorded remedy); non-collapse floor green. 7 new unit tests (230 lib at the feature posture, 219 default — the arm compiles out), clippy `-D` clean both postures. Bench `.benchmarks/092_distributional_layer_poc/`.

## 2026-09-29

- **The paw-LOCAL posture twins for the new suites are MEASURED — bench 090
  (the 088/089 handoff's last open lane task, closed same day).** The 056
  verdict repeats on prompt_injections (0.6379 == hosted exactly), xnli_en
  (0.7133 vs 0.7200), massive_intent_en (0.5133 vs 0.5100, refusals
  118 vs 119) and typed_decisions (0.5955 vs 0.5925, score MAE 0.6387 /
  within-1 0.7963 — both beat modelless locally too): deltas −0.7 to
  +0.3 pt, refusal counts within 1, det ✓ ×4, and TWO full runs
  byte-identical on every pick/acc/refusal/det field (`results_det_rerun.json`).
  **The rule-had-not-generalised class, caught pre-cell:** the 087
  per-shape spec extension landed in the HOSTED lane only — the local lane
  still called the suite-wide-only `paw::load_spec`, so all four suites
  came back honest-absent (no committed spec). Fixed at root cause:
  `paw::resolve_shapes` is now the ONE per-shape resolver (per-qid wins
  over suite-wide, drift guard, partial-coverage refusal) consumed by BOTH
  lanes; the local lane dispatches each question to ITS shape's program
  over per-shape cache keys (`key_for_qid`). New gates:
  `multi_shape_suite_dispatches_each_question_to_its_own_program` (the
  dispatch law over scripted channels) +
  `multi_shape_missing_one_shape_refuses_partial_coverage`; 22/22 lane
  tests, lib 219/0, clippy `-D` ×2. **M3 venv stood up** (`.raw/paw-env`,
  programasweights 0.4.10 / llama-cpp-python 0.3.19): the install carries
  a measured sccache trap (llama-cpp-python's setup picks sccache as its
  compiler launcher and sccache 0.13.0 fails with "failed to zip up
  compiler outputs" — mask it from PATH; documented in AGENTS.md); the 24
  ft bundles pre-warmed (first-use downloads bimodal 6.5 s–765 s, ~2 h).
  Wall: 6.5 min per full run (2716 q, four suites) vs the hosted 089's
  81 min for typed alone. Board tier pinned
  (`PAW_COMPILER=paw-ft-bs48-20260530` — mandatory: both tiers are cached
  for the new suites). Accuracy-only posture (load 12.71→14.99,
  latency_quotable: false). Record:
  `.benchmarks/090_paw_local_new_suites_m3.md`.

- **The PAW dataset board is COMPLETE — all four remaining suites measured
  (Benches 088 + 089, 18 new programs, both compiler tiers on 088).** The
  087 row's standing coverage note ("paw × typed / prompt / massive / xnli
  reachable but not run") is discharged. Specs for all 18 shapes frozen in
  `9dc33da` BEFORE any cell (the 049 discipline, this time in a prior
  commit): prompt_injections (noul), xnli_en (3-way choice),
  massive_intent_en (59-token intent universe), typed_decisions (15 qids
  across the 5 workflows — 4 Choice / 5 Score / 6 Noul; the Score specs
  carry the exact level lines because those ARE the option_set keys the
  never-guess parser matches verbatim). The committed-spec drift guard
  pins every file + every option union. **Bench 088** (pi + xnli + massive,
  716 q): the first run forgot `PAW_COMPILER` and served the BASE tier
  (`paw-4b-qwen3-0.6b-20260407` — the anonymous default, 049's posture);
  re-run under the board tier (`paw-ft-bs48-20260530` +
  `PAW_COMPILE_ASYNC=1`, the 054 law) and BOTH kept — the 054 both-tiers
  pattern on three more suites. ft 0.6379 / 0.7200 / 0.5100; base 0.6983 /
  0.5833 / 0.0967; det ✓ all six; modelless drift pins byte-identical to
  the published rows (0.4828 / 0.3400 / 0.3533). Findings: the tier flip
  is suite-dependent (base wins pi by +6.0, ft wins xnli +13.7 and
  massive +41.3); massive carries the sampled-presentation refusal class
  (20-of-59 per row → wrong-unpresented picks are honest refusals:
  119/300 ft, 238/300 base; samples show both `qa_factoid`-unpresented
  and `iot_hue_sound`-hallucinated classes). **Bench 089** (typed, 2000 q,
  ~81 min run): **0.5925** with 2/2000 refusals (score MAE 0.6485 /
  within-1 0.7937 — BOTH beat the modelless lane's 0.7046 / 0.7275; the
  verbatim-copy score levels held), board second behind agentjev 0.7715,
  above openthai 0.5345 / gliner 0.528. Accuracy-only posture throughout
  (preflight REFUSED at load 17.84; harness load 12.56–13.30 — the 049
  law). `harness_families` stays PAW-less by board symmetry; paw-local
  twins for the new suites landed same-day as bench 090 (M3, above — the
  "belongs on the 4090" note was superseded: the venv stands up on the M3
  in minutes and the lane is box-agnostic).
  Records: `.benchmarks/088_paw_ft_pi_xnli_massive_m3.md` (canonical + the
  base-tier dir), `.benchmarks/089_paw_ft_typed_m3.md`; highwater 87→89.

- **The PAW lane gained per-question-shape programs + its first
  `code_fixtures` cell (Bench 087, 0.6250 ft-bs48).** Until this row the
  PAW comparison lane was the one lane without a `code_fixtures` cell on
  any host (openthai 084/086, gliner 037, agentjev 039 all carry one):
  the suite serves TWO question kinds per case (`module` Choice +
  `is_pub` Noul) on ONE input, and the lane's one-program-per-suite law
  structurally refused the second shape (one program cannot answer two
  questions on the same text — a ~50% accuracy ceiling wearing a
  measurement). The extension (`src/lanes/paw.rs`): one program per
  DISTINCT resolved spec — `{suite}.{qid}.txt` per question shape with
  the qid in the cache key, the suite-wide `{suite}.txt` fallback
  unchanged for the single-shape suites (byte-identical behavior AND the
  historical key); partial coverage is a loud error naming the missing
  file, never a partial row; multi-shape rows disclose every program
  (ids/paths/digests comma-joined, compile wall summed). Pinned by three
  new stub-server tests + the committed-spec drift guard extended to both
  spec files. Cell (hosted-anonymous, `paw-ft-bs48-20260530`, accuracy
  only — preflight REFUSED at load 13–16, siblings): **0.6250** (31/32
  answered, det ✓, server p50 77 ms) — second on the suite behind gliner
  (0.6667), above openthai (0.5938) and laya·en (0.5833); the one
  refusal is the program answering a plausible-but-nonexistent module
  name (`harness::families`), refused by the never-guess law. The
  extension also makes paw × typed_decisions (a THREE-shape suite)
  reachable for the first time.

- **The 084 publish arc completed + the 426-T5 corpus-synthesis lane
  landed (serving riir-train plan 426).** The 084 row's deliberate
  non-publish (concurrent-lane handoff) resolved: the site cells landed
  via reflex-site (`1999b38` the 4090 fill + `1e893ae` the clean re-read
  + `6484273` the m3 fill; CF `2303b558`), Benches 085 (M3 clean re-read:
  the 1.63 s massive cell REPRODUCES at 1.71 s — contamination refuted,
  all four m3 openthai cells quotable from preflight-passed runs) and 086
  (M3 lane fill: openthai 17/17 quotable on BOTH hosts; the option-count
  scaling law measured as a full curve, 1.2× at 3 options → 17.2× at
  banking77's 77; 9/13 accuracy cells byte-exact vs the 4090 board).
  Same day `8426cef` landed the coverage-directed corpus-synthesis lane
  (`--synth-corpus` / report-only `--synth-plan` / the V5 `--corpus-ab`
  gate) over sealed corpus artifact v2 (SYNT magic + blake3 sidecar;
  tampered/unsealed artifacts refuse at load) with the openthai agreement
  veto — cross-frame span transplantation + per-intent E0 rumor weighting
  + integer-scaled allocation with per-label caps; 14 new tests; T4's
  dump + the veto+A/B arms auto-queue on the riir-train side (plan 426
  T4 in-flight there, M3 venue). Both lanes now documented in AGENTS.md
  Build Commands (openthai was never in the list — the 084 HISTORY row
  was its only doc home).

## 2026-09-28

- **The `openthai — not run` lane-update: all 17 suites measured on the 4090
  (Bench 084).** The bench page showed `openthai — not run` on 13 of 17 suites —
  the Issue-025 class verbatim (the lane is opt-in `--openthai`, default off, and
  every published run since Bench 074 left it off; no issue owned it, none filed:
  documented default posture, not a defect). Server re-provisioned from scratch on
  this box (the M3's `.raw` clone was removed post-record per the research rule):
  `iapp-technology/openthai-systemone` @ the Bench-074 pin `5d04bcca`, uv venv py3.10
  + torch 2.6.0+cu124 + transformers 5.17, weights HF `iapp/OpenThai-SystemOne`
  (cached `.raw/hf`), FastAPI loopback :8000, `permutations=1` pinned. **Numerics
  pinned to the board's fp32** — their client defaults CUDA→bf16, so a 4-line
  disclosed `.raw` patch adds `OPENTHAI_SYSTEMONE_DTYPE=float32` — and the
  cross-check vs the M3 board confirms it: massive_intent_en **0.9200 == 0.9200
  exact**, thai_sib200 **0.8382 == 0.8382 exact**, xnli 0.9000 (+0.3 pt), wisesight
  0.4675 (−0.8 pt). Determinism pin green ×17; 15 default suites + the 2 Thai probe
  suites (`--suites thai_wisesight,thai_sib200` — the default population excludes
  them, which is why 074 ran them separately). Records:
  `.benchmarks/084_openthai_lane_update_4090.md` (+ `/thai/`); plan 003 addendum.
  **Site publish deliberately NOT executed this session** (concurrent-lane handoff:
  reflex-site `data/bench.json` is a collision surface) — the exact steps, the
  `PUBLISH_BENCH_LANES=openthai` filter law (our stale modelless control cells MUST
  NOT overwrite published postures), the shikuwa→4090-win host alias (env unset;
  the Issue-033 PAW precedent), and the server restart command are recorded in the
  084 doc's §Publish. Server stopped after the runs (unattended GPU processes are
  the sibling-gate exclusivity hazard); restart ~1 min from the cached weights.

- **Issue 047 CLOSED measured-negative — the xnli M1 reopen executed
  under its own pre-registration and the head itself is refuted (Bench
  073).** The full R1–R6 protocol ran on the FRESH validation slice
  (n=2490, fetched once; the spent test split never loaded):
  `--nli-m1 --suites xnli_en_val` over a 19,782-item cross-fit pool.
  Verdict **negative, overdetermined** — (a) λ* = 0 from a
  contaminated pool ladder (the plan's "NB self-reference is
  negligible" disclosure FALSIFIED at nb-scale 16: pool OOF A0 read
  0.9108, self-retrieval), so the run is VOID as an M1-promotion
  instrument under S2; (b) the salvage-rule numbers carry the negative
  anyway: head-alone **0.5205 vs A0 0.5410** on validation at 10× the
  old training size (the 4-feature LDA is data-saturated), oracle
  win/loss **397/448** (net-negative component → no pick-level mechanism
  riding it can clear +5 pt), all four promotion legs FAIL; (c) M2's
  +1.68 pt observation and M3's zero switches close the secondaries.
  The 068 "+5.67 pt harvest" is resolved as selection artifact, exactly
  as the 069 review diagnosed. **Instrument finding (R6 payoff): the
  shipped calibrated readout on xnli is NEAR-BINARY** — 95.5% of items
  at confidence exactly 0.0, 4.5% at 1.0, zero between, 1,242 right at
  conf-0 — so the lane's own G1 face (ECE 0.0028 PASS) is the binned-ECE
  self-removal artifact the review's hole-5 named, and M1's
  `logit(c_eng)` feature was a constant step function (why S2 fired
  correctly). Honest xnli confidence work starts from the RAW max-prob
  surface (continuous, AUROC 0.6542), never the calibrated readout.
  The gap stands ACCEPTED: xnli −28.2 pt and ag_news −6.8 pt vs laya
  are closed questions of the 044/047 arc. Lane + instrument remain in
  the tree report-only; the published test-split row is untouched;
  a future attempt needs a new slice, fold-engines, and a continuous
  confidence source — none planned. Record:
  `.benchmarks/073_nli_m1_validation.md`; pre-registration:
  `.plans/006_nli_m1_reopen.md` (`6ea4bfbc`, instrument `1e19be7`,
  pre-read fixes `b8d8f17`/`b8f4ea9`). Removed issue file: this row.

- **Issue 045 reflex half LANDED — `harness_cache_reuse` answers
  modelless, POSITIVE (Bench 072).** The T3 carve-out is reversed on its
  own issue's two premises (text-decidable; no LLM-lane winner to
  protect at 0.5000): the family ships the authored per-class corpus
  (12 docs) + cal front (20 cases) it always lacked, `modelless_lane:
  true`, and the noul count-table polarity (issue 038) is the lever —
  cal-selected, never fixed: yes→domain 1 arms at +25 pt over off on the
  cal front (polarity-0 reads 0.2500, inverted), and the single test
  read lands **0.9167 (11/12)** vs the LLM lane's 0.5000 — the suite
  flips. G2 p50 0.009 ms; bit-identical across three runs. Two enabling
  changes worth naming: `selection_slice` gained the synthetic fallback
  (no pool rows → the authored cal front is the labelled selection
  slice; before this, ANY select knob on ANY synthetic suite errored
  "empty stratified slice"), and NB-selection eligibility extends to
  synthetic suites while HEADS stay dataset-only (the families'
  choice-route baseline rows are the harness sanity pins — byte-identical
  posture for the five siblings, no published row moved). Gates: families
  7/7 (noul-aware smoke; the LLM-only gate REPLACED by
  `cache_reuse_grounded_posture_discriminates` through the production
  seat path), seat 5/5, clippy clean at default + selective. The 12 eval
  fixtures + gold are byte-unchanged (non-goal law, gate-pinned). G1
  power statement in the bench record: n=12 → Wilson [0.646, 0.985],
  published with wide bars, no accuracy claim. **CLOSED 2026-09-28 —
  both halves landed** (`2d3ce6a` reflex docs; reflex-site `d046a67` +
  CF `620f7323`): the modelless row publishes 0.9167 (p50 0.009 ms,
  verdict QUOTABLE, source_run sha 1615642) via the lane-scoped
  `PUBLISH_BENCH_LANES=modelless` publish (the doc's by-product laya
  slot dropped by the issue-033 law); km_vs_laya pairing resolves
  same-sample (both digests `fnv1a64-7340afff…`) — 29 proven pairs,
  pairing gate green, page smoke PASS. The arena TL;DR's not-run row
  reads the modelless answer now; the instinct arena can seat the
  family (its Bench-011 refusal is gone upstream). En route the site
  publisher gained the openthai lane class + the suite-join path (the
  Bench 074 Thai board, same publish session) and the page smoke's
  stale blanket gliner-not-run check was retyped data-aware — it had
  been red-by-data since the harness families joined, hidden by the
  wrapper's CWD-relative playwright probe (also fixed). Issue file
  removed this commit; full life: `git log --follow --
  .issues/045_answer_harness_cache_reuse_modelless.md`.

- **Issue 050 CLOSED — `nb_ridge` honestly declares its `option_cond` read
  (`a6e04c3`).** The feature table's `nb_ridge = ["nb_scope"]` contradicted
  the ridge lane's unconditional `crate::option_cond::OcEvent` plumbing
  (`ridge_lane::build_ridge_selection`'s `events_for_pool` + the unguarded
  `oc_events_for` call in `runner.rs`), so the selective consumer posture
  `--no-default-features --features modelless,nb_scope,nb_ridge` — the
  exact shape `nb_ridge_probe`'s own `required-features = ["nb_ridge",
  "option_cond"]` row anticipates — failed E0433 + E0425 (reproduced at
  HEAD before the fix). Repair: the recommended honest declaration
  `nb_ridge = ["nb_scope", "option_cond"]` (the two ship together — the
  measured reality the issue records; the cfg-seam alternative was
  declined as an `#[cfg]` through the selection signature for no
  consumer). Validated: the repro posture, ridge-alone
  (`modelless,nb_ridge` — option_cond now pulled transitively) and the
  default all compile; clippy `-D warnings` green at the selective
  posture and default. Consumer note: riir-instinct's dep row KEEPS its
  explicit `option_cond` (deliberate — the row names the consumer's
  selection set; the feature pull now makes it redundant-but-harmless,
  so no instinct commit rides this fix).

- **The build-stamp pin FIXED for the flag-OFF posture.**
  `tests/build_stamp.rs::incomplete_build_prints_stale_naming_the_gap_and_rebuild`
  pinned `missing() == ["laya-riir"]` — true only when the engine's
  default features are compiled in — so `cargo test
  --no-default-features` (a tested posture per AGENTS.md) red on the
  exact-set assert while the STALE-line assert passed (`contains` —
  `laya-riir` sorts first either way). Repair: the expected gap derives
  from the posture (`cfg!(feature = "modelless")` → one gap vs both),
  keeping the exact-set pin at BOTH postures; the STALE-line assert now
  checks the full rendered gap list. `rebuild_command()` is UNTOUCHED and
  verified honest at every posture — `cargo build --profile dist
  --features laya-riir` re-enables the manifest defaults on any
  invocation (no `--no-default-features` on the command), so it supplies
  the full release posture including the unstamped default knobs
  (nb_scope/option_cond/nb_ridge/vessel_public_read). Validated: the
  build_stamp suite 3/3 at default AND `--no-default-features`; clippy
  `-D warnings` green at both.

- **instinct Proposal 001 T7's REFLEX half LANDED (`e66588c`) + the
  proposal's T3/T4/T7 rows checked (`instinct 599633d`).** The T4 landing
  (`9901a52`, 2026-09-27) shipped code + 9 gates but no living docs — the
  laws deferral is now paid: AGENTS.md gains the Head vessels section (A1
  capability — `vessel_public_read` ONLY, the HOSTED reader has no
  selectable path in any feature combination, verified by grep; A8 no
  runtime minting — fit at mint time, serve loads + verifies; A10 moat —
  the demo heads are public BY DESIGN, no GAME-IP content in
  PUBLIC-RELEASE vessels) + the pins-first-wildcard trust anchoring + the
  riir-reflexer sibling row; README gains the game-lanes serve posture
  (`RIIR_REFLEX_HEADS_DIR` + `RIIR_REFLEX_HEADS_PUBKEY` + `reflex
  mint-heads`) and the fitted-heads section is superseded to the
  mint-time-fit posture with the ships-next-release caveat (≤ v0.2.3
  archives boot-fit from fixtures, no mint-heads); `.docs/01_orientation/sibling_layout.md`
  (the full dependency law) un-staled — the laya lane's 2026-09-24 move
  to riir-infer (Issue 008 T4) and the reflexer-vessel dep join the
  diagram + the law, verified against Cargo.toml (optional dep,
  default-on) and BOUNDARY.md (the dep row landed there with T4,
  `.issues/051`). docs_shape_gate PASS. The instinct proposal's T3 row
  also closed: the reader-capability axis landed in all three repos
  (reflexer P3.1 both readers; instinct P4 the opt-in hosted reader;
  reflex `vessel_public_read` only).

## 2026-09-27

- **Issue 048 CLOSED — T5 discharged: the combined-posture LCB leg is a
  PROVABLE NO-OP + the Bench-070 repro is BYTE-EXACT (Bench 071, prereg
  `eedfe24`).** Both deferred 046-T5 legs ran in one session,
  preregistered BEFORE the run (P1–P3 predictions, K1–K2 kill criteria;
  the issue-046 pattern). Run A (066 `both/` posture +
  `--cascade-worthiness-lcb 0.05`): all ten dataset rows byte-identical to
  the committed 066 `both/` artifacts — the three armed rows clear the
  margin on their own (probe Δ +0.2750/+0.2075/+0.3514) and every
  disarmed row's LCB sits below 0.05, so the floor family rescues exactly
  one cell workspace-wide: the fused posture's ag_news. The combined probe
  sets are SMALLER than the fused family's (selection-slice fit raises
  thresholds: ag_news 53 vs 190, massive 47 vs 60), weakening the LCBs
  where it matters (massive −0.0086, sst5 +0.0152). Run B (the exact 070
  command): byte-identical to `.benchmarks/070_cascade_probe_lcb/`
  INCLUDING the probe_lcb floats — the pinned vectors reproduce a second
  time, giving lever 4 the same second-independent-repro standing the
  margin-0.16 promotion rode. P2 honesty: two numeric sub-prediction bands
  missed (sst5 +0.0152 vs [−0.02, +0.01]; massive n 47 not 60) while the
  decision predictions held — no kill criterion fired. Watch item
  recorded: sst5's combined LCB +0.0152 is the closest a flip-class suite
  has come to the floor (~0.035 of arming); the 066 third-flip tripwire
  (per-suite probe-size floor, not a margin change) is the recorded
  answer. Posture UNCHANGED: the recommended combined command needs no LCB
  flag; the fused posture keeps it opt-in; no promotion trigger. Preflight
  REFUSED at launch (load 6.92, siblings) — latency unclaimed; accuracy
  pick-count/load-immune; runs at `eedfe24` (code-identical to `a6bfec3`).
  Record: `.benchmarks/071_combined_lcb.md`; issue removed per the
  noise-reduction rule.

- **Issue 046 CLOSED — the probe-LCB arm leg LANDED and VERIFIED EXACTLY
  AS PREREGISTERED (Bench 070, lever `a6bfec3`).** The fused probe
  family's ag_news/massive tie (T3's recorded +6.75 price) is a SUPPORT
  problem, not a magnitude problem: massive outranks ag_news (+0.1500 vs
  +0.1316) so no margin arms ag_news without the −3.0 flip, but ag_news
  carries 3.2× the probe support and its one-sided-95% LCB clears
  +0.0854 against the flip-prone pair's ≤ +0.0064. Lever 4
  (`--cascade-worthiness-lcb <F>`, additive, default-off):
  `armed = delta ≥ margin || probe_LCB95 ≥ floor` — two-proportion,
  CONSERVATIVE on paired data (SE ≥ McNemar's whenever the reads
  correlate). Preregistered BEFORE the run in `.issues/046` (rule, floor
  0.05, predictions, kill criteria) with the three LCB vectors pinned as
  unit tests; the verification lane then reproduced every prediction:
  ag_news armed via the LCB leg (+0.0675 @ 93.0% disclosed escalation,
  cascade 0.9500 == laya's 0.9500), sst5 + massive modelless-EXACT
  (LCBs −0.0059 / +0.0064 < 0.05), every other row byte-identical to
  063@0.16, probe deltas byte-identical (G5), and the live LCBs equal the
  pinned vectors to 4dp. Fused lane armed total +0.394 → +0.4612 at the
  same flip-safety. Preflight PASSED at launch (load 5.68) but the run
  banner read 7.25 (sibling mid-run) → latency NOT quotable; accuracy is
  pick-count, load-immune, and the byte-match to 063 is the load-immunity
  witness. Posture: OPT-IN — library defaults, the combined lane's T4′
  record, and the recommended lane command UNCHANGED; the combined LCB
  leg + second repro deferred (046 T5) under the 042 promotion-trigger
  discipline. Honest limitation carried verbatim: support-confidence,
  NEVER a cal→test-shift guarantee — what it prices is the measured flip
  class. Scope boundary stated: this bench reads the standard lane slices
  for the escalation question only; issue 047's fresh-slice rule and its
  A1–A4 ag_news modelless-mechanism work (count-table features) are a
  different surface. Record: `.benchmarks/070_cascade_probe_lcb.md`;
  issue removed per the noise-reduction rule.

- **Issue 044 CLOSED — the xnli promotion path measured NEGATIVE (Bench
  069, the G1-constrained NLI blend λ).** The pre-registered reopen path
  068 recorded was executed faithfully: the `--nli-feature-ab` arm grew
  the `blend_g1_constrained` posture — λ selected CAL-SIDE ONLY under the
  calibration constraint (interleaved cal halves, fit never scores its
  own fit; ONE engine-derived conformal floor on the held-out half; the
  blend's RECALIBRATED readout — the lane's own `SigmoidGateCalibrator`
  family — must beat BOTH its own uncalibrated surface AND the floor,
  and never lose cal accuracy vs the unarmed λ=0 rung; ladder extended
  down to 0; no feasible rung → λ*=0 + `constraint_unsatisfiable`).
  Verdict: **UNSATISFIABLE — λ*=0, zero gain, the promotion criterion
  fails on its accuracy leg.** The published xnli row is UNCHANGED
  (0.5233); the blend stays report-only. Two findings a negative result
  still pays: (1) the mini-G1 screen is NOT a transferable predictor at
  n_cal=200 — the cal half said recalibration hurts every rung, the test
  side said the full-cal refit at λ=0 passes the floor (0.0964 ≤ 0.1351);
  a 100-pair Platt refit is split-unstable in binned-ECE terms (Platt
  minimizes NLL, not ECE). Do not reuse this screen as a verdict
  instrument at this cal size. (2) There is nothing to promote even where
  recal passes — the only passing surface (λ=0 recal, 0.0964) is the
  engine's own picks at zero accuracy delta and 14× worse than the lane's
  shipped calibrated readout (0.0067). Closing shape: the pair-feature
  signal is REAL (decorrelated, 56/53; +5.67 pt at the pick level) but no
  promotable confidence surface for it exists in the shipped machinery —
  a future mechanism must bring its own calibrated surface, never the
  max-prob blend one. Run 1 of the bench is VOID and recorded (the
  miniature calibrator never fitted — `observe` records, `refit()` fits;
  the strict feasibility leg decided rungs by f32 rounding noise —
  fixed, tested, re-run). Instrument defects caught by the bench's own
  module tests en route: the constant-confidence Platt window is
  collinear and keeps identity parameters while `refit()` reports moved —
  the behavioral-identity leg handles it. Issue 044 removed; carry-forwards:
  ag_news −6.8 pt (no shipped lever — measured three times; needs a NEW
  modelless mechanism that clears G1 first) and routing/sensitivity (3
  questions each — the lever is authored questions, not mechanisms).
  Record: `.benchmarks/069_nli_g1_constrained.md` + `results.json`.
  **Post-landing Claude review (rounds 1–2): verdict AGREE — the negative
  is overdetermined** (it survives the raw-surface feasibility object:
  every rung's raw_b sits below floor_b, so the screen would pick
  λ=0.125, whose test pick count the void run 1 already measured at
  +3.33 pt — below the bar). Seven record holes accepted (in-sample head
  on cal; the NLL-vs-ECE explanation untested pending a (w,c) diagnostic;
  a ~50%-split-noise bar needing bootstrap intervals; the cross-surface
  floor; the 0.0067 sharpness caveat — a G1 binned-ECE loophole; the
  harvest relabeled 3–6 pt grid-dependent with McNemar uncomputable from
  aggregates; the spent 300-item test set as the binding constraint).
  Reopen rules + mechanism candidates filed as `.issues/047_accuracy_gap
  _reopen_protocol.md` (renumbered from 045 — dual allocation with a
  sibling session's `045_answer_harness_cache_reuse_modelless.md`).

- **Issue 044 EXECUTED — the accuracy-gap audit + the frozen code_fixtures
  population + the NLI pair-feature head (Bench 068, `fe7c123`).** Five
  tasks, five measured verdicts. (1) The Wilson screen (T2): harness_routing
  and harness_sensitivity's published gaps are NOT real — 3 questions each,
  laya inside reflex's 95% CI — the families owe authored questions, not
  mechanisms. (2) code_fixtures (T4) was commit-relative and ROTTEN: the
  laya/agent.rs + laya/router.rs moves left 4 of 8 option labels without
  corpus docs, and the published number moved with the tree (0.2500
  published vs 0.2917 same-week). The population is now a BLAKE3-pinned
  committed fixture (`code_fixtures_frozen.json`, digest `034774df…b44362`,
  8 healthy modules with full slices; `examples/gen_code_frozen`
  regenerates; hand edits fail the parse-time digest check) — and the
  published +41.7 pt gap COLLAPSES on the frozen population: reflex 0.3750
  vs laya 0.4062 @ n=32, laya inside reflex's CI — republished lane-scoped
  both lanes. (3) xnli (T3): the report-only `--nli-feature-ab` arm (new
  `src/harness/runner/nli_lane.rs`: closed-form diagonal-LDA over 10 lexical
  pair features, cal-fit, one test read, three postures + the blend's G1
  triple) found REAL decorrelated signal — head alone 0.5333 > engine
  0.5233, additive blend at cal-selected λ=0.5 **0.5800 (+5.67 pt)**, oracle
  56/53 — and REFUSED its own promotion: the blended readout's ECE 0.1596
  breaches the conformal floor 0.1351 (the Bench-064 massive law, second
  instance). Reopen = pre-registered cal-side G1-constrained λ. The module
  test caught the fit's class counter never incrementing en route — the
  head would have been dead-on-arrival. (4) ag_news (T1): no shipped lever
  moves it — volume (065, negative), genome (064/065, held twice), the
  selection posture itself (reconfirmed byte-exactly at 0.8825); the
  residual −6.8 pt is encoder-owned word-order and awaits a new modelless
  mechanism that can clear G1. Postures elsewhere byte-identical
  (ag 0.8825 / xnli 0.5233 / routing 0.4375 / sensitivity 0.4000).

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
- **Issue 052 — typed_decisions TRAIN pull un-capped: 400 of 1200 rows
  were never fetched; modelless row 0.4655 → 0.5725 measured at the
  published posture; issue RESOLVED + removed** (2026-09-28; fix + record
  `0e8896d`, reflex re-measure Bench 078, cross-checked with the instinct
  lane's Bench 015 A0' read — same number, same corpus bytes). The birth
  cap in `scripts/fetch_datasets.sh` (typed train 800) had stopped inside
  the security_incidents train block (offsets 900–1199) plus invoice rows
  200–299, so the corpus guard self-doc'd security_incidents at every
  published run (the ⛔ fallback line in every table since Bench 001).
  The lift: cap 800 → 1200, resume-fetch brought exactly pages
  train-008…011; pages 000–007 + test + splits byte-verified unchanged
  (`cmp`) against both the pre-extension envelope and the instinct lane's
  verified copy — the **test split is untouched**, so the claim stays on
  the same 400 cases / 2000 questions. New page digests + the full-pull
  aggregate row joined the dataset manifest; the stale "typed caps
  already cover the train split" note was rewritten. Measured at the
  published posture (registry defaults; nb/oc/ridge cal-selected; heads
  off; genome off): acc 0.5725, macro F1 0.5429, per-kind choice/noul/
  score 0.5483/0.7300/0.4725 (from 0.4383/0.6200/0.3700 — every kind up),
  G1 PASS (calibrated readout ECE 0.0114 beats the conformal floor
  0.1818), the security_incidents fallback line GONE, latency QUOTABLE
  (m3, AC/high-power, load ~3.6–4.8, p50 0.815 ms). ⛔ CORRECTED premise:
  the issue draft claimed the cal front is a positional first-N cut and
  therefore unchanged — false for this harness since Issue 039 T2 (the
  cal slice is the STRATIFIED round-robin front over the WHOLE split);
  the wider pool re-derives it (oc cal-selected oc@4 where the capped
  pool selected oc@2), so the row publishes only at its own re-measured
  posture — which is what Bench 078 is. Downstream: README typed row
  0.5725 + footnote ³⁵; the reflex-site typed row republishes from these
  tables (site lane), and instinct's site-parity re-pin rides the same
  republish (their Bench 015 already records both postures). laya·typed
  0.7445 unaffected by construction (frozen checkpoint scored on the
  byte-identical test split; the corpus feeds only the modelless drafter
  + cal selects). Consumers of the 800-row bytes: none found (no test
  pins the corpus digests).
