# Issue 069 — Public artifact fetch lane

**Status:** OPEN — workspace artifact consolidation, public lane

## Why

The workspace is standardizing how published artifacts are distributed: public artifacts land in
the org's public HuggingFace datasets, and every repo can pull them with BLAKE3 verification —
the same protocol this repo already uses for datasets (`scripts/fetch_datasets.sh` + digest
manifest). This repo's game-head lane is already committed and re-derivable at boot, so the work
here is only the fetch lane for any future public-class assets.

## Scope

- [ ] `scripts/fetch_artifacts.sh`: pull public artifacts from the org HF dataset lane
      (`gist-rs/<repo>-artifacts`) into `artifacts/cache/` (gitignored by the directory law),
      verify BLAKE3 against the consuming manifest before use — never a silent unverified cache;
      the public manifest carries public rows only (workspace gate law)
- [ ] Wire the lane into the README build-surface docs (opt-in; committed fixtures remain the
      default path and are unchanged)
- [ ] If any public-class demo asset is ever minted for this repo: manifest rows + publish half

## Acceptance

- Fresh clone + pull → the lane boots with witness digests matching the committed fixtures
- Unverified/corrupt cache is refused loud (BLAKE3 mismatch = exit non-zero, never a fallback)

## Notes

Public repo: this issue covers the public distribution channel only. Private artifact classes are
governed in the private repos and are out of scope here.
