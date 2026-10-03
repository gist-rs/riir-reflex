# Bench 116 — paw timing re-run: paw_local 9/9 quotable on m3; hosted 3/9 (their hub outage mid-run)

**Status: MEASURED — paw_local: all 9 m3 cells `latency_quotable: true` (p50 geomean ≈ 33 ms, first quotable paw_local lane on the board). Hosted: 3/9 cells quotable (prompt_injections, code_fixtures, xnli_en) before programasweights.com's inference endpoint went down (HTTP 502 `inference_failed`, every program, site root still 200 — their-side outage, ~00:10 +0700); the 6 remaining hosted suites are 065 T2's open tail — re-attempt #2 (bench 117) found it still down and left the faithful health probe (`scripts/paw_hub_probe.sh`).**

Date: 2026-10-04 · host m3-max-metal · reflex `0dc2c20` · publish: reflex-site lane-scoped update (data/bench.json, `PUBLISH_BENCH_LANES="paw,paw_local"`, current bench.json as primary — the bench-115 update path)

## Box state — the session gate

Session preflight at 23:50 +0700: `PROVENANCE: power=AC Power load=2.95 swap=29459.75M canary=141.5us/best5 powermode=2(high)` — PASSED (a genuinely quiet window; load stayed 1.7–4.9 across every doc below). Every published doc carries its own start+end box state, both judged quotable.

## What this run measured (the bench-115 pattern, applied to the paw family)

1. **The paw family is network-bound, not CPU-bound** — unlike bekko's CPU-heavy banking77 lesson, every paw doc's end load was AT OR BELOW its start (self-load ≈ 0). Per-suite docs passed first-try all evening; the bekko-style baseline-drift lottery never bit.
2. **paw_local bundles were already warm for ALL 9 suites on m3** (the bench-090 preload warmed more than the 090 four) — total local sweep ≈ 3.5 min for 9 suites (longest: banking77 20.9 s).
3. **Their hosted inference endpoint went down mid-session**: prompt/code_fixtures/xnli answered (~1.0–1.3 s/q wall), then EVERY warmup infer 502'd (`inference_failed / Please try again`) — first at emotion, then ag_news, then a direct curl of a previously-working program. The site root still 200s. Outage, not rate-limiting (we made ~800 requests over ~20 min, then polls only).

## paw_local — 9/9 quotable (the m3 primary lane; all bundles cache-hit)

| suite | acc | p50 | wall | note |
|---|---|---|---|---|
| prompt_injections | 0.6379 | 24.4 ms | 4 s | == hosted reading (identical picks) |
| code_fixtures | 0.59375 | 36.8 ms | 3 s | refusals 1/32 (same as hosted) |
| ag_news | 0.7925 | 31.1 ms | 19 s | vs hosted board 0.79 |
| emotion | 0.4900 | 28.5 ms | — | |
| sst5 | 0.3983 | 27.1 ms | — | |
| xnli_en | 0.7133 | 35.9 ms | — | −1.0 pt vs hosted (the posture delta, bench-090 profile) |
| massive_intent_en | 0.5133 | 32.2 ms | — | +0.3 pt vs hosted |
| banking77 | 0.4180 | 39.0 ms | 20.9 s | refusals 177/500 (its known sampled-presentation refusal class; 4090 cell: 0.412) |
| typed_decisions | 0.5955 | 65.2 ms | — | vs hosted 089: 0.5925 |

p50 geomean ≈ **33 ms** — the local runtime is ~30–40× faster than the hosted round trip (24–65 ms vs ~1044 ms wall per question).

## paw hosted — 3/9 quotable before the outage

| suite | acc | server p50 | wall p50 | vs board |
|---|---|---|---|---|
| prompt_injections | 0.6379 | 76.9 ms | 1043.9 ms | == board acc |
| code_fixtures | 0.6250 | 79.2 ms | 1048.0 ms | == board acc (refusals 1/32 both) |
| xnli_en | 0.7233 | 96.1 ms | ~1.2 s | **+1/300 vs board 0.72** — their endpoint drifted one question since Sept 29; honest new dated reading, disclosed |

Remaining (blocked by their outage, 065 T2 tail): emotion, ag_news, sst5, massive_intent_en, banking77, typed_decisions.

## 065 T3 assessment (recorded, owner-gated)

- **(a) harness-from-m3 against 4090-served endpoints**: nothing to point at — no lane servers (vLLM/uvicorn/python) running on the 4090 (checked 2026-10-04 ~00:2x +0700; GPU 0% util but 23.8 GB held by a sibling agent's `plan614_phase3_floorgap` tests). Standing them up would fight sibling work for GPU memory; the timing would also carry the network hop (disclosed caveat in the issue text).
- **(b) port the box-state probes into the 4090 runner**: "the Windows-side probe set needs an owner pass" — explicitly owner-gated in the issue.

The (a)/(b) pick is the owner call the issue names; this session records the state and leaves both roads open.

## Publish (T4)

`PUBLISH_BENCH_LANES="paw,paw_local"` with the current data/bench.json as primary + the 12 docs as extras (the bench-115 update path). The filter dropped the 24 out-of-scope control slots (modelless+laya declarations) loudly — by design. Landed:
- `areas.timing.paw` = `{n_used: 3, n_unquotable: 2, n_unjudged: 4, p50_geomean_ms: 1046.65}` — the lane's first plottable cells (was 0/5/4).
- `areas.timing.paw_local` = `{n_used: 9, n_unquotable: 0, n_unjudged: 0, p50_geomean_ms: 34.15}` — the m3 primary lane; the `paw_local@4090-win` host-tagged rollup retired by design (the primary has measured it; the 4 4090 cells stay in the per-suite tables).
- The paw TIMING_METHOD string updated ("accuracy cells only today" was stale — paw now publishes latency; `--rederive` applied, cells byte-identical).
- Smoke re-pins: `chart_render_smoke` sz-break signs 2 → 3 (paw's ~1.1 s hosted aggregate crosses the 500 ms break); `bench_page_smoke`'s data-derived break-count derivation EXTENDED to count paw/paw_local cells (it predated their visible latency; the 6 acc-only cells carry no latency fields so the null check excludes them naturally). Publisher self-test 84/84; `home_page_smoke` + `public_copy_gate` green as-is.

## Flags (both postures)

`--nb-select --oc-select --ridge-select` (the board-canonical selection flags; modelless control rows byte-identical to the published board on every doc — the drift guard's premise). Hosted: `PAW_COMPILER=paw-ft-bs48-20260530 PAW_COMPILE_ASYNC=1` (054 law). Local: `PAW_LOCAL_PYTHON=.raw/paw-env/bin/python` + `PAW_COMPILER` (disambiguates the 2 cached program variants per shape — the lane refuses without it).
