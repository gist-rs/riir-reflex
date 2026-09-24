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
- the laya-family comparison lane + harness (opt-in `laya`, T1.4/T1.5):
  native-Rust forward over their Apache-2.0 safetensors UNMODIFIED, G5
  parity gate before any published number, CI-regenerated honest tables

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
| The public arena site + distribution | Phase 2 (separate plan, GREEN-LIT, not this repo's Phase 1) |

## May depend on

| Crate | Location | Condition |
|---|---|---|
| katgpt-core | `../katgpt-rs/crates/katgpt-core` | non-optional, `default-features = false` — the ONE code-level dep (the modelless substrate: `decision_wire`, `action_bridge`, `sigmoid_calibration`, `distance_abstain`, `compression_drafter`, `variable_rank_domain_expert`) |
| riir-infer | `../riir-infer` | **pre-declared** (Issue 008 P2, 2026-09-22): the laya lane's future home — when `src/laya` moves there (Backend trait, flat-Vec ops, encoder/head, MSL kernels, tokenizers substrate, G5 fixtures), reflex consumes it via its existing `laya-riir` feature (a `pub use` shim; public API unchanged). **riir-infer ONLY — never riir-ai** (the layering survives: reflex → riir-infer → katgpt-core, all public). No dep edge lands until the P2 move itself (the row-before-edge discipline; C3 never sees an undeclared measured edge) |
| serde_json | crates.io | non-optional — the HTTP/JSON edge ONLY (cold path; hand-rolled JSON would be absurd duplication — the Plan-005 nuance). No engine code reads it |
| serde + derive | crates.io | non-optional — the `/feedback` envelope at the HTTP edge (rides katgpt-core's non-optional serde in-tree; declared for the derive feature) |
| gemm | crates.io | opt-in `laya-riir` ONLY (`.issues/002`) — the pure-Rust GEMM kernel, the riir-owned forward's ONE kernel dep; WAS version-matched to candle's CPU backend — the rationale died with the candle lane (`.issues/006` T5); floats freely now, ANY bump re-runs G5 at both postures (the gate law) |
| libm | crates.io | opt-in `laya-riir` ONLY (`.issues/003`) — `libm::erff` (0.2, PINNED ON NUMERICS GROUNDS, not build-sharing): bit-identical gelu so the G5 drift budget is spent on reductions, not an erf approximation; the pin survives the candle lane's removal (`.issues/006` T5) |
| metal + objc2 | crates.io | opt-in `laya-riir-metal` ONLY (`.issues/005`) — the Apple Metal backend of the riir-owned forward; WAS version-matched to candle's Metal lane deps — floats freely post-candle (`.issues/006` T5), G5 re-run on any bump; macOS-only BY CONSTRUCTION — never in the release feature set (the release matrix ships linux/windows) |
| tokenizers | crates.io | opt-in `laya-riir` ONLY — the HF fast-tokenizer implementation, loads the pinned BPE JSONs directly. **v1 attempted + measured NEGATIVE** (`.issues/006` T3): 1.0.0-rc.2 refuses the pinned english/typed tokenizers (14 genuinely-missing ByteLevel byte atoms — 0.22 lazy, v1 strict); stays 0.22 until 1.0.0 stable relaxes that |
| sha2 | crates.io | opt-in `laya-riir` ONLY — the WEIGHT pins are SHA-256 (the HF LFS oids, an external fact); the house blake3 rule yields to the pin's own hash family |
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
