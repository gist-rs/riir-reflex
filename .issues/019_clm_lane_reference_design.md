# Issue 019 — CLM comparison lane (Contrastive-LM/CLM; Research 001's actionable half)

**Status:** OPEN — reference design distilled 2026-09-24 (`.research/001_CLM_Contrastive_Language_Models.md`); no Rust code landed. The `clm` lane joins the harness as a third comparison lane (modelless · laya · clm) behind an opt-in feature; no default-build changes.

**Source pin:** github.com/Contrastive-LM/CLM @ `cca045ffdb07b3ebcfe6938537cdeac5e14899c9` (Apache-2.0; full-tree read, clone deleted after pin) · HF `Contrastive-LM/CLM-v0.1-8B` (reference head `CLM_v0.1-8B.pt`, 75 MB, Qwen3-8B last-token pooling) · blog contrastive-lm.notion.site (2026-09-23, Kwok/Ré/Mirhoseini et al.).

## Why the lane is cheap and on-charter

1. **The wire is the one we already speak.** CLM serves the TypeSafe `POST /v1/systemone` shape (`schema.py`'s own docstring): `state` + `questions {noul|choice|score}` → distributions + `confidence = top − mean(rest)`. `decision_wire` (katgpt-rs Plan 603 T1.2) maps 1:1; the adapter is a thin HTTP client, no harness change, byte-identical questions by construction.
2. **It is the first open lane claiming to beat the incumbent at the incumbent's game** — Jev-parity zero-shot at up to 9× lower latency (T-Rex in-repo: CLM p50 16.5 ms vs Jev 149.8 ms), and SOTA as a best-of-N verifier (DeepSWE 81.6% / Terminal-Bench 2.1 87.6%) where Jev scores below pass@1. A publishable `/bench` row — and an honest latency-LOSS row is a SUCCESSFUL outcome too: if CLM's new-state p50 lands under laya's Metal numbers on our short-state fixtures, that cell still ships (the arena exists to publish the truth, not the win).
3. **The deployment shape fits the box.** Encoder = vLLM Qwen3-8B pooling server (one-time boot on the 4090, `--gpu-memory-utilization 0.35`); `clm-serve` heads are CPU-scale (20M params, 75 MB artifact, hot-reload). The lane is a *comparison* lane — never a default-on product lane.

## What MUST be copied, not approximated

1. **The prose-rendering law** (`to_text` + `state_text`): objects render as `key: value` fields (never JSON), arrays as `- item` lines, **context first, question last, blank-line separated** — "the layout the heads were trained on". The adapter must reproduce this byte-for-byte or scores drift silently. Options embed their own description (or the key when the description is empty); `noul` candidates default to `"Yes. This is true: {instructions}"` / `"No. This is false: {…}"`.
2. **Temperature semantics:** CLM's `temperature ∈ (0, 100]` DIVIDES the logits before softmax (default 1). Our adapter pins 1 and never reuses our τ vocabulary.
3. **The heads are encoder-bound** ("a head only makes sense with the encoder and pooling it was trained against") — the reference head requires Qwen3-8B last-token pooling. Swapping encoders = retraining (out of scope here; flywheel territory).
4. **Determinism:** fixed head + L2-normalised vectors + fixed temperature → deterministic scores. Repeats must be byte-identical (a lane parity pin in the harness, the laya-lane precedent).

## NOT transferable

- **Softmax-over-cosine** — diverges from the house law (sigmoid, never softmax; decision_wire is sigmoid-then-L1). The lane maps THEIR distribution verbatim; our engines keep ours. Both columns publish.
- **Abstention** — CLM has none (the inherited Jev flaw). Our wire keeps abstain first-class; the lane never abstains and the table says so.
- **Their Jev numbers** — vendor measurements (their endpoint/protocol). We publish OUR cells only; their JSONs (T-Rex) are reference cells, re-run under our protocol before any site number.
- **Python at runtime** — the lane's Rust side is an HTTP client to `clm-serve` + vLLM (their stack, comparison-fidelity posture — the same split as the ANE lane's offline conversion: their Python serves, our Rust measures). No Python in OUR tree.

## Reference implementation path (Rust, opt-in `clm-lane` feature)

- Feature `clm-lane` (opt-in; implies nothing else; the harness lane register grows one row; `required-features` law per T1.1e).
- `src/lanes/clm.rs`: a `ClmLane` adapter — build the request (prose rendering law above), POST to `CLM_SERVE_URL` (default `http://127.0.0.1:8700/v1/systemone`), map answers into `decision_wire`'s `DecisionResponse` shape (distribution + confidence columns; NO abstain mapping), record latency + `usage.input_tokens`.
- Harness: the 9 dataset suites + the 6 decision-point families grow a `clm` column (loud SKIPPED without `clm-lane`, the `harness_cache_reuse` precedent); CI-regenerated tables only.
- Parity/determinism pin: same request → byte-identical response (repeated N times, head mtime unchanged).
- `/bench` tables: publish accuracy + ECE-protocol metrics + latency columns per lane; the `clm-raw` ablation row optional (their own zero-head control — isolates the heads' contribution in OUR tables).
- VectorArena pattern (B+ distillable) is deliberately NOT in this issue's scope — it touches the serve edge + game heads and deserves its own issue if adopted (the pattern record lives in Research 001 §3.2).

## Tasks

- [ ] Gate 0: owner gate — is a `clm` lane wanted, and at what scope (lane-only vs lane + `clm-raw` ablation row)? Want-it is INFERRED here from the arena's standing purpose (every open Jev-lineage model is a lane candidate) + the 2026-09-24 distill directive; ratify or re-scope before T1 (the Issue-017 gate precedent — its gate 1 ran before any code).
- [ ] T1 Adapter skeleton behind `clm-lane` + the prose-rendering law, byte-pinned against their `to_text`/`state_text` outputs on fixture states (golden from the pinned sha; re-clone at the pin when writing them).
- [ ] T2 Harness lane row: the 9 suites + 6 families grow the `clm` column (loud SKIPPED without the feature); determinism pin (N-repeat byte-identity).
- [ ] T3 4090 bench posture: vLLM Qwen3-8B pooling + `clm-serve` boot script (`scripts/`), serialized posture, box state recorded per the standing rule; first same-box cells: modelless vs laya vs clm (vs Jev BYO-key where the visitor lane exists).
- [ ] T4 T-Rex re-run under OUR protocol (their harness is Apache-2.0; their JSONs are reference only) — optional row, gated on T3.
- [ ] T5 `/bench` publish + README row (lane posture: comparison lane, Apache-2.0 attribution, not affiliated; CLM named to compare).
- [ ] T6 HISTORY + AGENTS rows; boundary check (no new deps beyond std HTTP in the default build; `clm-lane` adds none).

## Fusion tail (novelty TBD — from Research 001 §6)

CLM's cached-action-embeddings × our game heads × the corpus flywheel: a trained-head lane whose ACTION side is BLAKE3-pinned per fixed option set (the game-head digest law) and whose STATE side is the only live forward — "System One at 20 Hz" with a published recipe (head-only InfoNCE, ~1 h on this box's 4090 at 60M pairs; N\* ∝ D^1.02 sizes the head from the corpus budget). That is the flywheel-scale follow-up (riir-clippy Issue 125's reopen path), NOT this issue.
