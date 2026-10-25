# Bench 162 — the T10.2 LOCAL mammoth panel: all 12 suites landed on the seal1 self-host (mammoth-1 bf16 onejev81ce62f, MPS), determinism_ok 12/12, parity vs the board's own prints holds on every comparable column — banking77 0.9290 (board skill 0.9383), winogrande 0.9204 (0.9226), mmlupro 0.7411 (0.7183), anli 0.7554 (0.7443), hellaswag 0.9680 (0.9797)

**Status:** RECORD — panel executed 2026-10-24T18:07Z → 2026-10-25 ~12:35+07 (watchdog `55972` → marker `/tmp/mammoth_panel_done.marker` at ~12:20+07); fold executed same day (rethink side: rethink bench 073). Rethink Issue 032 T10.2's LOCAL-FIRST arm is DONE — the hosted (Baseten-direct) half still waits on T10.1's owner key (window ends 2026-10-27 23:59 PT); this record is the parity reference that hosted run will be checked against.

## 1. The panel

`riir-reflex/.raw/mammoth-local/run_panel.sh` — 12 sequential `harness --mammoth --skip-laya --suites <s>` runs over the canonical `.raw/datasets` (no `--datasets-dir` workaround), `--out .benchmarks/mammoth_local_panel/<s>/` (results.json + TABLES.md each). Serve: `.raw/mammoth-local/serve_mammoth.py` — the LOCAL self-host of **mammoth-1**, bf16 onejev81ce62f weights (~55 GB RSS), MPS, `MAMMOTH_DEVICE=local-seal1-mps-bf16-onejev81ce62f`, loopback :8793 (no forwarder — the local serve binds directly). Every suite prints `harness: PASSED — 1 suite(s), no absences`.

The boot path is the quiet-box watchdog (`watchdog.sh`, the 2026-10-24 stand-down law): two consecutive quiet polls (swap < 8 GB, free > 55%, no sibling bench, load < 4.0) before serve boot — it found its window 2026-10-24T18:06Z and launched the panel at 18:07Z.

## 2. The numbers (mammoth lane; board metric; the per-suite results.json is the instrument of record)

| suite | n (questions) | accuracy | macro_f1 | ECE(readout) | Brier | mammoth wall (s) | p50 ms |
|---|---:|---:|---:|---:|---:|---:|---:|
| banking77 | 500 | — | **0.9290** | 0.0361 | 0.0507 | 3,117 | 5,267 |
| winogrande_jdi | 1,267 | **0.9605** | 0.9605 | 0.0272 | 0.0363 | 1,923 | 1,491 |
| mmlupro_jdi | 2,000 | **0.7720** | 0.7711 | 0.0364 | 0.1194 | 11,166 | 2,770 |
| anli_jdi | 3,200 | 0.8372 | **0.8369** | 0.0762 | 0.1283 | 9,919 | 2,350 |
| hellaswag_jdi | 2,000 | **0.9765** | 0.9765 | 0.0084 | 0.0181 | 8,027 | 2,754 |
| sst5 | 600 | **0.5517** | 0.5361 | 0.1106 | 0.2582 | 1,126 | 1,864 |
| ag_news | 400 | **0.8875** | 0.8787 | 0.0620 | 0.0896 | 819 | 1,876 |
| emotion | 400 | **0.5875** | 0.4926 | 0.1077 | 0.2048 | 717 | 1,775 |
| massive_intent_en | 300 | **0.9367** | 0.9320 | 0.0210 | 0.0375 | 841 | 2,633 |
| prompt_injections | 116 | **0.7241** | 0.7071 | 0.1854 | 0.2021 | 263 | 1,785 |
| xnli_en | 300 | **0.9300** | 0.9302 | 0.0755 | 0.0653 | 624 | 1,939 |
| typed_decisions | 2,000 (400 cases) | **0.7605** | 0.7590 | 0.1654 | 0.1947 | 2,940 | 7,063 |

Bold = the metric the JDI board scores that benchmark on (macro_f1 for BANKING77/ANLI, accuracy elsewhere). `determinism_ok` **True on all 12** (the verbatim observed-repeat check) — the accuracy columns are load-INDEPENDENT (deterministic serve); only the latency columns carry the box-state caveat below.

## 3. Skill-level parity vs the board's own prints (the fold's per-column view)

The board's per-gid jdi_skill prints (0.4.1 snapshot `da1586d5`, pre-computed in rethink 032 as the expectations) vs this panel's same-formula skill:

| gid | benchmark | local jdi_skill | board print | Δ | verdict |
|---|---|---:|---:|---:|---|
| 4 | banking77 (macro_f1) | 0.9290 | 0.9383 | −0.0093 | inside the band |
| 28 | winogrande (acc) | 0.9204 | 0.9226 | −0.0022 | essentially exact |
| 57 | mmlupro (acc) | 0.7411 | 0.7183 | **+0.0228** | reads HIGH |
| 12 | anli (macro_f1) | 0.7554 | 0.7443 | +0.0111 | inside the band |
| 29 | hellaswag (acc) | 0.9680 | 0.9797 | −0.0117 | inside the band |

Five comparable columns, every one inside ±0.023. The systematic uncertainty of ANY our-fold-vs-board-print comparison is 0.0149 aggregate pts (Bench 072's OUR-SERVED bound) — all five deltas sit at or under twice that. The stress column (hellaswag, the highest bar — where the reference-GDN fallback, fla/causal-conv1d absent on macOS, was expected to stress hardest) reads −0.0117: the fallback is real but does not break parity. mmlupro's +0.0228 is the one above-board column — favorable direction, disclosed not explained (same-weights self-host reading above the board's own print; possible board-side posture deltas are the hosted run's to reconcile).

## 4. Box state + the contention disclosure (read beside every latency number)

- Panel START (banking77 meta, 2026-10-24T18:59Z): AC power, load 1.37, swap 21.6 GB (falling), `latency_quotable: true`.
- Panel END (typed_decisions meta): AC power, load 10.64, `latency_quotable: false` — `load 10.64 > 6 — a sibling job is on the box` (the harness's own refusal recorded in the meta).
- The sibling load: riir-train issue-628 delta-decay probes filled the box from ~05:30+07 (5–7 processes at 150–250% CPU each; load 13–14 for hours). The serve starved to ~5 req/min for ~2 h (from ~25) and recovered when probes batched. **The p50 columns are therefore contention-inflated and NEVER citable as mammoth-1 latency rows** — the board-parity and JDI claims rest on the accuracy/skill columns only, whose load-independence is evidenced by the 12/12 determinism pins.
- **The taskpolicy incident (self-reported):** at 09:19+07, while diagnosing the starvation, this session ran `taskpolicy -b -p <serve-pid>` — the WRONG direction (background = demote). The serve sat background-throttled (~4.8% CPU) for ~2 minutes until `taskpolicy -B` + `-t 0 -l 0` restored it (60% CPU at the next sample; request rate 5 → 21 req/min). Impact: ~2 minutes of extra-slow in-flight requests; zero data impact (requests are stateless, answers deterministic; the affected window's responses are part of hellaswag, whose determinism_ok is True). Lesson for the next operator: `taskpolicy -b` DEMOTES on this box's Darwin task policy — verify the direction before `-p` on a shared-box serve.

## 5. What this buys (and what it does not)

- **Buys:** (1) the LOCAL-FIRST parity reference — the hosted Baseten run (T10.1, owner key) now has five per-column Δ expectations to reconcile against, and the local arm is proven end-to-end on the exact wire the hosted posture drives; (2) the JDI rows for OUR lane — the rethink fold (bench 073) scores mammoth-1 through the same formula as every board entrant: **subset aggregate 86.98 / full-panel zeros-included 62.38** — the routed clef⊕pplx⊕mammoth lane reads **88.31 / 63.34**; (3) the teacher-capture premise (train-640) is de-risked: the local serve answers the wire correctly at parity for hours under adverse scheduling.
- **Does not buy:** a board RANK (5/37 rows served; public_skill stays the only self-servable aggregate, never the rank — the 032 edition law); the hosted-row substitution (a hosted run is a DIFFERENT row — transport hop + routing; never pooled with this one, the clef A0 law); any serving-side change (the stack is untouched — MEASUREMENT-ONLY, no RIDT arm).

## Provenance

- Harness: reflex release binary at worktree HEAD `53cf6f5` (the panel spans the `cb221da` opendecider-lane landing — the lane touches neither `mammoth.rs` nor the runner's mammoth path; diff-verified disjoint). Datasets: canonical `.raw/datasets`, blake3-pinned by `.docs/02_protocols/dataset_manifest.md` (typed_decisions test aggregate `01d24801…`, pinned 2026-10-25).
- Serve: `.raw/mammoth-local/serve_mammoth.py` (gitignored operator rig), bf16 onejev81ce62f weights, MPS, 55 GB-class RSS, loopback :8793, `MAMMOTH_FORWARDER_AUTH=1` bearer enforced.
- Watchdog: `/tmp/mammoth_watchdog.log` (boot 18:06:46Z, healthy 18:07:16Z); panel marker `/tmp/mammoth_panel_done.marker` (~12:20+07 2026-10-25); per-suite logs `/tmp/mammoth_panel_<s>.log`; serve log `/tmp/mammoth_serve.log` (11,000+ 200-OK POSTs, zero non-200s in the panel window).
- Fold: `rethink scripts/jdi_aggregate.py` over the pinned corrected command (032 T10.2; the two archived banking77 rows included) → rethink bench 073.
- Session: zed seal1 2026-10-25 (fold-readiness → fold session), epoch ~1792950000.
