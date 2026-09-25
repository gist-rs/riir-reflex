# Plan 003 — Thai lane, research-sake: the OpenThai measurement lane + Thai posture pins (EN code untouched)

**Status:** OPEN — filed 2026-09-25 from `.research/003`; research-sake priority (owner: "we not focus in Thai but just for research sake") — idle-window work, never ahead of owner-flagged lanes. This plan is the task tracker (no `.issues` twin — the noise-reduction rule; issues fire only if a phase splits into parallel independent work).

The owner ask: *handle Thai without contaminating EN code as possible, e.g. around tokenizer and other — feature flag or new crate as boundary, for research sake.* The research note's verdict shapes this plan:

1. **No tokenizer work exists to do** — OpenThai's Thai-ness is CPT weights over stock byte-level BPE; our laya lane is the same shape (pinned `tokenizers` 0.22 BPE); the modelless embedder is script-agnostic-by-hashing. The contamination question is about code seams, not tokenizers.
2. **Our lanes already degrade safely on Thai** (frozen G5 rows `ml-thai` / `ml-thai-collapse`) — Phase 1 pins the one UNPINNED posture (modelless) as contract.
3. **Real Thai capability = external specialist measured, not ported** — `--openthai` lane (their stack serves, our Rust measures; the agentjev lane-family law).

## Non-contamination contract (every phase answers to these)

- **G-ISO-1 (serve-path byte-identity):** the `reflex` serve bin's answers are byte-identical before/after every landing here; lane code is reachable only from the `harness` bin — pinned by a serve-path test (the game-heads byte-identity pattern).
- **G-ISO-2 (default suites unchanged):** the harness's default suite set is untouched; Thai suites are opt-in via `--suites thai_*` exactly like every other suite selection.
- **G-ISO-3 (G5 fixtures frozen):** `tests/fixtures/laya_parity_v1.jsonl` + expected capture untouched (`ml-thai` rows already frozen — no re-capture, no re-pin).
- **G-ISO-4 (imports law, not zero-dep):** the lane stays **agentjev-shaped** — zero new packages AND no katgpt-core surface beyond what the default build activates (imports only `crate::harness::suites` + std + serde_json). NOT "zero deps ⇒ ungated": `clm-lane` is zero-new-packages yet gated, because it imports `katgpt_core::decision_wire` (inactive at `--no-default-features`). **Trigger, recorded**: if any openthai module or test ever imports `katgpt_core::decision_wire`, the lane takes `openthai-lane = ["katgpt-core/decision_wire"]` (the clm-lane shape) and G-ISO-4 reads as satisfied BY that feature, not violated.

## Phase 1 — Thai posture pins (tests only, zero runtime code)

- [ ] T1.1 Modelless Thai posture pin (`tests/`, modelless-gated): embed + full engine decide on the G5 `ml-thai` fixture text (reuse the exact state strings from `laya_parity_v1.jsonl`) → assert determinism (bit-identical ×2) and RECORD the distance-gate verdict honestly: abstain expected; if the engine answers with confidence, that is a FINDING (file it — do not tune it away silently).
- [ ] T1.2 Serve-edge Thai safe-degradation pin: `/decide` on a Thai state returns a well-formed answer (expected: abstain-forward) — no panic, no NaN, no 5xx; the zero-vector law covers empty embeddings.
- [ ] T1.3 Record the pinned posture in `.docs/` (one row in the appropriate protocol/doc page — the posture table from research 003 becomes repo-visible).

## Phase 2 — the `--openthai` comparison lane (self-contained module)

- [ ] T2.1 `src/lanes/openthai.rs` — self-contained (own ~90-line HTTP/1.1 client; the third HTTP client — `lanes/http.rs` extraction deliberately DEFERRED, see Phase 4, isolation chosen over DRY for this landing). Wire: `POST {OPENTHAI_SERVE_URL:-http://127.0.0.1:8000}/v1/systemone`; body `{state, questions: {qid: {type, instructions, criteria}}}` built from `SuiteCase` (near-native vocabulary: noul/choice/score + criteria dict/array pass through); response mapping per research 003: noul `noul`→`[1-p, p]` (conf = max side); choice `probabilities` dict read in criteria insertion order, `choice` field → pick, `confidence` → conf, `abstain` recorded as provenance observation; score `probabilities` keyed `"0".."k-1"` in level order, pick = argmax (their `score` is E[level] — float; pick from the distribution), conf = `confidence`.
- [ ] T2.2 Provenance posture (divergence from agentjev recorded): no `/api/info` endpoint — the lane records the response's `model` field + `usage.input_tokens`; `/healthz` is the liveness probe. Never a hardcoded model id (the gliner law).
- [ ] T2.3 Determinism pin: `permutations=1` explicitly (their auto mode fires 8 cyclic perms at choice ≥ 11 options — banking77-class suites MUST pin `permutations: 1` for byte-comparable rounds; the pin uses `decide_raw` ×2 byte-compare, the agentjev law).
- [ ] T2.4 Wire-shape unit tests over a stub HTTP listener (the clm T2(b) pattern): body build per QKind, answer mapping incl. missing-qid error, URL parse, HTTP split. **Import law enforced here**: these tests take `SuiteCase` + stub bytes only — the moment any openthai module or test needs `katgpt_core::decision_wire`, land `openthai-lane = ["katgpt-core/decision_wire"]` in the SAME commit (the G-ISO-4 trigger).
- [ ] T2.5 Harness flag `--openthai` + lane column wiring (mirror the agentjev flag block; loud absence when the server is unreachable — never a silent skip).
- [ ] T2.6 EN cross-check cells (free, no new data): `--openthai --suites massive_intent_en,xnli_en` against their published 88.3 / 89.0 — our measurement vs their card on suites we already fetch.

## Phase 3 — Thai probe suites (opt-in, small, honest)

- [ ] T3.1 `scripts/fetch_datasets.sh`: add `thai_wisesight` (4-class sentiment choice — their WEAKEST set 51.6/ECE 0.353, deliberately chosen for honesty, not cherry-picking) + `thai_sib200` (7-way topic, 204 rows whole-set) via the datasets-server mechanism; blake3 digests into the dataset manifest (the existing law). Defer `thai_massive`/`thai_xnli` (2.5k–5k rows) until the probe lanes prove interesting.
- [ ] T3.2 Suite defs in `src/harness/suites.rs` — opt-in only; no default-suite-list change (G-ISO-2).
- [ ] T3.3 First Thai board run (4090 or M3 window; their server via `uvicorn openthai_systemone.server:app`, MPS/CUDA posture recorded) → `.benchmarks/041_openthai_thai_probe.md` with box-state provenance (the `bench_preflight` law where applicable) — lanes: openthai vs laya-multilingual vs modelless (the posture from Phase 1) — the arena's first Thai board.
- [ ] T3.4 Site republish IF the board lands (`../reflex-site/scripts/republish_bench.sh` — the mirror law).

## Phase 4 — deferred arms (`- [-]`, reopen triggers recorded)

- [-] T4.1 `lanes/http.rs` extraction (third HTTP client landed in T2.1; extraction touches the EN lanes' files — deferred to keep this landing EN-untouched; trigger: the FOURTH HTTP lane or any behavior fix that all three need).
- [-] T4.2 `thai` engine feature (default-off): script-detect → segment → space-join → the EXISTING embedder unchanged; byte-identity gate when off. Substrate candidates (research 003): unigram Viterbi over a Thai-covering vocab (in-stack precedent) / newmm-class dictionary (open-licensed wordlist asset — PUBLIC repo constraint) / char-n-gram hashing. **New crate `crates/riir-infer-thai` only if the vocab/tokenizers tree is needed** (the laya-substrate precedent) — else reflex-local module. Trigger: an owner product call for Thai-capable modelless decisions.
- [-] T4.3 riir-train Thai decision-head training — recipe recorded in research 003; trigger: 002's trigger AND a Thai product surface.

## Validation

- Every phase: `cargo clippy --all-targets -- -D warnings` + `cargo test` green at default features; G5 parity green at `laya-riir` posture (untouched by construction).
- G-ISO-1 byte-identity test green after each landing.
- The lane's numbers publish ONLY from a run with the determinism pin green (permutations=1) — no PROVISIONAL cells in committed tables without the pin.
