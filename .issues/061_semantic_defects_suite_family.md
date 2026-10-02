# Issue 061 — semantic_defects suite family (code-defect question family; authoring task, perch-derived taxonomy)

**Status:** OPEN — proposed 2026-10-02 from the riir-refine distill of `lakeday-org/perch` (distill-skill verdict, 2-round Claude AGREE; queue line in riir-refine `.distill/001_mining_queue_snapshot.md`). Authoring task, NOT a port: perch ships no labelled eval set (verified below), so the gold labels are ours to write. Nothing wired; no lane added.

**What:** a `semantic_defects` harness family (or a `code_fixtures` extension) measuring decision models on code-defect questions — the class perch (perchscan.com) serves in production with System One decision models. Seeds from the source (all at the pin below):

- **Taxonomy** (perch's defect classes): `wrong_order`, `inverted_condition`, `off_by_one`, `unhandled_null`, `error_ignored`, `wrong_return_value`, `bad_state_change`, `wrong_lookup`, `resource_leak`, plus `does_not_do_what_it_claims` (a method whose name/contract misdescribes its behavior).
- **Question-shape grammar** (`scan.yaml`, 37 declared questions): boolean questions carry DUAL grounding criteria — explicit `"true"` and `"false"` text, i.e. the calibration target is stated on BOTH sides, not just the positive; per-question belief floors (`min:` 60–75%) that gate DISPLAY while ranking counts the whole distribution (no cliff at the boundary); conditional gating (a finding = class × reachability — "where two answers are both needed for a problem to be real they multiply"); `type: choice` with per-option descriptions; `type: score` with 4 descriptive severity levels reported as the expected level over the distribution (mean over the rubric, band = rounded mean, likeliest kept separate).

**Open question the cell answers:** do our lanes (agentjev, openthai, laya, PAW, gliner) answer code-defect questions at all — a new question family beside `typed_decisions` (score questions over scenario states) and `code_fixtures` (module + is_pub)? Secondary serving datum: perch's question-batching contract — each model advertises how many questions per request it takes, `PERCH_MAX_QUESTIONS` overrides, and their tip commit is literally "perch sent a model more questions per request than it accepts (#265)" — real client-side evidence about the System One wire's per-request question capacity (see the issue 054 Part 4 addendum filed alongside this).

**Authoring shape:** small ORIGINAL code fixtures (Rust first — our lanes' home language), one known defect each (or a clean control), gold = the perch-class label + the defective line. Sizing precedent: bench 039's typed_decisions gold-label row. The scan.yaml dual-grounding pattern is the authoring quality bar: every gold-labelled boolean ships its true-criteria AND false-criteria text, so ambiguous fixtures are caught at authoring time, not at scoring time.

**Non-goals:**
- **No `--perch` lane.** perch is a CLIENT, not a model — a lane through it measures whatever `PERCH_BASE_URL` points at (the same cells agentjev/openthai already own). The wire question is settled by issue 054; this issue adds a SUITE, not a lane.
- **No cloud dependency.** perch ships no labelled eval set (verified at the pin): `test/fixtures/` is six per-language `order-service` acceptance apps + `unnamed-code`; `scripts/verify-fixtures.mjs` ("Live CLI acceptance tests... real Jev requests", requires `PERCH_API_KEY`) asserts method counts and CLI exit codes — zero defect-gold assertions. Defect-named strings in `test/cli.test.js`/`test/scan.test.js` are synthetic unit-test code.

**References:**
- Source pin: `lakeday-org/perch @ 07d38cacba96f1def641899ec695dceeb0fd7f43` (MIT, 2026-10-01) — distill verdict MARGINAL C+ corpus / LANE INTEL HIGH, riir-refine `.distill/001_mining_queue_snapshot.md` 2026-10-02
- Issue 054 (the System One wire pin; its Part 4 addendum, same commit as this file, records perch as a second production deployment of that wire)
- reflex bench 039 (the agentjev gold-label authoring posture), the `code_fixtures` suite
