# Plan 011 — the Clef comparison lane + the Jev Decision Index protocol adapter

**Status:** IN FLIGHT — filed 2026-10-02 from `.research/005_Cloudflare_Clef_Jev_Decision_Index.md`
(owner ask: research the Clef blog, bench Rethink against the Jev Decision Index like the blog does,
plan a `--clef` run published to reflex.gist.rs/bench like the other lanes). Hosted-Clef credentials
are owner-gated; phases run in order, each commits+pushes its own unit.

Every number this plan publishes obeys the house laws: their stack serves, our Rust measures
(laya-python measurement law); latency = HTTP round-trip with the serving posture + box state quoted
(bench_preflight / G2 box-state law); a lane without creds REFUSES loud (agentjev law), never
half-runs; published tables are CI-regenerated, never hand-typed.

## Phase A — the `--clef` comparison lane (riir-reflex)

- [ ] A0 **transport decision (blocks A1 — verdict round 1 finding):** the harness lanes are
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
- [ ] A1 `src/lanes/clef.rs` — sibling of `agentjev.rs` (NOT a config flag):
      `ClefLane { url, token, model, timeout }`, `from_url`, a **probe handshake `info()` = one
      minimal `noul` request** (the Workers AI route has no `GET /api/info`; the reply proves the
      wire shape and carries model identity when the envelope provides it), `decide(case)` /
      `decide_raw(case) -> AnswerTriple` + raw probs; `CLEF_SERVE_URL` (default the loopback
      forwarder), `CLEF_API_TOKEN` (bearer), `CLEF_MODEL` (`clef`|`clef-flash`),
      `CLEF_TIMEOUT_MS`.
- [ ] A2 wire mapping tests in the lane module (the agentjev.rs test shape): body passes `state`
      through; `choice` renders `criteria` as the options map (key = desc), `score` renders the
      levels list, `noul` carries no options (wire law); answers map back in OUR option order;
      missing qid in the reply is a loud error, never a positional guess; the Workers AI
      `{result, success, errors}` ENVELOPE is unwrapped explicitly (the raw envelope is never
      parsed as an answer set).
- [ ] A2.5 **wire fixture (the compatibility evidence — verdict round 1):** capture ONE real
      request/response pair (envelope included, token redacted) as a golden fixture + BLAKE3 pin
      in the lane module. Until this lands, the wire claim stays "Jev-shaped per the vendor; to be
      verified" — never asserted.
- [ ] A3 `run_clef_lane` in `harness/runner.rs` + `SuiteResult.clef: Option<LaneResult>` +
      `RunMeta.clef_lane` + `--clef` in `src/bin/harness.rs` (opts, parse, module docs) — the
      exact agentjev wiring incl. warmup (fixed throwaway case, never a measured one), the trim
      law (same question cap as the laya lanes), and the determinism rerun probe.
- [ ] A4 metric tail: gold-label accuracy + per-option agreement + abstain handling + **ECE ONLY
      if the reply carries per-option probabilities** (the agentjev triple relies on
      `top_probability`/level probs; Clef's hosted reply may not return them — absent probs → an
      accuracy-only row with the disclosure, never a fabricated ECE).
- [ ] A5 clippy `-D` clean at default + `--all-features`; `cargo test` gates green; harness
      `--skip-laya --clef --suites typed_decisions` smoke (refusal path verified WITHOUT creds
      first — the loud refusal is itself a gate).
- [ ] A6 owner step (blocked on creds): `CLOUDFLARE_ACCOUNT_ID` + a Workers AI API token in the
      environment; recorded in the lane's docs as an env contract, never a committed value. Spend
      ceiling: `CLEF_SMOKE_MAX_CASES` (default 50) — a run above the ceiling REFUSES unless
      explicitly overridden (pricing is undisclosed; no unbounded spend).

## Phase B — the JDI protocol adapter (riir-reflex)

- [ ] B1 snapshot the JDI reference into `.benchmarks/data/jdi/`: `index.json` + `README.md`
      excerpt with blake3 pins + `generated_utc` + suite `corpus_sha256` recorded in a
      `PROVENANCE.md` (read-only reference data; NEVER our gold; re-snapshot = new bench number).
- [ ] B2 macro-F1 readout in the harness metrics (per-suite, one-vs-rest over gold labels) — the
      JDI's retrieval-area metric; additive column, existing accuracy unchanged.
- [ ] B3 chance-corrected skill column: `(score − chance)/(1 − chance)` with chance levels
      **computed from the label distribution actually evaluated** on our (capped) suite — a
      stratified cap that drops labels has a different chance level than the board's full-label-set
      figure (verdict round 1); the pinned board chance values stay REFERENCE columns only.
- [ ] B4 coverage disclosure: answered/unsupported/errors printed per lane row (JDI law: an
      unanswered request counts wrong — our lanes already refuse-and-fail-loud, this makes it
      visible in the table).
- [ ] B5 the honesty law, enforced in the table header text: "JDI-comparable crosswalk, not a JDI
      board row — different corpus, protocol caps, hardware; board membership requires their full
      frozen suite." The harness families stay quarantined from any JDI column (Issue 059 law).
- [ ] B6 gates: unit tests for macro-F1 (hand-checked toy), skill score (clipping + chance
      arithmetic), and the refusal shapes; count pins in the existing gate files.

## Phase C — Rethink-on-JDI cells + the site publication

- [ ] C1 run the dataset suites with `--clef` on the 4090 box (smoke: typed_decisions + banking77
      first, capped by `CLEF_SMOKE_MAX_CASES`; then the 15-suite lane as budget explicitly allows)
      — record in `.benchmarks/NNN_clef_lane/` (next number per `.benchmarks/.highwater`) with
      tables + provenance + det probe result + the forwarder posture disclosed.
- [ ] C2 the Rethink cells on the same runs come from the instinct arena's published rows
      (hybrid_lane_doc) — add the macro-F1 + skill columns to the lane-doc build so Rethink and
      the lanes read on the same axes (riir-instinct side; small, display-only). **Case-identity
      pin (verdict round 1): the Clef run and the Rethink cell must carry the SAME case-ID set /
      split hash and the same trim cap — asserted before any crosswalk cell is published; without
      the pin, the cell carries the B5 "different corpus" caveat like every other crosswalk row.**
- [ ] C3 reflex-site publication (owner-adjacent repo): the crosswalk table (Clef vs Rethink hybrid
      vs Rethink encoder vs laya vs GLiNER vs CLM vs AgentJev vs the JDI board reference rows) on
      /bench/ + the TL;DR card; caveat text from B5 rendered verbatim; `publish_bench.py` +
      `republish_bench.sh` flow.
- [ ] C4 headline verdict recorded in HISTORY.md: where Clef actually lands vs Rethink on
      typed_decisions + banking77, with the cost + latency posture disclosed. **Reading note
      (verdict round 3): neither current Rethink-side figure decides the comparison — Rethink's
      0.7550 is record-only and AgentJev's 0.7715 is det ✗ — so CLEF'S HOSTED RUN ON THE SAME
      CASE SET (the C2 pin) is the cell that decides the comparison.**

## Phase D — deferred (lane-intel follow-ons, each its own session)

- [-] D1 local Clef serving in riir-infer as a PREFILL-LEAGUE workload (Clef = prefill-only 27B
      parallel-scoring in our league arch; EXL3 4-bit quants exist). GGUF is NOT sufficient (custom
      scoring head is transformers code — llama.cpp cannot reproduce typed outputs); vLLM + their
      custom-code is the local path. Gated on a GPU window + the prefill league's next cell.
- [-] D2 riir-train recipe note: Brier calibration term + label-smoothed CE + RLCD partial-credit /
      reference-penalty vs our t608 NLEH v2 per-option head's loss; file as a riir-train research
      note + optional plan when a typed-head retrain window opens.
- [-] D3 submit Rethink to the actual JDI board (typesafe-diffusion-lab suite, their frozen corpus,
      their protocol) — the full "on the board" form. Owner-gated (GPU budget + their submission
      process); only meaningful after C4 shows the crosswalk is worth formalizing.

## Validation

Per phase: clippy `-D` + cargo test + the lane's own wire tests (A), metric unit tests (B), a real
smoke run with det probe + box-state provenance + republished site (C). No phase publishes without
its gate.
