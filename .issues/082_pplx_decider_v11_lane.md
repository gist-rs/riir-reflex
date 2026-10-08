# Issue 082 — pplx-decider v1.1 (27B) lane: measure the JDI #1 on OUR suites (the Plan-011 clef template), teacher candidacy rides the read

**Status:** OPEN — filed 2026-10-08. The JDI board moved under our pinned reference (release-v2.1, 2026-09-28: Jev #1, pplx v1 #2): **Perplexity Decider v1.1 (27B) published 2026-10-05, #1 on the 2026-10-07 index** (balanced_raw 71.66 / balanced_skill 62.75 / median 104.1 ms — Jev 55.64 at #13, clef 53.08 at #18, clef-flash 47.61 at #30). License VERIFIED at filing: **Apache-2.0, NOT gated** (HF api `perplexity-ai/pplx-decider-v1.1-27b`: 11 safetensors shards, LICENSE + NOTICE present, `license:apache-2.0` tag, 2026-10-05). This issue measures it on our frozen splits before any "use it" claim — the B5 law cuts both ways.

## Why now (and why we didn't already)

- Our crosswalk's `board_reference` pins release-v2.1 (7 rows, 2026-09-28) — v1.1 did not exist then. **No standing watch re-checked the board since.** The refresh of the pinned snapshot is T4 here; the standing re-check trigger is T5 (the perf-rematch `watch_repo` pattern, applied to the JDI space's `data/index.json`).
- JDI rank ≠ our-suite rank, in BOTH directions: clef is #18-by-skill on JDI yet holds our banking77/massive/sst5 bars; pplx v1.1 is unmeasured on ours. The lane decides.

## The JDI category slice that matters to us

Our three trailing suites are language-class. On JDI `language`: **pplx v1.1 0.7647 raw / 0.6748 skill** > clef-27B 0.7241/0.6127 ≫ clef-flash-9B 0.6174/0.4722 (the lane we measured locally). pplx's #1 is built on language + arts + breadth; **clef-27B still wins retrieval (0.741 vs 0.720) and tools (0.835 vs 0.817)** — the #1 is not uniformly better, which is exactly why the per-suite lane read, not the board rank, is the decision surface.

## Tasks

- [ ] T0 — serve feasibility: the repo is `custom-code` (Qwen3.8-27B base + `decision_config.json`; transformers/vLLM path, `trust_remote_code`; NOT a llama.cpp arch). 4-bit ≈ 15 GB → the 4090 (24 GB) or m3; bf16 (11 shards ≈ 55 GB) is m3-only if at all. Record the posture actually served (local, our splits — the hosted posture is owner-gated and never pooled, the clef-law).
- [ ] T1 — the lane: `src/lanes/pplx.rs` (the `clef.rs`/`agentjev.rs` sibling; http-oracle over a loopback vLLM/transformers serve) + `SuiteResult.pplx` + wire tests (Plan 011 A1–A5 shape).
- [ ] T2 — run the 9 dataset suites at the published posture, frozen splits, digest-pinned; acc + ECE-only tail (the clef metric law; no JDI-skill claim on our side).
- [ ] T3 — publish: publish_bench lane rows + the verdict block picks it up automatically (instinct.js now counts every published lane — reflex-site `06e50b7`). No site code change expected.
- [ ] T4 — refresh the crosswalk `board_reference` snapshot to the 2026-10-07 index edition (re-pin `.research/005`; keep the REFERENCE-ONLY caveat verbatim) so the site stops quoting the pre-v1.1 board.
- [ ] T5 — the standing watch: a cheap re-check of the JDI space `data/index.json` sha (the `watch_repo` pattern) so the next board move files itself instead of waiting for a user report.

## Teacher candidacy (the second consumer of the same read)

riir-train Issue 623 (clef-flash teacher lane) gains a second candidate from this lane: if pplx v1.1 holds ≥ clef-flash on our language-class suites, it is the stronger open teacher (Apache-2.0, +6pt JDI-language over clef-27B). The lane read decides the teacher; 623's gates stay the arbiter. DO NOT start a teacher capture before T2's read exists.

## References

- Live board: `multimodalart/jev-decision-index` `data/index.json` (generated 2026-10-07T18:25:58Z; 114 entrants) — fetched + parsed at filing; snapshot `/tmp/jdi_index.json` is session-local, re-derive from the URL.
- `.research/005` (the JDI/Clef distillation record; pins release-v2.1), Plan 011 (CLOSED — the lane template), reflex-site `06e50b7` (verdict counts every lane).
- riir-train Issue 623 (teacher lane), 608/609 (bekko precedent), 599 (openthai teacher).
