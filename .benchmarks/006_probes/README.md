# Bench 006 Addendum 4 — probe sources (archived, not built)

Measurement-only probes behind Addendum 4 (Issue 020 T9). Archived here so
the numbers stay reproducible. **None of this is a cargo target** (under
`.benchmarks/`, so it is out of cargo's auto-discovery).

To re-run, put each file back where it ran from:

| file | put it at | notes |
|---|---|---|
| `laya_seq_sweep.rs` | `examples/` | `#![cfg(feature = "laya-riir")]`; needs an `[[example]]` row with `required-features = ["laya-riir-metal"]` before it can be committed there |
| `laya_seq_sweep.py` | `scripts/` | the interleaved rust-vs-python driver (`REPO` = its parent's parent) |
| `ktime_sweep.py` | `.benchmarks/006_probes/` (as is) | imports `scripts/laya_seq_sweep.py` |
| `riir-infer-ktime-split.diff` | `git apply` in `../riir-infer` | `LAYA_SPLIT=1` encoder/head timers, plus `LAYA_KTIME=1`, which commits and waits **per dispatch**. That serializes the GPU and adds about 0.1–0.2 ms per op, so it's for attribution only and never gives a latency figure |
| `sgemm_shape_timing-m-sweep.diff` | `git apply` here | `SGEMM_M` length sweep |
| `torch_gemm.py`, `torch_attn.py` | anywhere | `uv run --no-project --with torch python …` |

Build and run in a DETACHED worktree with its own `CARGO_TARGET_DIR`
(Addendum 2's discipline), for example:
`LAYA_DEVICE=metal cargo build --release --features laya-riir-metal --example laya_seq_sweep`.

## Addendum 5 (T7 pick candidate + T10 rung 1)

- `riir-infer-gemm-pick-probe.diff` — probe-only env switches
  `LAYA_GEMM_FORCE`, `LAYA_XWIDE_N_MIN`, `LAYA_GEMM_WAVE` plus the KTIME/split
  counters, on top of riir-infer `0121a3b`; it ALSO carries the T10 rung-1
  kernel edit, which landed separately as riir-infer `0ec88a9`. Never commit
  the probe half.
- `harness_ab.sh` + `harness_ab_report.py` → `harness_ab.out` — the 6-round
  paired harness A/B of `LAYA_XWIDE_N_MIN=1024` (verdict: inside noise, not landed).
- `flash_par_patch.py` — the T10 rung-1 source transform (applied → `0ec88a9`).
- `flash_ab.py` → `flash_ab.out` — two-binary paired encoder A/B + serialized
  flash_attn per-kernel pass.
