# Issue 108 — the LOCALIZE cell (both seats' quotes over the 22 disagreement items)

**Status:** LANDED — the re-judge arc's localize step is DONE (seam
population measured, named; no re-price, per the frozen-read law). Run
2026-11-06 on bmo (both seats loopback, quiet rig): seat A = Bonsai-27B
PQ2_0 over **the prismml fork llama-server** (`--jinja
--no-reasoning-preserve`, the first cell's seat-ops — upstream master
refuses the GGUF outright: `tensor 'output.weight' has invalid ggml type
142` — PQ2_0 is Prism-private, `GGML_TYPE_PQ2_0 = 142` only exists in the
fork) :8083; seat B = Qwen3-8B Q8_0 over upstream master `45b6604`
(`--jinja --reasoning off`) :8082. Instrument:
`ceiling_localize.py` here — replicates the harness's fixture
construction (front 600/600 gold-label match against BOTH probe JSONs
before any judge call) and the exact `judge_messages` prompt bytes;
verifies quotes per `quote_verifies` (whitespace-run collapse,
case-sensitive, context or options). Artifacts: `localize_read.json`
(per-item text + both seats' verdict + quote + verification +
reproduction flag). No pricing artifact is written; both frozen reads
stand untouched.

## Reproduction check (the instrument's own validity)

- Seat B (Qwen3-8B) re-read reproduced its frozen reading **22/22** — the
  fixture reconstruction is provably byte-faithful.
- Seat A (Bonsai PQ2_0) reproduced **19/22**; the three misses
  (sst5:63, sst5:270, sst5:533) returned **empty content at
  finish=length** — the template's reasoning channel fired, starved
  `content` inside max_tokens 256, exactly the first cell's seat-ops
  failure mode. `--no-reasoning-preserve` moves the channel out of
  `content` but does not stop the model from thinking; on ~14% of this
  prompt family the think exceeds 256 tokens and the call dies. **Seat
  finding, named: Bonsai PQ2_0 is an UNSTABLE adjudicator on this prompt
  family** — the 116 unknowns (19.3%) of the first cell were this same
  class, not gold noise.

## The seam population (mechanical fields; judgment classes marked for the rulings pass)

| field | value |
|---|---|
| disagreement directions | A-rejects/B-accepts **15** · A-accepts/B-rejects **7** (p≈0.13 — direction not decisive) |
| gold-grade × direction | very negative 8 (all A-rej) · neutral 5 (all A-rej) · very positive 4 (all A-acc) · positive 5 (A-acc 3: 117/282/424 · A-rej 2: 221/355) |
| quote discipline | B quote-verified 15/22 (all its supported) · A quote-verified 6/22 — and A's quotes are SHORT sentiment phrases ("uniformly excellent", "best", "so good", "good", "intelligent fiction") vs B's near-wholesale sentence copies |

Per-item (both quotes in `localize_read.json`; classes are the
rulings pass's INPUT, not applied rulings):

| case | gold | A (bonsai) | B (qwen3-8b) | localization |
|---|---|---|---|---|
| sst5:5 | very negative | not_supported | supported | grade-threshold (text plainly negative: "never really feel involved") |
| sst5:6 | very positive | supported "one of polanski 's best films" | not_supported | grade-threshold (strong praise, no intensifier) |
| sst5:54 | neutral | not_supported | supported | genuine ambiguity ("just another genre picture" — neutral vs mild-negative) |
| sst5:63 | very negative | **MALFORMED (empty)** | supported | grade-threshold + seat-noise ("confused mediocrity") |
| sst5:117 | positive | supported "intelligent fiction" | not_supported | **B strictness error** (clear praise rejected) |
| sst5:145 | neutral | not_supported | supported | genuine ambiguity (mixed crime-drama sentence) |
| sst5:148 | very positive | supported "uniformly excellent" | not_supported | grade-threshold |
| sst5:221 | positive | not_supported | supported | grade-threshold (abstract positive) |
| sst5:233 | very negative | not_supported | supported | grade-threshold ("any more generic… universal product code") |
| sst5:253 | neutral | not_supported | supported | genuine ambiguity ("desire disrupts stasis") |
| sst5:270 | very positive | **MALFORMED (empty)** | not_supported | seat-noise (A froze; B rejects "wonderful" as very-positive-grade) |
| sst5:282 | positive | supported "best" | not_supported | **B strictness error** ("best american movie…" is praise) |
| sst5:330 | neutral | not_supported | supported | genuine ambiguity ("often moving" vs neutral gold) |
| sst5:355 | positive | not_supported | supported | **B leniency error candidate** ("harsh existence" text, gold positive — wholesale quote, weak support) |
| sst5:388 | very negative | not_supported | supported | grade-threshold ("too silly to be frightening… lazy affability") |
| sst5:424 | positive | supported "good" | not_supported | grade-threshold ("good ol' … stuff" — dismissive tone; genuinely close) |
| sst5:443 | very positive | supported "so good" | not_supported | grade-threshold |
| sst5:471 | very negative | not_supported | supported | grade-threshold ("a movie about goodness is not… a good movie") |
| sst5:478 | very negative | not_supported | supported | grade-threshold ("boring , formulaic" — plainly negative) |
| sst5:533 | very negative | **MALFORMED (empty)** | supported | seat-noise + grade-threshold |
| sst5:537 | neutral | not_supported | supported | **gold-noise candidate** ("was witty" → gold neutral; polarity reads positive) |
| sst5:546 | very negative | not_supported | supported | grade-threshold ("forgot to add any genuine tension") |

## Named classes (the rulings pass's seam registry)

1. **S-GRADE-THRESHOLD (12 items: 5, 63, 148, 221, 233, 388, 424, 443,
   471, 478, 533, 546)** — both seats polarity-correct; they disagree on
   whether the text's intensity reaches the TYPED grade (very-X vs X).
   This is SST-5's published granularity-noise class (human inter-annotator
   κ on fine sentiment grades is ~0.55-0.75). A and B sit on OPPOSITE
   sides of the threshold by gold pole: Bonsai rejects the very-negative
   grade, Qwen3-8B rejects the very-positive grade.
2. **S-AMBIGUOUS-NEUTRAL (4 items: 54, 145, 253, 330)** — neutral golds
   over genuinely mixed/mild text; both readings defensible.
3. **S-B-STRICT (2 items: 117, 282)** — Qwen3-8B rejects clear praise;
   its non-quote refusal path gives no evidence to weigh.
4. **S-B-LENIENT (2 items: 355, 537)** — Qwen3-8B accepts on wholesale
   quotes with weak/mismatched support; 537 doubles as a gold-noise
   candidate ("was witty" golded neutral).
5. **S-SEAT-NOISE-A (3 items: 63, 270, 533)** — Bonsai's reasoning
   channel starved content on re-read; the seat's readings on the
   boundary are per-call unstable at temperature 0.

## What this hands the rulings pass (and what it does NOT do)

- The κ 0.6387 is now LOCALIZED: ~12/22 of the disagreement mass is the
  benchmark's own grade-granularity seam (an SST-5 property, not a seat
  defect), ~4 items are Qwen3-8B errors split both directions, 4 are
  genuinely ambiguous, and 3 expose Bonsai's per-call instability.
- **No re-price**: both frozen cells stand judge-conditioned. The
  contract's re-judge step needs the rulings applied under a contract
  edit (V2) — the same act as any κ-bar decision. Owner-gated by design;
  the evidence is now on the table.
- The next seat generation should log `reasoning_content` + finish-reason
  per call (this instrument's one gap: the empty-content calls' raw field
  carries no reasoning echo). The harness arm's JudgeVerdict could carry
  it additively under a V2 wire note.
