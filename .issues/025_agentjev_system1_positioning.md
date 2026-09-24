# AgentJev / System-One positioning — the typed-decision category grew a beat-laya challenger

**Status:** OPEN — the three ungated bench-doc tasks LANDED 2026-09-24 (numbers verified at the pinned sha; headline 79.25 lives in `typed_decisions/agentjev_v1_report.json` `/trained/*` — `comparison.json` carries it too under `/agentjev/trained`, beside the phase-4 pre-run baseline). The stretch lane-3 spike is DEFERRED by verdict 2026-09-24 (named checkable reopen triggers in-file). A new optional MEASURE-only row (gold-label re-score through their own service) rides the 019 T3 4090 window (`.issues/027`; renumbered from 026 — upstream dual-allocated 026) — do NOT auto-start.

## Why

AgentJev-0.6B (open, Apache-2.0 code + weights on HF `aimeigaoshou/agent-jev`)
publishes **79.25%** on the Typed Decisions official test split — the SAME
400-case/2000-question split our harness already runs — against the
laya-typed checkpoint we ported (their table's Laya row 77.00%, card-copied;
our measured `laya-riir·typed` 74.45% @ 1312 ms p50/case metal). The category
("System One" typed decision models for agent gates/routes/scores) now has:
TypeSafe Jev (namesake, zero-shot 72.7%), Laya (convaiinnovations, the
reference we ship G5-parity), AgentJev (the new accuracy leader), and us.
Our README and bench tables cite only laya — a reader comparing categories
cannot find this.

Our measured differentiators (publish them beside the row): modelless core
0.472 ms p50/case (~100–1000× the specialists, zero weights), abstention
first-class (they have none — confidence via threshold), conformal-floor
calibration gate (G1), game heads, single-binary matrix.

## Tasks

- [x] Bench docs (`.benchmarks/001_phase1_tables/TABLES.md` header block or a
      `## landscape` section + README results section): add the published
      AgentJev-0.6B row — 79.25% (bool 88.83 / choice 75.33 / score 75.00),
      ~60–70 ms p50/case cuda:0, 2048 ctx — with the protocol footnote:
      their accuracy = agreement with teacher argmax; their Laya row is
      card-copied not re-scored; our 74.45% is gold-label measured under the
      standard harness protocol; their wide-load shared-prefix figure
      (298.91 ms @ 66 paths / 33.5k tokens) is their box, not ours.
      **DONE 2026-09-24** — TABLES.md is regenerated wholesale by
      `render_markdown`, so the durable `## Landscape` section lives in the
      RENDERER (`src/harness/runner.rs`) and survives every run; README
      results section carries the paragraph + the reflex-axis positioning.
- [x] README competitive-landscape paragraph: Jev / Laya / AgentJev / reflex
      axes — accuracy bar for specialists is now ~79%; reflex's axis is
      modelless latency + abstention + calibration floor + hybrid lane
      (`Lane::Hybrid`: specialist proposes, modelless gates). LANDED 2026-09-24
      (README Results section).
- [x] Verify before citing: pull the AgentJev published numbers from their
      README/`typed_decisions/comparison.json` at the pinned sha (do not
      retype from memory; the note's table is the transcript). DONE 2026-09-24
      — clone verified at `a965ca8f` then removed: README table (79.25 ·
      1585/2000), `agentjev_v1_report.json` `/trained/*` (0.7925 /
      0.88833 / 0.75333 / 0.75), `comparison.json` `/agentjev/trained` (the
      `/phase4` block there is the pre-run 38.70% baseline — do not cite it
      as the headline), latency table (Laya 41.53 ms faster on short inputs;
      shared-prefix 298.91 ms vs 609.65 unshared), README L174 (33,547 →
      2,551 = 92.4%).
- [-] (stretch, owner-gated — VERDICT 2026-09-24: DEFER) lane-3 feasibility spike:
      serve `aimeigaoshou/agent-jev` Apache-2.0 safetensors through
      riir-infer as a third lane — Qwen3-0.6B backbone (causal, arch family
      already in our serving stack for the big model) + the 2-layer
      permutation-equivariant set head + shared-prefix KV branching (their
      `jev_service/prefix.py` is the reference; 92.4% token-op reduction at
      wide loads). DEFERRED per the standing owner-gated delegation
      (reviewer verdict `#Verdict: AGREE`, round 1, 2026-09-24): riir-infer
      has multiple arcs mid-flight (op-layer unification T7, the EXL3
      trellis lane, the ANE lane) — a backbone port now is maximal
      collision, and it is a multi-week arc, not a spike; `Lane::Hybrid`
      needs A lane to sit behind, and laya already fills that seat.
      **Reopen triggers (checkable):** riir-infer op-layer T7 AND the EXL3
      trellis lane AND the ANE lane each closed, OR the first production
      consumer of `Lane::Hybrid` lands. The non-goal stands: no
      shared-prefix work without a causal decision lane to host it.
      License: Apache-2.0 weights are compatible with our MIT public
      surface (attribution required).
- [ ] (optional, rides the 019 T3 4090 window `.issues/027` [renumbered from 026] — do NOT
      auto-start) the MEASURE-vs-SERVE split (Gate-0 verdict amendment 4):
      the data point actually missing is AgentJev's **gold-label** accuracy
      on our 400-case split — their 79.25% is teacher-argmax agreement, a
      different protocol from our measured 74.45%. Obtainable the same way
      019 measures CLM: their Apache-2.0 Python service runs on the 4090,
      our Rust harness measures over HTTP — no riir-infer port, no
      collision with the in-flight arcs. **Wire condition RESOLVED at the
      contract level 2026-09-24** (read at the pinned sha, clone removed):
      their service is `POST /api/evaluate` (`jev_service/contract.py`),
      NOT the TypeSafe systemone wire — but the mapping is direct and
      client-side only: our noul → their `type: boolean` (criteria, or the
      default `TRUE`/`FALSE`), choice → `type: choice` (options array),
      score → `type: score` (levels, 2..10 ordered descriptions); response
      `{results: [{id, answers: […probabilities]}], usage: {wall_ms, …}}`
      maps into `DecisionResponse` with no server shim. Latency column:
      record BOTH client round-trip and their `usage.wall_ms` (server-side
      measure), prefer client round-trip for cross-lane consistency.

## Non-goals (decided in the research verdict)

- No `margin` wire field — derivable from the full per-option distribution
  `Answer::probabilities` already carries.
- No shared-prefix work without a causal decision lane to host it (the laya
  backbone is bidirectional; the modelless lane has no KV).
- riir-train deferral for an own ternary decision head is justified in the
  note (consume Apache-2.0 first; reopen trigger recorded there).
