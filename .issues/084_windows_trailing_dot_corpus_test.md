# Issue 084: corpus dot-name test red on every Windows box (trailing-dot dirs unrepresentable on NTFS)

**Status:** FIXED in-flight — `directory_names_load_verbatim_including_dots` now carries
a `#[cfg_attr(windows, ignore = …)]` with the platform reason (a VISIBLE skip, never a
silent `#[cfg]` compile-to-nothing), and a cross-platform companion
`directory_names_load_verbatim_interior_dots` keeps the loader-doesn't-trim class
covered on Windows.

## Finding

`corpus::tests::directory_names_load_verbatim_including_dots` (Issue 081 T2c) creates
corpus domain dirs named `No further round helps.` / `Another round would help.` —
trailing-dot names — and asserts they load byte-intact. On Windows/NTFS, Win32 path
normalization strips trailing dots (and spaces) from the LAST path component at
creation: `create_dir_all("…helps.")` silently creates `…helps`, the loader then reads
the stripped name, and the byte-intact assertion fails. Reproduced on this box
(4090-windows, 2026-10-10):

```
assertion failed: loaded.domains.contains(&"No further round helps.".to_string())
```

The loader is innocent — on Windows a trailing-dot domain dir cannot exist via any
normal filesystem API, so the production surface (by-name option routing `==` against
the domain name, Issue 081 T2c) is unthreatened there. The property must keep its
guard on the deployment platforms (macOS/Linux hosts, CI ubuntu) where such dirs DO
exist.

## Fix

1. `#[cfg_attr(windows, ignore = "…")]` on the original test — the skip prints in
   test output with its reason; the fixture and assertion stay compiled everywhere.
2. New `directory_names_load_verbatim_interior_dots` (no cfg, runs on every platform):
   dots in NON-terminal positions (`state.depth.0`, `v1.2 release notes`) — the
   byte-intact property stays exercised on Windows, and a loader that trims at the
   first dot or normalizes names still fails it.

## Verification

`cargo test --lib directory_names` on Windows: interior-dot test passes; trailing-dot
test reports `ignored` with the reason. macOS/Linux: both run green (verify on the M3
at next touch — the cfg_attr is no-op there).
