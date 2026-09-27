# Issue 051 — the `reflexer-vessel` dep has no BOUNDARY allowlist row

**Status:** OPEN 2026-09-27 — fixable (missing declaration; the dep itself is sanctioned)

## The finding

`Cargo.toml` carries the optional path dep
`reflexer-vessel = { path = "../riir-reflexer/crates/reflexer-vessel", default-features = false, optional = true }`
(landed with the v0.2.x game-head vessel work — `vessel_public_read` /
`vessel_hosted_read` features forward the reader-capability axis), but
`BOUNDARY.md`'s "May depend on" table has NO row for it. The boundary
contract's own rule: an edge without a row is a contract violation (or
contract rot) regardless of intent — `../riir-ai/scripts/ci_boundary_contract.sh`
reads the allowlist, not the intentions.

## Why the dep is sanctioned (the row's content)

`reflexer-vessel` is the PUBLIC vessel format crate (MIT, in
`../riir-reflexer` — the rulebook engine + format repo, itself public):
the blake3 + ed25519-dalek-only reader half. reflex consumes the
`vessel_public_read` capability (the game heads serve from PUBLIC-RELEASE
vessels; Plan 001's format lane). All-public chain: reflex (public) →
reflexer-vessel (public) → katgpt-core (public); zero private surface, zero
back-edge. The layering holds the same way the `riir-infer-laya` row does.

## The fix (same commit as this filing)

Add the allowlist row: opt-in `vessel_public_read` /
`vessel_hosted_read` — the PUBLIC vessel FORMAT reader
(blake3 + ed25519-dalek only); the HOSTED-ONLY reader capability exists
at the format crate but reflex selects the public one; the game heads'
artifact consumption (Plan 001/002). One-way edge into the public format
crate — never a dep on riir-train (the HOSTED minter) or any private
sibling.

## Drift ledger

Row disposition: this was `fixable` (missing declaration) — filed + fixed in
the same commit; the row moves to the allowlist proper, this issue closes
with it (row ⟺ open issue; closing removes the ledger row, the ALLOWLIST row
stays — it is the contract, not the drift).
