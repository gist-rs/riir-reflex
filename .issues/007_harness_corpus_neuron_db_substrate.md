# Issue 007 — harness families + corpus on the neuron-db substrate (Warm-tier kv via the RELEASED `ndb` CLI now, latent semantics DECIDED: in-harness, Issue 024)

**Status:** P2 DECIDED 2026-09-24 (neither neuron-db route; in-harness `slice_leak` report filed as Issue 024). P1 LANDED 2026-09-24 (this session, dedicated per the addendum —
no table-regeneration ran alongside). `corpus_db` feature (opt-in, native-only,
DEFAULT-OFF): `src/harness/corpus_db.rs` subprocess store (`--json`-only,
stdin writes, one-corpus-one-row, BLAKE3 digest-pinned corpus keys,
NDB_ASSUME_YES, NDB_PASSPHRASE/insecure-plaintext posture, NDB_BIN → PATH →
loud-refuse resolution) + harness bin `--runs-kv` (one row per run: value =
the exact results.json bytes) + `--save-corpus <suites>` (digest-pinned row +
read-back verification) + the consumer-side golden pin against the REAL
binary (`binary_wire_golden_round_trip`, loud-skip without NDB_BIN — live-
verified 2026-09-24 against ndb 0.1.0: round-trip byte-exact, scan, wrong-
digest refusal, not_found classification) + BOUNDARY.md runtime-dep row.
Corpus LOAD-from-kv: DEFERRED (touches `prepare`/`load_rows` + the digest-
pin-vs-local gate — next slice). P2 latent semantics: DECIDED 2026-09-24,
the trigger measured as fired (P2 below). Every default-posture gate stayed green (corpus_db off).

---

**Earlier status (superseded by the P1 landing above):** REVISED 2026-09-22 (round 2, verdict AGREE) — owner directive ("release riir-neuron-db as binary/cli like riir-clippy new release repos and let riir-reflex use as is bc i dont want to leak db src code") — **P1 is CLI-first: consume the `ndb` binary as a subprocess, ZERO sibling cargo deps. P2 (latent semantics) is UNDECIDED between the CLI route and an opt-in compiled dep — decided by the measurement trigger, not by this issue.** Governing proposal: [riir-neuron-db Proposal 002](../../riir-neuron-db/.proposals/002_ndb_binary_only_distribution.md) (verdict-rounded with Claude: round-1 REVISE applied — value-only law + `--json` contract, local-build-first sequencing, `NDB_BIN` override; round-2 AGREE). Originally FILED 2026-09-22 as a path-dep plan; superseded same day by the owner call before any dep landed (the boundary-gap-first pattern held: no `Cargo.toml` dep ever existed).
**2026-09-23 owner-verdict addendum:** P1 lands in a DEDICATED session, not alongside a table-regeneration run (runner-ordering: the tables this session publishes come FROM the runner P1 would rewire; landing both in one window ships tables from a runner that no longer exists — Claude verdict, substrate-readiness grounds). The substrate-readiness measurement below converts this from "blocked on substrate" to "ready, scheduled".

## Why the substrate is right (the honest verdict)

riir-clippy's precedent splits two things this repo currently conflates in
compiled-in Rust + an overwritten `results.json`:

1. **Accumulating runtime data → Warm-tier kv.** The harness today
   overwrites `.benchmarks/001_phase1_tables/results.json` every run —
   there is NO run history, so no trend gate, no regression detection
   across commits, no calibration-observation store. riir-clippy keeps
   self-evolve trajectories in `riir-neuron-db` Warm-tier
   (`.heal/trajectories.kv`, `LocalKvStore`, 826 ns writes) for exactly
   this shape. The reflex analog: per-run, per-suite metric rows
   (acc/ECE/abstain/latency + the commit sha + box state) in a
   `.harness/runs.kv` — driven through `ndb` (the released CLI over the
   same store) instead of a compiled-in `LocalKvStore`.
2. **Growing corpora → data, not source.** `families.rs` + the dataset
   corpora grow per Issue-004-style intake; every growth recompiles the
   binary and re-pins nothing. Corpora as kv rows (BLAKE3-digested, the
   `dataset_manifest.md` law) let suites grow and version without
   recompiling — with the compiled-in set staying as the hermetic gate
   floor (below).

3. **Latent semantics (the owner's stated goal) — P2, DECIDED 2026-09-24
   (in-harness report, Issue 024; see Phases).** Latent
   slice-disjointness + corpus KNN over `ShardIndex`; today the
   self-inclusion-leak law is EXACT-string equality
   (`tests/harness_families_gates.rs`), so a cal case that is a
   near-paraphrase of an eval case leaks today. Whether P2 rides the CLI
   or an opt-in compiled dep is OPEN (below) — the need itself is
   unmeasured until a near-duplicate leak is caught.

## The consumption law (P1 — what changed and why)

**No `riir-neuron-db` (or riir-rag) cargo dep for P1, in any feature. The
Warm tier is consumed as the `ndb` binary via `std::process`.**

- The no-source-leak posture becomes **structural** (a missing link dep)
  instead of conventional (release-set policy). Released reflex binaries
  (`RELEASE_FEATURES = modelless+laya`) can never carry compiled db code —
  and neither can any future feature promotion, by construction.
- The default build stays ONE sibling code-level dep (katgpt-core)
  FOREVER for P1 — no second boundary dep row, no riir-ai tree pull, no
  SIBLING_REPOS_TOKEN fallout.
- Workspace precedent for the posture: riir-deployer's zero-sibling-deps
  law ("shells out to cargo/wrangler/docker/ssh"). Harness persistence is
  per-run — subprocess cost (~1 ms/spawn) is irrelevant at run frequency.
- **The law is VALUE-ONLY (verdict round 1, code-verified):** the VALUE
  payload is schema-agnostic — `ndb` stores opaque bytes, THIS repo owns
  100% of the row schema (metric/corpus rows = our JSON + our BLAKE3
  digests, blake3 from crates.io — public dep, boundary-clean). But the
  CLI's key grammar, stdout grammar, and error taxonomy ARE shared
  neuron-db-owned wires — so **reflex only ever calls `--json` forms**
  (Proposal 002 Phase 1 adds the machine mode + a stable exit-code
  taxonomy + fail-closed `get`), and pins them with a consumer-side
  golden test against the actual binary (the riir-clippy Issue 084
  discipline — the consumer-side pin over a producer's wire keys,
  renumbered from the doubly-allocated 082).
- **Binary resolution: `NDB_BIN` env/config override FIRST**, PATH
  fallback. CI, locally built binaries, and version pinning across a
  drifting fleet all need the override. Probe `ndb --version` at first
  use; REFUSE loud (naming the install one-liner + the `NDB_BIN` escape)
  when absent.
- **Writes: stdin or `--file`, NEVER argv** — payload in `ps`/argv leaks
  on multi-agent boxes and is ARG_MAX-bounded.
- **Row shape: one corpus = ONE row** (a single digest-pinned JSON blob —
  the manifest law wants this anyway). `scan` returns keys+lengths only
  and there is no batch get: an N-row corpus read is `1 + N` spawns, each
  re-opening + decrypting the WAL. `scan` is for enumeration/inventory
  only.

## Substrate readiness (MEASURED 2026-09-23 — P1's precondition is met; the next session starts here, not from a re-derivation)

- **The `--json` contract SHIPPED:** riir-neuron-db `2670621`
  ("ndb binary-only release Phase 1 — frozen --json contract + exit
  taxonomy, sync-fabric cut, release pipeline") — global `--json` flag
  over every consumer-facing subcommand, one compact JSON object per
  line on stdout, classified failures as
  `{"v":1,"ok":false,"code":"…","error":"…}` with the stable `code`
  taxonomy, `ndb --json version` as the consumer version probe (clap's
  `--version` short-circuits before dispatch, so the probe must be the
  subcommand form). Sync fabric CUT from the public release as Proposal
  002 wanted (`sync` remains a local module, not a released surface).
- **`NDB_ASSUME_YES` shipped:** `crates/neuron-db-cli/src/lib.rs`
  `ASSUME_YES_ENV` — the agents/CI consent bypass P1 specified, already
  CLI-side.
- **Local-build path live:** `cargo build -p neuron-db-cli` in
  `../riir-neuron-db` produces the `ndb` binary; P1's `NDB_BIN`
  resolution (override → PATH fallback → loud refuse naming the build
  one-liner) has a real target on this box today.
- **Verified by reading the code this session** (this repo's session,
  owner verdict of the same date): `crates/neuron-db-cli/src/bin/ndb.rs`
  — the `--json` seam (`json_out`/`json_error`), the `CliError` code
  classification, the get/put/scan subcommand surface, base64 for
  non-utf8 values under `--json`.
- What P1 still owes at landing (unchanged from the phases above):
  the consumer-side golden pin against the REAL binary (the
  riir-clippy Issue-084 discipline), the BOUNDARY.md runtime-dep row
  in the same commit as the code, value-only law (our JSON + our
  BLAKE3 digests, blake3 from crates.io), stdin/`--file` writes never
  argv, one-corpus-one-row, and the compiled-in corpora staying the
  hermetic gate floor.

## Phases

- [x] **P1 — `corpus_db` (opt-in, native-only, DEFAULT-OFF, subprocess;
  LOCAL-BUILD FIRST, not gated on the v0.1.0 release):** resolve the
  binary via `NDB_BIN` (dev: the sibling's `cargo build -p neuron-db-cli`
  output — a path to a BINARY is still zero cargo deps; the M3 has the
  sibling checked out); `--json`-only calls; `.harness/runs.kv`
  run-history rows + one-row corpora; agents/CI pass
  `NDB_ASSUME_YES=1` (already shipped CLI-side). The compiled-in corpora
  stay the deterministic gate floor: every gate test must stay green with
  the feature OFF, and the kv corpora are digest-pinned against the
  compiled-in set when both exist. No dep allowlist row — a BOUNDARY.md
  **runtime-dep row** lands with the code in the same commit
  (boundary-gap-first). The flag never joins the release set (moot under
  the CLI posture, kept as belt-and-braces). **LANDED 2026-09-24 —
  store dir default `.harness/ndb-data` (`--kv-dir` override); corpus
  LOAD-from-kv is the deferred remainder (below).**
- [-] **P1 remainder — corpus LOAD-from-kv** (deferred with reason):
  loading a suite's envelopes FROM the digest-pinned row (instead of
  `.raw/datasets/`) touches `prepare`/`load_rows` (the run's data path)
  and needs the digest-pin-vs-local check wired per suite ("digest-pinned
  against the compiled-in/local set when both exist") — a runner-surgery
  slice with its own e2e (load-from-kv run must produce byte-identical
  metrics to the local-files run for the same digest). Write + verify
  side is live (`--save-corpus` read-back-verifies each row); no consumer
  exists yet, so the load path has no caller to serve — land it when the
  first kv-only run is actually wanted (e.g. CI without the dataset
  checkout).
- [x] **P2 — DECIDED 2026-09-24 (owner delegated the call): NEITHER route
  for the leak gate; (b) REJECTED outright; (a) kept only as a scale-gated
  reopen.** The trigger was MEASURED rather than awaited
  (`scripts/slice_leak_probe.py`, 2.3 s on a load-6 M3, zero deps): test
  rows with a >= 0.8 char-4-gram Jaccard (or exact) twin in the fetched
  train slice — ag_news **6.8%** (the same wire story from two outlets),
  banking77 **3.2%**, massive **2.9%** (13 exact, **2** of them
  label-conflicting), prompt_injections 1.7%, emotion/sst5/xnli ~0, and
  ~98% of the pairs share the label. typed_decisions' 12.75% is a
  fixed-JSON-schema TEMPLATE artifact, not a paraphrase leak. So the
  trigger has FIRED, and what it asks for is a **report**, not storage:
  - Why not (a): a leak check over <= 4000 x 3000 rows is an in-memory
    inverted index that finishes in seconds. `ndb shard` + relocating
    the DFT embedding into the storage leaf is a cross-repo proposal
    whose only gain would be persistence and KNN at a scale this corpus
    does not have.
  - Why not (b): a compiled `riir-neuron-db` dep breaks the CLI-only
    posture this issue is built on, adds a boundary row, and buys
    nothing the in-harness index does not already give.
  - What lands instead: **Issue 024**, an in-harness `slice_leak` report
    (opt-in feature, modelless). The probe's counts are its known-answer
    oracle.
  - ⚠ Scope: the references use these same public test splits, so the
    leak affects them as well. It does NOT void our comparison against
    published numbers. It DOES inflate absolute accuracy for any
    retrieval/centroid lane, which is why it becomes a disclosed column
    rather than a filter.
  - Reopen (a) only if the harness corpus outgrows memory (full-split
    fetch, or cross-run corpus KNN is actually wanted).
- **P2 (superseded text, kept for the record) — `latent_eval` (opt-in): UNDECIDED between two routes.**
  (a) CLI route: latent slice-disjointness gate + corpus KNN via a future
  `ndb shard` family (requires the modelless BLAKE3+DFT embedding's
  relocation from riir-rag into the storage leaf — its own proposal
  there, argued on DRY/boundary merits, never inherited from the
  distribution decision); (b) opt-in compiled `riir-neuron-db` dep for
  P2 ONLY — never in any release set, its own issue+row+code commit.
  **The trigger that decides: a near-duplicate leak measured in the
  wild** (two slices sharing a paraphrase). Until then neither route is
  taken and no GOAT bench is written for a guessed need.

## Non-goals

- No chain/`SyncBlock` involvement — corpora and run history are local
  durability (Warm tier), never chain-committed state.
- No training dependency — BLAKE3+DFT latent projections and sigmoid
  gates only (modelless-first mandate; freeze/thaw is the only weight
  mutation if a frozen embedding snapshot is ever added).
- No hot-path storage through the CLI — subprocess cost (~ms/spawn,
  1+N-row reads without the one-corpus-one-row law) bounds this to
  run-history, corpora, and gate queries. Hot-path consumers stay
  compiled-in (the riir-clippy posture; not this repo's shape).
- No `serve`/`sync` usage — the sync fabric is CUT from `ndb`'s public
  release (Proposal 002) and is out of scope here regardless.

## Reopen triggers

The harness grows a third lane (arena server-side outcomes) that needs
cross-process durability → P1 moves up. A near-duplicate leak is
measured in the wild (two slices sharing a paraphrase) → P2's route
decision opens (and with it, neuron-db Proposal 002 Phase 4 iff the CLI
route wins). **FIRED and DECIDED 2026-09-24:** see P2 above. The
remaining (a) reopen is scale only: a corpus larger than memory. The CLI proves measurably burdensome in CI beyond
`NDB_ASSUME_YES` → the `--no-identity` posture decision opens
neuron-db-side.
