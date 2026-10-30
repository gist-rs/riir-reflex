# Bench 172 — label-perm: seeded label-STRING bijection vs identity (riir-train 657 R8)

**Verdict: NAME-CONDITIONING CLASS IS ALIVE — no null.** Under a Sattolo label-string σ (options AND gold follow the same bijection, corpus + domains + question keys renamed consistently — the invariance arm), both lanes lose material accuracy: the lexical lane degrades gracefully (banking77 −11 to −19 pp, all 3 seeds' CIs exclude 0; ag_news −5.5/−7.5 pp), while the encoder lane collapses near-totally (ag_news 0.965 → 0.03–0.065, **below the 0.25 chance line**; xnli 0.84 → 0.05–0.11 vs 0.33 chance; emotion −46 to −51 pp; banking77 → ~0.01). The keep-corpus control (questions+labels renamed, corpus NOT relabeled) confirms the structural layer: banking77 → ~0.01 (chance at 77 classes) — the by-name option→domain routing is load-bearing, exactly the Issue-079/README-36 routing law.

Spec of record: riir-train `.issues/657` R8 (dispatch mail `01a18dc0`, executed by the claiming session). **NOT Bench 119** — Bench 119's axis is formatting churn of the state text; this axis is the label identity itself. Distinct from Bench 128 (`--perm-probe`): Bench 128 permutes option ORDER with names intact (modelless gate median ≤ 2 pt = PASS); this probe permutes the NAMES with order intact — the deltas here are 5–20× larger, which is the finding.

## Postures (all: `.raw/datasets` canonical, 3 seeds/suite, seeds = base ^ FNV-1a(suite) + {0,1,2})

| posture | dir | slice | lanes |
|---|---|---|---|
| invariance (full-consistent σ) | `172_label_perm/modelless/` | full (400/400/300/500) | modelless |
| keep-corpus control | `172_label_perm/modelless_keep_corpus/` | full | modelless |
| encoder | `172_label_perm/metal/` | ≤200/suite cap | modelless + laya/english (Metal) |

Key deltas (Δ = acc@σ − acc@id; range over seeds 0–2; full tables + JSON in the dirs):

| suite | modelless invariance | modelless keep-corpus | laya/english (keys σ, desc intact) | laya acc@id |
|---|---|---|---|---|
| banking77 | **−10.8 … −19.2** (all CIs excl. 0) | **−38.4 … −38.6** (→ chance) | −41.3 … −41.9 | 0.425 |
| ag_news | −7.5 / −5.5 / +0.0 | −20.5 … −17.0 (all excl. 0) | **−90.0 … −93.5** (below chance) | 0.965 |
| emotion | −12.0 / −1.3 / +1.0 | −16.0 / −1.8 / +0.8 | −46.0 … −51.0 | 0.570 |
| xnli_en | −0.7 … −2.3 (CIs incl. 0) | −0.7 … −1.3 (CIs incl. 0) | −73.3 … −78.7 (below chance) | 0.840 |

## Readings

1. **Label-name tokens are load-bearing features in the lexical lane** (not mere identifiers): even under full-consistent σ the drafter loses the name↔class-body token-affinity channel. The loss scales with how world-knowledge-dense the names are — banking77 (77 fine-grained intent names, heavy doc-vocab overlap) loses most; xnli (3 generic labels) loses ~nothing. RT-J's "names are world-knowledge-dense" axis reproduced on our suites.
2. **The by-name routing channel is structural** (keep-corpus control): renamed keys with an unrenamed corpus drop banking77 to chance — the corpus→domain resolution keys on the name string. Disclosed control, not a gate.
3. **The encoder head's output layer is bound to trained name strings**: with descriptions intact it still collapses below chance — it cannot emit or rank renamed keys. The Δ conflates output-vocabulary binding with ranking failure; the clean second measurement is a RANK-ONLY variant (feed renamed keys, accept the head's original-name output, score through σ⁻¹) — filed as the follow-up, owner-gated to the Rethink lane (the encoder is Rethink's product row; record-only here).
4. Graceful degradation contrast: under σ the lexical lane retains corpus-routing signal (banking77 0.21–0.29 vs 0.013 chance); the encoder retains ~none. Informs the training-recipe lanes: a specialist head trained on name-conditioned vocabulary carries zero OOD-name robustness — relevant to R7 (EMA/SWA selection stability) and any future name-space migration of the serving suites.

## Caveats (honest scope)

- **Modelless identity-arm absolutes are plain-posture** (probe recipe = Bench 128's `perm_modelless`: plain posture, oc/nb select refused, no cal fit) and are NOT board rows (board A0 emotion 0.885 is the arena posture). Only the paired deltas are the measurement.
- **Laya numbers: G5 standing green** at the pinned `english` checkpoint (SHA-verified at load; weights unchanged since the parity gate last ran green). No site bench republish — no served-posture row moved (identity arms are probe-posture, not board rows).
- Laya cap 200/suite (forward-cost bound); n per row in the tables. Modelless postures ran full slices.
- Descriptions stayed INTACT everywhere (spec of record). A desc-blanking knob is the cheap follow-up if the rank-only variant is taken.

## Provenance

- date 2026-10-30T19:13–19:20Z · binary sha `56fa2a7` · host m3 (M3 Max, AC power)
- ⛔ **PROVENANCE CORRECTION (2026-10-30T19:26Z, post-run):** the box was NOT quiet — i1117 (pid 19526, `flashmemory_decode_gate`, the 128K Bonsai decode-gate latency benchmark, ~21 GB RSS, started Fri 01:42) was in flight throughout; an in-flight mail warning never reached the executing turn. CPU arms add load ≈19:11–19:16Z; the laya-metal arm is direct Metal GPU contention ≈19:16–19:20Z (feature build ≈19:05–19:14Z). **The accuracy figures are load-invariant and STAND** (deterministic Metal/CPU math; contention moves wall-clock, not values; slice-integrity rows all OK); **no latency figures are claimed** — but any i1117 latency analysis must quarantine samples in the 2026-10-30T19:05–19:21Z window (Bench-649 class); the affected lane has been notified. The original "Metal exclusive" line below was written from an incomplete box-state read.
- Vehicle: `--label-perm` harness arm (landed `56fa2a7`: pure `src/harness/label_perm.rs` + runner `src/harness/runner/label_perm.rs`; tests 14/14 at default + laya-riir-metal postures; clippy `-D warnings` both postures)
- Slices pinned by fnv1a64 slice-integrity rows in each posture dir (test/cal/pool), seeds per-suite recorded in `label_perm.json`.
- Executor session: zed-m3-r8run (claim `9a97a154` in riir-train, lease folded on completion).
