# Bench 154 — CorpusDistanceGate drift vs corpus growth (Issue 093 T1/T1a/T2/T5)

**Status:** MEASURED 2026-10-17 — drift CONFIRMED; T2 close condition FAILS (the issue does not close negative); refit false-accept decay 48.7% → 77.0% is the headline finding; lever arm (T3) premise-fail, no lever. Probe: `--gate-drift-probe` (report-only, landed this bench). Record: `.benchmarks/154_gate_drift/` (json + md) · Issue: `riir-reflex/.issues/093_abstention_gate_corpus_drift.md`.

## Question

`CorpusDistanceGate` compares the routed domain's max cosine similarity against a FIXED absolute midpoint (`GATE_MID = 0.35`, slope 8). Does the off-corpus population drift UP toward the midpoint as the corpus grows (M-chances at an accidental near-neighbour), and does either posture keep the gate honest under that growth?

## Instrument (pre-registered in Issue 093 T1/T1a; recorded here so the numbers carry their construction)

- **Sizes 1× / 2× / 4×** by suite concatenation over `[ag_news, emotion, sst5, xnli_en]` (disjoint label vocabularies): domains 4 → 10 → 18, corpus rows 256 → 640 → 1152 at the registry cap 64/label (asserted). Engine per size = the deployed build path (`build_engine_with`, `EngineConfig::default()`).
- **Off-corpus probes** (T1a-i): test rows of `[banking77, massive_intent_en]` — sources held out from EVERY family suite at EVERY size, so the populations never mix. Stride-sampled 200/source (banking77 193 + massive 199 measured after empty-text drops: 8 dropped, counted).
- **In-corpus probes**: each family suite's stratified eval front (registry test caps), 100/suite per surface.
- **Two surfaces, never pooled**: *corpus-shaped* (both populations — probe text embedded ALONE, the corpus-row shape; routed via `semantic_route`; plus the whole-corpus max over all N gates = the M-chances statistic, bit-identical rows to the shipped gates); *engine-shaped* (in-corpus only — real eval requests through `solve_into`, the routed gate on the context embedding `sc.q`, exactly what the deployed fused gate consumes).
- **Both postures, separately** (the issue's confounder law): fixed = the birth boundary (`max_sim < 0.35` ≡ `conf < 0.5`); refit = the ρ=30 percentile distance threshold re-fitted on THIS size's pooled cal slice (200/suite → 200/400/800 observations). Plus a disclosed addition: the 1× fitted threshold HELD at later sizes (the aging-deployment posture).
- **Tail support** (T1a-ii): every quantile carries rank + support (`n − rank`) in the record; the max is reported separately, never laundered through a percentile label. The 1× in-corpus p99 rows (sup 1) are exactly the trap the condition guards — one near-duplicate probe, disclosed as support-1, never cited as a tail.
- **Null overlay**: labeled isotropic reference `√(2·ln M / 256)` at M = corpus rows.

## Provenance (box-state law)

`bench_preflight.sh` REFUSED (load 28.96 > 6 — sibling dist build on the box; swap 7486 MB; powermode high): `PROVENANCE: power=AC Power load=28.96 swap=7486.62M powermode=2(high)` — **no latency number appears in this record**, and none is owed (T4's G2 bar belongs to the lever arm, which did not land). The measured quantities are deterministic arithmetic over fixed inputs (no RNG, no timing): the distributions, rates, and verdict inputs are load-independent; only wall time moved. BoxState start/end recorded in the json (`latency_quotable: false`, refusal named). Datasets: `.raw/datasets` (canonical pull). Probe sha `86aa735` (the claim commit; probe code landed in the same unit, see the commit ref below).

## Results (the probe's GATE_DRIFT.md carries the full tables; the load-bearing rows)

Off-corpus, whole-corpus read (the M-chances statistic; the routed read tracks it ~0.02–0.07 below):

| size | M | null | off p50 | off p90 | off p99 | off max |
|---|---|---|---|---|---|---|
| 1× | 256 | 0.2081 | 0.2673 | 0.3634 | 0.4490 | 0.4813 |
| 2× | 640 | 0.2247 | 0.3489 | 0.4560 | 0.5566 | 0.6673 |
| 4× | 1152 | 0.2347 | 0.3693 | 0.4670 | 0.5687 | 0.6673 |

Abstain rates (corpus-shaped routed read; engine-shaped in-corpus rates in the record):

| size | refit thr (conf) | refit thr (max_sim) | off: fixed abstain | off: refit abstain | **refit off FALSE-ACCEPT** | in: fixed abstain | in: refit abstain | in ENGINE: refit abstain |
|---|---|---|---|---|---|---|---|---|
| 1× | 0.4105 | 0.3063 | 90.8% | 51.3% | **48.7%** | 68.0% | 19.0% | 9.0% |
| 2× | 0.4160 | 0.3076 | 73.7% | 24.0% | **76.0%** | 43.5% | 10.5% | 6.0% |
| 4× | 0.4257 | 0.3126 | 71.9% | 23.0% | **77.0%** | 51.5% | 15.2% | 8.2% |

Held-1× posture ≈ refit at every size over this growth range (off abstain 51.3% → 23.5% → 21.4%) — the aging-deployment posture does not degrade materially beyond the refit's own decay over 1×→4×.

## Findings

1. **The drift is real and ~6–10× the isotropic null.** Off-corpus p50 rises +0.102 (1×→2×) then +0.020 (2×→4×) against a null prediction of +0.017/+0.010. The clustered/anisotropic near-duplicate structure of hashed-bag embeddings dominates the random-chance effect, exactly as the issue's null-model caveat pre-registered — the null is RULED OUT as an explanation of the level, and the measured curve is the drift.
2. **The off-corpus population crosses the midpoint between 1× and 2×** (p50 0.2673 → 0.3489 → 0.3693 across mid 0.35). The birth desert, to the extent one existed on the birth corpus, does not exist on these real-text corpora at any measured size: BOTH populations straddle mid from the start (in-corpus routed p05 = 0.1913/0.2221/0.2134 — below mid everywhere).
3. **The deployed re-fit posture degrades with growth on the axis it does not calibrate.** The ρ=30 fit targets the IN-CORPUS distribution; its off-corpus false-accept rate roughly DOUBLES across the measured growth (48.7% → 76.0% → 77.0%). The separating work the gate does at 1× (off 51.3% vs in 19.0% abstain) is mostly gone by 4× (23.0% vs 15.2%). This is the capacity-law cost, measured on what ships.
4. **The fixed birth posture is already inside both populations at 1×** (false-abstain on 68.0% of in-corpus probes; the runner's own gate-fit comment has said the birth constants "do NOT transfer to these suites" since the harness measured 100% abstain on several) — this bench QUANTIFIES that transfer failure per population and per size.
5. **Growth curve is monotone but concave** in suite count: the first concatenation (+emotion, +384 rows, +6 domains) moves off p50 six times more than the next two combined. Domain-count growth (more routing chances), not row growth within a domain (K pinned at cap 64), is the dominant carrier on this axis.

## Verdicts per the issue's own gates

- **T1: DONE** — both populations measured at 1×/2×/4×, both postures, both surfaces, drift confirmed (finding 1/3).
- **T1a: DONE** — held-out probe sources recorded verbatim; support columns on every quantile; the p99 support trap disclosed in-record (sup 4 at n=392; sup 1 in the 1× in-corpus cell — one near-duplicate probe, cited as exactly that).
- **T2: the close condition FAILS — the issue does NOT close.** The off-corpus upper tail is NOT clear of mid at 4× in either posture (whole-corpus p99 0.5687 and max 0.6673 > 0.35; routed p99 0.4936 > 0.35; refit false-accept 77.0%). Per T2's own law, a negative close is unavailable; the measurement STANDS AS THE OPEN RECORD.
- **T3 (conditional lever): premise FAILS — no lever.** `mid(M)` compensation presumes a desert that drifts shut. The measurement shows the boundary sits INSIDE both populations at every size (in-corpus p05 < mid everywhere; off-corpus p50 already ≈ mid at 2×). No midpoint schedule restores a separation the fixed frame never had on these corpora; the lever arm is dead by premise-fail, recorded here rather than built. (The issue's own law: no lever lands without a measured curve — the measured curve now exists and it vetoes the lever.)
- **T4: n/a** — the lever arm did not land; no G1/G2/G4 runs owed.
- **T5: DONE** — at the registry caps (64/label) no birth-frame desert exists at any measured size; the Issue-013 caps (ag_news 64 / banking77 40) are accuracy/latency knobs and were never holding a gate desert open on these suites. banking77's cap-40 reads the same law by the monotone envelope (smaller K lowers E[max] per domain but the measured overlap is population-level, not K-level). The separating work on the serving path is the REFIT — re-confirmed as the standing design.

## Caveats (honest scope)

- The growth axis here is SUITE-CONCATENATION (domain count + total rows at pinned per-domain K). The OTHER growth axis — per-domain K growth on a single suite (cap 64 → 128 → 256) — is NOT measured by this instrument; the null's M-chances frame applies to it more directly, and it is the natural follow-up if anyone revisits the gate's capacity behavior. (Concatenation was the issue's pre-registered reading; the cap axis was not pre-registered, so it was not measured.)
- The off-corpus sources (banking77/massive) are real customer-service/intent text: "off-corpus" means held-out from the corpora, NOT semantically unrelated to every row — some probes are genuinely near rows (the 0.6673 maxes are near-duplicates). That is the deployed gate's real operating condition; the birth measurement's 0.00–0.02 off-corpus population is not reachable by any real-text probe on these corpora.
- p99 supports are 4 (off, n=392) and 1–4 (in-corpus cells) — the tails are disclosed, not asserted; the population-level claims ride p50/p90 (supports 196/40 and 50/10).
- One probe run, deterministic; re-running reproduces the record byte-for-byte modulo the box-state lines (no RNG; stride sampling).

## What responds to this (owner-visible; nothing landed)

The deployed ρ=30 refit is calibrated to in-corpus abstention, and that contract it keeps (in-corpus refit abstain 9–19% engine-shaped — near its target). Off-corpus rejection was never its calibrated job, and the measurement prices what that costs under corpus growth (finding 3). Any change — a second, off-corpus-targeted tail threshold; an explicit off-corpus probe fit; or an accepted-risk note — is a DESIGN act on the serving posture and stays owner-gated. Nothing here edits the gate constants (the load-bearing law holds: the only code change is the probe itself + the consts' visibility).

## Files

- Probe: `src/harness/runner/gate_drift.rs` (+ `--gate-drift-probe`/`--gate-drift-off`/`--gate-drift-in` in `src/bin/harness.rs`; `GATE_MID`/`GATE_SCALE`/`gate_confidence` made pub in `src/engine.rs` — the one-home confidence formula the probe reads).
- Record: `.benchmarks/154_gate_drift/{gate_drift.json,GATE_DRIFT.md}`.
- Issue: `.issues/093_abstention_gate_corpus_drift.md` (T1/T1a/T2/T3/T5 marked with this bench; status updated; stays OPEN carrying the record).
