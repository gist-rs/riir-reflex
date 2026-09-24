# Issue 013 — modelless accuracy levers (the honest-gap follow-up from the
# 15-suite python-reference run)

**Status:** OPEN — planning + bounded probes; implementation gated on probe
evidence (not owner-blocked: `do remains` instruction covers the filing; the
accuracy work itself starts only on a measured win).

## Context

Bench 001 Addendum 6 + the laya-python reference lane (issue 012, `67470be`)
settled the honest verdict: modelless is not a laya competitor on open-domain
accuracy (ag_news 0.510 vs 0.95, xnli 0.347 vs 0.86 — corpus-bound by design)
but is not useless — 3-5 orders of magnitude faster (0.06-0.5 ms vs 19-350 ms),
ties-or-beats laya on its own served domain (visibility 0.375 vs 0.3125;
tool_fit/permissions ties), and its calibrated abstention works (acc@50cov >
raw acc on most suites). This issue enumerates the levers if the owner wants
modelless accuracy UP, each with its measured starting point.

## Levers (in expected-yield order)

1. **[x] Bigger per-label corpora.** MEASURED 2026-09-24 — the premise
   inverts: the cap is a per-suite tuning knob, not a bigger-is-better
   lever. ag_news: peak AT the default 64 (.5100; declining after — 512 is
   −4.75 pp). banking77: interior peak at 128 (.5040, +6.6 pp over the
   default; saturation ≥256 reads .4560; acc@50cov +6.8, AURC −5.3,
   macro-F1 +3.3; ECE +6.6 WORSE, Brier/NLL flat; p50 0.50→0.875 ms).
   Deterministic: every reading byte-reproduced. Full tables + the
   promotion deferral: [.benchmarks/003_corpus_cap_lever.md](../.benchmarks/003_corpus_cap_lever.md).
   **Promotion of banking77 64→128 DEFERRED on a protocol hole:** the
   sweep reads the TEST split — shipping 128 ships a test-set-selected
   hyperparameter (the selection-biased-gain class). Unblock: cal-slice
   cap selection (cal-acc-per-cap runner plumb, pick on cal, report test
   once), then re-measure.
2. **[-] ROUTE_SCALE sweep.** The option-rank blend scale (`ROUTE_SCALE`,
   Issue 004 T7) is now a config knob (`EngineConfig.route_scale`, default
   8.0 unchanged) with a synthetic-family probe
   (`examples/route_scale_probe.rs`). **Probe run 2026-09-23 — FLAT within
   noise, promotion DECLINED on evidence:**

   | scale | visibility | permissions | tool_fit | routing | sensitivity |
   |------:|-----------:|------------:|---------:|--------:|------------:|
   |   2.0 |      0.375 |       0.417 |    0.583 |   0.438 |       0.400 |
   |   4.0 |      0.375 |       0.417 |    0.583 |   0.438 |       0.467 |
   |   8.0 |      0.375 |       0.417 |    0.500 |   0.438 |       0.400 |
   |  16.0 |      0.375 |       0.500 |    0.583 |   0.500 |       0.400 |
   |  32.0 |      0.375 |       0.500 |    0.333 |   0.562 |       0.200 |

   No scale dominates: 16.0's nominal mean is +2.6 pp over the default but
   that is 1-2 questions on these eval slices (5-16 cases/family),
   sensitivity reads best at 4.0, and 32.0 collapses tool_fit (-0.25) and
   sensitivity (-0.20). The GOAT shape (win at parity) is not met — the
   default stays 8.0. DEFERRED: a dataset-suite sweep through the harness
   (needs a `--route-scale` CLI plumb) — weak prior after this flat grid,
   pick up only if lever 1 also reads flat.
3. **[ ] More fitted heads (the Bench 881 precedent).** The Tetris game head
   (λ=1, 44/120 anchors) answered its served domain from a decoded head —
   the one place modelless accuracy is not corpus-bound. Candidates: the
   other lanes flagged in issue 011 (flappy unblock path), or decoding heads
   for the dataset suites' most-confused label pairs. Each head is its own
   freeze/thaw artifact + gate — never a runtime regression for the
   modelless default path.

## Non-goals

- No accuracy gating: accuracy stays RECORDED, published, never gated
  (the gates-test law — floors gate DOWNWARD only).
- No laya-lane changes; this is modelless-only.
- No training: any head decode is a freeze artifact consumed modellessly
  (the modelless-first mandate; training belongs to riir-train and is out
  of scope here).

## Acceptance

- Each lever lands with a measured before/after table in
  `.benchmarks/` (its own bench number, not this issue).
- A lever that measures flat/dead is recorded as such and checked off —
  a negative result on a lever is a result.
