# Open-Jev-Fast serving techniques → reflex/infer mapping (Research 221 follow-through, issue 054)

**Status:** RECORD — the transferable technique IMPLEMENTED (modelless engine case-level state derivation, bit-identical — `DecisionEngine::solve_sample_into` case-level block + the `per_question_slots_are_independent_of_their_neighbors` gate); the GDN-lane candidate 1 verdict recorded (absorbed by the v3 chunked-prefill design, residual re-arm named); no model loaded, no bench run, no arena lane (owner call: the 27B model is too huge for the fleet — Part 1 of issue 054 stays blocked/re-armed).

> Parent distill: riir-refine Research 221 (`open_jev_fast_cuda_gdn_serving_distill`, Batch 189 kernel_opt) — the source (`lyuyiqi/open-jev-fast @c52b8bb9`) was distilled into corpus rules THERE. This note is the reflex-side follow-through the distill mapped to issue 054: which of the four serving techniques have a home in OUR stack, and what landed.

## TL;DR

Issue 054's Part 1 (the `openjev` comparison lane) is refused by the owner for now — Open-Jev-27B-v1.1 needs ~98 GiB in bf16 and the fleet's biggest GPU is 24 GiB; the cell would not exist honestly at any quantized posture the source itself rejects. Part 2 (prefix-state handoff) is measured-REFUTED for the laya encoder (bidirectional ModernBERT couples the shared span — `riir-infer` `prefix_state_coupling.rs`, 5.1e2 drift). What remains actionable is exactly what this note records: the four serving TECHNIQUES behind the blocked lane, mapped onto our stack. One transfers cleanly and is now implemented — **the prefix-tree root-node sharing**, in the one place the shape fits: the modelless decision engine, whose hashed-bag features have no attention coupling, so sharing the case state across a request's questions is BIT-IDENTICAL by construction. The other three get grounded verdicts (absorbed / adjudicated / N-A), so no future session re-derives them.

## The four techniques, verdicts

| # | open-jev-fast technique | our surface | verdict |
|---|---|---|---|
| 1 | Two-level prefix tree (`gtree_build`/`forward_gtree`): compute the shared root/question text once; candidates ride ancestors | **modelless engine** (`src/engine.rs`): the case state IS the root — implement. **riir-infer-gpu GDN chunked prefill**: ABSORBED (below). **laya lane**: refuted (Part 2). | **IMPLEMENTED (engine) / ABSORBED (GDN) / REFUTED (laya)** |
| 2 | Ancestor-mask bit-visibility block-skip tree attention (`fattn_k`): per-row bit words, CTA unions visibility, skips key blocks no row sees | our league lane is single-stream (the distill's own NOT-transferable note); our `deltanet_tree_verify_cubecl.rs` is LINEAR-attention forward substitution, not softmax attention — no key-block visibility mask exists to skip | NOT TRANSFERABLE |
| 3 | Exact full-bf16-domain activation LUT (`act_tables_k`): 65,536 entries, the hot kernel is a gather | the modelless engine is f32 scalar sigmoids (N per question — no transcendental hot spot); the GDN lane's gate/RMSNorm ops already live FUSED inside kernels — the source's own measured anti-data says the LUT LOSES where folding removed the op (GDN_VER 5) | N/A (regime absent) |
| 4 | Shape-bucketed CUDA-Graph cache, LRU 512, copy-in-at-replay (`server.py`) | our 4090 prefill lane captures ONCE at `capture_p()` and replays — deliberately: "chunks of any OTHER `p` never capture/replay (the captured grids are `p`-shaped); a wider warm-up prefill therefore cannot steal the capture" (`prefill_cuda_full.rs`). The multi-bucket ladder was PRICED and CLOSED status-quo at riir-infer Issue 967 (`prefill_gv_fallback_counts()` is the standing ladder-gap counter) | ALREADY ADJUDICATED (Issue 967 status-quo) |

### 1a. The implemented piece — engine case-level state derivation

`DecisionEngine::solve_sample_into` used to re-derive every state-alone quantity **once per question**: the state embedding (`embed_into(req.state)`), the cosine route terms, the per-domain fitted-head blends, the count-table token view + per-domain in-scores (consumed by the route-nb arm, the noul polarity arm AND the option-conditioned arm — three re-hashes), and the ridge token view + per-domain scores. For a typed_decisions case (5 questions, long state) that is 5× the state hash pass and 5× the per-domain scorings for identical values.

The hoist derives all of them **once per case**, before the question loop:

- `state_q` — the state embedding (shared base).
- `route_base` — the N cosine terms (state alone).
- `head_base` — the N per-domain head blends.
- `nb_in_base` + the `nb_tok` fill — one hash pass serves all three nb consumers.
- `sc.ridge_tok` + `sc.ridge_in` + `r_temp` — one fill + one getter read.

Per question, the work that remains is only what genuinely depends on the question: the ctx embed (`state+prompt+criteria`, per-question by nature), the option→domain mapping, the drafter pass, the readout, the gates. MC perturbation stays per-question BY DESIGN (its seed folds `qi`) and re-derives from the shared base — the same bytes the pre-hoist form embedded fresh, so the perturbed arms are bit-identical too.

**Why exact:** every hoisted quantity is a pure function of `req.state` bytes; computing it once and reusing the value cannot move a bit. This is the bag-semantics case of the issue's Part 2 law — the laya encoder could not share the span because bidirectional attention couples it; a hashed bag has no coupling at all. The Part 2 refutation's closing sentence ("the lead's SHAPE stays valid where the serving model is linear/causal") is this implementation.

**Gates:** the full lib suite at default (263) and `mc_ensemble` (274) postures, including every existing engine bit-identity pin (`drafter_fix_off_keeps_the_shipped_scores_bit_identical`, `head_off_is_byte_identical_…`, the sigmoid delegation pin) plus the NEW regression gate `engine::tests::per_question_slots_are_independent_of_their_neighbors` — a mixed 4-question case (by-name route / k==N route / drafter-only / noul) solved in one request must match each question solved alone, bit for bit, which reds any future leak of question-local scratch across questions. G4: no new allocation (stack arrays + the existing scratch buffers; `prepare()` capacity law untouched).

**Honest scope:** the win is WORK-VOLUME (state hash + per-domain scoring passes: 5×→1× on a 5-question case), not a measured latency number — the owner constraint for this task is "no bench", so no G2 reading is claimed; the next natural bench/seat run reads it for free. A drafter-only-only request shape now pays ONE state embed per case it previously skipped (the embed used to sit inside the route-active gate) — bounded by the state length, dwarfed by the drafter pass over the same bytes × options, and per CASE not per question.

### 1b. The GDN chunked prefill verdict — absorbed, residual named

The distill's GOAT lead (`two-chunk-state-independent-overlap-serial-chunk-recurrence`) targets a monolithic serial kernel: chunk n+1's state-independent prologue (gate cumsum, beta scaling, K·Kᵀ, the (I+X)⁻¹ solve) executes inside chunk n's barrier window; only the state handoff stays sequential (24 → 11 barriers on B300).

`riir-infer-gpu`'s `prefill_cuda_gdn_chunked.rs` (the league prefill arm, Issue 734 Arm 12) already absorbs the insight in a STRONGER form: the v3 design hoists the ENTIRE state-independent set for ALL chunks into parallel pre-pass kernels before any serial work — `gdn_pf_decay` (log-γ/ratios) → `gdn_pf_gram` (X/QKR from smem-staged k) → `gdn_pf_tinv` (the explicit (I+X)⁻¹, thread-per-column, batched over all (head, chunk)) — and the serial kernel `gdn_pf_chunk_seq` is pure matvecs with the state row in registers. The source's rule keeps the next chunk's prologue merely OVERLAPPED; this design removed it from the serial chain entirely.

The residual serial chain is the true state-carried dependency — per chunk: rhs/qs0 (the S₀·k_i and S₀·q_i dots), U = T·RHS, O, the rank-64 state update. The one remaining scheduling idea on this surface is the CROSS-CHUNK state-dot recurrence: maintain S·k_j and S·q_j for the next chunk's tokens incrementally (next_dots = γ·cur_dots + K·U with a precomputed cross-chunk token gram), converting the per-chunk 64×128 state dots into 64×64 matvecs. That is a numerics-affecting rewrite of a league-ranked kernel (different summation order — it fits the chunked arm's owner-approved numerics class, argmax-stability + distributional agreement, NOT bit-identity) and therefore a MEASURED-GOAT re-arm on the 4090 prefill row, exactly as Research 221 filed it (detection-only, the riir-ai/riir-infer owning session). NOT done blind here — no bench was in scope, and a blind numerics change to the league kernel is how the 1.55-rel lesson (the module doc's own v1 record) happens.

## What was deliberately NOT done

- **No model load** — no laya/candle/jev weights touched; every gate is synthetic.
- **No bench** — no G2/latency claim; the engine hoist carries a work-volume argument only.
- **No arena lane** — Part 1 stays blocked (98 GiB); the re-arm triggers stand in issue 054.
- **No CUDA-lane code change** — candidate 4 is Issue-967-adjudicated; candidate 1's residual needs its measured re-arm.

## References

- riir-refine `.research/221_open_jev_fast_cuda_gdn_serving_distill.md` — the source distill (Batch 189 rules)
- riir-reflex `.issues/054_openjev_lane_and_prefix_state_handoff.md` — the lane candidate (blocked) + the laya refutation
- riir-infer Issue 967 — the capture-ladder pricing (candidate 4 status-quo)
- riir-infer `crates/riir-infer-gpu/src/prefill_cuda_gdn_chunked.rs` — the v3 chunked prefill (candidate 1 absorbed)
