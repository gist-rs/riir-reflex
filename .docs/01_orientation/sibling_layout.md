# Sibling layout — the day-one dependency graph artifact (Plan 603 T1.1c)

Path deps assume this on-disk layout:

```
/git/riir-reflex      ← this repo
/git/katgpt-rs        ← katgpt-core (the engine substrate, non-optional)
/git/riir-infer       ← the laya lane substrate (`crates/riir-infer-laya`, non-optional
                        path dep — the lane MOVED here 2026-09-24, Issue 008 T4; the
                        ungated pyjson writer keeps the manifest always-resolved)
/git/riir-reflexer    ← the vessel format repo (`crates/reflexer-vessel`, optional path
                        dep activated by `vessel_public_read`, default-on — the head
                        lane's PUBLIC-RELEASE reader + the mint path's public writer)
/git/riir-ai          ← NOT a dep (game runtime; the boundary counter-case is
                         Proposal 017: engine substrate must be consumable
                         WITHOUT the game stack, so no edge exists in either
                         direction)
```

## The dependency law

- **The ENGINE substrate is `katgpt-core`**
  (`default-features = false`, six named features forwarded through this
  crate's `modelless` feature — see the root `Cargo.toml`). Everything the
  decision engine does rides LANDED katgpt-core primitives; nothing is
  re-implemented here (substrate-first).
- **`reflexer-vessel` joins ONLY behind `vessel_public_read`**
  (default-on, instinct Proposal 001 T3+T4): the heads lane's signed
  PUBLIC-RELEASE reader + the mint path's public writer. The feature set
  is the capability law (A1): `vessel_public_read` is the ONLY reader
  capability this repo selects — the HOSTED-ONLY reader has no selectable
  path in ANY feature combination (the moat), and no GAME-IP content may
  ever ride a PUBLIC-RELEASE vessel through this repo (A10).
- **`serde_json`** is the second manifest dep and it is EDGE-ONLY: the
  localhost HTTP/JSON boundary (cold path). Engine code never touches it.
  Re-deriving a JSON parser would be absurd duplication — the Plan-005
  nuance that sanctions opt-in whole-substrate consumption.
- **The candle lane is GONE** (`.issues/006`, owner directive 2026-09-22
  "no candle at all cost") — the `laya` feature, candle-core dep, and its
  G5 twin were deleted; `laya-riir` is the ONE laya backend. The
  `grep candle Cargo.lock` = zero-rows property is contract
  (BOUNDARY.md §Not-allowed).
- **`gemm` + `libm` join ONLY behind the opt-in `laya-riir` feature**
  (`.issues/002` + `.issues/003` — the riir-OWNED forward, owner directive
  2026-09-22): the lane is candle-FREE; both deps WERE version-matched to
  candle's CPU calls — the rationale died with the candle lane
  (`.issues/006` T5: `gemm` floats freely with a G5 re-run on any bump;
  `libm` stays pinned on NUMERICS grounds — bit-identical gelu). Each
  landed in the same commit as its BOUNDARY.md row + the lane's own G5
  parity `[[test]]` row (`required-features = ["laya-riir"]`).
- **No game crate is reachable from ANY feature combination** — the
  new-repo decision (root `BOUNDARY.md` §Owns) rests on that property.
- **Heads lane = load + verify only at serve time (A8):** the serve
  binary carries no fixture bytes and fits nothing; `reflex mint-heads`
  is the OFFLINE mint front (fixture-digest-gated, deterministic) and the
  serve path only loads signed vessels (strict ed25519 + BLAKE3, monotonic
  apply). Trust anchoring: compiled pins first, then the
  `RIIR_REFLEX_HEADS_PUBKEY` operator wildcard; vessels-without-anchor is
  a loud exit-2 config gap, never a per-vessel unknown-key failure.
- **No Python anywhere** (owner directive): no sidecar, no `uv`, no HF
  transformers. The laya lane is the native-Rust port over their
  safetensors, weights runtime-downloaded with BLAKE3-pinned digests —
  never bundled.
- **Follow-up CLOSED 2026-09-22 (`.issues/006` T4):** the shipped
  `RELEASE_FEATURES` is `modelless + laya-riir` — the candle-free release
  cut (v0.2.0) executed under the issue's own owner gate.

## Engine primitive → substrate feature map (measured at birth)

| Engine head | katgpt-core primitive | feature |
|---|---|---|
| Wire contract (request/response, abstention) | `decision_wire` | `decision_wire` |
| Domain routing (argmax over unit centroids) | `variable_rank_domain_expert::pick_domain` | `variable_rank_domain_expert` |
| Confidence calibration (Platt-style refit) | `sigmoid_calibration::SigmoidGateCalibrator` | `sigmoid_calibration` |
| OOD abstain (corpus distance) | `distance_abstain::CorpusDistanceGate` | `distance_abstain` |
| Corpus-is-the-model option scoring | `compression_drafter::Lz4FlexDrafter` | `compression_drafter` |
| (routing-margin head, fixed-A lanes) | `bridge::{ActionBridge, calibrated::CalibratedActionBridge}` | `action_bridge` |

`action_bridge` is forwarded because the calibrated-bridge vocabulary is the
engine's calibration lineage (bench 808); the day-one engine wires the
calibrator on the ANSWER confidence (option spaces are request-time-dynamic,
`A` is const — the fixed-A wrapper fits lanes, not dynamic options).

Deferred arms (named in Plan 603, NOT landed at birth):
`structured_read` (lives in katgpt-forward behind `dllm` + `structured_reads`;
the root-promotion line lands in THAT repo the same commit this engine first
consumes it) and the `rule_embed` vessel (the frozen-artifact pattern from
riir-clippy bench 817/099 — an opt-in arm when the harness needs it).

## Build commands

```bash
cargo clippy --all-targets                          # default (the product)
cargo clippy --all-targets --all-features           # combo lane
cargo clippy --all-targets --no-default-features -- -D warnings   # flag-OFF posture
cargo test                                          # semantics gates
cargo bench --bench decision_set_goat               # G2 latency + G4 alloc (release by construction)
./scripts/ci_feature_guard.sh                       # the whole local gate
```
