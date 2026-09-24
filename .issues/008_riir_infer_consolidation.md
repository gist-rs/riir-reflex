# Issue 008 — the riir-infer consolidation: check the Metal lane against riir-ai's kernels; verdict + refactor plan for the public LLM-inference substrate

> **Numbering note:** born as `007_riir_infer_consolidation.md`, renumbered
> `008` the same day — a same-worktree allocation race with the sibling
> session's `007_harness_corpus_neuron_db_substrate.md` (committed first,
> `6759a79`; first commit wins per the collision convention, and this file
> had zero inbound mentions). The `.highwater` tell: the sibling's bump to
> 7 made this session's own `7`-write a silent no-op. Re-scan at WRITE time
> is the rule; a shared worktree makes the window between read and write
> the hazard.

**Status:** OPEN — P0 + P1/T3 LANDED 2026-09-22; **T5 fence gate + P3
slice 1 LANDED 2026-09-23** (see tasks below; plan: riir-ai
`.plans/610_riir_infer_gpu_carve_slice1.md`). Remaining: P2/T4 (encoder move — **UNBLOCKED 2026-09-24**: the harness
sibling WIP landed, worktree clean; the old WAITING note was stale —
Claude verdict r1), **S6b landed 2026-09-24 at honest scope (the edge-drop franchise owner-gated on the training-families home; 12/81 mirrors measured)**, **S7 landed 2026-09-24 (riir-ai `498b1732b9` — dead riir-gpu-async row removed; riir-router measured NOT dead; flip residue complete by compilation)**, P5/T7 (gated on T4). P4/T6 is
DONE (task row below). Owner directive 2026-09-22: *"file issue to
check about
metal lane against riir-ai, we maybe separate around that (riir-infer-core)
so it can consolidate with related metal and consume by both … we plan to
open src infer and reflex btw so maybe new repos named riir-infer to contain
that context … the main idea is open src infer part that related to llm
without leak other part — do verdict for possibility and how to refactor"*.

Refined 2026-09-22 against the code + riir-ai Proposal 041: this issue IS
041's Phase 2 pull-trigger **T-B firing** (§Alignment below); three factual
corrections landed (LN is bias-free mean-centered, not with-bias;
tokenizers **0.22** moves — 006 T3's v1 bump is deferred-on-a-negative;
naming follows 041 Q2's decided answer, name-unchanged, no shim).

## The owner plan (restated so the issue can be adjudicated against it)

1. New repo **`riir-infer`**: the LLM-inference substrate — weights /
   quant / models / ops — **public**.
2. It consolidates **"the related metal"**: riir-ai's CubeCL/CUDA kernel
   layer AND reflex's hand-MSL encoder kernels — one op layer, not two.
3. Consumed by BOTH: riir-ai (re-export, the Proposal 041 zero-breakage
   law) and riir-reflex (the laya lane's backend).
4. riir-infer AND reflex open-source — a **Research 003 amendment**
   ("anything riir-* is internal, no exceptions" gets its first sanctioned
   exceptions), owner authority, recorded here and dated in the executing
   plan.

## Alignment with riir-ai Proposal 041 (checked against code 2026-09-22)

Proposal 041 owns this seam: its Phase 1 extracted `riir-infer-core`
(2026-08-27, 44 files / 30,373 LOC, zero-breakage same-path re-exports from
riir-engine) and its §Session 5 decided Phase 2 = **GO-behind-a-pull-trigger**
with three fires — T-A (measured graph pain), **T-B (a repo outside riir-ai
needs `riir-infer-core`/`riir-gpu` WITHOUT `riir-engine`)**, T-C (riir-ai
contention top pain). **This issue IS T-B firing** — reflex needs the
inference substrate engine-free — so the deferral is discharged and the
promotion runbook is 041's T2.1–T2.4, extended by P2/P3 below. 041's
sequencing rule binds: the carve is the FIRST act of this campaign, never
mid-campaign.

Where this plan refines (not contradicts) 041:

- **Naming — follow 041 §Session 5 Q2's decided answer: keep the crate
  name `riir-infer-core`** inside a repo named `riir-infer`. Moving the
  crate name-unchanged needs **no compat shim at all**: riir-engine's T1.2
  re-exports (`pub use riir_infer_core::{quant, types, deltanet,
  transformer, rope, gemma_layer, llama_layer, ternary_layer, simd,
  spec_types, wall, safetensors_loader, gguf_loader, dflash}` — verified in
  engine lib.rs) keep resolving once the dep path retargets. A public rename
  crate → `riir-infer` remains an option but must explicitly supersede 041
  Q2 and pays the shim — owner call at execution.
- **P3 supersedes 041's crate-contents row "`riir-gpu` moved verbatim"**:
  measured in Check part 2, riir-gpu carries never-move surfaces (game/,
  game_mux, weaver_gpu*, gpu_thoughtfold, memory_soup_gpu, rosetta_gpu,
  moa, spec_marketplace + the riir-router dep). A wholesale move would ship
  private game/cognition code into a public repo. Only the CLEAN modules
  move; the residue stays as riir-ai's riir-gpu and DEPENDS on riir-infer
  (041's own dep-direction row `riir-gpu ─► riir-infer-core`, unchanged).
- **Riders adopted from 041's runbook:** T2.3 (riir-train + riir-clippy
  path deps retarget so inference-only consumers stop dep-ing riir-ai —
  riir-clippy's opt-in arms were 041's own named first candidate), and the
  D4 widening re-narrowing (riir-ai BOUNDARY D4: bundled into the promotion
  — re-narrow what the new structure makes unnecessary, as part of T2.x,
  never before).

## Check part 1 — is there really a consolidation? (metal lane vs riir-gpu)

Op-by-op, reflex `src/laya/riir/metal.rs` (MSL) vs `riir-ai/crates/riir-gpu`:

| op | reflex MSL | riir-gpu | verdict |
|---|---|---|---|
| gemm | stride-general (one kernel, stride args) | `matmul_cubecl` + the ternary gemm/gemv families | OVERLAP — unify on one op layer |
| elementwise | add / add_bias / scale / relu / gelu-erf / glu | `elementwise_cubecl` (+ fused GEGLU gemv) | OVERLAP |
| norm | true LayerNorm, **bias-free** (mean-centered `ln_rows`; encoder.rs pins "NO bias tensors anywhere in the encoder" — the trait op is `layer_norm_nobias_into`) | `norms_cubecl` — RMSNorm-shaped, no mean-centering (decoder) | GAP — **mean-centered LN** is reflex's seed; port in |
| softmax | row kernels | fused in attention + `cpu_reference::softmax` | OVERLAP-ish |
| rope | MSL row kernel | rope kernels (decode/causal-shaped) | OVERLAP (rope is rope) |
| attention | **non-causal**, alternating full + sliding-window layers (bidirectional window `lo=qi−w, hi=qi+w`; additive `f32::MIN` mask via the Backend `add` — composed from primitives, not a fused kernel) | causal KV-cache / GQA / block-causal decode family | GAP — no non-causal kernel exists there; reflex's is the seed |
| runtime | per-op command buffers (objc2) | `cubecl_runtime` / context | RUNTIME CALL — one dispatch layer; the Backend trait stays the seam |

Verdict: the overlap is real (~60% of the op surface) and the two GAPs are
exactly what reflex already wrote. This is two seeds of one op layer, not a
from-scratch build.

Verified against code 2026-09-22 (every claim the table rests on):
reflex — one stride-general `gemm_tiled` ✓, `ln_rows` mean-centering MSL ✓,
`softmax_rows` ✓, `apply_rope`/`rope` MSL ✓, the bidirectional window
construction ✓, per-op command buffers over objc2-metal (lazy-flush) ✓;
riir-gpu — `MatmulCubeCL` (`matmul_cubecl`) ✓, `elementwise_cubecl`
(Split4/Situ) ✓, `norms_cubecl` (RmsNorm* — no mean-centered variant) ✓,
`cpu_reference::softmax` ✓, fused `attention_cubecl` (causal/GQA) ✓.

## Check part 2 — how extractable is riir-ai's kernel layer? (measured 2026-09-22)

- **`riir-infer-core` is ALREADY the clean leaf — Proposal 041 did the
  hard part.** Its deps are katgpt-core + katgpt-transformer /
  katgpt-speculative / katgpt-forward / katgpt-quant / katgpt-attn (all
  PUBLIC katgpt-rs crates) + crates.io only. **Zero `riir-*` deps**
  (verified: the manifest is exactly those six + half/rayon/anyhow/memmap2/
  fastrand/bytemuck/serde/serde_json/thiserror/log/blake3). Its
  own header: "NO cognition (measured: 0 cognition imports), NO
  training". It owns GGUF loading, the quant zoo (q2k → q8kv, q2_0
  ternary, ptq), architectures (gemma/llama/ternary/wall layers,
  deltanet, transformer, rope), the safetensors loader, CPU references.
  Today it sits inside the riir-ai workspace with `publish = false`
  (Research 003) — this move makes it a repo and lifts that.
- **riir-gpu's engine coupling is thinner than the grep suggests.** Its
  `riir_engine::quant::q8kv`, `riir_engine::deltanet::forward::*`,
  `riir_engine::types::DeltaNetLayerType` imports RESOLVE THROUGH
  riir-engine's T1.2 re-exports of riir-infer-core. Flip the re-export
  direction (engine re-exports FROM riir-infer) and those import paths
  keep resolving — the vocabulary does not move a second time.
- **The real hazards are elsewhere in riir-gpu**: `riir-engine` is a
  NON-OPTIONAL dep of the crate (forward drivers live there);
  `riir-gpu-async` is a sibling path dep; and the module list carries
  surfaces that must NEVER move: `game/`, `game_mux`, `weaver_gpu*`,
  `gpu_thoughtfold`, `memory_soup_gpu`, `rosetta_gpu`, `moa`,
  `spec_marketplace` (triage), plus `riir-router` (routing policy stays
  private).

## The verdict — possibility

**FEASIBLE, and further along than expected.** The owner's instinct is
right, and the extraction seam already exists inside the workspace; this
move is that seam made into a repo.

Net-new IP leak from opening: **LOW**. The ternary/modelless approach is
already public (katgpt-rs IS the public funnel); the quant formats are
GGUF-standard; the architectures are public models (gemma, llama, qwen,
modernbert); kernels are engineering craft, and the perf league already
benchmarks against public opponents. Research 003's moat is
product/chain/tokenomics/cognition — none of it is in the move-set.

Conditions (the fence — each is load-bearing, none optional):

1. **Research 003 amendment recorded** — dated, owner authority, in the
   executing plan; reflex's "Private forever" language is amended in the
   same stroke.
2. **Fresh git history** — the public repo starts at the carve commit; no
   workspace narrative in commit messages (the cargo-heal dist-repo
   pattern, with source this time).
3. **Sanitized public docs** — new README + public BOUNDARY; the
   workspace AGENTS.md narrative, Research 003 quotes, perf-league
   internals do NOT ship. Internal lineage stays recorded in riir-ai.
4. **Module fence enforced by CI** — a grep gate in riir-infer red on any
   cognition/game/router import (the `standalone_dep_gate.sh` pattern).
5. **Contract registration** — `repo_set.txt` + BOUNDARY rows:
   riir-infer → katgpt-rs only; riir-ai → riir-infer allowed
   (dep-direction row added); reflex → riir-infer allowed (reflex
   BOUNDARY amendment: riir-infer ONLY, never riir-ai — the layering
   survives: reflex → riir-infer → katgpt-core, all public).
6. **Weights stay private** — riir-train's checkpoints never move;
   riir-infer ships LOADERS, not weights.

reflex's own opening rides the same conditions with one extra vector: its
docs/history carry the workspace narrative (AGENTS.md quotes Research 003
and names siblings) — the opening pass is a docs-sanitize +
history-strategy work item of its own, separate from the carve.

## The refactor — how (phases; each independently landable)

- **P0 — contract first** (no code): this issue + riir-ai's mirror issue;
  BOUNDARY rows, dep-direction rows, repo_set registration, the 003
  amendment note.
- **P1 — carve v1**: create `/git/riir-infer` (fresh init). Move
  `riir-infer-core` out **name-unchanged** (041 §Session 5 Q2's decided
  naming; path deps `../../../katgpt-rs` → `../katgpt-rs`; workspace
  members updated both sides; riir-engine's dep retargets and its T1.2
  `pub use riir_infer_core::X` re-exports keep resolving — **no shim, zero
  consumer edits**). (Optional public rename crate → `riir-infer` with a
  path-dep compat shim must explicitly supersede 041 Q2 — owner call at
  execution.)
- **P2 — the encoder lane consolidates** ("the related metal"): move
  reflex's `src/laya` (Backend trait, flat-Vec ops, encoder/head, the MSL
  kernels, tokenizers substrate, G5 fixtures + parity gates) into
  riir-infer as the ModernBERT encoder family. reflex consumes via its
  existing `laya-riir` feature = a `pub use` shim — public API unchanged,
  tests keep passing, and the candle-free property is preserved
  (gemm/libm/tokenizers **0.22** move WITH the lane — 006 T3's v1 bump is
  DEFERRED on a measured negative, so the 0.22 pin lands in riir-infer's
  manifest and the 1.0.0-stable reopen trigger rides along; 006's plan is
  otherwise unaffected).
- **P3 — GPU kernel migration** (the big program, separately gated): the
  T2 audit classifies every riir-gpu module CLEAN / SEAM / STAYS; CLEAN
  kernel modules move into riir-infer's gpu layer importing vocabulary
  DIRECT; riir-gpu re-exports during transition; the re-export-direction
  flip (engine re-exports FROM riir-infer) lands here. **Riders from 041's
  runbook, executed here:** T2.3 consumer retarget (riir-train +
  riir-clippy path deps → the new repo) and the D4 widening re-narrowing
  (per 041's D4 discharge).
- **P4 — open**: MIT, sanitized docs, cargo-about licenses, CI (the
  katgpt-rs public-repo pattern: native lanes + the fence gate); then
  reflex's opening pass.
- **P5 — the duplicate retires**: the hand-MSL op layer and the CubeCL op
  layer unify behind the Backend trait; A/B against the chart numbers; G5
  at both postures; the loser is deleted. This is where the "rewrite
  candle twice" debt actually gets paid down — AFTER the chart exists,
  never before (005 T5 sequencing stands).

## Sequencing vs 005 / 006

Non-blocking and non-overlapping: 005's riir-Metal lane lands first and
takes the chart (006 T1 unchanged); 006's candle removal proceeds
(riir-infer is candle-free by construction — the lane moves into it
carrying the tokenizers-0.22 pin/gemm/libm; 006 T3's v1 bump stays deferred
with its reopen trigger). 006's "re-open path, deliberately not
wired" paragraph is SUPERSEDED by this owner pull — this issue is the
wiring.

## Tasks

- [x] **T1** riir-ai mirror issue + BOUNDARY / dep-direction rows + the
      003-amendment note (P0) — landed 2026-09-22: mirror is riir-ai
      `.issues/996`; BOUNDARY rows (Owns strikethrough, Does-not-own,
      May-depend-on, CANONICAL matrix, D4 T-B annotation) + the Research
      003 dated amendment + this repo's BOUNDARY amendment (Private-forever
      note + the pre-declared riir-infer row) all in the same window.
- [x] **T2** The riir-gpu module audit: per-module CLEAN / SEAM / STAYS
      classification — recorded as the table in riir-ai `.issues/996`
      (171 modules: CLEAN 98 / SEAM 34 / STAYS 39 / UNRESOLVED 0;
      comment-stripped grep + per-line verification; the verified 14-entry
      T1.2 re-export list; riir-router + riir-gpu-async measured as
      zero-reference dead deps from riir-gpu's side).
- [x] **T3** Carve riir-infer v1 (P1): repo + crate move (name-unchanged
      per 041 Q2 — no shim); both workspaces green; `cargo tree` proves
      riir-infer has zero riir-* deps; the engine re-exports compile clean
      (no flip needed at P1). The 041 riders (T2.3 retarget + D4
      re-narrowing) execute with P3/T7, not here. — LANDED 2026-09-22:
      gist-rs/riir-infer @ 86a5986 (fresh history, sanitized docs posture
      from birth); riir-ai retarget + gate re-points landed with it;
      Cargo.lock byte-identical (path deps record no source); registered
      as the 23rd contract repo (katgpt-rs bd2ce3cca); 4090 synced
      (E:/git/riir-infer).
- [x] **T4** Encoder-lane move (P2): reflex `src/laya` → riir-infer; the
      reflex shim; G5 green from the new home (same fixtures, same
      gates); the tokenizers **0.22** pin lands in riir-infer's manifest
      (006 T3: the v1 bump is deferred on a measured negative — the
      1.0.0-stable reopen trigger rides along, nothing else changes).
      **Unblocked 2026-09-24** (harness sibling WIP landed; stale WAITING
      note cleared — Claude verdict r1). **Three measured costs ride the
      slice (Claude verdict r1, manifest-verified):** (a) `metal`/`objc2`
      land TARGET-SCOPED in riir-infer's manifests
      (`[target.'cfg(target_os = "macos")'.dependencies]`, the
      riir-infer-gpu `metal_tensor_gemm` pattern) — reflex carries them
      unscoped-optional and riir-infer CI runs `--all-features` on
      ubuntu; never "fix" by narrowing CI's feature coverage (that
      trades a red lane for silent coverage loss); (b) `metal` version
      divergence reflex 0.29 vs infer-gpu 0.31 — one workspace, one
      version; price the 0.31 bump (the G5 re-run is the acceptance),
      never two `metal` crates in one graph; (c) riir-infer
      `BOUNDARY.md` gains the consumer row in T4's FIRST commit (reflex
      pre-declares the edge; the boundary contract must not see an
      undeclared measured edge). Record rules: G5-from-the-new-home is
      an M3 **workstation** verdict (box state beside the numbers —
      riir-infer CI is ubuntu and can never measure it); reflex's T4
      record cites the riir-infer commit SHA; riir-infer commit
      messages stay sanitized (no internal numbers / sibling names /
      box references) and REBASE onto the squashed line, never merge.
      — **LANDED 2026-09-24**: riir-infer `c6716a4` + reflex (this
      commit).
      **Layout:** new lane crate `crates/riir-infer-laya` (the
      gpu-crate member pattern — lane-free consumers never resolve the
      tokenizers/gemm tree). Moved: the whole `src/laya/` tree
      (substrate: config/lang/render/router/temps/tokenize/types/weights
      + the `riir/` forward: agent/backend/encoder/head/metal/ops/weights
      — 16 files, byte-faithful at reflex-HEAD content) + `src/pyjson.rs`
      (the UNGATED Python-JSON writer — one DRY home, now
      substrate-side; the reflex file is a re-export shim so the
      ungated harness keeps compiling at default features — the reason
      the reflex dep is NON-OPTIONAL) + `tests/metal_ops_smoke.rs`
      (PROMOTED into the lane crate per its own "delete or promote
      before the lane lands" note; one dead shadowed binding fixed —
      never before compiled under `-D warnings`). Reflex keeps:
      `src/laya/mod.rs` (glob `pub use` shim — every
      `crate::laya::*` / `riir_reflex::laya::*` path unchanged), the G5
      parity test + frozen fixtures (CONSUMER-side gate), the harness,
      the examples. Features forward: `laya-riir =
      ["riir-infer-laya/laya-riir"]`, `laya-riir-metal = ["laya-riir",
      "riir-infer-laya/laya-riir-metal"]`; `RELEASE_FEATURES` stamp
      unchanged. CI: both reflex workflows gain the `gist-rs/riir-infer`
      sibling checkout (the path dep is always resolved now).
      **Costs:** (a) DONE — metal 0.31 + objc2 are
      `[target.'cfg(target_os = "macos")'.dependencies]` optional rows
      in the lane manifest; every metal code site is
      `cfg(all(target_os = "macos", feature = "laya-riir-metal"))`
      (already the shape reflex carried); `--all-features` clippy green
      on macOS, structurally clean for ubuntu (the gpu-crate pattern).
      (b) DONE — the moved lane adopts **metal 0.31** (one workspace
      version with the gpu crate; reflex's graph now has ONE metal,
      transitive). **The bump needed ZERO API fixes** — the owned-API
      surface (Device/CommandQueue/ComputePipelineState/
      MTLResourceOptions/CompileOptions/new_library_with_source) held
      0.29→0.31; acceptance below. (c) DONE — riir-infer BOUNDARY.md
      carries the lane dep rows + the consumer row (`gist-rs/riir-reflex`,
      one-way, zero back-edge) IN c6716a4 (the first commit); fence
      gate self-ref exemption extended to the third own crate;
      359 .rs walked, 0 findings 0 pins.
      **Gates (M3, `CARGO_TARGET_DIR` isolated):** riir-infer: clippy
      `-D` green at default / laya-riir / laya-riir-metal /
      `--all-features`; fence green; lane lib tests 7/7 (pyjson);
      `metal_ops_smoke` 7/7 on metal 0.31; workspace test suite green.
      reflex (via shim): clippy `-D` green at default / laya-riir /
      laya-riir-metal; `cargo test --workspace` green (38+3+8+14+7+33
      +7+9, 0 failed). **G5 from the new home, BOTH postures (the
      acceptance):** CPU — english 26/26, typed 26/26, multilingual
      36/36 = 88/88 forwards, top-1 agreement **1.000000** (gate
      ≥ 0.999), worst prob drift **3.013e-6** (gate ≤ 1e-3); METAL —
      88/88, top-1 **1.000000**, worst drift **4.016e-6** (~250×
      headroom; the metal 0.31 bump HOLDS). Box state: M3 Max, macOS,
      aarch64, release profile; load 13.99-18.31 (shared box,
      concurrent agent sessions), RAM 88% free, ON BATTERY 56%
      (discharging) — correctness gates, duration-only sensitivity.
      ⚠ **Sibling-WIP handoff:** a concurrent session's uncommitted
      issue-018 Metal-latency work sat on exactly the moved files while
      this landed. It was NEVER stashed or committed by this session:
      the move carried reflex-HEAD content, the WIP was snapshotted to
      `/tmp/t4_reflex_wip_snapshot/` (full patch + per-file copies),
      and the session then re-applied its work onto the LANE CRATE
      itself (uncommitted edits in riir-infer's working tree observed
      mid-move — the right home; those edits are theirs to land). The
      snapshot also lives DURABLE, untracked, at
      `.issues/020_wip_snapshot_t4move/` in this repo (the /tmp copy is
      reboot-volatile) — the issue-020 session's commit 86a11d3
      recorded its Class-B measurements as CLOSED while the closing
      code was still uncommitted working-tree state; that code is what
      this snapshot preserves. Applies onto
      `crates/riir-infer-laya/src/laya/` with path adjustments if the
      live tree loses it.
      Follow-ups recorded, not done here: the lane's default weights
      cache path still names this repo
      (`~/.cache/riir-reflex/laya` — kept so the 2.2 GB local cache
      stays valid; LAYA_WEIGHTS_DIR/LAYA_HOME override); T7 (op-layer
      unification) remains open.
- [x] **T5** Fence gate (P4 precondition): the CI grep gate red on
      cognition/game/router imports + the public-docs checklist (no
      workspace narrative ships). — LANDED 2026-09-23:
      `riir-infer/scripts/fence_gate.py` (comment+string-masked import
      fence over all foreign `riir_*` tokens with the self-reference
      exemption, path-dep allowlist = `../katgpt-rs` only, membership
      pin file `fence_expected.txt` deliberately empty, walk floors,
      planted-violation self-test on every run) + `.github/workflows/
      ci.yml` (fence + check/clippy/test with the public sibling
      checked out) + `.docs/p4_opening_checklist.md` (the judgment
      half of the sanitized-docs fence) + the toolchain comment
      sanitized (sibling names + internal issue refs removed). Gate
      green at landing: 213 tracked .rs (vendor forks included in the
      walk), 0 findings, 0 pins.
- [x] **T6** Open riir-infer (P4) + the reflex opening pass (docs
      sanitize + history decision) — owner-gated go. — EXECUTED 2026-09-23
      (owner green-light in-session; both repos PUBLIC):
      - **riir-infer**: history squashed to FIVE sanitized commits
        (86a5986 core / 0643575 gpu layer / f8dfc69 doctest +
        reader-protection fixes / acbcdad opening posture — MIT LICENSE,
        cargo-about THIRD_PARTY_LICENSES.md, BOUNDARY private-sibling
        paths sanitized / 6b6de2b the gemma decode stack + gemv split2
        rung, the twin-landing reconciliation). CI green on the public
        repo (run 35835743560 — the private-repo lane never started:
        Actions spending limit; the opening's full-gate execution also
        caught the doctest-blindspot class + a green-zero test target,
        both fixed). The twin S5 landing merged the old history back
        mid-opening — re-squashed ONCE with lease, and the standing
        rule for every future pusher: REBASE onto the squashed line,
        never merge, and keep commit messages free of internal numbers,
        sibling names, and box references. The 4090 checkout was
        fast-forwarded clean; its in-flight S5 work landed via the
        reconciliation untouched.
      - **reflex** (the owner plan's "riir-infer AND reflex
        open-source"): history decision = FRESH two-commit history
        (622b5e5 engine root, c3f9083 opening posture) — the old
        history carried the full campaign narrative; the source-side
        v0.2.0 tag deleted (the dist repo's releases are the install
        record); README/AGENTS/BOUNDARY private-forever statements
        rewritten; LICENSE added; minimal ci.yml (check at the shipped
        release set — NOT --all-features: laya-riir-metal is
        macOS-only by construction — clippy -D, test --release for the
        G2 latency gates); machine-local path scrubbed from the tracked
        benchmark results. Verified green in an isolated worktree
        (clippy -D + cargo test --release) before the flip.
      - `publish = false` stays both sides — crates.io remains
        owner-gated. Remaining in this campaign: T4 (encoder move) +
        T7 (op-layer unification).
- [ ] **T7** Op-layer unification (P5): A/B vs the chart numbers, G5 both
      postures, delete the duplicate. **SEQUENCING CALL 2026-09-24 (this
      session): deferred behind reflex issue 020 T5** — the T5 batched
      forward (riir-infer `da30007`) landed first because S8/T7's A/B would
      otherwise measure a moving target while 020's measured levers were
      still landing; T5 is 020's largest remaining lever, owner-directed,
      and was ungated. Nothing of S8/T7 landed. Re-open after 020's
      publishable batched-vs-loop A/B (Bench 006 Addendum 7) settles the
      numbers this A/B compares against.

## P3 progress (the GPU kernel migration — executing in slices, plan:
## riir-ai `.plans/610_riir_infer_gpu_carve_slice1.md`)

- [x] **S1 LANDED 2026-09-23** — the base GPU runtime cluster moved:
      `buffer`, `context`, `pool_poison`, `weight_buffer_cache`,
      `gpu_transpose`, `cubecl_runtime` (+ the adapter VRAM probe,
      cut from riir-gpu's `vram_budget` — its only caller moved) now
      live in `crates/riir-infer-gpu` in the riir-infer repo.
      riir-gpu re-exports the modules (`pub use riir_infer_gpu::...`,
      cfg-matched) — zero consumer edits; riir-gpu lib suite 426/0
      from the re-export layer; clippy `-D warnings` clean both
      postures both crates. **Two vendor forks ride the slice**
      (byte-identical copies + `[patch.crates-io]` in riir-infer's
      workspace root — patches do not cross workspaces and the
      public repo can never patch from a private sibling):
      `cubecl-runtime` (drop-queue fix, upstream #1359) and
      `wgpu-hal` (the `total_video_memory_bytes()` accessors the
      VRAM pre-flight probes — discovered mid-slice, the audit's
      moving→STAYS edge list did not name it).
- [x] **S2 LANDED 2026-09-23** — the elementwise/norm/matmul/attention
      family (28 files) moved to `crates/riir-infer-gpu`: cpu_reference,
      the GEMV family (autotune/cubecl/f16/geglu/geglu_f16/qkv_f16/q4k
      + batched + rmsnorm-fused + qkv_q4k + geglu_q4k), matmul +
      swap_ab, attention (flash/causal-fused/q8kv + tests),
      gemma2_d2f_sc, norms/sampling/elementwise, epilogue, and
      params_cache (the blake3-keyed params-handle cache — the audit's
      edge list missed it; slice-1's vram-probe class). The five
      quant SEAM imports became `riir_infer_core::quant`; six
      riir-gpu features mirrored + forwarded (swap_ab_gemm, gemma2_d2f,
      q4k_rowtiled_gemv, gemv_fma_contract, fold_dispatch,
      q8kv_sink_guard); half/papaya/blake3 + the riir-infer-core path
      dep added to the manifest; the fence gate's F1 path-dep check
      fixed to real containment (the leading-`..` heuristic would have
      red the in-repo `../..` dep by construction) + selftest arms.
      Zero consumer edits — riir-gpu module re-exports keep every
      `crate::<mod>`/`riir_gpu::<mod>` path resolving (riir-train-engine,
      riir-train-gpu, riir-poc verified green). Tests: riir-infer-gpu
      206/0 at the full-feature combo; riir-gpu lib 283/0 (the 28
      gemma2 tests exercise the moved kernels through the re-exports on
      Metal); one forced divergence recorded (ArgmaxCubeCL
      pub(crate)->pub); found pre-existing bench_603 all-features rot
      (S3 territory, verified not ours).
- [x] **S3 LANDED 2026-09-23** — the ternary gemv/gemm + metal + CUDA-raw
      families (33 files: 31 .rs + 2 .metal) moved to
      `crates/riir-infer-gpu`: the gemv family around the
      gemv_ternary_cubecl hub, the gemm family (incl. the #[path]
      simdgroup_smem child — audit gap noted in the plan), the macOS
      metal-tensor trio (+ the .metal sources; include_str! paths
      resolve same-dir), the CUDA raw prefill family, and
      deltanet_input_proj_fused (CLEAN row, pulled forward by the
      bench_603 rot fix). Nine pub(crate)->pub widenings (recorded in
      the plan) for the remaining SEAM consumers;
      canonical_expand_forms moved with its kernel family; 10 features
      mirrored + forwarded (ternary_gemv drops the riir-engine leg
      infer-side — fence); katgpt-core joins via a new
      [workspace.dependencies] table (fence raw-prefix check stays
      green). bench_663_t5_single_gemm_isolation moved with its kernel.
      **bench_603 rot FIXED** (Issue-980 GateProjWeights ripple;
      pre-existing at HEAD, stash-verified) — and the same rot class
      fixed in riir-train-engine's bonsai-go lane (f5bfe3f9, 1691/0).
      Tests: infer-gpu 225/0 all-features (Metal); riir-gpu 267/0
      default + the 4 gemma2_d2f re-export-path tests pass individually
      (the all-features lib suite is jetsam-killed on the loaded M3 AT
      HEAD TOO — stash-verified, box-state not slice); clippy -D at all
      postures both crates. Commits: riir-infer `0c2f3b4`, riir-ai
      `027aef2d6`, riir-train `f5bfe3f9`.
- [x] **S4a + S4b + S5 LANDED 2026-09-23** (the 4090 box; full records in
      plan 610): S4a the qwen-attention + deltanet CLEAN kernels (12 files);
      S4b the qwen38/cudarc SEAM cluster + the deltanet-forward family +
      ane_prefill (31 files ~45k LOC) — the two S4 adjudications RESOLVED
      (backward.rs → riir-train-gpu as `cudarc_backward_kernels`; the L179
      gated training pair stripped, zero external callers) — **the T2.3
      retarget UNBLOCKED** (`TernaryDeltanetGpuForward` re-exported at
      `riir_gpu::`); S5 the gemma-cluster unlock (wall_config re-homed to
      infer-core; gemma2/gemma4/llama + cluster riders, 17 files ~13.6k
      LOC). A concurrent M3 S4b landing was reconciled at riir-infer
      `4f354f5` (the Issue-825 class; the twin-landing lesson: fetch before
      EVERY slice commit). Post-S5 owner pass fixed the Issue-1001 d2f
      threshold (eb3438f) + restored the S5 gemma block lost in the S4b
      rebase (d058f48).
- [x] **S6a LANDED 2026-09-24** (the M3 box; the T2.3 clippy retarget + two
      carve riders — full record in plan 610): riir-clippy's
      `ternary_inference` lane consumes `riir-infer-core` + `riir-infer-gpu`
      directly — its two riir-ai inference edges are GONE (the first
      consumer fully off riir-ai for inference; the 041 T-B candidate).
      Riders: `tokenizer.rs` moved riir-engine → infer-core (the bench's
      BpeTokenizer; gguf_loader-resident; sentencepiece row follows; engine
      same-path re-export, zero consumer edits) and `speculative_decode/`
      moved riir-gpu → infer-gpu (the modelless CPU-side drafters; module
      path re-exported, maglev lanes unchanged); infer-gpu root gains the
      item re-exports making the consumer swap textual. The arc-swap
      genlock row STAYS in riir-clippy, measured: riir-rag's OPTIONAL
      riir-engine dep is feature-resolved BEFORE pruning, so the row is
      load-bearing for every posture until the rag embedder edge itself
      retargets. D4 re-narrowing landed for this consumer (riir-ai BOUNDARY
      CANONICAL row → riir-rag only, per 041's T2.x bundling). Boundary
      contract clean 23 repos / 321 edges; the retargeted bench runs
      end-to-end on Metal (prefill 29.2 tok/s). Commits: riir-infer
      `8ecbc7f`, riir-ai `50f644f3d`, riir-clippy `cda28b7e`.
- [x] **S6b LANDED 2026-09-24 at honest scope** (the 4090 box, in a
      `riir-train.s6b` worktree — sibling session live in the main checkout,
      zero overlap measured; riir-train `7e0e7d74`, riir-ai `75b7630b38`).
      Landed: patch rows re-pointed at `../riir-infer/vendor` (byte-identical
      forks, `diff -r` rc=0) + the TrainingProvider adjudication (stays
      engine-side; ZERO impls + ZERO engine-side consumers measured) + the
      stale-description fixes. **The full edge-drop measured BLOCKED: only
      12 of riir-train's 81 forwards are mirrored in infer-gpu — the 69
      training families stay riir-ai-side BY DESIGN (riir-infer is public,
      Research 003).** Owner-gated now: the training-families long-term home.
      Partial swap evaluated + rejected (two vocabularies = churn). Full
      record: plan 610's S6b row (riir-ai). Validation: default + CUDA
      posture checks, clippy -D, boundary contract all green on the 4090.
- [x] **S4+** — DELIVERED IN FULL across S4a+S4b+S5 (the SEAM adjudications
      — backward.rs → riir-train-gpu, the L179 pair stripped — and the
      gemma-cluster WallConfig re-home), S6a (the T2.3 clippy retarget +
      D4 re-narrowing for riir-clippy) and S6b (the riir-train patch-row
      re-point + adjudications; the full edge-drop owner-gated on the
      training-families home), with S7 completing the flip residue. The
      remaining open item of this issue is T7 only.
