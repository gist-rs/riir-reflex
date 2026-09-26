# Issue 036 — modelless: the drafter-only path is degenerate on dynamic option spaces

**Status:** OPEN — filed 2026-09-26 from Issue 035 T4 (Bench 048). **T1 DONE** (2026-09-26: `Slot.drafter_only` + the `; drafter-only=N` routing-reason disclosure, engine-tested — any wire caller can now see the weakest-scorer posture). **T2 DONE AS MEASURED NEGATIVE + T3 gate CLOSED 2026-09-26** (`EngineConfig::drafter_fix`, four candidates; ALL fail the constant-skip floor on the cua VALIDATION split — best `ncd` 11.17% vs floor 52.22% — so NOTHING promotes and the test split stays unread, per the gate's own rule; full numbers below). The disclosed-abstain path (T1) is the product mitigation.

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
- [x] **T2 — candidate modelless fixes, selected on cua `validation.jsonl` only:**
      LANDED as `EngineConfig::drafter_fix` (opt-in, default Off, engaged ONLY on the
      drafter-only path — route-armed suites byte-identical by construction, test
      `drafter_fix_off_keeps_the_shipped_scores_bit_identical`): (a)
      `per_byte` (length-normalized delta) + `ncd` (delta − standalone compressed
      cost, the NCD-style conditional term); (b) `shared_prefix` (the LONGEST
      prefix carried by a MAJORITY of options, ≥ 4 B — the all-options common
      prefix is empty on mixed sets like cua's fills+check+click, measured en
      route); (c) `key_only` (bytes before the first `: `). MEASURED on the full
      validation split (22,054 rows, cap 64/action, train corpus unchanged):

      | fix | top-1 | pick collapse |
      |---|---|---|
      | off (shipped) | 4.63% | check 20,915/22,054 |
      | per_byte | — (3000-row stride: 2.80%, WORSE) | all-fill |
      | **ncd** | **11.17%** (stride 11.07%) | all-fill |
      | shared_prefix | — (stride 3.90%) | all-check |
      | key_only | — (stride 6.77%) | fill/check split |

      Floors: uniform chance 5.61% · constant-skip **52.22%**. NCD more than
      DOUBLES the shipped scorer (4.63 → 11.17) and kills the check-collapse —
      the length prior was real and the conditional term removes it — but the
      scorer stays 4.7× below the trivial floor: on a dynamic option space the
      corpus-is-the-model signal identifies the ACTION family, never the
      specific field.
- [x] **T3 — gate:** CLOSED NEGATIVE — no candidate beats the constant-skip
      floor on validation, so none promotes and the cua test split is NEVER
      read (the gate's read-once rule applies to a survivor; there is none).
      The 15-suite no-regression clause holds by construction (the fixes
      engage only on drafter-only questions; bit-identity pinned by test).
      `drafter_fix` stays an opt-in measurement knob; if the lane is ever
      revisited, `ncd` is the recorded best base.

## References

- `.benchmarks/048_cua_s1_forms_arena.md` — the measurement.
- `src/engine.rs` module doc §3 (the Issue 004 T7 note) + `solve_into` route gating.
