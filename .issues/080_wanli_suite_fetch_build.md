# Issue 080 — wanli_en suite: fetch lane, licence ledger row, harness wiring + first baseline

**Status:** Resolved — the reflex-side half of the ESC re-source landed in one
lane (fetch → ledger → wiring → Bench 130); the rethink-side ESC re-fit + GOAT
stays tracked in riir-rethink Issue 024.

## What this is

riir-rethink Research 002 verified WANLI (alisawuffles/WANLI, AI2) as the ONLY
clean re-source candidate for the licence-barred xnli_en niche (⛔ CC BY-NC):
CC BY at the primary host, direct 3-choice wire fit (premise/hypothesis/gold),
attribution Liu et al. 2022. This issue owns the reflex-side half:

- **Fetch lane** — `scripts/fetch_datasets.sh` row 11: `probe_splits` +
  `probe_license` (`alisawuffles/WANLI`, hub tag `license:cc-by-4.0` — the
  drift tripwire) + `fetch_suite` test WHOLE (5,000 rows — the eval cap lives
  in the harness, the banking77 posture) + train at TRAIN_CAP=20000 (the pool
  floor sits on this pull) + `dedupe_train` (0 exact-duplicate rows found).
- **Licence ledger** — `.docs/02_protocols/dataset_manifest.md` §Licences row
  (**CC BY 4.0**, verified AT SOURCE 2026-10-08: AI2's own page states
  "License: CC BY", HF tag `cc-by-4.0`; a hub-tag echo is NOT the
  verification) + §Suites 11 with all 251 file digests, and the seed-derivative
  caveat carried VERBATIM from Research 002 (GPT-3-generated, MNLI in-context
  SEEDS never copied; MNLI's own terms unstated at its primary page — the open
  question rides the row for the owner).
- **Harness wiring** — `SuiteSpec wanli_en` (named_only TRUE — opt-in via
  `--suites`, the thai_* precedent, never a default-run member; test_cap 300,
  cal_cap 200, corpus 64/label, coverage_audit on) + `build_wanli_en`
  (build_xnli_en's presentation template verbatim: XNLI_KEYS order, premise/
  hypothesis in the same template slots; WANLI's string `gold` maps through
  the key order, unknown strings drop) + `train_row_label`/`train_docs`
  arms (pair-as-unit corpus text, same as xnli_en) + the runner's
  option-key-union and expected-keys arms (wanli_en pins the SAME XNLI_KEYS
  set) + `slice_guard` floor row (16,000 — the same TRAIN_CAP=20000 pull
  shape as xnli_en: pool maxes 19,800, floor = ~80%) + `slice_leak`
  `eval_case_text` arm (pair-as-unit) + the t3 wiring test's `build_suite`
  arm.
- **First baseline** — `.benchmarks/130_wanli_en_baseline` (Bench 130):
  A0 modelless acc **0.3100** (300 cases, det ✓), G1 PASS (calibrated readout
  ECE 0.0871 < conformal floor 0.2921), slice-integrity OK (pool 19,800).
  A1/H2 intentionally absent — no wanli specialist exists; those cells are
  the rethink-side re-fit's product.

## What remains

- **riir-rethink Issue 024** owns the ESC re-fit: the specialist fit +
  fusion postures against this suite's floor, the T2 strict-superiority
  gate, and the served-arm decision. Nothing on the reflex side blocks it.
- The **open derivative question** (MNLI seed-influence) stays an OWNER
  call on the ledger row — recorded, not settled by assertion.
- `t3_runner_wiring_reproduces_the_probe_over_built_cases`
  (`--features slice_leak`) fails on **thai_sib200** (no `build_suite` arm)
  — PRE-EXISTING on develop (verified on clean HEAD 59b02c8; the suite's
  data landed 2026-10-02 and the opt-in test wasn't run since). NOT fixed
  here (unrelated failure, reported per the hygiene rule); wanli_en's own
  arm IS in place so the test's wanli_en leg works once thai_sib200 is
  adjudicated.
