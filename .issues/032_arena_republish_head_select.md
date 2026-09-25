# Issue 032 — republish the arena table at the `--head-select` posture (Bench 040's promoted arena protocol never reached reflex.gist.rs/bench)

**Status:** OPEN — filed 2026-09-25 at the Issue 030 close-out (the one unfinished item that issue carried). **The 4090 leg of task 1 LANDED 2026-09-26 (Bench 043, `.benchmarks/032_headsel_4090/`): full-harness `--head-select --skip-laya` at reflex `0418d33`, PASSED (the one absence = harness_cache_reuse, LLM-lane-only, expected), and the cross-host determinism gate PASS — 14/14 suites bit-identical accuracy vs Bench 040's M3 reference, banking77 0.6840 / massive 0.7933 confirmed on x86_64-Windows. Remaining: the M3 preflight-clean leg (the box carried a sibling's 8-core GPU run all session), the merged publish + CF deploy + live verify, README rows, close-out.** Nothing else landed yet.

## Finding

Bench 040 (`69a6eae`, Issue 030 lever 4) promoted `--head-select` as the
ARENA protocol (engine default stays `head_scale = 0`): banking77
0.4460 → 0.6840 (+23.8 pt), massive_intent_en 0.6900 → 0.7933 (+10.3 pt),
every other suite bit-identical or ECE-better. The one run published since —
Bench 041 (`ae80abc`, the T11 republish; site `b935412`) — ran at
`head_posture: "OFF (head_scale 0 — the published baseline posture)"` in
both `041_t11_m3_republish` and `041_t11_4090_modelless`. So the live
`/bench` modelless rows are still the pre-head numbers, understating the
modelless lane on exactly the two rows where it now beats laya-best.

Measured 2026-09-25: `grep head_posture` over both Bench 041 `results.json`
→ OFF; the live `reflex.gist.rs/data/bench.json` is byte-identical to
reflex-site HEAD, which carries that run.

## Tasks

- [-] Modelless-only `--head-select` run on BOTH hosts (m3 + 4090-windows),
      preflight-clean (`scripts/bench_preflight.sh`, PROVENANCE line quoted),
      never beside a sibling latency A/B. The modelless lane is
      deterministic — accuracy is the claim, so the two hosts must agree
      bit-for-bit on accuracy (the Issue 023 T5 pairwise drift gate).
      **4090 half DONE (Bench 043): 14/14 bit-identical vs the M3 b040
      reference. M3 half waits for the quiet box — the run must be
      preflight-clean because its cells join a merged publish (a loaded-box
      latency cell would poison the published table).**
- [ ] Publish through `../reflex-site/scripts/publish_bench.py` as a
      lane-update merge (laya / clm / gliner / agentjev cells carried
      untouched), with `RunMeta.head_posture` + the per-suite selected scale
      disclosed on the page.
- [ ] Manual CF deploy from the M3 (`npx wrangler deploy`, the standing
      manual-deploy posture) + live verify (`curl` the live `bench.json`,
      compare to site HEAD).
- [ ] README Results rows for banking77 / massive at the head-select numbers
      (labelled with the posture), Bench record for the republish, close this
      issue into HISTORY.md with the hashes.
