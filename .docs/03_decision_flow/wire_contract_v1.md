# The decision_wire contract — v1 (the R7 freeze)

**Status:** FROZEN 2026-10-08 — the v1 break is SPENT (reflex issue 074,
closed; the edge landing: riir-instinct `db7490e`). From here the contract
is **additive-only forever**: v2 appends optional fields, never a shape
change.

This is the decision_wire half of riir-refine Plan 202's R7 v1-breaking
ledger (the manifest grammar half lives in the refine repo). The wire the
ledger freezes is the hosted serving edge's `POST /decide` —
`../riir-instinct/src/serve_edge.rs` — the plane the first customer
(refine's seat-rerank escalation) rides and the Rethink lane re-shares over
the extension seam. The repo-law routing: the decision_wire vocabulary is
THIS repo's (born Plan 603; the instinct edge borrows the serve SHAPE, not
the module), so the contract record lives here beside
`decision_flow.md`.

## T1 — the version law (`contract_version`)

- The REQUEST carries `contract_version` — an integer, `1` at this break.
- **Absent = 1**: every client predating the field is a v1 client by
  construction (back-compat, pinned by a gate).
- **Unknown version → `400 unsupported_contract_version`, fail-closed**,
  naming the supported set — never a guessed dialect. The refusal precedes
  the suite lookup, so it answers even where the suite does not exist (the
  gate asserts 400, not 404, on suite `nope`).
- A wrong-TYPED version (a string, a float) is the same loud refusal, not a
  parse error that names nothing.
- The RESPONSE echoes the served version on every answer: the single
  decision doc, the multi-question envelope, and each element of
  `decisions[]` — so a proxy/lane mismatch is visible at the client.

## T2 — the frozen response field set (additive-only)

Every 200 answer carries exactly these fields; **nothing removable; new
fields append**:

`contract_version` · `suite` · `arm` · `lane` · `options` · `pick` ·
`pick_index` · `probabilities` · `specialist_scores` · `confidence` ·
`escalated` · `abstained` · `us` · `lane_load` ·
`receipt{build, features, input_blake3, decision_blake3, lane}`

The multi-question form wraps the same per-decision objects (plus
`question_id`) in a top-level envelope (`suite`, `lane`, `contract_version`,
`n_decisions`, `decisions[]`).

En-route alignment (same break, same commit): the receipt keys were renamed
to the freeze spec's spellings — `input_blake3` / `decision_blake3` (were
`input` / `decision`). The refine client's `SeatReceipt` already parsed the
spec spellings and had been silently reading `None` since R3 (Bench 107
finding 4 named the fields by the spec, not the wire).

## T3 — the gates (a silent field removal reds)

- `riir-instinct tests/ext_seat_rerank_gates.rs` — the rerank route (the
  first customer's plane, runnable on any box with the seat datasets):
  echo + field-set MEMBERSHIP + receipt spellings; the unknown-version
  refusal (ungated — it precedes any lane work, so it runs on fresh
  clones, exactly the boxes most likely to carry a stale client); the
  absent-field back-compat arm; the multi-form echo.
- `riir-instinct tests/serve_gates.rs` — the version-refusal arms in the
  data-independent face, and the happy-path echo + membership + explicit
  `contract_version: 1` request (data-gated; the ag_news full envelope).

## The frozen laws (no wire surface — decided, recorded, binding)

1. **Consent stays local-only.** The code-egress scope
   (`RIIR_REFINE_CODE_EGRESS=snippet`, refine side) gates every socket
   pre-connect; no consent fields ride the wire. The endpoint's trust
   anchor is the transport + the receipt.
2. **The rerank min-pool gate is client-side.** Rerank arms fire at pool
   ≥ 2 only (refine's manifest grammar, `DEFAULT_MIN_POOL`); the wire never
   sees a 1-option rerank from the first customer. (The edge independently
   refuses `<2` presentations with `422 bad_field` — the malformed-body
   crash class the contract gates found; belt and suspenders, never a
   substitute for the client gate.)
3. **One `heal` suite for v1** (Plan 202's R7 freeze + Bench 107 addendum):
   one seat lane hosts the healing rerank expert; per-domain lanes wait for
   measured need. The suite is an opaque routing key on the wire — this
   decision needs no wire change either way.
4. **Rule-id context is an expert-QUALITY lever, not a contract need**
   (Bench 107 finding 7). It becomes an ADDITIVE optional field
   (`context_rule_id`) in v2 under `contract_version` — never a v1 break.
5. **Keep-alive / pooled connections are the v2 lever** (Bench 107 finding
   3: the ~0.8–6 ms loopback transport floor is v1-accepted).

## Where things live

| surface | home |
|---|---|
| The edge + `CONTRACT_VERSION` | `../riir-instinct/src/serve_edge.rs` (landed `db7490e`) |
| The rerank client (first customer) | `../riir-refine/src/seat_rerank.rs` (`SeatAnswer`/`SeatReceipt`, lenient-additive parse) |
| The manifest grammar half of R7 | `../riir-refine/.plans/202_decision_stack_arsenal_alignment.md` |
| The closed issue | reflex `.issues/074_decision_wire_contract_version_freeze.md` (removed at close-out; git history holds it) |
