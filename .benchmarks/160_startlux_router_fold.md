# Bench 160 — riir-train 635 T4 router fold: the StartLux column adds ZERO routed lift on our suites today; the one lift (emotion +14.1 pp) is blocked by two named instrument gaps

**Status:** RECORD — fold executed + root-caused 2026-10-25 (4090 idle-lane session). The pairwise router run (`--teacher-router .raw/startlux_teacher,.raw/distill_teacher_d3 --router-out .raw/router_startlux_d3`, harness release at develop `53cf6f5`) joined 3 suites and honestly refused 4; this record carries the refusals' root causes (verified, not guessed), the per-suite lift assessment, and the unblock recipe. **Verdict: no promotion, no router change, no recapture spend — the modelless-first gate (the router must show a lift before more instrument work) does not fire today.**

## 1. What joined (the clean half)

| suite | sources (train acc) | gate g | routed acc | cv (loo) | vs the 067 column set |
|---|---|---:|---:|---:|---|
| banking77 | startlux 0.9207 ⊕ d3 0.9090 | 0.7633 | 0.9222 | 0.9222 | **clef holds 0.9492** — no lift |
| massive_intent_en | startlux 0.9510 ⊕ d3 0.9375 | 0.7949 | 0.9511 | 0.9511 | **clef holds 0.9685** — no lift |
| ag_news | startlux 0.9061 (passthrough) | — | 0.9061 | — | **laya-english holds 0.9769** (bench 159) — no lift |

## 2. What refused, root-caused (the expensive half)

Four suites refused the join for TWO orthogonal, verified reasons:

**(a) The classes-spelling era split.** The startlux capture (10-22, sha `ad2bb03`) ran at a tree carrying the 623 StudentUniverse fix (`b0ec691`): its bins carry documented `classes_spelling` maps and STUDENT-spelled classes arrays — emotion `['0'..'5']` (map: sadness→0, joy→1, love→2, anger→3, fear→4, surprise→5), xnli `['0','1','2']` (entailment→0, neutral→1, contradiction→2), injections `['no','yes']` (0→no, 1→yes). The d3/pplx/clef captures (10-14 era) PREDATE the fix: corpus (name) spelling, no field. The router joins on classes-array equality and has no `classes_spelling` consumer → emotion/xnli/injections refuse on the spelling, not on the data.

**(b) The datasets-vs-datasets_t20k row-posture shift.** The startlux column was captured over `.raw/datasets`; d3/pplx/clef over `.raw/datasets_t20k`. emotion: t20k inserts one duplicate row at position 13846 (both postures' row SETS are equal; 15999 vs 16000) → everything after 13846 shifts by one. sst5: t20k has 11 extra survivors (8544 vs 8533). The router's positional gold-pin join correctly refuses both.

**The verification that makes this a finding and not a shrug** (all read-only, over the sealed bins):
- startlux golds vs `.raw/datasets` labels: emotion 15999/15999, sst5 8533/8533, xnli 20000/20000 — **0 mismatches**; massive gold sequence == d3's exactly.
- After the bins' OWN documented `classes_spelling` remap: **xnli golds join 20000/20000 positionally** (startlux acc 0.9073 in d3 class space — matches distill.json exactly); injections join 546/546 under the 0→no/1→yes map.
- After remap + a TEXT-join walk over the shift: **emotion golds join 15999/15999** (0 mismatches) — the data is row-aligned in content; only the instrument cannot express the alignment.

## 3. Per-suite lift assessment (the verdict table)

| suite | startlux | best current column | verdict |
|---|---:|---:|---|
| banking77 | 0.9207 | clef 0.9492 | no lift (routed 0.9222 stays under) |
| massive_intent_en | 0.9510 | clef 0.9685 | no lift (routed 0.9511 stays under) |
| ag_news | 0.9061 | laya 0.9769 | no lift |
| xnli_en | 0.9073 | d3 0.9166 / pplx 0.9207 / laya 0.9654 | no lift (joinable after (a) alone) |
| prompt_injections | 0.9835 | d3 0.9890 | no lift (joinable after (a) alone) |
| sst5 | 0.5998 | clef 0.5988 | marginal (gate ≈0.55); blocked by (b) |
| **emotion** | **0.7712** | pplx 0.6301 / d3 0.6246 | **+14.1 pp — THE lift; blocked by (a)+(b)** |

**The StartLux column adds zero routed lift on our suites today.** The emotion cell is the only value, and claiming it needs BOTH: (1) the router consuming `classes_spelling` (a clean, well-scoped feature — the map is in-bin, injective, and already gold-verified by this record), AND (2) emotion/sst5 row alignment — either a datasets_t20k recapture of those two suites (~55 + ~29 min GPU at the documented boot line; note a recapture does NOT fix (a) — StudentUniverse is the standing 623 law, the spelling map is by design) or an offset/text-join in the router (a bigger instrument change, declined for now — a positional join that silently spans a row shift is exactly the mislabel class the gold pins exist to catch).

## 4. The 067 half of T4 — unchanged, blocked elsewhere

The pinned-snapshot 067 routed-oracle re-read needs the JDI public-suite capture dirs, which are not on this box (bench 159's caveat stands). Nothing in this fold moves it.

## 5. NC-lineage (the 635 law)

Every routed/passthrough bin in `.raw/router_startlux_d3/` blends or copies NC-derived columns (the taint composes per the 635 T0 adjudication): **NC-lineage: startlux-decision-v1.1 (CC BY-NC 4.0) — measurement/internal use only, adoption fail-closed.** Nothing here feeds a serving path.

## Provenance

- Router run: harness release binary at develop `53cf6f5`, built 2026-10-25 (release, default features); output `.raw/router_startlux_d3/{router.json,ROUTER.md,*_teacher.bin+blake3}` (untracked, regenerable — the command is one line).
- Capture dirs: `.raw/startlux_teacher/` (10-22, sha `ad2bb03`, `MANIFEST.md` binding) × `.raw/distill_teacher_d3/` (10-14); the pplx_v2/clef dirs hold the same-era corpus-spelled bins (spot-verified: pplx emotion 16000 name-spelled classes, no spelling field).
- Verification reads: `py` one-offs over the RIDT layout (magic `RIDT`, u32 ver, u32 header_len, header JSON, 8+4·C records) — gold sequences, classes arrays, `classes_spelling` maps, and the text-join; no bin modified.
- Session: zed-4090-idle-1025 (2026-10-25 +07; zero GPU, CPU-only fold).
