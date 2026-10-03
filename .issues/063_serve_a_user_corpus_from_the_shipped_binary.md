# Issue 063 — the shipped binary cannot serve a user's corpus, so no "first corpus" walkthrough can run

**Status:** OPEN — filed 2026-10-03; blocks reflex-site Issue 006 T4.

## Evidence

- `src/serve.rs::run` always builds `demo_engine()` — two fixed expert texts (ops, support).
  There is no env var, flag or route that loads anything else (grep of `src/serve.rs`,
  `src/main.rs` for a corpus path: none).
- The agent skill says so plainly (§5: "Serving a custom corpus is the in-repo harness lane, not a
  shipped wire surface").
- reflex-site Issue 006 T4 asks for "your first corpus in 5 minutes: write a few docs, label
  rows, fit a threshold, ask". Against the release binary that walkthrough cannot be written:
  the only text corpus a user can reach is the demo one, which abstains on every text question
  (captured: `reflex-site/data/wire.json`, cases `abstained` and `off_corpus`).

## Design constraint

`DecisionEngine<const N: usize, const D: usize>` fixes the expert (domain) count at compile
time. A user corpus has a runtime domain count. Options, cheapest first:

1. A closed enum over monomorphised `N ∈ 1..=8` (match once at boot, one arm per N); over 8
   domains refuses loudly. No hot-path cost: dispatch happens once per request, outside the
   per-option loop.
2. A runtime-`N` engine variant: touches the zero-allocation hot path, so it needs the G2/G4
   gates re-run.

Recommendation: option 1. It keeps the gated hot path byte-identical.

## Tasks

- [ ] T1 `RIIR_REFLEX_CORPUS=<dir>`: one `<domain>.md` (or `<domain>/` of `.md` files) per
  domain → `ExpertSpec::new(domain, texts)`; absent var = today's demo engine, unchanged.
  Boot line names the domains and file counts; a malformed dir refuses to start (never a
  silent demo fallback — the per-lane-claims law).
- [ ] T2 `/healthz` discloses `corpus: "demo" | {"domains": [...]}`.
- [ ] T3 a sample corpus under `examples/first_corpus/` (public-origin text only) + a test that
  boots on it and gets a non-null answer on an in-corpus question.
- [ ] T4 G2/G4 re-run on the dispatch wrapper (must be unchanged within noise — the
  dispatch is per request, not per option).
- [ ] T5 release; then reflex-site 006 T4 writes the walkthrough from captured output.
