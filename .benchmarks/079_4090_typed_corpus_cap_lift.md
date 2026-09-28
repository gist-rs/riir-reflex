# Bench 079 — Bench 078 cross-host verified on the 4090: typed modelless row 0.5725 byte-identical

**Verdict: PASS — cross-host reproduction.** The Bench 078 typed row
(m3) row re-measured on the second published host (4090, Windows, release
build at the same commit `2cbbce7`), same posture (registry defaults;
nb/oc/ridge cal-selected; heads off; genome off; `--skip-laya`):

| host | n | acc | macro F1 | ECE(maxp) | readout-ECE | det |
|---|---|---|---|---|---|---|
| m3 (Bench 078) | 2000 | 0.5725 | 0.5429 | 0.0993 | 0.0114 | ✓ |
| 4090 (Bench 079) | 2000 | **0.5725** | 0.5429 | 0.0993 | 0.0114 | ✓ |

Every headline metric is **identical to four decimals** — the
modelless lane's determinism claim holds across hosts: same inputs,
same deterministic selection laws — identical is the expected result,
and this is exactly the host pair where split-compare claims usually
break, which is why it is recorded. Per-kind rows identical too: choice 0.5483 / noul 0.7300 /
score 0.4725. Corpus-fallback guard: 0 (the security_incidents
self-doc line absent on both hosts).

Latency differs as expected (p50 0.815 ms m3 vs 1.280 ms 4090) — the
box-state line prints UNJUDGED on this host (the power/load probes are
not wired on Windows); latency columns carry no box state here and
stay unquoted.

## Provenance

- Corpus sync: the four new train pages scp'd from the M3's verified
  envelope (byte sizes identical; pages 000–007 already matched).
- Repo sync: `git bundle` (c464a8a..2cbbce7) scp'd + fast-forward — the
  4090 box's `git fetch origin` hangs (GitHub egress/credential stall,
  observed twice today); bundles avoid the egress path entirely.
- First launch attempt (detached via Start-Process) died silently after
  the suite banner — no output, no exit record; the foreground rerun
  completed clean (exit 0, PASSED, 16.1 s suite wall). Not
  investigated further: the foreground path is the record.

## Downstream

The reflex-site typed row can now republish citing BOTH hosts at
0.5725 (this run's TABLES.md + results.json ride beside Bench 078's).
