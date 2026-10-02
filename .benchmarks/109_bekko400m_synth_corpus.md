# Bench 109 — the bekko-400M synth-corpus read (riir-train Issue 609 stretch): V5 PASS (0.7800 → 0.8100, paired LB95 +0.0086) but BELOW the openthai incumbent (0.8133 / +0.0110) — no adoption

**Status:** MEASURED 2026-10-02 — Issue 609's pre-registered stretch, executed now
that T2/T3 landed (the issue's own deferral wording: "defers until T2/T3 land").
Same generation law as Bench 104's Path 2, ONLY the teacher differs (the 68M →
the 400M via T1's pin flip). The corpus axis is confirmed ALIVE and slightly
teacher-sensitive — and still not won by bekko. The incumbent openthai-veto
corpus stays; nothing serves changes.

## The run (the 104 commands, re-issued at the 400M pins)

```
BEKKO_PYTHON=$PWD/.raw/bekko-env/bin/python \
target/release/harness --synth-corpus --synth-teacher bekko \
  --suites massive_intent_en --datasets-dir .raw/datasets_t20k \
  --synth-out .raw/corpus_synth_bekko400m
target/release/harness --corpus-ab .raw/corpus_synth_bekko400m/massive_intent_en_synth.jsonl \
  --datasets-dir .raw/datasets_t20k --synth-out .raw/corpus_synth_bekko400m
```

No `BEKKO_MODEL` override — the teacher ran at T1's default
(`hotchpotch/bekko-system-one-v0-400m` @ `4aeb85b9…`), the semantic delta of
Issue 609 T1.

## The read

| corpus (the veto teacher) | modelless A0 gold→synth | paired LB95 | V5 | vs incumbent |
|---|---|---|---|---|
| openthai (the incumbent, Bench 0029-era) | 0.7800 → **0.8133** | +0.0110 | PASS | — |
| bekko-68M (Bench 104) | 0.7800 → 0.8067 | +0.0062 | PASS | −0.7 pt, LB95 smaller |
| **bekko-400M (this bench)** | 0.7800 → **0.8100** | **+0.0086** | **PASS** | **−0.3 pt, LB95 smaller** |

- Generation: 212,912 candidates → **2048 accepted** (the full budget; the 68M
  accepted 1975) / 1652 vetoed / 3700 forwards / 1990 s M3 CPU (the 68M's was 455 s —
  the 400M is ~4.4× per forward; p50 526 ms under load). Corpus blake3
  `cb94fbbc01b93c62…`; per-label caps ≤128 held; span ≤4; same per-intent E0
  weighting (dedup pool 65 / cal 27 / dup 4598 / out 0).
- The A/B: gold 11,314 docs → synth 13,362; arm A reproduces the published 0.7800
  EXACTLY (the aliveness anchor); flips 10 synth-only wins vs 1 gold-only;
  determinism true; latency gold p50 112 µs / synth p50 118 µs (V6 sub-ms holds).

## The verdict

**No adoption.** The bekko-400M corpus lifts the modelless row and beats the 68M's
corpus on the same axis (+0.3 pt) — the teacher prior DID transfer a little here,
unlike the student axis (instinct Bench 0056: the T2 gates MISS) — but it does not
beat the incumbent openthai corpus (0.8100 < 0.8133, LB95 +0.0086 < +0.0110). Bench
104's adoption bar stands: a different-teacher veto needs to BEAT 0.8133, not
approach it. The incumbent stays; the corpus lever on this cell is openthai's.

With this, **Issue 609's three paths are all measured at the 400M and all three
return negatives on their adoption gates** — the lane re-closes (the honest-outcome
rule). The landed assets are unchanged from 104: the pluggable teacher seam +
attribution machinery, now defaulting at the 400M pins.

## Box state

Preflight REFUSED at the session's launch (`load 8.21 > MAX_LOAD 6.0` — sibling
sessions); the synth + A/B ran under moderate load (the box spiked to load 33
mid-day from a sibling's batch, recovered to ~5–9). The accuracy/LB95 figures are
deterministic integer arithmetic on frozen picks — box-independent. The 1990 s
generation wall and the 526 ms p50 are load-affected, quoted as magnitude only.

## Artifacts

- `.raw/corpus_synth_bekko400m/` — `massive_intent_en_synth.jsonl` + SYNTH.md +
  CORPUS_AB.md + corpus_ab.json + synth_report.json (DATA on disk, gitignored)

## Cross-refs

- riir-train Issue 609 — the re-open this completes (T1/T2/T3/stretch all measured)
- Bench 104 — the Path-2 shape + the 68M corpus negative this replicates at 400M
- instinct Bench 0056 — the T2 gate reads (the same re-open's student axis)
- Bench 107 — the board priors that funded the re-open
