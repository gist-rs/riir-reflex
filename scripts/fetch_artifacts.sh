#!/bin/sh
# fetch_artifacts.sh — the public artifact fetch lane (Plan 623 T6 / Issue 069).
#
# Pulls PUBLIC artifacts from the org HF dataset lane
# (`hf://gist-rs/<repo>-artifacts/<blake3_cipher>`) into
# `artifacts/cache/<name>`, verifying BLAKE3 + exact size against the
# consuming manifest BEFORE the bytes are used — `blake3_plain` is the
# trust root; a fresh download is verified at `<name>.new` and promoted
# into the cache only on a match. Never a silent unverified cache.
#
# A protected row in this manifest is refused loud: a public repo's
# manifest carries public rows only (the workspace gate's law, re-checked
# at fetch time — this lane never fetches the moat).
#
# Self-contained by design: a fresh clone of this PUBLIC repo has no
# sibling checkouts, so the lane is shell + python3 + b3sum + the `hf` CLI —
# no deployer tool, no private repos. (The workspace-internal pull/push
# power tool is riir-deployer's `artifact-sync`; this lane is the
# standalone public distribution channel, Issue 069's scope.)
#
# Postures:
#   scripts/fetch_artifacts.sh            # fetch + verify into artifacts/cache/
#   CHECK=1 scripts/fetch_artifacts.sh    # verify already-cached bytes only (no network)
#
# Exit: 0 = clean (including the honest "no manifest yet — nothing to
#            fetch" state; this repo's fixtures are committed, the lane
#            exists for future public-class assets)
#       1 = findings (hash mismatch, refused protected row, failed download)
#       2 = REFUSED (malformed manifest, missing load-bearing tool)

set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
MANIFEST="$ROOT/artifacts/manifest.toml"
CACHE="$ROOT/artifacts/cache"
CHECK="${CHECK:-0}"

file_bytes() {
    wc -c < "$1" | tr -d ' '
}

if [ ! -f "$MANIFEST" ]; then
    echo "fetch_artifacts: no artifacts/manifest.toml — this repo carries no"
    echo "  public artifact rows yet (the committed fixtures are the default"
    echo "  path; this lane exists for future public-class assets). Nothing"
    echo "  to fetch; this is a statement, not a failure."
    exit 0
fi

command -v python3 >/dev/null 2>&1 || {
    echo "REFUSED: python3 is not available — the manifest parser is load-bearing" >&2
    exit 2
}
command -v b3sum >/dev/null 2>&1 || {
    echo "REFUSED: b3sum is not available — the trust root is load-bearing" >&2
    exit 2
}

# Parse the manifest: "class|name|hf_remote|blake3_plain|plain_bytes" per
# artifact row, "PIN|name|-|-|-" per source pin (nothing to fetch).
ROWS=$(python3 - "$MANIFEST" <<'PYEOF'
import sys, tomllib

with open(sys.argv[1], "rb") as f:
    data = tomllib.load(f)
unknown = set(data) - {"artifact", "source_pin"}
if unknown:
    print(f"REFUSED: unknown top-level tables {sorted(unknown)} (schema may not drift)",
          file=sys.stderr)
    sys.exit(2)
for pin in data.get("source_pin", []):
    print(f"PIN|{pin['name']}|-|-|-")
for row in data.get("artifact", []):
    remote = row.get("remote") or []
    hf = next((u for u in remote if u.startswith("hf://")), None)
    print(f"{row['class']}|{row['name']}|{hf or '-'}|{row['blake3_plain']}|{row.get('plain_bytes', 0)}")
PYEOF
) || exit 2

mkdir -p "$CACHE"

fetched=0
verified=0
failed=0

while IFS='|' read -r class name hf hash bytes; do
    case "$class" in
        PIN)
            echo "  pin: $name (source pin — nothing to fetch)"
            continue
            ;;
        protected)
            echo "REFUSED: row '$name' is PROTECTED — a public repo's manifest"
            echo "  carries public rows only (the workspace gate's law); this"
            echo "  lane never fetches the moat."
            exit 1
            ;;
    esac
    if [ "$hash" = "-" ]; then
        echo "  skip: $name (no hf remote declared)"
        continue
    fi
    dest="$CACHE/$name"
    pending=""
    if [ -f "$dest" ]; then
        :
    elif [ "$CHECK" = "1" ]; then
        echo "MISS: $name is not cached (run without CHECK=1 to fetch)"
        failed=$((failed + 1))
        continue
    else
        if ! command -v hf >/dev/null 2>&1; then
            echo "REFUSED: the hf CLI is not available and '$name' is not cached." >&2
            echo "  Install huggingface_hub (pip/uv) or pre-populate" >&2
            echo "  artifacts/cache/ (public datasets need no token)." >&2
            exit 2
        fi
        key=$(printf '%s' "$hf" | sed 's|.*/||')
        repo_id=$(printf '%s' "$hf" | sed 's|^hf://||; s|/[^/]*$||')
        tmp=$(mktemp -d "${TMPDIR:-/tmp}/fetch-artifacts.XXXXXX")
        if ! hf download "$repo_id" "$key" --repo-type dataset --local-dir "$tmp" >/dev/null 2>&1 \
            || [ ! -f "$tmp/$key" ]; then
            echo "FAIL: hf download of '$name' from $repo_id ($key)"
            rm -rf "$tmp"
            failed=$((failed + 1))
            continue
        fi
        mv "$tmp/$key" "$dest.new"
        rm -rf "$tmp"
        pending="$dest.new"
        fetched=$((fetched + 1))
    fi
    # The trust root: verify the PLAIN hash + exact size before use.
    target="${pending:-$dest}"
    actual=$(b3sum "$target" | cut -d' ' -f1)
    if [ "$actual" != "$hash" ]; then
        echo "HASH MISMATCH: $name carries $actual, the manifest pins $hash —"
        echo "  the bytes are refused (never a silent unverified cache)."
        rm -f "$dest.new"
        failed=$((failed + 1))
        continue
    fi
    if [ "$bytes" != "-" ] && [ "$(file_bytes "$target")" != "$bytes" ]; then
        echo "SIZE MISMATCH: $name is $(file_bytes "$target") B, the manifest pins $bytes B."
        rm -f "$dest.new"
        failed=$((failed + 1))
        continue
    fi
    if [ -n "$pending" ]; then
        mv "$pending" "$dest"
    fi
    verified=$((verified + 1))
    echo "  ok: $name (${hash})"
done <<EOF
$ROWS
EOF

if [ "$failed" -ne 0 ]; then
    echo "fetch_artifacts: FAILED — $failed finding(s), $verified row(s) verified"
    exit 1
fi
echo "fetch_artifacts: clean — $verified public row(s) verified, $fetched fetched"
exit 0
