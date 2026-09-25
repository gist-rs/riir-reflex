# HISTORY.md — riir-reflex

Durable records for closed issues (the noise-reduction rule: a resolved
issue file is removed from `.issues/`; its record lands here, hash-pinned).
A removed file's full life: `git log --follow -- .issues/<file>`. Open work
lives in `.issues/` and `.plans/`, never here.

## 2026-09-25

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
  `.docs/agent-skill/SKILL.md` authored as the source of truth in-repo,
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
