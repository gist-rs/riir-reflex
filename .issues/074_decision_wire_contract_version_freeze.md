# Issue 074 — R7 ledger, decision_wire half: the v1 break = `contract_version` + the frozen field set

**Status:** OPEN — filed 2026-10-08 from riir-refine Plan 202 R7 after the R2+R3 dogfood
spike (riir-refine Bench 107: the first real cross-process refine-client → instinct-seat
traffic, 57 round trips, zero contract violations). This is the decision_wire HALF of the
R7 v1-breaking ledger — Plan 202's repo-law routing (decision_wire belongs to reflex;
refine's manifest grammar half lives in Plan 202's R7 row).

## The decision (what v1 spends the one-time break on)

The spike's headline: the rerank envelope `{suite, state, options}` carried EVERYTHING the
contract needs — no wire change was FORCED by traffic. The v1 break is therefore one
deliberate field, spent now while only we + low-impact users ride the wire:

- [ ] **T1 — `contract_version` on the request and echoed on the response.** An integer,
  `1` at this break. Server: unknown version refuses `4xx` loud (fail-closed, naming the
  supported set); absent field = `1` (the current clients never send it — back-compat by
  construction). Response: echo the served version so a proxy/lane mismatch is visible at
  the client. Every FUTURE field is additive from here (v2 = new optional fields, never a
  shape change).
- [ ] **T2 — freeze the response field set as contract** (additive-only forever):
  `suite, arm, lane, options, pick, pick_index, probabilities, specialist_scores,
  confidence, escalated, abstained, us, lane_load, receipt{build, features, input_blake3,
  decision_blake3, …}`. Nothing removable; new fields append. The spike confirmed the
  seat already returns all of these and they are stable across the rerank route.
- [ ] **T3 — pin the frozen fields in a gate** (the serve-side byte pins exist per lane;
  add the `contract_version` echo + the field-membership set to the existing edge gates
  so a silent field removal reds).

## The frozen laws (no wire surface, recorded here so they stay decided)

1. **Consent stays local-only.** The code-egress scope (`RIIR_REFINE_CODE_EGRESS=snippet`
   on the refine side) gates every socket pre-connect; no consent fields ride the wire.
   The endpoint's trust anchor is the transport + the receipt.
2. **The rerank min-pool gate is client-side** (refine's manifest grammar, Plan 202 R7):
   rerank arms fire at pool ≥ 2 only — the wire never sees a 1-option rerank from our
   first customer.
3. **The healing suite vocabulary freezes with the manifest grammar** (one `heal` suite vs
   suite-per-domain) — the spike borrowed `sst5` (the R3-proven vehicle); a healing
   domain has no dataset row on the seat today. Decision lands in Plan 202's R7 freeze;
   the wire needs no change either way (suite is an opaque routing key).
4. **Rule-id context on the wire is an expert-QUALITY lever, not a contract need** — the
   spike's finding 7. It becomes an ADDITIVE optional field (`context_rule_id`) in v2
   under `contract_version`, never a v1 break.

## Measured context (Bench 107, refine side)

Cold start = one declined call + ~100 ms under budgeted retry (the lazy 503 window);
warm = 0.8–6 ms/decision loopback release; abstain first-class; 57 round trips, zero
contract violations. The client currently DROPS the seat receipt — refine's half of the
freeze surfaces a receipt tier on `SeatAnswer` (Plan 202 R7 finding 4).

## Close-out

Land T1–T3 (the edge + gates), record the freeze in `.docs/03_decision_flow/`, remove
this file, cite both commits.
