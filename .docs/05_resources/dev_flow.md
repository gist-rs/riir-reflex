# Dev flow — build a Reflex lane

> **Purpose:** the development loop for a Reflex decision lane — author
> rules/grammar (the tetris-head pattern: state + options as closed-grammar
> sentences), label a small calibration slice, refit the thresholds, serve
> on loopback, ship the binary. The compact source below renders
> `reflex_dev_flow.svg` for the site's `/resources` education page (ai
> Proposal 053); this doc block is the source of truth — re-render from it,
> never hand-edit the SVG.

## The compact source (renders `reflex_dev_flow.svg`)

Lanes are who runs what, all in Reflex's own accent: you offline (docs,
labels, the fit), your machine (the loopback edge), the release. Six steps
in path order, all live.

```gfflow
file  = "reflex_dev_flow.svg"
title = "Reflex: author a lane — grammar, labels, refit, serve, ship"
accent = "reflex"
intro = "Press play to walk the dev loop one step at a time, or click a dot to jump. Each step shows what goes in and what comes out where a wire exists — captured from the real release binary."

[[lane]]
id = "desk"; label = "You, offline";  note = "docs · labels · the fit"; color = "reflex"
[[lane]]
id = "run";  label = "Your machine";  note = "loopback · one origin"; color = "reflex"
[[lane]]
id = "ship"; label = "The release";   note = "one static binary"; color = "reflex"

[[step]]
id = "author"; n = "1"; lane = "desk"; col = 0
title = "Author the grammar"; body = "state and options as closed-grammar sentences"
status = "live"
[[step]]
id = "label"; n = "2"; lane = "desk"; col = 1
title = "Label a slice"; body = "a small calibration set — yours, offline"
status = "live"
[[step]]
id = "refit"; n = "3"; lane = "desk"; col = 2
title = "Refit the gates"; body = "thresholds from your labels, one-off"
status = "live"
[[step]]
id = "serve"; n = "4"; lane = "run"; col = 3
title = "Serve on loopback"; body = "one binary, one allowed origin"
status = "live"
[[step]]
id = "decide"; n = "5"; lane = "run"; col = 4
title = "Decide"; body = "an answer with confidence — or an honest abstain"
status = "live"
[[step]]
id = "bin"; n = "6"; lane = "ship"; col = 5
title = "Ship the binary"; body = "corpus and thresholds ride inside"
status = "live"

[[edge]]
from = "author"; to = "label"
[[edge]]
from = "label"; to = "refit"
[[edge]]
from = "refit"; to = "serve"
[[edge]]
from = "serve"; to = "decide"
[[edge]]
from = "decide"; to = "bin"

[[walk]]
title = "Author the grammar"
text  = "Write the domain as sentences: a state sentence and option sentences in a closed grammar. The grammar bounds what can be asked — nothing outside it is ever answered."
steps = ["author"]
[[walk]]
title = "Label a slice"
text  = "A small set of your own labeled examples, kept offline. This is the only data the fit consumes — no telemetry feeds it."
steps = ["label"]
[[walk]]
title = "Refit the gates"
text  = "Fit the gate thresholds from your labels, offline, one-off. The calibrator stays bit-identical until you refit it."
steps = ["refit"]
[[walk]]
title = "Serve on loopback"
text  = "Run the edge on your machine with one allowed origin. The health read is the loopback edge answering: which lanes are ready, which heads are fitted."
steps = ["serve"]
out   = { lang = "json", src = "data/flows/reflex_dev_flow/04_out.json", label = "GET /healthz — the edge is up" }
[[walk]]
title = "Decide"
text  = "The live path: an answer with calibrated confidence, or an honest abstain when the evidence is thin. This runbook question is answered by the corpus the loop just authored."
steps = ["decide"]
in    = { lang = "json", src = "data/flows/reflex_dev_flow/05_in.json", label = "POST /decide — a question your corpus covers" }
out   = { lang = "json", src = "data/flows/reflex_dev_flow/05_out.json", label = "the corpus's answer" }
[[walk]]
title = "Ship the binary"
text  = "One static release per platform; the corpus and the fitted thresholds ride with it. Same binary, same answers, on every host."
steps = ["bin"]
```

## What each step means

- **author rules + grammar** — write the domain as sentences: a state
  sentence and option sentences in a closed grammar (the tetris-head
  pattern). The grammar bounds what can be asked; nothing outside it is
  ever answered.
- **label a calibration slice** — a small set of your own labeled examples,
  kept offline. This is the only data the fit consumes.
- **refit** — fit the gate thresholds from your labels, offline, one-off.
  No telemetry feeds this step; the calibrator stays bit-identical until
  you refit it.
- **serve on loopback** — run the edge on your machine with one allowed
  origin. Answers are microsecond-class; abstains are a designed output.
- **decide** — the live path: an answer with calibrated confidence, or an
  honest abstain when the evidence is thin.
- **ship the binary** — one static release per platform; the corpus and the
  fitted thresholds ride with it.

## The walk (what each step's wire looks like)

The step-through is served with the figure; its payloads are derived by
reflex-site `scripts/build_flow_walks.py` from the captured release-binary
wire — never typed. The loopback step shows the health read (the edge is
up, which lanes are ready); the decide step shows a captured corpus
question and its answer — the corpus the dev loop just authored, answering.

## Rendering

The SVGs are rendered centrally from the fenced block above —
`reflex-site/scripts/render_flows.py` (the family flow renderer, Plan 620).
Edit the block here; never hand-edit the rendered SVG. The rendered
`reflex_dev_flow.svg` + `_m` + `.walk.json` are committed BESIDE this doc
(the same both-mirrors law as `03_decision_flow/`: doc-adjacent copy + the
site's `assets/reflex_dev_flow.svg`, byte-identical), and
`scripts/sync_mirror.py --check` in reflex-site is the drift detector
between renders.
