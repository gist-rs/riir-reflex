# Issue 037: the e8 table converter arm — riir-reflex half of riir-infer Plan 612

**Status:** OPEN — filed 2026-09-26 from the Plan 612 verdict (converter ownership: the offline quantizer + manifest rows + conversion log live in THIS repo; the runtime half lives in riir-infer).
**Filed from:** `../riir-infer/.plans/612_laya_ane_e8_table_stack.md` (Phase 1) + `../riir-infer/.research/002_FluidUse_e8_Int8_Embedding_Stack.md`
**Extends:** `.plans/002_ane_lane_bench.md` P0 (the converter this arm extends: `scripts/ane_convert.py` + `assets/ane/{manifest.json,conversion_log.md}`)

## Problem

riir-infer Plan 612 (the e8 int8-embedding table stack for the ANE lane) puts the
OFFLINE quantization in this repo — the conversion discipline lives here, and a second
converter would be a DRY violation. Plan 612's G3 ("the diff touches only `ane.rs` +
the converter") hid this repo's ownership; this issue is the named home for the
converter half.

## Tasks

- [ ] T1 — extend `scripts/ane_convert.py` with `--table-precision e8`: linear-symmetric int8 over `encoder.embeddings.tok_embeddings.weight` with **per-row (vocab-axis) f32 scales** (~256k scales ≈ 1 MB, outlier-isolated per token row — the axis decision and its rationale are pinned in Plan 612 Phase 0), emitted as `<ane_root>/<model>/table_e8.safetensors` (i8 tensor + scales tensor).
- [ ] T2 — manifest rows `<model>/table_e8` in `assets/ane/manifest.json` (BLAKE3 digest + shapes + scale dtype, same discipline as artifact rows) + one `conversion_log.md` entry per checkpoint documenting the refused-variant boundary (w8/w8e/w6/w4 fail ANE parity — FluidUse `@ 5beb3400` evidence, adopted as our refusal boundary).
- [ ] T3 — determinism golden: two consecutive runs produce byte-identical sidecars (closed-form min/max symmetric — assert, never assume).
- [ ] T4 — one sidecar per checkpoint (english / multilingual / typed), each serving every bucket of its checkpoint; no artifact reconversion (the six BC1S FP16 `.mlpackage`s stay byte-identical).

## Non-goals

- No runtime change in this repo — `gather_e8`, the `LAYA_ANE_TABLE=e8` env, and the
  G5-ANE re-pass are riir-infer Plan 612 Phases 2–3.
- No encoder-weight int8, no sub-8-bit palettes (the refusal boundary above).

## Reference

- Scheme pinned from mobius `models/computer-use/laya/coreml/quantize.py` @
  `5beb34007656d16349fec757c379a9beae45c4d5` (their published variant is per-channel;
  our sidecar defaults to per-row scales — Plan 612 Phase 0 reads their artifact's
  dequant axis as a PRIOR only, never as our spec).
