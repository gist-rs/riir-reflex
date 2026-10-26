# HISTORY.md — riir-reflex

Durable records for closed issues (noise-reduction rule: a resolved issue file
is removed from `.issues/`; its record lands here, hash-pinned). A removed
file's full life: `git log --follow -- .issues/<file>`. Open work lives in
`.issues/` and `.plans/`, never here.

## 2026-10-26 — Issue 097 CLOSED: the `--all-features` suite failures — density's inline GMM state was a stack bomb; the slice-leak oracle carried three drifted text-rule copies

**Status: Resolved. Landing: reflex `d696538` (claim `764964d`). All failures reproduced at `--features density_gate` ALONE — a 16-feature bisect (15 controls green); full `cargo test --all-features` now exits 0 (23 binaries, lib 551 passed where the harness previously aborted mid-run); clippy `-D warnings` clean at default / all / none.**

Found by the `062ec1e` engine-split attribution control (predates it). Three verdicts:

1. **The two stack overflows (lib `b0_judge`, seat gates) — `density_gate`'s inline GMM state.** `DomainExpert.density: Option<SupportGate<64, 16>>` = ~16.7 KiB of inline `[[f32; E]; K]` arrays per expert (2 × `DiagGmm`); `DecisionEngine<842, 256>` measured 14.85 MiB vs ~1.2 MiB pre-066. The dispatch lanes hold engines BY VALUE and debug builds do not elide moves: `build_b0_judge`'s 12-arm const-generic table materialized Σ-arms × (engine + move closure) ≈ 64+ MiB in ONE frame — past its own 64 MiB probe thread (lldb fp-walk: 12 frames, guard fault 64 MiB below SP, crash at the `build_b0_judge` prologue; `RUST_MIN_STACK` is a no-op against an explicit `stack_size`); the seat test's nested posture lanes overflowed the 2 MiB libtest thread. Fix: `Option<Box<SupportGate<..>>>` — the one field that grew is heap-boxed, every site returns to the pre-066 engine size, the serving knob stays false, read paths pay one deref. Substrate-side `DiagGmm` change rejected (layout/alloc-discipline blast radius for the same effect).
2. **`laya_ane_parity` g5 — PASSES on the M3** (234.9 s cold / 14.6 s warm): the amo failure is box/environment-dependent (ANE availability or a timeout on the 60 s+ run), not code drift; adjudicating it fully needs the amo box.
3. **`slice_leak_oracle` g1 + both t3 — genuine three-copy wiring drift, FIXED.** (a) The oracle's `build_suite` match + the shared `eval_case_text` had no Plan-010 s1mb arms — the probe auto-discovers those dataset dirs, so t3's `no builder` panic masked every alphabetically-later suite. (b) The pair-join drifted: probe + oracle `row_text` joined xnli/wanli pairs with `" || "`, the engine corpus rule joins with `"\n"` — post-normalization `" || "` leaves a DOUBLE space, flipping boundary-adjacent Jaccard (wanli rows 364/1674: probe j 0.696/0.771 vs wiring j 0.850/0.804). Probe + oracle copies now join with `"\n"` (the oracle measures the texts the engine sees); the engine side is UNTOUCHED (a `train_docs` change would byte-drift every xnli/wanli decision — serving-posture act). Wanli probe near reads 3 (was 1) from now on; Bench 130's quoted `near 1/1,999` is a dated historical record, no gate pins near counts.

## 2026-10-25 — Bench 161: the 635 T4 router-fold unblock — `classes_spelling` consumer + the t20k emotion/sst5 recapture — the emotion lift CONFIRMED (routed 0.7712, +14.1 pp over pplx)

**Status: Record.** Instrument reflex `22fb1c2` (the router canonicalizes both columns into the sorted-corpus space: names inverted through the in-bin `classes_spelling` maps, gold/pick/q permuted when the source carried the student order; malformed maps refuse loud; 3 new tests, 468 green), recapture `.raw/startlux_teacher_t20k/` (emotion 16000 @ 0.7712 / sst5 8544 @ 0.6000, accs replicating the 10-22 column, blake3-sealed), folds ZERO refusals: **emotion routed 0.7712 (gate 1.0000, CV 0.7712) = +14.1 pp over pplx 0.6301 / +14.7 over d3 — the 160-predicted cell measured; sst5 routed 0.6018 (CV 0.6015); the (a)-only rows (xnli 0.9171, injections 0.9890) show no lift as predicted.** NC-lineage binds every routed artifact (measurement-only, adoption fail-closed); T5 consumption stays owner-gated. Record: [Bench 161](.benchmarks/161_startlux_router_fold_r2.md); riir-train 635 T4 updated same day.

## 2026-10-24 — Issue 095 CLOSED: the StartLux-Decision v1.1 comparison lane — T1–T4 all landed; 4/8 incumbent suites beaten; the NC-lineage teacher corpus is sealed

**Status: Resolved. Landing: reflex `3a9e8f5` (T1 lane module + T2 harness wiring — `src/lanes/startlux.rs`, `--startlux`, `--distill-teacher startlux`, 5 stub-listener tests, clippy 3 postures + lib 443 green); T3 board pass = [Bench 155](.benchmarks/155_startlux_board.md); T4 capture sealed in `.raw/startlux_teacher/MANIFEST.md` (six windows 2026-10-21T23:43Z → 2026-10-22, NC-lineage license token verified in every bin header).**

The lane answered riir-train 635 T3's B5 law: measure before any teacher-capture spend. StartLux-Decision v1.1 27B Instant (public_skill 72.88 selection-on-public) over THEIR Apache-2.0 `startlux-decide` engine on the pplx/d3 `/v1/systemone` wire (one wire, many oracles), booted on the 4090 (llama-server b11549 `-ngl 99` Q4_K_M :8081 + the bidir wrapper :8090) — the construction gate checks the POSTURE, not liveness (`kind == "instant"`, `prompt_attention == "bidirectional"`, `engine.bidir_mode == "server"`; the causal/off-bidir misread is the −26 pt class).

Bench 155, one frozen read, 9/9 suites, det ✓ ×9: **4/8 incumbent suites BEATEN** (typed +2.25 / sst5 +1.17 / injections +6.90 / xnli +3.67; loses: massive −2.0 / ag_news −3.25 / emotion −8.75 / banking77 −4.0) → **T5 earned on the four**. Latency not quotable (sibling build load 13.68 at run start — box-state law; accuracy unaffected). typed_decisions ECE 0.1581 = the T2b overconfidence shape, smaller at serve posture.

T4 sealed the teacher corpus for the four winning families' complements: ag_news 20000 rows (acc 0.9061) / emotion 15999 (0.7712) / sst5 8533 (0.5955) / injections 546 (0.9835) / xnli 20000 (0.9073) / massive 11514 (0.9510) / banking77 9993 (0.9207, p50 986 ms — the knockout cost). **NC-lineage law binding every derived artifact:** `NC-lineage: startlux-decision-v1.1 (CC BY-NC 4.0)` — measurement posture, adoption fail-closed (635 T0). Incident disclosed: the backend died silently at ~4h11m/~217k read-outs (no trace; one reboot per multi-hour window is the disclosed envelope); emotion re-sealed in window 2; zero corruption. Stack torn down at close (VRAM 331 MiB baseline).

The remaining `- [ ]` in the issue was a scope-law negation (perm/null probes + JDI adapter + hosted posture are NOT in this lane) — no work was open at close; the claim (zed-635-startlux-t3) expired 2026-10-24T00:00Z with all four tasks landed. Lease discharged, file removed under the noise-reduction rule.

## 2026-10-23 — Issue 096 CLOSED: fetch_datasets resume truncation at a deduped page — the predicate consults the page's own num_rows_total

**Status: Resolved. Landing: reflex `9f686a8` (fix + two-sided self-test +
guard layer 8 + the manifest pin-state note).**

The defect: `scripts/fetch_datasets.sh`'s paging stop condition is "a page
returns fewer than 100 rows", but the end-of-run Issue-058 dedupe rewrites
mid-sequence train pages IN PLACE — a 100-row page becomes a 99-row FILE.
On any later resume the skip path read that as the terminal page and
stopped CLEANLY (`failed_requests=0`, no FAIL line, a summary that looked
like success): the rethink Bench-071 provisioning hit it twice on this box
(sst5 t20k truncated at 1199/8544-style pulls; the harness's
slice-integrity floor caught the shrunken pool at seat time, but a suite
with no floor reader would have truncated silently).

The fix (issue T1): the skip path consults the skipped page's OWN
`num_rows_total` — the /rows API stamps it on EVERY page and the dedupe's
`.rows |=` rewrite preserves every top-level field — so `offset+n < total`
means the dataset holds more rows and the walk CONTINUES (`resume` log
line); a short page with no readable total takes the legacy stop, LOUDLY
(`note` line). Shape (a)+(c) of the issue's candidates, fused — and it
sidesteps size.json's lossy config/split keying for the as_split-renamed
suites (anli/mmlupro store a differently-named split as `train`). `BASE`
became an env override (the test seam).

The proof (T2): `scripts/fetch_datasets.sh --self-test` — a hermetic
loopback python3 fixture server (zero hub traffic; the hub licence probe
pre-planted cached), three scenarios: (1) the defect shape — resume over a
deduped 99-row page-000 completes the pull (349 rows / 4 files, pages
002+003 fetched); (2) idempotence + the true terminal — the re-run fetches
NOTHING, fires the resume line only for the still-short page-000, and the
genuinely terminal 50-row page-003 stops without one (no page-004);
(3) degraded metadata — a short page with no `num_rows_total` takes the
loud legacy stop. TWO-SIDED: the new code PASSES; a copy with
`rows_total_of` stubbed to empty (the exact legacy predicate) FAILS with 6
findings naming the truncation. Wired as `ci_feature_guard.sh` layer 8
(guard 9 → 10 layers; the two doc layers renumbered 9/10 — the [n/10]
numbering is the quick-mode partial signal).

The manifest note (T3): the aggregate-digest table's rows were derived
2026-09-26, BEFORE the Issue-058 dedupe (2026-10-01) — each row pins the
tree AS RECORDED, not "any tree with these page counts". Measured both
directions on this box's post-dedupe t20k: sst5 re-derives `c2bf5abc…` at
86 pages / 8533 rows (11 exact dupes dropped from the raw 8544; ≠ the
recorded `ebfb0317…`/8544), while xnli_en re-derives byte-identical to its
recorded row (the dedupe dropped 0 rows there) — the two-sided proof that
the dedupe, not drift, moves the figure. The fix itself changes NO
artifact bytes (the resume predicate only decides whether to keep
walking).

REAL-TREE validation (same day, the artifact the defect actually bit):
re-running `SUITES=sst5 OUT=.raw/datasets_t20k TRAIN_CAP=20000` over this
box's repaired t20k walks past EVERY deduped mid-sequence page — train-011
and train-013 (the issue's own reproduction), plus 032/047/051/057/059… —
and stops only at the true terminal page-085 (44 rows, offset 8500+44 =
8544 = total): `8533 rows in 86 file(s)`, exit 0, zero network (every page
skips). The legacy predicate would have stopped at train-011 declaring a
clean 1199-row pull — the exact defect shape, now impossible.

## 2026-10-18 — Issue 093 CLOSED: CorpusDistanceGate capacity-law drift MEASURED (Bench 154) — drift confirmed, no desert, response owner-gated

**Status: Resolved (recorded negative — T2's negative-close condition FAILED
by its own law, so the record carries instead of a close; file closed at
owner direction). Landing: Bench 154 `6a9376e` (`--gate-drift-probe`,
report-only lane;
[.benchmarks/154_gate_drift_capacity.md](.benchmarks/154_gate_drift_capacity.md)).**

The one clean capacity-law consumer, measured both populations vs corpus
growth (1×/2×/4× suite concatenation; probes = banking77+massive test rows,
held out from EVERY concatenated suite; every quantile row carries rank +
support): off-corpus whole-corpus p50 0.2673 → 0.3489 → 0.3693 across the
fixed mid 0.35 — drift ~6–10× the isotropic null, which is RULED OUT as the
level's explanation (as pre-registered); in-corpus routed p05
0.1913/0.2221/0.2134 sits BELOW mid at EVERY size — the birth-frame desert
never existed on real-text corpora. The deployed ρ=30 refit holds in-corpus
calibration while its off-corpus FALSE-ACCEPT roughly doubles across the
growth (48.7% → 76.0% → 77.0%) — the standing open design question. T3's
`mid(M)` lever premise FAILS (the boundary sits inside BOTH populations at
every size — no midpoint schedule restores a separation the fixed frame
never had; no lever built); T4 n/a; T5: the Issue-013 corpus caps are
accuracy/latency knobs and were never holding a gate desert open. Any
RESPONSE (a gate redesign) is an OWNER-GATED design act: file a NEW issue;
until then the gate constants stay load-bearing (shift the threshold behind
a default-OFF lever, never edit the default; sigmoid, never softmax).

## 2026-10-18 — Issue 085 CLOSED: the Drex v1.5 revisit — T0 resolved + T1 published; T2/T3 owner-gated (re-open = explicit written permission only)

**Status: Resolved (agent-executable work done; remainder owner-gated). Landings:
T1 reflex-site `5fc1649` (full-surface drex lane registration + the DLM cells
live: 4090 BF16/CUDA bench-126 timing-quotable 106/167 ms det ✓ 10/10; M3
Q8_0/Metal bench-125 acc-only — typed 0.5855 / ag_news 0.895, preflight
REFUSED so ms never quoted); T0 owner-delegated verdict 2026-10-11 —
permission-first, GGUF deleted (sha256 `7ff32856…4ed5` recorded before
deletion, byte-verifiable re-download).**

`nace-ai/drex-v1.5` (9B `qwen35`+pointer, RAIL-M modified v1.0) is a NEW lane
row per the scope law — never a continuation of the CC BY-NC Drex-DLM rows
(bench 073 typed 0.5865 / sst5 0.5900; bench 125 the independent M3 repro;
bench 126 the det-✓ re-read). RAIL-M §2(c) (competition clause; §6 Use
includes "running … evaluating" and "distilling"; NO personal/research
exception) reaches a decision-model serving business → measurement AND
publication are unlicensed until Nace grants in writing. T2/T3 (the 4090
Q8_0 measurement + the v1.5 site row) stay blocked on the OWNER's permission
email to nischay@nace.ai — 14 days' silence or a denial = option (c)
skip-per-license, NO re-download; persistence after a NO goes to a lawyer's
§2(c) opinion first. RE-OPEN CONDITION: an explicit written grant (asked
separately for run+evaluate AND public publication) → file a NEW issue,
re-download `nace-ai/drex-v1.5-Q8_0` + re-verify the recorded sha256, then
the staged posture from the removed file (fork @ `e8f79610…` llama-server,
the `-b/-ub` flags load-bearing per bench 125). The published CC BY-NC DLM
row (T1) is unchanged by the ruling.

## 2026-10-18 — Issue 090 CLOSED: the jeva lane + the null/K-stability probe family — all legs landed and measured

**Status: Resolved (lane landed + board measured; C2 closed NEGATIVE). Landings:
B1+B2 `83031d2`, Bench-148 cells `a3cd05c`, C1 probes `a829918`+`85d0232`, C2
posture `63ccf78`+`8546a68`+`9494d83`; the A1–A3/B2/C1 probe families ride
`--null-probe`/`--perm-probe`/`--rubric-probe` (Benches 147/148/149/153).**

The first decision-native PAPER model lane (Jeva-4B, arXiv:2610.18773) on our
split — full 9-suite board (typed 0.7680, massive 0.9200, banking77 0.9160,
xnli 0.8800; emotion 0.5225 the show-loss vs our A0), perm-probe 0 flips /
255 slots, null-probe A2 0 flips + A3 bit-identical noul + A1 their-gate drift
+2.6..+12.5 pt at the `JEVA_ALLOW_NULL=1` posture [Bench
148](.benchmarks/148_jeva_measured_cells.md). License law: weights UNDECLARED
→ measurement + teacher-capture posture, adoption fail-closed (riir-train 632
T0). The `--distill-teacher jeva` capture seam rides the same server (B2
rider). The downstream RunOptions break class got its structural repair:
`RunOptions::lanes_off(dir, suites)` (`a7ba797`) — a field add now reds THIS
repo's build inside the adding commit, never a downstream consumer's
(instinct 023 closed on it). C2 (the modelless `set_refine` operator,
katgpt-rs Plan 627 substrate composition) measured NEGATIVE at the GOAT — G1
FAIL −12.3..−14.0 pt, monotone in γ (unit-centroid keys regress the state
toward the corpus mean); lane stays opt-in with the record; re-open = a
different KEY family, never a γ/τ retune [Bench 150](.benchmarks/150_set_refine_ab_goat).
Board read + posture disclosure: reflex AGENTS.md §Build Commands (`--jeva`,
`--null-probe`, `--set-refine-ab`) — the serve side documents the same lanes.

## 2026-10-16 — Issue 092 CLOSED: the 2-feature readout-gate candidate MEASURED NEGATIVE (Bench 151); the 047 acceptance stands

**Status: Resolved (negative; landing `fbdbf10`, issue filed+claimed `f767c64`).**

Bench 147's 10× addendum left the deployed confidence readout RED vs the
conformal-naive floor on the AUROC axis (ag_news −0.026, emotion −0.046 — xnli's
red is tie-zone Δ 0.0024). Issue 092 priced the one mechanism family neither
adjudicated lane had measured: a 2-feature logistic gate
`σ(a·maxp + b·(1−H/lnK) + c)` over the two SHIPPED readout functionals,
fit on the same cal slice (the `label_heads` sequential-SGD law at D=2,
harness-local, deterministic), judged by the floor's own weak-dominance rule.

T1 landed as REPORT-ONLY instrumentation (`FloorObs`, `ArmReadout.inv_ent`,
`TwoFeatureGate`, `CandidateCell`, the factored `dominates`; 5 unit tests,
clippy `-D` clean, lib 432/0) — the floor cells reproduced the 147 addendum
BIT-IDENTICALLY on all six suites, proving the refactor preserved the
instrument. T2 (Bench 151, the exact 147 10× posture): **NEGATIVE — 1/6 pass
(xnli, tie-zone noise), 1/6 within 1e-4 (ag_news 0.5845 vs 0.5846 with a huge
ECE win), 4/6 clear fails including REGRESSIONS on all three green suites.**

The mechanism is the Bench-052 lesson one level up, now measured on the FIT
itself: the cal slice is not a ranking-training signal. typed's fit went
NEGATIVE on maxp (−2.013 — cal says lower-maxp→more-correct) and regressed test
AUROC to 0.4296 while buying ECE 0.0066; emotion's fit found ≈0 weights exactly
where the ranking gap is largest; where the fit DID recover maxp's direction
(ag_news +3.826) it only tied the floor. T3 (the engine lane) never fired —
the 047 acceptance, the 052 demotion, and the Bench-817 dispatch law all
STAND. The candidate leg stays in the `--null-probe` instrument (report-only);
the record prices the binding constraint for any future re-open: a mechanism
whose CAL-SLICE FIT TRANSFERS TO TEST RANKING — not ECE, not the functional
family. Full table + provenance: [Bench 151](.benchmarks/151_readout_floor_candidate.md).

## 2026-10-16 — Issue 091 CLOSED: jdi_tranche1_fetched_count_floors now skips LOUD on a present-but-PARTIAL fetch

**Status: Resolved (fix `c48779f`, canary-proven both arms).** The count
floor's skip logic covered only the ABSENT-dir case; a dir present but short
(measured posture: hellaswag_jdi 6400/8000 after a flaked datasets-server
fetch — later completed to 8000 on disk, verified by direct row count before
the fix landed) flowed into the count assert and read as a code failure. The
fix splits the directions: a SHORT envelope (`test_rows.rows.len() < pinned`)
skips loud with `fetched/pinned` disclosed and `JDI_REQUIRE_DATA=1` still
turns it into a failure; a count OVER the pin falls through to the assert
(pin drift = a re-pin decision, never a silent skip); a FULL envelope whose
builder output lands short stays a hard red (the builder-regression
direction this floor exists to catch). ⛔ The first version of the fix read
`env["test_rows"]` instead of `env["test_rows"]["rows"]` — the datasets-server
ENVELOPE shape — so every suite read 0 rows and the test passed by skipping
EVERYTHING (the green-everything trap, caught by the all-suities-skip output
before commit); the corrected count reads the same source the builders
iterate. Canaries: pin 8001 → `SKIP — 8000/8001` + test ok; same pin +
`JDI_REQUIRE_DATA=1` → panic, FAILED. Both proved by execution.

## 2026-10-15 — Issue 089 CLOSED: the margin-null T1 — perm-disagreement carries NO signal on the modelless lane (0/345 untied slots); the residual laya signal closes it as a comparison note

**Status: Resolved (Bench 145; the issue's pre-registered scope statement
executed as written).** T1 ran the Issue-077 perm probe extended additively
(`CaseRow`/`CaseSpread` gained `gold`, `margin_top2`, `tie` — the one-artifact
join) over three records on the M3 (`145_issue089_margin_null/`: modelless
6 suites; laya/english 5 suites @ 20; laya/typed @ 30; G5 GREEN pre-run,
2/2 checkpoints 28.1 s). Pre-registered floors judged at the pinned point
bars: **modelless 0 untied disagreements in 345 slots** — the
K-orderings-disagree ⇒ top-2-tied identity is now a measurement, not the
Bench-128 control's implication; primary MISS, the deterministic-engine
delta is gone. **laya/english ΔAUROC +0.063 (point bar ≥ 0.02 PASS; 95%
bootstrap CI [−0.073, +0.196] disclosed straddling zero) with partial r
+0.470 against the 0.05 floor** — signal only on a trained/order-sensitive
lane, concentrated on massive_intent_en (Δ +0.094, dis rate 0.30 — Bench
128's own tail-fragile cell). Scope statement: closes as a comparison note
inside the published PriDe/self-consistency class; no trained-lane serving
issue filed (thin support, not the product lane). drex/agentjev/d1 unmeasured
(4090-hosted servers, box unreachable 2026-10-15) — named absent postures;
the closure class rests on the modelless null alone. Disclosures: the T1
error convention counts abstentions as errors (rank-based reads unaffected);
typed_decisions modelless no longer reproduces the Bench-128 k==N red
(0.00 pt median, tie-class flips — issue 079's binding), the defect-lane
exclusion kept regardless. Removed file: `.issues/089_perm_consistency_serving_mode.md`;
full life via `git log --follow`.

## 2026-10-14 — Issue 088 CLOSED: the mmlupro_jdi "unreplicable sample law" was a TRANSCRIBED PRIME — the law runs as documented, under a house fold that is not fnv1a64

**Status: Resolved (docs + pins; the prime itself stays, owner-gated).**
The 4090 lane (Bench 143/144, riir-train 629 T2) observed the mmlupro_jdi
sample stable across population changes yet matching NO replicable ordering
law, and filed `.issues/088`. Adjudicated on the M3 the same day: the ROOT
CAUSE is a digit transposition in the fold's multiplier —
`0x0010_0000_01b3` (2³⁶+435) where the FNV-1a 64 prime is
`0x0100_0000_01b3` (2⁴⁰+435) — offset standard, so the harness digest
family (`cases_digest`, `corpus_digest`, `slice_guard` fold,
`jdi_row_identity_hash`) is deterministic but NOT canonical fnv1a64 despite
the `fnv1a64-` wire prefix; `permutation.rs` alone carries the true prime.
Verified end-to-end on the canonical pool: load_rows → stratified_split →
build → cases_digest reproduces the recorded digest
`6a2d24628ce7b916` AND the served gold sequence [7,9,5,6,2,2,1,7,0,0,0,6]
byte-identically under the house fold (Python replication with the true
multiplier matches the served order exactly; the 4090 author's canonical-fnv
replication therefore could never match — their methodology was sound). The
test-113 insensitivity is ALSO reproducible under the house fold: the
structured multiplier keeps all 100 of that page's qids outside the top-2000
(the uniform-hash expectation does not apply). Landed: docs corrected at
every law site (suites.rs `jdi_row_identity_hash` + the builder note,
runner.rs `cases_digest`/`corpus_digest`, slice_guard fold, dataset-manifest
§15 both JDI rows); regression vectors
`house_fold_known_vectors_not_canonical_fnv1a64` (house "9400" =
`0x10b13a74d09bdec0` vs canonical `0x15109e34d09bdec0`) + the end-to-end
`mmlupro_runtime_chain_serves_the_recorded_sample` (dataset-gated, skip
loud). **The multiplier itself is NOT fixed**: doing so re-orders every JDI
sample (different questions served) and invalidates every recorded JDI
number, digest pin (manifest §15, bench records, cross-repo seat records
via `Seat::cases_digest`), i.e. a measured-wire break — an owner-gated call,
deliberately not taken; the vectors above make any future fix LOUD.
Clippy `-D warnings` clean; 384 lib tests + all suites green. Removed file:
`.issues/088_mmlupro_sample_law.md`; full life via `git log --follow`.

## 2026-10-14 — Issue 087 CLOSED: the JDI panel instrument complete end-to-end (Plan 012 A+B+C+the B2 clef fold) — the floor read exists as rethink Bench 067's three-lane record

**Status: Resolved.** The harness gained the JDI panel's four choice-shaped
public benchmarks as suites (`winogrande_jdi` / `hellaswag_jdi` / `anli_jdi`
/ `mmlupro_jdi` — the official splits ARE the board's row counts,
1267/10042/3200/12032), the ladder ran on them, and the record lives in
riir-rethink (`.benchmarks/067_jdi_public_skill_read/`, three lanes: **A0 4.40
/ ENC 12.99 / clef 36.71 zeros-included public_skill** vs the 58.0 M1 bar;
clef subset 71.00). Landing commits: reflex `8d1a9f7` (suites + instrument,
gates green 499/0/1) · rethink `aba7389` + the B2 fold `fcbf34d` (bench 067)
· reflex B2 close `c8e7152` (Plan 012 COMPLETE). The B2 clef column:
detached rig run 2026-10-13 (det ✓ ×4, zero case absences), folded 2026-10-14
with four disclosures in the record (the wino/anli digest delta = the
pre-commit WIP build vs the committed `8d1a9f7` instrument — same population,
witnessed by the B2 modelless lane reproducing the tranche A0 rows
bit-identically; latency not quotable under sibling load; the runner
DONE-marker false negative; banking77's clef cell carried from bench 113).
The aggregate's merge is per-lane latest-wins since the fold (whole-set
replacement would clobber the tranche ENC rows). Laws carried: B5 crosswalk
caveat (never a board row), edition law (0.3.2 + generated_utc + snapshot
sha256 on every quoted number), grammar-not-answers (frozen test reads, never
corpus fill). Tranche-2 + composition reads stay QUEUED in Plan 012 Phases
D/E (`- [-]`, each its own session slice); the rethink router (032 T3.5) got
its clef input from the fold, the pplx capture remains its gap. Removed file:
`.issues/087_jdi_panel_read.md`; full life via `git log --follow`.

## 2026-10-11 — code_fixtures RETIRED from the harness (owner call; board 9 → 8)

**Status: Resolved.** The last self-authored synthetic suite — harvested from
this repo's own fn spans, n=32, BLAKE3-frozen fixture
(`code_fixtures_frozen.json`), seat-marked `synthetic` — is REMOVED. It
survived the 10-02 families retirement only because the engine read
off-chance on it, and the gap row it carried (−12.5 vs pplx local) was the
noisiest cell on the public board (ECE 0.186 at that population). Removed
with it: the frozen population (`src/harness/code_frozen.rs` + the fixture),
the builders + SUITES row + the in-process seat/prep arms (`runner.rs`), the
e0/distill skip arms, the fixture-fleet history pin, the PAW per-shape specs
(`scripts/paw_specs/code_fixtures.{module,is_pub}.txt`) + their lane/gate
arms, and the `gen_code_frozen` generator example. The measured history stays
in `.benchmarks/` — 001 (timing addenda), 044 (the frozen-population arc,
incl. the commit-relative-harvest lesson), 087 (the first per-shape PAW
cell), 116 — as records, never a re-open.

## 2026-10-10 — Issue 084 CLOSED: corpus dot-name test red on every Windows box — the trailing-dot fixture is unrepresentable on NTFS (visible-skip + interior-dot companion)

**Status: Resolved** (reflex `bca951c`). `corpus::tests::directory_names_load_verbatim_including_dots`
(Issue 081 T2c) failed on every Windows box: Win32 path normalization strips trailing dots (and spaces)
from the LAST path component at dir creation, so `create_dir_all("…helps.")` silently created `…helps`, the
loader read the stripped name, and the byte-intact assertion failed
(`loaded.domains.contains(&"No further round helps.")` — reproduced 4090-windows 2026-10-10). The loader is
innocent: a trailing-dot domain dir cannot exist via any normal Windows API, so the production surface
(by-name option routing `==`) is unthreatened there — the property matters on the macOS/Linux deployment
platforms where such dirs DO exist. Fix, both halves in one commit: `#[cfg_attr(windows, ignore = …)]` on
the original test (a VISIBLE skip whose reason prints in test output — never a silent `#[cfg]`
compile-to-nothing, per the green-zero rule) + `directory_names_load_verbatim_interior_dots` (no cfg, runs
everywhere: `state.depth.0`, `v1.2 release notes` — dots in non-terminal positions, still catching a loader
that trims at the first dot or normalizes names). Verified on Windows: 376 lib tests pass + 1
ignored-with-reason; clippy `--lib --tests` clean. M3/Linux verification of the (no-op there) cfg_attr rides
the next macOS touch.

## 2026-10-10 — Issue 083 CLOSED: the site modelless timing graduates to quotable (bench 137's quiet-box re-read) + the vacuous-carry publisher fix

**Status: Resolved.** The home chart's "timing failed the loaded-box check — re-run pending" presence row
(`Reflex · modelless`) retired. The debt: every m3 run since the 2026-09-24 primary (`8028a10`, load 6.31>6)
started/ended under sibling load, so each posture refresh (`f068ae6` 10-08, `0413dcd` s1mb) published acc-only
(the `d3aeeae` load-wall convention; Issue 021's measured 12×-swing wall held every time).

The re-read (bench 137, reflex `197e855`): `bench_preflight.sh` PASSED (load 2.96, AC settled, High Power),
~11.5 min, 13/13 suites, **start load 4.0 / end load 2.5 — latency QUOTABLE both ends**; accuracies AND corpus
digests digit-match every published cell (a pure timing attach; binary rebuilt at `197e855` after the sibling's
`src/harness/metrics.rs` commit correctly tripped the armed watcher's src-stamp guard). The armed watcher
(`scripts/quiet_box_timing_rerun.sh`, bench 137) proved all three behaviors live across the wait: two fired
attempts ended unquotable when siblings returned mid-run (start 5.76 → end 11.48), then GAVE UP on a sibling's
transient `src/` dirt — the wall and the guard both held. Published via the full `republish_bench.sh` pass
(self-test 93/93 · chart smoke · pairing 18 · mirror parity · chromium bench-page smoke), deployed (reflex-site
`1b78072`, wrangler `6577167f`); live: typed 0.762 ms · banking77 0.357 · sst5 0.090 · semantic_defects 0.024,
areas timing `p50_geomean 0.1572 ms, n_used 9, n_unquotable 0`.

The publish surfaced a **publisher defect, fixed same day** (reflex-site `1b78072`): an acc-only-stripped
incumbent owns NO timing cells, so LANE-CARRY had nothing to donate — but `apply_lane_carry` fired anyway,
popping the update's own `latency_quotable: true` and stamping a false `latency_provenance` note. The
Issue-003 T2 suppression cannot see this shape (it checks the incumbent's verdict `is False`; a stripped
incumbent's verdict is ABSENT). Fix: the vacuous-carry guard (`not any(k in src_lane for k in
LANE_LATENCY_FIELDS)` → skip) + self-test `case_stripped_incumbent_cannot_vacuously_carry` (93/93 green,
prove-fires with the guard removed). Remaining acc-only timing debt on the board: the pplx lane (the
bench-131 law — its own quiet-box re-read).

## 2026-10-09 — Issue 081 CLOSED: the Jev-Mem System-One backend lane — zero-fork T1 adapter live, T2 parity-plus at −21% work, the stopping lever measured CLOSED on this substrate (T2b/T2c); T4 declined by the owner

**Status: Resolved.** The lane's full arc (Benches 133/134/135/136, reflex `a53ce4b`+`a3ba564`): T1 landed the native
`POST /v1/systemone` TypeSafe route (zero-fork, 14 wire gates, honest abstain mapping, `model:"reflex-modelless"`);
T2 measured the full LoCoMo-10 matrix — **0.7921 vs mock 0.7847 pooled evidence_recall at −20.8% edges examined**
(paired 61W-41L-1438T, p=0.0297) with the write-path flat-reject posture beating mock's cosine links; T3 measured the
real stopping surface as a BINARY expand-once gate (depth ≤ 1 everywhere; the per-round thresholds are not the lever);
T2b's corpus lever dominated the mock-stop-1 floor on both axes but missed the recall hold (0.96pt) — four mechanism
laws recorded (routing-is-the-classifier, option-delta surface, pin-steal, traversal self-symmetry); T2c built the
`nb_pair_scale` pairwise-count-table head + serve knobs and REFUTED the depth-1 stop as a mock-arm artifact (the gold
lives in routing-bounded beam waves 2-3 — a PERFECT depth-1 stop still costs −15.6pt s5 recall; four more laws:
averaging/collision/burial/depth-limit; the v5 label closed content-inseparable on a second estimator). T5 published the
memory-control pack structure/mapping (`.docs/02_protocols/memory_control_pack.md`). **T4 (LoCoMo answer-model
landscape cell) DECLINED by the owner 2026-10-09** — after the T2c refutation the cell buys landscape color, not a
decision. The lane's durable outputs: the `/v1/systemone` surface (the funnel artifact for Jev-Mem-class consumers),
the head machinery for any future O(1)-token policy surface, and the rig under `.raw/locomo/` (corpora v5-v11,
captures s0-9). The pplx latency quiet-box re-read (082 leftover) stays box-gated; the 7331 serve stays DOWN
(restart line in the Bench-136 record).

## 2026-10-09 — Issue 082 CLOSED: the pplx-decider v1.1 lane measured end-to-end (bench 132, MLX-4bit posture on M3) — board rank does NOT transfer to our suites at amplitude in BOTH directions (banking77 −15pt vs clef-flash; prompt_injections +34pt, xnli +14pt); site edition 2026-10-9 deployed; the 623 per-suite-teacher decision recorded; the standing JDI sha-watch armed


**Status: Resolved.** The JDI board's new #1 (Perplexity Decider v1.1 27B, Apache-2.0, published
2026-10-05) measured on our 9 frozen suites via the pplx lane (`src/lanes/pplx.rs`, the
clef/agentjev sibling shape, 11 wire tests, reflex `741bda3`). Posture: the 4090 bf16-offload
plan died on a real hole (32 GB RAM box cannot hold 52 GB resident; disk-spill ≈ 15-20 h AND
blocks sibling training) — pivoted to `mlx-community/pplx-decider-v1.1-27b-4bit` on Metal
(the clef-lane posture class; their measured fidelity 33/33 argmax, mean KL 0.0014) through a
byte-deterministic reference-wire adapter (`.raw/pplx_srv/pplx_mlx_adapter.py` — the port's
own server would false-fail the det probe with its uuid `id`; det held 10/10 byte-identical
on every suite). **The read (bench `132_pplx_lane/mlx4bit`)**: pplx LOSES banking77 to
clef-flash 0.8040 vs 0.9540 (−15pt) but WINS prompt_injections 0.9224 vs 0.5862 (+34pt),
xnli 0.9567 vs 0.8133 (+14pt), typed 0.7350 vs 0.6955 (+4pt); ECE 0.033-0.080 on 8/9 —
well-calibrated out of the box. acc-only publish (preflight REFUSED under sibling load — the
bench-131 law; latency stays acc-only until a quiet-box re-read, the bench-118 precedent).
Site: the full bekko/clef footprint extended to pplx (reflex-site `69143ce`+`4f77c79`, edition
2026-10-9, deployed + live-verified 9/9 cells + 9 crosswalk rows; `PUBLISH_BENCH_LANES=
pplx:acc-only` also protected the published modelless cells from the doc's by-product
default-posture control — the .issues/033 law). Crosswalk `board_reference` re-pinned to the
JDI v3 edition (11 rows, every number verified against the live fetch; corrected the filing
session's off-by-one ranks; clef is public-#3 at 61.71 — pplx's #1 is carried by the 80%
private weight; reflex `0413dcd`, `.research/005` Update note) — and that commit also PAID the
blocking bench-131 debt (two stale chart_render_smoke pins: the `[p50-family]` presence-row
set now DERIVED from the render's own last-stats hook; `[ladder]`'s synthetic fixture now
STATE-PROOF owning all three rungs' timing, both directions asserted). Standing watch (T5):
`scripts/watch_jdi.py` + pin `scripts/jdi_board.sha` (exit 0/1/2; `--update` the only writer;
both verdict arms proven live) — the next board move files itself. Teacher candidacy: dead
heat on the language-class mean (0.7801 vs 0.7793) with opposite strengths → **per-suite
teachers** recorded in riir-train 623 (`9f041a28`: clef-flash → banking77/sst5/massive; pplx →
xnli (0.9567 clears the 0.9000 class bar where openthai 0.8600 did not) + emotion); capture
still gated on 623 T1 + the Research-464 §3 owner gate. The distillation record: `.research/010`.
The 4090's verified 52 GB bf16 snapshot stays as the local reference for any future
exact-posture run on bigger iron; its dead serve-deps install is irrelevant.

## 2026-10-09 — Issue 080 CLOSED: the wanli_en ESC re-source lane complete end-to-end (fetch → ledger → wiring → Bench 130 → the last leftover fixed); the t3 slice_leak wiring gate un-panicked for the Thai suites

**Status: Resolved.** The reflex-side half of the ESC re-source (rethink Issue 024 owns
the re-fit) landed in one lane 2026-10-08; the leftover PRE-EXISTING failure it reported
(the opt-in `t3_runner_wiring_reproduces_the_probe_over_built_cases` gate panicking
`thai_sib200: no builder — the wiring cannot scan it`, verified at clean HEAD `59b02c8`)
is fixed at `6002a4c`: the test-local `build_suite` dispatch gained the
`thai_sib200`/`thai_wisesight` arms — the builders always existed in `suites.rs` (the
runner dispatches to them); only the gate's own match was never extended when the
Plan 003 T3.2 suites landed 2026-10-02, and the opt-in feature meant nothing ran it.
Counts reproduce the live probe exactly (thai_sib200 exact=143 near=1/61;
thai_wisesight exact=280 near=2/120 — G1's numbers, now also asserted through the
built-case path). Full local gate green: slice_leak oracle 6/6, default `cargo test`
353 lib + all integration targets, clippy `-D warnings` at 3 postures. Standing note:
the wanli_en arm predates this fix and validates wherever its data is fetched (this
box does not carry the suite); the MNLI seed-influence owner call rides the ledger row
(`.docs/02_protocols/dataset_manifest.md` §Licences + §11) — recorded, not settled.

## 2026-10-08 — Issue 079 CLOSED: the legacy `k == N` position binding replaced by the index-anchored law — typed content-bound at the default (0.5630; board posture + per_byte lever 0.5700, −0.25 pt vs the leaked 0.5725), sst5's ordinal binding byte-preserved, the perm control PASSES on all ten suites, instinct H2 typed seat digit-holds 0.6475 (T2 PASS)

**Status: owner-gated A/B/C resolved via the Claude verdict protocol (round 1 REVISE — four
amendments, all executed; round 2 re-confirmation RATE-BLOCKED at the reviewer's weekly
limit — the round-1 verdict + the measured gates carry the decision; the posture is
reversible: `legacy_kn_route` + `--drafter-fix` are both flags).** Found by bench 128
(issue 077): option AT position i bound domain i whenever by-name failed and k == N —
typed's criteria phrases are workflow-unrelated, so the binding was cross-workflow
position noise (typed control 4.70 pt median swing, 68.3% flips — the probe's last RED).

- **Instrument first** (`59b02c8` + `03533fb`): `EngineConfig::legacy_kn_route` (default
  true = incumbent bytes) + the `--no-kn-route` harness flag reaching every modelless
  build incl. the perm probe (route posture disclosed in the probe meta) + the
  `--drafter-fix <mode>` flag (the tuned-posture axis; oc_scale re-selects per posture:
  incumbent oc@4 → content oc@0.25). En route: `b3520a5` fixed the stale
  `train_docs_rules` expectations (red on develop since the 077 content binding).
- **The leak decomposition** (verdict amendment a; `scripts/issue079_gold_position.py`):
  gold piles on position 2 (40–45% sampled vs ~25% chance share, positional chance
  0.2333) — the binding harvested that pile by construction. Perm-averaged plain-posture
  baselines: incumbent 0.1833 vs content-bound 0.1000 (probe posture has no levers —
  different population from 0.5725; disclosed).
- **Tuned B** (amendment b): off 0.5630 · per_byte 0.5700 (chosen) · ncd 0.5670 ·
  shared_prefix structural no-op (0/600 typed questions share a prefix) · key_only 0.5535.
- **The G3 finding** (amendment c predicted it): sst5 also rode the blind fill — 0.3967 →
  0.1567 under knob-off (option strings are the ordinal level PHRASES, domains the index
  strings, by-name can never arm). But sst5's binding is LEGAL — score questions are
  ordinal (position = level = the corpus's index label; the noul-precedent class). So the
  landed fix (`4ad25df`) REPLACES the blind fill with the **index-anchored law**: route
  arms via by-name (any kind) OR the option's index string matching the domain names
  (Score, k == N, names exactly "0".."N-1"); choice never takes the arm. Verified
  digit-exact: sst5 0.3967 preserved; typed 0.5630 == the content-bound bytes;
  full-registry diff vs incumbent: ONLY typed moves; perm control typed PASS 0.00/0.00
  ties-only; 339 lib tests green (`f068ae6` names the disclosure string).
- **Cross-host**: the full 4090-windows cell at the board posture — bit-identity PASS
  10/10 (hard + calibrated abstain + cases_digest identical; ULP auxiliary drift
  disclosed). Board posture: `--skip-laya --nb-select --oc-select --ridge-select
  --drafter-fix per_byte`; typed reads 0.5700 (README ³⁵³⁶).
- **Downstream re-read** (amendment d — actually run, not footnoted): instinct H2 typed
  hybrid seat digit-HOLDS 0.6475, T2 PASS LB95 +0.0657 (improved from +0.0580; the H2
  pick path is engine-independent, only the A0 denominator moved) — `riir-instinct`
  Bench 0058, commit `6ec4eb2`; their arena also carried a pre-existing
  `presented_keys` array-spelling regression fix (typed had been unseatable on their
  develop since 10-03), disclosed there. laya/typed re-read RED 4.12 pt unchanged (the
  trained-encoder lane; riir-train Research 471 item 4's option-shuffle augmentation is
  its known lane, not this engine).
- **The trade, stated**: −0.25 pt hard (0.5700 vs the leaked 0.5725) buys perm-invariance
  on a wire that declares no option order; calibration improves at the default (ECE cal
  0.0192 vs floor 0.2778; incumbent 0.0496/0.1818) and stays under floor at per_byte
  (0.0946/0.2164); selective coverage shifts toward coverage (736/2000 at sel-acc 0.5747
  vs incumbent 566 at 0.6537). Owner follow-ups recorded, not acted: reflex-site
  republish (two-host cells in the record), the instinct `REFLEX_BASELINE_SHA` bump.

Bench 131 `.benchmarks/131_kn_route_ab.md` (renumbered from 130 — the wanli sibling's
committed `130_wanli_en_baseline` took the number first; `dual_allocation_gate` green
after the renumber). Commits: `b3520a5` `59b02c8` `03533fb` `4ad25df` `f068ae6`.
Note: the wanli sibling's runner.rs hunks rode `03533fb` (shared-worktree staging
overlap), acknowledged in their `459d344` message.

## 2026-10-08 — Issue 078 CLOSED: the `--d1` lane landed — d1-3B measured third on typed (0.6510) but BEST-in-family calibrated (ECE 0.037–0.051 raw, no temperature artifact shipped), order-biased on 3 of 6 probed suites; Windows torch cannot run their one-pass tree (flash kernels absent through 2.11+cu128) — the windows-split posture disclosed

The lane (`src/lanes/d1.rs`, mirror of drex.rs on their official
`/decisions/v1/systemone` wire, `D1_SERVE_URL` default loopback:8078; `--d1` flag,
`SuiteResult.d1`, `RunMeta.d1_lane`, perm-probe `--d1` mount, JDI crosswalk + dump-items +
their-confidence render joins; 11 module tests incl. the stub-listener round trip) + the
reference server (`.raw/d1_server.py` + `boot_d1.cmd` — their in-repo `D1Model` UNMODIFIED
behind a stdlib listener, the Drex-lane shape; their repo ships no HTTP layer). License law
discharged: `lfm1.0` = LFM Open License v1.0 — $10M-revenue Threshold on commercial use,
measurement-only research licensed, redistribution per §4 (not NC, not Apache/MIT; the
blog's "without restrictions" oversold it). **The windows-split finding:** their
`hybrid.py` tree path calls `aten::_flash_attention_forward` / `_scaled_dot_product_flash_attention`
directly, and NO Windows torch wheel compiles those kernels — the `USE_FLASH_ATTENTION was
not enabled for build` guard verified failing on torch 2.6.0+cu124 (drex-env) AND a
throwaway 2.11.0+cu128 venv (deleted after; a build flag, not a version gap). The server
answers multi-question requests one question per `system_one` pass — the vendor's
single-question reference path, code unmodified ("mathematically each row alone"); kernel
batch shapes + per-question state re-encode disclosed beside every cell, input_tokens
matching their own single-question accounting. **Cells** (Bench 129
`.benchmarks/129_d1_lane.md`, 4090-windows, fp16 · calibration=None · windows-split,
preflight PASSED): typed 0.6510 (choice 0.6033 / noul 0.8050 / score 0.5713) · xnli 0.8167 ·
massive 0.9067 vs their-card 87.3/85.0 on THEIR split — crosswalk, never a board row;
typed ranking AgentJev 0.7715 > laya 0.7445 > d1 0.6510 > modelless 0.3345. det ✓ all
suites (observed-repeat, byte-identical under the latency-tail strip). **The calibration
claim SURVIVES the open weights:** the card says "calibrated" but the open weights ship NO
temperature artifact (`config.json` none, `D1Model.engine` passes `calibration=None`) and
the raw softmax still reads ECE 0.037–0.051, beating its split-half conformal floor
everywhere (0.15–0.42) — the per-type temperatures are a hosted-tier artifact we could not
test; the confidence==max-prob coincidence is disclosed (one axis, two names). **T4 perm
probe on d1:** RED ×3/6 (typed 7.70pt median / 13.3% flips, emotion 4.92 / 12.5%, banking77
5.58 / 10.3%), PASS ag_news/xnli/massive — their option-shuffling training lever bought
robustness on the wide suites only; canary flips on every suite (the probe provably fires).
The typed modelless-control RED (4.70pt/68.3%) is the standing Issue-079 owner-gated
finding, disclosed (the same shape as Bench 128), not a regression. T6 stays DEFERRED
(the issue's own task list, preserved here now that the file is removed): SQuAD2.0-as-noul
external abstention-anchor suite + the wider seven-benchmark panel
(SQuAD2.0/CivilComments/BoolQ/PubMedQA/PAWS-X) as harness suites; the hosted-`d1:free`
posture stays owner-gated/closed. Landed: lane `f6337a1`, bench + closure `4310988`.

## 2026-10-08 — Issue 077 CLOSED: the option-permutation probe falsified "invariant by construction" and the fix landed — content binding for the classification suites; drex measured order-biased, agentjev content-bound, laya tail-fragile

The probe (`--perm-probe`, exclusive early-exit mode): every probed lane answers the SAME
choice questions under K deterministic option orderings (identity + 4 SplitMix64 shuffles,
`slot_seed(case_id,qid)`; every permutation vector recorded) through its OWN production
decide path (the `ChoiceOracle` seam — never a parallel rendering). Pure core
`src/harness/permutation.rs` (orderings, permutation, label-space mapping, spread math, the
≤2pt gate, the canary; mock-lane tests pin the canary both directions) + lanes in
`src/harness/runner/perm_probe.rs` (modelless control / drex / agentjev / laya primary-
checkpoint; bucket-skip convention; stride sampling; strict-byte + tie-aware control
columns). **The control FALSIFIED the issue's premise on run 1** (Bench 128
`.benchmarks/128_perm_spread_option_order.md`): typed 4.70pt median / 68.3% flips, ag_news
0.95 flip rate, emotion/xnli 0.35/0.37 — root cause the legacy `k == N` index alignment in
`solve_sample_into` (option-at-position-i gets domain-i's route term when by-name fails),
armed on every suite whose domain names were the train docs' INTEGER labels; plus the L1
normalizer's presentation-order f32 sum (ULP class, bounded < 0.005pt, disclosed not fixed —
a canonical sum would move the canonical bytes published tables pin). **The fix**
(content binding, canonical-order byte-identical): `train_row_label` maps the int class
label through hoisted option-key consts (`AG_NEWS_KEYS`/`EMOTION_KEYS`/`XNLI_KEYS`/
`WISESIGHT_KEYS` — builders AND label rule, ONE spelling for corpora/stratification/audit/
domains); prepare labels := option-key union with the fetch guard now the EXPECTED KEY SET
(exact membership pin). Post-fix: ag_news/emotion/xnli PASS at 0.00pt / 0 flips (was
0.95/0.35/0.37); massive/banking unchanged PASS; strict byte-identity stays a disclosed
column (control_invariant) with the control verdict keyed on ties+the 0.01pt measured
envelope. **typed_decisions stays RED** (its 4-option questions ride the k==N binding
against 4 workflow domains — load-bearing for the published typed numbers) → issue 079
(owner-gated, three priced options). **Comparison lanes** (servers booted, preflight
PASSED, G5 parity GREEN pre-laya at the CPU posture, 2/2): drex RED ×5 (emotion 14.92pt
median, typed 11.93pt, banking 8.65pt, massive max 76.29pt; canary flips on EVERY suite at
27.5pt — the d1 recipe's "shuffling answer options" failure class, independently measured
on a third-party decision model); agentjev PASS ×6 (0.00–0.27pt, the recorded wobble class
disclosed); laya median-holds/tail-fragile (english 0.01–1.53pt medians but max swings to
99.97pt and 30% flips on massive; typed RED 4.12pt) — riir-train Research 471 item 4's
measured evidence that option-shuffle AUGMENTATION is worth its LENC-cache cost for the
laya family. Wire-up: `--perm-probe` / `--perm-k` / `--perm-max-cases` (comparison lanes
ride --drex/--agentjev; laya rides its feature + --skip-laya); a control red refuses the
run verdict after writing the record. Issue file removed per the noise rule; the follow-up
lives in `.issues/079_typed_kn_route_binding_owner_gated.md`.

## 2026-10-08 — Issue 076 CLOSED (Drex follow-ups): the det ✗ was the timing tail, measured twice more; the per-kind cells are lane surface

Three tasks, all landed: T3 the `decide_raw` latency-tail strip + test (`cd0248b`, from
Bench 125's control — a raw byte-compare measures their clock, never the decision); T2 the
per-kind confidence cells + split-half conformal floor on the upstream lane (`7162840` —
`map_confidences` reads THEIR wire confidence per question (noul None, never fabricated;
missing field = loud error), `assemble_drex_conf_readout` through `harness::metrics` only
(`DREX_MIN_FLOOR_PAIRS` 40, first-half-cal/second-half-test), carried on
`LaneResult.drex_conf_readout` skip-when-none, plus the four surfaces the upstream lane was
missing entirely: TABLES.md row, the their-confidence detail line, JDI crosswalk inclusion,
the meta line); T1 the BF16/CUDA det re-read on the 4090 (Bench 126
`.benchmarks/126_drex_det_reread/`: **det ✓ 10/10 under the strip** with accuracy,
readout_ece/brier and token counts EXACTLY the pre-strip T5's — 0.5865 / 0.1929-0.2663 /
191,962 in · 164,821 out — so no measured reduction nondeterminism exists on that posture,
and the agentjev lane's recorded det ✗ carries the same artifact — fixed the same day by
the `usage.wall_ms` strip in `AgentJevLane::decide_raw`, the Drex law mirrored, and the
post-strip re-read measured the same day on the 4090 (Bench 127
`.benchmarks/127_agentjev_det_reread/`: **det ✓ 10/10 byte-identical** at their CUDA/BF16
posture, acc 0.7720 == bench 123's cell exactly — the det ✗ was the timing tail, all of
it; their bf16 wobble unfired in 20 observed requests; Plan 011's "unquoted (det ✗)"
caveat is LIFTED at the current posture). The
not-calibrated verdict is now THREE-posture: choice 0.1471 / score 0.2477 vs split-half
floor 0.1220 at BF16/CUDA (Bench 126) agreeing with Q8_0/Metal's 0.1460/0.2495 vs 0.1158
(Bench 125) — the card's disclaimer beats the homepage marketing everywhere measured.
Research note 008's status line refreshed to the closure records.

**Collision record (the Issue-825 class, resolved clean):** a sibling session ran the SAME
T1 independently on the same box at the same time (their `e7111bc`, unpushed; host label
`shikuwa` vs this session's `4090-windows`) — byte-agreeing cells, preserved as
`results.sibling_run.json` in the bench dir, their 073 REFUTED addendum salvaged verbatim,
their dir folded into the canonical `126_drex_det_reread/`. Their session's lesson, kept:
the run dir was first named `076_*` after the ISSUE number — bench numbers are their own
counter (the `.benchmarks` highwater was 125); renamed before commit. Neither preflight
saw the other as more than a transient load tripwire.

## 2026-10-08 — Issue 075 CLOSED (the G4 800-alloc red at the nb_ridge posture): a per-question env lookup, not the ridge math

Root cause measured, not guessed (`284cc98`): `std::env::var_os` ALLOCATES on Windows even
for a MISSING variable (probe: 1000 missing-var lookups → 1000 allocations, standalone
counting-allocator binary), and the `[ridge-dbg]` guard read the env PER ROUTE-ACTIVE
QUESTION — 4 of the bench request's 8 questions (noul never takes route terms) — so the
G4 loop's 200 solves × 4 = exactly the 800 allocations the gate counted. The issue's
"likely shape" (ridge readout allocating / katgpt-core 0.4.1 growth) was WRONG: the ridge
math (`in_score`, `blend_term`, the capacity-retained scratch buffers) was already
alloc-free; the issue's option (b) (re-gate the G4 arm) was also wrong — the arm was
correct to arm the serve posture, the code was in violation. Fix: the flag cached in a
`OnceLock<bool>` (`ridge_debug_enabled`, engine.rs) — one-time init rides the first solve
(warmup); a var set after that is not observed (debug flag, never a config surface); the
per-question Win32 env call + lock leave the hot path entirely. Attributed-pre-existing
confirmed: the lookup landed with the readout itself (issue 038 T7a), dormant until the
073 session's final guard run reached this cell red. Validation: the bench at all four
postures G4=0 (nb_ridge 800→0, G2 p99 45 µs); `ci_feature_guard.sh` layers 1–8 green
(clippy ×5 postures, 303 lib tests, G5 laya parity, bench, docs shape); flag semantics
verified live (RIIR_DEBUG_RIDGE=1 → 2812 [ridge-dbg] lines = 703 solves × 4). Windows
lesson for the record: a fresh `sentencepiece-sys` C++ build in an ISOLATED target dir
fails MSVC C1056 under 24-way cmake parallelism (the katgpt-rs load-robustness class:
C1001/D804/C1056) — the repo's warm `target/` never rebuilds it; cap
`CMAKE_BUILD_PARALLEL_LEVEL` when a cold laya build is unavoidable.

## 2026-10-08 — Issue 073 CLOSED (the Drex DLM comparison lane): measurement-only, card vindicated over homepage

Closed by the sibling session the same day it landed (T1–T4+T6 @ `48206d5`, T5 @
`a1bae34`); the issue file's own status block said "removed per the noise-reduction rule"
but the removal never happened — this entry completes the closure and the file goes now
(hygiene, nothing references it but a path line in the bench record, updated in the same
commit). The standing documentation is `.benchmarks/073_drex_lane_t5_suite_pass.md`
(serving posture + box state + turnkey serving setup) + the `src/lanes/drex.rs` module
doc. Headline cells: typed_decisions acc 0.5865, readout_ece 0.1929 / readout_brier
0.2663 — their confidence is NOT calibrated (the card's disclaimer vindicated over the
homepage claim); sst5 smoke 0.5900 above every published sst5 bar (disclosed board
re-pricing, never a seat — CC BY-NC, measurement only). Open follow-up: issue 076 T1
(the BF16/CUDA det re-read under the `latency_ms` strip).

## 2026-10-07 — Issue 072 CLOSED (`--dump-items`): the per-item outcome dump instrument served rethink 028's POC the same day it landed

Landed as `fd8e832` (same commit as this issue file, per the cross-repo hygiene rule) and closed the
same day: riir-rethink's Bench 062 (population-scaling POC, Issue 028 T1–T6) cites it as its
instrument — the close condition. What shipped: `ItemOutcome` (`src/harness/item_dump.rs`, pure +
ungated), `LaneResult.items` (`#[serde(skip)]` — always built in memory by both lane tails
(`run_modelless` + `assemble_laya_lane_result`, covering every laya-family oracle lane), never
serialized into results.json), the `--dump-items` CLI flag persisting
`<out>/items/<suite>.jsonl` (join key `(case_id, q_idx)`; paw lanes disclosed-not-dumped), and
`AbstainCause::as_str()`. Verified byte-neutral on results.json (only run-varying
latency/load/date fields differ between flagged and unflagged runs) and cold-run-gated on emotion
(400 items, item acc == table acc). The frozen evidence (BLAKE3-pinned item dumps + the POC
analysis) lives in riir-rethink `.benchmarks/062_population_scaling_poc/`.

## 2026-10-07 — Plan 011 CLOSED (the Clef comparison lane + JDI protocol adapter): the C4 headline verdict on the local posture

The owner call (2026-10-07, "close on local posture") supersedes C4's verdict-round-3 gate (the
hosted-Clef deciding cell): the hosted Workers-AI run never became credentialable — both provided
CF tokens lack Workers-AI scope (probed 2026-10-03: 403 `ai/models/search`, 401 `ai/run/clef[-flash]`,
account `7e11517c4dd4f6e9cede7da9b60d66eb`; the scoped token was never minted) — and the owner had
already pivoted to the open weights ("download model and run it like other", 2026-10-03). **The
verdict — both sides live-measured, quotable, on IDENTICAL population pins (`fnv1a64-6e37760ee2a5b6c9`
typed · `fnv1a64-dd8ab35333abb82a` b77), the crosswalk's digest asserts holding on every row:**

- **typed_decisions (400 cases / 2000 q): RETHINK LEADS** — encoder 0.7550 acc / 0.7445 macro-F1 /
  0.6537 skill @ p50 47.335 ms (re-seated quotable, rethink issue 021 / Bench 061; serve-refused
  per the 014 class-wide latency class — record cell, the incumbent arm serves) vs Clef-local
  0.6955 / 0.6742 / 0.5696 @ p50 1690 ms (bench 118 quiet-box): **+5.95 pt accuracy at ~36× lower
  p50**. Instinct hybrid 0.6475 third; AgentJev's 0.7715 stays unquoted (det ✗).
  **2026-10-08: caveat LIFTED** — the det ✗ was their `usage.wall_ms` timing tail in the
  compare input (Bench 127: post-strip det ✓ 10/10, acc 0.7720 at the current
  `@68883998` posture); the 0.7715 cell is quotable again with its det column clean
  (see the bench-039 addendum 2).
- **banking77 (500 cases): CLEF-LOCAL LEADS EVERY MEASURED LANE** — 0.9540 / 0.9536 / 0.9533 @
  p50 3368 ms vs Instinct 0.8540 (best other), modelless 0.8420, bekko 0.7920, openthai 0.6560.
  The Rethink encoder carries NO b77 cell (tier-fallback, excluded by construction — rethink
  issue 016 D1 seating, owner-gated).
- **Cost + latency posture (disclosed):** Clef = `mlx-community/clef-flash-4bit` (community 4-bit
  9B quant) on this M3 via the local MLX rig (loopback HTTP, `.raw/clef_srv/`, zero marginal
  spend); Rethink = local Metal encode (laya-typed encoder + NLEH v2 per-option head).
  **Hosted caveat: this is NOT the blog's hosted 27B Clef** — a hosted 27B cell could still move
  typed (b77 at 0.9540 has little headroom); that cell stays unmeasured for want of a scoped
  token, and the B5 crosswalk caveat (different corpus/protocol caps/hardware vs the JDI board)
  governs every row. The site already renders these verdicts data-derived (TL;DR card,
  reflex-site `dd5409d`; rethink cell `260705a`) — this entry is the record, not a new publication.

Evidence chain: [bench 113](.benchmarks/113_clef_lane/RECORD.md) (full-N) + [bench 118](.benchmarks/118_clef_quiet_reread/RECORD.md)
(quiet re-read) + [bench 119](.benchmarks/119_clef_lane_tail/) (9-suite tail); rethink Bench 061 +
issue 021; site edition 2026-10-2 (`5c31fa6` lane cells → `4297d32` timing graduation → `365d0a0`
9-suite → `53d518f` crosswalk → `dd5409d` TL;DR card). Dispositioned defers at close (never
executed; conditions recorded in the plan rows): A2.5/A6 (hosted creds — the wire golden fixture
implements FROM a real capture if a scoped token ever lands), D1 (Clef-27B prefill-league
workload, own session per Phase D), D3 (JDI board submission, owner-gated). The lane (`--clef`)
and the JDI axes (macro-F1 / chance / skill / coverage) remain live harness surface; D2's recipe
note lives in riir-train `.research/464` (`115e5690`).

## 2026-10-07 — the artifact-lane structural adoption (riir-ai Plan 623 T5): manifest seed + gitignore law + gate green; the synth-corpus durability finding recorded

`artifacts/manifest.toml` seeded (schema v0, ZERO rows) + the directory law (`artifacts/**` + `!artifacts/manifest.toml`); `ci_artifact_boundary --repo` PASSED zero findings and the sweep reads `✓ riir-reflex: migrated`. **Inventory finding — the repo's ONE irreplaceable-in-scratch class:** `.raw/corpus_synth*` (the SYNT corpora, 18 files / 22 MB with their run artifacts) are teacher-generated (openthai, 3,523 forwards / ~4 h per SYNTH.md), blake3-pinned, NOT bit-reproducible by re-run, and NOT committed — git is not their backup. **Interim durability landed: cold archive** at `/Volumes/SDXC1TB/moat-archive-20261005/riir-reflex/` (copy-first, b3sum-verified 18/18 byte-identical, nothing deleted from the source tree — the riir-train 613 interim pattern). The PERMANENT home pends an OWNER CLASS DECISION: this repo is PUBLIC, so committing/publishing the corpora is a publication act (licence note: teacher-generated text conditioned on MASSIVE CC BY 4.0 seeds — plausibly public-class, not adjudicated here), and the protected class pends T8 provisioning (scoped DEKs + buckets). Datasets (.raw/datasets*, ~170 MB) adjudicated re-fetchable (source-pin class, never artifacts); game-head mint fixtures + frozen captures are committed (git = backup); bekko-env/clef_srv-class caches are the archived/derivable classes.

## 2026-10-06 — Issue 071 CLOSED (the harness's duplicate SplitMix64 streams → one substrate home)

Duplicate SplitMix64 (`suites.rs` + `echo_gates.rs`) consolidated: minimal `479b042` + substrate half katgpt-rs `514989664` exporting `katgpt_types::rng` + reflex `147309c` re-exporting `katgpt_core::types::rng::SplitMix64`; frozen-golden bit-identity pin, lib 288/288. seal-view/seal-node copies adjudicated the one-shot-hash-mix class (NOT delegated); the entry carries the 070→071 renumber record (`66a195e`, first holder keeps the number).

## 2026-10-06 — Issue 069 CLOSED (the public artifact fetch lane, riir-ai Plan 623 T6)

`scripts/fetch_artifacts.sh` — pulls public-class assets from `hf://gist-rs/<repo>-artifacts` into `artifacts/cache/`, BLAKE3 + exact-size verified before use; protected rows refused; self-contained (a fresh public clone is the acceptance env); the workspace-internal power tool is riir-deployer's `artifact-sync` (riir-ai Plan 623 T7). No manifest = honest no-op; AGENTS.md/BOUNDARY.md wired, drift row A1 closed WITH the issue.

## 2026-10-05 — Issue 070 CLOSED (the eval path's allocation surface: the scratch-refill face `eval_case_into`; serve-path pins 83 → 42 and 157 → 79)

Filed from instinct Issue 021's close-out: `eval_seat_abstained` (`1701f82`) + `eval_case_into`/`CaseEvalScratch` (`e0c43c7`) — one caller-owned frame, `engine_request_into` refill, `answer_probs_pick` the ONE noul-flip/forced-argmax law; parity gate in `tests/harness_seat_gates.rs`.Consumers: instinct `9ac84c8` 83→42, Rethink `c46ef1a` 157→79; frozen-picks replays prove byte parity; remaining allocation is katgpt-core's wire response boundary.

## 2026-10-05 — Issue 068 CLOSED (the lane HTTP micro-client extraction: one `parse_http_response`, chunked handling for every lane)

`src/lanes/http_mini.rs` (`d12e5d6`) — one std-only HTTP/1.1 client; clef's chunked decoding folded in for EVERY lane; `LaneHttpError` Display formats byte-preserved; paw deliberately NOT a consumer (curl subprocess). Stub round trips green through the shared client: openthai 10, agentjev 7, clef 10, clm 12; wasm32 `--no-default-features` clean.

## 2026-10-05 — Issue 066 CLOSED (the fused-abstain density half: wired, measured, not certified — the many-label signal recorded)

[Bench 124](.benchmarks/124_density_gate_ab/RECORD.md) — the LSL App-E density half (arXiv:2610.02126) as the `density_gate` feature over katgpt-rs `gmm_support` (Plan 618 `1d8da4862`, Bench 908); verdict NOT CERTIFIED (LCB95 < 0 all six suites at ρ=30). The many-label signal: massive_intent_en +0.0408 / banking77 +0.0344 selective accuracy; few-label suites negative; `DENSITY_VAR_FLOOR = 1e-2` re-pin; τ provably inert; wire taxonomy closed; re-open triggers in Bench 124 §Verdict.

## 2026-10-05 — Issue 065 CLOSED (the quotable-timing backlog: every comparison lane now plots)

[Bench 123](.benchmarks/123_lanes_rerun_4090_boxstate/RECORD.md) — four 4090 lane re-runs, one window, BoxState-stamped (openthai 11/11, clm 8/8 44.46 ms, agentjev 8/8 45.17 ms, gliner 8/8 22.85 ms); modelless controls bit-identical everywhere. Real find: the cp874 STDIN decode — `sys.stdin.reconfigure(encoding="utf-8")` fixed in gliner/bekko/laya_python lanes; massive 0.8233→0.7267 and banking77 0.7060→0.7100 are the corrected honest cells.

## 2026-10-04 — Issue 064 closed (the projection-ascent density gate: works as designed, buys no accuracy)

[Bench 120](.benchmarks/120_synth_density_pilot/RECORD.md) kill-gate PASS; [Bench 121](.benchmarks/121_synth_density_p50_corpus_ab/RECORD.md) + [Bench 122](.benchmarks/122_synth_density_p75_dose_response/RECORD.md): lift tracks row count and UNGATED beats every gate strength — "a remedy without a disease"; the seated corpus stays ungated. Density-first ordering saved ~11 h of teacher calls; Task 3 (share the operator with riir-train Plan 438 Phase 2's RIDT v2) stays deferred-DRY; the instruments (`--synth-density-gate`, echo gates) stay zero-cost.

## 2026-10-04 — Issue 054 closed (all four parts terminal)

Part 1 openjev lane TERMINAL (owner: the 27B model is too huge); Part 2 prefix-state REFUTED for laya (ModernBERT is bidirectional — no causal prefix state; shape stays valid for GDN decoders); Part 3 landed in `.research/006` (prefix-tree root sharing, bit-identical); Part 4 recorded (question-batching is a first-class System-One wire contract; seed via Issue 061).

## 2026-10-03 — the v0.2.4 release cut

Tag `v0.2.4` → `e616814` (code identical to `4bea9fc`; docs-only sibling commit not force-moved over), 6 assets, brew tap `813cb68` + scoop `596e156`; headline lanes: Issue 063 first-corpus (`RIIR_REFLEX_CORPUS`, `/healthz` `"corpus":…`) + Issue 062 wire id fix; game heads MINT-ONLY (boot-fit retired, the release notes' breaking change). Gates 9/9 (incl. the flag-OFF layer the cut UNBLOCKED), G2 p99 44 µs / G4 alloc-free with `PROVENANCE:` quoted (power=AC, powermode=2, canary 113.9 µs/best5), leak scan ×5; disclosed: darwin binaries embed two id-shaped strings in Metal-kernel comments (`Issue 020`, katgpt-rs `Plan 616` — wire-inert, stripped in the laya substrate); site reflex-site `cce66a5`+`dfdd6d5` (CF `c345c993`).

## 2026-10-02

thai_sib200 pool floor 560→400 (`slice_guard::MIN_POOL_ROWS`, re-based on audited pool-after-cal); Bench 110 OpenThai EXL3 4.0-bpw board NO-GO by the pre-registered letter, PROVISIONAL (riir-infer Plan 617 Phase A5, `617_openthai_exl3_4090*`); `.benchmarks/.highwater` normalized single-line + `scripts/thai_rerun.sh` hardened (`head -n 1`). The six harness families RETIRED (owner call — home-made synthetic evals (Issue 004 / katgpt-rs Research 579) the modelless engine reads AT CHANCE on wide evals; reflex-site board rows removed, instinct arena 15→9); Issue 061 `semantic_defects` LANDED (102 cases, first modelless cell 0.1863 vs 0.1667 chance); fixture-fleet hygiene wall `3a307ee` (`fixture_hygiene.rs` + `tests/fixture_fleet_hygiene.rs`; code_fixtures pinned the fleet's WORST contamination, 0.9494 mean / 1.0000 max). Issues 058 (cal-selection ladder: root cause = silent datasets-dir move; TRAIN_CAP 20000 + `slice_guard` floors + canonical pool rebuild; Benches 100/101 board restored 15/15 bit-identity; deferral triggers recorded), 059 (wide-eval withdrawn then REVIVED as the quarantined our-lanes web section, REVISED-2), 053 (C8/C9/D6 ratification states), 060 (abstain CAUSE per case, `AbstainCause` 3-key taxonomy, `a7475c7`), 055 (MC ensemble PoC null, Bench 092) and 056 (Platt defect root-caused substrate-side, katgpt-rs Issues 909/910/911; `--gate-fit-calibrated` promoted default, Bench 095 15/15) all closed with records in their rows.

## 2026-10-01

Issue 057 CLOSED (Bench 102 + reflex-site `8d33e3f`+`2b699f0`): the LANE_CARRY corpus-axis gap — every host slot can now refresh published timing on a corpus change; the `_carry_into` defect (digest(new pool) + timing(old pool)) made m3 typed_decisions permanently unrefreshable. Fix keys ack staleness on own-timing incumbents (self-test 67/67); m3 typed cell refreshed to p50 0.782 ms at acc 0.5725 bit-identical.

## 2026-09-30

Plan-426-T6 seat half `776e8db`: `seat::prepare_seat_with_synth` — blake3-verified overlay applied at BUILD time, `specs_corpus_extended` SHARED with the corpus-ab lane; consumers instinct `c7b7165` (`INSTINCT_SYNTH_CORPUS_DIR`). Benches 096+097: the promoted gate posture republished on both hosts (hard accuracy bit-identical on all 15 suites; reflex-site `0ce46a8`, CF `6f5496e3`); en-route Issue 057 filed (typed m3 timing carried). Issue 054 Part 2 REFUTED (`prefix_state_coupling.rs`, riir-infer `ff5de37`: laya = ModernBERT/mmBERT bidirectional — shared-state drift 5.1e2 vs the 1e-5 budget; the probe is two-sided). Issue 056 CLOSED: the calibrated-ranking regression root-caused to a Platt-solver defect substrate-side (katgpt-rs Issues 909/910/911, commits `74e9d192d`→`b0d80979d`→`77eacda8f`, the Lin–Lin–Weng 2007 solve + w floor + AUC guard); `--gate-fit-calibrated` PROMOTED default (owner-delegated verdict; Bench 095: AUC regression gone 15/15). Issue 055 CLOSED (Bench 092: u_pair loses 8/8, LCB null; the PoC's real find = the calibration regression that became Issue 056); repair-0 `--gate-fit-calibrated` landed opt-in first (Bench 093).

## 2026-09-29

Bench 090: paw-LOCAL twins for the new suites (accuracy-neutral vs hosted, det ✓ ×4, byte-identical ×2; the rule-had-not-generalised class fixed at root — `paw::resolve_shapes` is the ONE per-shape resolver for BOTH lanes; M3 venv + sccache trap recorded). Benches 088+089: the PAW dataset board COMPLETE — 18 programs, both compiler tiers on 088 (ft 0.6379/0.7200/0.5100 vs base 0.6983/0.5833/0.0967; massive sampled-presentation refusal class), typed 0.5925 ft with MAE 0.6485 / within-1 0.7937 (second behind agentjev 0.7715). Bench 087: per-question-shape PAW programs (`<suite>.<qid>.txt`) + the first `code_fixtures` cell 0.6250 ft-bs48. The 084 publish arc + the corpus-synthesis lane `8426cef` (riir-train plan 426: SYNT v2 artifact + blake3 sidecar, openthai agreement veto, `--synth-corpus`/`--synth-plan`/`--corpus-ab`); Benches 085+086 (M3 clean re-read + lane fill; option-count scaling law 1.2×→17.2×).

## 2026-09-28

Bench 084: openthai all 17 suites on the 4090 (fp32 pinned via `OPENTHAI_SYSTEMONE_DTYPE`; massive 0.9200 == M3 exact; publish steps + `PUBLISH_BENCH_LANES` law recorded in the doc's §Publish). Issue 047 CLOSED measured-negative (Bench 073, pre-reg `.plans/006_nli_m1_reopen.md`: head 0.5205 < A0 0.5410 on the fresh validation slice; the shipped calibrated readout on xnli is NEAR-BINARY — honest confidence work starts from the RAW max-prob surface, AUROC 0.6542; the xnli −28.2 pt / ag_news −6.8 pt gaps stand ACCEPTED). Issue 045 CLOSED (Bench 072: `harness_cache_reuse` answers modelless 0.9167 vs LLM 0.5000 via the noul count-table polarity, cal-selected; `selection_slice` synthetic fallback + reflex-site publish `d046a67`). Issue 050 CLOSED (nb_ridge honestly declares `option_cond`); build-stamp pin fixed for the flag-OFF posture; instinct Proposal 001 T7 reflex half landed (`e66588c`: the Head vessels section, pins-first-wildcard anchoring, sibling-layout un-stale).

## 2026-09-27

Issue 048 CLOSED (Bench 071: the combined-posture LCB leg is a PROVABLE no-op, all ten rows byte-identical; the Bench-070 repro byte-exact incl. probe_lcb floats; watch item = sst5's combined LCB +0.0152). Issue 046 CLOSED (Bench 070: `--cascade-worthiness-lcb <F>` — ag_news/massive tie is a SUPPORT problem; armed ag_news +0.0675 exactly as preregistered; honest limit: support-confidence, never a cal→test-shift guarantee). Issue 044 CLOSED (Bench 069: the G1-constrained blend UNSATISFIABLE — λ*=0, the mini-G1 screen is not a transferable predictor at n_cal=200; Bench 068 `fe7c123`: routing/sensitivity gaps are n=3 artifacts, code_fixtures frozen `code_fixtures_frozen.json` — the +41.7 pt gap collapses, xnli pair-head +5.67 pt REFUSED on G1). Issue 042 CLOSED (Bench 066 `842f89d`: the COMBINED posture `--gate-fit-selection --gate-distance-only` @ margin 0.16 is the first passing full T4′ acceptance; sst5 twice-measured arm-side flip — a third flip files a per-suite probe-size floor). Issue 043 RESOLVED (positional-prefix hypothesis REFUTED as dangerous; the stratified lanes already select the right polarity; shipped row 0.7672 stands). Issue 038 CLOSED (T7c BM25-kNN + T7d Hebbian REFUTED, `scripts/issue038_t7cd_probe.py`; scoreboard: modelless ≥ laya best on 5/9). Issue 041 (Bench 065: ag_news full-pull volume NEGATIVE, +0.25 pt; `scripts/fetch_datasets.sh` gained a SUITES filter). Issue 038 T5 genome lane (Bench 064 `--genome-select`: banking77 0.8260→0.8620 + sst5 ADOPTED; massive REJECTED on G1; RRF declined). Issue 042 T3 decided (a) — fixed margin 0.16, Bench 063 re-run 10/10 PASS; lever 3 `--cascade-worthiness` (Bench 063, 8 suites). Issue 040 RESOLVED (lane-pairing guards: `cases_digest` + `check_lane_pairing.py` + `PUBLISH_ALLOW_SAMPLE_MISMATCH`; Bench 062: rust == py exact on all 15; renumber note 061→062). Issue 038 T7(a)+(b) (Bench 057: `option_cond` typed 0.33→0.4655, `nb_ridge` emotion 0.7375→0.8475, both promoted default-on; cross-host bit-identity re-proven 14/14). Issue 019 CLM T-Rex (Bench 059, `.benchmarks/057_clm_trex_4090/`: ours shield-on 5/5 survive / 0 deaths; the vendor 16.5 ms is the WARM steady state; window laws inherited from Issue 027).

## 2026-09-26/27

Issue 033 PAW lane CLOSED (record hash-pinned here): Posture A hosted + ft-tier cells (Benches 049/055/054 — the Issue-825 twin, ⚠ the M3 record was RENUMBERED 053→055) + Posture B local llama.cpp (Bench 056, accuracy-neutral, det 4/4, the uv-venv trampoline stdin-deadlock trap) + the republish (reflex-site `dffabe3`: `paw`/`paw_local` lane classes, `shikuwa`→`4090-win` alias, `PUBLISH_BENCH_LANES` with `:acc-only`). Issues 036 (drafter-only degenerate on dynamic option spaces — all `drafter_fix` candidates fail the constant-skip floor, gate closed NEGATIVE), 037 (e8 table converter arm, riir-infer Plan 612 reflex half) and 039 (the 4000-row train cap truncates the label universe; Bench 052 stratified protocol; T5 4090 both-hosts identity 14/14 modelless) closed; Issue 039's banking77 cuda repeat-check filed as riir-infer Issue 021 (RESOLVED `c64d0b1`).

## 2026-09-26

Issue 020 CLOSED — the riir Metal lane beats python torch MPS on every published cell, p50 AND p99 (Bench 050: 9/9 + 9/9; full-lane 17/17 + 17/17, reflex-site `6074b2b`): ladder T2–T12 in riir-infer (waste removal, coalesced Wᵀ, packed multi-question forward `da30007`, dispatch band, flash softmax rungs, split-K `512477e`/`ed1cb74`/`9b0e55c` + folds `53334f9`, deferred head `40d15dd`), closer T13/T13b Apple `MPSMatrixMultiplication` (`b0de034`/`5e18da4`, kill-switches `LAYA_METAL_MPS=0`/`_SPLIT=0`); measured-negative rungs on record; the `.metallib` precompile + rope-table caching priced and DECLINED (riir-infer `startup_rope_probe.rs`; reopen triggers recorded). Issue 008 CLOSED — the riir-infer consolidation (carve `86a5986` registered as the 23rd contract repo katgpt-rs `bd2ce3cca`; T4 the encoder-lane move riir-infer `c6716a4`, G5 88/88 from the new home; T7 the CubeCL backend — Bench 006: 5.1–8.4× slower than Metal, kept opt-in `laya-riir-cubecl`; T6 both repos public 2026-09-23; standing: the weights cache path still names this repo). Issue 035 CLOSED — the cua-s1-forms CoreML arm (Bench 048: coreml 24,359/24,370 = 99.9549% == their published result; laya-typed 29.95%, modelless 4.36% — both below the 52.16% constant-skip floor → Issue 036). Issues 032+034 CLOSED — head-select publish both hosts (Benches 045 M3 / 047 4090, bit-identical; the publisher wipe's real mechanism = the FRESH-DOCS publish path, not the update path; wall `guard_wholesale_replace` + `PUBLISH_BENCH_FULL_REPLACE=1` in reflex-site `1bc4bab`); Issue 024 CLOSED (de-leaked columns landed; the T3-owed clippy caught `laya-riir` missing `dep:blake3`); Issue 020's typed-trio lever priced + T12 built opt-in (riir-infer `be46033`, probe `abbcbb3`).

## 2026-09-25

Issue 007 CLOSED (`corpus_db` P1 `307a10b`; P2 DECIDED — the trigger was measured fired, answered in-harness as Issue 024's `slice_leak`; deferred: corpus LOAD-from-kv until a consumer exists). Close-out sweep: Issue 017 (ANE lane: P0 six BC1S FP16 artifacts, P1 substrate runtime riir-infer `78a91c3`+`5c8ec2a`, G5-ANE 76/76; defers carried as posture), 023 (option-NAME route: massive 0.077→0.690, G1 FAIL→PASS), 025 (AgentJev positioning; Bench 039 typed 0.7715; the lane-3 serving port DEFERRED owner-gated), 027 (CLM T3 4090 window Bench 034 + the window laws: serialized passes, `--gpu-memory-utilization 0.72`, one fixed warmup, same-box latency, `scripts/clm_serve_4090.sh`), 029 (GLiNER lane Bench 037; the publisher header sha law kept), 030 (noul anti-alignment `36e4e0a` + lever-4 heads `69a6eae`) closed; Issue 031 RESOLVED (`fixture_pins()` four-hash hardening + the parse-precision note). Issue 030 lever 4 LANDED (bench 040 `src/label_heads.rs`: banking77 +23.8 pt, massive +10.3 pt; noul never takes heads; scale > 1 fails G1 by construction; arena protocol `--head-select`). Issue 030 noul route fix (bench 038 `36e4e0a`: prompt_injections 0.4397→0.4828; pin `noul_never_takes_route_terms_even_when_k_equals_n`). Issue 029 GLiNER lane LANDED (`scripts/gliner_lane.py`; beats the laya BASE checkpoints on 9/15, loses classic NLU, does not touch the `typed` specialist; site lane-filter bar). Issue 027 executed (Bench 034, renumbered from 033 in a dual-allocation; the clm column + law goldens; the 024 leak columns rode the publish). README CUDA-row refresh 14–17× (same-binary env-flip). Bench 030 float4 sgemm (every suite −6.8..−16.7% p50; CUDA graphs closed NEGATIVE in riir-infer `.issues/005`). Bench 029 CUDA tile ladder (block-fit on SM count; `LAYA_CUDA_LADDER=0`; dynamic-smem launch defect fixed). Issue 019 T1+T2 (the CLM adapter + the byte-pinned `cca045ff` prose law, `scripts/clm_goldens.py`, 42 goldens; renumber 026→027 note). Issue 028 — the 026 bench CORRECTION: the CUDA packed-path ran dead attention on every multi-question case (typed 0.7445→0.2690); fixed by the substrate flash kernel; `laya_batch_parity` joined the cuda gate set (riir-infer `.issues/003` `75ed138`; site correction reflex-site `faf9b50` — the corrupted numbers never went live). Created retroactively 2026-09-22: Issues 001, 002, 003, 005.

## 2026-09-24

Issue 026 CLOSED — `laya-riir-cuda` green (backend riir-infer `.issues/002`, records `e99d767`/`1efc8b7`; cudarc 0.19 + nvrtc sm_89, the Metal architecture ported; `CudaSlice::clone()` is a dtod COPY, caches hold `Arc`); the 4090 row cpu→cuda (⚠ corrected 09-25, Issue 028: the multi-question suites ran dead attention; the 1-question suites were correct). Issue 025 — the `laya (python)` lane back on the bench (`.benchmarks/025_m3_laya_python/`, python accuracy == rust on every suite; publisher fix reflex-site `20ba115`). Issue 021 CLOSED — the power axis nothing was recording: `scripts/bench_preflight.sh` (battery/Low-Power/settle/load refusals + `PROVENANCE:` line; ⛔ `pmset powermode` is a THREE-state enum — only `1` refuses, fixed `b819718`; the canary is best-of-5 minima pinned `CANARY_REF_US=141`; `src/harness/box_state.rs` stamps power/powermode/load/swap + `latency_quotable` into results.json; AC re-bench = Bench 006 Addendum 2 `098b399`). Issue 022 CLOSED — the BOUNDARY row names the CRATE `riir-infer-laya` (one-cell fix). CLM distilled — Research 001 filed + Issue 019 (the first open contrastive System One in the Jev lineage, katgpt-rs Research 562→573→576; pin `cca045ff`, Apache-2.0; divergences held: sigmoid-then-L1 + first-class abstain stay; the dual-allocation 018→019 lesson: `ls .issues/` + fetch + `git log HEAD..origin` is the real pre-commit check). Issue 014 — engine sigmoid delegates to `katgpt_core::exact_sigmoid` (pin: max 3 ULPs at x=−16.68). Wide-BK=48 measured NEGATIVE + the `sgemm_shape_timing` probe landed (three birth traps: the as_micros/1000 unit bug, the begin_pass-less stale-slot aliasing, the pipelined-block posture). v0.2.3 release cut (`a386119`; ⛔ the wrong-repo finding: `gh release create` must target the dist repo **gist-rs/reflex**, never this source repo; ⚠ the `--clobber`-with-renamed-file trap: delete-asset + re-upload + verify). Issue 011 CLOSED — lanes + flappy heads join the serve lane (`d1eda08`: lanes digest `7d3f1d8e…09d34` @ λ 0.01, flappy `c93d36dc…e3c5` @ λ 1; the wire discovery: `noul` questions legally carry no options — the sequence rides `state` one per line; `/healthz` advertises all three heads). Issue 014 — `X-Reflex-Lane: raw` (the raw abstain baseline board). Issue 015 — parallel-Metal smoke RESOLVED (`a3215da`: the smoke violated the chain-cache epoch contract — no `begin_pass`, recycled addresses served stale DST slots; a heap-layout lottery, never contention; fix = `begin_pass` per arm boundary, 30/30 + 15/15 green; the containment rule RETIRED). Issue 018 CLOSED — the 4090-windows bench lane live end-to-end (run @`afacc3a`, publish, deploy, live-verified `e5055982`).

## 2026-09-23

crates.io publication DEFERRED (owner-gates menu v2 row 2) — no `cargo publish` until an external adopter asks; `katgpt-core` ships to crates.io so a future publish needs no sibling-strip. Issue 012 CLOSED — the laya-python reference lane (`scripts/laya_python_lane.py`, harness `--laya-python`: parity outside the fixture corpus; torch-MPS row batching ~2.7× at p50; "no Python anywhere" governs the shipped binary, not the bench reference). Plan 001 — the fitted game head served + Metal-default laya + release v0.2.2 (`367766c`+`932a2a3`, tag `v0.2.2`; head digest `00aa6221…c6e`; installers teach the `reflex` rename with a pre-v0.2.2 pin fallback). Issue 004 CLOSED — the Jev harness decision-point map (T3 the cache-reuse noul lane: laya·english 0.5000 at chance, n=12, a constant-confident one-class predictor; laya columns pinned PRE-MOVE at sha `2aa2dda`). Issue 006 CLOSED — candle removed entirely (T3 tokenizers v1 DEFERRED on a measured negative; **the reopen trigger lives in this row: tokenizers 1.0.0 STABLE, then G5 at both postures**; release v0.2.0 at `c25e1c3`, 5-target matrix, the strip-doesn't-fully-apply-cross-arch catch, tap `c30f0d6` `on_linux` fix, scoop `07cb8b6`). The T2 disclosure live on the site + tables (reflex-site `80d4fb1`; full both-lane regeneration at `0dc2397`, M3 LOADED box state disclosed). Issue 010 CLOSED — the agent-skill section (`.docs/04_agent_skill/SKILL.md` source of truth, wire examples captured against the real binary; the assets-root `.git/HEAD` exposure closed via `.assetsignore`, rider `47247a8`). Issue 005 CLOSED — the riir Metal backend (`laya-riir-metal`, 14 MSL kernels, G5 green; naive three-way 79.0/78.7/38.5). Issues 002+003 CLOSED — the kernel deps `gemm` 0.18 / `libm` 0.2 (version-matched to candle; the `libm` pin SURVIVES candle on numerics grounds). Issue 001 CLOSED — undeclared crates.io deps, BOUNDARY rows landed. Issue 009 CLOSED — the threshold-recommendation surface (the jimothy PATTERN: fit-time recommendation, null on thin support; byte-identical harness migration). Issue 012 remains + the G1 no-claim fix (`g1_verdict_of` `G1Verdict::{Pass,Fail,NoClaim}`, record Bench 001 Addendum 7). Issue 013 — modelless accuracy levers measured (lever 1 cap = per-suite knob, banking77 128 refused by the stratified protocol; lever 2 route_scale flat; lever 3 pair-head REFUTED; ⛔ CORRECTED by Issue 023: the "ceiling" was read with the centroid signal OFF and an index-misaligned cal slice). Issue 052 — typed_decisions TRAIN pull un-capped 800→1200 (`0e8896d`, Bench 078): 400 rows were never fetched; modelless 0.4655 → **0.5725**; the test split untouched (byte-verified); the cal-front premise corrected (it is the STRATIFIED round-robin front since Issue 039 T2).

## Lessons

- Box state is part of every latency claim: power source, power mode (`pmset powermode` is a 3-state enum — only 1 refuses), load, settle-since-plug-in; `scripts/bench_preflight.sh` + `box_state::capture()` stamp it, never assume it.
- Release objects go on the DIST repo `gist-rs/reflex`, never this source repo; `gh release upload --clobber` with a renamed file creates a second asset — delete, re-upload, verify.
- 4090 comparison-lane windows: serialized passes only (never beside laya CUDA or the perf league), `--gpu-memory-utilization 0.72`, one fixed warmup request first, same-box latency law.
- Numbering collisions are the recurring class (018→019, 026→027, 033→034, 053, 061→062, 070→071): `ls .issues/` + fetch + `git log HEAD..origin/develop --name-only` before allocating; numbers never reused, renumber + citation rewrite.
- Metal chain-cache contract: ops need `begin_pass` per boundary — `(ptr, len, epoch)` keys silently hit stale slots at recycled addresses (a red after gpu_lock = caller epoch violation, not contention); the weights cache deliberately has NO epoch defense.
- A Metal staging stride must EXCEED the staged tile's row width (overlap corrupts silently); compare only within position-balanced swapped pairs; a vendor library (Apple MPS) is a Metal-stack change — price it before any kernel rewrite.
- The fused GLU output must be written to its own contiguous buffer (`[rows, 2I]`); bit-parity ports must mirror the SIMD-strided reduction (`candle_vec_sum`) and use `libm::erff` — approximations drift 1e-2-class through 22–28 layers.
- Selection law: the STRATIFIED selection slice, never the raw label-clustered cal; cal never enters a fit, the test split is read once; an accuracy promotion that breaks the G1 floor is a failed gate (Bench 064's massive, the xnli blend).
- noul questions never take route terms (k==N==2 anti-alignment) nor fitted heads; `noul` wire questions carry no options — sequences ride `state` one line each.
- `code_fixtures` is commit-relative (its cases are this repo's own source spans) — population-excluded from the cross-host drift gate, frozen `code_fixtures_frozen.json` + BLAKE3 pin.
- Author-synthetic fixtures leak by construction (the four-time corpus-overlap recurrence): `fixture_hygiene.rs` is the one shared checker; at-chance wide-eval reads killed the six families.
- Python subprocess lanes on a cp874 box: reconfigure stdin AND stdout/stderr to UTF-8 (`backslashreplace`); uv-venv trampolines deadlock on close-then-wait (Drop kills-then-waits); sccache breaks llama-cpp-python builds — mask it from PATH.
- Site publishes are lane-scoped (`PUBLISH_BENCH_LANES`) — uncalibrated by-product controls must never overwrite published postures; a fresh-docs publish wipes undeclared lanes (`guard_wholesale_replace`), the update path does not.
- The publisher carries staleness keyed on own-timing incumbents (LANE_CARRY) — a carried incumbent's equal digest stamps accuracy, never timing.
- CUDA: `CudaSlice::clone()` is a dtod COPY (caches hold `Arc`); dynamic smem died at the 48 KB default (footprints are compile-time bounds); block-fit on SM count, not the Metal m-floor; CUDA graphs closed NEGATIVE (`gpu == wall`).
- `laya_batch_parity` (the multi-question gate) must run at EVERY device posture — single-question G5 passed while packed attention ran on zeros.
- HF fetches ride 429 walls: cooldown + resume, the `SUITES` filter avoids pointless pulls; dataset pages byte-verified with `cmp`/SHA256 before any re-measure.
- The `.benchmarks/.highwater` is a single-line counter — read it with `head -n 1` (a `tr -d` reader concatenated "109144").
- `REFLEX_BENCH_HOST` is REQUIRED on the 4090 (unset label refuses at row birth); a lane nobody owns renders as `not run` on the board — a documented default posture, not a defect.
- Density/synthesis gates: echo gates (abstention-entropy KL + OOD word-dropout) are mandatory on every `--corpus-ab` pass; a clean V5 PASS with a negative gate-rung reads ECHO = the lane dies.
