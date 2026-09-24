#!/bin/sh
# fetch_datasets.sh — riir-reflex Plan 603 T1.5 benchmark dataset fetch layer.
#
# Pulls every benchmark suite's rows from the HF datasets-server /rows API into
# .raw/datasets/<suite>/<split>-<NNN>.json — one file per 100-row page, offset
# paging until a page returns fewer than 100 rows or the suite's row cap is
# hit. Companion probes (/splits, /size) land beside the pages as splits.json
# / size.json. .raw/ is gitignored; re-running regenerates the files (the
# committed manifest .docs/dataset_manifest.md records their blake3 digests,
# byte sizes and row counts).
#
# Idempotent: an existing page file with the row count its position expects is
# skipped; FORCE=1 refetches everything. Politeness: 0.3s sleep between calls
# (SLEEP=<sec> overrides — empirically HF datasets-server 429s somewhere
# around ~130 rapid unauthenticated requests, so long runs want SLEEP=1.5 or
# more; re-run after a 429 wall once the limiter cools down, the skip logic
# resumes exactly where it stopped), curl --max-time 60 --retry 2, plus one
# 5s-backoff retry per page for non-200 / JSON-error responses. A failed page
# is recorded loudly and the remaining suites still run (never abort
# everything).
#
# banking77 — PolyAI/banking77 (the Colab variant) is script-based and 404s on
# /rows; the mteb/banking77 mirror (the bench_apps variant) is fetched instead.
# dataset, which the datasets-server no longer serves — /rows 404s and /splits
# 500s. The suite stays in the attempt list so every run re-proves the gap.
#
# Usage:  scripts/fetch_datasets.sh
#         FORCE=1 scripts/fetch_datasets.sh
#
# Exit: 0 = every attempted page fetched clean; 1 = at least one failure
# (see the FAIL lines and the manifest's Gaps section).

set -eu

PAGE=100
SLEEP="${SLEEP:-0.3}"
FORCE="${FORCE:-0}"
BASE="https://datasets-server.huggingface.co"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
OUT="$ROOT/.raw/datasets"
MAX_PARTS=200   # hard stop: 200 pages x 100 rows is far above every cap here

FILES=0
ROWS=0
FAILED=0

log() { printf '%s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

# Python interpreter for the jq-less fallback paths (Issue 018: the windows
# lane). `python3` on Windows is the Microsoft Store alias STUB — it exists
# on PATH but only prints "Python was not found". Resolve a WORKING
# interpreter once: prefer python3, fall back to `py -3` (the standard
# Windows launcher). Sets PY3 to the full invocation string, or empty when
# neither works (the jq paths below never consult PY3 when jq is present).
PY3=""
if ! have jq; then
    if have python3 && python3 -c pass >/dev/null 2>&1; then
        PY3="python3"
    elif have py && py -3 -c pass >/dev/null 2>&1; then
        PY3="py -3"
    fi
fi

# curl GET -> file; -f so non-2xx exits non-zero; --retry handles transient
# transport errors / 429 / 5xx at the curl level.
http_get() {
    curl -fsSL --max-time 60 --retry 2 -o "$2" "$1"
}

# Row count of a /rows response file.
rows_count() {
    if have jq; then
        jq '.rows | length' "$1"
    elif [ -n "$PY3" ]; then
        $PY3 - "$1" <<'EOF'
import json, sys
with open(sys.argv[1], encoding="utf-8") as f:
    print(len(json.load(f).get("rows", [])))
EOF
    else
        echo 0
    fi
}

# True if the file is a well-formed /rows response (an object with "rows").
rows_valid() {
    if have jq; then
        jq -e 'type == "object" and has("rows")' "$1" >/dev/null 2>&1
    elif [ -n "$PY3" ]; then
        $PY3 - "$1" <<'EOF'
import json, sys
try:
    with open(sys.argv[1], encoding="utf-8") as f:
        d = json.load(f)
    sys.exit(0 if isinstance(d, dict) and "rows" in d else 1)
except Exception:
    sys.exit(1)
EOF
    else
        return 1
    fi
}

# True if <suite>/splits.json shows the config/split pair.
splits_have() {
    if have jq; then
        jq -e --arg c "$2" --arg s "$3" \
           '.splits[]? | select(.config == $c and .split == $s)' \
           "$OUT/$1/splits.json" >/dev/null 2>&1
    elif [ -n "$PY3" ]; then
        $PY3 - "$OUT/$1/splits.json" "$2" "$3" <<'EOF'
import json, sys
try:
    with open(sys.argv[1], encoding="utf-8") as f:
        d = json.load(f)
    hit = any(s.get("config") == sys.argv[2] and s.get("split") == sys.argv[3]
              for s in d.get("splits", []))
    sys.exit(0 if hit else 1)
except Exception:
    sys.exit(1)
EOF
    else
        return 1
    fi
}

# fetch_suite <suite> <urlencoded-dataset> <urlencoded-config>
#             <urlencoded-split> <cap: number | "all">
fetch_suite() {
    suite=$1; ds=$2; cfg=$3; split=$4; cap=$5
    dir="$OUT/$suite"
    mkdir -p "$dir"
    part=0
    total=0
    files=0
    suite_failed=0
    while [ "$part" -lt "$MAX_PARTS" ]; do
        if [ "$cap" != "all" ] && [ "$total" -ge "$cap" ]; then
            break
        fi
        pagenum=$(printf '%03d' "$part")
        file="$dir/$split-$pagenum.json"
        want=$PAGE
        if [ "$cap" != "all" ]; then
            want=$((cap - total))
            if [ "$want" -gt "$PAGE" ]; then
                want=$PAGE
            fi
        fi
        if [ "$FORCE" != "1" ] && [ -s "$file" ] && rows_valid "$file"; then
            n=$(rows_count "$file" 2>/dev/null || echo 0)
            if [ "$n" -gt 0 ]; then
                log "  skip   $suite/$split-$pagenum ($n rows)"
                total=$((total + n))
                files=$((files + 1))
                if [ "$n" -lt "$want" ]; then
                    break   # a short page can only be the terminal page
                fi
                if [ "$cap" != "all" ] && [ "$total" -ge "$cap" ]; then
                    break
                fi
                part=$((part + 1))
                continue
            fi
        fi
        offset=$((part * PAGE))
        url="$BASE/rows?dataset=$ds&config=$cfg&split=$split&offset=$offset&length=$PAGE"
        tmp="$file.tmp"
        ok=0
        if http_get "$url" "$tmp"; then
            ok=1
        fi
        if [ "$ok" -eq 0 ]; then
            sleep 5
            if http_get "$url" "$tmp"; then
                ok=1
            fi
        fi
        if [ "$ok" -eq 0 ]; then
            rm -f "$tmp"
            log "  FAIL   $suite/$split-$pagenum (http error, offset=$offset)"
            FAILED=$((FAILED + 1))
            suite_failed=1
            break
        fi
        if ! rows_valid "$tmp"; then
            rm -f "$tmp"
            log "  FAIL   $suite/$split-$pagenum (JSON error body, offset=$offset)"
            FAILED=$((FAILED + 1))
            suite_failed=1
            break
        fi
        n=$(rows_count "$tmp" 2>/dev/null || echo 0)
        if [ "$n" -eq 0 ]; then
            rm -f "$tmp"
            log "  done   $suite/$split (no more rows at offset=$offset)"
            break
        fi
        mv "$tmp" "$file"
        FILES=$((FILES + 1))
        ROWS=$((ROWS + n))
        total=$((total + n))
        files=$((files + 1))
        log "  fetch  $suite/$split-$pagenum rows=$n offset=$offset"
        if [ "$n" -lt "$PAGE" ]; then
            break
        fi
        if [ "$cap" != "all" ] && [ "$total" -ge "$cap" ]; then
            break
        fi
        part=$((part + 1))
        sleep "$SLEEP"
    done
    if [ "$part" -ge "$MAX_PARTS" ]; then
        log "  FAIL   $suite/$split hit MAX_PARTS=$MAX_PARTS guard"
        FAILED=$((FAILED + 1))
        suite_failed=1
    fi
    rows_total=$(jq -r '.num_rows_total // "?"' "$dir/$split-000.json" 2>/dev/null || echo "?")
    if [ "$suite_failed" -eq 0 ]; then
        log "  ==     $suite/$split: $total rows in $files file(s) (dataset total: $rows_total)"
    else
        log "  ==     $suite/$split: $total rows in $files file(s) BEFORE FAILURE (dataset total: $rows_total)"
    fi
    sleep "$SLEEP"
}

# probe_splits <suite> <urlencoded-dataset> — save the /splits listing.
probe_splits() {
    suite=$1; ds=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/splits.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ]; then
        log "  skip   $suite/splits.json (cached)"
        return 0
    fi
    url="$BASE/splits?dataset=$ds"
    ok=0
    if http_get "$url" "$file.tmp"; then
        ok=1
    fi
    if [ "$ok" -eq 0 ]; then
        sleep 5
        if http_get "$url" "$file.tmp"; then
            ok=1
        fi
    fi
    if [ "$ok" -eq 1 ]; then
        mv "$file.tmp" "$file"
        log "  probe  $suite/splits.json"
        if have jq; then
            jq -r '.splits[] | "         \(.config)/\(.split)"' "$file"
        fi
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/splits.json probe"
        FAILED=$((FAILED + 1))
    fi
    sleep "$SLEEP"
}

# probe_size <suite> <urlencoded-dataset> — save the /size listing.
probe_size() {
    suite=$1; ds=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/size.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ]; then
        log "  skip   $suite/size.json (cached)"
        return 0
    fi
    url="$BASE/size?dataset=$ds"
    ok=0
    if http_get "$url" "$file.tmp"; then
        ok=1
    fi
    if [ "$ok" -eq 0 ]; then
        sleep 5
        if http_get "$url" "$file.tmp"; then
            ok=1
        fi
    fi
    if [ "$ok" -eq 1 ]; then
        mv "$file.tmp" "$file"
        log "  probe  $suite/size.json"
        if have jq; then
            jq -r '.size.splits[]? | "         \(.config)/\(.split) num_rows=\(.num_rows)"' "$file"
        fi
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/size.json probe"
        FAILED=$((FAILED + 1))
    fi
    sleep "$SLEEP"
}

main() {
    if have b3sum; then
        digest_tool="b3sum (blake3)"
    else
        digest_tool="shasum -a 256 (sha256 fallback)"
    fi
    log "fetch_datasets.sh: out=$OUT page=$PAGE force=$FORCE"
    log "digest tool seen on this box (used by the manifest): $digest_tool"
    mkdir -p "$OUT"

    log ""
    log "[probe] typed_decisions /splits"
    probe_splits typed_decisions "LocalLLaMA%2Ftyped-decisions"
    log "[probe] massive_intent_en /splits"
    probe_splits massive_intent_en "mteb%2Famazon_massive_intent"
    log "[probe] prompt_injections /size"
    probe_size prompt_injections "deepset%2Fprompt-injections"

    # 1. typed_decisions — test: ALL rows; train: first 800 if the probe
    #    shows a train split for config all.
    log "[suite] typed_decisions LocalLLaMA/typed-decisions config=all split=test cap=all"
    fetch_suite typed_decisions "LocalLLaMA%2Ftyped-decisions" all test all
    if splits_have typed_decisions all train; then
        log "[suite] typed_decisions config=all split=train cap=800 (train present per probe)"
        fetch_suite typed_decisions "LocalLLaMA%2Ftyped-decisions" all train 800
    else
        log "[suite] typed_decisions config=all split=train SKIPPED (no train split for config all, or probe missing)"
    fi

    # 2. ag_news
    log "[suite] ag_news fancyzhx/ag_news config=default split=test cap=400"
    fetch_suite ag_news "fancyzhx%2Fag_news" default test 400
    log "[suite] ag_news config=default split=train cap=4000"
    fetch_suite ag_news "fancyzhx%2Fag_news" default train 4000

    # 3. emotion
    log "[suite] emotion dair-ai/emotion config=split split=test cap=400"
    fetch_suite emotion "dair-ai%2Femotion" split test 400
    log "[suite] emotion config=split split=train cap=4000"
    fetch_suite emotion "dair-ai%2Femotion" split train 4000

    # 4. sst5
    log "[suite] sst5 SetFit/sst5 config=default split=test cap=600"
    fetch_suite sst5 "SetFit%2Fsst5" default test 600
    log "[suite] sst5 config=default split=train cap=4000"
    fetch_suite sst5 "SetFit%2Fsst5" default train 4000

    # 5. banking77 — currently 404s on /rows (script-based dataset; see
    #    header). Attempted so every run records the gap honestly.
# banking77 gap resolution: PolyAI/banking77 is a script-based dataset that
# datasets-server cannot serve (/rows 404). The reference itself has a second
# variant on the parquet-backed mirror mteb/banking77 (bench_apps protocol:
# labels = sorted unique label_text, gold = key position) — fetch THAT.
    log "[suite] banking77 mteb/banking77 config=default split=test cap=all (universe rows; eval cap lives in the harness)"
    fetch_suite banking77 "mteb%2Fbanking77" default test all
    log "[suite] banking77 mteb/banking77 config=default split=train cap=4000"
    fetch_suite banking77 "mteb%2Fbanking77" default train 4000

    # 6. prompt_injections — test: ALL (~116); train: first 1000 (actual
    #    size 546 per /size probe, so the cap exhausts the split).
    log "[suite] prompt_injections deepset/prompt-injections config=default split=test cap=all"
    fetch_suite prompt_injections "deepset%2Fprompt-injections" default test all
    log "[suite] prompt_injections config=default split=train cap=1000 (actual 546 per probe)"
    fetch_suite prompt_injections "deepset%2Fprompt-injections" default train 1000

    # 7. massive_intent_en — config verified via /splits: the configs are
    #    bare locale codes; the English one is "en" (not "en-US").
    log "[suite] massive_intent_en mteb/amazon_massive_intent config=en split=test cap=all (universe rows; eval cap lives in the harness)"
    fetch_suite massive_intent_en "mteb%2Famazon_massive_intent" en test all
    log "[suite] massive_intent_en config=en split=train cap=4000"
    fetch_suite massive_intent_en "mteb%2Famazon_massive_intent" en train 4000

    # 8. xnli_en
    log "[suite] xnli_en facebook/xnli config=en split=test cap=300"
    fetch_suite xnli_en "facebook%2Fxnli" en test 300
    log "[suite] xnli_en config=en split=train cap=4000"
    fetch_suite xnli_en "facebook%2Fxnli" en train 4000

    log ""
    log "summary: files=$FILES rows=$ROWS failed_requests=$FAILED"
    if [ "$FAILED" -gt 0 ]; then
        log "FAILURES PRESENT — see FAIL lines above and .docs/dataset_manifest.md Gaps"
        exit 1
    fi
}

main "$@"
