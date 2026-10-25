# Bench 163 — the OpenDecider-nano P0: **ADMITTED** — pooled typed_decisions 0.7850 ≥ bar 0.7825 (one frozen read, det 10/10, zero absences); posture-invariance EXACT (per-question bit-identical); board-8 recorded with the family split INVERTED vs the 013 expectation; perm-probe REDs 5/6 (order-sensitive choice answers — the load-bearing caveat)

**Status:** RECORD — the pre-registered admission read of rethink [Issue 038](../../riir-rethink/.issues/038_opendecider_nano_p0_admission.md), executed 2026-10-25 12:49–13:45+07 (seal1). Candidate: `manjunathshiva/opendecider-nano` (Apache-2.0, ~400M Ettin-encoder-400m/ModernBERT backbone + MLP decision head, HF sha `1e1f0a73…` — verified at read time: sha == pin, gated false, license tag apache-2.0, LICENSE file present). **Verdict: the bar CLEARS → the candidate is ADMITTED; rethink 034's recipe-work hold LIFTS.** The margin is +0.0025 — thin, and never citable as superiority (the selection-on-test law: the candidate was CHOSEN using a published read on this same cell; the P0 is an admission replication).

## 1. The gate (T3 — the adjudicating cell)

`OPENDECIDER_ALLOW_UNCAPPED=1 harness --opendecider --skip-laya --suites typed_decisions` (canonical `.raw/datasets`, whole-case posture, the frozen "none" render spec committed before the read) → `.benchmarks/163_opendecider_nano_p0/results.json`:

| column | value |
|---|---|
| **pooled accuracy (THE GATE)** | **0.7850** (n=2000) — bar 0.7825 → **PASS** |
| macro_f1 | 0.7810 |
| ECE(readout) / Brier | 0.1590 / 0.1650 |
| determinism (2× verbatim repeat) | **True, n=10** |
| p50 / p99 (client ms, local MPS serve) | 468 / 1351 |
| modelless on the same cell | 0.2940 (+49.1 pp for nano) |
| slice integrity | test 400 `fnv1a64-287d5f73` · cal 100 · pool 1100 — OK |

By question type: choice 0.7617 (n=600) · **noul 0.8300** (n=600) · score 0.7688 (n=800, score-MAE 0.195, within-1 0.996). The noul leg is nano's strongest — consistent with a calibrated-teacher distillation whose teachers were strongest on the boolean primitive.

## 2. T2 legs (both BEFORE the read — the 640-T0 law)

- **Model identity:** HF API re-read at the window — sha `1e1f0a7322e446c5ae9bbe840df11680f18a752a` == pin ✓, `gated: false` ✓, `license:apache-2.0` tag ✓, LICENSE file in the tree ✓ (blake3 `03cdfc12…`); weights blake3'd into the 038 record (`model.safetensors` 789,580,328 B `d52eddb1…`, `head.safetensors` `19807802…`). Weights never entered any repo (HF cache only).
- **Dataset identity:** the HF test split RE-FETCHED at revision `9ee69423` (dataset lastModified 2026-10-24) — all four pages byte-identical to our manifest pins (`0672487a/b80c98bb/47501688/5011d0b4`) → the fresh test aggregate equals our pinned `01d24801…` **exactly**. The published 0.796 was read on OUR EXACT test cell. No STOP condition. (Our per-split TEST pin was derived first, 05:49+07, reflex manifest.)

## 3. The secondary per-question posture — posture-invariance CONFIRMED EXACTLY

The pre-registration predicted agreement "to numerical noise" (an encoder-native prediction: each (state, question) pair is its own attended row; whole-case merely batches 5 rows per request). Measured: the per-question run (2000 single-question requests over a 1-question-per-row staging of the same test split — same harness, same lane, same scorer) returns accuracy 0.7850, ECE 0.15903139493335036, and every by-type cell **bit-identical** to the whole-case run; the ONLY differing lane fields are the timing/grouping columns (p50 244 vs 468 ms — the batching win; n_cases 2000 vs 400; wall 395 vs 229 s). Zero padding/batching non-invariance. The staged rig is gitignored (`.raw/datasets_opendecider_perq`); its results.json lives at `/tmp/opendecider_perq` (recorded here, never adjudicating).

## 4. T4 — the board-8 columns (record-only, never the gate; the B5 law holds HARD)

| suite | nano | ECE | the incumbent context (reflex board) |
|---|---:|---:|---|
| ag_news | 0.8100 | 0.037 | H2 0.8975 incumbent |
| banking77 | **0.6340** | 0.092 | clef 0.9540 / pplx 0.8040 — the 013 "BANKING77-strong" expectation REFUTED |
| emotion | 0.5700 | 0.103 | A0 0.8850 (the board's highest bar) |
| massive_intent_en | 0.7933 | 0.076 | H2 0.8400 |
| prompt_injections | 0.5172 | 0.347 | A1 0.8534 |
| sst5 | **0.5550** | 0.044 | A1 0.4217 — one of two cells where nano BEATS the incumbent |
| winogrande_jdi | 0.5067 | 0.153 | chance-adjacent (2-option) |
| xnli_en | 0.8300 | 0.198 | ENC 0.8600 / A0 0.5233 |

**The 013 family expectation is INVERTED**: nano reads AG News-fine (0.810) and BANKING77-weak (0.634) — the opposite of the predicted split. The B5 law (JDI rank ≠ our-suite rank) is the vindicated instrument: the published typed cell does NOT transfer across our suites. Each run: `PASSED — 1 suite(s), no absences`, under `board8/<suite>/`.

## 5. The perm-probe — **5/6 choice suites RED; the load-bearing caveat**

`--perm-probe --opendecider` over the six choice suites (K=5 orderings, ≤40 cases, the new probe arm reflex `1e6d4db`; modelless control PASSes 0.00 everywhere — the probe is healthy):

| suite | mean flip % | max TV | verdict |
|---|---:|---:|---|
| ag_news | 0.85 | 12.71 | PASS (the lone clean cell) |
| typed_decisions | 4.28 | 14.20 | RED |
| emotion | 4.27 | 18.17 | RED |
| xnli_en | 2.45 | 6.70 | RED |
| massive_intent_en | 2.67 | 67.05 | RED |
| banking77 | **12.96** | **64.87** | RED (worst) |

**nano's choice answers are OPTION-ORDER-SENSITIVE** — their card's "4-decimal cross-machine identity" claim is fixed-ordering determinism, not order-invariance (the [MASK]-per-option rows carry positional embeddings; permuting options moves the probabilities). Consequences, recorded not hedged: (a) the T3 gate stands AS MEASURED at the shipped dataset option order — the same fixed order every lane and every board entrant reads, so the admission is apples-to-apples; (b) any serving posture quoting nano choice numbers carries the order-sensitivity disclosure; (c) the 640-T4 router fold must not pool nano with order-invariant lanes without that disclosure; (d) a presentation-invariance fix is RECIPE work (the admitted candidate's first lever — e.g. option-sort canonicalization at the wire, which the render-spec law would re-register as a NEW pre-registration with a fresh read).

## 6. The verdict + what it lifts (T5)

- **P0 ≥ bar → ADMITTED.** rethink 034's hold LIFTS: recipe work resumes per its condition (iii) — the first levers being the order-invariance fix (§5d) and the banking77/emotion weakness (§4), both modelless-compatible or riir-train lanes (the spend gates T2/T4 unchanged — P0 survival + the owner's ESC re-arm decision still bind any serving move).
- **Still owner-gated (unchanged):** the ESC re-arm (Research 002), any seat/serving move, T2 fine-tune spend. The serving questions route to the owner WITH this record: nano's typed cell is the strongest measured encoder arm on our P0 cell (0.7850 vs the ENC rung's 0.7445 published baseline), but its order-sensitivity and the inverted family split say "recipe work first", not "seat move now".
- The one-frozen-read law: this record's T3 cell is THE adjudicating read; every other column is secondary (the per-question posture, board-8, perm-probe) and never substitutes.

## Provenance

- Lane: reflex `cb221da` (--opendecider, the pplx-family wire) + the perm-probe arm `1e6d4db` (475 lib green, clippy clean). Serve: `opendecider serve --model manjunathshiva/opendecider-nano` v0.5.x from the pinned snapshot (`.raw/opendecider-env`, CPython 3.12.15), 127.0.0.1:8000, dtype float32 ("as evaluated"), device auto (MPS), ~11.8 s load from the warm HF cache.
- Render spec: **"none"**, frozen in 038 before T3 (live wire-smoke: bare-noul handshake 200, 40/40 synthetic questions parsed + scored, determinism 8/8 — the smoke rig + staged dir are gitignored throwaways).
- Box state: AC power; the box carried the sibling i628 probe load (8–16) through the reads — accuracy/determinism are load-independent (the 10/10 verbatim repeats), the latency columns carry the shared-box caveat (never the claim). Memory law honored: the boot waited for the mammoth-panel marker (~12:20+07); the idle mammoth serve stayed resident (the hosted-parity arm wants it) beside nano's ~3 GB.
- Session: zed seal1 2026-10-25, epoch ~1792950000; the 038 claim line carries the session.
