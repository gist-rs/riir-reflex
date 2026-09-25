# Issue 033 — PAW comparison lane: ProgramAsWeights as the third external classifier oracle

**Status:** OPEN — filed 2026-09-25 from `.research/004_ProgramAsWeights_Serving_Landscape.md` (PAW distill, sdk @ `74919f6958b127f10776689277f5a74321857b40`). Nothing landed yet.

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

- [ ] Author per-suite PAW specs (start subset: banking77, ag_news, emotion, sst5 —
      banking77 77-way is the stress cell) + commit them with the lane.
- [ ] Posture A client (compile + infer REST, key from env, loud skip without key),
      program-id cache keyed (suite, spec-BLAKE3).
- [ ] Lane wiring in `src/harness/runner.rs` (`--paw` flag, no feature gate — the
      gliner precedent) + `src/lanes/paw.rs` (or scripts-side subprocess helper for
      posture B).
- [ ] Mapping law implementation + refusal accounting in the tables; no confidence
      columns (disclosed divergence).
- [ ] First cells + `.benchmarks/` record; arena republish via
      `../reflex-site/scripts/publish_bench.py` lane-update merge.
- [ ] Close into HISTORY.md with hashes; record which posture the published cells used.

## Open questions

- Owner: is a `PAW_API_KEY` provisioned for bench use (paid/authenticated rate limits)?
  Posture B avoids the key entirely at setup cost.
- Suite priority: full 15 or the 4-suite subset first? (Compile cost is per-suite,
  one-time, but server-side finetune latency is unknown until first run.)
- If PAW's accuracy is dominated by refusal/parse-failure on multi-class suites, do we
  still publish? YES — the honest table IS the finding (the 029 posture: beats laya
  base on 9/15, loses classic NLU, published as measured).
