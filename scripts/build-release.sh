#!/bin/sh
# ─────────────────────────────────────────────────────────────────────────────
# Plan 606 T1.6 — the manual release build pipeline (one command per target).
#
# Usage:
#   scripts/build-release.sh <target> [<target>...]
#   scripts/build-release.sh --licenses          # regenerate THIRD_PARTY_LICENSES.md only
#
# Per target: dist-profile build (strip + fat LTO + --remap-path-prefix) of
# the SHIPPING feature set (default + laya-riir — the candle-free cut,
# .issues/006; darwin targets carry laya-riir-metal since v0.2.2, the
# Plan 001 T4 Metal-default watchability lane) → package tar.gz (unix) /
# zip (windows) carrying the binary + THIRD_PARTY_LICENSES.md → one
# SHA256SUMS over every archive. The leak scan (scripts/binary_leak_scan.sh)
# runs separately over the packaged binaries.
#
# Cross targets: any triple ≠ the host routes through cargo-zigbuild
# (zig cc) — the windows-gnu + *-musl matrix; plain cargo cannot link
# those from macOS. Host triple builds with plain cargo, byte-identical
# to before.
#
# The home layout never reaches the artifacts: the remap prefix rewrites
# /Users/<user> (and the CARGO_TARGET_DIR if it lives under it) to /build —
# the Plan-105 measured fix for the 61-machine-path class.
# ─────────────────────────────────────────────────────────────────────────────
set -eu

cd "$(dirname "$0")/.."
REPO_ROOT="$(pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
PKG_DIR="$REPO_ROOT/dist/pkg"
LICENSES="$REPO_ROOT/THIRD_PARTY_LICENSES.md"

if [ "${1:-}" = "--licenses" ]; then
    cargo about generate --config .github/about/about.toml \
        .github/about/about.hbs > "$LICENSES"
    echo "licenses: $LICENSES ($(wc -l < "$LICENSES") lines)"
    exit 0
fi

[ $# -ge 1 ] || { echo "usage: $0 <target> [<target>...] | --licenses" >&2; exit 2; }
[ -f "$LICENSES" ] || { echo "error: $LICENSES missing — run '$0 --licenses' first" >&2; exit 1; }

mkdir -p "$PKG_DIR"
RUSTFLAGS="--remap-path-prefix $HOME=/build"
export RUSTFLAGS
BIN_NAME="reflex"
HOST_TARGET="$(rustc -vV | awk '/^host:/{print $2}')"

for target in "$@"; do
    CARGO_CMD="cargo"
    BUILD_SUB="build"
    if [ "$target" != "$HOST_TARGET" ] && command -v cargo-zigbuild >/dev/null 2>&1; then
        # cargo-zigbuild IS the build subcommand (cargo zigbuild, not cargo zigbuild build)
        CARGO_CMD="cargo zigbuild"
        BUILD_SUB=""
    fi
    # The darwin artifacts carry the Metal backend (v0.2.2 — the Plan 001
    # T4 watchability default: the laya lane runs Metal unless the visitor
    # opts out with LAYA_DEVICE=cpu). The metal deps are macOS-only by
    # construction; every other target keeps the CPU-only feature set.
    FEATURES="laya-riir"
    case "$target" in
        *darwin*) FEATURES="laya-riir-metal" ;;
    esac
    echo "== building $BIN_NAME v$VERSION for $target (profile dist, features: $FEATURES, via $CARGO_CMD)"
    $CARGO_CMD $BUILD_SUB --profile dist --features "$FEATURES" --target "$target" --bin "$BIN_NAME"

    EXE="$BIN_NAME"
    ARCHIVE_EXT="tar.gz"
    case "$target" in
        *windows*) EXE="$BIN_NAME.exe"; ARCHIVE_EXT="zip" ;;
    esac
    # Honor CARGO_TARGET_DIR (the sibling-lock isolation convention) —
    # cargo puts the artifact under <target-dir>/<triple>/<profile>/.
    TGT_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
    SRC_BIN="$TGT_DIR/$target/dist/$EXE"
    [ -f "$SRC_BIN" ] || { echo "error: built binary missing: $SRC_BIN" >&2; exit 1; }

    # [profile.dist] strip = true is honored by Apple ld64 (the host link)
    # and by zig's gnu-flavor ld (musl) but NOT reliably by zig's Mach-O
    # ld — measured v0.2.1: the x86_64-apple-darwin cross link kept its
    # symbol table (Onig data + rust_eh_personality) while the v0.2.0 cut
    # of the same manifest stripped clean. Strip Mach-O artifacts
    # explicitly (idempotent; __mh_execute_header is the stripped
    # baseline's one defined symbol) so the leak-scan gate's premise
    # holds by construction, and verify.
    case "$target" in
        *darwin*)
            SYMS=$(nm "$SRC_BIN" 2>/dev/null | grep -c -E '^[0-9a-f]{8,} [TDB]' || true)
            if [ "$SYMS" -gt 1 ]; then
                strip "$SRC_BIN"
                SYMS=$(nm "$SRC_BIN" 2>/dev/null | grep -c -E '^[0-9a-f]{8,} [TDB]' || true)
                echo "   stripped: $target (defined syms now $SYMS)"
                [ "$SYMS" -le 1 ] || { echo "error: strip failed to clear $SRC_BIN" >&2; exit 1; }
            fi
            ;;
    esac

    STAGE="$PKG_DIR/$BIN_NAME-v$VERSION-$target"
    rm -rf "$STAGE"; mkdir -p "$STAGE"
    cp "$SRC_BIN" "$STAGE/$EXE"
    cp "$LICENSES" "$STAGE/THIRD_PARTY_LICENSES.md"

    ARCHIVE="$PKG_DIR/$BIN_NAME-v$VERSION-$target.$ARCHIVE_EXT"
    rm -f "$ARCHIVE"
    # Contents at the archive ROOT (the cargo-heal installer + Homebrew
    # `bin.install` both expect `<binary>` + THIRD_PARTY_LICENSES.md at top
    # level — found by the G1 clean-install smoke, Plan 606 T2.5).
    case "$ARCHIVE_EXT" in
        tar.gz) tar -czf "$ARCHIVE" -C "$STAGE" "$EXE" THIRD_PARTY_LICENSES.md ;;
        zip) (cd "$STAGE" && zip -q "$ARCHIVE" "$EXE" THIRD_PARTY_LICENSES.md) ;;
    esac
    echo "   packaged: $ARCHIVE"
done

# One SHA256SUMS over every archive in the pkg dir (rebuilds wholesale —
# the file describes the whole release, never a subset).
SUMS="$PKG_DIR/SHA256SUMS"
rm -f "$SUMS"
for archive in "$PKG_DIR"/*.tar.gz "$PKG_DIR"/*.zip; do
    [ -f "$archive" ] || continue
    (cd "$PKG_DIR" && shasum -a 256 "$(basename "$archive")") >> "$SUMS"
done
echo "SHA256SUMS: $SUMS"
cat "$SUMS"
