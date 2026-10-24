# Bench 161 — riir-train 635 T4 router fold r2: the two Bench-160 instrument gaps are CLOSED — the router consumes `classes_spelling` (names inverted, gold/pick/q permuted into the sorted-corpus space), the emotion/sst5 t20k recapture joined, and the emotion lift is CONFIRMED: routed 0.7712 vs pplx 0.6301 (+14.1 pp) / d3 0.6246 (+14.7 pp)

**Status:** RECORD — instrument + recapture + folds executed 2026-10-25 (4090 idle-lane session, zed-4090-idle-1025). The Bench-160 unblock recipe ran in full: (a) the `classes_spelling` consumer landed in the teacher router (reflex `22fb1c2`), (b) emotion + sst5 recaptured over `.raw/datasets_t20k` (~55 + ~30 min GPU, `.raw/startlux_teacher_t20k/`). Both folds join with ZERO skips. **Verdict: the emotion column is the real lift the 160 record predicted — gate 1.0000, routed train acc 0.7712 (CV 0.7712), +14.1 pp over the best prior column (pplx 0.6301); sst5 routed 0.6018 (CV 0.6015, gate 0.8654) — a +0.3 pp nudge over startlux alone. The routed artifacts are the T5 distillation feed for emotion (owner-gated window); the (a)-only rows (xnli, injections) show NO lift — startlux loses there too, as 160 predicted.** NC-lineage binds every routed/passthrough artifact (see §6).

## 1. The instrument (Bench 160 finding (a) — reflex `22fb1c2`)

`run_teacher_router` now canonicalizes BOTH columns into the CANONICAL sorted-corpus index space before any join check:

- **Names:** a post-623 bin carries `classes_spelling: {law, map: corpus→student}` with student-spelled `classes`; the map is INVERTED (student→corpus) and every class name resolved. A pre-623 bin (field absent) or an identity bin (the `"corpus (identity)"` string) passes through as-is. A malformed map — non-injective, or missing a class — refuses the suite loud, never guesses.
- **Order:** the canonical space is the sorted corpus spelling — the distill corpus law (`classes.sort()`), which is exactly the native space of every pre-623 capture (d3/pplx/clef stay bit-identical). A source carrying the student ORDER (the xnli/emotion StudentUniverse permutations) gets gold/pick/q PERMUTED into the space; identity-order sources are untouched.
- **Emission:** the routed artifact's `classes` are the canonical names; the `router.classes_spelling_sources` block records both sides' raw spelling fields (null for pre-623). `RouterSource` stats are permutation-invariant (accuracy, mean max-prob, mean margin) and are computed pre-permutation, unchanged.
- **Cross-check that the permutation is right:** the startlux xnli column reads train acc **0.9073 AFTER the permutation** — byte-for-byte the number its own `distill.json` recorded at capture time (the 10-22 writer computed it in student space; the router recomputes it in canonical space from the permuted pick/gold). The gold-pin join (20000/20000) then verifies row alignment in the shared space.
- Tests: `classes_spelling_remap_joins_and_emits_canonical` (relabel join + canonical emission + provenance), `classes_spelling_malformed_map_refuses_loud` (missing map entry → suite skip, never a pass), `classes_spelling_student_order_is_permuted_into_canonical_space` (the xnli shape: data moves, routed q asserted in canonical space). 468/468 lib green; clippy `--all-targets -D warnings` clean both default and `--no-default-features`.

What the map inversion does NOT do: it never rewrites the data of a canonical-order source, never repairs a broken capture, and never spans a row shift (the positional gold-pin check below still refuses any misaligned posture — the (b) fix is the recapture, not a tolerance).

## 2. The recapture (Bench 160 finding (b))

`.raw/startlux_teacher_t20k/` — the startlux teacher re-read over the SAME sorted train pages the d3/pplx/clef captures used (`.raw/datasets_t20k`):

| suite | rows | classes | teacher train acc | p50 ms | p99 ms | wall | blake3 (first 16) |
|---|---:|---:|---:|---:|---:|---:|---|
| emotion | 16000 | 6 | **0.7712** | 201.0 | 582.0 | 3447 s | `65fb48871670f0a8` |
| sst5 | 8544 | 5 | **0.6000** | 196.0 | 421.0 | 1827 s | `a839c5605f3985e9` |

- The emotion acc **replicates the 10-22 datasets-posture capture exactly (0.7712)** — deterministic Instant bidirectional read-outs; the +1 duplicate row and the +11 sst5 survivors do not move the teacher's read.
- Posture: the 635 T2 line — Q4_K_M GGUF, llama.cpp **b11549** (`aa94f20`, MSVC) `-ngl 99 -c 8192 --parallel 1 -b 8192 -ub 8192 --threads 12` :8081, engine `STARTLUX_LLAMACPP_BIDIR=server` :8090 health-gated (`kind instant`, `prompt_attention bidirectional`, `bidir_mode server`). TRAIN rows only (the 629-T1 law); zero test rows.
- Box state (preflight PASSED): `PROVENANCE: power=AC scheme=high load=4.56 swap=7181MB gpu=4%, 18937 MiB` — the 18.9 GB is the loaded GGUF; a sibling cargo lane shared the box (load disclosed; accuracy unaffected — every read is a deterministic forward; the latency columns carry the shared-box caveat).
- No backend death this window (~86 min continuous, well inside the one-death-per-~217k-read-outs envelope); both suites sealed atomically with blake3 sidecars; stack TORN DOWN at close (ports freed, VRAM 382 MiB baseline).

## 3. The folds (r2)

**startlux_t20k ⊕ d3** (`.raw/router_startlux_t20k_d3/`):

| suite | sources (train acc) | gate g | routed acc | cv (loo) | verdict |
|---|---|---:|---:|---:|---|
| **emotion** | startlux 0.7712 ⊕ d3 0.6246 | 1.0000 | **0.7712** | 0.7712 | **THE lift: +14.7 pp over d3** |
| sst5 | startlux 0.6000 ⊕ d3 0.5813 | 0.8654 | **0.6018** | 0.6015 | small blend gain (+0.3 pp over startlux alone) |
| banking77 | d3 0.9090 only | — | — | — | passthrough (startlux t20k dir has no banking77) |
| massive_intent_en | d3 0.9375 only | — | — | — | passthrough |
| prompt_injections | d3 0.9890 only | — | — | — | passthrough |
| xnli_en | d3 0.9166 only | — | — | — | passthrough |

**startlux_t20k ⊕ pplx_v2** (`.raw/router_startlux_t20k_pplx/`) — the best-prior-column comparator:

| suite | sources (train acc) | gate g | routed acc | cv (loo) | verdict |
|---|---|---:|---:|---:|---|
| **emotion** | startlux 0.7712 ⊕ pplx 0.6301 | 1.0000 | **0.7712** | 0.7712 | **+14.1 pp over pplx — the 160-predicted cell, measured** |
| sst5 / xnli / injections | single-side | — | — | — | passthrough |

Zero refusals in both folds — the two 160 instrument gaps are gone. The gate at 1.0000 says the routed emotion artifact is startlux-dominant in practice (sigmoid saturation at T=0.01 for a 14 pp gap); the router's contribution is the CERTIFIED JOIN (spelling + row alignment + gold pins verified) and the drop-in artifact, not a blend.

## 4. The (a)-only rows — no lift, honestly (`.raw/router_startlux_d3_r2/`)

Re-running the 160 pairing (startlux datasets-posture ⊕ d3) with ONLY the spelling consumer (before the recapture) joined the two row-aligned suites:

| suite | sources | gate | routed | verdict |
|---|---|---:|---:|---|
| xnli_en | startlux 0.9073 ⊕ d3 0.9166 | 0.2819 | 0.9171 | no lift (stays under pplx 0.9207 / laya 0.9654) |
| prompt_injections | startlux 0.9835 ⊕ d3 0.9890 | 0.3660 | 0.9890 | no lift (d3 already holds it) |

Exactly the 160 prediction. emotion/sst5 still refused on row counts at that point — closed by §2.

## 5. What this buys (and what it does not)

- **Buys:** the teacher router now folds ANY cross-era column pair (post-623 StudentUniverse bins × pre-623 corpus-spelled bins) without manual remaps; the emotion routed column (0.7712 train) exists as a sealed RIDT artifact for **T5's owner-gated distillation window** — the student for emotion now has a teacher column +14 pp above the prior best; sst5 gets a certified +0.3 pp blend. The instrument upgrade is permanent (no per-fold scripting).
- **Does not buy:** anything serving-side (the serving stack is untouched — this is the offline T2.5 instrument); any promotion (the modelless-first gate fired in the opposite direction this time: the lift EXISTS, but T5 remains the owner gate); the 067 routed-oracle re-read (still blocked on the absent JDI capture dirs, bench 159/160 caveat stands).

## 6. NC-lineage (the 635 law, unchanged)

Every routed/passthrough bin in `.raw/router_startlux_t20k_d3/`, `.raw/router_startlux_t20k_pplx/`, and `.raw/router_startlux_d3_r2/` blends or copies NC-derived columns (the taint composes per the 635 T0 adjudication): **NC-lineage: startlux-decision-v1.1 (CC BY-NC 4.0) — measurement/internal use only, adoption fail-closed.** Nothing here feeds a serving path; T5 consumption is the owner-gated window.

## Provenance

- Instrument: reflex `22fb1c2` (the router spelling consumer; 468 lib tests + clippy 2 postures green). Binary built from `22fb1c2`; a sibling landed `cb221da` (the opendecider lane) mid-window — it touches neither `router.rs` nor the distill writer (diff verified), so the runs' code paths are identical across the two shas; the runtime banners print `cb221da` (worktree HEAD at run time).
- Folds: harness release binary, `--teacher-router` one-liners per §3/§4 (output dirs carry `router.json` + `ROUTER.md` + `*_teacher.bin(.blake3)` — untracked, regenerable).
- Capture: `.raw/startlux_teacher_t20k/{emotion,sst5}_teacher.bin` + sidecars + `distill.json`/`DISTILL.md` + binding `MANIFEST` posture in §2; host shikuwa (4090, Windows); window 2026-10-25 ~05:00Z→06:30Z; session zed-4090-idle-1025 (claim on riir-train 635, epoch 1792877646).
- Permutation cross-check: startlux xnli post-permutation train acc 0.9073 == its own capture-time `distill.json` number (independent-space recomputation agreeing).
