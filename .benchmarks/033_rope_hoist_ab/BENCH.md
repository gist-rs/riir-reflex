# Bench 033 — T10 rung 2 (rope hoist) promotion probe — **VERDICT: NOT PROMOTED, stays opt-in (measured 2026-09-25)**

The probe ran on a preflight-clean window (PROVENANCE at the time of launch:
power=AC powermode=2 settle=449min load=3.20 swap=1206M canary=135.5us/best5).
6 position-balanced rounds × 2 arms × 3 suites, same binary both arms (the
env flag IS the switch). Per-round load trace: rounds 3/4/6 clean (start
loads 2.9 / 2.2 / 3.0); rounds 1/2/5 rode load spikes to 6.5–6.7 (sibling
activity restarting) and are DISCARDED per the T5 confound lesson — kept in
the table below, marked, never averaged in.

## Per-round result (ON/OFF, p50 and p99, ms)

```
          banking77              code_fixtures           massive_intent_en
 r    p50        p99          p50        p99           p50        p99
r1 LOUD  62→78 1.258   72→136 1.889    78→69 0.885  294→328 1.116   37→47 1.270  44→80 1.818
r2 LOUD  63→78 1.238   77→105 1.364    63→93 1.476  286→327 1.143   40→43 1.075  61→59 0.967
r3 clean 67→68 1.015   91→85  0.934    65→65 1.000  305→316 1.036   43→45 1.047  53→59 1.113
r4 clean 70→66 0.943  108→79  0.731    67→64 0.955  321→265 0.826   43→43 1.000  52→59 1.135
r5 LOUD  87→66 0.759  122→80  0.656    64→64 1.000  275→274 0.996   50→44 0.880  74→62 0.838
r6 clean 68→65 0.956   91→78  0.857    64→65 1.016  276→267 0.967   44→42 0.955  59→52 0.881
```

Clean-round medians: banking77 p50 **0.956** (−4.4%, wins r4/r6, r3 +1.5% =
inside the 1-ms quantization at 67–68 ms) · banking77 p99 **0.857** ·
code_fixtures p50 1.000 · code_fixtures p99 **0.967** (−3.3%; r4 −17.4% is
the biggest single reading, r3 +3.6% the only loss, within quantization at
~300 ms ⇒ ±0.3%) · massive_intent p50 1.000, p99 1.113 (−/+, both inside
quantization at 52–59 ms).

## The verdict and why

**NOT PROMOTED — the flag stays opt-in (`LAYA_METAL_ROPE_HOIST=1`).** The
win is real but narrow: it lives in the p99s (banking77 −14.3%,
code_fixtures −3.3%) and the r4 round (−17…−26% across the attention-heavy
cells) more than in the p50 medians (banking77 −4.4% with one
quantization-level loss; two suites flat). Against the promotion bar this
lane actually used — rung 1 landed on **10/10 paired wins at −6…−8%**, rung
3 on **38/40 at −1.7…−3.5%** — rung 2's evidence (3 clean rounds, wins
concentrated in one round, medians −4.4% on ONE suite) is thinner than both.
No regression either (nothing consistently loses), so the rung is not
reverted: it stays available behind the flag for the attention-heavy p99
profile, and the door stays open — a rerun on a longer clean window, or the
f16-B GEMM rung moving the GEMM share down (making attention the dominant
term), can re-price it. The 09-24 issue tick is updated accordingly.

## Rung summary

- **Rung:** riir-infer `5ef7442` — `attn_rope` derives the Q/K rope once per
  layer into a packed `[2, seq, d]` device scratch; `flash_attn`'s staging
  copies instead of re-rotating per (query block × head × key tile).
  DEFAULT-OFF behind `LAYA_METAL_ROPE_HOIST=1`; gates green at both
  postures at landing (reflex `.issues/020_riir_metal_latency_parity.md`,
  rung 2 tick).
- **Arm switch:** the env flag IS the A/B — same binary, no rebuild between
  arms. OFF = `LAYA_METAL_ROPE_HOIST=0` (in-kernel rope, shipped behavior),
  ON = `LAYA_METAL_ROPE_HOIST=1` (hoist).
- **Instrument:** `bash .benchmarks/033_rope_hoist_ab/rope_hoist_ab.sh` — 6
  position-balanced rounds × 2 arms × 3 suites
  (`massive_intent_en,banking77,code_fixtures`; code_fixtures carries the
  512-token case where flash_attn dominates), load logged per arm-run to
  `/tmp/rhoistab/rope_ab.idx`.
- **Verdict rule applied:** position-balanced medians per arm; aliased
  rounds DISCARDED (r1/r2/r5), never averaged in. Binary: `/tmp/rhoistab`
  (release + `laya-riir-metal`, built from clean develop).
