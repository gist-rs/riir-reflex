# Issue 033 — PAW comparison lane: ProgramAsWeights as the third external classifier oracle

**Status:** OPEN — lane LANDED (`2e9f351`) + first cells MEASURED keyless (Bench 049, hosted anonymous posture, 4-suite subset, 2026-09-26). Open: `paw-ft-bs48` finetune cells, Posture B lane wiring, arena republish, HISTORY close. Filed 2026-09-25 from `.research/004_ProgramAsWeights_Serving_Landscape.md` (PAW distill, sdk @ `74919f6958b127f10776689277f5a74321857b40`).

## Finding

PAW (ProgramAsWeights, arXiv:2609.04199 for the compile-by-training follow-up) is the
closest **commercial** product to reflex's serving story: compile a natural-language
task spec into a tiny neural function (Q4_0 LoRA over GPT-2 124M / Qwen3-0.6B), run it
locally (llama.cpp / browser WASM) or hosted (~150 ms/call). Its hub carries ~3,800+
community programs, and its marketed workload list (classification, extraction, format
repair, log triage, intent routing) is reflex's decision-workload neighborhood. The
arena currently compares reflex against GLiNER (`--gliner`, 029), AgentJev
(`--agentjev`, 027), CLM (019), and the laya lanes — but not against the
compile-a-classifier product category itself. The lane's job: measure that category on
the same 15-suite harness instead of quoting their two-word accuracy tier table
("Standard: Higher / Compact: Lower").

Precedent shapes to follow (both measurement-only, their-stack-serves / our-Rust-measures):

- **029 GLiNER lane** — one subprocess per suite, line protocol `{"state", "questions":[{qid, def}]}` → `{"answers":{qid:{p, conf, choice}}}`, parity law for state rendering, refusals loud.
- **027 AgentJev lane** — HTTP oracle (`POST /api/evaluate`), per-question latency unit, adapter divergences documented, over-length input refusal fails the lane loudly.

PAW's difference from both: **programs must be COMPILED per task first** (server-side
finetune — minutes, API-keyed), and the answer surface is **free-form text** (their own
docs: callers "must validate the returned result"), not a probability vector.

## Proposed shape

**Posture A (hosted REST — cheapest first):**
- Compile once per suite: `POST /api/v1/compile` (sync allows 2,400 s) or
  `compile_async` + status polling, with a hand-authored NL spec naming the exact
  option set ("Return only 'credit_card', 'billing', ..."). Cache `program_id` per
  (suite, spec-BLAKE3) under the harness data dir so re-runs never recompile.
- Infer per question: `POST /api/v1/infer` `{program_id, input}` → `{output}`,
  `X-API-Key` from env (`PAW_API_KEY`). No key = lane SKIPS loudly (the corpus_db
  NDB_BIN skip posture), never silently.

**Posture B (local runtime subprocess):** `pip programasweights` in a venv (the
`.raw/gliner-env` precedent), one process per suite speaking the laya-python line
protocol, `temperature=0` fixed. No API key needed; heavier setup (base GGUF download
594/134 MB).

**Mapping law (the loud-refusal discipline, both postures):**
- choice → exact-match the stripped output against the option key/label set;
  unparseable/ambiguous → recorded refusal, counted per suite, NEVER guessed
  (the gliner lane law).
- noul → compile a boolean spec; `[p_no, p_yes]` is NOT available from PAW — the lane
  reports parsed-choice accuracy + refusal rate only, no confidence/ECE columns
  (disclosed divergence, the clm-lane noul law style).
- score → compile a spec naming the level set verbatim; levels matched as exact strings.
- Spec texts are committed artifacts (one per suite, in the lane's scripts dir) so the
  compile confound is reproducible and reviewable.

**Measurement honesty:**
- Accuracy is the claim; latency is quotable only with `scripts/bench_preflight.sh`
  PROVENANCE (box-state law) — and hosted-posture latency includes their network.
- Hosted inference is not deterministic across calls (server-side batching/model
  updates unstated) → accuracy runs recorded per run-date, no bit-identity expectation;
  the local greedy posture is the determinism-comparable cell.
- Published PAW numbers are marketing tiers; our table is the measurement. Disclose
  program sizes + compile wall-time per suite (that's part of the product comparison).

## Tasks

- [x] Author per-suite PAW specs (start subset: banking77, ag_news, emotion, sst5 —
      banking77 77-way is the stress cell) + commit them with the lane. — `2e9f351`,
      `scripts/paw_specs/<suite>.txt`: zero-shot (task line + the option set verbatim, one
      per line); a drift guard refuses a spec that stops naming a served option.
- [x] Posture A client (compile + infer REST, key from env, ~~loud skip without key~~),
      program-id cache keyed (suite, ~~spec-BLAKE3~~ compiler, spec-BLAKE3). — `2e9f351`,
      `src/lanes/paw.rs`: sync compile + `compile/async` polling (`PAW_COMPILE_ASYNC=1` +
      `PAW_COMPILER`), `curl` subprocess transport (hosted PAW is HTTPS-only, no TLS client
      in-tree, no new dep — BOUNDARY), key via a 0600 header file never argv, cache
      `.raw/paw/programs.json` (malformed = loud). The key is OPTIONAL — see Findings.
- [x] Lane wiring in `src/harness/runner.rs` (`--paw` flag, no feature gate — the
      gliner precedent) + `src/lanes/paw.rs`. — `2e9f351` (module rides `modelless` for
      the in-tree blake3; native-only; runner edits are thin: flag, `SuiteResult.paw`,
      meta line, table row + refusal detail line).
- [x] Mapping law implementation + refusal accounting in the tables; no confidence
      columns (disclosed divergence). — `2e9f351` (law in Findings; 9 in-module tests incl.
      the stub-HTTP wire pin: compile + infer shapes, cache hit = zero recompiles,
      refusal accounting, anonymous vs authenticated posture).
- [x] First cells + `.benchmarks/` record — **Bench 049** (hosted anonymous, compiler
      `paw-4b-qwen3-0.6b-20260407`, accuracy-only: the box was not latency-quotable).
- [ ] `paw-ft-bs48` finetune-compiler cells (the "much higher accuracy" tier; async path
      implemented, anonymous tier allows it — just compile budget + ~2–5 min/suite).
- [ ] Posture B lane: local runtime as a Python subprocess oracle (gliner precedent),
      full-N deterministic cells — feasibility MEASURED keyless (Findings).
- [ ] Arena republish via `../reflex-site/scripts/publish_bench.py` lane-update merge.
- [ ] Close into HISTORY.md with hashes; record which posture the published cells used.

## Findings (2026-09-26, `2e9f351` + Bench 049)

- **The key is NOT required — the issue's "no key = SKIP" premise was wrong.** Their SDK
  AGENTS.md: "Sign in for higher rate limits and program naming. Everything works without
  it." Measured: anonymous compile + infer both work (anonymous: 20 compiles/h, 1
  concurrent; infer window limit 10000). One catch, measured: an anonymous compile with
  `public: false` → HTTP 401 `auth_required` — **anonymous programs must be public**. The
  lane therefore runs anonymous (public) when `PAW_API_KEY` is unset, printed loud and
  stamped `hosted-anonymous` in the row; with a key it sends `X-API-Key` and compiles
  private. The owner key question is now only about rate limits / private programs, not
  about whether cells can exist.
- **Other SDK-vs-issue corrections:** base URL env is `PAW_API_URL` (SDK name); sync
  compile answers HTTP 202 with `status: "ready"` inline; the default mapper compiler
  compiles in ~4 s (not minutes) — only `paw-ft-*` finetunes take minutes (async only).
- **Mapping law as landed:** trim → strip ONE matched surrounding quote pair → trim → exact
  case-sensitive key match, else unique exact description match; else a REFUSAL (counted,
  scored wrong, never guessed). The quote strip exists because compiled pseudo-programs
  emit `"neutral"` with literal quotes (seen on train rows before any test cell) and is
  counted per suite (`quote_stripped`).
- **Cells (Bench 049, accuracy with refusal = wrong):** ag_news 0.7825 (0 refusals) ·
  emotion 0.4750 (2) · sst5 0.3283 (0; det ✗ — hosted is not repeat-stable) · banking77
  **0.1400 with 365/500 = 73% refusals** (answers in its own vocabulary: "card status",
  "track new card"…; answered-acc 0.5185). The refusal-dominance open question below is
  answered: yes, publish — the refusal column is the banking77 finding.
- **Posture B keyless: FEASIBLE, measured.** `uv`-installed `programasweights==0.4.10`, no
  key: `paw.function(<public program id>)` downloads the program + the 594 MB base
  (1002 s on this link) and runs locally; 5/5 sst5 test rows repeat byte-identically and
  agree with hosted at the decision level (one surface diff: local `neutral` vs hosted
  `"neutral"` — same label under the law). Hub programs were NOT used: our own compiled
  programs match our label sets exactly by construction (the drift guard), which a
  community hub program cannot promise.

## Open questions

- Owner: is a `PAW_API_KEY` provisioned for bench use (paid/authenticated rate limits)?
  Posture B avoids the key entirely at setup cost.
  → **Narrowed 2026-09-26:** cells exist without one (anonymous tier, Findings). A key
  buys only 60 vs 20 compiles/h, 2 concurrent, and private (unlisted) programs — still
  owner-gated, no longer blocking.
- Suite priority: full 15 or the 4-suite subset first? (Compile cost is per-suite,
  one-time, but server-side finetune latency is unknown until first run.)
- If PAW's accuracy is dominated by refusal/parse-failure on multi-class suites, do we
  still publish? YES — the honest table IS the finding (the 029 posture: beats laya
  base on 9/15, loses classic NLU, published as measured).
