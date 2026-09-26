# riir-reflex — boundary contract

> The single source of truth for what may live in and depend on this repo.
> Audited by the `boundary-guard` skill +
> `../riir-ai/scripts/ci_boundary_contract.sh`
> (they read this file; findings are contract violations or contract rot).
> Cross-repo rules LINK to their one canonical home — never copied.
>
> Drift ledger — Disposition: `fixable` | `owner-call` | `by-design`.
> `fixable`/`owner-call` rows REQUIRE an open issue (row ⟺ open issue); a
> `by-design` row cites the decision record instead. Issue closes → row removed
> in the same commit.
>
> Filed 2026-09-21 at repo birth (Plan 603 T1.1, owner-delegated).

## Owns

**Decision-engine serving + comparison + contribution** — the Proposal 014
Phase 1 deliverable (Plan 603):

- the modelless decision engine answering
  `katgpt_core::decision_wire` requests (`choice`/`score`/`noul`,
  abstention as a first-class answer): hashed-feature embedding →
  corpus-expert routing (`pick_domain` over unit centroids) →
  corpus-is-the-model option scoring (`Lz4FlexDrafter`) → sigmoid
  normalization → calibrated confidence (`SigmoidGateCalibrator`, the
  substance of the calibrated bridge on a dynamic option space) → fused
  abstain (score + `CorpusDistanceGate`)
- the confidence readout dispatch (narrow = inverted label entropy, wide =
  argmax-label-prob; Bench 817's verdict inherited, never re-derived)
- the localhost serving edge: ONE std-only HTTP binary, no daemon framework
- the laya-family comparison lane + harness (opt-in `laya-riir`, T1.4/T1.5):
  the lane SUBSTRATE moved to the inference-solution repo's
  `riir-infer-laya` crate (the encoder-lane move, Issue 008 T4 — this
  repo consumes it through the `src/laya/mod.rs` shim, public API
  unchanged); what stays HERE is the consumer side: the G5 parity gate
  (frozen fixture corpus + expected capture — a lane number is only
  published from a posture this gate greened), the harness + honest
  tables, and the Python-JSON writer's reflex-side re-export

**Domain test:** is this **decision-engine serving + comparison +
contribution** (NOT game runtime, NOT code healing)? NO → it belongs in
another repo; file there.

**The new-repo decision (the boundary-test passage):** a pattern is
borrowed, not a dependency — a pattern reusable in ~200 LOC with zero
game-domain coupling from ANY feature combination justifies a NEW repo
(the quest-grammar precedent). The counter-case that decides the other
branch is riir-ai Plan 484 (`../riir-ai/.plans/484_riir_games_domain_split_corrected.md`):
moving engine substrate INTO the game domain was REFUSED for bidirectional
coupling — the engine must be consumable WITHOUT the game stack, so it
lives here, upstream-clean. Zero game deps, zero Python deps (owner
directive — the laya lane is the native-Rust port, cost accepted).

**Private forever** per Research 003 — never part of the public katgpt-rs
strip. **Amended 2026-09-22** (owner directive, recorded in Research 003's
dated amendment + riir-ai Issue 998 / this repo's Issue 008): this repo is
one of the FIRST SANCTIONED EXCEPTIONS — directed to open source. The
opening (P4/T6) executed 2026-09-23 on the owner green-light: the repo is
public; `publish = false` stays until the owner decides crates.io
publication.

## Does not own

| Concern | Correct home |
|---|---|
| Engine primitives (wire contract, action bridge, calibrator, distance gate, drafter, pick_domain) | `../katgpt-rs` (`crates/katgpt-core`) — landed primitives are CONSUMED, never re-implemented (substrate-first) |
| The `structured_read` primitive + its root promotion | `../katgpt-rs` (`crates/katgpt-forward`) — the consumer line lands THERE when this engine first consumes it, never before |
| The laya reference models/protocol authorship | upstream (Apache-2.0) — mirrored UNMODIFIED under `.raw/` at a full sha before any published table (T1.4 pin) |
| Game runtime (NPC cognition, sync, perception) | `../riir-ai` |
| Code healing / lint tooling | `../riir-clippy` |
| The public arena site (static pages, charts, published data mirrors) | `gist-rs/reflex-site` (`../reflex-site`) — the product's site, fed FROM here: harness output → its `data/bench.json` publisher, doc/SVG mirrors synced by its `scripts/sync_mirror.py` (plan: `../katgpt-rs/.plans/606_reflex_phase2_site_distribution.md`). Display name: **Reflex** (lanes: *Reflex · modelless* = this engine, *Reflex · rulebook* = `../riir-reflexer`); KatGPT is credited as the substrate, never a lane name |
| The binary release channel | `gist-rs/reflex` (+ Homebrew tap / Scoop bucket) — built from this repo (`scripts/build-release.sh`, `dist/`), published there |

## May depend on

| Crate | Location | Condition |
|---|---|---|
| katgpt-core | `../katgpt-rs/crates/katgpt-core` | non-optional, `default-features = false` — the ONE code-level dep (the modelless substrate: `decision_wire`, `action_bridge`, `sigmoid_calibration`, `distance_abstain`, `compression_drafter`, `variable_rank_domain_expert`) |
| riir-infer-laya | `../riir-infer` (`crates/riir-infer-laya`) | **LANDED** (Issue 008 T4, 2026-09-24; pre-declared 2026-09-22 row-before-edge): the laya lane's home — tokenizer / config / weights substrate, flat-`Vec` forward, the MSL Metal backend (macOS target-scoped there). reflex path-deps `riir-infer-laya` behind its existing `laya-riir` / `laya-riir-metal` feature names (forwarding); the dep is NON-OPTIONAL only because the UNGATED Python-JSON byte writer moved with the lane (the harness consumes it at default features — at rest the crate compiles just that writer). **riir-infer ONLY — never riir-ai** (the layering holds: reflex → riir-infer → katgpt-core, all public). One-way edge, zero back-edge |
| lane deps — VIA the substrate crate now | `riir-infer-laya`'s own manifest | the lane's dep rows (tokenizers **0.22 pinned on a measured negative** — the 1.0.0-rc line refuses the pinned BPE files; sha2 — the weight pins are SHA-256, an external fact; gemm 0.18; libm 0.2 **numerics pin** — bit-identical erf; macOS target-scoped metal **0.31** + objc2) moved with the lane; ANY bump of any of them re-runs G5 at both postures before a number is published (the gate law) |
| serde_json | crates.io | non-optional — the HTTP/JSON edge ONLY (cold path; hand-rolled JSON would be absurd duplication — the Plan-005 nuance). No engine code reads it |
| serde + derive | crates.io | non-optional — the `/feedback` envelope at the HTTP edge (rides katgpt-core's non-optional serde in-tree; declared for the derive feature) |
| gemm | crates.io | MOVED with the lane (now `riir-infer-laya`'s dep — see its manifest row); was the riir-owned forward's ONE kernel dep (`.issues/002`) |
| libm | crates.io | MOVED with the lane (now `riir-infer-laya`'s dep); was `libm::erff` 0.2, PINNED ON NUMERICS GROUNDS — bit-identical gelu so the G5 drift budget is spent on reductions (`.issues/003`) |
| metal + objc2 | crates.io | MOVED with the lane (now TARGET-SCOPED optional in `riir-infer-laya`, macOS only — `.issues/005`); enabling `laya-riir-metal` on Linux/Windows is inert, never a dep-tree failure |
| tokenizers | crates.io | MOVED with the lane (now `riir-infer-laya`'s dep, still **0.22** — `.issues/006` T3's measured v1 negative stands; reopen at 1.0.0 stable) |
| sha2 | crates.io | MOVED with the lane (now `riir-infer-laya`'s dep — the weight pins are SHA-256, an external fact) |
| blake3 | crates.io | opt-in `laya-riir` OR `corpus_db` — the small-file pins (house hash) + the G5 pairing check + the corpus-row digests (Issue 007 P1) |
| `ndb` binary (runtime, NOT cargo) | `../riir-neuron-db` `target/release/ndb` (local build) or PATH / `NDB_BIN` | opt-in `corpus_db` ONLY (Issue 007 P1) — the harness's Warm-tier store as a SUBPROCESS (`std::process`), `--json`-only, writes via stdin never argv, one-corpus-one-row, consumer-side golden pin over the CLI wire. **Zero cargo dep on the storage leaf** — the no-source-leak posture is structural; NATIVE-ONLY (never a wasm32 combo) |

Explicitly NOT allowed from ANY feature combination: any game crate
(`riir-games*`, the engine facade, the game SDK — naming none of them in
code or manifest, ever), any Python dependency or sidecar (owner directive:
no Python required is the product story), and — since `.issues/006`, owner
directive "no candle at all cost" — **candle in any feature, any target,
any profile** (the reference lane's deletion is contract, not cleanup;
`grep candle Cargo.lock` stays at zero rows).

## Inherited boundaries (links)

- Dep-direction matrix + CANONICAL rows: `../riir-ai/BOUNDARY.md`
- Modelless-first mandate (exhaust freeze/thaw / reader-LoRA / latent-space
  before any training dependency): `../katgpt-rs/AGENTS.md`
- Wire-contract authority (golden pins, fail-closed validation):
  `../katgpt-rs/crates/katgpt-core/src/decision_wire.rs`

## Drift ledger (target vs actual)

None at birth — seeded empty by Plan 603 T1.8 (the ledger fills only with
a row + its open issue, never with silence).

| ID | Surface | Target | Actual (verified) | Workaround | Issue | Disposition |
|----|---------|--------|-------------------|------------|-------|-------------|
