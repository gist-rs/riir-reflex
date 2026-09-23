# Plan 001 — the fitted game head served over HTTP (the arena's modelless lane plays)

**Status:** COMPLETE — 2026-09-23, same day. Serving lane + Metal default + the v0.2.2 cut all landed and deployed; issue 011 carries the two remaining boards' unblock paths.

The Plan 607 follow-up the release session decision-listed: the arena's
modelless board "honestly abstains here and falls back to a random spot"
because the out-of-the-box engine has no game corpus. The corpus-fitted
heads exist in katgpt-rs (Bench 878/881/882, published anchors) — this plan
serves one over HTTP so the out-of-the-box engine PLAYS Tetris at the
published 44/120 class instead of random (10.8%-class).

## Design (decision-recorded)

**Serve the decoded Tetris head; lanes and flappy keep their honest
abstain.** Per-game analysis (the decision that scoped v1):

- **Tetris — SERVE (decode-based).** The spot features are fully
  self-contained: `laya-tetris-v2` decodes one option sentence →
  5 fill ordinals → the decoded feature row (Bench 881's decoded arm,
  the MEASURED-BEST tetris reader: λ=1, in-corpus 44/120, LOO 44/120 vs
  structured 36/120). Coverage is 100% of grammar-valid sentences (the
  site renderers are the grammar renderers, golden-proven) — live boards
  that drifted off the fixture corpus still decode. Zero ambiguity.
- **Lanes — ABSTAIN (cross-lane features).** `lanes_decoded_features`
  columns 6–7 count the OTHER lanes' blocked/free status: one lane
  sentence cannot reproduce the published 84/100 head, and a
  sentence→corpus-row lookup is ambiguous (the same sentence carries
  different neighbor contexts, and the head reads them). Unblock paths
  recorded in issue 011.
- **Flappy — ABSTAIN (render-gated).** The site renders frozen v2
  sentences (position band only) — the v2-decoded arm is measured
  DEGENERATE (Bench 881, 77/100 constant-pick). The v3 render (Bench 882,
  decoded Δ0 = structured 96/100) needs the site's v3 render upgrade —
  a site-side change coordinated with the recorded-demo lane (the demo
  oracle is v2-keyed). Unblock recorded in issue 011.

**Boot-fit from the embedded fixtures** — the fixtures are `include_str!`
(verbatim copies of katgpt-rs `tests/fixtures/`, BLAKE3-pinned) and the
heads are fitted at boot by the published recipe (standardize → λ by
state-level LOO MSE over the pinned grid → final fit). The fit is
deterministic and digest-asserted in tests: no weight artifact can drift,
and the cross-repo contract is the data digest + the agreement anchors,
not a copied weight table.

## Tasks

- [x] T1 — assets + `src/game_heads.rs`: fixture copies + BLAKE3 pins;
  the `laya-tetris-v2` grammar table port (vocabs/template, decode,
  decoded features); fixture parsing; the published fit recipe
  (Standardizer + LOO λ select + final fit over
  `katgpt_core::state_option_scoring::head`); `GameHeads::build()`.
  LANDED `367766c` — the fit reproduces the published anchors
  bit-identically (serving head digest `00aa6221…c6e`).
- [x] T2 — the serve path: the modelless `/decide` branch tries
  `GameHeads` before the cosine engine (question-pinned, grammar-invalid
  → fall through → abstain); wire response = `Outcome::Noul` +
  `probabilities[0]` (clamped p) + routing reason `game-head/tetris`.
  LANDED `367766c`.
- [x] T3 — tests `tests/game_heads_serve.rs`: fixture BLAKE3 pins; corpus
  round-trip (2660/2660 decode → re-render byte-identical); fit
  bit-determinism; the published agreement anchors (44/120 in+LOO at
  λ=1); serve-wire tests (hit p, abstain paths, question mismatch,
  validate_against). LANDED `367766c` — 6/6 green; full local guard
  PASSED (7 layers) with the G5 parity green at BOTH postures.
- [x] T4 — Metal-default for the laya lane: on macOS with
  `laya-riir-metal` compiled, unset `LAYA_DEVICE` → Metal (the measured
  ~2× forward, the arena watchability lever); explicit `cpu` opts out;
  explicit `metal` without the feature stays fail-loud. G5 parity re-run
  GREEN at the new default posture (3 checkpoints, 29 s vs 219 s CPU on
  this box); fresh interleaved row p50: metal 78.1 ms vs cpu 176.8 ms
  (2.26×, loaded box — the recorded quiet-box pair is 79/157).
- [x] T5 — gates: `cargo clippy --all-targets` clean; default +
  `--all-features` test suites green (61 lib + all integration incl. the
  G5 parity at BOTH postures); the G2/G4 engine-core benches untouched
  (the game-heads path is boot+edge, not the engine core).
- [x] T6 — release v0.2.2: darwin artifacts gain `laya-riir-metal`
  (the stamp check is missing-subset — unaffected); licenses regen; leak
  scan PASS ×5; host smoke (healthz / game-head 200 / abstain honest /
  laya ready at device=metal) + 4090 windows smoke (byte-identical head
  answer) PASS; GitHub release live with 6 assets; brew tap resolves
  0.2.2 hash-verified + audit clean; scoop manifest bumped; install.sh /
  install.ps1 handle the v0.2.2 `reflex` rename with a pre-v0.2.2 pin
  fallback (both paths live-verified); dist repo README command copies
  updated.
- [x] T7 — site copy + deploy (b09bffb, version e6ac2ce8): the modelless
  board plays Tetris out of the box (v0.2.2), Metal-default laya note,
  `reflex` launch command, the tetris explain blurb updated; golden
  tests + demo check still PASS; prod curl-verified on all three
  surfaces.
- [x] T8 — docs: AGENTS.md + README + HISTORY rows; this plan.
- [x] T9 — follow-up issue 011 filed (the lanes + flappy unblock paths,
  with the acceptance law: no serving from an unmeasured fit).

## Anchors (pinned in tests)

| thing | value |
|---|---|
| tetris fixture blake3 | `f32c8577bca50726618d2bb4fb27c904148161d650f16a59c01676a97fa540bb` |
| lanes fixture blake3 | `6a6d02af05b529749ddac3bf962344c2e22565b467f28a5eecd37031d0a4f600` |
| tetris decoded arm | λ=1 · in-corpus 44/120 · LOO 44/120 (Bench 881) |
| tetris structured (context) | λ=0.1 · 36/120 · LOO 35/120, head `65409c14…2e66` |
| lanes (context) | λ=0.01 · 84/100 both, decoded ≡ structured (300/300 rows) |
| flappy v3 (follow-up) | decoded Δ0 = structured 96/100, head `c93d36dc…3c5` |

Provenance: katgpt-rs Plan 607 T2/T3 (Bench 878 + 881 + 882), fixtures
`tests/fixtures/*_oracle_laya_en_*.jsonl` (the laya oracle over the T0b
state dumps, english checkpoint, G5-parity lane).
