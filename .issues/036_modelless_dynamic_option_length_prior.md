# Issue 036 — modelless: the drafter-only path is degenerate on dynamic option spaces

**Status:** OPEN — filed 2026-09-26 from Issue 035 T4 (Bench 048). **T1 DONE** (2026-09-26: `Slot.drafter_only` + the `; drafter-only=N` routing-reason disclosure, engine-tested — any wire caller can now see the weakest-scorer posture). T2/T3 open (measured finding, no fix attempted — a fix selected on the cua fixture would be test-tuned).

## Finding

On the cua-s1-forms test split (Bench 048 — their 24,370-row supplied-option form
task, options = up to 29 `fill <entity>: <value>` strings + `check`/`click`/`skip`),
the modelless engine scores **1,063 / 24,370 = 4.36%** — BELOW uniform chance
(5.57%) and far below the constant-`skip` floor (52.16%). Per gold action it gets
`check` 816/816, `fill` 247/9,802, `click` 0/1,040, `skip` 0/12,712: its picks
collapse onto the `check` option regardless of the context — the Issue 004 T7
**degenerate-lane** shape, resurfacing on a different path.

Mechanism (structural, not fixture-specific):

1. **Route terms are inactive.** They arm only when `k == N` (index alignment) or
   when EVERY option string names a domain (`option_name_route`, Issue 023). A
   dynamic option space — options minted per request (`fill Given name: Isla`) —
   satisfies neither, so the option score is the `Lz4FlexDrafter` compressed-length
   delta ALONE.
2. **The drafter delta carries an option-length / literal-cost prior.** Appending a
   candidate always grows the compressed stream; a short literal (`check`) grows it
   less than a long option whose row-unique value (`Isla`, a phone number) cannot
   match the corpus even when its key does. The context-conditioned signal (a train
   doc continuing `ELEMENT Edit "First Name" …` with `fill Given name:`) is smaller
   than that prior, so it only rarely wins (the 247 fill hits).

This is the same "the byte-level delta cannot rank options" note `engine.rs` records
for Issue 004 T7 — there the centroid cosine rescued it because `k == N`; here no
rescue term exists.

## Scope honesty

- The cua fixture is out of the engine's served shape (fixed label universes). The
  product surface is not affected on the 15 suites (every one arms route terms).
  But `decision_wire` accepts request-time options by contract, and a caller that
  sends minted options gets this degenerate behavior with no warning.
- Do NOT tune a fix on the cua test split (Bench 048 is a measurement, not a
  selection slice). Their `validation.jsonl` is the protocol-clean selection slice
  for any candidate.

## Plan

- [x] **T1 — detect + disclose first (cheap, no accuracy claim):** when a question
      arms neither route path, surface it (routing reason / a `drafter_only` flag)
      so a caller can see the engine is running its weakest scorer. — DONE:
      `Slot.drafter_only` (choice/score only — noul's no-route posture is by
      design, issue 030, and never counts) + `routing_reason` appends
      `; drafter-only=N` when any slot was drafter-only (rides the existing
      wire `Routing.reason` — serve edge, harness, and the GOAT bench's warm
      line all surface it). Test: `routing_reason_discloses_drafter_only_questions`.
- [ ] **T2 — candidate modelless fixes, selected on cua `validation.jsonl` only:**
      (a) length-normalized delta (per candidate byte, or delta minus the
      candidate's standalone compressed cost — the NCD-style conditional term);
      (b) score only the candidate SUFFIX that differs from its siblings (shared
      prefixes like `fill ` cancel); (c) key-only scoring (strip the row-unique
      value after `: `). Report each against the constant-`skip` floor.
- [ ] **T3 — gate:** a candidate is promoted only if it beats the floor on cua
      test (read once) AND is byte-identical on the 15 suites' route-armed path
      (the GOAT no-regression gate).

## References

- `.benchmarks/048_cua_s1_forms_arena.md` — the measurement.
- `src/engine.rs` module doc §3 (the Issue 004 T7 note) + `solve_into` route gating.
