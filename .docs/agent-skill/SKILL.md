---
name: reflex-integration
description: Integrate the riir-reflex local decision engine over its HTTP wire — install the binary, ask typed questions, wire abstention into a cascade, fit confidence thresholds from labeled data, and feed outcomes back for online calibration. Use when a task says decide, classify, route, triage, prioritize, spam, guardrail, approve, escalate, abstain, or decision engine — even when reflex is never named.
---

# reflex — a decision engine for your coding agent

reflex is a single binary that serves a typed decision engine on loopback HTTP.
You POST a state plus questions; it answers with a chosen option, a rubric
level, or a yes/no — or it **abstains**, which is a first-class answer, not an
error. The default lane is modelless (µs-tier, bit-deterministic, zero weights,
no network). Everything below is the HTTP wire only — that is the public
surface; the Rust crate is private and never required.

Skill version 1 · 2026-09-22 · tested against `riir-reflex` 0.1.1 (release set
`modelless+laya`) · wire examples captured live against the binary at commit
`2c6acb9`.

## Install this skill for your agent

Claude Code reads `.claude/skills/` natively; Zed reads `.agents/skills/`.

```sh
# Claude Code
mkdir -p .claude/skills/reflex-integration
curl -fsSL https://reflex.gist.rs/skills/reflex-integration/SKILL.md \
  -o .claude/skills/reflex-integration/SKILL.md

# Zed (and any agent that accepts a markdown instruction file)
mkdir -p .agents/skills/reflex-integration
curl -fsSL https://reflex.gist.rs/skills/reflex-integration/SKILL.md \
  -o .agents/skills/reflex-integration/SKILL.md
```

## 1. Install the engine

```sh
# macOS / Linux (curl)
curl -fsSL https://raw.githubusercontent.com/gist-rs/reflex/main/install.sh | sh

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/gist-rs/reflex/main/install.ps1 | iex

# Homebrew (macOS / Linux)
brew tap gist-rs/tap && brew install riir-reflex

# Scoop (Windows)
scoop bucket add gist-rs https://github.com/gist-rs/scoop-bucket && scoop install riir-reflex
```

Then check the build stamp — it prints the compiled feature set and refuses to
hide an incomplete one:

```console
$ riir-reflex --version
riir-reflex 0.1.1
compiled features: default modelless laya
```

A build missing the shipped release set prints a `release set: STALE — missing
…` line with the rebuild command. Treat a STALE stamp as "not the shipped
binary" — re-install or rebuild before citing any measured number.

## 2. Run it

Bare invocation serves; there is no daemon framework, no config file:

```sh
riir-reflex            # serves http://127.0.0.1:7331
RIIR_REFLEX_BIND=127.0.0.1:9400 riir-reflex   # override host:port
```

Three routes:

| Route | What it does |
|---|---|
| `POST /decide` | `DecisionRequest` JSON → `DecisionResponse` JSON |
| `POST /feedback` | `{"p": 0.74, "outcome": true}` → `{"refit": false}` — online calibration |
| `GET /healthz` | liveness (`ok`) |

CORS is closed by default — no browser page can reach the engine. Agents on
loopback need nothing; open `RIIR_REFLEX_ALLOWED_ORIGIN` only when a web page
must drive the engine.

## 3. The wire

The answer space is part of the REQUEST — options are defined at request time,
not pretrained. Three question kinds:

- `choice` — multiple choice over `options` (≥2)
- `score` — ordinal rubric; `options` are the levels, LOWEST first
- `noul` — typed yes/no; `options` must be empty/absent

A real request (captured verbatim against the shipped binary's demo corpus):

```sh
curl -s -X POST http://127.0.0.1:7331/decide \
  -H 'Content-Type: application/json' -d '{
  "state": "The staging deploy of release candidate 4.2 finished but the error budget is down to 12 percent and two health endpoints are flapping after the rollout. Rollback is one command. Decide what happens next.",
  "questions": [
    {"id": "route",    "kind": "choice", "prompt": "Route this ticket to the team that owns it.", "options": ["deploy-ops", "billing-support"]},
    {"id": "promote",  "kind": "noul",   "prompt": "Should the rollout be promoted to production right now?"},
    {"id": "severity", "kind": "score",  "prompt": "How severe is this situation?", "options": ["routine", "needs-attention", "critical"]}
  ]
}'
```

The real response (fresh engine, no feedback yet):

```json
{
  "answers": [
    {"question_id": "route",    "outcome": null, "probabilities": [0.52583224, 0.47416782], "confidence": 0.0019263625},
    {"question_id": "promote",  "outcome": null, "probabilities": [0.5135249],               "confidence": 0.00052791834},
    {"question_id": "severity", "outcome": null, "probabilities": [0.36018527, 0.2878931, 0.35192162], "confidence": 0.0043790936}
  ],
  "routing":     {"lane": "modelless", "reason": "modelless corpus routing: ops=3 support=0; fused abstain (score+distance) armed"},
  "calibration": {"method": "none", "temperature": 1.0}
}
```

Read it like this:

- `outcome` — `null` means **abstained**. Non-null spellings:
  `{"choice": {"index": 0}}` (index into `options`), `{"score": {"level": 1}}`
  (0 = lowest), `{"noul": {"yes": true}}`.
- `probabilities` — per-option calibrated probabilities in option order
  (`noul` carries exactly one, p(yes)). Emitted even when abstaining — that is
  the signal your cascade reads.
- `confidence` — the calibrated confidence readout your threshold gates on.
- `routing.reason` — why the lane was picked; includes the per-domain corpus
  hit counts (`ops=3 support=0` above).
- `calibration` — what calibration produced these numbers.
  `method: "none"` + `temperature: 1.0` is the honest RAW posture; it changes
  as your feedback refits the calibrator (measured: after one refit the demo
  engine reported `method: "sigmoid-gate", temperature: 0.306`).

## 4. Asking well

- **Front-load the state.** The state is the decision's universe — put the
  facts and constraints in it, not just a subject line. The corpus gates read
  the state; a bare "fix this" starves them.
- **Options are the contract.** Write them as the answers you would actually
  act on, mutually exclusive, in the order you index them back.
- **`criteria` shapes choice.** `{"criteria": "Ownership is decided by which team's runbook the situation matches."}` rides on `choice` (and may carry the rubric on `score`) and is worth one extra field — phrase the discriminator, not the question, again.
- **Wording moves results — this is measured, not folklore.** Renaming options,
  reordering a rubric, or rephrasing a prompt changes the numbers. When accuracy
  matters, try 2–3 phrasings on a labeled slice and keep the measured best.

## 5. Abstention is the integration

`outcome: null` is a first-class answer. Wire it as a branch, never an error:

```
answer = decide(state, questions)
if answer.outcome != null and answer.confidence >= my_threshold:
    act on answer.outcome
else:
    fallback   # human review, a heavier model, or the safe default action
```

Two gates are in play and you own one of each side:

1. **The engine's gate** — the fused score+distance abstain. Out of the box the
   demo engine abstains nearly everything (the honest posture: the measured
   birth constants abstained 64–100% on the Phase-1 real-corpus suites —
   default thresholds do not transfer; that result is why §6 exists).
2. **Your gate** — the threshold you fit on `confidence` in YOUR integration.
   This is the one that tunes your risk–coverage tradeoff.

The shipped binary carries a two-domain demo corpus (deploy-ops and
billing-support routing) so the whole pattern above runs end-to-end today.
The engine's competence is **corpus-scoped**: far off-corpus it abstains rather
than guessing (measured: a sourdough-support ticket against the demo corpus →
abstain at confidence 0.00027, probabilities still near-uniform). Serving a
custom corpus is the in-repo harness lane, not a shipped wire surface — measure
before promising competence on your domain.

## 6. Fit thresholds from data (do not guess them)

The measured lesson, twice over: default thresholds abstain 64–100% on real
suites; thresholds fitted on one task do not transfer to another. Fit yours:

1. Run `/decide` over a **labeled calibration slice** of your task (≥16 cases —
   below that, no fit; the engine's own floor is `THIN_SUPPORT_FLOOR = 16`).
   The slice must be data your corpus has not memorized — never calibrate on
   rows that are inside the reference corpus.
2. Collect `(confidence, was_the_top_answer_correct)` per case.
3. Sort the confidences ascending and take the **ρ-quantile** as your
   threshold, ρ = 0.30 to start (the Phase-1 arena posture: ~30% of
   in-distribution questions abstain). `confidence >= threshold` passes.
4. Disclose the support with every number you report: `n` cases the fit saw,
   `n_pass` at or above, `n_abstain` below.

This reproduces the engine's own fit surface (`engine::threshold_recommendation`,
landed `a732bcf`) over the wire: same quantile law, same thin-support null
(fewer than 16 observations → no recommendation exists — say so, don't
improvise one), same support disclosure. The target-accuracy alternative is the
other posture that surface ships: the LOWEST cutoff whose at-or-above accuracy
on the slice meets your target (e.g. 0.90) — pick max answered volume at target
quality; when nothing meets it, report the best-achievable cutoff as unmet,
never silently.

Track the tradeoff as you move ρ or the target: answered volume vs accuracy on
the answered set. That is the whole dial.

## 7. The feedback loop (online calibration)

Every decided case with a known ground truth is calibration evidence:

```sh
curl -s -X POST http://127.0.0.1:7331/feedback \
  -H 'Content-Type: application/json' \
  -d '{"p": 0.74, "outcome": true}'
# {"refit": false}
```

- `p` is **the engine's own confidence (or winning probability) for that case**,
  `outcome` is whether the answer was right. Send the engine's number back —
  never an invented one. Measured failure mode: feeding arbitrary p values
  flipped `refit` at the 64-observation floor and COLLAPSED the calibrated
  confidences to ~1e-10 (the calibrator believed your lies). Garbage feedback
  is worse than no feedback.
- `refit` flips `true` once the calibrator holds ≥64 observations
  (`cal_min_obs`); before that every report returns `{"refit": false}` and
  changes nothing.
- After a refit the `calibration` metadata changes (`method`/`temperature`) —
  read it per response; it tells you whether you are consuming raw or
  calibrated numbers.

The loop is: decide → record confidence → observe ground truth → feedback →
refit → better-calibrated confidences → your §6 threshold keeps its meaning.

## 8. Choose your lane

- **modelless (default)** — µs-tier per decision, bit-deterministic (the same
  request returns byte-identical `probabilities` — verified on repeat probes),
  zero weights, no network, corpus-scoped competence. This is the lane the
  shipped binary serves.
- **laya (opt-in)** — the accuracy heavyweight — the laya decision model
  (a ModernBERT-large encoder port) with downloadable weights (default cache
  `~/.cache/riir-reflex/laya`). Any laya number you cite must be preceded by
  its parity gate (top-1 agreement vs the reference checkpoint) — a lane that
  has not passed parity is PROVISIONAL, and its numbers are not citable.
- The response's `routing.lane` tells you which lane served (`modelless` |
  `laya` | `hybrid`). Scope every latency/determinism claim to the lane that
  produced it.

## 9. Traps we already paid for

1. **Default thresholds do not transfer.** Birth constants abstained 64–100%
   on real suites. Fit from data (§6) — there is no dignity in rediscovering
   this one.
2. **`ECE 0.000` can mean "no data", not "perfect".** A lane that abstained
   everything collapses confidence to exactly 0 and falls into no calibration
   bin (left-open bins). Read it as n/a, never as perfect.
3. **Wording moves results.** Re-phrasing a prompt or renaming options changes
   the numbers; measure phrasings against a labeled slice before believing one.
4. **Front-load the state.** Thin states starve the corpus gates; the facts
   belong in `state`, not in your head.
5. **Feedback lies collapse confidences.** Send back the engine's own number
   for the case (§7) — measured: invented p values took the demo engine's
   confidence from 1e-3 to 1e-10 in one refit.
6. **A `none` calibration is honest, not broken.** `method: "none"`,
   `temperature: 1.0` means you are consuming the raw posture; feed the loop
   (§7) before publishing accuracy claims.
7. **Empty `questions` is legal.** `{"state": "", "questions": []}` validates
   and answers `[]` — request validation is per-question; a 400 means
   malformed JSON or a question-level violation (empty prompt, `noul` with
   options, `< 2` options on choice/score, duplicate ids).

## 10. Measured basis (do not cite unmeasured)

Modelless lane, Phase-1 harness (run `86e3727`, 2026-09-22, release profile,
M3, in-process — the HTTP hop is cold-path JSON on top):

| Suite | acc | abstain raw/cal | p50 / decision | p99 (tail support) | det |
|---|---|---|---|---|---|
| typed_decisions (2000 q) | 0.319 | 0.82 / 0.93 | 0.488 ms | 1.753 ms (5) | ✓ |
| ag_news (400 q) | 0.510 | 0.49 / 0.26 | 0.151 ms | 0.171 ms (5) | ✓ |
| emotion (400 q) | 0.283 | 0.62 / 0.36 | 0.116 ms | 0.139 ms (5) | ✓ |
| sst5 (600 q) | 0.217 | 0.57 / 0.32 | 0.097 ms | 0.115 ms (7) | ✓ |

Calibration honesty (G1, readout ECE vs the conformal-naive floor): PASS on
typed_decisions, emotion, sst5; **FAIL on ag_news** (calibrated 0.380 vs floor
0.251). Some suites fail the calibration floor — measure YOUR task and publish
the loss, don't average it away. The arena publishes every number above with
its losses at <https://reflex.gist.rs/#bench>.

refit floor 64 observations · thin-support floor 16 · ρ = 0.30 arena posture —
all engine constants, all cited in their sections above.

## Freshness

This skill carries the version stamps at the top. The curl one-liner for THIS
file is re-verified on every release-matrix cut (fetch what we claim to fetch —
the v0.1.0 nested-archive lesson). If `riir-reflex --version` reports a newer
engine than the stamp above, re-check the measured table at
<https://reflex.gist.rs/#bench> — numbers there regenerate from harness runs;
the ones here are the snapshot the skill was tested against.
