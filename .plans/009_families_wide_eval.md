# Plan 009 — wide template-disjoint eval for the six harness families

Status: REVISED 2026-10-02 (owner verdict): WITHDRAWN before execution — the wide eval is
NOT built. The six harness families stay INTERNAL TEST FIXTURES with their existing
n = 12–16 evals. Issue 059 removed; the Issue-008-T8 re-open chain is DEAD.

## Verdict (owner call 2026-10-02 — supersedes the original design below)

The six harness decision-point families are **internal test fixtures, not benchmarks**.
The wide eval is not built, and the Issue-008-T8 re-open condition is adjudicated
CLOSED rather than satisfied by construction work:

- **Authored data cannot become a benchmark by self-defined disjointness.** The proposed
  gates (unigram ceiling, 3-gram wall, label bans, digest pin) reduce token-surface
  leakage, but question shapes, distractor styles and class balance remain ours — and
  corpus/cal (the train substrate) were to stay unchanged, so train and eval would come
  from the same authored distribution. A specialist "win" would prove grammar-fit, not
  capability, and could not be cited beside an external index. Jev-style standardization
  lives on the dataset suites + bench.json (reflex-site Plan 001), which the families can
  never join.
- **The lane had no consumers.** Instinct dropped the six families from the specialist
  covered set (instinct Issue 008 T8), law-excludes them from the arena, and the site
  card already dropped their cells. The wide eval's only would-be consumer was the
  hypothetical riir-train trainer extension — refused below.
- **The re-open condition was necessary, not sufficient.** "A larger template-disjoint
  reflex-side eval" fixes statistical power, not external validity — larger + disjoint
  does not make authored data a benchmark.

## Keep (unchanged, already load-bearing)

- The families stay exactly as landed: G2/G4 measurement workloads, engine regression
  pins (discrimination floor, determinism, anti-pathology, gold-agreement), existing
  n = 12–16 evals, frozen fixtures, the `a0_stands` fallback.
- `harness_cache_reuse` keeps GATE standing (grounded posture, Bench 072: modelless
  0.9167 vs LLM-lane 0.5000) — a real gate, not merely a fixture.
- No code changes; no deletion of families or gates. Their existing evals are the right
  size for that job.

## What happened (the record)

- The plan + anchor issue were authored as filed and committed for the record at
  `72c1238` — no code, no evals, no bench record ever landed (verified 2026-10-02: no
  `families_eval_wide.rs`, no landing commits, no `.benchmarks/104_*` record).
- This revision withdraws the work before any build; issue 059 is removed per the
  noise-reduction rule with its record in `HISTORY.md` (2026-10-02 entry).

## Tasks

- [x] Adjudicate: withdraw the wide eval (owner verdict, this revision)
- [x] Preserve the as-filed plan + issue in git history (`72c1238`)
- [x] Remove `.issues/059` (noise-reduction rule; record = the HISTORY.md 2026-10-02 entry)
- [x] HISTORY.md: the Issue-008-T8 re-open condition adjudicated CLOSED — authored
      harness families are internal fixtures; specialist/training claims on them are
      OUT OF SCOPE
- [x] riir-train: nothing filed (the original trainer-extension handoff is dead)
- [x] Commit + push (develop)

## Dead (explicitly refused by this revision)

- [-] Authoring ~96 wide eval cases per family (6 × ~96 authored cases)
- [-] `src/harness/families_eval_wide.rs` + all six `FamilyDef.eval` rewiring
- [-] Disjointness gate code (unigram-overlap ceilings / 3-gram wall / label-token bans /
      BLAKE3 digest pin)
- [-] A0 re-baseline + `.benchmarks/104_families_wide_eval_a0.md` record
- [-] riir-train trainer-extension issue + instinct re-seat (re-open chain steps 2–3)

## Non-goals

- No external claims: family rows stay out of bench.json / the reflex-site card /
  instinct's seat or specialist population (already absent — keep it that way; if
  publish_bench ever picks family rows up from harness output, that is the drift to
  guard).
- No eval expansion, no re-runs, no specialist or training work against authored
  families, no instinct re-seat, no site cell.
- If a decision-point family ever needs a real eval again, the honest path is mapping it
  onto a public dataset slice — never more authored templates.

## Original design (historical — full text at `72c1238` in git history)

Wide per-family eval populations (~96 cases, class-balanced, 88–104 accepted) replacing
each family's 12–16-case eval, with template-disjointness enforced by gate (per-family
unigram-overlap ceilings measured against the old eval, shared-3-gram wall, label-token
bans, BLAKE3 digest pin over all six populations), corpus/cal unchanged, A0 re-baselined
into a bench record, and a re-open chain: this eval → riir-train trainer extension →
instinct winner_bridge re-seat. Superseded by the Verdict above before any of it was
built.
