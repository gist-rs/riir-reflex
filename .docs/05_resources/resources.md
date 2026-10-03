# Reflex — the modelless lane

Reflex is the family's free floor — a modelless decision engine that runs on your machine, answers from a corpus you author, and abstains by design when the evidence is thin. [Modelless: no neural network — no training run. Corpus: your documents — the corpus is the model. Abstain: the engine says "I don't know" instead of guessing.]

## What Reflex is

A product you install, not a service you sign up for: an open-source (MIT)
decision engine built on the public
[KatGPT-RS](https://github.com/katopz/katgpt-rs) modelless inference
primitives. There are no trained weights inside — answers come from
compression and similarity over your own documents, and the engine consumes
those primitives without ever editing or training them. Because nothing is
trained, nothing drifts under you: the same corpus and the same binary give
the same answer.

## What it answers

Typed decisions over a closed grammar you define: **choice** (pick one
option), **score** (rate every option), **yes-no** (a fixed pair). Every
question carries the state it is about, and the answer comes back with a
calibrated confidence — or, when the confidence is thin or the state sits
outside your corpus, with an abstain. The full distribution still rides with
an abstain, so your code can decide what a no-answer is worth instead of
catching an exception.

## Instinct — the idea on top

An abstain is not the end of the question; it is the moment a specialist
earns its keep. Instinct composes a specialist trained for exactly that
domain on top of the free engine, and the **fused gate** [the confidence
check that decides whether to consult a specialist] routes the abstained
cases to it. The specialist scores the survivors on top, never instead: the
free answer stands wherever it was confident, and the specialist pays only
where it was not. The open teaching lane lives at
[gist-rs/riir-instinct](https://github.com/gist-rs/riir-instinct).

## Where it runs

- **Browser (wasm):** the arena head and playground on
  [reflex.gist.rs](https://reflex.gist.rs) — decisions run in the visitor's
  own browser.
- **Mac, PC, Linux:** release binaries — one static native binary per
  platform, installable by script, homebrew or scoop.
- **Self-hosted server:** the same binary serving a small HTTP edge
  (`/decide`, `/healthz`) — no daemon framework.
- **Loopback by design:** the serve edge listens on your machine and answers
  browser calls only from origins you allow — no telemetry, no phone-home.
- **Latency class:** microsecond-class decisions on the modelless lane.

## Measure it

Every performance claim on this page is a class, not a figure. The measured
side lives on the site's bench lanes: [/bench/](/bench/).

- GitHub: [gist-rs/riir-reflex](https://github.com/gist-rs/riir-reflex)
- Measured lanes: [/bench/](/bench/)
- The one decision-flow figure:
  [decision_flow.md](https://github.com/gist-rs/riir-reflex/blob/develop/.docs/03_decision_flow/decision_flow.md)
