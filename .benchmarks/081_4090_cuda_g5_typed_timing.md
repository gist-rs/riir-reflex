# Bench 081 — 4090 CUDA laya lane: full consumer-side G5 GREEN + the first CUDA timing triple (weights synced over scp)

**Verdict: PASS — the deferred lane is unblocked.** The laya weights
(2.2 GB, three checkpoints) did not exist on the 4090; they were synced
from the M3 cache (`~/.cache/riir-reflex/laya` →
`C:\Users\katopz\.cache\riir-reflex\laya`) as one tar over Tailscale
scp and verified **15/15 files byte-identical** (per-file SHA-256 diffed
both sides, CRLF-normalized; AppleDouble `._*` junk stripped). With
weights present, the full consumer-side G5 parity gate ran at the CUDA
posture and passed, and the fixture-timing example produced the lane's
first CUDA numbers.

All runs at reflex **`0c9fcab`** (both hosts' HEAD; riir-infer on the
box at its local `46b8deb` — the state Bench 080 validated the CUDA op
lane at).

## G5 parity (the gate that must pass before any published number)

`LAYA_DEVICE=cuda cargo test --release --features laya-riir-cuda --test laya_riir_parity`

**ok. 2 passed; 0 failed — 8.20 s** (top-1 ≥ 99.9% + prob drift ≤ 1e-3
per checkpoint, all three checkpoints, vs the frozen captures). The
first consumer-side CUDA G5 on record.

## Timing triple (`examples/laya_fixture_timing`, rows=125/125/175, reps=5)

| checkpoint | row p50 | row p90 | ms/question |
|---|---|---|---|
| typed | **13.8 ms** | 19.7 ms | 20.2 |
| english | **13.3 ms** | 18.3 ms | 16.5 |
| multilingual | **6.8 ms** | 9.3 ms | 9.2 |

(A first pass at `98d06df` read 14.1 / 14.0 / 7.2 — run-to-run noise,
no ordering change.)

## Cross-host context — FROZEN citations, not a fresh pair

Against the recorded M3 rows (`.benchmarks/001` addendum 6,
interleaved same-session, same fixture corpus + example): riir Metal
28.3/28.3/12.2 row p50 · torch MPS 25.7/25.5/16.2. The CUDA triple is
≈**2× faster than M3 Metal** on typed/english and ≈1.8× on multilingual.

⛔ This is NOT an interleaved cross-host pair: the M3 preflight REFUSED
today (load 6.38 > MAX_LOAD 6.0, 1.7 GB swap — a sibling session's
benches were running), so no fresh M3 Metal number exists at this
commit. The comparison cites frozen historical rows and their recorded
posture. A fresh interleaved pair at `0c9fcab` is the follow-up when
the M3 is quiet.

## Box state (both sides, per the G2 discipline)

- **4090**: RTX 4090 24 GB, driver 610.62. During the runs: desktop-only
  compute tenants (dwm/explorer/EdgeWebView — no CUDA compute), 21% util
  baseline / 721 MiB resident; post-run 51°C, 2535 MHz (boost active, no
  throttle). Mains power.
- **M3**: preflight `REFUSE — load average 6.38 exceeds MAX_LOAD=6.0`,
  swap 1698M, powermode=2(high), AC — hence no M3 reading in this record.

## Transfer provenance

- Path: M3 → 4090 over Tailscale (`100.85.179.44`, direct path, no
  relay): **2.37 GB tar in 65 s (~36 MB/s)**. LAN (`192.168.1.33`) was
  refused this session — Tailscale is the working path.
- scp push M3→4090 works (the pull direction failed on a
  remote-file-not-found, which had been mistaken for a protocol
  failure). bsdtar on the box; extraction needed a re-run after a local
  quoting error — the first attempt never reached ssh.
- The box's `git fetch origin` egress stall (Bench 080 note) does not
  affect scp-to-C: transfers; both paths coexist (git sync via M3-side
  push, bulk data via scp).
