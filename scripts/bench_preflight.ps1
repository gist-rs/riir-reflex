# bench_preflight.ps1 — the Windows sibling of bench_preflight.sh (Issue 065
# T3(b), owner call 2026-10-04: port the box-state probes to the 4090 runner
# so 4090-hosted runs stamp their own verdict).
#
# REFUSE to publish a latency number the box invalidates. Same law, same exit
# contract as the .sh: 0 = fit to bench (quote the PROVENANCE line in the
# record); 1 = refused, reason named; 2 = the instrument itself could not
# read the box (never a green zero).
#
# Platform notes (the honest divergences from the macOS gate):
#   - POWER SOURCE: the .NET PowerLineStatus enum (Online/Offline/Unknown) —
#     NOT Win32_Battery.BatteryStatus, which is a charge-LEVEL enum and
#     cannot name the source. A desktop (no battery) reads Online by
#     construction — sound.
#   - POWER MODE: the active power SCHEME (powercfg). "Power saver" refuses;
#     Balanced/High/Ultimate are fit arms (the macOS three-state law: only
#     the low arm refuses, the others are disclosed).
#   - LOAD: Windows has no 1-min loadavg cheaply. CIM LoadPercentage
#     (utilization %) is converted to loadavg-equivalent units
#     (pct x cores / 100) so the shared MAX_LOAD ceiling means the same
#     thing in runnable-thread terms. It is an instantaneous-ish reading —
#     run the gate at BOTH ends of a window, the way box_state.rs stamps.
#   - GPU: nvidia-smi utilization/memory is DISCLOSURE-ONLY. A lane server
#     (vLLM/uvicorn) is legitimately busy while its requests are being
#     timed — GPU utilization must never be a refusal axis here.
#   - CANARY: skipped — the .sh canary times a laya-METAL kernel, which does
#     not exist on this platform. There is no pinned Windows canary yet;
#     build one before quoting sub-ms numbers (ratchet-only, the .sh law).
#
# Usage (from the repo root on the Windows box):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\bench_preflight.ps1
#   $env:MAX_LOAD = '12'; scripts\bench_preflight.ps1     # raise the ceiling

$ErrorActionPreference = 'Continue'
$MAX_LOAD = if ($env:MAX_LOAD) { [double]$env:MAX_LOAD } else { 6.0 }

$fail = 0
function Note($m) { Write-Output $m }
function Refuse($m) { Write-Output "REFUSE - $m"; $script:fail = 1 }

# ---- 1. power source -------------------------------------------------------
try {
    Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
    $pls = [System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus
} catch {
    $pls = $null
}
if (-not $pls) {
    Note "UNK  powerline   unreadable (System.Windows.Forms unavailable) - the power axis is UNKNOWN, not fine"
    exit 2
} elseif ($pls -eq 'Online') {
    Note "ok   power       AC (PowerLineStatus=Online)"
    $power = 'AC'
} elseif ($pls -eq 'Offline') {
    Refuse "on BATTERY (PowerLineStatus=Offline) - plug in and re-run"
    $power = 'Battery'
} else {
    Note "UNK  powerline   PowerLineStatus=$pls - the power axis is UNKNOWN, not fine"
    exit 2
}

# ---- 2. power scheme -------------------------------------------------------
$scheme = (powercfg /getactivescheme) -join ' '
if (-not $scheme) {
    Note "warn scheme      unreadable (disclosed, not assumed)"
    $mode = '?'
} elseif ($scheme -match '\(([^)]+)\)\s*$') {
    $name = $Matches[1].Trim()
    switch ($name) {
        'Power saver'         { Refuse "Power saver scheme is ON - it caps clocks by design"; $mode = 'low' }
        'Balanced'            { Note "ok   scheme      Balanced (auto)"; $mode = 'auto' }
        'High performance'    { Note "ok   scheme      High performance"; $mode = 'high' }
        'Ultimate Performance'{ Note "ok   scheme      Ultimate Performance"; $mode = 'high' }
        default               { Note "UNK  scheme      unrecognised: $name"; $mode = "unknown($name)" }
    }
} else {
    # No friendly name (localized?) - the well-known GUID fallback.
    if     ($scheme -match 'e9a42b02') { Refuse "Power saver scheme (GUID e9a42b02) is ON"; $mode = 'low' }
    elseif ($scheme -match '381b4222') { Note "ok   scheme      Balanced (GUID fallback)"; $mode = 'auto' }
    elseif ($scheme -match '8c5e7fda') { Note "ok   scheme      High performance (GUID fallback)"; $mode = 'high' }
    else   { Note "UNK  scheme      unrecognised: $scheme"; $mode = '?' }
}

# ---- 3. load ---------------------------------------------------------------
$cores = [Environment]::ProcessorCount
$lp = $null
try {
    $lp = (Get-CimInstance Win32_Processor -ErrorAction Stop |
        Measure-Object -Property LoadPercentage -Average).Average
} catch {}
if ($null -eq $lp) {
    Note "UNK  load        unreadable"
    exit 2
} else {
    $load_eq = [math]::Round($lp * $cores / 100.0, 2)
    if ($load_eq -gt $MAX_LOAD) {
        Refuse "load-equivalent $load_eq (${lp}% x $cores cores / 100) exceeds MAX_LOAD=$MAX_LOAD - a sibling session is on the box"
    } else {
        Note "ok   load        ${lp}% x $cores = $load_eq loadavg-equivalent (ceiling $MAX_LOAD)"
    }
}

# ---- 4. memory pressure ----------------------------------------------------
$swap = $null
try {
    $pf = Get-CimInstance Win32_PageFileUsage -ErrorAction Stop | Select-Object -First 1
    if ($pf) { $swap = $pf.CurrentUsage }
} catch {}
if ($null -eq $swap) {
    Note "warn swap        unreadable (or no pagefile)"
} elseif ($swap -gt 4096) {
    # 4096MB, not the .sh's 1024: this box idles at ~1.4GB pagefile usage
    # measured 2026-10-04 (a 40GB allocation on a 64GB box) - a 1GB warn
    # would fire on every healthy run (the cries-wolf law).
    Note "warn swap used   ${swap}MB - paging can masquerade as a slow kernel"
} else {
    Note "ok   swap used   ${swap}MB"
}

# ---- 5. GPU (DISCLOSURE ONLY - see header) ---------------------------------
$gpu = $null
try { $gpu = (nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv,noheader) -join ' ' } catch {}
if ($gpu) {
    Note "info gpu         $gpu (disclosure only - a lane server is legitimately busy while serving timed requests)"
} else {
    Note "warn gpu         nvidia-smi unreadable"
}

# ---- 6. canary --------------------------------------------------------------
Note "info canary      SKIPPED - no pinned Windows canary (the .sh canary is laya-Metal-specific)"

# ---- provenance -------------------------------------------------------------
$canary_prov = 'skipped'
$prov = "power=$power scheme=$mode load=$load_eq swap=${swap}MB gpu=$gpu canary=$canary_prov"
Note ""
Note "PROVENANCE: $prov"

if ($fail -ne 0) {
    Note "X preflight REFUSED - do not publish a latency number from this box now"
    exit 1
}
Note "OK preflight PASSED - quote the PROVENANCE line in the bench record"
exit 0
