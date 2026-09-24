# ANE conversion log (Plan 002 P0)

Appended by `scripts/ane_convert.py` per run. Artifacts themselves are local-only (gitignored); this log + `manifest.json` are the committed record.
## Smoke — multilingual — 2026-09-24 10:51 +07
- fixtures: ch5-mixed-crit#q0, sc5-levels#q0, noul-structured#q0, state-mask-literal#q0, ch2-email-triage#q0, ch3-dict-crit#q0, ch1-single-option#q0, ch10-bucket-6-10#q0
- reference: frozen G5 goldens (`tests/fixtures/laya_parity_expected_v1.json`, corpus blake3 f2a00354ef554f36…); head = fp32 torch mirror of `src/laya/riir/head.rs`; encoder = the ANE artifact
- structural parity (markers/seq_len/bucket/temperature): exact on all 8
- top-1 agreement: **8/8** · max prob err **0.0200** · max act err 0.0000
  - ok `ch5-mixed-crit#q0` n=53 L64 k=5 choice:3-5 arg 0->0 margin=0.5752 err=0.0024
  - ok `sc5-levels#q0` n=63 L64 k=5 score:3-5 arg 2->2 margin=0.2423 err=0.0038
  - ok `noul-structured#q0` n=59 L64 k=2 noul:2 arg 1->1 margin=1.0000 err=0.0000
  - ok `state-mask-literal#q0` n=56 L64 k=2 choice:2 arg 0->0 margin=0.8908 err=0.0005
  - ok `ch2-email-triage#q0` n=84 L128 k=3 choice:3-5 arg 2->2 margin=0.9030 err=0.0012
  - ok `ch3-dict-crit#q0` n=89 L128 k=3 choice:3-5 arg 0->0 margin=0.3751 err=0.0200
  - ok `ch1-single-option#q0` n=67 L128 k=1 choice:2 arg 0->0 margin=1.0000 err=0.0000
  - ok `ch10-bucket-6-10#q0` n=111 L128 k=10 choice:6-10 arg 2->2 margin=0.9851 err=0.0003
- near-tie band: golden margin < 0.04 (issue 017 tolerance 0.02 × 2); flips listed above, never hidden
- box: {"machine": "arm64", "macos": "26.6.2", "load1": 7.02, "load5": 12.04, "load15": 16.14, "power": "Now drawing from 'Battery Power'", "free_ram_gb_approx": 30.0}

## Smoke — english — 2026-09-24 10:52 +07
- fixtures: ch5-mixed-crit#q0, ch1-single-option#q0, sc5-levels#q0, noul-structured#q0, ch2-email-triage#q0, ch3-dict-crit#q0, ch12-bucket-11plus#q0, ch10-bucket-6-10#q0
- reference: frozen G5 goldens (`tests/fixtures/laya_parity_expected_v1.json`, corpus blake3 f2a00354ef554f36…); head = fp32 torch mirror of `src/laya/riir/head.rs`; encoder = the ANE artifact
- structural parity (markers/seq_len/bucket/temperature): exact on all 8
- top-1 agreement: **8/8** · max prob err **0.0077** · max act err 0.0000
  - ok `ch5-mixed-crit#q0` n=46 L64 k=5 choice:3-5 arg 3->3 margin=0.3800 err=0.0017
  - ok `ch1-single-option#q0` n=59 L64 k=1 choice:2 arg 0->0 margin=1.0000 err=0.0000
  - ok `sc5-levels#q0` n=52 L64 k=5 score:3-5 arg 3->3 margin=0.1521 err=0.0015
  - ok `noul-structured#q0` n=60 L64 k=2 noul:2 arg 1->1 margin=0.6206 err=0.0012
  - ok `ch2-email-triage#q0` n=80 L128 k=3 choice:3-5 arg 0->0 margin=0.2880 err=0.0020
  - ok `ch3-dict-crit#q0` n=84 L128 k=3 choice:3-5 arg 0->0 margin=0.9992 err=0.0001
  - ok `ch12-bucket-11plus#q0` n=123 L128 k=12 choice:11+ arg 2->2 margin=0.7254 err=0.0058
  - ok `ch10-bucket-6-10#q0` n=101 L128 k=10 choice:6-10 arg 0->0 margin=0.5584 err=0.0077
- near-tie band: golden margin < 0.04 (issue 017 tolerance 0.02 × 2); flips listed above, never hidden
- box: {"machine": "arm64", "macos": "26.6.2", "load1": 5.98, "load5": 11.29, "load15": 15.73, "power": "Now drawing from 'Battery Power'", "free_ram_gb_approx": 28.9}

## Smoke — typed — 2026-09-24 10:52 +07
- fixtures: ch5-mixed-crit#q0, ch1-single-option#q0, sc5-levels#q0, noul-structured#q0, ch2-email-triage#q0, ch3-dict-crit#q0, ch12-bucket-11plus#q0, ch10-bucket-6-10#q0
- reference: frozen G5 goldens (`tests/fixtures/laya_parity_expected_v1.json`, corpus blake3 f2a00354ef554f36…); head = fp32 torch mirror of `src/laya/riir/head.rs`; encoder = the ANE artifact
- structural parity (markers/seq_len/bucket/temperature): exact on all 8
- top-1 agreement: **8/8** · max prob err **0.0146** · max act err 0.0000
  - ok `ch5-mixed-crit#q0` n=46 L64 k=5 choice:3-5 arg 3->3 margin=0.1218 err=0.0009
  - ok `ch1-single-option#q0` n=59 L64 k=1 choice:2 arg 0->0 margin=1.0000 err=0.0000
  - ok `sc5-levels#q0` n=52 L64 k=5 score:3-5 arg 3->3 margin=0.2867 err=0.0035
  - ok `noul-structured#q0` n=60 L64 k=2 noul:2 arg 0->0 margin=0.2352 err=0.0006
  - ok `ch2-email-triage#q0` n=80 L128 k=3 choice:3-5 arg 0->0 margin=0.0827 err=0.0012
  - ok `ch3-dict-crit#q0` n=84 L128 k=3 choice:3-5 arg 0->0 margin=0.5780 err=0.0004
  - ok `ch12-bucket-11plus#q0` n=123 L128 k=12 choice:11+ arg 2->2 margin=0.5173 err=0.0146
  - ok `ch10-bucket-6-10#q0` n=101 L128 k=10 choice:6-10 arg 2->2 margin=0.0737 err=0.0022
- near-tie band: golden margin < 0.04 (issue 017 tolerance 0.02 × 2); flips listed above, never hidden
- box: {"machine": "arm64", "macos": "26.6.2", "load1": 5.46, "load5": 10.84, "load15": 15.46, "power": "Now drawing from 'Battery Power'", "free_ram_gb_approx": 28.0}

## Conversion — 2026-09-24 10:47–10:52 +07 — all six artifacts, T0.2/T0.3 GREEN

- Tool: `scripts/ane_convert.py convert` (coremltools 9.0, torch 2.7.0, Python 3.12 via uv;
  Python 3.12 REQUIRED — coremltools' compiled compute-plan bindings load only there; on the
  default 3.14 interpreter the `_MLComputePlanRemoteProxy` silently fails and placement
  verification would be blind).
- Gate (T0.3): `MLComputePlan` over the COMPILED artifact at `ComputeUnit.CPU_AND_NE` —
  100% of device-bearing ops ANE-preferred AND 0 device transitions, else the tool REFUSES
  (exit non-zero, no manifest entry). The ALL posture is recorded beside it as observation.

| artifact | device ops | ANE | transitions | size | blake3-dir-v1 |
|---|---|---|---|---|---|
| multilingual/L64 | 1373 | 1373 | 0 | 210.8 MiB | b46a95d1d66bc19e2682878b8d7801e2895798beb54d468d979d087c1ae1fe16 |
| multilingual/L128 | 1387 | 1387 | 0 | 210.9 MiB | 2e4d47246130a5b3a5619c31b5d800b57d308811faae31a11cdc8602d1d1deeb |
| english/L64 | 1745 | 1745 | 0 | 655.1 MiB | 7a56479c4c17b353aac30a027b317cc758a5542d64684ae86e79abe4a13f8478 |
| english/L128 | 1763 | 1763 | 0 | 655.2 MiB | 1b50ccf4ba6c0d8bf07fd4dd5dc9598bce32df62cd682067854249c64a31d0f2 |
| typed/L64 | 1745 | 1745 | 0 | 655.1 MiB | 0b1d0302eb69a992e83c02eee6f1e5117584841e5b3a0b9815f7a3b62a800d66 |
| typed/L128 | 1763 | 1763 | 0 | 655.2 MiB | 01085c6c3ae7e007a0549665acdf60bcf8052099b0a2e95b0572ca73ca45a5de |

- L128 = L64 + one `slide_bias` add per sliding layer (multilingual 14, english/typed 18):
  the sliding-window mask is constant-folded per bucket, built only when window < L-1 (the
  Rust lane's own mask-skip rule — at L64 the window covers the bucket, no mask exists).
- Design finding (measured, drove the artifact shape): the vocab-sized embedding gather
  (`ios16.gather`) is the ONE op the ANE compiler will not place — first conversion attempt
  read 1373/1374 ANE with the gather on CPU and the tool REFUSED it per the gate. The artifact
  shape moved the gather HOST-side: the artifact input is the pre-gathered fp16 `embeddings`
  [1,L,d] (a bit-exact row copy from the safetensors table — the P1 Rust runtime holds that
  table resident anyway), and 100% of the remaining graph places on the ANE. Re-converted;
  every artifact above is the post-fix shape.
- Honest debugging note: before the fix, a draft invocation also contained an argument-parsing
  bug (unsplit `--buckets` string) that produced two bogus `L6`/`L4` artifacts (sequence
  length 6 and 4). They passed nothing, were never manifest-pinned, and were deleted before
  any commit. No artifact above derives from that run.
- Weight source: the repo-pinned safetensors (SHA-256 verified before conversion —
  english `891102d3…`, multilingual `9d628fd9…`, typed `4fa56de7…`), fp16 mapped verbatim.
- Box state at conversion: M3 Max, macOS 26.6.2, arm64; **AC/BATTERY: the runs drew from
  BATTERY power** (loadavg 7–36 across the runs, sibling agents compiling concurrently;
  free RAM ~30 GiB). Placement verdicts are correctness facts and do not depend on load;
  no latency number is recorded here (that is P1's serialized bench).
