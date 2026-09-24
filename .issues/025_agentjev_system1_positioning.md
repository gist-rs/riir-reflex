# AgentJev / System-One positioning — the typed-decision category grew a beat-laya challenger

**Status:** OPEN — filed from `.research/002_agentjev_system1_landscape.md` (external repo `malevrigns/agent-jev` @ `a965ca8ff06ccabc0c796dca5447b55cc2069cee`, Apache-2.0).

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

- [ ] Bench docs (`.benchmarks/001_phase1_tables/TABLES.md` header block or a
      `## landscape` section + README results section): add the published
      AgentJev-0.6B row — 79.25% (bool 88.83 / choice 75.33 / score 75.00),
      ~60–70 ms p50/case cuda:0, 2048 ctx — with the protocol footnote:
      their accuracy = agreement with teacher argmax; their Laya row is
      card-copied not re-scored; our 74.45% is gold-label measured under the
      standard harness protocol; their wide-load shared-prefix figure
      (298.91 ms @ 66 paths / 33.5k tokens) is their box, not ours.
- [ ] README competitive-landscape paragraph: Jev / Laya / AgentJev / reflex
      axes — accuracy bar for specialists is now ~79%; reflex's axis is
      modelless latency + abstention + calibration floor + hybrid lane
      (`Lane::Hybrid`: specialist proposes, modelless gates).
- [ ] Verify before citing: pull the AgentJev published numbers from their
      README/`typed_decisions/comparison.json` at the pinned sha (do not
      retype from memory; the note's table is the transcript).
- [ ] (stretch, owner-gated — do NOT auto-start) lane-3 feasibility spike:
      serve `aimeigaoshou/agent-jev` Apache-2.0 safetensors through
      riir-infer as a third lane — Qwen3-0.6B backbone (causal, arch family
      already in our serving stack for the big model) + the 2-layer
      permutation-equivariant set head + shared-prefix KV branching (their
      `jev_service/prefix.py` is the reference; 92.4% token-op reduction at
      wide loads). Blocked on owner call: does reflex want a model-based
      accuracy lane beyond laya at all. License: Apache-2.0 weights are
      compatible with our MIT public surface (attribution required).

## Non-goals (decided in the research verdict)

- No `margin` wire field — derivable from the full per-option distribution
  `Answer::probabilities` already carries.
- No shared-prefix work without a causal decision lane to host it (the laya
  backbone is bidirectional; the modelless lane has no KV).
- riir-train deferral for an own ternary decision head is justified in the
  note (consume Apache-2.0 first; reopen trigger recorded there).
