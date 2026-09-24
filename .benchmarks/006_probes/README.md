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
