# Issue 108 — the CEILING_CONTRACT_V2 re-judge cell (both seats, bmo)

**Status:** CLOSED — CELL LANDED, κ re-price **FAIL at the unchanged 0.70
bar: κ = 0.4496** (V1 was 0.6387). Recorded negative per the contract's
pre-registered V2 verdict: the LOCAL seat ladder cannot price sst5 gold at
the agreement bar; the only remaining rung is a HOSTED seat (an owner act).
Both V2 ĝs stand as judge-conditioned readings only.

Run 2026-11-06 on bmo (Mac17,15, 96 GB, 30 cores, load ~1.3 at start,
~2.7 during the Bonsai pass; AC). The rig queue saw bmo OFFLINE (its
Tailscale link was down), so the run went over amo's Thunderbolt bridge
(`192.168.10.2`) as a plain `nohup` job — not via `agent-mail rig run`.
The box had no other heavy jobs, and this cell measures CORRECTNESS, not
latency, so the box state can't move the verdict. Harness `fdb3913`
(contract V2, committed + pushed BEFORE the first V2 judge call), separate
worktree `~/git/riir-reflex-108v2`, `.raw` symlinked from the bmo
checkout. Posture identical to the V1 cells: the whole 600-item sst5 front,
40 twin controls, default seed.

## Seats

| seat | server | flags | wall |
|---|---|---|---|
| Qwen3-8B Q8_0 (fallback / cross-family) | upstream llama.cpp `45b6604` (`~/scratch/llama-up`) :8082 | `--jinja --reasoning off` | ~1 min |
| Bonsai-27B PQ2_0 (primary) | prismml fork (`~/git/prismml-llama.cpp`) :8083 | `--jinja --no-reasoning-preserve` | ~41 min (331 calls, median 5.1 s, max 16.1 s) |

Both seats ran `-c 8192 -ngl 99 --parallel 1`, `max_tokens` 1024 (V2), and temperature 0. Teardown was clean (no `llama-server` orphans after either seat).

## Results

| read | Bonsai (A) | Qwen3-8B (B) |
|---|---|---|
| validity: flipped catch | **40/40 = 1.000 PASS** | **39/40 = 0.975 PASS** |
| supported-half catch | 23/40 (not_supported 7 · unknown 10) | 27/40 (not_supported 12 · unknown 1) |
| **ĝ** (Wilson 95%) | **0.7379** [0.6738, 0.7932] | **0.6911** [0.6307, 0.7455] |
| half-width / Kish deff | 0.0597 / 1.0602 | 0.0574 / 1.0458 |
| adjudicated / unknown | 206 / 45 (**all 45 seat-starved**, finish=length) | 246 / 5 |
| ceiling-adjusted (JOIN) | 0.2171 over n 152 | 0.2000 over n 170 |
| correct ∧ not_supported | 13 | 16 |

Lane half bit-exact again: thresholds score 0.20488091 / distance
0.41663086 at ρ 0.3 (same as both V1 cells). Coverage ceiling 0.4183, raw
lane accuracy 0.0867 (the headline) on both.

## The κ re-price (the pre-registered bar)

`../108_ceiling_cross_family/ceiling_kappa.py ceiling_probe_bonsai.json ceiling_probe_qwen3-8b.json`:

| read | V1 | **V2** | bar |
|---|---|---|---|
| sample alignment | 600/600 | 600/600, 0 unmatched | — |
| both-adjudicated pairs | 135 | **201** | — |
| raw agreement | 0.8370 | **0.7811** (157/201) | — |
| κ three-category | 0.6387 | **0.4496** | 0.70 → **FAIL** |
| κ decisive-only | 0.6387 | **0.4496** (no unclears) | 0.70 → **FAIL** |

Confusion (rows A, cols B): ns/ns 33 · ns/sup 20 · sup/ns 24 · sup/sup 124.

## Why it failed (measured, not guessed)

1. **The grade ruling did not take on Qwen3-8B's very-positive line.**
   The largest V2 disagreement class is *very positive, Bonsai accepts,
   Qwen3-8B rejects*: **18/44** (the V1 S-GRADE-THRESHOLD pole, now
   one-sided). Qwen3-8B still rejects very-X grades without an explicit
   superlative, even though the prompt ruling says it should not.
2. **Qwen3-8B is PROMPT-SENSITIVE, Bonsai is STABLE.** From V1 to V2,
   over the items both runs adjudicated, Qwen3-8B changed **40/246**
   readings (22 ns→sup, 17 sup→ns, 1 → unclear), while Bonsai changed
   **7/117**. The rulings moved the fallback seat in both directions
   rather than pulling it onto Bonsai's line. Agreement on the 22
   V1-disagreement items rose to 8/22, but 22 new disagreements appeared
   on items that agreed under V1 (11 of the 22 V2 disagreements that
   both V1 seats adjudicated had agreed under V1).
3. **V1's κ was measured on an easier slice.** V1's both-adjudicated
   set was 135 because Bonsai's 116 V1 unknowns (quote failures and
   malformed replies) removed items. V2's 1024-token budget eliminated
   the quote-failure class (45 unknowns, all seat-starved, none
   quote-failed) and widened the set to 201. Plausible but unproven:
   the items V1 dropped are the harder boundary items, which would mean
   the V1 0.6387 overstated agreement.
4. **The seat-starved class persists at 1024 tokens**: 45/251 judged
   items (about 18%) still return empty content at finish=length. They
   spread across every grade (negative 14, neutral 11, very negative 10,
   positive 6, very positive 4). The Bonsai think channel is unbounded
   on this prompt family, and a bigger budget only buys time.

## Verdict (per the contract's pre-registered V2 clause)

**FAIL → recorded negative.** sst5 gold cannot be priced at κ ≥ 0.70 by
the local seat ladder (Bonsai PQ2_0 + Qwen3-8B Q8_0) under V1 or V2. The
substantive reading is still stable across seats and contracts: **ĝ sits
around 0.63–0.74**, so about a quarter to a third of sst5's adjudicated
gold is not defensible as typed. Every ĝ remains a judge-conditioned
reading (calibration-identity law), never a ceiling of record. The bar
does not move. The next rung is a hosted seat (an owner act: keys + ToS),
or a third local family as a tiebreak. Neither is opened here; both need
a V3 contract edit first.

The lane-score consequence is unchanged: raw accuracy stays the headline,
and the ceiling-adjusted JOIN (0.20–0.22 vs raw 0.0867) rides beside it
as a seat-conditioned disclosure only.

## Artifacts

- `ceiling_probe_{bonsai,qwen3-8b}.json` + `CEILING_PROBE_{bonsai,qwen3-8b}.md` — verbatim harness output (each item now carries `finish_reason`).
- κ: the committed `../108_ceiling_cross_family/ceiling_kappa.py` (unchanged).
