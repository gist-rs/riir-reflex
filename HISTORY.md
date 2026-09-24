# HISTORY.md — riir-reflex

Durable records for closed issues (the noise-reduction rule: a resolved
issue file is removed from `.issues/`; its record lands here, hash-pinned).
A removed file's full life: `git log --follow -- .issues/<file>`. Open work
lives in `.issues/` and `.plans/`, never here.

Created retroactively 2026-09-22: four issues (001, 002, 003, 005) had
already closed with records only in git history.

## 2026-09-24

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
  - **lanes** — Bench 880's lossless decoded arm served at λ 0.01,
    in-corpus 84/100, head digest `7d3f1d8e…09d34` FULL (matches the
    published pin; the decoded arm is exactly lossless so the digest IS
    the structured arm's). Protocol: the JOINED-STATE path (011's option
    1) — one `/decide` per turn, `state` = the three lane sentences one
    per line (pinned left/middle/right), exactly three noul questions,
    answer i = lane i's P(safe); the lane-name fill must equal the line's
    position (a swapped turn refuses). Modelless lane only — the laya
    lane's measured per-option shape is site-side and untouched.
  - **flappy** — Bench 882's v3 decoded arm served at λ 1, in-corpus
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
    is exactly why Bench 882 pins TWO digests (`dc6bcf73…` structured,
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
  shape. Unblocks reflex-site's 3-board arena layout (Plan 607 roadmap).
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
  at `8e58cc5`; the rows landed in the same window as the Plan 603 T1.4
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
  Instruments stay, report-only, default posture byte-identical: the
  always-on `modelless.confusion` readout + `harness --pair-head-ab`
  (both firing gates, gold-in-pair split, n_counted disclosure;
  deterministic byte-reproduced). Full guard PASS incl. G5 parity.
