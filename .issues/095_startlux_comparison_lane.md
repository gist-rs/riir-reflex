# Issue 095 — StartLux-Decision v1.1 comparison lane (`--startlux`)

**Status:** PARTIAL — T1+T2 LANDED 2026-10-21 (`3a9e8f5`), T3 board pass EXECUTED same day (Bench 155 — 4/8 incumbent suites beaten), T4 capture in flight. Filed 2026-10-21 (4090 box, feeding riir-train [635](../riir-train/.issues/635_4090_startlux_teacher_capture_lane.md) T3). **Claimed:** zed-635-startlux-t3, 1792611000, expires 2026-10-24T00:00:00Z.

## Why

riir-train 635's T3 needs a `--startlux` comparison lane in THIS repo's harness (the
`--d3` shape) before any teacher capture or retrain spend (the B5 law: the lane's
read decides). StartLux-Decision v1.1 27B Instant is board #1 (public_skill 72.88,
coverage 1.0, ECE 0.0261 on the pinned snapshot), open weights, speaks
`/v1/systemone` NATIVELY (their Apache-2.0 engine `startlux-decide` v1.1.0 — the
TypeSafe SystemOne dialect this harness already parses twice, pplx + d3), and
shares the serving base family (Qwen3.8-27B — same base as pplx v1.1, d3, and our
`pmscr` row). The 72.88 is a SELECTION-ON-PUBLIC number; the our-suite read is the
honest comparator (the B5 law cuts both ways).

## License (the lane's first divergence from its siblings)

Weights **CC BY-NC 4.0** (DECLARED-NC → RESTRICTED posture, adjudicated in 635 T0:
capture-for-MEASUREMENT allowed, teacher-CAPTURE allowed, any ADOPTION
fail-closed). Unlike pplx/d3 (Apache-2.0, teacher-eligible-and-shippable) and like
jeva/drex, this is a measurement posture lane: comparison reads + RIDT captures
with the license token riding the header — never a product lane, never a shipped
column. The **NC-lineage law** (635, binding): every artifact deriving from
StartLux captures carries `NC-lineage: startlux-decision-v1.1 (CC BY-NC 4.0)`.

## The wire (pinned from THEIR source, engine v1.1.0 — not a captured fixture)

Same TypeSafe SystemOne dialect as pplx/d3 — `POST /v1/systemone` — with THREE
pinned divergences (all verified in `startlux_decide/{server,controls,pipeline,
prompt,readout}.py` @ the clone's v1.1.0):

1. **Request extras are IGNORED, not forbidden**: their server reads only
   `state`, `questions`, `images` + the known controls (`think_mode`, `gate`,
   `mix`, `think_budget`, `allow_none`, `none_check`, `return_details`,
   `return_thought`, `thought_generation`) — an extra `model` field (pplx
   `build_body`'s shape) is never read. So the pplx wire functions are REUSED
   verbatim (one wire, many oracles — the d3 law).
2. **Health is `status: "ok"`** (never "ready"/"loading") and the FULL shape
   (no API keys on a loopback serve) carries `kind`, `prompt_attention`,
   `engine` (with `bidir_mode`), `defaults`, `context_length`. The lane's
   construction gate checks the POSTURE, not just liveness: `kind == "instant"`,
   `prompt_attention == "bidirectional"`, `engine.bidir_mode == "server"` — a
   causal/off-bidir serve misreads the Instant checkpoint (the −26 pt class,
   635 T2's reason for the exact-tag non-causal build). Refusal names what to
   fix, never a half-run.
3. **Answers carry `none_probability`** (choice answers, `none_check` default-on
   at serve) — extra vs the pplx parser, which ignores unknown fields. The pplx
   `map_answers` parses the rest verbatim: noul `{"type","noul"}` (no confidence
   on their wire either), score `{"type","score","confidence","probabilities"}`
   keyed `"0".."k-1"`, choice `{"type","choice","confidence","probabilities"}`.
   Their `usage` carries `input_tokens`/`output_tokens` (thought tokens);
   `timing_ms` rides ONLY with `return_details` — which the lane never sets, so
   the no-clock law holds verbatim (determinism compare = raw body, compute
   determinism, never their clock).

Bounds: choice criteria 2..=26 per pass (their `MAX_OPTIONS = 26`, knockout
beyond — 77-way banking77 rows go multi-pass, their engine's job); score 2..=10
(`MAX_LEVELS`); noul criteria optional. All satisfied by the pplx `build_body`
bounds check (1..=255 choice is looser than their knockout path allows — fine).

## Plan

- [x] **T1 — the lane module** `src/lanes/startlux.rs` (ungated: the import
  law — `crate::harness::suites` + `crate::lanes::http_mini` + std + serde_json;
  wire fns reused from pplx). Env contract: `STARTLUX_SERVE_URL` (default
  `http://127.0.0.1:8090` — the engine's own default port), `STARTLUX_TIMEOUT_MS`
  (default 300000), `STARTLUX_DEVICE` (the quant-posture disclosure stamp — the
  lossy-surface law: which quant produced the scores is PINNED and quoted; our
  T2/T2b posture is Q4_K_M `-ngl 99`), `STARTLUX_API_KEY` (optional bearer;
  loopback serve runs without). Stub-listener tests: health posture refusals
  (wrong kind / causal bidir / keys-only public shape), choice+none_probability /
  score / noul round trips through the REAL micro-client. — **LANDED `3a9e8f5`:
  5 tests green; clippy 3 postures (default / --no-default-features /
  --all-features) + lib 443 green.**
- [x] **T2 — the harness wiring**: RunOptions.startlux, SuiteResults.startlux,
  RunMeta.startlux_lane + render line, `--startlux` flag, dump-items list,
  `--distill-teacher startlux` (TeacherForward::Startlux + the license token
  carrying the NC-lineage string). — **LANDED `3a9e8f5` (the dump-items list
  also gained the missing `jeva` row — the same omission class).**
- [x] **T3 — the board pass** (riir-train 635 T3): boot the documented stack on
  the 4090 (llama-server b11549 `-ngl 99` :8081 + `STARTLUX_LLAMACPP_BIDIR=server`
  wrapper :8090), `--startlux --skip-laya` over the 9 suites, one frozen read;
  the paired table vs per-suite incumbents (clef 0.9540 banking77 / 0.9333
  massive / 0.6033 sst5; agentjev 0.7720 typed; openthai 0.9000 xnli; laya 0.9500
  ag_news). A lane that beats no incumbent still feeds 635 T4 (router columns are
  complements). — **EXECUTED 2026-10-21 (Bench 155, `.benchmarks/155_startlux_board.md`):
  9/9 suites, det ✓ ×9, G1-class ECE sane except typed_decisions 0.1581 (the
  T2b overconfidence shape, smaller at serve posture). 4/8 incumbents BEATEN:
  typed +2.25 / sst5 +1.17 / injections +6.90 / xnli +3.67; loses: massive −2.0 /
  ag_news −3.25 / emotion −8.75 / banking77 −4.0. T5 EARNED on the four; latency
  NOT QUOTABLE (sibling build load 13.68 at run start — box-state law, accuracy
  unaffected).**
- [-] **T4 — the capture**: `--distill --distill-teacher startlux` over the T3
  suites — RIDT keyed-probs, license token in the header, captured-but-suspect
  flag + dual-shuffle acceptance gates per the 635 T2b consequences. — **IN
  FLIGHT (same window): full-pool train-row capture running detached
  (`.raw/startlux_teacher/`); ag_news ≈ 90 min (20k rows), banking77
  knockout-dominated (p50 ratio ≈ 4-5×/row); manifest (NC-lineage + suspect
  flag) lands with the bins.**
- [ ] **NOT in this lane** (scope law): perm-probe/null-probe lane arms (the
  order-sensitivity question was answered by the 635 T2b probe: 2.71% — the
  shuffle-robust gate law rides the capture, not another probe), the JDI
  public-suite adapter read (bench-067 instrument — separate window, its own
  datasets fetch), any hosted-endpoint posture (owner-gated, the clef law).

## Queue position

Immediate — riir-train 635 T3 is the unblocked item (T2b done, the stack's boot
line documented, ~3 min reload). GPU-exclusivity law per run; the 10-19
resident-stack lesson applies (tear down after the window).
