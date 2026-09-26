# Issue 037: the e8 table converter arm — riir-reflex half of riir-infer Plan 612

**Status:** CLOSED 2026-09-26 — T1–T4 done (commit `6535b75`: converter `--table-precision e8` + manifest rows + conversion-log entries; 3 sidecars emitted local-only under the repo's artifact rule, `.gitignore` row added; determinism asserted in-run AND cross-run; the six `.mlpackage`s byte-identical throughout). The runtime half (`gather_e8`, `LAYA_ANE_TABLE=e8`, G5-ANE re-pass) stays riir-infer Plan 612 Phases 2–3.
**Filed from:** `../riir-infer/.plans/612_laya_ane_e8_table_stack.md` (Phase 1) + `../riir-infer/.research/002_FluidUse_e8_Int8_Embedding_Stack.md`
**Extends:** `.plans/002_ane_lane_bench.md` P0 (the converter this arm extends: `scripts/ane_convert.py` + `assets/ane/{manifest.json,conversion_log.md}`)

## Problem

riir-infer Plan 612 (the e8 int8-embedding table stack for the ANE lane) puts the
OFFLINE quantization in this repo — the conversion discipline lives here, and a second
converter would be a DRY violation. Plan 612's G3 ("the diff touches only `ane.rs` +
the converter") hid this repo's ownership; this issue is the named home for the
converter half.

## Tasks

- [x] T1 — DONE (`6535b75`): `scripts/ane_convert.py` `table` subcommand +
      `--table-precision e8`: linear-symmetric int8 over the pinned
      `model.safetensors` `encoder.embeddings.tok_embeddings.weight`
      (verified against the existing SHA-256 checkpoint pins — the table
      rides the safetensors, the mlpackages exclude it by design), per-row
      (vocab-axis) f32 scales, emitted as `<ane_root>/<model>/table_e8.safetensors`
      (hand-written safetensors container: i8 `table` + f32 `scales`, sorted
      tensor names, no `__metadata__` — the one nondeterminism smuggler).
      Closed-form `scale = float32(amax|row|)/127`, `q = clip(rint(w/scale),
      -127, 127)`; NaN/Inf rows and dtype/shape mismatch hard-refuse.
- [x] T2 — DONE: manifest rows `artifacts/<model>/table_e8` (BLAKE3 + shapes
      + scale dtype + quant scheme + the `serves_buckets: [64, 128]` row) +
      one `conversion_log.md` entry per checkpoint carrying the
      refused-variant boundary (w8/w8e/w6/w4 fail ANE parity — FluidUse
      `@ 5beb3400` evidence, adopted as our refusal boundary) and the
      axis-prior note (mobius is per-channel; ours is per-row on
      outlier-isolation grounds; flipping is one line + regen, decided by
      G1).
- [x] T3 — DONE: determinism golden two ways — every invocation runs TWO
      independent quantize passes and requires payload-byte-identity
      (assert, exit non-zero on mismatch), and the second RUN rewrites each
      sidecar only if its blake3 matches the fresh bytes (cross-run
      closed; all three reproduced).
- [x] T4 — DONE: one sidecar per checkpoint (english / multilingual /
      typed; L64 and L128 share it — the table is per-checkpoint, not
      per-bucket): multilingual `[256000, 768]` 197,632,160 B blake3
      `0276e83f…558ee`; english/typed `[50368, 1024]` 51,778,464 B blake3
      `ebaf93be…45c4d` / `1899dcdb…0f86e2`. Local-only (the 100 MB git
      per-file limit; the artifact rule) — `.gitignore` row added, digests
      pinned in the manifest. No artifact reconversion: the six BC1S FP16
      `.mlpackage`s byte-identical (digests verified before/after).

## Non-goals

- No runtime change in this repo — `gather_e8`, the `LAYA_ANE_TABLE=e8` env, and the
  G5-ANE re-pass are riir-infer Plan 612 Phases 2–3.
- No encoder-weight int8, no sub-8-bit palettes (the refusal boundary above).

## Reference

- Scheme pinned from mobius `models/computer-use/laya/coreml/quantize.py` @
  `5beb34007656d16349fec757c379a9beae45c4d5` (their published variant is per-channel;
  our sidecar defaults to per-row scales — Plan 612 Phase 0 reads their artifact's
  dequant axis as a PRIOR only, never as our spec).
