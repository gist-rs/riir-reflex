# Issue 103 — serve slow-drip lane-hold: per-read timeout is not a request deadline

**Status:** OPEN — robustness defect, verified 2026-10-30 (distill session zed-m3-impeccable-distill / riir-refine Research 356; independently confirmed by the verdict reviewer)

## Finding

`src/serve.rs:45,1197` sets ONLY a per-read timeout before reading the request:

```rust
const READ_TIMEOUT: Duration = Duration::from_secs(10);
...
stream.set_read_timeout(Some(READ_TIMEOUT))?;
```

`SO_RCVTIMEO` bounds a **single blocked `read`**, and **any byte received resets it**. A slow-drip peer (1 byte per 9 s) keeps each read returning `Ok(1)` forever — the request read never returns, the connection slot is pinned indefinitely. No total request deadline, no watchdog, no `shutdown(Both)`.

Severity is bounded (the serve edge is localhost; the arena/site lanes are loopback-facing), but the lane-hold is real: one hung or malicious local client holds a connection slot forever.

Related site to check at fix time: `src/lanes/http_mini.rs:96` also sets a bare read timeout (context not verified — client-side vs server-side read unknown).

## Reference fix shape (external, measured)

`pbakaus/impeccable` @ `02523aec` `crates/common/src/http.rs:461–531` fixed exactly this class: a **total wall-clock deadline across the whole request read** enforced by a 50 ms-polling watchdog thread that calls `shutdown(Both)` when it elapses, then restores the prior per-read timeout (kept as a coarse backstop).

Platform caveat carried in their fix (measured, triage D3 BLOCKER): **on Windows, a `recv` already parked in the kernel is NOT unblocked by another thread's `shutdown`** — the per-read timeout must stay as the second enforcement axis there; the watchdog alone cannot unblock a parked read. The deadline must be enforced on BOTH axes per-platform.

## Fix sketch

1. Wrap the request read in a total-deadline form: deadline (10 s is generous), watchdog thread + `done` AtomicBool, `shutdown(Both)` on expiry, restore prior timeout after.
2. The per-read `READ_TIMEOUT` stays as the Windows backstop + coarse limit.

## Refs

- riir-refine `.research/356_impeccable_design_detector_distill.md` (the distill verdict; the corpus rule candidate `per-read-timeout-vs-request-deadline` will encode the general law)
- riir-instinct issue 024 (the same defect in the instinct serve edge)
