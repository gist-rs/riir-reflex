#!/usr/bin/env bash
# riir-reflex local CI guard — the same lanes every sibling repo runs.
# Layers (9): (1) clippy default -D warnings, (2) clippy --all-features,
# (3) clippy --no-default-features (the flag-OFF posture must compile),
# (4) cargo test (semantics gates, default features), (5) clippy at the
# laya feature set, (6) the G2+G4 decision-set bench (full mode — cargo
# bench is release by construction: a latency gate in a debug build
# measures an unoptimised binary), (7) the G5 laya parity gate (full
# mode; SKIP-LOUD when no weights root resolves — a skip is a deferral,
# never a green zero, and the final line NAMES what was skipped),
# (8) the .docs numbered-book shape gate (cheap — always runs), and
# (9) the site-mirror parity check against ../reflex-site (cheap —
# always runs; SKIP-LOUD when the sibling checkout is absent).
#
# Steps 6-7 run in full mode only; in quick mode the [n/9] numbering IS
# the partial signal (cargo steps stop at [5/9]; the two cheap doc
# layers 8-9 still run).
set -euo pipefail

LAYER="${1:-full}"
SKIPPED=""

echo "== [1/9] clippy default (all targets, -D warnings)"
cargo clippy --all-targets -- -D warnings

echo "== [2/9] clippy --all-features"
cargo clippy --all-targets --all-features -- -D warnings

echo "== [3/9] clippy --no-default-features (flag-OFF posture)"
cargo clippy --all-targets --no-default-features -- -D warnings

echo "== [4/9] cargo test (default features)"
cargo test

echo "== [5/9] clippy at each lane's own feature set (laya-riir, laya-riir-metal)"
cargo clippy --all-targets --features laya-riir -- -D warnings
cargo clippy --all-targets --features laya-riir-metal -- -D warnings

if [ "$LAYER" = "full" ]; then
  echo "== [6/9] decision_set_goat bench (G2 latency + G4 alloc, release)"
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
    echo "== [7/9] G5 laya parity (the riir lane — candle removed, .issues/006) + the laya-gated lib suite (release)"
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

# ── The two cheap doc layers run in BOTH modes (a directory walk and a
# file compare cost nothing against the cargo lanes above). Layer 8 fails
# the guard on any shape finding; layer 9 is the site-mirror parity check
# — exit 2 (sibling checkout absent) is a LOUD skip and a deferral, never
# a green (the required_features_touched.sh precedent).

echo "== [8/9] .docs numbered-book shape (scripts/docs_shape_gate.py)"
python3 scripts/docs_shape_gate.py

echo "== [9/9] site-mirror parity (.docs/ ↔ ../reflex-site, sync_mirror.py --check)"
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

if [ -n "$SKIPPED" ]; then
  echo "== guard PASSED (PARTIAL — skipped: $SKIPPED)"
else
  echo "== guard PASSED (full)"
fi
