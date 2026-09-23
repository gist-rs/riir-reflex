#!/usr/bin/env bash
# riir-reflex local CI guard — the same lanes every sibling repo runs.
# Layers (7): (1) clippy default -D warnings, (2) clippy --all-features,
# (3) clippy --no-default-features (the flag-OFF posture must compile),
# (4) cargo test (semantics gates, default features), (5) clippy at the
# laya feature set, (6) the G2+G4 decision-set bench (full mode — cargo
# bench is release by construction: a latency gate in a debug build
# measures an unoptimised binary), (7) the G5 laya parity gate (full
# mode; SKIP-LOUD when no weights root resolves — a skip is a deferral,
# never a green zero, and the final line NAMES what was skipped).
#
# Steps 6-7 run in full mode only; in quick mode the [n/7] numbering IS
# the partial signal (steps stop at [5/7]).
set -euo pipefail

LAYER="${1:-full}"
SKIPPED=""

echo "== [1/7] clippy default (all targets, -D warnings)"
cargo clippy --all-targets -- -D warnings

echo "== [2/7] clippy --all-features"
cargo clippy --all-targets --all-features -- -D warnings

echo "== [3/7] clippy --no-default-features (flag-OFF posture)"
cargo clippy --all-targets --no-default-features -- -D warnings

echo "== [4/7] cargo test (default features)"
cargo test

echo "== [5/7] clippy at each lane's own feature set (laya-riir, laya-riir-metal)"
cargo clippy --all-targets --features laya-riir -- -D warnings
cargo clippy --all-targets --features laya-riir-metal -- -D warnings

if [ "$LAYER" = "full" ]; then
  echo "== [6/7] decision_set_goat bench (G2 latency + G4 alloc, release)"
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
    echo "== [7/7] G5 laya parity (the riir lane — candle removed, .issues/006) + the laya-gated lib suite (release)"
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

if [ -n "$SKIPPED" ]; then
  echo "== guard PASSED (PARTIAL — skipped: $SKIPPED)"
else
  echo "== guard PASSED (full)"
fi
