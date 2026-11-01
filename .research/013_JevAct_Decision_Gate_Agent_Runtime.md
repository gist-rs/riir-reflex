# JevAct — Decision-Gate Runtime for LLM Agents: the Placebo-Arm Lesson for Our Gates

> **Source:** Shoham, David, Akrabi, Bilgory, Levy, Marreed, Zeltyn, Shlomov (IBM SIL)
> [arXiv:2610.38620 "Think Before You Act: Decision Gate for LLM Agents"] — 2026-10-28,
> CC BY-SA 4.0. PDF extracted at `katgpt-rs/.raw/` (removed after commit).
> **Date:** 2026-11-01
> **Status:** Done — **Gain**: measurement-methodology adoption, two issues filed
> (refine escalation-chain placebo arm; rethink agreement-replay VoI proxy —
> mechanics live in those private repos, this note stays architectural).
> No new primitive; the decision-model half of this family is saturated (562/573/574/576/579, reflex 002/005/006/008).
> **Related Research:** katgpt-rs 562 (Jev distill — the decision model this paper's judge clones),
> 573 (open decision-head recipe — Kev-27B is a Qwen3.8-27B fine-tune, i.e. our league backbone),
> 579 (the harness-engineering synthesis — this paper is the RUNTIME-GATE half of that map),
> reflex 002 (AgentJev landscape — its consumer surface already shipped a Claude Code
> PreToolUse gate hook: blocks on score level 3 AND boolean-unsafe, fail-open when down),
> 005/006/008/010 (the Jev serving lanes), rethink 007 (Jeva teacher — private)
> **Classification:** Public

## TL;DR

JevAct puts a **selective verification gate between an agent's proposal and its
execution**: offline, an LLM compiler turns a domain policy into executable
state checks + yes/no questions + Boolean rules + hand-off checklists; online,
only consequential proposals (write calls, hand-offs) are intercepted, checked
by code (facts from tool records) plus a **prefill-only decision model** (Kev —
an open Jev-type model, Kev-27B = Qwen3.8-27B fine-tune, used WITHOUT further
training), and flagged proposals get **bounded revision** (R=2 writes / R=1
hand-offs) with **temporary feedback that never enters the history**.

For the stack this is the third independent validation of the prefill-only
decision-serving posture (after 562's product distill and reflex 002's
landscape), and it carries ONE genuinely new, actionable item: the **placebo
ablation**. The paper's Table 3 shows a "flag every write once, check nothing"
control matches ReAct exactly for GPT-4.1 (50.0 = 50.0) — and worse, for
gpt-oss-120b **random flags at the gate's rate produce as much gain as the full
gate** (32.0 → 42.0 pass^3 both ways). Whether a gate's JUDGMENT matters beyond
the revision opportunity it grants is an empirical property of the consumer —
so any gate feature's A/B must carry a judgment-free placebo arm at the same
trigger rate before a gain is attributed to the gate's intelligence. The
stack's code-healer ships this exact architecture (executable verify + bounded
revert loop + a candidate-selection judge layer in a consumer stack) — that
measurement discipline is filed healer-side (private issue).
Second adoption: the paper's **judge-agreement replay** (609 frozen hand-off
states replayed; Kev-9B agrees with Kev-27B on 97.0%, Kev-0.8B on 90.6%) is a
receipts-free robustness probe any escalation lane can run without outcome
labels — filed decision-family-side (private issue). Everything else is
validation of shipped design, itemized below.

## The paper's numbers that matter

| Finding | Number | Read |
|---|---|---|
| Telecom pass@3, 8 agents averaged | +16.0 pts (all 8 improve) | the hand-off check targets premature give-up — the dominant failure mode there |
| Airline write-gate ablation (GPT-4.1) | ReAct 50.0 → full gate 72.0; **code checks alone 68.0**; judge questions alone 57.1 | executable checks carry most of the gain; the judge adds ~4 pts on top |
| **Placebo: every write flagged once, no check** | **50.0 = ReAct** (GPT-4.1); for gpt-oss-120b **random flags at gate rate = full gate** (32→42) | the revision OPPORTUNITY can be the whole effect; judgment attribution needs the placebo arm |
| Judge cost | Kev-27B $0.0003–0.0017/task vs API judges $0.0127–0.0231 (7–40×) | prefill-only serving economics, third independent confirmation |
| Judge size | NOT monotonic: Kev-0.8B best for Opus 4.6 on Telecom (96.5 pass^1 vs 93.6 for 27B); 9B≈27B on 97.0% of replayed states | capacity ≠ verification quality; context length + thresholds dominate; per-suite/per-consumer selection law |
| Asymmetric thresholds | write gate: answer = "no" only when P(yes) ≤ 0.05 (default-allow); hand-off step OPEN iff P(required) ≥ 0.5 ∧ P(settled) ≤ 0.5 | asymmetric cost matching: cheap side = allow, expensive side = flag+revise; conjunctive two-sided rule for checklist items |
| Coverage caveat (their own) | 42/101 incorrect calls break NO compiled question | the questions encode the POLICY, not the user's request — the relevance gap, third domain to hit it |
| Trigger selectivity | flag rate < 50% of write calls; 0.03–0.38 flags/task on Airline | selective interception, not blanket gating |
| Compile-time filters | questions the judge answers confidently-wrong on >10% of dev calls are DROPPED; checklist steps flagged >5% in solved episodes dropped | prune the question set by downstream judge reliability, label-free |

## Coverage map (signal-diffed, not name-matched)

| JevAct component | Ships here? | Diff |
|---|---|---|
| Prefill-only judge, fixed questions, probabilities in one shared pass | **SHIPS** — `decision_wire` choice/score/noul + per-option probabilities + `decide_multi` batching (562 distill → reflex engine); the paper is consumer-side validation of the serving posture | none — this is the third external confirmation |
| Executable state checks before commit | **SHIPS (healer shape)** — the healer's compile-gated verify (apply → re-check → auto-revert breaking edits) IS a write gate over code edits; riir-ai shipped typed decision gates at agent-loop route points (`987f2e787`, ex-1005) with bounded-state serialization | the paper's checks compile from a written POLICY doc; ours are hand-authored oracles (compile, clippy re-run). A policy→check COMPILER is the one piece nowhere in the stack — 579 recorded the DSL class as adjacent-only. Fusion idea, no consumer today |
| Bounded revision budget + temporary feedback | **SHIPS (shape)** — the healer's revert loop is budget-bounded; feedback-in-history discipline matches scoped re-checks (only touched packages re-verified, not the world) | same shape, different domain |
| Selective trigger (write calls + hand-offs only) | **SHIPS** — escalation lanes trigger on gate flags, not on every decision; rate guard latches | validated |
| Hand-off check (premature-termination detection; "never discussed ≠ ruled out") | **GAP-shaped, no consumer** — no stack surface detects "giving up too early" (the healer has no notion of premature fix-acceptance beyond the oracle re-run; NPCs don't hand off). The unclosed-world checklist discipline ("a condition never discussed is NOT ruled out") is the transferable half | recorded; reopen trigger = any surface that terminates attempts |
| Judge-agreement replay (receipts-free robustness probe) | **NOT SHIPPED anywhere** — closest is per-suite winner selection (served == best measured), which needs outcome labels; the replay probe needs none | **ADOPTED** → decision-family issue (private) |
| Placebo-arm gate evaluation | **NOT SHIPPED anywhere** — measured-negative precedents exist (set_refine Bench-150-class refusals) but no judgment-free placebo-at-same-rate arm convention | **ADOPTED** → healer-side issue (private) + the general law below |
| Question-set pruning by judge reliability (label-free, dev-log-derived) | **SHIPS (class)** — corpus-cap levers + per-suite calibration fits + corpus generations regenerated against the score-bench | same mechanism, corpus domain |

## The general law (the durable takeaway)

**A gate's A/B is not complete without a judgment-free placebo at the same
trigger rate.** A gate feature grants a revision opportunity (a second attempt,
a bigger model, a re-check round); that opportunity alone can produce the entire
measured gain, and WHETHER it does is a property of the consumer (the paper
measured both outcomes across two agents). Any future or standing gain claim
for a gate-shaped feature — escalation lanes, rerank seats, verify loops,
abstention-then-escalate — must survive: `baseline` vs `gate` vs `placebo`
(same trigger rate, judgment replaced by a seeded random/no-op choice). Gate
beats baseline is necessary; gate beats placebo is the attribution claim.

## Adoption (files created — mechanics in the private repos)

- **riir-refine `.issues/195`** — the escalation chain's production
  measurement (its serve-boundary wiring is live) carries the
  three-arm gate: baseline / real / seeded-random-pick placebo at the same
  escalation rate; attribution requires beating the placebo, not just baseline.
  Generalizes to any future gate feature in that repo (hosted generate lane,
  per-domain verifiers).
- **riir-rethink `.issues/049`** — the agreement-replay probe as a receipts-free
  VoI proxy for the ESC escalation lane: replay frozen decision states through
  cheap + think legs, report agreement/disagreement as LEVEL readouts (no
  dynamics signal; stationary-surface discipline). Answers swap-safety and
  wasted-escalation upper bounds; never answers which side is right — that
  stays with the receipts-blind VoI observer until an outcome join lands.

## Validations (no action)

- Prefill-only serving economics (7–40× over generative judges) — reflex's
  reason to exist, third external confirmation. Also: Kev-27B is a Qwen3.8-27B
  fine-tune — the judge class runs on the stack's league backbone kernels
  (573's open-recipe class) if a served gate consumer ever appears.
- Judge-size non-monotonicity + 97%/90.6% replay agreement — supports the
  per-suite winner law (registered == served) and adds a cheap instrument for
  swap decisions; small decision heads are competitive WHEN context length and
  thresholds are sized for the task (41% of their Airline inputs exceeded the
  small judges' validated context — the failure was length, not capacity).
- 42/101 policy-vs-intent coverage gap — the Issue-030 relevance lesson
  (a syntactic check is not a relevance check; the healer's input-scope gate)
  re-confirmed in a third domain: rules encode the policy, the misses queue
  censuses the rest.
- Asymmetric threshold shape (default-allow, act only on confident violation,
  P ≤ 0.05) + conjunctive checklist rule (P(required) ≥ 0.5 ∧ P(settled) ≤ 0.5)
  — the gate-shape vocabulary for sigmoid thresholds where flag-side and
  allow-side costs differ. No surface currently mis-shaped; keep the pattern
  for new gates.
- Game-context: the gate-before-consequence pattern is the authority plane's
  server-side command validation + the chain's rate-test settlement gate
  (Glacial cadence = the write gate's trigger selectivity). Per-NPC cognition:
  no consumer (NPCs propose no tool calls); the pattern would only enter via
  chain settlement or GM-command validation.

## Routing

- Note: `riir-reflex/.research/013_JevAct_Decision_Gate_Agent_Runtime.md` (this file).
- Issues: refine `.issues/195_escalation_chain_placebo_arm.md`,
  rethink `.issues/049_esc_agreement_replay_voi_proxy.md` (both private —
  this note intentionally carries none of their internals).
- PASS-Redirects appended to: reflex 002, katgpt-rs 562, katgpt-rs 579.

## What would change this verdict

- A served gate consumer appearing in the stack (agent-loop route points
  graduating beyond riir-ai's shipped decision gates) → the policy→check
  compiler becomes a real lane; re-distill then.
- Any standing gate-feature gain claim that predates this law → re-gate with
  the placebo arm before quoting it again.
