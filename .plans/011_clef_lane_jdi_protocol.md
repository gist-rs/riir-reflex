# Plan 011 — the Clef comparison lane + the Jev Decision Index protocol adapter

**Status:** IN FLIGHT — filed 2026-10-02 from `.research/005_Cloudflare_Clef_Jev_Decision_Index.md`
(owner ask: research the Clef blog, bench Rethink against the Jev Decision Index like the blog does,
plan a `--clef` run published to reflex.gist.rs/bench like the other lanes). Hosted-Clef credentials
are owner-gated; phases run in order, each commits+pushes its own unit.
**Progress 2026-10-02: Phase A + Phase B LANDED** (lane + wiring + JDI adapter + snapshot + gates;
A2.5/A6 blocked on owner creds; Phase C blocked on A6 — C1's hosted run needs the forwarder +
token). **D2 LANDED 2026-10-02**: riir-train `.research/464_Clef_PostTraining_Recipe_vs_Typed_Head.md`
(riir-train `115e5690`) — the recipe note with the pre-registered per-arm gate; the optional plan
derives when the §3 trigger fires. Phase D's remaining rows stay deferred per their own wording.
**A6 probe 2026-10-03: the provided tokens DON'T carry Workers AI scope** — `CF_API_TOKEN` and
`CF_DEPLOY_API_TOKEN` (both in the gitignored local `.env`) are valid CF tokens for account
`7e11517c4dd4f6e9cede7da9b60d66eb` but fail AI read (403 code 10000 on `ai/models/search`) and
AI run (401 code 10000 on `ai/run/clef` and `…/@cf/cloudflare/clef[-flash]`, both spellings) —
the riir-dao credential class exactly. Owner unblock: mint a token with **Account → Workers AI →
Edit/Run** scope into `.env`. The A0 forwarder + probe rig are staged locally, gitignored
(`.raw/clef_fwd/` — never repo code); rerun is one command from its README once the scoped token
lands. **Phase C UNBLOCKED the other way the same day (owner call: "download model and run it
like other"): the LOCAL lane is live** — `mlx-community/clef-flash-4bit` on this M3 behind
`.raw/clef_srv/clef_lane_server.py` (the list→dict wire translation + the Workers-AI envelope;
wire contract + rigs: `.research/007`). Smoke (50-case caps, det ✓, box load 6.36 — latency
provisional): banking77 **acc 0.9800** ECE 0.0585 · typed_decisions **acc 0.6000** ECE 0.0742.
Full-N comparable runs (banking77 500 + typed 400, `CLEF_SMOKE_MAX_CASES=1000` — the local
no-spend raise of the ceiling) in flight into `.benchmarks/113_clef_lane/`. The hosted lane stays
blocked on the AI-scoped token; `clef.rs` info() posture string made posture-neutral (the local
serve must not read as "Workers AI hosted") in the same change.
**Progress 2026-10-07 (doc-sync repair — the body outran the header): C1 + C2 + C3 ALL COMPLETE.**
The header's 10-03 "in flight" runs landed as bench 113 (full-N, accuracy-first), the quiet-box
re-read made latency QUOTABLE both ends both suites (bench 118: b77 p50 3368 ms / typed 1690 ms;
site cells graduated, reflex-site `4297d32`), and the 7-suite dataset tail reached the lane's
9-suite index with modelless controls == board on all seven (bench 119, p50 geomean 680 ms;
reflex-site `365d0a0`). The site carries the lane cells (edition 2026-10-2, `5c31fa6`), the JDI
crosswalk (`53d518f`), and the TL;DR card (`dd5409d`) — all live-verified. C4 stays deferred
(owner-gated: the HOSTED Clef run) and its reading note is updated below — rethink issue 021
re-seated the typed encoder cell QUOTABLE on the same C2 pin. A2.5/A6 (Workers-AI-scoped
token), D1 (GPU window), D3 (board submission) stay deferred per their own wording.

Every number this plan publishes obeys the house laws: their stack serves, our Rust measures
(laya-python measurement law); latency = HTTP round-trip with the serving posture + box state quoted
(bench_preflight / G2 box-state law); a lane without creds REFUSES loud (agentjev law), never
half-runs; published tables are CI-regenerated, never hand-typed.

## Phase A — the `--clef` comparison lane (riir-reflex)

- [x] A0 **transport decision (blocks A1 — verdict round 1 finding):** the harness lanes are
      deliberately std-only plaintext HTTP (`TcpStream`, zero TLS deps — the clm-lane law), so the
      hosted Workers AI HTTPS endpoint is unreachable from the agentjev lane shape. Default
      posture: a **loopback TLS-terminating forwarder** (operator-run ~20-line process outside the
      repo, e.g. any TLS-capable runtime; NOT committed as repo code — the no-Python/no-sidecar
      repo law) exposing `http://127.0.0.1:8791` → the Workers AI endpoint with the bearer token
      injected; `CLEF_SERVE_URL` defaults to the loopback address and the extra hop is DISCLOSED
      in the run posture (it is part of serving latency, exactly like the JDI's own
      "network round-trip" rows). Owner-gated alternative: a feature-gated `ureq` (rustls) dep for
      direct HTTPS — a BOUNDARY.md change (allowlist row + boundary-guard run + the one-dep law
      amended); take it only if the forwarder posture proves annoying in practice. Rejected:
      committing Python (repo law); local vLLM serving stays deferred to D1.
- [x] A1 `src/lanes/clef.rs` — sibling of `agentjev.rs` (NOT a config flag):
      `ClefLane { url, token, model, timeout }`, `from_url`, a **probe handshake `info()` = one
      minimal `noul` request** (the Workers AI route has no `GET /api/info`; the reply proves the
      wire shape and carries model identity when the envelope provides it), `decide(case)` /
      `decide_raw(case) -> AnswerTriple` + raw probs; `CLEF_SERVE_URL` (default the loopback
      forwarder), `CLEF_API_TOKEN` (bearer), `CLEF_MODEL` (`clef`|`clef-flash`),
      `CLEF_TIMEOUT_MS`.
- [x] A2 wire mapping tests in the lane module (the agentjev.rs test shape): body passes `state`
      through; `choice` renders `criteria` as the options map (key = desc), `score` renders the
      levels list, `noul` carries no options (wire law); answers map back in OUR option order;
      missing qid in the reply is a loud error, never a positional guess; the Workers AI
      `{result, success, errors}` ENVELOPE is unwrapped explicitly (the raw envelope is never
      parsed as an answer set). 12 tests in-module, all green.
- [-] A2.5 **wire fixture (the compatibility evidence — verdict round 1):** capture ONE real
      request/response pair (envelope included, token redacted) as a golden fixture + BLAKE3 pin
      in the lane module. BLOCKED on owner creds (A6) — the lane reads strict until then (every
      missing/foreign shape refuses loud citing A2.5; a reply WITHOUT distributions refuses
      citing A4 — the accuracy-only fallback is deliberately unimplemented against an unverified
      shape).
- [x] A3 `run_clef_lane` in `harness/runner.rs` + `SuiteResult.clef: Option<LaneResult>` +
      `RunMeta.clef_lane` + `--clef` in `src/bin/harness.rs` (opts, parse, module docs) — the
      exact agentjev wiring incl. warmup (fixed throwaway case, never a measured one), the trim
      law (same question cap as the laya lanes), and the determinism rerun probe.
- [x] A4 metric tail: gold-label accuracy + per-option agreement + abstain handling + **ECE ONLY
      if the reply carries per-option probabilities** (the agentjev triple relies on
      `top_probability`/level probs; Clef's hosted reply may not return them — absent probs → an
      accuracy-only row with the disclosure, never a fabricated ECE). Refined in implementation:
      v1 REFUSES on a distribution-less reply (loud, citing A4) rather than shipping a fallback
      row against an unverified shape — implement the fallback FROM the A2.5 fixture, not from
      the guess.
- [x] A5 clippy `-D` clean at default + `--all-features`; `cargo test` gates green; harness
      `--skip-laya --clef --suites typed_decisions` smoke (refusal path verified WITHOUT creds
      first — the loud refusal is itself a gate). Done 2026-10-02: lib+all-targets+all-features
      clippy green, `cargo test --lib` 256/0, harness_units 37/37; the smoke ran on
      `harness_cache_reuse` (synthetic, no datasets needed on this box) and the refusal fired
      loud with the full env contract. A5 EN-ROUTE repairs (pre-existing-at-HEAD breaks the
      all-features/all-targets lanes hit on this box, fixed en route, not plan work): the clm
      loop's dropped `let req = clm_request(case)?;` (its own restore comment in runner.rs) + the
      stray duplicate `determinism_ok`; `examples/typed_case_split.rs`'s whole-file `#![cfg]`
      → the house two-arm main-dispatch shape (E0601 on non-macOS).
- [-] A6 owner step (blocked on creds): `CLOUDFLARE_ACCOUNT_ID` + a Workers AI API token in the
      environment; recorded in the lane's docs as an env contract, never a committed value. Spend
      ceiling: `CLEF_SMOKE_MAX_CASES` (default 50) — a run above the ceiling REFUSES unless
      explicitly overridden (pricing is undisclosed; no unbounded spend). The ceiling + override
      enforcement SHIPPED with the lane (A1/A3); only the creds themselves are owner-gated.
      **Probed 2026-10-03 (m3):** the two provided tokens (`CF_API_TOKEN`, `CF_DEPLOY_API_TOKEN`)
      are valid CF tokens (accounts list 200, one account visible) with **no Workers AI scope** —
      403 code 10000 on `ai/models/search`, 401 code 10000 on `ai/run` for `clef` AND the full
      `@cf/cloudflare/clef[-flash]` spellings. Account id for the run path:
      `7e11517c4dd4f6e9cede7da9b60d66eb`. The scoped token (Account → Workers AI → Edit/Run) is
      the remaining owner act; local rig in gitignored `.raw/clef_fwd/` (probe + A0 forwarder +
      runbook README).

## Phase B — the JDI protocol adapter (riir-reflex)

- [x] B1 snapshot the JDI reference into `.benchmarks/data/jdi/`: `index.json` + `README.md`
      excerpt with blake3 pins + `generated_utc` + suite `corpus_sha256` recorded in a
      `PROVENANCE.md` (read-only reference data; NEVER our gold; re-snapshot = new bench number).
      Done 2026-10-02: fetched 952,868 B, snapshot sha256 `37526dd3…0d50`, in-file generated_utc
      `2026-09-28T00:39:36+00:00` + edition `release-v2.1` + corpus sha256 `b2b56d6f…d5` — all
      three match the research pin exactly. Note: `.benchmarks/data/` is gitignored, so the
      snapshot is LOCAL-ONLY by design and the pins live in the tracked PROVENANCE.md there —
      re-fetch with the recorded URL + verify the sha256. ⚠ A webReader fetch of the same URL
      served a DIFFERENT cached variant (2026-09-22 edition) — direct curl is authoritative.
- [x] B2 macro-F1 readout in the harness metrics (per-suite, one-vs-rest over gold labels) — the
      JDI's retrieval-area metric; additive column, existing accuracy unchanged. DISCOVERY on
      implementation: the harness ALREADY computes `hard.macro_f1` (protocol §5.1 port, on every
      lane row since the openthai/agentjev lanes) — the plan's premise ("our harness currently
      reads accuracy") was stale. JDI-comparability note: it classes on gold option-INDICES;
      every JDI-relevant suite carries FIXED option sets (banking77 presents all 77 in ClassLabel
      order per case), so index identity IS label identity there; only sampled-distractor suites
      diverge, and they carry no JDI crosswalk cell.
- [x] B3 chance-corrected skill column: `(score − chance)/(1 − chance)` with chance levels
      **computed from the label distribution actually evaluated** on our (capped) suite — a
      stratified cap that drops labels has a different chance level than the board's full-label-set
      figure (verdict round 1); the pinned board chance values stay REFERENCE columns only.
      Shipped as `chance_majority` (majority gold-class share) + `chance_corrected_skill`
      (clipped [0,1]; chance→1 guards to 0) in `harness/metrics.rs`, carried on every
      `LaneResult` as `jdi_chance`/`jdi_skill` (results.json), rendered in the crosswalk table.
- [x] B4 coverage disclosure: answered/unsupported/errors printed per lane row (JDI law: an
      unanswered request counts wrong — our lanes already refuse-and-fail-loud, this makes it
      visible in the table). Rendered as the crosswalk header's coverage line (our rows are
      structurally answered=n/0/0 — a lane row exists only if EVERY case was answered; per-row
      columns would be constants). Clef adds the RunMeta posture line with the forwarder hop.
- [x] B5 the honesty law, enforced in the table header text: "JDI-comparable crosswalk, not a JDI
      board row — different corpus, protocol caps, hardware; board membership requires their full
      frozen suite." The harness families stay quarantined from any JDI column (Issue 059 law).
      The crosswalk renders ONLY for the dataset population — read off the SUITES registry
      (`synthetic: None ∧ !named_only`), never a hand-typed name list; verified on the A5 smoke
      (0 renders on harness_cache_reuse).
- [x] B6 gates: unit tests for macro-F1 (hand-checked toy), skill score (clipping + chance
      arithmetic), and the refusal shapes; count pins in the existing gate files. Done:
      `chance_majority` + `chance_corrected_skill` known-answer tests in `tests/harness_units.rs`
      (37/37 green); macro-F1's existing tests unchanged; the lane's refusal shapes pinned by the
      12 in-module clef tests (A2).

## Phase C — Rethink-on-JDI cells + the site publication

- [x] C1 run the dataset suites with `--clef` — **PARTIALLY DONE 2026-10-03 via the LOCAL lane**
      (owner pivot: no Workers AI creds needed — run the open weights like every other lane):
      smoke runs landed (`.benchmarks/113_clef_lane/smoke_{banking77,typed}`, 50-case caps,
      det ✓, envelope+model provenance verified); full-N comparable runs (banking77 500 + typed
      400) in flight into `.benchmarks/113_clef_lane/full_*` — record doc + provenance + the
      preflight-quated load caveat (latency rows provisional at load 6.36; accuracy
      load-invariant) lands with them. The HOSTED C1 run stays deferred per A6.
      **The latency caveat CLOSED 2026-10-04 (bench 118, the quiet-box re-read):** both full-N
      cells re-measured on a preflight-clean box (load 2.57→1.07 / 1.22→2.80, AC, High Power;
      the 29 GB swap drained across the runs) — **latency QUOTABLE both ends both suites**:
      b77 p50 3368 ms (was 3901 provisional, −14%) · typed 1690 ms (was 2377, −29%), with
      accuracy/ECE/macro-F1 and the population digests reproducing 113 EXACTLY. The site cells
      graduated from acc-only (reflex-site `4297d32`, deployed `d1997b32`); the p50 presence-row
      pin retired with the state (clef plots a real bar).
      **THE DATASET-SUITE TAIL LANDED 2026-10-04 (bench 119, the time-not-money remainder):**
      the standing local rig answered the remaining 7 suites as per-suite docs on a quiet box
      (every doc quotable both ends, loads 1.1–2.7; modelless controls == board on all seven —
      the drift guard). code_fixtures 0.5938/673 ms · prompt 0.5862/330 ms · xnli 0.8133/351 ms ·
      massive **0.9333**/903 ms · emotion 0.5925/416 ms · ag_news **0.9000**/456 ms · sst5
      0.6033/409 ms. `areas.timing.clef` = **9/0/0** (p50 geomean 680 ms) over the lane's full
      9-suite index (the bekko/openthai/paw comparison population — thai/s1mb stay out per lane
      consistency). The crosswalk auto-grew to 9 suites (the publisher's clef-carrying
      population; the digest-pin excludes unpinned hybrid/encoder rows loudly) and the TL;DR
      card now covers every suite with honest TIE phrasing (code_fixtures: clef/openthai/bekko
      all 0.5938 — “ties”, never a false “leads”; renderer fix + smoke arm). Clef leads
      sst5/massive/banking77; is led against on typed (Rethink rec), ag_news (bekko 0.915),
      emotion + prompt (Instinct), xnli (openthai 0.8967). Published (reflex-site `365d0a0`,
      deployed `fd9515a3`, live-verified).
- [x] C2 the Rethink cells on the same runs come from the instinct arena's published rows
      (hybrid_lane_doc) — add the macro-F1 + skill columns to the lane-doc build so Rethink and
      the lanes read on the same axes (riir-instinct side; small, display-only). **Case-identity
      pin (verdict round 1): the Clef run and the Rethink cell must carry the SAME case-ID set /
      split hash and the same trim cap — asserted before any crosswalk cell is published; without
      the pin, the cell carries the B5 "different corpus" caveat like every other crosswalk row.**
      **LANDED 2026-10-04 (the hybrid half; instinct `a2112e9` + reflex `8cfc6bc`):** reflex exports
      `Seat::cases_digest` (the population-identity law, ONE home — the consumer calls it, never
      re-derives); the arena freezes `test_digest` + per-question `gold` on every run; the
      lane-doc builder derives `macro_f1`/`jdi_chance`/`jdi_skill` (reflex's laws mirrored —
      pinned against reflex's OWN harness_units hand-case vectors, 11/15 + 1/3 exact) natively or
      via a digest-gated `--gold-from` seat_identity dump for legacy records; the `seat_identity`
      example dumps the pin offline. **Live-proven on real data**: the rebuilt seats' digests ==
      the clef full-run docs' `cases_digest` (b77 `fnv1a64-dd8ab35333abb82a`, typed
      `fnv1a64-6e37760ee2a5b6c9`) — the cross-repo pin works end-to-end. First JDI-axis readings
      for a Rethink cell (the bench-041 typed encoder record via --gold-from): macro_f1 **0.7445**,
      jdi_skill **0.6537** (chance 0.2925) vs Clef's reflex-computed macro_f1 0.6742 on the same
      digest. **REMAINS for C2:** the ENCODER lane-doc emitter is an archived hunk in
      riir-rethink post-split (`bin_hunks/arena_encoder_lane_doc.rs`) — new encoder records gain
      the axes only when rethink re-activates its emitter (the bench-041 legacy cell already
      carry them via the --gold-from path, which is what C3 cites). The arena stamping change is
      in effect for every FUTURE instinct record. **REMAINDER DISCHARGED 2026-10-04 (the rethink
      half; reflex `macro_f1_of` + rethink's emitter re-activation):** reflex exports the
      macro-F1 law as `harness::metrics::macro_f1_of` (the ONE home `hard_metrics` itself
      consumes — picked against the SAME 7/9 hand vector instinct's Python builder pins with,
      cross-media); rethink's active emitter (`esc_margin_refit.rs`, the post-split home of the
      lane-doc laws) stamps every record NATIVELY — `test_digest` (the seat's own
      `cases_digest`, consumed), per-question `gold`, and `jdi_test` (cheap/think/composed
      macro_f1+jdi_chance+jdi_skill, reflex's laws consumed, never mirrored) on BOTH records
      (0057 predictions.json + 0058 esc_lane_doc.json + its RESULTS.md JDI table). **Live-proven
      by re-run:** all four suites reproduce the prior accuracy AND escalation rates EXACTLY
      (the determinism witness), the typed digest == the C2 pin (`fnv1a64-6e37760ee2a5b6c9`),
      and the typed think axes reproduce the --gold-from readings BIT-FOR-BIT (mF1 0.7445 /
      skill 0.6537 / chance 0.2925) — the native path == the join path.
- [x] C3 reflex-site publication (owner-adjacent repo): the crosswalk table (Clef vs Rethink hybrid
      vs Rethink encoder vs laya vs GLiNER vs CLM vs AgentJev vs the JDI board reference rows) on
      /bench/ + the TL;DR card; caveat text from B5 rendered verbatim; `publish_bench.py` +
      `republish_bench.sh` flow.
      **PARTIAL 2026-10-04 (reflex-site `5c31fa6`, deployed `4cdaa395`, live-verified): the LANE is
      published** — edition **2026-10-2** (the lane-set basis bump, the ledger's second row):
      publisher support (LANE_CLASSES/DISPLAY/KIND/TIMING/AREA_LANES + the merge carry block),
      `clef:acc-only` cells live on /bench/ (banking77 **0.9540** / typed **0.6955**, carrying
      macro_f1 + jdi_skill on the same axes as every lane), the clef palette slot + filter chip +
      table rows + hero bars (data-derived smoke arms added: 2 rows, 12 unmeasured-suite not-runs,
      the p50 PRESENCE-row pin re-stated as "names WHO wears it" — clef, acc-only re-read pending);
      the outgoing 2026-10 table frozen to data/archive. **REMAINDER DISCHARGED 2026-10-04 (reflex-site
      `53d518f`, deployed `53515e17`, live-verified): the CROSSWALK TABLE is live** on /bench/#crosswalk —
      `compute_crosswalk` (the compute_areas precedent) builds the side-by-side per clef-carrying suite:
      every measured row OUR cell with its `cases_digest` ASSERTED EQUAL to the clef cell's pin at build
      time (a different population or an unpinnable lane is EXCLUDED with the reason rendered, never
      silently mixed; derived tier-fallback encoder cells excluded by construction); the JDI axes read
      both cell spellings (harness cell-level vs instinct-doc hard-level). The JDI board's own rows +
      Clef's blog banking77 macro-F1 table ride as curated REFERENCE constants (the AREA_CHANCE
      precedent, provenance-pinned to `.research/005` / the B1 snapshot) under the B5 caveat VERBATIM
      (publisher-carried; the renderer never re-words it). `stamp_cell` carries suite-level
      `test_digest` onto the cell as `cases_digest` — the hybrid/encoder cells joined the pin via the
      C2 doc rebuilds (instinct `0ac9375`: the 0052 rebuild + the 041 surgical stamp). First live
      read: b77 clef 0.9540/skill 0.9533 vs Instinct 0.8540/0.8519 on pin fnv1a64-dd8ab…; typed clef
      0.6955/0.5696 vs Rethink encoder 0.7550/0.6537 (record-only) vs Instinct 0.6475/0.5018 on pin
      fnv1a64-6e37…. Smokes: 3 new data-derived crosswalk arms (tables count / caveat verbatim /
      pins) + the full gate set green. **TL;DR CARD LANDED 2026-10-04 (reflex-site `dd5409d`, deployed
      `6bee75de`, live asset verified):** one verdict line per suite under the caveat — the leader by
      accuracy (rec tag when record-only) with skill axes, plus clef's own p50 posture from its lane
      cell (`at 3.37 s p50 (clef-flash-4bit)` / `1.69 s`). Everything data-derived from the crosswalk
      block + the clef lane cell, never typed (the site law); OUR rows only — the board/blog reference
      numbers never enter it, so the B5 caveat governs what it excludes by construction. First live
      lines: `typed_decisions — Rethink [rec] leads at 0.7550 acc / 0.6537 skill over Clef 0.6955 /
      0.5696 at 1.69 s p50` · `banking77 — Clef 0.9540 acc / 0.9533 skill at 3.37 s p50 leads every
      measured lane (best other: Instinct 0.8540)`. Smoke arm pins the leaders against the data's
      argmax (clef-led suites read the fixed `— Clef` phrasing; record-only leaders carry the rec tag).
- [-] C4 headline verdict recorded in HISTORY.md: where Clef actually lands vs Rethink on
      typed_decisions + banking77, with the cost + latency posture disclosed. **Reading note
      (verdict round 3): neither current Rethink-side figure decides the comparison — Rethink's
      0.7550 is record-only and AgentJev's 0.7715 is det ✗ — so CLEF'S HOSTED RUN ON THE SAME
      CASE SET (the C2 pin) is the cell that decides the comparison.**
      **Update 2026-10-07 (rethink issue 021 / Bench 061 / reflex-site `260705a`): the Rethink-side
      disqualifier is HALF-GONE — the typed encoder cell is re-seated QUOTABLE** (fresh live Metal
      encode over the IDENTICAL pin `fnv1a64-6e37760ee2a5b6c9`: acc 0.7550 byte-consistent,
      p50 47.335 ms / p99 74.193 ms, both-ends box_state quotable; the prior 364 ms read was
      ambient load, 7.7×). "Record-only" now names the SERVE posture only (the 014 class-wide
      refusal — the incumbent arm serves), never an unfit timing. Quotable local-posture picture:
      typed — Rethink 0.7550/0.6537 skill @ 47 ms vs Clef-local 0.6955/0.5696 @ 1690 ms (Rethink
      leads +6.0 pt at ~36× lower p50); banking77 — Clef-local 0.9540/0.9533 leads every measured
      lane (the Rethink encoder's b77 row is tier-fallback, excluded by construction — rethink
      issue 016 D1 seating, owner-gated). The cell that still decides the HEADLINE (the blog's own
      hosted posture) remains Clef's HOSTED 27B run — A6 owner-gated.

## Phase D — deferred (lane-intel follow-ons, each its own session)

- [-] D1 local Clef serving in riir-infer as a PREFILL-LEAGUE workload (Clef = prefill-only 27B
      parallel-scoring in our league arch; EXL3 4-bit quants exist). GGUF is NOT sufficient (custom
      scoring head is transformers code — llama.cpp cannot reproduce typed outputs); vLLM + their
      custom-code is the local path. Gated on a GPU window + the prefill league's next cell.
- [x] D2 riir-train recipe note: Brier calibration term + label-smoothed CE + RLCD partial-credit /
      reference-penalty vs our t608 NLEH v2 per-option head's loss; file as a riir-train research
      note + optional plan when a typed-head retrain window opens. **DONE 2026-10-02**:
      riir-train `.research/464_Clef_PostTraining_Recipe_vs_Typed_Head.md` at `115e5690`
      (verdict-pinged 3 rounds to AGREE). Key corrections the review survived: ADD vs MIX Brier
      spellings are the same training run under Adam — the lever is the relative confidence-
      weighting shape, λ=1 pure-Brier replacement the honest maximum; ordinal partial credit
      applies to the security_incidents severity question ONLY (action keys nominal; text→rank
      table, never option index); label smoothing ranks LOW on the v1 teacher arms' every-lane
      negative; the gate is paired holdout accuracy LB95 ≥ 0 AND paired pick-level Brier LB95 < 0
      over the consumer's exact read (`score_row_v2` conf = the picked option's raw sigmoid), ECE
      disclosure-only, NLL printed beside. Cost CPU-minutes/arm — the retrain window is
      owner-priority-gated, not resource-gated; the plan derives from the note's §3 when the
      trigger fires (C4 reads Clef ≥ 0.7550 on typed_decisions, or a serving posture appears).
- [-] D3 submit Rethink to the actual JDI board (typesafe-diffusion-lab suite, their frozen corpus,
      their protocol) — the full "on the board" form. Owner-gated (GPU budget + their submission
      process); only meaningful after C4 shows the crosswalk is worth formalizing.

## Validation

Per phase: clippy `-D` + cargo test + the lane's own wire tests (A), metric unit tests (B), a real
smoke run with det probe + box-state provenance + republished site (C). No phase publishes without
its gate.
