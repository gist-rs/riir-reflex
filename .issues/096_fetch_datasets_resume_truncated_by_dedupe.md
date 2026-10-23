# Issue 096 — fetch_datasets.sh: the resume path truncates at a deduped page (a `<100`-row mid-sequence page reads as the terminal page)
**Status:** OPEN — filed 2026-10-23 (found the honest way: the rethink Bench-071 provisioning hit it twice on this box; the sst5 t20k pull took three rounds to notice the wrong shape).
**Claimed:** zed-m3-idle-loop, 1792773914, expires 2026-10-25T09:25:14Z

## The defect

`scripts/fetch_datasets.sh`'s paging stop condition is "a page returns fewer than 100 rows" — the terminal-page signal. But the post-fetch DEDUPE step rewrites mid-sequence pages in place (dropping exact-duplicate rows: self + test overlap), so a 100-row page becomes a 99-row FILE. On any later resume run, the skip logic walks the existing pages, hits the 99-row file, reads "fewer than 100 rows" as the dataset's end, and STOPS — with `failed_requests=0`, no FAIL line, and a summary that looks like success:

```
  ==     sst5/train: 1199 rows in 12 file(s) (dataset total: 8544)
summary: files=0 rows=0 failed_requests=0        # ← voluntarily stopped at the deduped page
```

The suite's slice-integrity floor (the harness's MIN_POOL_ROWS) catches the shrunken pool at seat time — that gate is what surfaced this (rethink `serve_encoder_parity` refused at pool 2598 < floor 6800). But the FETCH script itself reports a clean resume, and a suite with no floor reader would truncate silently.

## Reproduction (this box, 2026-10-23)

1. `SUITES=sst5 OUT=.raw/datasets_t20k TRAIN_CAP=20000 fetch` → 429 wall mid-sequence → the end-of-run dedupe rewrites pages 011/013 to 99 rows.
2. Re-run the same command → the script skips 000–010, reads train-011 (99 rows), declares the dataset complete at 1199/8544 rows, exit-clean.

## The fix shape

Distinguish "the API returned fewer rows than requested" (terminal) from "the file holds fewer rows than the API returned" (post-fetch transform). Candidates, cheapest first: (a) the skip logic checks the file's RAW fetch metadata (a sidecar or the header line the page was fetched with) instead of the row count; (b) dedupe writes to a SUFFIXED file (`.deduped`) or defers to a suite-block-end pass that never runs mid-sequence on an incomplete pull; (c) the stop condition also compares the page's offset against the dataset total from size.json (already probed per suite). (a)+(c) are local and don't change the artifact shape; (b) changes what `b3sum train-*.json` covers — the manifest's aggregate-digest derivation would need the same-day note (precedent: the typed_decisions Issue-052 lift recomputed its aggregate).

## Workaround used (until fixed)

Delete the deduped pages before resuming (the re-fetch returns the raw 100-row page; the dedupe rewrites at the end of a COMPLETE pull — harmless). A `wc -l`-style cleanup is a TRAP: the pages are single-line compact JSON (0 newlines), so a line-count filter deletes every raw page while keeping the pretty-printed deduped ones.

- [ ] T1 fix the resume predicate (one of the shapes above; the 429-retry loop in rethink Bench 071's provisioning session is the reference consumer)
- [ ] T2 a self-test arm: plant a 99-row mid-sequence page + a size.json total, assert the resume does NOT stop
- [ ] T3 re-derive the sst5 t20k aggregate digest note IF the fix changes artifact bytes (the manifest's `ebfb0317…` row was derived pre-dedupe — this issue's investigation shows the recorded aggregate can't be re-derived on a post-dedupe tree; document which state the row pins)

Session: seal1-m5ultra-campaign, 1792740000
