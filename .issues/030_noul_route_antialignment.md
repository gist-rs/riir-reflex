# Issue 030 — noul questions take route terms through the legacy `k == N` index path, and on prompt_injections the alignment is ANTI-correlated (the recorded T7 −4.3 pt regression's root cause)

Status: RESOLVED — the anti-alignment fix LANDED at `36e4e0a` (bench 038); lever-4 fitted heads LANDED at the bench-040 protocol (`--head-select`, +34.1 pt net, zero regressions). The residual emotion route-margin gate is MOOT (emotion selects scale 0 and is bit-identical to baseline).

## The finding

The Issue-004 T7 option-rank blend attached centroid route terms to options
through two paths (`src/engine.rs` `solve_into`):

1. **by-name** (Issue 023): the option string equals a domain name —
   correctly EXCLUDES noul (`!matches!(q.kind, QuestionKind::Noul)`).
2. **legacy `k == N` index alignment**: when the option count equals the
   domain count, option `i` → domain `i`. This path has **no noul guard**.

`prompt_injections` arms exactly `N = 2` domains — `labels = ["0", "1"]`
(`runner.rs` `prepare`), i.e. domain 0 = benign corpus, domain 1 = injection
corpus — and every question is **noul** with internal option order
`[yes, no]` where yes = "this text IS an injection" (`suites.rs`
`build_prompt_injections`, gold idx 1 = injection; the engine's internal noul
order is `[yes, no]` per the eval-engine flip convention).

So the legacy path maps option 0 ("yes, injection") → domain 0 (the **benign**
centroid): the route term is **anti-aligned by construction**. Whenever the
state resembles either corpus, the cosine pushes toward the WRONG answer —
a below-chance accuracy is the expected signature, and it is measured:
**0.4397 vs 0.4828 drafter-only and 0.50 chance** (T7 Addendum 5,
`001_phase1_harness.md`; published TABLES.md row confirms at HEAD).

The `k == N` index rule is only sound when the option list enumerates the
label universe in label order. A noul question's `[yes, no]` is
question-semantic vocabulary, never a label list — attaching it to label
corpora by index is arbitrary (anti-correlated here, would be luck elsewhere).
Noul-only suites are even documented as label-armed, option-key-exempt
(`prepare`'s arming comment) — the scoring side just never got the same memo.

## The fix

`route_active` requires a Choice/Score question: noul never takes route terms
through either path. Blast radius: only engines where `N == k == 2` on a noul
question — `prompt_injections` is the only dataset suite in that shape
(typed_decisions/harness-family noul runs on N ≥ 3 engines, already dark).
The serve edge is a 7-domain engine — unaffected.

## Tasks

- [x] engine.rs: noul excluded from `route_active` + regression test
      (route_scale=∞ vs 0 bit-identical on noul; choice control still blends).
- [x] Before/after modelless harness A/B (deterministic lane; accuracy is
      the claim, latency is not) — bench 038: prompt_injections
      0.4397 → 0.4828, every other suite bit-identical.
- [x] Bench record + issue close-out with the commit hash.
- [x] Route-margin confidence gate for the residual emotion −1.3 pt
      (5 questions of delta — defer until the fitted-head arc lands, then
      re-read; a 5-question gate is not worth its own knob today).
      RESOLVED MOOT: under the bench-039 selection protocol emotion picks
      scale 0 and reads byte-identical to baseline (0.2825) — the
      residual is the pre-head lane's own, unchanged.
- [x] Lever 4 (the feature-class arc): fitted per-label heads over the
      hashed-bag features, trained at engine-build time from the labeled
      corpora (one-vs-all logistic + sigmoid — never softmax), frozen into
      the build; the game-heads arc (Benches 881/882, 44/120 → served head)
      and riir-clippy rule_embed (Bench 099, 94 → 98) are the sibling
      precedents. Blast-radius warning honored: the `Embedder` is UNTOUCHED
      (every frozen fixture, game-head anchor and corpus gate bit-stable).
      LANDED (bench 040): `src/label_heads.rs` + `EngineConfig.head_scale`
      (default 0 = byte-identical baseline; `build()` refuses a non-zero
      scale it cannot fit) + harness `--head-select` (stratified-slice
      selection, ladder [0, 0.25, 0.5, 1], 5 pt promotion bar, ties → 0).
      Measured: banking77 +23.8 pt (0.4460→0.6840), massive +10.3 pt
      (0.6900→0.7933), every other suite bit-identical or ECE-better
      (sst5), G1/G2/G4 PASS, G3 bit-identical. The Embedder blast-radius
      warning was the one constraint the implementation never touched.
- [x] After promotion: the published-table republish is the one open item
      — the arena protocol posture is `--head-select`; the published
      bench.json/001_phase1_tables + reflex-site deploy follow in the
      republish pass (both laya lanes, preflight-clean, manual CF deploy).
