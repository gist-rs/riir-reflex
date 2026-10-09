---
name: grammar-not-answers
description: The doctrine for scoring a corpus, RAG, or decision system on an external benchmark without contamination. Never ingest test rows in any proportion. Build the corpus AROUND the test with teacher-synthesized train-pool data, echo-gate and leak-scan the artifact, score one frozen read. Use when curious whether we can score on a new benchmark, when tempted to seed retrieval or training with test data, when synthesizing eval-adjacent corpora, or when a near-perfect score looks suspicious.
---

# Grammar, Not Answers — the around-the-test doctrine

Our systems sit between RAG (retrieve) and LLM (generate). The corpus learns a
domain's **grammar and vocab** — rules, bags, option-rank heads. That position
makes the cheating temptation structural: the easiest way to score on any
benchmark is to pour its test rows into the corpus. This skill is the named,
gated, shame-free alternative. **Invoking it IS the honest path** — follow the
steps and quote the number proudly; skip a gate and you don't have a number yet.

## The law — two halves

1. **Never fill test rows into the corpus, retrieval store, or training — at any
   percentage.** 30% or 70%, the score dies the same death: the system stops
   demonstrating grammar and starts retrieving answers. A contaminated run is
   VOID even when its numbers look fine (Bench 073 precedent — in-sample
   contamination voided the promotion run; only pool-free aggregates were
   salvaged).
2. **Build the corpus AROUND the test.** A teacher synthesizes domain-adjacent
   data from the TRAIN pool only; the artifact is sealed + BLAKE3-hashed; gates
   prove the synth rows carry the distribution (grammar), not memorized test
   (echo).

Why this is legitimate: the honest claim is "learned the domain's grammar and
vocab," never "saw the questions." The gates make that difference measurable —
**the measurability IS the legitimacy.**

## Name the posture before acting

| posture | shape | verdict |
|---|---|---|
| answer-key | test rows or shipped reference solutions enter the corpus/retrieval/training | CHEAT — 100% is retrieval of the key. TermGrade ships `solution/` per task and gold-pass-filters on it; executing it scores 100% by construction and measures nothing |
| empty-hope | no corpus work; borrow an LLM's rules as the judge; hope | WEIRD — unfair judge, uninterpretable score |
| around-the-test | teacher synthesizes from the train pool; sealed; echo-gated; one frozen test read | THE DOCTRINE — the only posture that yields a quotable number |

## Workflow

**Step 0 — Shape test.** Can our system speak the bench's wire? choice/score/
noul over presented options vs free-form act generation (a decision engine
cannot emit a shell command; an agent bench needs a scaffold we may not have).
If the wire cannot speak, the honest answers are "project the bench into
decisions" or "decline" — never force it.

**Step 1 — Freeze the split.** Locate train/test. The test is READ-ONLY and
read ONCE, by the scoring harness only. The teacher never reads it either —
the synthesis pool is the train split (in this repo's lane the pool is
literally `prepared.train`).

**Step 2 — Synthesize around** (this repo's harness):

```sh
cargo run --release --bin harness -- --synth-plan ...    # report only
cargo run --release --bin harness -- --synth-corpus ...  # writes the sealed SYNT artifact + blake3 sidecar
```

The teacher veto (openthai agreement) is the acceptance authority.
`--synth-rescue-prefilter` only NARROWS candidates ahead of the veto (AUGMENT,
never replace). Code: `src/harness/runner/synth.rs`.

**Step 3 — Gate before any number.** Every `--corpus-ab` pass computes the
echo gates: abstention-entropy KL ≤ 0.05 + the OOD word-dropout ladder. A
clean pass with a negative gate-rung LB95 reads ECHO — **the lane dies.** Ride
the near-duplicate leak scan (`--features slice_leak`). Code:
`src/harness/runner/{corpus_ab,echo_gates}.rs`.

**Step 4 — Score once, frozen.** `--corpus-ab` over ONE frozen test read,
paired LB95 gold-only vs +synth. Check option-order sensitivity
(`--perm-probe`) before quoting choice-suite numbers.

**Step 5 — Read the score suspicion-first.** Near-100% or any big jump ⇒
suspect leakage, degenerate options, or echo BEFORE skill. Re-run gates + leak
scan. A legitimately earned high score on unseen data is a GOAT — quote it
proudly. **The shame is only ever in a number that lies, never in the knowing.**

## Precedents on record

- The synth lane + V5 gate — `../riir-train/.plans/426_surpass_openthai_teacher_fusion.md` T5; this repo's harness `--synth-*` / `--corpus-ab`.
- Bench 073 — contaminated selection VOIDED the run (the salvage rule kept only pool-free aggregates).
- Bench 085 — a contamination suspicion answered by a clean re-read (measurement decides, both directions).
- Benches 120/121/122 — density gates measured NEGATIVE; the seated corpus stays the ungated artifact (gates guard honesty, not density).
- Issue-004 — six home-made families retired (evals the engine read at chance); the answer-key posture is the same disease mirrored.
- `.research/005` (JDI protocol) — the industry posture agrees; entrants lose contaminated rows.

## Boundaries

- The teacher may be an LLM; its POOL is train-only. Second-hand contamination
  through the teacher is still contamination.
- Sealed artifacts only. An unsealed edit after gating re-opens every gate.
- One frozen read per claim; re-reads need a reason (a box-state suspicion,
  Bench-085-style), never a better score.
- Mining external code sources for rules/heuristics is the `distill` skill in
  `../riir-refine` (sibling). This skill is its benchmark-facing twin: how to
  MEET a benchmark honestly.
