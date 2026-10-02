# Research 005 — Cloudflare Clef decision models + the Jev Decision Index

**Status:** RECORD — distilled 2026-10-02 (owner ask). The comparison-lane + JDI-protocol plan is
filed: `.plans/011_clef_lane_jdi_protocol.md`. The hosted-Clef serving credentials (Cloudflare
account ID + Workers AI API token) are owner-gated; nothing in this note requires them to be true.

## TL;DR

Cloudflare open-sourced two **decision models** — `Clef` (frozen Qwen3.8-27B backbone + routing
head + rank-256 LoRA) and `Clef-flash` (Qwen3.5-9B) — Apache-2.0 on HF, served on Workers AI,
**Jev-API compatible per the vendor** (`state` + `questions{noul,choice,score,criteria}` — the
shape of our `decision_wire`; UNVERIFIED by us until the Phase-A wire fixture lands: the Workers
AI REST route wraps replies in a `{result, success, errors}` envelope and may not return
per-option probabilities). They benchmark on the **Jev Decision Index (JDI)**, the community leaderboard at
`multimodalart/jev-decision-index`, where **three of our existing comparison lanes already carry
board rows** (laya, GLiNER2.5-Decide, CLM). Clef itself is too new for the current board snapshot
(HF created 2026-09-30; `data/index.json` generated 2026-09-28) — the blog's table is Cloudflare's
own run. Action: (1) add a `--clef` comparison lane to the harness exactly like `--agentjev`
(same wire, different endpoint); (2) add a JDI-protocol metrics adapter (macro-F1 + chance-corrected
skill + coverage disclosure) so Rethink's cells become comparable to the board where protocols
overlap; (3) the headline comparison cell is **typed_decisions** — the blog's "Security incidents"
workflow is the same shape our typed_decisions suite models (severity levels × action keys), where
Rethink's typed head reads **0.7550** (NLEH v2, record-only, riir-instinct Bench 040/041) against
the laya-typed floor 0.7445, and AgentJev read 0.7715 gold-label (det ✗ — its disclosed bf16 HTTP
wobble).

## The sources (pinned)

| Source | Pin | License | Read |
|---|---|---|---|
| Cloudflare blog "Introducing Clef" | `blog.cloudflare.com/clef-decision-models/`, fetched 2026-10-02 | prose | full |
| `Cloudflare/clef` (HF) | created 2026-09-30T21:15:35Z, base `Qwen/Qwen3.8-27B`, `custom-code`, 393★ | Apache-2.0 | metadata only (weights not pulled) |
| `Cloudflare/clef-flash` (HF) | created 2026-09-30T21:15:36Z, base `Qwen/Qwen3.5-9B`, `custom-code` | Apache-2.0 | metadata only |
| JDI space README | `multimodalart/jev-decision-index`, fetched 2026-10-02 | — | full |
| JDI leaderboard data | `data/index.json`, `generated_utc: 2026-09-28T00:39:36+00:00`, suite corpus `sha256 b2b56d6f…d5` (their own pin), edition `release-v2.1` / "Decision Index 0.2.1", hardware "1 × RTX PRO 6000", Jev `jev-1.13.0` | — | full JSON |

Community quant ecosystem landed within ~24h of the weights: GGUF (bartowski, prithivMLmods,
abenzerps incl. imatrix), MLX 4/8-bit (mlx-community, TrevorJS), **EXL3 4-bit (ramgpt)**, NVFP4/FP8
compressed-tensors for vLLM (simonlehmann, prithivMLmods), ONNX (ollaya-dev). Note for the
local-serving question (plan §D): the GGUF conversions carry the **backbone only** — Clef's scoring
head ships as transformers `custom-code`, so a llama.cpp GGUF cannot reproduce typed outputs;
local serving = vLLM/transformers with their code, not llama.cpp.

## What Clef is (architecture, as published)

- Frozen Qwen3.8-27B (Clef) / Qwen3.5-9B (Clef-flash); **prefill-only** pass; schema choices scored
  **in parallel** from internal backbone representations — non-autoregressive decision step, no
  intermediate text.
- Two-stage attention routing: per-choice evidence extraction → cross-field + back-to-payload
  attention → schema-bound scoring, with a **lexical prior** preserving semantic intent across options.
- Post-training: routing head + rank-256 LoRA jointly; **label-smoothed CE** + **Brier loss** for
  probability calibration; synthetic permutation datasets (field order, prompts, schema shapes);
  **RLCD** (RL for Calibrated Decisions) secondary objective — partial credit for adjacent ordinal
  choices, record-level precision reward, reference penalty against distribution shift.
- Wire: identical to Jev — `state`, `questions` keyed by id with `type: noul|choice|score`,
  `criteria` as list (score levels) or map (choice options). Vision encoder + 64k context on Clef.

## The JDI protocol — what makes a number comparable

From `data/index.json` (edition 0.2.1):

- **38 index benchmarks** in 5 areas: knowledge(10) · language(10) · retrieval(6) · tools(5) ·
  arts(7). Area weights: arts fixed 10%, the rest √-proportional (knowledge 25.85%, language
  25.85%, retrieval 20.02%, tools 18.28%). Gold ★ benchmarks weigh 1.2 in-area.
- Per benchmark: chance-corrected skill `(score − chance)/(1 − chance)` clipped [0,1]; headline
  `balanced_skill = 100·Σ_c 0.2·category_skill`; raw accuracy kept on model pages.
- **Coverage-adjusted: an unanswered/unsupported request counts as wrong** (no silent replacement;
  `answered/unsupported/errors` disclosed per row).
- **Contamination is enforced**: rows an entrant trained on count wrong (measured examples on the
  board: `lev` lost 22 BANKING77 rows, `Eikos-27B` had its ENTIRE FinEntity benchmark zeroed, the
  pngwn scorer lost 9,542 MMLU-Pro rows).
- Per-entrant **calibration block**: acc/conf/ECE/Brier + reliability bins + per-area ECE — the
  same idea as our G1 calibration gate, published as first-class board data.
- **Hosted-API rows are accepted** (Jev's own row is "network round-trip from our lab"; its latency
  block explicitly says not comparable to on-card figures) — so a Workers-AI-hosted Clef lane is
  protocol-clean, with the serving posture disclosed.

## Clef vs the board vs our lanes (crosswalk)

Blog's own table (their run, not the board): BANKING77 macro-F1 **Clef 94.20 / Clef-flash 90.93 /
Jev 79.74 / Kev-9B 84.83 / laya 14.29**; latency medians **Clef 209.3 / Clef-flash 38.8 / Jev
524.1 / laya 5.8 ms**; Typesafe eval "Security incidents" **Clef 62.9 / Jev 61.7**.

Board rows our lanes already hold (0.2.1, their frozen corpus, their hardware):

| Entrant | balanced_raw | balanced_skill | median ms | note |
|---|---|---|---|---|
| Jev (jev-1.13.0, hosted) | 68.09 | 57.91 | 524.1 | ECE 0.074 |
| pplx-decider-v1-27b (Qwen3.8-27B full FT) | 66.89 | 56.40 | 101.4 | strongest open entrant |
| Winnow-12B (Q8 GGUF/llama.cpp) | 61.91 | 50.02 | 72.5 | |
| Rune 26B-A4B v3 | 67.30 | 57.44 | 120.5 | |
| **GLiNER2.5-Decide** (our GLiNER lane) | 32.35 | 11.21 | 23.3 | |
| **CLM-v0.1-8B** (our CLM lane) | 27.94 | 7.40 | 46.8 | |
| **laya** (our laya lane's source) | 27.53 | 6.04 | **5.8** | fastest on board |
| Clef / Clef-flash | — | — | — | **not yet on the board** (too new) |

Our own cells on our own splits (NOT board-comparable, different corpus/protocol): Rethink typed
head (NLEH v2, record-only `serve: ✗`) **0.7550** (riir-instinct Bench 040/041); the laya-typed
floor **0.7445** (the published laya-typed read — reference, not a Rethink cell); AgentJev typed
gold-label **0.7715** (det ✗); banking77 H2 0.8540; laya base 0.528 typed / 0.498 banking77
(reflex bench 038).

The headline comparison the owner asked for — **Clef vs Rethink** — lands on two axes:
1. **typed_decisions** (≈ the "Security incidents" workflow both vendors measured): Clef hosted vs
   Rethink typed head **0.7550** (record-only) vs AgentJev **0.7715** (det ✗) vs the laya-typed
   floor **0.7445**, same cases, our Rust measuring — with
   a case-identity pin (same case-ID set / split hash + same trim cap) asserted before the cell
   is published.
2. **banking77 under macro-F1** (the JDI's retrieval-area metric): needs the macro-F1 adapter (plan
   §B) — our harness currently reads accuracy.

## Distill verdict (per the `distill` skill shape)

- **Verdict line: NO (riir-refine corpus) / LEAGUE-INTEL HIGH ×3 / no product-coverage axis.**
  Post-verdict transport finding (verdict round 1): the harness lanes are **std-only plaintext
  HTTP/1.1** (`TcpStream`; no TLS crate in `Cargo.toml` — the deliberate zero-new-deps lane law),
  so a Workers-AI HTTPS endpoint is unreachable from the agentjev lane shape. The default lane
  posture is a loopback TLS-terminating forwarder (operator-run, outside the repo; extra hop
  disclosed); direct HTTPS via a feature-gated `ureq` dep is the owner-gated boundary-change
  alternative.
- **Corpus axis (riir-refine rule corpora):** the sources are a Python/transformers serving stack
  and an eval suite — no Rust rule material for kernel_opt/rust_perf/clippy_lints. Not a mining
  candidate. (The `.raw` clone rule was not needed: no corpus claim is being made.)
- **Lane-intel axis (three mappings, each actionable):**
  1. **riir-reflex decision-serving lane** — a new `--clef` comparison lane (mirror of `--agentjev`;
     same Jev wire, different endpoint + auth) + the JDI metrics adapter. This is the primary
     deliverable; plan §A/§B.
  2. **riir-infer prefill league** — Clef is a **prefill-only 27B workload in OUR LEAGUE ARCH
     (Qwen3.8-27B)** with parallel scoring reads: exactly the shape "beat llama.cpp/vLLM for
     prefill" cares about. EXL3/GGUF quants exist for weight-format work. Not started; plan §D
     (deferred).
  3. **riir-train training recipe** — Brier-loss calibration term + label-smoothed CE + RLCD's
     partial-credit/reference-penalty shapes are directly applicable to our per-option typed head
     (t608 NLEH v2) which is trained by riir-train. Not started; plan §D notes it.
- **Product-coverage axis:** n/a (no famous-lib Rust family in play).

## Closest cousins (and why they don't cover this)

- `src/lanes/agentjev.rs` — the wire template (their `jev_service`, malevrigns/agent-jev). Clef is
  the SAME wire with a different endpoint + bearer auth + `clef`|`clef-flash` model selector; the
  lane is a sibling file, not a config flag, because the auth header + model field + hosted-URL
  default differ.
- OpenThai lane (Plan 003) — the hosted-teacher measurement law (their stack serves, our Rust
  measures; latency = round-trip; loud refusal without creds) is the posture this lane adopts.
- The G1 calibration gate — JDI publishes ECE/Brier per entrant; our G1 already computes ECE, so
  the calibration comparison needs only a per-lane ECE readout, not new machinery.
- Issue 059 / Plan 009 quarantine law — the harness families are "not comparable to the Jev
  Decision Index" by standing owner decision. The JDI adapter applies to **dataset suites only**;
  the families section stays quarantined. This plan does not touch that boundary.

## Risks / honest caveats

1. **JDI-comparability is protocol-thin by default.** Our harness reads stratified capped suites
   with accuracy; JDI reads their full frozen corpus with macro-F1 + coverage adjustment on their
   hardware. Any "JDI-comparable" column we publish is a *crosswalk*, never a board claim — the
   board requires all 38 index benchmarks (0.2.1 panel) on their corpus, on one card, with their
   contamination audit. Chance-corrected cells on OUR suites must use chance levels computed from
   the label distribution actually evaluated (a capped suite that drops labels has a different
   chance level than the board's full-label-set figure) — board chance values stay reference
   columns. The plan carries an explicit honesty law for this.
2. **Transport.** The repo's lanes are deliberately std-only plaintext HTTP (zero new deps — the
   clm-lane law). Reaching the hosted Workers AI endpoint needs either a loopback TLS-terminating
   forwarder (default posture; the extra hop is part of the disclosed serving latency, exactly
   like the JDI's own "network round-trip" rows) or an owner-gated `ureq` dep (a BOUNDARY.md
   change).
3. **Hosted latency is network-bound.** Clef's 209.3/38.8 ms medians are Workers-AI-edge figures;
   our cells measured from the 4090 box will carry different network posture. Quote box state +
   serving posture (bench-preflight law) beside every number; never compare to on-card lanes.
4. **Cost + creds are owner-gated.** Workers AI pricing for `@cf/cloudflare/clef` is not in the
   blog; a full-suite run spends real tokens. The lane REFUSES loud without `CLEF_SERVE_URL` +
   `CLEF_API_TOKEN` (the agentjev law), never half-runs, and the plan prices a smoke-first cadence.
5. **Determinism unproven.** Hosted decision APIs have wobbled before (AgentJev's disclosed bf16
   HTTP wobble). The lane ships the determinism rerun probe from day one; det ✗ is a row, not a
   blocker, and gets disclosed in the table.
6. **Calibration is Clef's headline claim and our G1 is the honest test of it** — if the hosted
   outputs are over-confident on our splits, the blog's Brier-loss story gets a third-party read.

## Next

`.plans/011_clef_lane_jdi_protocol.md` — Phase A: the `--clef` lane. Phase B: the JDI metrics
adapter (macro-F1, chance-corrected skill, coverage disclosure, pinned index.json snapshot).
Phase C: Rethink-on-JDI cells + the reflex-site comparison publication. Phase D (deferred):
local Clef serving in riir-infer (prefill-league workload) + the Brier/RLCD recipe note to
riir-train.
