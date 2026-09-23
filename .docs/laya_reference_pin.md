# Laya reference pin — provenance, geometry, and the port contract (Plan 603 T1.4 slice 1)

**Status:** PORT LANDED 2026-09-22 — one atomic commit: `laya` feature + candle-core 0.11
(CPU f32) + ModernBERT forward + decision head + `build_sequence` + tokenizers-crate loading
+ runtime weight download (SHA-256 pins) + the G5 `[[test]]` row. **G5 PASSES per
checkpoint: 88/88 forwards, top-1 agreement 1.0, probability drift ≤ 3.1e-6 vs the 1e-3
gate, act probabilities exact, conf drift ≤ 5.1e-5 — both dev and release profiles.**
The gate still binds every published number. The pin itself landed 2026-09-22 (provenance +
geometry recorded BEFORE any port code — the Research 576 live-fetch caveat discharged).

**Candle lane RETIRED 2026-09-22 (`.issues/006`, owner directive "no candle at all
cost"):** the reference-lane ROLE this doc recorded is history — candle is deleted
from the repo (features, dep, tests). The PINS below are NOT history: weights,
tokenizer BLAKE3s, geometry, and the inference contract are the goldens'
provenance and stay load-bearing — the riir lane's G5 gate replays the same
frozen captures (candle-independent). The tokenizers UNIFICATION note below
keeps its version as a measured fact; the unification RATIONALE (one build
with candle) died with the lane, and the v1 bump attempt + negative is
recorded in `.issues/006` T3.

This is the canonical record for the native-Rust laya lane. The upstream research note is
`katgpt-rs/.research/576_Laya_Open_Jev_Generalist_RLCD_Comparison_Arena.md`; this file
carries the load-bearing facts the port compiles against.

## Source pins

| Artifact | Pin | Verified |
|---|---|---|
| laya code repo | `github.com/NandhaKishorM/laya` @ `573e5b62696ba441230cd6be71d593331b5d23af` (release 0.3.5, 2026-09-21 23:47 +0530) | full clone at `.raw/laya` (gitignored) |
| `laya` checkpoint (English, ModernBERT-large) | full-file SHA-256 `891102d372688fc2a094dac56a384bc537b87c63f21f9f3dac0be2b7cbc8d86c` (842,609,210 B) | HF LFS oid, tree API 2026-09-22 |
| `laya-multilingual` checkpoint (mmBERT-base) | full-file SHA-256 `9d628fd971b700382ac6f65920a86f149777b2e748e0c955fb3b19695aa8f204` (643,835,514 B) | HF LFS oid |
| `laya-typed-decisions` checkpoint (ModernBERT-large) | full-file SHA-256 `4fa56de72383a9d3efa9cfa78955733c81b9fc8067a587ca4beb82c78107a24e` (842,609,220 B) | HF LFS oid |

All three checkpoints ship inside ONE hub repo (`convaiinnovations/laya`, subfolder
packaging — the code's own download path). Small files were downloaded 2026-09-22 and
BLAKE3-pinned (house hash; these are the byte-exact copies the port's golden tests read):

| File | Bytes | BLAKE3 |
|---|---|---|
| english/tokenizer.json | 3,583,228 | `d8d4c8b30443535d6416e4f542d0a77e7f10e2809df063d11f7642a667037030` |
| english/tokenizer_config.json | 308 | `8b275b6f13f464ceb2e6268c566ab3b40c8bfdcd6af7a5a299539194450083f0` |
| english/encoder_config.json | 2,083 | `3b1e7e84b9d90a3835a55ed2a81395698e1d143f3553123c1e498649c7464098` |
| english/rl_agent_config.json | 745 | `dbc03fdb75def9d9ea5218dd6df470d4bb5842584ea372bd9232ded658ed0f8f` |
| multilingual/tokenizer.json | 34,363,188 | `01e0f0d31015fa48f99c5e7aea639fbf136181e632cfbf40156862aced7b20b1` |
| multilingual/tokenizer_config.json | 524 | `8486197d7556f869092d214857a10fc9b75f8108250197ba08e7e9df8dba6637` |
| multilingual/encoder_config.json | 1,938 | `a831925d30809ab5bb2ad5424eff016a422310a8989758c44631748276eee06a` |
| multilingual/rl_agent_config.json | 472 | `dd2da6b7f43afd2babffa290bb1857784e49d825e6f76c5c0ab1d0fbdf441714` |
| typed/tokenizer.json | 3,583,228 | `d8d4c8b3…` — **byte-identical to english** (same HF git oid `2f4d8583`) |
| typed/tokenizer_config.json | 337 | `66b37468dfcfbc4b9022c2026d602bbd3dac39903fccd4361d81caa5e3ab4056` |
| typed/encoder_config.json | 2,084 | `e724310ec3024cb84e2d7ad571f1ff20f2f3d4c1021b18cc43140a51a0a3a02e` |
| typed/rl_agent_config.json | 847 | `4fbe491144bb1b3119a7ad308b6942523e85b9b35d554329b8d95d17e71e51d7` |

Weights are RUNTIME-DOWNLOADED with the SHA-256 pins above as verification targets — never
bundled (the Phase-2 lean-assets rule). The safetensors headers were read from a 2 MB
range fetch per file (safetensors: 8-byte LE header length, then JSON header); the tensor
inventory below comes from those headers, not from a full weight load.

## Encoder geometry (from the configs + safetensors headers)

| | english / typed-decisions | multilingual |
|---|---|---|
| backbone | ModernBERT-large (`ModernBertForMaskedLM`) | mmBERT-base (same architecture family) |
| layers / hidden / heads | 28 / 1024 / 16 | 22 / 768 / 12 |
| intermediate (MLP) | 2624 | 1152 |
| vocab (config) | 50,368 | 256,000 |
| attention | global every 3rd layer (layers 0, 3, 6, …); sliding window 128 otherwise | same pattern |
| RoPE theta | full 160,000 · sliding 10,000 | 160,000 everywhere |
| absolute position embeddings | NONE (the config's `position_embedding_type: absolute` is vestigial — no such tensor exists; RoPE is the only position signal) | `sans_pos` — none, as configured |
| biases | none anywhere in the encoder (`attn/mlp/norm/attention_bias` all false; confirmed: no bias tensors in the header) | none |
| layer-0 norm quirk | layer 0 has `mlp_norm` only — no `attn_norm` (the embeddings norm feeds attention directly); layers ≥ 1 carry `attn_norm` + `mlp_norm` | same |
| encoder tensor names | `encoder.embeddings.tok_embeddings.weight`, `encoder.embeddings.norm.weight`, `encoder.layers.N.attn.Wqkv.weight` (fused 3d×d), `attn.Wo.weight`, `mlp.Wi.weight` (fused 2I×d gate+up), `mlp.Wo.weight`, `attn_norm.weight`, `mlp_norm.weight`, `encoder.final_norm.weight` | same names/shapes at d=768 |
| weights dtype | F16 throughout (the sole exception below) | F16 |

Checkpoint tensor counts: english 206, multilingual 170, typed-decisions 206.

## Decision-head geometry (torch `nn.TransformerEncoderLayer`, norm_first, batch_first)

Present in all three checkpoints at the checkpoint's `d` (1024 / 768):

- `head.layers.{0,1}.self_attn.in_proj_weight` [3d, d] + `in_proj_bias` [3d] — fused QKV,
  torch MHA layout (Q rows 0..d, K rows d..2d, V rows 2d..3d), **with biases** (torch
  default — unlike the encoder).
- `head.layers.{0,1}.self_attn.out_proj.weight` [d, d] + bias.
- `head.layers.{0,1}.norm1/norm2` LayerNorm weight+bias [d].
- `head.layers.{0,1}.linear1` [4d, d] + bias, `linear2` [d, 4d] + bias — **ReLU between,
  NOT gelu**: the reference constructs `nn.TransformerEncoderLayer(d, nhead, 4d, dropout,
  batch_first=True, norm_first=True)` whose `activation` parameter keeps torch's default
  `F.relu`. This document originally said "GELU between" here — wrong; the torch source is
  the oracle. (The scorer's own `nn.GELU()` is genuinely gelu.)
- `type_emb.weight` [3, d] — one row per question type (choice=0, score=1, noul=2).
- `scorer.0` LayerNorm(d) → `scorer.1` Linear(d→d) → GELU → `scorer.3` Linear(d→1).
- `act_head.0` Linear(d+4 → 256) + bias → GELU → `act_head.2` Linear(256 → 2) + bias.
  The +4 features are `[top1, top1 − top2, normalized_entropy, k/255]` (single-option
  questions pad top2 with 0.0, giving top1 − top2 = 1.0).
- `temperature` [3] — **F32 in english and multilingual, F16 in typed-decisions.** The
  port's loader must not assume one dtype for this tensor.

## Tokenizer facts (corrects the plan's guess)

Both tokenizers are **BPE** (HF fast-tokenizer JSON), NOT WordPiece — Plan 603's
"WordPiece-class" note was a guess and is wrong. The Rust `tokenizers` crate loads both
files directly via `Tokenizer::from_file`; no porting needed.

- english: BPE, base vocab 50,280 + 116 added tokens; config ids: bos/cls 50281,
  eos/sep 50282, pad 50283 (`[MASK]` rides the vocab/added set). `model_max_length` 8192.
- multilingual: BPE, vocab 256,000 + 249 added; specials `<pad>`=0, `<eos>`=1 (sep),
  `<bos>`=2 (cls), `<unk>`=3, `<mask>`=4. `model_max_length` 8192.

## The inference contract (from `laya/common.py` + `laya/agent.py` @ the pinned sha)

1. **Sequence** (`build_sequence`): `[CLS] <type> question: <ins> [SEP] [MASK] opt0
   [MASK] opt1 … [SEP] state [SEP]`; options truncated to 48 tokens each; head budget
   `head_max_len` (192 EN / 256 ML+TD) with a ≥16-token option floor and a ≥8-token
   instruction floor; state truncated to the remaining window (right-truncate by
   default); `[MASK]`-literal strings neutralized to spaces before tokenizing; total ≤
   `max_len` (512 EN / 1024 ML+TD). Marker positions = each option's `[MASK]` index.
2. **Encoder forward**: ModernBERT (embeddings + norm; alternating global/sliding-window
   attention with per-layer-type RoPE; fused-Wqkv SDPA attention; no-bias MLP with fused
   gate/up + GELU) → `last_hidden_state`.
3. **Head forward**: `h += type_emb[qtype]` (broadcast over positions); 2 pre-norm
   transformer layers (`x = x + MHA(norm1(x), key_padding_mask)`; `x = x + FF(norm2(x))`);
   gather marker positions; scorer → per-marker logits; masked markers filled −1e4.
4. **Probabilities**: per question, k = marker count;
   `t = temperature_by_options[bucket(qtype, k)]` falling back to `temperature[qtype]`;
   `z = logits[:k] / t`; `p = exp(z − max(z)) / Σ exp(z − max(z))`.
   **The clamp is parity-load-bearing**: `clamp_temperature` confines t to [0.5, 5.0],
   mapping non-finite → 1.0 — and the english checkpoint's `choice:11+` bucket ships
   t = 0.1006, which the reference REWRITES to 0.5 (with a printed warning). The port
   mirrors the clamp exactly; it does NOT "fix" the shipped value.
5. **Confidence**: `1 − H(p)/ln(k)` clipped to [0, 1] (k < 2 → 1.0); answers round to 4
   decimals. `score` answers report `Σ i·p_i`; `noul` reports p(true) and
   max(p_true, p_false) as confidence; `choice` reports argmax + the full probability map.
6. **Act head**: pooled CLS row (position 0) + the 4 features → act logits → softmax →
   `act_probability = p(slot 0)`. (act_costs: `escalate: 0.5`, n_act = 2.)

**Softmax stays.** The house "sigmoid, never softmax" rule governs OUR modelless
primitives; this lane is a parity port of a reference implementation whose math is
softmax — G5 (top-1 ≥ 99.9%, p-drift ≤ 1e-3) demands the faithful form. Do not "fix" it.

## Rendered-criterion byte format (from their test suite — parity-critical)

`render_criterion` / `render_options` have exact-output tests (`tests/test_criteria.py`)
that pin byte formats the port must reproduce:

- Criterion values: str passes through; dict / list / int / bool → **JSON with Python
  separators — `", "` between items and `": "` after keys** (e.g. `{"desc": "phishing"}`,
  `["a", "b"]`, `3`, `false`), **`ensure_ascii=False`** (non-ASCII kept raw, e.g.
  `{"d": "münchen"}`); unserializable values fall back to `str(v)`.
- ⚠ **Rust trap: `serde_json::to_string` emits NO spaces** (`{"desc":"phishing"}`) — the
  port needs a Python-JSON-compatible writer for criterion rendering, or the tokenized
  option text diverges and every downstream probability shifts.
- `choice`: value `None` or `""` → bare key (`tech`); **`0` and `False` are real values**
  (`zero: 0`, `no: false`) — only None/"" mean "no description".
- `noul` defaults when `crit` is None or a side is missing: exactly
  `false: no, the statement does not hold` / `true: yes, the statement holds`.
- `score`: `level {i}: {render}` per index (crit may be a list of mixed types).
- Their round-trip check: the emitted JSON must parse back.
- Single-option questions (k=1): forward must not crash — softmax over one logit is 1.0,
  the padded top2 gives top1 − top2 = 1.0, logits/act stay finite (`tests/test_decision_model.py`,
  the #96 regression).
- Router detection honesty (`tests/test_router.py`): surrounding-language KEYS must not
  drive detection (English keys around Devanagari content stay non-English); undecidable
  Latin languages report `language_undecided` rather than a guess; script profiles are
  per-script ratios (e.g. `{"armenian": 1.0}`); an explicit model choice overrides script
  routing; auto task detection is opt-in.

## What the next slice owes (T1.4 proper)

- `laya` feature + candle dep in this repo; ModernBERT forward (both geometries,
  alternating window attention) + the head; `tokenizers`-crate loading of the pinned JSON.
- The G5 parity `[[test]]` row with `required-features = ["laya"]` in the SAME commit as
  the first port code (the repo's green-zero law).
- The G5 question corpus EXISTS: `tests/fixtures/laya_parity_v1.jsonl`
  (BLAKE3 `f2a00354ef554f36d99509df160aaf8f35e68ac9058631d283d842ea329d9d26`, 36 rows /
  37 questions / 98 forwards; protocol in the sibling README) — the capture step below
  reads exactly this file.
- ~~Reference logits captured at pin time~~ **DONE 2026-09-22** —
  `tests/fixtures/laya_parity_expected_v1.json` (BLAKE3
  `4f4dcc377c78c1010b017d51737ba46d630f3978bb1cceffcba272454b9c6bcb`): 88 reference
  forwards via the PINNED laya code imported from `.raw/laya` (agent.py-verbatim load:
  fp32 weights over F16 storage, `reference_compile=False`, eval, CPU, no autocast).
  Verified in-capture: `choice:11+` resolves to the CLAMPED 0.5 on both English-lane
  checkpoints; `score:2` falls back to `temperature[1]` and the checkpoints genuinely
  differ there (1.2514 english vs 1.0374 typed); truncation hits the exact windows
  (512/1024); `ml-thai-collapse` reproduces the documented collapse (english on Thai:
  conf 0.0002 at ≈0.49/0.51). The capture procedure is a /tmp-only script — the repo
  stays Python-free; the expected file's `_meta` records the environment and load path.
- ~~Weights runtime-download path verifying against the SHA-256 pins above.~~ **DONE** —
  `src/laya/weights.rs`: hub layout resolved (english = repo ROOT; `multilingual/` +
  `typed-decisions/` prefixes; `tokenizer/tokenizer.json`, `encoder/config.json` hub paths
  → flat local names), curl transport, `.part` + rename-after-verify, dual-layout local
  reads (flat cache OR an HF snapshot root via `LAYA_WEIGHTS_DIR`).

## Verdict-review closures (2026-09-22, same day as the port)

- **The Rust script-detector Router is PORTED** (`src/laya/lang.rs` +
  `src/laya/router.rs`): `lang.py`'s script ranges / stopword margins /
  diacritic-rate laws and `router.py`'s decision precedence (explicit
  model > explicit task > detected workflow (opt-in) > explicit lang >
  script/language > default) with the exact reason strings. The reference's
  own `tests/test_router.py` behavioral suite — script detection, the
  #35 Romanian honesty law, the Turkish-"para" no-guess law, state-keys-
  ignored flattening, workflow exact-set signatures, alias normalisation,
  all 17 routing cases — ships as Rust unit tests and passes in full.
  Structural deviation, recorded: the Python `Router`'s LRU checkpoint-
  loading machinery is NOT ported (the harness holds loaded agents; the
  router stays pure decision logic — the plan's named scope).
- **tokenizers UNIFIED to candle-core's own 0.22.2** — the port originally
  pulled 0.23.2 beside candle's 0.22.2 (two copies of a heavy crate); the
  manifest now resolves ONE copy and G5 was RE-RUN on the unified version
  (88/88, drift ≤ 3e-6 — parity holds; the structural checks make the
  version choice empirically safe rather than assumed).
- **`_meta.captured_with` gap noted**: the capture recorded
  torch/transformers but not the PYTHON tokenizers version; with the Rust
  lane now verified against 0.22.2 the open question is retired for G5
  purposes (any future re-capture should record it).

## Measured port traps (each cost a 4-orders-of-magnitude drift hunt)

1. **candle's `Tensor::gelu()` is the TANH approximation.** torch's `nn.functional.gelu`
   (the reference's `hidden_activation: "gelu"` and the scorer) is erf-based. candle ships
   the exact form as `gelu_erf()`. Using the tanh form alone did NOT dominate the drift —
   but it is wrong, and with the window bug fixed it is the difference between ~1e-6 and
   ~1e-3-class per-op error compounding through 28 layers.
2. **The sdpa sliding-window mask radius is `local_attention // 2` (= 64), NOT +1.** The
   attention module computes `self.sliding_window = config.sliding_window + 1` (= 65) for
   flash-attention's inclusive-boundary bookkeeping — but that value rides the FLASH-path
   kwarg only; the SDPA mask the capture ran under is built from the UN-incremented config
   property with the bidirectional overlay `|q - kv| <= window`. Porting the +1 into the
   mask showed up as length-correlated drift (max 2.9e-2 prob drift on the 512-token
   truncation rows, 100% top-1 agreement preserved — the gate's structural checks held
   while the numbers were wrong 4 orders over). With the radius corrected: drift ≤ 3e-6.
3. **The fixture rows carry the SHORT question form** (`t`/`ins`/`crit` — the capture's
   transport shape); the reference API parses `type`/`instructions`/`criteria`. The parity
   test translates — the capture harness's shape is part of the contract the test must
   replay.
