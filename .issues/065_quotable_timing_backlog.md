# Issue 065 — every comparison lane except openthai carries zero quotable latency, so the charts can never plot them

**Status:** OPEN — awaiting quiet-box re-run windows (filed 2026-10-03 from the reflex-site home-chart
quotable-latency work; ref reflex-site `6ed8f8a` — the home summary now plots quotable runs only, which made
this backlog visible as "named in the note, never plotted").

## Evidence — the published verdict counts (data/bench.json areas.timing, 2026-10-03)

| lane | clock | quotable cells | verdict breakdown | why |
|---|---|---|---|---|
| `bekko` | subprocess | **9 of 9** ✅ (2026-10-03, bench 115 — T1 DONE) | — | landed via **per-suite docs** (each doc's span fits a baseline dip; the 15-min full-run shape cannot pass while the editor is alive — baseline drift 3.2→8.8 measured; banking77 landed via the `[banking77, typed, code_fixtures]` triple whose registry-order tail is code_fixtures' 20 s light run). p50 geomean 231.7 ms published; record `.benchmarks/115_bekko400m_timing/RECORD.md` (issue 067, closed with this T1) |
| `paw` | http | **3 of 9** (2026-10-04, bench 116 — partial: their hub outage) | 2 unfit + 4 unjudged | 3 quotable (prompt_injections, code_fixtures, xnli_en; ~1.0–1.2 s/q wall) landed before programasweights.com's `/api/v1/infer` went 502 on every program (~00:10 +0700, their-side outage). Remaining 6 re-run when healthy |
| `paw_local` (host `4090-win`) | local-runtime | **superseded by the m3 primary lane** — m3 **9 of 9** ✅ (2026-10-04, bench 116) | 0/0 on m3 | the m3 primary `paw_local` lane (p50 geomean 34.2 ms, all bundles cache-hit) replaced the @4090-win host-tagged rollup by design; the 4 4090 cells stay in the per-suite tables (unjudged, 065 T3) |
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

- [x] T1 Re-run `bekko` on a quiet M3: **DONE 2026-10-03 — bench 115, 9/9 cells quotable, published**. What actually worked was NOT one quiet 15-min window (none exists while the editor runs; two full-run attempts + the light-tail pairing went start-quotable → end-unfit) but per-suite docs with retries, plus the triple doc for banking77 (registry order puts banking77 last in ANY doc, so its tail is always the heavy suite — the triple ends on code_fixtures). **Priced in `.issues/067`** (filed 2026-10-03, closed same day with this T1): ≈21 min of lane wall (run 1 15.0 min + run 2 6.3 min, Bench 107's measured per-suite seconds); the gate that failed in Bench 107 was
  the box going LOADED mid-run after a green launch preflight (load 3.89 → 9.05), so the quiet-box check must hold
  at BOTH ends — 067 carried the verbatim re-run protocol; bench 115's RECORD carries the measured post-mortem and the doc shapes that pass.
- [ ] T2 Re-run `paw` hosted + `paw_local` the same way (`--paw` with the tier env
  `PAW_COMPILER=paw-ft-bs48-20260530` + `PAW_COMPILE_ASYNC=1`, `--paw-local` with `PAW_LOCAL_PYTHON`) —
  preflight green, one window, both postures. Expected: 5 unfit hosted cells + the local posture cells → quotable
  (the 4 unjudged hosted cells become judgable the same way, since the harness itself probes the m3 box).
  **PARTIAL 2026-10-04 (bench 116): paw_local 9/9 quotable on m3** (p50 geomean 34.2 ms — the m3 primary lane replaced the
  @4090-win host-tagged rollup by design; the 4090 cells stay in the per-suite tables). **Hosted 3/9** (prompt_injections,
  code_fixtures, xnli_en quotable, ~1.0–1.2 s/q wall) — then programasweights.com's inference endpoint went down mid-run
  (HTTP 502 `inference_failed` on EVERY program incl. previously-answered ones; site root still 200 — their-side outage,
  ~00:10 +0700). Remaining: emotion, ag_news, sst5, massive_intent_en, banking77, typed_decisions — re-run when their hub
  is healthy (the failed attempts' docs sit beside the good ones in `per_suite/`, the bench-115 pattern). One hosted acc
  drift disclosed: xnli 0.72 → 0.7233 (+1/300 — their endpoint evolved since Sept 29; honest dated reading).
- [ ] T3 The 4090-hosted lanes (`clm`, `gliner`, `agentjev`, and the `openthai`/`paw` @4090 extra-host cells) are
  unjudged because the 4090 harness has no box-state probes — two roads, pick one (owner call recorded here):
  - [ ] (a) run the harness FROM the m3-max-metal against the 4090-served endpoints — the probes read the m3 box,
    the cells become quotable; caveat to disclose in the record: the p50 then includes the network hop (the
    clock class is `http` either way, the timing table already says which clock each number uses), OR
  - [ ] (b) port the box-state probes (`scripts/bench_preflight.sh` equivalent) into the 4090 runner so 4090-hosted
    runs stamp their own verdict — the honest long-term fix; the Windows-side probe set needs an owner pass.
  **ASSESSED 2026-10-04 (bench 116 record): (a) has nothing to point at right now** — no lane servers (vLLM/uvicorn/python)
  running on the 4090 (checked ~00:2x +0700; GPU 0% util but 23.8 GB held by a sibling agent's plan-614 tests — standing
  servers up would fight sibling work for GPU memory). (b) stays owner-gated. Note the `paw_local`@4090 cells no longer
  roll up host-tagged (the m3 primary lane superseded them, bench 116) — the remaining @4090 extra-host rollups are
  clm/gliner/agentjev (+ openthai's 11 cells, which still win the accuracy pick on 4 suites and plot marked-unverified).
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
