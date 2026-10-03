# Issue 068 — Lane HTTP micro-clients: 4 diverged copies of `parse_http_response` (+ per-lane `request()`)

**Status:** OPEN (filed 2026-10-04 by the substrate-first mode-2 drift audit; LOW priority, deliberately NOT a drive-by refactor — the lanes are frozen measurement surfaces with pinned wires)

## Finding

The comparison lanes each carry a hand-rolled std-only HTTP micro-client. The
`parse_http_response` parser exists **4×** with per-lane divergence (all four
hash-different — accumulated tweaks, not copy-paste):

| lane | `parse_http_response` | `request()` | notes |
|---|---|---|---|
| `src/lanes/openthai.rs` | ✓ (`:220`) | ✓ | `Result<_, String>` |
| `src/lanes/agentjev.rs` | ✓ (`:224`) | ✓ | `Result<_, String>` |
| `src/lanes/clef.rs` | ✓ (`:286`) | ✓ | longer variant (33 lines vs 24) |
| `src/lanes/clm.rs` | ✓ (`:580`) | — | own error enum `ClmError` |
| `src/lanes/paw.rs` | — | ✓ ×2 | hosted + local runtime paths |

This is the workspace's recurring "the rule landed in one instrument and never
generalised" shape — each lane PR pinned its own wire first and the extraction
never happened.

## Classification (substrate-first mode 2)

**DRY violation, low severity, deliberate-variance flavor.** Not a parallel
*system* (no state, no policy — a ~24-line parser + a ~30-line TCP sender), but
four diverged copies of the same two functions is real drift surface: a parser
fix (e.g. a chunked-encoding or header-case quirk) must be applied four times,
and the clef variant has already grown a third again in size.

## Proposed repair (when a lane next changes — never mid-season)

Extract `src/lanes/http_mini.rs`:

- `pub(crate) fn request(host, port, method, path, body, timeout) -> Result<HttpReply, LaneHttpError>`
  with the per-lane knobs as parameters (timeout, extra headers);
- one `parse_http_response` with the clef variant's extra handling folded in;
- per-lane error conversion at the call site (clm keeps its `ClmError` via
  `From`).

Constraint (unchanged): **std-only** — the zero-dep edge is a product law
(AGENTS.md), and this extraction must not introduce an HTTP crate.

## Explicitly NOT now

- The lanes are measurement surfaces pinned to external services with frozen
  goldens (`clm_goldens.json`, the agentjev/openthai/clef wire pins); a
  cross-lane refactor mid-experiment season risks perturbing measurement code
  for zero measurement gain.
- Trigger: the NEXT lane addition or the next wire fix touching ≥2 lanes —
  extract then, with each lane's golden tests as the regression gate.

## References

- Substrate-first skill (mode 2) — the audit that filed this
- `src/lanes/mod.rs` — the lane registry
