# Issue 069 — Public artifact fetch lane

**Status:** LANDED 2026-10-06 (Plan 623 T6) — the fetch lane + README
surface; the conditional third row (publish half) stays closed until a
public-class asset is actually minted for this repo

## Why

The workspace is standardizing how published artifacts are distributed: public artifacts land in
the org's public HuggingFace datasets, and every repo can pull them with BLAKE3 verification —
the same protocol this repo already uses for datasets (`scripts/fetch_datasets.sh` + digest
manifest). This repo's game-head lane is already committed and re-derivable at boot, so the work
here is only the fetch lane for any future public-class assets.

## Scope

- [x] `scripts/fetch_artifacts.sh`: pull public artifacts from the org HF dataset lane
      (`gist-rs/<repo>-artifacts`) into `artifacts/cache/` (gitignored by the directory law),
      verify BLAKE3 against the consuming manifest before use — never a silent unverified cache;
      the public manifest carries public rows only (workspace gate law)
- [x] Wire the lane into the README build-surface docs (opt-in; committed fixtures remain the
      default path and are unchanged)
- [ ] If any public-class demo asset is ever minted for this repo: manifest rows + publish half


## Landed (2026-10-06)

- `scripts/fetch_artifacts.sh` — self-contained (no sibling checkouts; a
  public fresh clone is the acceptance environment): shell + python3
  (tomllib manifest parse, unknown-table refusal) + b3sum + the `hf` CLI.
  BLAKE3 + exact-size verified against `blake3_plain`/`plain_bytes`
  BEFORE use; a fresh download verifies at `<name>.new` and promotes into
  the cache only on a match (never a silent unverified cache). A
  protected row in this manifest is refused loud (the workspace gate's
  public-only law, re-checked at fetch time). Postures: fetch (default) +
  `CHECK=1` (verify cached bytes only, no network). Exits: 0 clean (the
  honest no-manifest statement included — this repo's fixtures are
  committed, the lane exists for future public-class assets) · 1 findings
  · 2 REFUSED. Six postures proven on a synthetic tree (honest pass,
  corrupt cache red, size-mismatch red, protected refusal, parse refusal,
  no-manifest statement); the live hf-download branch is
  construction-verified only until a dataset is actually published.

## Acceptance

- Fresh clone + pull → the lane boots with witness digests matching the committed fixtures
- Unverified/corrupt cache is refused loud (BLAKE3 mismatch = exit non-zero, never a fallback)

## Notes

Public repo: this issue covers the public distribution channel only. Private artifact classes are
governed in the private repos and are out of scope here.
