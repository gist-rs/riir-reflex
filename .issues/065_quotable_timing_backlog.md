# Issue 065 — every comparison lane except openthai carries zero quotable latency, so the charts can never plot them

**Status:** OPEN — awaiting quiet-box re-run windows (filed 2026-10-03 from the reflex-site home-chart
quotable-latency work; ref reflex-site `6ed8f8a` — the home summary now plots quotable runs only, which made
this backlog visible as "named in the note, never plotted").

## Evidence — the published verdict counts (data/bench.json areas.timing, 2026-10-03)

| lane | clock | quotable cells | verdict breakdown | why |
|---|---|---|---|---|
| `bekko` | subprocess | **0 of 9** | 9 unfit | the run's own box state read NOT QUOTABLE (loaded box at run time) |
| `paw` | http | **0 of 9** | 5 unfit + 4 unjudged | mixed: some cells unfit (box state), some from a doc with no readable probes |
| `paw_local` (host `4090-win`) | local-runtime | **0 of 4** | 4 unjudged | the 4090 harness has no box-state probes |
| `clm@4090-win` | http | **0 of 8** | 8 unjudged | same — no probes on the 4090 |
| `gliner@4090-win` | subprocess-python | **0 of 8** | 8 unjudged | same |
| `agentjev@4090-win` | http | **0 of 8** | 8 unjudged | same |
| `openthai` | http | **9 of 9** | — (index clean) | its m3 index is fully quotable (Bench 085 re-read), BUT its 11 `@4090-win` extra-host cells are unjudged, and on 4 suites (typed, emotion, sst5, xnli) those unjudged cells outrank the m3 cells on accuracy — so the reflex-site home summary's pick drops those 4 suites from the openthai row (7/14) |

Consequence: the site's Issue-021 law ("unfit timing is shown in the tables, never plotted") is enforced on the
reflex-site home summary too — unfit cells never plot there, and since reflex-site `4b7fe65` a lane left with
nothing plottable keeps a PRESENCE row with the reason in place (bekko, currently) instead of vanishing, while
UNJUDGED cells (no probes) plot again marked ⚠ unverified and fallback cells plot as the served product (↩).
What remains true: none of these lanes' timing is verified, so the home chart's latency story for them is
thin-to-absent until the re-runs below land. openthai plots 11/14 with 4 of them unverified for the same reason
one layer down (unjudged 4090 cells winning the accuracy pick).

The publisher's walls are already in place and stay: a doc whose box state says NOT QUOTABLE refuses to publish
timing unless `PUBLISH_BENCH_ALLOW_UNQUOTABLE` names the host; an `:acc-only` suffix publishes accuracy while
stripping the latency fields; unjudged docs publish with a loud note. This issue is only about re-RUNNING the
timing when the box is quiet so the cells earn `latency_quotable: true` on their own.

## Tasks

- [ ] T1 Re-run `bekko` on a quiet M3: `scripts/bench_preflight.sh` green first (quote its `PROVENANCE:` line in
  the bench record), then `cargo run --release --bin harness -- --bekko --skip-laya --out .benchmarks/<n>_bekko_tables`.
  Expected: 9 cells flip unfit → quotable.
- [ ] T2 Re-run `paw` hosted + `paw_local` the same way (`--paw` with the tier env
  `PAW_COMPILER=paw-ft-bs48-20260530` + `PAW_COMPILE_ASYNC=1`, `--paw-local` with `PAW_LOCAL_PYTHON`) —
  preflight green, one window, both postures. Expected: 5 unfit hosted cells + the local posture cells → quotable
  (the 4 unjudged hosted cells become judgable the same way, since the harness itself probes the m3 box).
- [ ] T3 The 4090-hosted lanes (`clm`, `gliner`, `agentjev`, and the `openthai`/`paw` @4090 extra-host cells) are
  unjudged because the 4090 harness has no box-state probes — two roads, pick one (owner call recorded here):
  - [ ] (a) run the harness FROM the m3-max-metal against the 4090-served endpoints — the probes read the m3 box,
    the cells become quotable; caveat to disclose in the record: the p50 then includes the network hop (the
    clock class is `http` either way, the timing table already says which clock each number uses), OR
  - [ ] (b) port the box-state probes (`scripts/bench_preflight.sh` equivalent) into the 4090 runner so 4090-hosted
    runs stamp their own verdict — the honest long-term fix; the Windows-side probe set needs an owner pass.
- [ ] T4 After each window: publish the lane-scoped update (`PUBLISH_BENCH_LANES="<lane>" python3
  ../reflex-site/scripts/publish_bench.py <results.json> ../reflex-site` — the update path, never fresh-docs),
  re-run the reflex-site smokes (`chart_render_smoke.cjs` pins the p50-family counts — re-pin when the quotable
  verdicts move), commit + push both repos.
- [ ] T5 Close-out: when every row of the table above reads 100% quotable (or the lane is retired), fold the
  residual into HISTORY and remove this file.

## Non-goals

- The encoder (Rethink) cell verdicts are NOT this issue's — filed in `../riir-rethink/.issues/021` (the unfit
  typed read) and its fallback cells are the answering tier's clock by construction, never re-runnable as encoder
  timing.
- No change to the publish walls, the Issue-021 verdict rule, or the site's quotable-only plotting.
