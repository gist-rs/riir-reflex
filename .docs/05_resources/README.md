# 05_resources — teach the lane (the /resources DRY sources)

| File | What it covers |
|---|---|
| `resources.md` | The canonical teaching doc — what Reflex is, what it answers, Instinct the idea, where it runs (the DRY source mirrored to the public site's /resources page) |
| `dev_flow.md` | The development build-flow figure — author grammar, label a calibration slice, refit, serve on loopback, ship the binary (the source block for the site's `reflex_dev_flow.svg`) |

Both files are the DRY sources for reflex.gist.rs's `/resources` education
page (ai Proposal 053): the site mirrors them (`docs/reflex/resources.md`,
`docs/reflex/dev_flow.md`, `assets/reflex_dev_flow.svg`) instead of
re-typing the content — edit here, the `.docs/` copies are the source of
truth. The SVG itself is rendered centrally from `dev_flow.md`'s fenced
block; no `.svg` is committed in this folder.
