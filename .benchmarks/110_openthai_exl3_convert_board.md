# Bench 110 — the OpenThai-SystemOne EXL3 4.0-bpw convert board (riir-infer Plan 617 Phase A5): NO-GO by the pre-registered letter — PROVISIONAL (GPU-shared)

**Status:** RECORDED 2026-10-02 — the 4090 session's board runs land here as this
repo's record of the riir-infer lane; results committed under
`617_openthai_exl3_4090{,/retry,/thai}/` (named for the owning plan; these dirs
are the raw results homes, NOT bench allocations — this record is the one
`.highwater` allocation). **PROVISIONAL per the GPU-exclusivity rule's own
clause: the sibling riir-train CUDA training held the GPU (~100%, 12–16 GiB)
throughout — accuracy/ECE/determinism are contention-immune numerics, the
latency columns are UNQUOTABLE (Issue-021 law; TABLES.md itself prints `box
state UNJUDGED`). A quiet-window re-confirm is owed before any number is
published to the site.** The verdict is the owning plan's: **NO-GO — Phase B
(Metal serving arm) does not fire** (owner-delegated Claude verdict AGREE,
2026-10-02; the re-open is a NEW pre-registered decision — conditions below).

## What ran (the convert chain, all committed in riir-infer)

- Extraction surgery: 320 tensors renamed into the tower namespace,
  **320/320 byte-compare** (`fd6d7ed`); head trio split; tie/mtp divergences
  documented. exllamav3 1.5.3 KNOWS the Qwen3.5 GDN hybrid — only the
  `OpenThaiSystemOneForDecision` wrapper needed surgery.
- Clean convert @ 4.0 bpw (era 1.5.3), pack 920 MB (layers ~408 MB).
- **Per-pack bit-exact gate** (`real_pack_openthai_bit_exact_full`, `7d48248`):
  **151/151 layers · 751,435,776 weights · 0 mismatches**, 6.6 s — the evidence
  the era gate's known-good set was extended to `{"1.4.2","1.5.3"}` with
  (era-gate tests 10/10 incl. the refuse side).
- Serving: the fp32-parity hybrid server (`09b1eb2` + `6a55b71`) — their
  Formatter imported (byte-identical prompts by construction), head math
  verbatim, the AUTO permutation law (8 cyclic orders on ≥11-option choice)
  mirrored after root-causing the first attempt's 500s. Parity probe vs the
  fp32 server: **6/6 top-1, max prob delta 0.008–0.074, mean L1 0.089** — the
  healthy 4-bpw signature.
- Board: reflex harness @ `f730497`, `REFLEX_BENCH_HOST=4090-windows`,
  determinism pin ✓ (×2 byte-compare) on every row.

## The board (11 comparable suites; fp32 pins = 084's own 4090 table)

| suite | exl3 acc | fp32 pin | dAcc | dECE | acc bar (|d|≤1.0) | ECE bar (d≤+0.05) |
|---|---:|---:|---:|---:|---|---|
| typed_decisions | 0.5260 | 0.5360 | −1.00 | +0.0078 | AT BAR (20 q of 2000) | PASS |
| ag_news | 0.9050 | 0.8900 | **+1.50** | −0.0152 | **BREACH (improvement dir)** | PASS |
| emotion | 0.5900 | 0.5950 | −0.50 | +0.0031 | PASS | PASS |
| sst5 | 0.4233 | 0.4333 | −1.00 | +0.0094 | AT BAR (3 q of 300) | PASS |
| prompt_injections | 0.6379 | 0.6293 | +0.86 | −0.0142 | PASS | PASS |
| banking77 | 0.6500 | 0.6540 | −0.40 | +0.0223 | PASS | PASS |
| code_fixtures | 0.5625 | 0.5938 | **−3.13** | +0.0477 | **BREACH (1 q of 32)** | PASS (hair) |
| xnli_en | 0.9000 | 0.9000 | +0.00 | +0.0013 | PASS | PASS |
| massive_intent_en | 0.9167 | 0.9200 | −0.33 | −0.0134 | PASS | PASS |
| thai_wisesight | 0.4675 | 0.4675 | +0.00 | +0.0041 | PASS | PASS |
| thai_sib200 | — | 0.8382 | ABSENT | — | see below | — |
| semantic_defects | 0.4118 | — (no fp32 pin; beats modelless 0.1863) | — | — | unpinned | — |

## The verdict, and why the letter stands

- **The pre-registered mixed-outcome rule refuses the post-hoc suite-scoped
  carve-out BY NAME** — ag_news's breach is the IMPROVEMENT direction, but on a
  lossy surface (+1.5 is still divergence from the fp32 reference; the two-sided
  bar is the correct retention gate), and code_fixtures is ONE question at
  n=32 (granularity 3.125 pt vs the bar's ≤0.8 pt noise premise — a
  pre-registration defect for this suite, explainable-as-noise only AFTER
  seeing it).
- **GO is unreachable even with both breaches excused:** the bar needs ALL
  suites to pass, and 2 of 13 are unmeasurable today — sib200 (the slice-integrity
  floor mispin, fixed this session — see HISTORY; source yields 701 train rows,
  pool-after-cal 501, the old floor 560 was derived from the raw pull) and
  semantic_defects (born after the 084 pin run; carries no fp32 pin).
- **typed_decisions −1.00 at n=2000 is the most statistically meaningful
  at-bar cell** (20 net questions) — not obviously noise the way one-of-32 is.
- ECE bar PASS 11/11 (largest +0.0477, code_fixtures); determinism green
  everywhere.
- Economics unchanged (the owning plan's lane verdict): EXL3's axis is MEMORY
  not throughput; the only consumer was an M3-resident reflex comparison lane
  (the Rethink product posture already declined on three stacked grounds); the
  pack buys 0.4 GB vs 1.6 GB bf16.

**Re-open conditions (recorded with the owner-delegated verdict — a NEW
pre-registered decision):**
1. The bar moves to **per-suite paired discordance** (exact binomial / McNemar
   on per-question fp32-vs-EXL3 flips, pre-registered) — fixes the
   code_fixtures granularity problem without a special case and asks the right
   question of typed_decisions.
2. **sib200 measured** after the floor fix, and the suite population re-pinned
   to today's harness (11 comparable + semantic_defects once it carries an
   fp32 pin) so "ALL N" names a set that exists.
3. The re-open is **tied to an actual consumer need** for the 0.4 GB
   M3-resident footprint — never to the gate fix or a quiet GPU window alone.

## Box state

PROVISIONAL (above). The sibling riir-train training (plan435/607 lineage) held
the GPU before, during, and after the run; the determinism pin (×2
byte-compare) is green on every row, so the accuracy/ECE readings are
contention-immune by construction. Latency: do not quote.

## Artifacts

- `.benchmarks/617_openthai_exl3_4090/` — main board (results.json + TABLES.md)
- `.benchmarks/617_openthai_exl3_4090/retry/` — massive_intent_en + banking77
- `.benchmarks/617_openthai_exl3_4090/thai/` — the thai pair

## Cross-refs

- riir-infer Plan 617 (`A5` verdict §) + Issue 034 — the owning lane
- riir-infer commits: `fd6d7ed` (surgery) · `7d48248` (pack gate + era set) ·
  `09b1eb2`/`6a55b71` (server + parity probe) · `038f984`/`972546d` (A6/A5 verdicts)
- Bench 084 — the fp32 pins the bar measured against (same box)
- `src/harness/slice_guard.rs` — the sib200 pool floor, re-based this session
  onto the audited pool-after-cal (the mispin this board surfaced)
