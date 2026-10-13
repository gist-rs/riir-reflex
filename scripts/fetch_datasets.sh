#!/bin/sh
# fetch_datasets.sh — riir-reflex Plan 603 T1.5 benchmark dataset fetch layer.
#
# Pulls every benchmark suite's rows from the HF datasets-server /rows API into
# .raw/datasets/<suite>/<split>-<NNN>.json — one file per 100-row page, offset
# paging until a page returns fewer than 100 rows or the suite's row cap is
# hit. Companion probes (/splits, /size) land beside the pages as splits.json
# / size.json, and the hub licence probe (riir-rethink Issue 023 T1) lands as
# license.json — the drift tripwire beside the manifest's VERIFIED licence
# table. .raw/ is gitignored; re-running regenerates the files (the
# committed manifest .docs/02_protocols/dataset_manifest.md records their blake3 digests,
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
# OUT / TRAIN_CAP overrides (issue 038 T2): a larger train pull goes to a
# SEPARATE dir so the canonical .raw/datasets (the manifest-digested set
# every published table reads) is never rewritten under a sibling run.
OUT="${OUT:-$ROOT/.raw/datasets}"
TRAIN_CAP="${TRAIN_CAP:-20000}"
# Issue 058 (verdict condition 1, 2026-10-01): the default OUT is the
# canonical .raw/datasets, which is now the FULL-pull pool — a 4000-row
# default here was the root cause of the silent board regression (a fresh
# fetch rebuilt 4k-row pools under the same dir name and every number
# moved). The default now matches the t20k re-baseline scale; the
# slice-integrity gate's per-suite pool floors (MIN_POOL_ROWS) refuse a
# shrunken pull at run time regardless.
# SUITES (issue 041): comma-separated subset filter, e.g.
#   SUITES=ag_news OUT=$ROOT/.raw/datasets_agnews_full TRAIN_CAP=120000 scripts/fetch_datasets.sh
# Default "all" = the birth behavior (every suite's blocks run). The
# probes at the top of main() honor the filter too — a filtered run
# spends its politeness budget only on the named suites.
SUITES="${SUITES:-all}"
MAX_PARTS=$(( TRAIN_CAP / 100 > 200 ? TRAIN_CAP / 100 + 1 : 200 ))   # hard stop: never below the 200-page birth guard

FILES=0
ROWS=0
FAILED=0

log() { printf '%s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

# True when SUITES is "all" or names <suite> in its comma list.
want_suite() {
    [ "$SUITES" = "all" ] && return 0
    case ",$SUITES," in *",$1,"*) return 0 ;; esac
    return 1
}

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
#             <urlencoded-split> <cap: number | "all"> [<as-split>]
#
# <as-split> (Plan 012): the FILENAME prefix, when the stored split must
# differ from the fetched one — mmlupro_jdi has no train split, so its
# 70-row validation pool is stored as the suite's `train` (the corpus/cal
# pool the harness reads), and anli_jdi's train_r1 rounds are stored as
# `train`. Defaults to <split> (every pre-existing caller is unchanged).
fetch_suite() {
    suite=$1; ds=$2; cfg=$3; split=$4; cap=$5
    as_split="${6:-$split}"
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
        file="$dir/$as_split-$pagenum.json"
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
                log "  skip   $suite/$as_split-$pagenum ($n rows)"
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
            log "  FAIL   $suite/$as_split-$pagenum (http error, offset=$offset)"
            FAILED=$((FAILED + 1))
            suite_failed=1
            break
        fi
        if ! rows_valid "$tmp"; then
            rm -f "$tmp"
            log "  FAIL   $suite/$as_split-$pagenum (JSON error body, offset=$offset)"
            FAILED=$((FAILED + 1))
            suite_failed=1
            break
        fi
        n=$(rows_count "$tmp" 2>/dev/null || echo 0)
        if [ "$n" -eq 0 ]; then
            rm -f "$tmp"
            log "  done   $suite/$as_split (no more rows at offset=$offset)"
            break
        fi
        mv "$tmp" "$file"
        FILES=$((FILES + 1))
        ROWS=$((ROWS + n))
        total=$((total + n))
        files=$((files + 1))
        log "  fetch  $suite/$as_split-$pagenum rows=$n offset=$offset"
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
        log "  FAIL   $suite/$as_split hit MAX_PARTS=$MAX_PARTS guard"
        FAILED=$((FAILED + 1))
        suite_failed=1
    fi
    rows_total=$(jq -r '.num_rows_total // "?"' "$dir/$as_split-000.json" 2>/dev/null || echo "?")
    if [ "$suite_failed" -eq 0 ]; then
        log "  ==     $suite/$as_split: $total rows in $files file(s) (dataset total: $rows_total)"
    else
        log "  ==     $suite/$as_split: $total rows in $files file(s) BEFORE FAILURE (dataset total: $rows_total)"
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

# probe_license <suite> <urlencoded-dataset> — save the HUB metadata's
# licence signal (riir-rethink Issue 023 T1: the licence ledger's fetch-side
# tripwire). The hub API carries the licence as a TAG (`license:xxx`) for
# most datasets and as the top-level `license` field for some; the probe
# records BOTH plus the cardData license when present. The VERIFIED table
# in .docs/02_protocols/dataset_manifest.md §Licences stays AUTHORITATIVE
# (a card saying only `license:other` — e.g. dair-ai/emotion's
# research-only terms — is resolved by the human read, never guessed);
# this file exists so a hub-side licence CHANGE surfaces at refetch time
# (diff the probe against the manifest row).
HUB="https://huggingface.co/api/datasets"
probe_license() {
    suite=$1; ds=$2
    dir="$OUT/$suite"
    mkdir -p "$dir"
    file="$dir/license.json"
    if [ "$FORCE" != "1" ] && [ -s "$file" ]; then
        log "  skip   $suite/license.json (cached)"
        return 0
    fi
    url="$HUB/$ds"
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
        log "  probe  $suite/license.json"
        if have jq; then
            lic=$(jq -r '([.license] + [.tags[]? | select(startswith("license:"))]) | map(select(. != null and . != "")) | join(", ")' "$file" 2>/dev/null)
            log "         licence signal: ${lic:-none — hub tags absent, the manifest licence table is authoritative}"
        fi
    else
        rm -f "$file.tmp"
        log "  FAIL   $suite/license.json probe"
        FAILED=$((FAILED + 1))
    fi
    sleep "$SLEEP"
}

# dedupe_train <suite> — the Issue-058 slice-integrity law at the fetch
# plane: a train pull is de-duplicated against itself and against the test
# split at EXACT-row identity, dropping only TRAIN-side copies (the test
# population is the eval surface and is never rewritten). The harness's
# slice_guard refuses a suite whose pool∩test or pool∩cal rows are not
# distinct, so a fetch that skips this law produces a pull the run gate
# refuses — the disclosure here is the cheap half. jq-path only: without
# jq the pass SKIPS LOUD (the 4090 syncs data by copying from this box —
# the 077 procedure — so the jq box is the producer of clean pulls).
dedupe_train() {
    suite=$1
    dir="$OUT/$suite"
    if ! have jq; then
        log "  dedupe $suite/train SKIPPED (no jq on this box — sync data from a jq host)"
        return 0
    fi
    [ -d "$dir" ] || return 0
    ls "$dir"/test-*.json >/dev/null 2>&1 || return 0
    keys=$(mktemp)          # newline-separated JSON strings (the seen set)
    kjson=$(mktemp)         # the same set as one JSON array (jq --slurpfile)
    for f in "$dir"/test-*.json; do
        jq -c '.rows[] | (.row | to_entries | sort_by(.key) | from_entries | tojson)' "$f" >> "$keys"
    done
    jq -s . "$keys" > "$kjson"
    removed=0
    for f in "$dir"/train-*.json; do
        [ -s "$f" ] || continue
        n=$(jq --slurpfile seen "$kjson" '
            [.rows[]
             | (.row | to_entries | sort_by(.key) | from_entries | tojson) as $k
             | select($seen[0] | any(. == $k))] | length' "$f") || n=0
        if [ "${n:-0}" -gt 0 ]; then
            jq --slurpfile seen "$kjson" '
                .rows |= [.[] | select(
                    (.row | to_entries | sort_by(.key) | from_entries | tojson) as $k
                    | ($seen[0] | any(. == $k)) | not)]' "$f" > "$f.tmp" \
                && mv "$f.tmp" "$f"
            removed=$((removed + n))
            log "  dedupe $suite/$(basename "$f"): dropped $n exact-duplicate row(s)"
        fi
        # Extend the seen-set with this page's survivors so later pages
        # drop internal duplicates too.
        jq -c '.rows[] | (.row | to_entries | sort_by(.key) | from_entries | tojson)' "$f" >> "$keys"
        jq -s . "$keys" > "$kjson"
    done
    rm -f "$keys" "$kjson"
    if [ "$removed" -gt 0 ]; then
        log "  dedupe $suite/train: $removed exact-duplicate row(s) dropped (self + test overlap)"
    fi
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
    log "fetch_datasets.sh: out=$OUT page=$PAGE force=$FORCE suites=$SUITES"
    log "digest tool seen on this box (used by the manifest): $digest_tool"
    mkdir -p "$OUT"

    if want_suite typed_decisions; then
        log "[probe] typed_decisions /splits"
        probe_splits typed_decisions "LocalLLaMA%2Ftyped-decisions"
    fi
    if want_suite massive_intent_en; then
        log "[probe] massive_intent_en /splits"
        probe_splits massive_intent_en "mteb%2Famazon_massive_intent"
    fi
    if want_suite prompt_injections; then
        log "[probe] prompt_injections /size"
        probe_size prompt_injections "deepset%2Fprompt-injections"
    fi
    if want_suite thai_wisesight; then
        log "[probe] thai_wisesight /splits"
        probe_splits thai_wisesight "pythainlp%2Fwisesight_sentiment"
    fi
    if want_suite thai_sib200; then
        log "[probe] thai_sib200 /splits"
        probe_splits thai_sib200 "Davlan%2Fsib200"
    fi
    if want_suite wanli_en; then
        log "[probe] wanli_en /splits"
        probe_splits wanli_en "alisawuffles%2FWANLI"
    fi
    if want_suite winogrande_jdi; then
        log "[probe] winogrande_jdi /splits"
        probe_splits winogrande_jdi "allenai%2Fwinogrande"
    fi
    if want_suite hellaswag_jdi; then
        log "[probe] hellaswag_jdi /splits"
        probe_splits hellaswag_jdi "Rowan%2Fhellaswag"
    fi
    if want_suite anli_jdi; then
        log "[probe] anli_jdi /splits"
        probe_splits anli_jdi "facebook%2Fanli"
    fi
    if want_suite mmlupro_jdi; then
        log "[probe] mmlupro_jdi /splits"
        probe_splits mmlupro_jdi "TIGER-Lab%2FMMLU-Pro"
    fi

    # Licence probes (riir-rethink Issue 023 T1): one hub call per selected
    # suite — the drift tripwire beside the manifest's VERIFIED table.
    # NOTE: the hub API takes the dataset id with a LITERAL slash (unlike
    # the datasets-server endpoints, which take %2F).
    if want_suite typed_decisions; then
        probe_license typed_decisions "LocalLLaMA/typed-decisions"
    fi
    if want_suite ag_news; then
        probe_license ag_news "fancyzhx/ag_news"
    fi
    if want_suite emotion; then
        probe_license emotion "dair-ai/emotion"
    fi
    if want_suite sst5; then
        probe_license sst5 "SetFit/sst5"
    fi
    if want_suite banking77; then
        probe_license banking77 "mteb/banking77"
    fi
    if want_suite prompt_injections; then
        probe_license prompt_injections "deepset/prompt-injections"
    fi
    if want_suite massive_intent_en; then
        probe_license massive_intent_en "mteb/amazon_massive_intent"
    fi
    if want_suite xnli_en; then
        probe_license xnli_en "facebook/xnli"
    fi
    if want_suite thai_wisesight; then
        probe_license thai_wisesight "pythainlp/wisesight_sentiment"
    fi
    if want_suite thai_sib200; then
        probe_license thai_sib200 "Davlan/sib200"
    fi
    if want_suite wanli_en; then
        probe_license wanli_en "alisawuffles/WANLI"
    fi
    # Plan 012 (the JDI panel tranche 1): the four board rows' licence
    # probes — the hub-side tripwire beside the manifest's VERIFIED rows.
    if want_suite winogrande_jdi; then
        probe_license winogrande_jdi "allenai/winogrande"
    fi
    if want_suite hellaswag_jdi; then
        probe_license hellaswag_jdi "Rowan/hellaswag"
    fi
    if want_suite anli_jdi; then
        probe_license anli_jdi "facebook/anli"
    fi
    if want_suite mmlupro_jdi; then
        probe_license mmlupro_jdi "TIGER-Lab/MMLU-Pro"
    fi

    # 1. typed_decisions — test: ALL rows; train: ALL 1200 rows (Issue 052:
    #    the birth cap 800 stopped inside the security_incidents train block
    #    (offsets 900–1199) + 100 invoice rows; the extended pool measures
    #    A0' 0.5725 vs the capped A0 0.4655 at the published posture, Bench
    #    078. The cal slice is the STRATIFIED round-robin front over the whole
    #    split (Issue 039 T2) — it re-derives on the wider pool; the test
    #    split is untouched and the published row is re-measured).
    if want_suite typed_decisions; then
        log "[suite] typed_decisions LocalLLaMA/typed-decisions config=all split=test cap=all"
        fetch_suite typed_decisions "LocalLLaMA%2Ftyped-decisions" all test all
        if splits_have typed_decisions all train; then
            log "[suite] typed_decisions config=all split=train cap=1200 (train present per probe)"
            fetch_suite typed_decisions "LocalLLaMA%2Ftyped-decisions" all train 1200
            dedupe_train typed_decisions
        else
            log "[suite] typed_decisions config=all split=train SKIPPED (no train split for config all, or probe missing)"
        fi
    fi

    # 2. ag_news
    if want_suite ag_news; then
        log "[suite] ag_news fancyzhx/ag_news config=default split=test cap=400"
        fetch_suite ag_news "fancyzhx%2Fag_news" default test 400
        log "[suite] ag_news config=default split=train cap=$TRAIN_CAP"
        fetch_suite ag_news "fancyzhx%2Fag_news" default train "$TRAIN_CAP"
        dedupe_train ag_news
    fi

    # 3. emotion
    if want_suite emotion; then
        log "[suite] emotion dair-ai/emotion config=split split=test cap=400"
        fetch_suite emotion "dair-ai%2Femotion" split test 400
        log "[suite] emotion config=split split=train cap=4000"
        fetch_suite emotion "dair-ai%2Femotion" split train "$TRAIN_CAP"
        dedupe_train emotion
    fi

    # 4. sst5
    if want_suite sst5; then
        log "[suite] sst5 SetFit/sst5 config=default split=test cap=600"
        fetch_suite sst5 "SetFit%2Fsst5" default test 600
        log "[suite] sst5 config=default split=train cap=4000"
        fetch_suite sst5 "SetFit%2Fsst5" default train "$TRAIN_CAP"
        dedupe_train sst5
    fi

    # 5. banking77 — currently 404s on /rows (script-based dataset; see
    #    header). Attempted so every run records the gap honestly.
# banking77 gap resolution: PolyAI/banking77 is a script-based dataset that
# datasets-server cannot serve (/rows 404). The reference itself has a second
# variant on the parquet-backed mirror mteb/banking77 (bench_apps protocol:
# labels = sorted unique label_text, gold = key position) — fetch THAT.
    if want_suite banking77; then
        log "[suite] banking77 mteb/banking77 config=default split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite banking77 "mteb%2Fbanking77" default test all
        log "[suite] banking77 mteb/banking77 config=default split=train cap=4000"
        fetch_suite banking77 "mteb%2Fbanking77" default train "$TRAIN_CAP"
        dedupe_train banking77
    fi

    # 6. prompt_injections — test: ALL (~116); train: first 1000 (actual
    #    size 546 per /size probe, so the cap exhausts the split).
    if want_suite prompt_injections; then
        log "[suite] prompt_injections deepset/prompt-injections config=default split=test cap=all"
        fetch_suite prompt_injections "deepset%2Fprompt-injections" default test all
        log "[suite] prompt_injections config=default split=train cap=1000 (actual 546 per probe)"
        fetch_suite prompt_injections "deepset%2Fprompt-injections" default train 1000
        dedupe_train prompt_injections
    fi

    # 7. massive_intent_en — config verified via /splits: the configs are
    #    bare locale codes; the English one is "en" (not "en-US").
    if want_suite massive_intent_en; then
        log "[suite] massive_intent_en mteb/amazon_massive_intent config=en split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite massive_intent_en "mteb%2Famazon_massive_intent" en test all
        log "[suite] massive_intent_en config=en split=train cap=4000"
        fetch_suite massive_intent_en "mteb%2Famazon_massive_intent" en train "$TRAIN_CAP"
        dedupe_train massive_intent_en
    fi

    # 8. xnli_en
    if want_suite xnli_en; then
        log "[suite] xnli_en facebook/xnli config=en split=test cap=300"
        fetch_suite xnli_en "facebook%2Fxnli" en test 300
        log "[suite] xnli_en config=en split=train cap=$TRAIN_CAP"
        fetch_suite xnli_en "facebook%2Fxnli" en train "$TRAIN_CAP"
        dedupe_train xnli_en
        # Issue 047 R1: the M1 reopen lane's confirmation surface — the
        # full validation split (~2490 rows), fetched once into the SAME
        # suite dir and read ONCE by `--nli-m1 --suites xnli_en_val`.
        log "[suite] xnli_en config=en split=validation cap=all (issue 047 confirmation surface)"
        fetch_suite xnli_en "facebook%2Fxnli" en validation all
    fi

    # 9. thai_wisesight — Plan 003 T3.1: the OpenThai board's WEAKEST
    #    published set (51.6 / ECE 0.353 — chosen for honesty, not
    #    cherry-picking). 4-class sentiment choice; the ClassLabel order
    #    pos/neu/neg/q is verified at fetch and pinned in
    #    dataset_manifest.md (the ag_news VERIFY-AT-PORT law).
    if want_suite thai_wisesight; then
        log "[suite] thai_wisesight pythainlp/wisesight_sentiment config=wisesight_sentiment split=test cap=400"
        fetch_suite thai_wisesight "pythainlp%2Fwisesight_sentiment" wisesight_sentiment test 400
        log "[suite] thai_wisesight config=wisesight_sentiment split=train cap=$TRAIN_CAP"
        fetch_suite thai_wisesight "pythainlp%2Fwisesight_sentiment" wisesight_sentiment train "$TRAIN_CAP"
        dedupe_train thai_wisesight
    fi

    # 10. thai_sib200 — Plan 003 T3.1: the SIB-200 Thai topical probe
    #     (7-way topic), test + train fetched WHOLE (204 / 701 rows).
    if want_suite thai_sib200; then
        log "[suite] thai_sib200 Davlan/sib200 config=tha_Thai split=train cap=all (701 rows whole-set)"
        fetch_suite thai_sib200 "Davlan%2Fsib200" tha_Thai train all
        dedupe_train thai_sib200
    fi

    # 11. wanli_en — the ESC re-source NLI lane (riir-rethink Issue 024 /
    #     Research 002; reflex issue 080): WANLI re-enters the xnli_en niche
    #     at the same 3-choice wire (premise/hypothesis/gold, gold a STRING
    #     in entailment/neutral/contradiction — no ClassLabel order to
    #     verify at the port, the builder maps the string through XNLI_KEYS).
    #     Test fetched WHOLE (5,000 rows; the eval cap lives in the harness,
    #     the banking77 posture), train capped at $TRAIN_CAP (the slice
    #     floor 16_000 sits on this pull).
    if want_suite wanli_en; then
        log "[suite] wanli_en alisawuffles/WANLI config=default split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite wanli_en "alisawuffles%2FWANLI" default test all
        log "[suite] wanli_en config=default split=train cap=$TRAIN_CAP"
        fetch_suite wanli_en "alisawuffles%2FWANLI" default train "$TRAIN_CAP"
        dedupe_train wanli_en
    fi

    # 12. winogrande_jdi — Plan 012 A1 (the JDI panel tranche 1): the
    #     pronoun-resolution board row (validation 1267 = the board's cases;
    #     HF `test` is unlabeled). Train fetched for the corpus pool.
    if want_suite winogrande_jdi; then
        log "[suite] winogrande_jdi allenai/winogrande config=winogrande_debiased split=validation cap=all (the board's population; eval cap lives in the harness)"
        fetch_suite winogrande_jdi "allenai%2Fwinogrande" winogrande_debiased validation all
        log "[suite] winogrande_jdi config=winogrande_debiased split=train cap=4000 (the corpus pool)"
        fetch_suite winogrande_jdi "allenai%2Fwinogrande" winogrande_debiased train 4000
    fi

    # 13. hellaswag_jdi — Plan 012 A2: the 4-way continuation board row
    #     (validation 10042). Eval universe fetched at 8000 (the eval cap
    #     lives in the harness — the banking77 posture); train at the same
    #     bound (the corpus pool needs no more).
    if want_suite hellaswag_jdi; then
        log "[suite] hellaswag_jdi Rowan/hellaswag config=default split=validation cap=8000 (eval universe; eval cap lives in the harness)"
        fetch_suite hellaswag_jdi "Rowan%2Fhellaswag" default validation 8000
        log "[suite] hellaswag_jdi config=default split=train cap=8000 (the corpus pool)"
        fetch_suite hellaswag_jdi "Rowan%2Fhellaswag" default train 8000
    fi

    # 14. anli_jdi — Plan 012 A3: the NLI board row = test_r1+test_r2+test_r3
    #     concatenated in that order (3200 = the board's cases); the harness
    #     reads the comma-list eval split (load_rows). Train rounds: train_r1
    #     ONLY (r2/r3 trains are huge; one round's pool is enough for the
    #     corpus and keeps the fetch bounded) — stored as the suite's `train`.
    if want_suite anli_jdi; then
        log "[suite] anli_jdi facebook/anli config=plain_text split=test_r1 cap=all (1000 rows)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text test_r1 all
        log "[suite] anli_jdi config=plain_text split=test_r2 cap=all (1000 rows)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text test_r2 all
        log "[suite] anli_jdi config=plain_text split=test_r3 cap=all (1200 rows)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text test_r3 all
        log "[suite] anli_jdi config=plain_text split=train_r1 cap=4000 (round-1 pool only; stored as the suite's train)"
        fetch_suite anli_jdi "facebook%2Fanli" plain_text train_r1 4000 train
    fi

    # 15. mmlupro_jdi — Plan 012 A4: the knowledge board row (test 12032;
    #     universe rows, the eval cap lives in the harness). NO train split
    #     exists — the 70-row validation split (the few-shot prompt pool) is
    #     fetched AS the suite's `train` (the corpus/cal pool; the starved
    #     corpus is disclosed in the registry + the manifest).
    if want_suite mmlupro_jdi; then
        log "[suite] mmlupro_jdi TIGER-Lab/MMLU-Pro config=default split=test cap=all (universe rows; eval cap lives in the harness)"
        fetch_suite mmlupro_jdi "TIGER-Lab%2FMMLU-Pro" default test all
        log "[suite] mmlupro_jdi config=default split=validation cap=all (70 rows — no train split exists; stored AS the suite's train = the corpus/cal pool)"
        fetch_suite mmlupro_jdi "TIGER-Lab%2FMMLU-Pro" default validation all train
    fi

    log ""
    log "summary: files=$FILES rows=$ROWS failed_requests=$FAILED"
    if [ "$FAILED" -gt 0 ]; then
        log "FAILURES PRESENT — see FAIL lines above and .docs/02_protocols/dataset_manifest.md Gaps"
        exit 1
    fi
}

main "$@"
