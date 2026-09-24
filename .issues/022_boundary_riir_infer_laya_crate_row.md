# Issue 022 — boundary: the `riir-infer` allowlist row names the REPO, the gate matches the CRATE

**Status:** OPEN — filed 2026-09-24 by the boundary-guard run (katgpt-rs
`.agents/skills/boundary-guard`, 145th run). Disposition `fixable`; carried by
drift row **D1** in `BOUNDARY.md` until closed.

## Finding

`riir-ai/scripts/ci_boundary_contract.sh` (workspace form) exits **1** with one
C3 violation, introduced by `04531ae` (Issue 008 T4, 2026-09-24 10:55 +07 —
after the 144th run, which read exit 0 at 07:57):

```
✗ VIOLATION  riir-reflex → riir-infer-laya is NOT in riir-reflex/BOUNDARY.md 'May depend on' [normal:req:dfoff@riir-reflex/Cargo.toml ]
=== Summary ===
1 violation(s), 0 contract-rot finding(s).
```

The dep is **intended and declared** — `BOUNDARY.md` § May depend on carries the
row-before-edge entry (`LANDED`, Issue 008 T4). But that table's first column
is **Crate**, and the row writes the repo name:

```
| riir-infer | `../riir-infer` (`crates/riir-infer-laya`) | **LANDED** ...
```

C3 matches the measured dep crate name EXACTLY against the Crate column
(`grep -qx "$dep"`), so `riir-infer-laya` is not found. Contract-shape error,
not an undeclared edge — the layering (reflex → riir-infer → katgpt-core, no
riir-ai) is as the row says.

## Fix (one cell)

- [ ] Rename the Crate cell `riir-infer` → `riir-infer-laya` (Location already
      names `../riir-infer` + `crates/riir-infer-laya`).
- [ ] Re-run `../riir-ai/scripts/ci_boundary_contract.sh` → exit 0, C3 green.
- [ ] Close: remove drift row D1 in the SAME commit that removes this file.

Left to the owning riir-reflex session (live on Issue 008/020 lanes at filing
time) per the boundary-guard issue-before-fix discipline.
