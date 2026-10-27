#!/usr/bin/env bash
# riir-reflex local CI guard — the same lanes every sibling repo runs.
# Layers (11): (1) clippy default -D warnings, (2) clippy --all-features,
# (3) clippy --no-default-features (the flag-OFF posture must compile),
# (4) cargo test (semantics gates, default features), (5) clippy at the
# laya feature set, (6) the G2+G4 decision-set bench (full mode — cargo
# bench is release by construction: a latency gate in a debug build
# measures an unoptimised binary), (7) the G5 laya parity gate (full
# mode; SKIP-LOUD when no weights root resolves — a skip is a deferral,
# never a green zero, and the final line NAMES what was skipped),
# (8) the fetch_datasets resume self-test (Issue 096 — cheap, hermetic
# loopback, always runs), (9) the .docs numbered-book shape gate (cheap
# — always runs), (10) the site-mirror parity check against
# ../reflex-site (cheap — always runs; SKIP-LOUD when the sibling
# checkout is absent), and (11) the fmt gate (Issue 099 — bare rustfmt
# per file with skip_children, NEVER `cargo fmt --all`: it would format
# the path-dep siblings, incl. katgpt-rs's INTENTIONAL pinned drift;
# cheap — always runs).
#
# Steps 6-7 run in full mode only; in quick mode the [n/11] numbering IS
# the partial signal (cargo steps stop at [5/11]; the four cheap layers
# 8-11 still run).
set -euo pipefail

LAYER="${1:-full}"
SKIPPED=""

echo "== [1/11] clippy default (all targets, -D warnings)"
cargo clippy --all-targets -- -D warnings

echo "== [2/11] clippy --all-features"
cargo clippy --all-targets --all-features -- -D warnings

echo "== [3/11] clippy --no-default-features (flag-OFF posture)"
cargo clippy --all-targets --no-default-features -- -D warnings

echo "== [4/11] cargo test (default features)"
cargo test

echo "== [5/11] clippy at each lane's own feature set (laya-riir, laya-riir-metal, clef-ane)"
cargo clippy --all-targets --features laya-riir -- -D warnings
cargo clippy --all-targets --features laya-riir-metal -- -D warnings
cargo clippy --all-targets --features clef-ane -- -D warnings

if [ "$LAYER" = "full" ]; then
  echo "== [6/11] decision_set_goat bench (G2 latency + G4 alloc, release)"
  cargo bench --bench decision_set_goat
else
  SKIPPED="decision_set_goat bench"
fi

# The G5 parity gate: minutes-long, GEMM-bound, and conditional on a
# weights root. The skip names the env that arms it, and the FINAL LINE
# of this script reports it — a reader who reads only the last line must
# never read an unqualified green over an unrun gate (the full_gate
# PARTIAL lesson).
if [ "$LAYER" = "full" ]; then
  if [ -n "${LAYA_WEIGHTS_DIR:-}" ] || [ -n "${LAYA_HOME:-}" ] || [ -d "${HOME}/.cache/huggingface/hub/models--convaiinnovations--laya" ] || [ -d "${HOME}/.cache/riir-reflex/laya" ]; then
    echo "== [7/11] G5 laya parity (the riir lane — candle removed, .issues/006) + the laya-gated lib suite (release)"
    # --lib AND the parity target: the router/lang unit suite (27 tests,
    # incl. the occurrence-counting regression) is laya-riir-gated too — a
    # --test-only run would compile it and never execute it. The lane
    # runs at its default CPU posture here; the METAL posture is a GPU gate
    # (GPU-exclusivity rule) and runs via
    #   LAYA_DEVICE=metal cargo test --release --features laya-riir-metal \
    #     --test laya_riir_parity
    # before any riir-Metal number is published (.issues/005 T4).
    cargo test --release --features laya-riir --lib --test laya_riir_parity
  else
    SKIPPED="${SKIPPED:+$SKIPPED, }G5 laya parity (no weights root — set LAYA_WEIGHTS_DIR/LAYA_HOME)"
  fi
fi

# ── The three cheap layers run in BOTH modes (a loopback self-test, a
# directory walk and a file compare cost nothing against the cargo lanes
# above). Layer 8 fails the guard if the resume predicate regresses;
# layer 9 fails the guard on any shape finding; layer 10 is the
# site-mirror parity check — exit 2 (sibling checkout absent) is a LOUD
# skip and a deferral, never a green (the required_features_touched.sh
# precedent).

echo "== [8/11] fetch_datasets resume self-test (Issue 096 — hermetic loopback)"
sh scripts/fetch_datasets.sh --self-test

echo "== [9/11] .docs numbered-book shape (scripts/docs_shape_gate.py)"
python3 scripts/docs_shape_gate.py

echo "== [10/11] site-mirror parity (.docs/ ↔ ../reflex-site, sync_mirror.py --check)"
if [ ! -f "../reflex-site/scripts/sync_mirror.py" ]; then
  echo "⚠ SKIPPED — ../reflex-site not checked out (or sync_mirror.py not landed there yet), so this layer verified NOTHING."
  echo "  (the site mirror at reflex.gist.rs serves .docs/03_decision_flow/decision_flow.svg + .docs/04_agent_skill/SKILL.md;"
  echo "   the sibling checkout arms this layer)"
  SKIPPED="${SKIPPED:+$SKIPPED, }site-mirror parity (no ../reflex-site sibling checkout)"
else
  set +e
  python3 ../reflex-site/scripts/sync_mirror.py --check
  MIRROR_RC=$?
  set -e
  if [ "$MIRROR_RC" -eq 2 ]; then
    echo "⚠ SKIPPED (rc=2) — the sibling named its own absence; the site-mirror layer is DEFERRED, not green."
    SKIPPED="${SKIPPED:+$SKIPPED, }site-mirror parity (sync_mirror.py reported no sibling)"
  elif [ "$MIRROR_RC" -ne 0 ]; then
    echo "✗ mirror DRIFT — run: python3 ../reflex-site/scripts/sync_mirror.py  (then commit BOTH repos)"
    exit 1
  else
    echo "   mirror parity holds (.docs/ is the source of truth)"
  fi
fi

echo "== [11/11] fmt gate (Issue 099 — scripts/fmt_gate.py; drain list scripts/fmt_pending_expected.txt)"
python3 scripts/fmt_gate.py

if [ -n "$SKIPPED" ]; then
  echo "== guard PASSED (PARTIAL — skipped: $SKIPPED)"
else
  echo "== guard PASSED (full)"
fi
