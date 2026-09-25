# Issue 032 — republish the arena table at the `--head-select` posture (Bench 040's promoted arena protocol never reached reflex.gist.rs/bench)

**Status:** OPEN — filed 2026-09-25 at the Issue 030 close-out. **2026-09-26 UPDATE: the publish is RE-SHAPED by the owner (option 3) — the head-select accuracy publish is DEFERRED until preflight-clean FULL-lane re-runs exist for both hosts, because a modelless-only update wipes the host's incumbent laya + comparison lanes in publish_bench.py's host-keyed containers (measured on four dry-runs: 49 lanes → 14; the whole finding + plan filed as `.issues/034_publish_bench_lane_replace.md`).** What DID land 2026-09-26: the 4090 leg re-run at HEAD with the leak scan (Bench 044, `.benchmarks/044_headsel_4090_leak/`) — 14/14 accuracy bit-identical vs BOTH Bench 043 and the Bench 040 M3 reference (three-run, two-host facts), `head_posture` label fixed (d727196), Issue 024's 4090 leak columns closed by the same run; plus the additive publisher laws in reflex-site (LANE_FACT_META_KEYS + LANE-CARRY, pinned 17/17) that will keep the FUTURE publish honest. Remaining = Issue 034's task list (the two full-lane re-runs + the publish + deploy + README rows + close-out). (Bench 043 `.benchmarks/032_headsel_4090/` stays as the first 4090 leg record; its stale posture label is superseded by 044.)

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
      **4090 half DONE twice (Bench 043 at 0418d33, re-landed as Bench 044 at d727196 with the fixed `head_posture` label + `--features slice_leak`): 14/14 bit-identical vs the M3 b040 reference both times. M3 half waits for the quiet box — the run must be
      preflight-clean because its cells join a merged publish (a loaded-box
      latency cell would poison the published table); take it with
      `--features slice_leak` so it also carries Issue 024's M3 leak
      columns in the same pass.**
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
