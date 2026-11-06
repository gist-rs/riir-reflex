# Issue 108 — the cross-family judge cell (Qwen3-8B fallback seat, bmo)

**Status:** CLOSED — CELL LANDED, κ RECHECK **FAIL at the 0.70 bar** (the
pre-registered localize→rulings→re-judge→re-price arc opens, see §Verdict).
Run: bmo (m5ultra, 96 GB, quiet/exclusive via the rig queue) 2026-11-06,
`agent-mail rig run` job `01a1ae5b`; harness `775b882`, contract
`CEILING_CONTRACT_V1`, posture identical to the first cell (sample 600 =
whole sst5 front, controls 40, default seed).

## Seat (the ladder's fallback arm, proven on-box)

- **Judge seat:** Qwen3-8B — `Qwen/Qwen3-8B-GGUF` **Q8_0** single file
  (8,709,518,112 B; sha256 `408b955510e196121c1c375201744783b5c9a43c7956d73fc78df54c66e883d6`,
  `qwen3.sha256` here) served by **llama.cpp upstream `45b6604`** built from
  source on bmo (cmake 4.4.4, Metal, `-ngl 99`, `-c 8192`, `--parallel 1`,
  `--alias qwen3-8b`), loopback `127.0.0.1:8082`.
- **Thinking is OFF**: `--jinja --reasoning off` (upstream's spelling of
  `enable_thinking=false`; the seat-ops lesson from the 4090 Bonsai boot —
  a reasoning channel starves `content` inside `max_tokens 256` — applied
  at the template, upstream master has the first-class flag). Sanity probe:
  clean JSON verdict, no `<think>` in content, 18 completion tokens.
- **Seat cost, disclosed:** unknown rate **2/600 = 0.33%** (first cell,
  Bonsai PQ2_0: 116/600 = 19.3% — quote-verification failures + malformed
  verdicts). The Qwen3-8B Q8_0 seat's format discipline is near-perfect
  under the verbatim-quote law; the Bonsai seat's unknowns were the seat,
  not the gold.
- First boot attempt (job `01a1ae58`) died at the SCRIPT's sanity grep
  (the wire body carries `\"verdict\"`; the grep pattern `"verdict"` never
  matches across the backslash — script bug, not a seat bug; the seat's
  reply was already clean). Fixed in-place on the scratch script; the
  landed run is `01a1ae5b`. No orphans either way (teardown verified).

## Results (mirrors the first cell's posture, read once)

- **Validity gate PASS** — flipped catch **38/40 = 0.9500** (bar 0.8500);
  supported-half catch **28/40 = 0.7000**, split carried (`17e82e0`):
  not_supported 12 (true gold-rejections — the ceiling's own signal) ·
  unclear 0 · unknown 0 (seat cost, never folded).
- **ĝ = 0.6747** (Wilson 95% [0.6142, 0.7299], half-width 0.0578 — the
  contract's ≤ 0.05 bar is MISSED, like the first cell's 0.0804; whole-front
  sst5 is the population, the width is what it is) · Kish deff 1.0464
  (n_eff 237).
- Fate census (n 600): correct supported 30 · correct NOT-supported 21 ·
  wrong supported 138 · wrong not-supported 60 · unclear 0 · abstained 349
  · unknown 2. Coverage ceiling 0.4183 · raw lane accuracy 0.0867 (the
  headline) · ceiling-adjusted 0.1786 over n 168.
- **The lane half reproduced bit-exact across boxes** — thresholds
  score 0.20488091 / distance 0.41663086 at ρ 0.3 on both runs (4090 first
  cell vs this bmo run), and the 600 (case_id, gold_label) sample keys
  align 600/600 with ZERO unmatched — the probe is a deterministic
  instrument, and the only moving part between the cells is the judge seat.
  That is exactly what a cross-family cell needs.

## The κ recheck (the cell's purpose)

Joined on (case_id, gold_label) over the items BOTH seats adjudicated
(`.raw/ceiling_kappa.py`, committed beside this record):

| read | value | bar |
|---|---|---|
| both-adjudicated pairs | **135** (of 600 shared; B adjudicated 249, A 135-eligible after its 116 unknowns) | — |
| raw agreement | **0.8370** (113/135) | — |
| **κ three-category** | **0.6387** (jackknife SE 0.0061, 95% CI [0.6268, 0.6506]) | **0.70 → FAIL** |
| κ decisive-only | 0.6387 over n=135 (both seats read unclear = 0 — identical to three-category) | **0.70 → FAIL** |
| disagreement | 22 items: A=not_supported/B=supported **15**, A=supported/B=not_supported **7** (direction p≈0.13 — not decisive) | — |
| supported share, both-adjudicated set | A 85/135 = 0.6296 [0.5456, 0.7064] · B 93/135 = 0.6889 [0.6065, 0.7608] — intervals overlap | — |

Confusion (rows = A/Bonsai, cols = B/Qwen3-8B): ns/ns 35 · ns/sup 15 ·
sup/ns 7 · sup/sup 78. Qwen3-8B is the MORE LENIENT seat (higher
supported-half catch 0.700 vs 0.275, higher ĝ, 93 vs 85 supported) — the
disagreement mass is Bonsai REJECTING golds Qwen3-8B accepts, and the
rejecting seat is the one with the 19.3% unknown rate, i.e. the seat with
demonstrated format strain.

## Verdict (honest, per the pre-registered arc)

**κ 0.6387 < 0.70 — the cross-family agreement bar FAILS.** The contract's
process is localize → rulings → re-judge → re-price; this record OPENS that
arc and hands it its first evidence:

> **[2026-11-06 arc progress: the LOCALIZE step is DONE —
> `LOCALIZE.md` in this directory.** Headline: 12/22 of the disagreement
> mass is the benchmark's own grade-granularity seam (very-X vs X), 4 are
> Qwen3-8B errors split both directions, 4 genuinely ambiguous, and 3
> expose Bonsai PQ2_0's per-call instability (reasoning channel starves
> content at max_tokens 256 on ~14% of re-reads — the first cell's 116
> unknowns were this class, not gold noise). Both seats' quotes are in
> hand; re-judge/re-price remain contract-edit (owner) steps.]

1. **The two seats' ĝs agree within overlapping CIs** (0.6296 vs 0.6747;
   both [0.55, 0.73]-ish) — the substantive reading "sst5's headroom is
   capped by gold quality, roughly two-thirds supported" is STABLE across
   families even at κ < bar. The failure is an agreement-classification
   failure, not a ceiling-instability fact.
2. **The disagreement mass is seat-sensitivity, not gold noise**: the
   stronger seat on the twins (supported-half catch 0.700 vs 0.275) is the
   more lenient one, and the seat rejecting more gold is the one paying
   19.3% unknowns. A plausible localization (unverified here): Bonsai's
   `not_supported`-heavy tail mixes true rejections with strain-induced
   rejections — exactly the seam class a rulings pass is for.
3. **What the re-price needs** (next arc, not this record): a third-seat
   tiebreak (hosted seats are owner acts) or a rulings pass over the 22
   disagreement items with BOTH seats' quotes in hand. This record does
   NOT re-price ĝ — both cells stand as judge-conditioned readings under
   their own stamps (calibration-identity law), and the harness lane
   scores stay gold-absolute headlines with the ceiling beside them.
4. **The contract V1 κ bar itself is now evidence-bearing**: OntoSmith's
   own ruled contract measured κ ≈ 0.43 and their published bar was their
   95% line; OUR 0.70 was pre-registered above that precedent. 0.6387 with
   CI [0.627, 0.651] vs bar 0.70 is a clean miss, not noise. Any bar move
   is a CONTRACT V2 edit with its own rationale — never a re-read of V1.

## Artifacts here

- `CEILING_PROBE.md` + `ceiling_probe.json` — the bmo run (verbatim harness output).
- `seat_boot.log` — the judge server boot + the 332-call transcript tail (timings).
- `qwen3.sha256` — the model file digest.
- κ computation: `../../.raw/ceiling_kappa.py` (run line inside the file doc).

## Box posture

bmo (Mac17,15, m5ultra-class, 96 GB, 30 cores), exclusive lane via the rig
queue, load ~2-3 throughout, AC power. Judge timing: prompt ~70-250 ms,
generation ~10 ms/token (~100 tok/s), whole probe ≈ 2 min wall.
