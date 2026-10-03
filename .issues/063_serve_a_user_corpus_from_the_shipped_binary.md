# Issue 063 — the shipped binary cannot serve a user's corpus, so no "first corpus" walkthrough can run

**Status:** LANDED (source, this commit) — T1–T4 done; T5 waits on the next release cut, after which reflex-site 006 T4 writes the walkthrough from captured output.

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

- [x] T1 `RIIR_REFLEX_CORPUS=<dir>`: one `<domain>.md` (or `<domain>/` of `.md` files) per
  domain → `ExpertSpec::new(domain, texts)`; absent var = today's demo engine, unchanged.
  Boot line names the domains and file counts; a malformed dir refuses to start (never a
  silent demo fallback — the per-lane-claims law). LANDED: `src/corpus.rs` (the loader,
  unit-tested) + `resolve_serve_corpus()`/`serve_boot()` in `src/serve.rs` — the corpus is
  resolved BEFORE the bind (a malformed dir exits 2 before anything listens), domains sort,
  `<name>.md` + `<name>/` collision refuses, over 8 refuses naming the count. Live smoke:
  boot log `corpus: 3 domain(s) from examples/first_corpus — billing(3), deploy(3),
  onboarding(2)`; malformed dir → exit 2 with the named reason.
- [x] T2 `/healthz` discloses `corpus: "demo" | {"domains": [...]}`. LANDED via the
  `serve_listener_heads_corpus` funnel (the legacy `serve_listener*` fns delegate with
  `CorpusInfo::Demo`, so their healthz bytes stay honest and no test call site moved);
  `tests/engine_gates.rs`'s exact-bytes healthz pin re-pinned with the additive field.
- [x] T3 a sample corpus under `examples/first_corpus/` (public-origin text only) + a test that
  boots on it and gets a non-null answer on an in-corpus question. LANDED:
  `tests/serve_corpus.rs` (5 tests — in-corpus answers with the right pick, byte-identical
  repeat, off-corpus abstains, both healthz postures). `[[test]]` row with
  `required-features = ["modelless"]` in the same commit (the T1.1e rule).
- [x] T4 G2/G4 re-run on the dispatch wrapper: **G2 PASS (p99 45 µs ≤ 1000 µs) · G4 PASS
  (core alloc-free, canary armed)**. PROVENANCE: power=AC load=8.94 powermode=2 — the
  preflight REFUSED (sibling benches running), so read the p50/p99 as provisional
  workstation readings, never a published number; the structural claim carries the verdict:
  `benches/decision_set_goat.rs` drives `engine` directly and does not import `serve` — the
  dispatch is per-process at boot (one monomorphised arm per `N ∈ 1..=8`), the hot path is
  untouched by construction, and G4's zero-alloc core is asserted green on this exact tree.
- [ ] T5 release; then reflex-site 006 T4 writes the walkthrough from captured output.

## The measured posture decision (the part the plan did not spell out)

The default fused-gate thresholds (score 0.35 / distance 0.5) **do not transfer to a user
corpus** — measured on the sample corpus: a k=3 choice question whose options exactly name
the domains reads near-uniform probabilities (0.334/0.341/0.325), confidence ≈ 0.0002, and
abstains at EVERY default score threshold (the harness's own finding, Bench 001 addendum:
"the fused-gate birth thresholds (0.35/0.5) do NOT transfer"). A first corpus is also below
the threshold-fit support floor, so the harness's cal-slice fit is not reachable either.

The corpus boot therefore serves the **distance-only posture** (`score_threshold = 0.0`,
issue 042 lever 2's measured lever): the corpus is labeled by construction (every doc's
domain is known), the corpus-distance gate decides — in-corpus answers with the engine's best
pick (deploy state → index 1, billing state → index 0, live-verified), off-corpus still
abstains (sourdough → null; in-corpus distance confidence ≈ 0.93+, off-corpus ≈ 0.06, the
0.5 default midpoint sits in the desert). The demo posture keeps the default config —
byte-identical. The boot prints the posture loudly: `corpus gates: distance-only (score
axis open — a first corpus is too thin to calibrate the confidence readout; off-corpus still
abstains)`.

**Disclosed caveats (wire-honest, not hidden):**
- An answered corpus answer still carries the SCORE readout's confidence (≈ 0.0002 on the
  sample corpus) — the pick is distance-gated, the confidence field semantics are unchanged
  (the confidence readout). Consumers read `outcome`, not confidence, for "did it answer".
- noul questions on a first corpus answer with the drafter's yes/no polarity under the same
  distance-only gate — honest for in-corpus text, abstains off-corpus.
- The demo posture is UNCHANGED byte-for-byte (default config, same engine); the 063 posture
  applies only when `RIIR_REFLEX_CORPUS` names a directory.
