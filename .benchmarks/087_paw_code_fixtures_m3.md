# Bench 087 — PAW on `code_fixtures`: the per-question-shape lane extension + the first cell

**Status:** MEASURED 2026-09-29 on the M3 Max — the PAW comparison lane's
first multi-shape suite. Until this bench the lane had run only the four
single-shape suites (ag_news / emotion / sst5 / banking77; benches 049,
054–058); `code_fixtures` was missing from the PAW board on EVERY host —
the last comparison lane without a `code_fixtures` cell (openthai 084/086,
gliner 037, agentjev 039 all carry one).

## Why the lane needed a change first (the structural gap)

`code_fixtures` serves TWO question kinds per case (`module` Choice over
the 8 frozen module labels + `is_pub` Noul) on ONE input (the function
source). The lane's law was one compiled program per suite answering every
question with the same single-input call — one program cannot answer two
different questions on the same text, so the second shape would have been
a structural refusal factory (~50% accuracy ceiling, nonsense half).

The extension (`src/lanes/paw.rs`, this commit): one program per DISTINCT
question shape. Spec resolution is `{suite}.{qid}.txt` first, the
suite-wide `{suite}.txt` as the fallback (the single-shape suites keep
byte-identical behavior AND the historical cache key
`(suite, compiler, BLAKE3(spec))`; a per-shape program keys
`(suite, qid, compiler, BLAKE3(spec))`). A partially-specced suite is a
LOUD error naming the missing file — never a partial row. Multi-shape rows
disclose every program (ids / spec paths / digests comma-joined in shape
order; compile wall summed; cache-hit true only when every shape hit).
Pinned by three new stub-server tests (two-shape compiles one program per
shape with per-shape cache keys; suite-wide fallback stays one program
with the old key; partial coverage is loud, full absence stays the honest
`Ok(None)`) + the committed-spec drift guard extended to both
`code_fixtures.*.txt` files. 20/20 paw tests, 217/217 lib, clippy `-D`
clean.

## Provenance

- **Box state: ⛔ latency NOT QUOTABLE — accuracy-only record** (the 049
  posture: hosted-PAW latency is network-dominated and is never engine
  latency). Preflight REFUSED (load 13.47 > 6, sibling sessions — the
  riir-infer 022 quant-arm + riir-clippy mining runs); harness box_state
  load 16.28 → 15.55, `latency_quotable: false` BOTH spans, AC, powermode
  high. Cite `server p50` for their stack's own latency; the client
  round-trip columns are observation only.
- **Code:** reflex at `bb86069` + the UNCOMMITTED per-shape lane patch
  (this commit carries it — the run measured exactly the code that ships).
  `--release`, default features (modelless), `--skip-laya`.
- **PAW side:** hosted API, anonymous posture (no key; both programs are
  PUBLIC on their hub), compiler `paw-ft-bs48-20260530` (the lane's
  canonical finetune compiler since 054–058), async compile
  (167.8 s module + 78.3 s is_pub = 246.1 s), temperature 0, max_tokens 48.
  Programs: module `e5c9b00b49fb144810e4`, is_pub `1788474fc4e77eb38e3f`.
  Specs (`scripts/paw_specs/code_fixtures.{module,is_pub}.txt`) authored
  once, zero-shot, frozen before the first cell (the 049 discipline).
- The suite: 16 cases / 32 questions (8 frozen modules × 2 fn spans × 2
  questions), population BLAKE3-pinned (issue 044 T4) — identical bytes
  across hosts and runs.

## Cells (accuracy = correct / every served question; a refusal counts wrong)

| lane · model | n | acc | answered-acc | refusals | det | modelless same-run | reference rows |
|---|---|---|---|---|---|---|---|
| paw · paw-ft-bs48-20260530 (M3, hosted-anonymous) | 32 | **0.6250** | 0.6452 | 1/32 (3.1%) | ✓ | 0.3750 | gliner 0.6667 (037, 4090) · laya·en 0.5833 (037, 4090) · agentjev 0.3750 (039, 4090) · openthai 0.5938 (084/086, both hosts) |

- **The finetune compiler's shape transfers to a fifth domain.** PAW-ft
  lands SECOND on the suite (0.6250), above the laya english checkpoint
  (0.5833) and the openthai teacher (0.5938), below gliner (0.6667) —
  the same "between laya and gliner" band the ft compiler held on
  ag_news (0.79 vs laya 0.95 vs gliner 0.70: there it SPLIT the pair)
  and emotion (0.50 vs 0.59 / 0.57). The refusal column stays the
  finding: 1/32 (3.1%) — small-label-set discipline, exactly the 049
  law (the one refusal is the program answering `harness::families`, a
  plausible-but-nonexistent module name — the never-guess law refused
  it rather than fuzzy-matching into `harness::suites`).
- **Determinism ✓** (observed-repeat, first 10 questions) — the ft-bs48
  hosted posture stays deterministic (058's reading, reproduced), which
  the zero-shot posture's sst5 row (049) was not.
- **Modelless drift pin:** 0.3750 byte-identical to the published 044/086
  cell — the harness measured the same suite the other lanes did.
- No promotion claim — comparison lane, report-only (the 074 law).

## The lane's `code_fixtures` board, complete

| model | code_fixtures | host | record |
|---|---|---|---|
| gliner | **0.6667** | 4090 | 037 |
| paw-ft-bs48 | **0.6250** | M3 (hosted — host-independent) | 087 |
| openthai | 0.5938 | both hosts (byte-exact) | 084 + 086 |
| laya · english | 0.5833 / 0.4062 | 4090 / M3 | 037 / 044 |
| agentjev | 0.3750 | 4090 | 039 |
| modelless | 0.3750 | everywhere (drift pin) | 044 |

(The laya M3/4090 spread on this suite predates the frozen population —
044's M3 cell read the commit-relative harvest, 037's 4090 cell the
12-case pre-freeze shape; the current suite is the 16-case frozen board
both lane runs measure going forward. The PAW cell is the first measured
on the frozen population beside modelless/openthai/gliner.)

## What remains

- paw × typed_decisions / prompt_injections / massive / xnli / harness
  rows: the per-shape extension makes these REACHABLE (typed is a
  THREE-shape suite — Choice + Noul + Score), but each needs a committed
  spec + the anonymous compile budget; not asked for here, filed as the
  lane's standing coverage note (this record is the vehicle).
- A quotable-box re-run is owed ONLY if anyone ever wants to cite the
  client round-trip — the lane's own law says that column should never
  be cited as engine latency (server p50 is the PAW-side figure and is
  box-independent), so this record claims accuracy only and stands.

## Artifacts

- `087_paw_code_fixtures_m3/TABLES.md` + `results.json`
