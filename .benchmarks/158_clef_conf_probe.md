# Bench 158 — Issue 094 T3: clef headful confidence candidate (last-position entropy vs both floors)

**Status:** COMPLETE — report-only; the entropy candidate reads NEGATIVE on 8/9 suites, and the single letter-of-the-law PASS (banking77) is disqualified as a usable signal by its INVERTED AUROC. Nothing armed.

**Date:** 2026-10-24 · **Box:** m3 (M3 Max) · **Posture:** clef-local ANE lane (clef-text-0.6b @ `3c61a609`, buckets [256,512,1024,2048], CpuAndNe, bit-exact parity authority FluidUse@0230049c) · vocab 151936 · ≤100 cases/suite · 1209 question rows, 0 skips · run: `cargo run --release --features clef-ane --bin harness -- --clef-conf-probe --clef-conf-max-cases 100 --out .benchmarks/158_clef_conf_probe`

**Provenance (verbatim preflight):** `PROVENANCE: power=AC Power load=7.18 swap=0.00M canary=137.2us/best5 powermode=2(high)` — preflight REFUSED for latency claims (load 7.18 > 6.0, sibling session on the box). The accuracy/ECE/AUROC columns are LOAD-INVARIANT (deterministic ANE forwards — the lane's bit-exact parity gates — + fixed-order entropy folds + integer census + case-order split-half conformal), so the frozen read stands; the `mean_case_ms` column is characterization-under-load, NEVER a bar.

## The candidate

Per case: one `LocalAneLane::decide_with_entropy` pass (substrate riir-infer `0f876b8`) — the serving answers plus the LAST-POSITION vocabulary entropy from the SAME decode: the final hidden at the record's last real token projected through the tied table (`h·Eᵀ`, one dot per vocab row, four fixed-lane accumulators), peak-subtracted softmax (the answer walk's own convention), read as `H = peak + ln Z − Σ exp(l−peak)·l / Z` NATS. Entropy→confidence map: `(1 − H/ln V).clamp(0,1)` — monotone (AUROC map-invariant), into the [0,1] scale `ece_of`'s 15 bins read. CASE-LEVEL signal: one record = one last position, so every question of a case carries its case's entropy (typed_decisions' 5 questions share one scalar — the AUROC's tie handling prices that).

**Lens depth is DEFERRED, not skipped silently:** the ANE posture ships ONE output (the final states); per-layer states never leave the Core ML graph, so a logit lens at depth needs a CPU forward port — a substrate lane of its own (the unblock condition, recorded in the issue).

## The board

| suite | q | acc | AUROC(−H→correct) | ECE(H) | ECE(conf) | conformal floor | modal acc | modal ECE | mean ms/case | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| typed_decisions | 500 | 0.4220 | 0.4340 | 0.4599 | 0.1578 | 0.3319 | 0.4640 | 0.1308 | 335.7 | NEGATIVE(conformal ≥ floor; modal ≥ floor; acc ≤ modal acc — mimicry) |
| ag_news | 100 | 0.8700 | 0.6198 | 0.0406 | 0.0671 | 0.4733 | 0.2625 | 0.0088 | 84.4 | NEGATIVE(modal 0.0406 ≥ floor 0.0088) |
| emotion | 100 | 0.6500 | 0.5741 | 0.2874 | 0.1352 | 0.1745 | 0.3125 | 0.2123 | 85.7 | NEGATIVE(conformal ≥ floor; modal ≥ floor) |
| sst5 | 100 | 0.4700 | 0.5183 | 0.4859 | 0.1214 | 0.2263 | 0.1383 | 0.0790 | 84.4 | NEGATIVE(conformal ≥ floor; modal ≥ floor) |
| prompt_injections | 58 | 0.4828 | 0.5298 | 0.4558 | 0.3994 | 0.3494 | 0.5345 | 0.0043 | 81.8 | NEGATIVE(conformal ≥ floor; modal ≥ floor; acc ≤ modal acc — mimicry) |
| xnli_en | 100 | 0.7700 | 0.5935 | 0.1659 | 0.1411 | 0.3584 | 0.3333 | 0.0283 | 80.5 | NEGATIVE(modal ≥ floor) |
| massive_intent_en | 100 | 0.8100 | 0.4659 | 0.1126 | 0.1329 | 0.2878 | 0.1100 | 0.0440 | 171.5 | NEGATIVE(modal ≥ floor) |
| banking77 | 100 | 0.8300 | **0.3650** | 0.1460 | 0.0807 | 0.3200 | 0.0460 | 0.1771 | 795.4 | **PASS (letter) — DISQUALIFIED (inverted AUROC)** |
| semantic_defects | 51 | 0.4510 | 0.5202 | 0.4706 | 0.2245 | 0.2426 | 0.1667 | 0.1111 | 130.0 | NEGATIVE(conformal ≥ floor; modal ≥ floor) |

Floors: (a) conformal = the Drex lane's split-half shape over the lane's OWN max-prob conf pairs (first half calibrates, second half audited, occupancy 40 — typed/xnli/ag/massive/prompt/emotion/sst5/banking77/semantic_defects pooled pairs 500/100/100/100/58/100/100/100/51); (b) modal = the T1 census with the CAL slice as reference (`census_with_pairs`), priced BOTH as accuracy (the mimicry bar for the lane) and as calibration (the pick's census share through the same `ece_of`).

## Verdict — the G1 discipline fires

1. **The candidate fails BOTH floors on 6 suites outright** (typed, emotion, sst5, prompt_injections, semantic_defects) or fails the modal floor on 3 more (ag_news, xnli, massive). The entropy ECE is nowhere near accuracy on the hard suites (typed 0.460 vs acc 0.422; sst5 0.486 vs 0.470; semantic_defects 0.471 vs 0.451) — a mid-vocab entropy reads ~half the vocabulary mass no matter the question, and the `(1 − H/ln V)` map lands it far off the accuracy scale exactly where calibration matters.
2. **The single PASS is not a signal.** banking77 clears both floors by the letter (0.1460 < 0.3200 conformal, < 0.1771 modal) while its AUROC(−H→correct) = **0.3650 — INVERTED** (lower entropy correlates with being WRONG on the 77-class suite). A confidence readout that ranks backwards passes an ECE bar only by scale coincidence; adopting it would ANTI-select. The G1 law's intent (a usable confidence signal) fails, and the record says so.
3. **The modal floor dominates again** (the paper's anti-mimicry law, second consecutive bench): the census-share ECE is tiny on most suites (0.0043–0.13) because the census pick is near-deterministic per key. Any model-derived confidence must beat a nearly-deterministic baseline to arm — the bar doing exactly its job.
4. **Where the entropy ECE beats the incumbent's raw ECE** (ag_news 0.0406 vs 0.0671; massive 0.1126 vs 0.1329; xnli 0.1659 vs 0.1411), it still fails the modal floor — no suite shows a monetizable candidate.

**Recorded NEGATIVE for confidence use on the clef lane; the probe ships report-only, arms nothing, changes no serving path.** The keeper artifacts: the per-suite entropy distributions (rows json) as a checkpoint fingerprint, and the `decide_with_entropy` seam itself (one-pass answers+diagnostics, no second ANE forward) — the plumbing any future headful candidate reuses.

## Cost (T4 disclosure)

`mean_case_ms` above is measured UNDER LOAD (preflight refused; sibling session) — characterization, not a bar. Shape: the ANE forward dominates (banking77's bucket-1024 inputs read ~8× ag_news's bucket-256); the tied-table entropy sweep is ~155 MFLOP/case (151936 × 1024, four-lane fixed-order folds) — low-ms in release, and the incumbent readout (max-prob) adds ZERO (free with the answers). Any future arming re-prices G2 on a quiet box per the issue's cost law.

## Provenance

- Substrate seam: riir-infer `0f876b8` (`answer_with_entropy` + `last_position_entropy`, the fixture-pinned peak term).
- Consumer: reflex `--clef-conf-probe` (feature `clef-ane`, macOS) — `src/harness/runner/clef_conf_probe.rs`; modal pairs via `modal_floor::census_with_pairs` (the share = the pick's pool-mass share, LOO-adjusted; fallback = the global share).
- Raw: `.benchmarks/158_clef_conf_probe/{clef_conf_probe.json, CLEF_CONF_PROBE.md}` (auto-written by the lane).
