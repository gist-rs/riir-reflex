# Bench 082 — the FULL harness on the 4090 at the CUDA posture: 15/15 PASSED, laya·typed acc byte-identical cross-host, 2.6× the M3 Metal row

**Verdict: PASS.** The Bench 081 weights sync unblocked more than the
fixture timing: with `.raw/datasets` also synced (16 MB, scp -r — see
provenance), the complete Plan-603-T1.5 harness ran on the 4090 with
both lanes at `LAYA_DEVICE=cuda`:

```
LAYA_DEVICE=cuda cargo run --release --features laya-riir-cuda --bin harness \
    -- --out .benchmarks/082_4090_cuda_full_harness
```

**PASSED — 15 suite(s), no absences**, exit 0, tables + results.json in
`.benchmarks/082_4090_cuda_full_harness/` (this dir; the canonical
M3 tables in `001_phase1_tables/` are untouched).

## Headline — the typed specialist row, cross-host

| | acc | macro F1 | ECE | Brier | p50 | p99 |
|---|---|---|---|---|---|---|
| M3 · laya-riir-metal (baseline `aa37823`, committed `001_phase1_tables/`) | 0.7445 | 0.7349 | 0.1924 | 0.4099 | 431.0 ms | 850.0 ms |
| **4090 · laya-riir-cuda (`97c2b1f`)** | **0.7445** | **0.7349** | **0.1924** | **0.4099** | **164.0 ms** | **294.0 ms** |

- **Accuracy byte-identical across host AND commit** — the same four
  metrics plus the choice/noul/score extras (0.7333/0.7850/0.7225 on
  both sides). The laya lane's determinism claim now has a cross-host
  cross-GPU cell: same picks, same numbers, different vendor backend.
- **Typed p50 164 ms vs the M3 Metal lane row 431 ms — 2.63×** (the
  committed table's comparison annotation separately cites a
  "1312 ms (m3 metal, that baseline)" figure from an older CPU-era
  reading; the lane row 431 ms is the same-table metal number).
- Per-question CUDA cost on the harness's real typed questions
  (~164 ms / 5 q ≈ 33 ms/q) is ~1.6× the Bench 081 fixture-timing row
  (20.2 ms/q) — the harness questions are longer than the parity
  fixtures; both are honest, differently-labeled distributions.

## Full-run shape

- All 15 suites both lanes clean; determinism ✓ on every row.
- Modelless G1 census this run: 7 PASS / 2 FAIL (massive, banking77 —
  the known narrow-suite cal-slice class) / the synthetic families
  NO-CLAIM, matching the addendum-7 census shape.
- The modelless typed row (0.3345) differs from the committed M3
  table's 0.3190 — EXPECTED, not drift: the committed table pins the
  `aa37823` (09-23) engine; Issue 038/039 knob and sampling work landed
  since. The extras (0.1717/0.5650/0.2838) ARE identical across the two
  tables. The laya rows are the stable comparison, per the lane's own
  frozen-teacher design.
- Header box-state line reads UNJUDGED (the Issue-021 probes are
  macOS-only) — supplemented here: RTX 4090, driver 610.62, desktop-only
  compute tenants, GPU boost clocks active during the run, no CUDA
  compute competitor (Bench 081 box state, same session).

## Findings landed with the record

- **The stale `PRE-MOVE BASELINE` header line is retired** (this
  commit, `runner.rs`): the 008 T4 move landed 2026-09-24, but the
  disclosure printed "scheduled to move" whenever the laya feature was
  on — the Bench 082 tables were the first to carry the stale label
  across hosts. The header now prints nothing for posture (the sha +
  device lines carry the identity); the cap line for PARTIAL runs is
  kept. No gate or script pinned the line (checked: tests/,
  scripts/, reflex-site publish tooling grep clean); lib 196/196 +
  clippy `-D` clean at the cuda posture.

## Provenance

- `.raw/datasets` sync to the 4090: 8 of 11 dirs were already there
  (a 09-26 session); `tar -xf` on bsdtar 3.8.8/exFAT silently skipped
  the remaining three dirs (`_reference`, `thai_sib200`,
  `thai_wisesight`) with a 0-byte error file and an empty-then-error
  exit-code trail — NOT diagnosed to root cause. **`scp -r` per-dir
  works** (14/12/45 files verified byte-count-identical) and is the
  recorded path for small dataset syncs to this box.
- The 4090's reflex sits at `97c2b1f` (Bench 081's bundle sync).
- Run identity from the tables header: `97c2b1f` on `unknown`
  (2026-09-28T07:13:04Z, UTC — the Windows host name does not resolve
  in the runner's probe; the box is the 4090). **Relabeled 09-28**:
  `meta.host` `unknown` → `4090-windows` (one line, the runner's own
  `REFLEX_BENCH_HOST` value per the issue-033 label law — the run
  missed the env; measured bytes untouched). The site publisher now
  REFUSES an `unknown` host so this class cannot mint a phantom host
  row silently; `REFLEX_BENCH_HOST=4090-windows` is REQUIRED on this
  box (`uname -n` unresolvable).
