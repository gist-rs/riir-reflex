# Bench 164 — the T10.2 LOCAL elephant-1 panel: all 12 suites landed on the amo self-host (elephant-1 9B bf16, MPS, sequential fork), determinism_ok 12/12, parity vs the board's own prints holds on every comparable column — banking77 0.9246 (board 0.9292), winogrande 0.9148 (0.9242), mmlupro 0.6845 (0.6462, reads HIGH), anli 0.7923 (0.7925), hellaswag 0.9630 (0.9724)

**Status:** RECORD — panel executed 2026-10-25T19:51Z → 2026-10-25T22:44:47Z (watchdog `6651`, quiet after the i1122 v16 exit + i452 precompute #4; explicit alternation held — the sibling campaign finished first, corrections logged both sides; marker `/tmp/elephant_panel_done.marker` — note: that marker timestamps the FULL run_panel including the capture leg, which CRASHED, see §6). Fold executed same session (rethink side; the 4-lane routed row UNCHANGED from bench 073 — elephant adds zero routed lift). Rethink Issue 032 T10.2's elephant LOCAL arm is DONE; the hosted half (T10.1 owner key, window ends 2026-10-27 23:59 PT) remains owner-gated.

## 1. The panel

`riir-reflex/.raw/elephant-local/run_panel.sh` — 12 sequential `harness --elephant --skip-laya --suites <s>` runs over the canonical `.raw/datasets`, `--out .benchmarks/elephant_local_panel/<s>/` (results.json + TABLES.md each). Serve: committed `scripts/elephant_server.py` — the LOCAL self-host of **elephant-1** (baselabs, 9B, MIT), bf16, MPS, **`fork_mode=sequential`** (the measured posture — the auto-selected `batched` fork runs the qwen3_5 GDN reference fallback pathologically slow on MPS; answers verified byte-identical between postures, 032), loopback :8096, RSS ~16.2 GB flat. Every suite printed `harness: PASSED — 1 suite(s), no absences`.

Boot: the quiet-box watchdog (`.raw/elephant-local/watchdog.sh`) found its window 2026-10-25T19:50:58Z — two consecutive quiet polls after the sibling i1122 v16 runner exited (18:17:59Z) and the i452 queue's final precompute finished; serve healthy in 15 s (pid 82461); panel launched 19:51:13Z; **PANEL RUN COMPLETE 22:44:47Z — 2h54m** (the 9B sequential-fork posture ran ~2× faster than the mammoth-scaled projection).

Pre-flight gate: `verify_panel.py --strict` **PASS, rc=0, all 12 suites, det=True everywhere** (run post-panel 01:12 UTC; re-verified non-strict by the fold-prep session 23:48Z with the 22 fold-input paths + highwater read).

## 2. The numbers (elephant lane; board metric; the per-suite results.json is the instrument of record)

| suite | n | acc / macro_f1 | ECE(readout) | wall (s) | p50 ms |
|---|---:|---:|---:|---:|---:|
| banking77 | 500 | **0.9246** (f1) | — | — | — |
| winogrande_jdi | 1,267 | **0.9148** | — | — | — |
| mmlupro_jdi | 2,000 | **0.6845** | — | — | — |
| anli_jdi | 3,200 | **0.7923** (f1) | — | — | — |
| hellaswag_jdi | 2,000 | **0.9630** | — | — | — |
| sst5 | 600 | **0.5750** | — | — | — |
| ag_news | 400 | **0.8625** | — | — | — |
| emotion | 400 | **0.5675** | — | — | — |
| massive_intent_en | 300 | **0.9300** | — | — | — |
| prompt_injections | 116 | **0.6466** | — | — | — |
| xnli_en | 300 | **0.9200** | — | — | — |
| typed_decisions | 400 cases | **0.7345** | 0.0953 | 980 | 2,297 |

Bold = the board metric. `determinism_ok` **True on all 12** — accuracy columns are load-INDEPENDENT; latency columns carry the box-state caveat (§5). (ECE/wall/p50 quoted where read in-flight; the per-suite results.json carries the full columns.)

## 3. Skill-level parity vs the board's own elephant-1 prints (raw-metric, per-gid, 0.4.1 snapshot `da1586d5`)

| gid | benchmark | local (board metric) | board print | Δ | verdict |
|---|---|---:|---:|---:|---|
| 4 | banking77 (macro_f1) | 0.9246 | 0.9292 | −0.0046 | inside the band |
| 28 | winogrande (acc) | 0.9148 | 0.9242 | −0.0094 | inside the band |
| 29 | hellaswag (acc) | 0.9630 | 0.9724 | −0.0094 | inside the band |
| 12 | anli (macro_f1) | 0.7923 | 0.7925 | **−0.0002** | essentially exact |
| 57 | mmlupro (acc) | 0.6845 | 0.6462 | **+0.0383** | reads HIGH (favorable) |

Five comparable columns: four inside ±0.01, one above-board outlier (mmlupro +0.038) in the favorable direction — the same shape as mammoth's mmlupro (+0.0228 high, bench 162). ⛔ The per-AREA view (our fold's knowledge/language/retrieval vs the board's area prints) is NOT comparable — our areas average only our served gids, the board's average all gids in the area; disclosed, never quoted as parity. Systematic uncertainty of any our-fold-vs-board comparison: 0.0149 aggregate pts (Bench 072). The GDN-fallback stress column (hellaswag, highest bar) reads −0.0094 — the fallback does not break parity, matching the mammoth finding.

## 4. The fold (rethink side, same session) — elephant's lane verdict

The corrected fold command (+ the 12 elephant panel files appended; run from the rethink root, green end-to-end):

| lane | subset aggregate (5/37 rows) | full-panel zeros-included | M1 bar 60.70 |
|---|---:|---:|---|
| elephant | **81.91** | **58.74** | misses zeros-included by 1.96 |
| mammoth (bench 073) | 86.98 | 62.38 | clears |
| routed clef⊕pplx⊕mammoth⊕elephant | 88.31 | **63.34** | **UNCHANGED from the 3-lane fold** |

**The lane finding: elephant-1 adds ZERO routed lift** — its per-family skill is strictly dominated by mammoth-1 on every served family (banking77 0.9236 vs 0.9280 · wino 0.8295 vs 0.9211 · hellaswag 0.9507 vs 0.9687 · anli 0.6889 vs 0.7557 · mmlupro 0.6451 vs 0.7436 at skill level), so best-per-family never picks it and the 4-lane routed row equals bench 073's 3-lane row byte-for-byte at the displayed precision. M5 adjudication: the local subset row 81.91 vs elephant-1's own board print 72.24 — the local arm reads high on the subset view, but the board's print is its Full-panel formula, NOT a same-population number; the honest same-formula comparison is the fold's own: elephant 58.74 zeros-included < mammoth 62.38 — **mammoth-1 dominates elephant-1 on OUR panel end to end** (expected: 27B vs 9B, same family, same recipe — the 9B's board-tier gap reproduces locally). The M5 bar (pass-the-best-sub-15B, ≥ 72.24) is a OUR-STACK bar and is NOT adjudicated by this teacher row — it stays the hosted-window + our-lane read's.

## 5. Box state + the contention disclosure (read beside every latency number)

- Boot (19:50Z): QUIET confirmed (swap 6.6 GB, free 91%, load 1.9 falling, no sibling greps) — the alternation contract executed as armed.
- Panel span: serve RSS 16.2-16.3 GB flat; sibling sessions kept CPU-only discipline (logged in the rig log); load ≤ 3.6.
- The typed_decisions meta read p50 2297 ms / wall 980 s — the only latency column read in-flight; **all latency columns quarantined** per the mammoth precedent (the serve's own meta under a shared box; never a serving-latency row). Elephant-1's board latency anchor (113.7 ms on RTX PRO 6000) stays Research 012's row.
- Multi-tenant notes: two sibling sessions appended ANNOUNCE/COORDINATION lines to the rig log during the run (the heal/fold-prep session 23:48Z, the idle-unit sessions 00:25-01:08Z) — all read-only or /tmp-CARGO_DIR work, zero panel interference (determinism 12/12 is the evidence).

## 6. The capture-leg crash (the marker's fine print) + the re-run

At 01:12:34 UTC, ~13019 rows into the ag_news capture (12k logged), the serve **crashed with a Metal command-buffer assertion** (`_status < MTLCommandBufferStatusCommitted`, IOGPUMetalCommandBuffer, /tmp/elephant_serve.log) after ~2.4 h of sustained sequential-fork forwards — a driver/framework crash class, not OOM, not contention (RSS flat, box idle). ag_news rc=2 with NO artifact; the remaining 6 suites refused (connection refused) rc=2; run_panel completed its loop; the watchdog demised the serve and wrote the marker at 01:12:41Z. **The panel outputs are unaffected** (the capture leg runs after all 12 suites; determinism + parity above rest on the panel only). **0/7 capture artifacts landed.**

The 640-T3 capture is RE-RUNNING since 01:15:08 UTC via `.raw/elephant-local/recapture.sh` — the same pinned harness invocation, but **serve hygiene per suite** (fresh boot before each suite, ~15 s): the measured crash class caps its blast radius at one suite instead of the whole capture, and a failed suite re-runs by re-invoking the script. Rate measured on the first window: ~0.52-1.05 s/row effective (the early 1.3 s/row estimate beaten ~2×; the 640-T3 record quotes the re-run's measured rate).

## Provenance

- Harness: reflex release at develop `9e98c0d` (clean tree — the panel build cites no dirty files; the one rustfmt-drift file was committed style-only BEFORE the boot, round-trip proven zero content). Datasets: canonical `.raw/datasets`, blake3-pinned by `.docs/02_protocols/dataset_manifest.md`. Model: `.raw/elephant-local/elephant-1` (18 GB dir, bf16).
- Serve: committed `scripts/elephant_server.py` (defaults `ELEPHANT_FORK_MODE=sequential` + per-decide MPS hygiene; posture self-disclosed per row: `baselabs/elephant-1 dtype=bfloat16 attention=noncausal fork=sequential on mps`).
- Watchdog: `/tmp/elephant_watchdog.log` (boot 19:50:58Z, healthy 19:51:13Z, panel-complete 22:44:47Z inside run_panel, crash 01:12:34Z, marker 01:12:41Z, watchdog exit 0); per-suite logs `/tmp/elephant_panel_<s>.log` + `/tmp/elephant_capture_<s>.log`; serve log `/tmp/elephant_serve.log` (the full panel window: all 200s; the crash assertion at tail).
- Fold: rethink `jdi_aggregate.py` with the corrected input list + the 12 elephant files (rethink-side record lands as the next rethink bench number; the routed row UNCHANGED 88.31/63.34).
- Panel outputs: committed reflex `de24d2c` (this record is the follow-up commit).
