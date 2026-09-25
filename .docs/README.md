# riir-reflex — Documentation

The decision-engine repo's `.docs/` book, organized the fleet way: **numbered
folders** for sort order, **bare slugs** for files, a `README.md` index in
every folder, and this file as the top-level index.

## Convention

- **Folders are numbered** (`01_orientation/`, `02_protocols/`, …) for sort
  order. This matches `katgpt-rs/.docs/` and the rest of the fleet.
- **Files inside have NO number prefix** — add a new doc by dropping
  `slug.md` in the right folder and adding one line to that folder's
  `README.md` index table.
- The shape is gate-enforced: `scripts/docs_shape_gate.py` (wired into
  `scripts/ci_feature_guard.sh`) walks this tree, asserts every numbered
  folder carries its README index and every doc is listed in it, and fails
  loud on a folder/file-count regression.

## Folders

| Folder | What it covers |
|---|---|
| [`01_orientation/`](01_orientation/) | Repo orientation: the sibling dependency-graph artifact |
| [`02_protocols/`](02_protocols/) | The laya comparison lane's contracts: benchmark task protocols, checkpoint provenance pin, fetched-dataset manifest |
| [`03_decision_flow/`](03_decision_flow/) | The end-to-end decision flow narrative + its SVG diagram (the diagram is mirrored to the arena site) |
| [`04_agent_skill/`](04_agent_skill/) | The `reflex-integration` agent skill (SKILL.md) — the in-repo source of truth; published on reflex.gist.rs through a mirror |

## Mirrors

Two files in this book are published on [reflex.gist.rs](https://reflex.gist.rs)
through mirrors in `../reflex-site` (`assets/decision_flow.svg`,
`skills/reflex-integration/SKILL.md`). The `.docs/` copies are the source of
truth: edit here, then run `python3 ../reflex-site/scripts/sync_mirror.py`
and commit both repos. The guard's site-mirror layer fails on drift.
