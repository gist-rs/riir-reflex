# Issue 113 — the JDI board moved (2026-11-06 edition): watch CHANGED, REAX 1.3 new blended-headline #1, teacher pointer unchanged

**Status:** OPEN → RESOLVED in one sitting — the watch signal is read, filed, and re-pinned (the lesson this watch was built for: pplx-decider 2026-10-05 took #1 and nothing noticed for two days).

## The signal

`python3 scripts/watch_jdi.py` (2026-11-07, m3): **CHANGED** (exit 1)

```
old sha bdfdd5ff3e7700f6 ... 2026-10-29T14:04:16Z / decision-index-0.3   (pin fetched 2026-11-02)
new sha 66d257218ef2f6d4 ...
generated_utc 2026-11-06T09:05:08Z edition decision-index-0.3
  new #1: REAX 1.3 (balanced_skill 73.98, rank_v 1)
  tracked: JEV-27B 57.35 (r77) | clef 54.92 (r90) | clef-flash 44.43 (r131) | GLiNER2.5-Decide 10.14 (r260)
           | Laya 9.21 (r261) | CLM-v0.1-8B 6.49 (r267) | clef-omni 45.04 (r129)
           | Perplexity Decider v1.2 67.98 (r17) | PD Flash v1.2 54.29 (r93) | AutoTrust-JEV-27B v1.1 67.95 (r18)
           | WeMM-JEV-27B 68.49 (r12) | Torchcast Decision 27B 61.49 (r49)
```

The move happened between the 2026-10-29 board and the 2026-11-06 board; the pin was four days stale (fetched 11-02, no watch run since). Suite label is now **Decision Index 0.4.1** (edition release-v4) with panel_id still decision-index-0.3 — cite both.

## The read (one aggregate discipline — never fold)

- **Blended headline (`balanced_skill`, the board's main ranking): REAX 1.3 is #1 (73.98)** — mammoth-1 sits at 68.44, not top-8 blended.
- **`public_skill` (the law-9 GOAT aggregate): mammoth-1 still #1 (75.34)**, REAX #2 (73.71). mammoth-1 keeps BOTH law-9 seats (board #1 + highest open-weights on the public rank).
- Full distill + the teacher-pointer verdict: **rethink Research 019** (`../riir-rethink/.research/019_reax13_board_move_teacher_verdict.md`, sha `66d25721…` pinned there). Verdict: teacher UNCHANGED (mammoth-1); REAX impractical on the fleet (57.4B dual-part FP8, CUDA 13/SM120); mammoth licence discrepancy flagged (board "proprietary" vs HF tag `license:mit` vs no LICENSE file — redistribution-bearing use blocked pending a per-use read).
- Downstream lane consequence: train 668 closed DONE-NEGATIVE the same morning → massive/banking77 gap work routes to the ESC/serve-posture lane (rethink Issue 052, bmo queue staged).

## Disposition

- [x] File the signal (this issue) — the watch's own instruction ("file the lane/refresh issue; re-pin only via --update after the read").
- [x] The read: rethink Research 019 (born-complete, edition + sha pinned).
- [x] Re-pin via `watch_jdi.py --update` (the only writer) — commit lands with this issue.
- [x] The tracked-entrants table recorded above verbatim from the watch output (the rows whose movement matters to our lanes; none of ours moved seat — JEV-27B r77, clef r90/r131/r129, GLiNER r260, Laya r261, CLM r267, Perplexity r17/r93).
- [x] Infra note (verdict round 1 minor): 2026-11-07 is the SECOND consecutive bmo session needing the amo thunderbolt jump — `katopz@100.100.117.44` (Tailscale) timed out both days while the box itself answers on `192.168.10.2` via the bridge (tailscaled down, box up). Not silent-normal: if a third lane needs the jump, check bmo's tailscaled before queueing anything latency-sensitive.

Standing watch discipline (unchanged): a CHANGED exit is a file-an-issue signal, never an auto-repin; `--update` is the only writer; offline boxes skip loud (exit 2), never a CI hazard.
