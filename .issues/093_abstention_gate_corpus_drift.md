# Issue 093: CorpusDistanceGate abstention drift vs corpus growth (capacity-law diagnostic)

**Status:** Open — diagnostic-first; no lever lands without a measured curve

**Source:** [katgpt-rs/.research/624_Associative_Memory_Capacity_Laws.md](../katgpt-rs/.research/624_Associative_Memory_Capacity_Laws.md) (blog: alexwang.ai "You could have invented transformers from memory", 2026-10-15; theory: Zhong et al. arXiv:2505.19488 interference-to-signal)
**Related:** Issue 013 (corpus-cap levers: ag_news 64 / banking77 40, chosen empirically) · Issue 066 (`density_gate`) · katgpt-core `ssmax.rs` + Plan 411 (the log-N law, attention-side) · harness G1–G5 gates · Issue 021 box-state provenance

## Finding (narrowed in review — one clean consumer; drift population corrected in round 2)

`CorpusDistanceGate` is the ONE serving surface where the capacity law binds cleanly: the gate compares **`max_sim` over the whole corpus** against a **fixed absolute midpoint** — `engine.rs`: `CorpusDistanceGate::new(rows, GATE_MID = 0.35, GATE_SCALE = 8.0)`, with the birth-time measurement "in-corpus max-cos ≈ 0.68–0.70, off-corpus ≈ 0.00–0.02 — the midpoint sits in the desert between the two populations".

**The drift carrier is the OFF-CORPUS population** (corrected in review — the in-corpus reading is backwards):

- In-corpus queries sit at ≈ 0.68–0.70, already **above** `mid`; adding rows can only raise each query's max over rows, so the in-corpus distribution moves further away from `mid` as M grows. The in-corpus tail is NOT the drift.
- An **off-corpus** query (unrelated to every row) gets M chances at an accidental near-neighbour: its max_sim (≈ 0.00–0.02 at birth) creeps **up toward `mid`** as the corpus grows. Once it crosses, the gate stops abstaining on exactly the inputs it exists to reject — **false-accept drift toward never-abstain**.
- The isotropic null `E[max_sim] ≈ √(2 ln M / d_k)` describes precisely this off-corpus statistic (a random query against M unrelated keys). The two sides of the desert move in OPPOSITE directions: the off-corpus upper tail rises toward `mid`, the in-corpus distribution rises away from it.

The null is a **NULL MODEL, not a law**: real hashed-bag embeddings are clustered and anisotropic, so `max_sim` is dominated by near-duplicates. Pre-registered semantics: a measured mismatch rules out the null; it does NOT refute the drift (the drift claim rests on the measured off-corpus tail, not on the null).

Explicitly out of scope after review (do not re-litigate here):
- `route_scale` — `route_base` scores the **N domains** (fixed count); corpus growth changes centroid quality, not M. A blend-weight calibration question (Issue-013-style sweep), not a capacity item.
- Option scoring (`DEFAULT_SCORE_TEMPERATURE`) — bounded option sets, blend-weight class; kept only as the deferred lowest-priority tail (none here — tracked in the note's consumer table).
- Any pure-ranking surface — a monotone sigmoid gate cannot change a ranking outcome at any sharpness or threshold (Research 624 §2.1 scope law 1).

Confounder to control: the cascade re-fit posture (`harness/cascade.rs` — gate fitted at the cal-slice **30th percentile**) re-derives the threshold from measured geometry, so it may absorb the drift **by construction**. The fixed-`mid` default and the re-fit posture are measured SEPARATELY; a drift found only under fixed `mid` must be reported as such.

## Tasks

- [ ] **T1** Measure BOTH populations vs corpus size (1× / 2× / 4× concatenated suites): (a) **off-corpus `max_sim`** — the drift carrier — overlaid against the labeled isotropic-null reference curve `√(2 ln M / d_k)`; (b) in-corpus `max_sim` — the gap-width check (does the desert stay open from the other side). Record the **abstain rate split into off-corpus (true abstains) and in-corpus (false abstains)**. Both at (i) the fixed `GATE_MID=0.35` default posture and (ii) the re-fit posture, separately. Record `bench_preflight.sh` provenance beside any number.
- [ ] **T2** Verdict gate: close with the negative measurement only if the **off-corpus upper tail stays clear of `mid` at 4× in BOTH postures** (the in-corpus check passing alone is NOT a close — it passes by construction).
- [ ] **T1a** Verdict round-3 conditions, recorded verbatim from the reviewer (non-blocking, bind when T1 runs): (i) the off-corpus probe set must be drawn from a source **held out from EVERY concatenated suite** — a query off-corpus for one suite can become in-corpus once suites are concatenated, which mixes the two populations at every size beyond 1×; record how the probe set was built in the bench doc. (ii) tail quantiles must **report their sample support** — the drift lives in the upper tail, and a small probe set yields the MAXIMUM labeled as a quantile (the repo's p99/percentile-support trap).
- [ ] **T3** If (and only if) T1 shows **off-corpus** drift at the default posture: `mid(M)` compensation — either extreme-value-informed or per-domain measured — behind an env/feature lever, default OFF, byte-identical when OFF (A0 identity pin must hold: arena == run() == published row).
- [ ] **T4** Gates (lever arm only): G1 calibration (ECE/Brier + Report-the-Floor vs the conformal-naive floor), G2 p99 ≤ 1 ms in-process, G4 alloc-free; abstain-rate on the frozen suites must not regress at current corpus sizes when OFF.
- [ ] **T5** Re-measure the Issue-013 corpus caps (ag_news 64 / banking77 40) under the T1 curves: do the caps still bind at the calibrated gate? Record the answer either way — an empirical constant replaced by a measurement, or the caps vindicated.

## Constraints

- Sigmoid, never softmax — shift the threshold; no normalization, no softmax on the serving path.
- Public repo: no serving-posture numbers in code comments; measurements land in `.benchmarks/` with box state.
- The gate constants are load-bearing at birth measurement (engine.rs doc comments) — any `mid` change ships behind the lever, never edits the default.
