# Bench 173 — Issue 107: reflex routing-state probe + pinned-path replay PoC — NEGATIVE (no signal beyond the argmax; no pin headroom)

**Date:** 2026-11-04
**Issue:** reflex [Issue 107](../.issues/107_routing_state_probe_and_pinned_path_poc.md) (Research 014 / arXiv:2611.03526, the Library-of-Layers distill).
**Instrument:** `harness --routing-state-probe` (`src/harness/runner/routing_state_probe.rs` + the read-only `DecisionEngine::routing_directions()` accessor in `src/engine.rs`); report + GATE on the witnesses (a witness-red lane refuses the run).
**Verdict: NEGATIVE — T1 AGREE-WITH-MI on the LOW-MI trio + banking77 UNDERREAD (fit capacity, not absent signal); T2 no headroom anywhere; T3 NOT TRIGGERED; T2.3 held (the seam stays production-disarmed).**

## Run

- sha `b8ec0eb` (the claim commit; the landing commit carries this record), host `amo`, 2026-11-04T04:38:40Z, AC power, lowpowermode 0, load ~3.4 (quiet box; CPU-only lane — no GPU consumer).
- Determinism ×2: the full JSON is byte-identical across two runs minus the timestamp.
- Populations: eval slots = stride_indices' own semantics over the choice/score test cases at max 300 → ag_news 200 · emotion 200 · sst5 300 · banking77 250; the fit slice = each suite's whole cal slice (200 cases, disjoint from the test split by construction — different source splits).

## T1 — the supervised probe (gates at the pre-nominated λ = 10.0)

| suite | fit | eval | N | MI bits | H(label) | routed acc | probe acc (score) | Δ | 2σ | macro AUROC (score) | incl/excl | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ag_news | 200 | 200 | 4 | 0.1506 | 1.9686 | 0.3650 | 0.3600 | −0.0050 | 0.0681 | 0.5880 | 4/0 | AGREE-WITH-MI |
| emotion | 200 | 200 | 6 | 0.1197 | 2.2993 | 0.1450 | 0.1950 | +0.0500 | 0.0498 | 0.4955 | 5/1 | AGREE-WITH-MI |
| sst5 | 200 | 300 | 5 | 0.0677 | 2.2606 | 0.2133 | 0.2333 | +0.0200 | 0.0473 | 0.5187 | 5/0 | AGREE-WITH-MI |
| banking77 | 200 | 250 | 77 | 4.8947 | 6.2547 | 0.5120 | 0.1920 | −0.3200 | 0.0632 | — | 0/77 | UNDERREAD |

- The score-vector arm is the MANDATORY arm: the pre-argmax router scores re-derived bit-exactly from `Scratch.q` + `routing_directions()` with `pick_domain`'s exact f32 fold, per-slot asserted three ways (engine domain == pick_domain re-derivation == plain-scan argmax). No mismatch fired on 950 captured slots.
- The one-hot arm (the MI consistency check) agrees everywhere it should: ag_news 0.3650 (= routed), emotion 0.1650, sst5 0.2267, banking77 0.3800 (the per-domain modal-class lookup — the routed domain index IS the class there, so identity lookup 0.512 > modal lookup 0.380 is the expected ordering).
- MI recomputed on eval slots is comparable to Bench 167's slot-level read (banking77 4.89 vs 5.59 at the smaller sample; the LOW-MI trio 0.07–0.15 vs 0.14–0.28).
- **banking77's UNDERREAD is fit capacity, not absent signal:** the λ grid is monotone in the fit direction — probe acc 0.192 at λ=10 → 0.248 at λ=1 → 0.340 at λ=0.1 → 0.316/0.348 at λ=1e-3/1e-2 — and even the unregularized end stays ~17–20 pp under routed: 2.6 fit samples per class over 77 classes cannot fit 77 one-vs-rest boundaries on 250 eval slots. The probe's linear capacity is the binding constraint, exactly the class of disagreement T1.4 pre-registered to surface rather than bury.
- **The λ-grid tension, disclosed:** ag_news's optimistic-λ row (λ=1e-3) reads acc 0.4550 = +9.0 pp over routed — ABOVE the 2σ band — but its macro AUROC (0.6899) stays UNDER the pre-registered 0.70 OVERREAD bar and the gate row (λ=10) reads AGREE. The verdict stands per pre-registration; the optimistic bound is on the record and does not clear the bar either.
- T1.3: AUROC-only claim; no threshold/abstention consumer proposed → the conformal-naive floor does not apply.

## T2 — mode-path pins (report-only; `forced_domain` stays production-disarmed)

| suite | pin | paired | live | pinned | Δ | LB95 | verdict | abst Δ | fallback |
|---|---|---|---|---|---|---|---|---|---|
| ag_news | suite_constant | 100 | 0.4000 | 0.4000 | 0.0000 | 0.0000 | NON-INFERIOR | +0.055 | 0 |
| ag_news | predicted_class | 109 | 0.4037 | 0.4037 | 0.0000 | 0.0000 | NON-INFERIOR | +0.010 | 0 |
| ag_news | gold_oracle | 104 | 0.4327 | 0.4327 | 0.0000 | 0.0000 | NON-INFERIOR | +0.040 | 0 |
| emotion | suite_constant | 132 | 0.3258 | 0.3182 | −0.0076 | −0.0224 | INFERIOR | −0.020 | 0 |
| emotion | predicted_class | 132 | 0.3258 | 0.3182 | −0.0076 | −0.0224 | INFERIOR | −0.020 | 0 |
| emotion | gold_oracle | 132 | 0.3258 | 0.3182 | −0.0076 | −0.0224 | INFERIOR | −0.020 | 0 |
| sst5 | suite_constant | 176 | 0.1591 | 0.1591 | 0.0000 | 0.0000 | NON-INFERIOR | +0.013 | 0 |
| sst5 | predicted_class | 175 | 0.1543 | 0.1543 | 0.0000 | 0.0000 | NON-INFERIOR | +0.020 | 1 |
| sst5 | gold_oracle | 176 | 0.1591 | 0.1591 | 0.0000 | 0.0000 | NON-INFERIOR | +0.013 | 0 |
| banking77 | suite_constant | 6 | 0.3333 | 0.3333 | 0.0000 | 0.0000 | NON-INFERIOR | +0.440 | 0 |
| banking77 | predicted_class | 58 | 0.5862 | 0.6379 | +0.0517 | −0.0058 | NON-INFERIOR | +0.216 | 0 |
| banking77 | gold_oracle | 66 | 0.6212 | 0.6364 | +0.0152 | −0.0145 | INFERIOR | +0.176 | 0 |

The pre-registered expected shape confirmed on every suite:

- **Suite-keyed constants regress on banking77-class suites** — the class signal dies: +44 pp abstentions (the distance gate fires against a far forced corpus) leaves 6 paired slots; the flat "NON-INFERIOR" on 6 pairs is meaningless support, the abstain delta is the readout. On the LOW-MI trio the constant pin is byte-flat (ag_news/sst5) to slightly inferior (emotion −0.76 pp) — the answers barely depend on the domain there.
- **Predicted-class pins can at best match live routing (ceiling = the probe's accuracy)** — banking77's +5.2 pp paired gain is a SELECTION EFFECT, bought with +21.6 pp abstentions: probe-right cases force the gold corpus and answer; probe-wrong cases abstain through the distance gate and drop out of the paired set. It never exceeds the ceiling the issue pre-named, and its all-slot behavior is worse than live.
- **The gold ORACLE pin bounds what a perfect classifier-keyed pin could achieve — and it is INFERIOR at the bar on the only suite with signal** (LB95 −0.0145 < −0.01): live routing is already 0.51 domain-accurate at MI 4.89 bits — the router IS most of a classifier, so forcing has no headroom over it. This is the decisive T2 read: the LoL forcing mechanism transfers mechanically (adherence asserted green per slot on all 12 pins) but buys nothing on reflex's engine.
- Extraction witnesses: bit-identical re-extraction (BLAKE3 digest equality) green on all four suites; capture-determinism witness (first 10 fit slots re-decide bit-identically, scores compared by bits) green. Digests: ag_news `deaab4c2…` · emotion `597e2138…` · sst5 `d1aa9adb…` · banking77 `28baaed3…` (full digests in the JSON).

## Verdict chain

1. **T1: the supervised probe finds NO label-relevant signal beyond the routed argmax at the pre-registered bar** on reflex's hashed-bag routing features (LOW-MI macro AUROCs 0.49–0.59 at the gate λ; even the optimistic λ row 0.6899 < 0.70). banking77's underread is a fit-capacity finding with the λ grid as the mechanism evidence. The Bench-167 MI picture is now completed by its supervised bound: MI predicted exactly what the probe found.
2. **T2: the pinned-replay posture has no headroom** — the oracle pin itself fails non-inferiority on the only suite with class signal; everywhere else the pins are flat-to-inferior or abstain-destroyed.
3. **T3 NOT TRIGGERED** — no OVERREAD verdict (T3.1's condition), no parity posture (T3.2's condition). No follow-up issues filed.
4. **T2.3 held** — the forced-domain seam stays disarmed on production paths exactly as Bench 167 left it. Nothing was wired.

## Cross-repo standing

- The supervised-readout class on reflex features is now **0-for-2**: Bench 157 (residual stream, depth-as-confidence) + Bench 173 (routing state, supervised probe). Both are different feature sources measured to the same conclusion — the engine's signal lives in the CONTENT features (LabelHeads/NbRidge), not in routing-state side channels.
- refine-side campaign-row probing (Research 014's consumer-side note) adopts ONLY if this PoC passes — it did not; that adoption is dead too.
- The 2610.07774 prior-art class (router-logit safety probe) remains published prior art; this bench adds a measured negative on hashed-bag routing features, not a contradiction — their routers read trained-LM hidden state mass, ours read hashed-bag corpus affinity, and the MI numbers predicted the outcome in advance.

## What landed (reflex-side diff)

- `src/engine.rs` — `routing_directions()`, the read-only N×D direction accessor (16 lines, the `gate()` posture: instrumentation-only, no production caller).
- `src/harness/runner/routing_state_probe.rs` (+ `runner.rs` wiring) — the probe: T1 capture (score-vector + one-hot arms, per-slot consistency gate) + ridge OVR (the `nb_ridge` solve shape via katgpt-core `ridge_solve_direct_f32`) + macro-OVR AUROC with tail support + the T1.4 verdict + T2 extraction/BLAKE3-digest/adherence-gated replay + witnesses. Five unit tests (ridge separability, AUROC perfect/flipped/thin, digest determinism + modal-sensitivity, the `pick_domain` tie-rule pin, degenerate detectors).
- `src/bin/harness.rs` — the `--routing-state-probe` / `--routing-state-probe-max-cases` exclusive early-exit mode (witness-red refuses).
- fmt gate PASSED (126 files, drained a pre-existing `examples/pair_free_probe.rs` drift — never pinned); clippy −D warnings at default + all-features + no-default-features; 517 lib tests green.

## Posture disclosure

Deployed engine build (`fit_posture_inner` + `build_engine_with`, selection levers pinned off, cal-slice `observe` calibration) — the same posture as Bench 167, so the two records read against each other directly. The probe adds NO production path: the accessor has no production caller; the pins run only through the diagnostic seam inside this lane.
