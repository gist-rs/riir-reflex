//! Box-state provenance for a harness run (Issue 021 T7).
//!
//! A latency number without its box state is not a measurement, and on a
//! laptop the POWER SOURCE is a first-order arm: every Issue 020 A/B was taken
//! on battery and nothing recorded it. `scripts/bench_preflight.sh` is the
//! refusal gate a human runs first (its Windows sibling is
//! `scripts/bench_preflight.ps1`); this module is the half that cannot be
//! forgotten — the state is stamped into `results.json`'s `meta` at the start
//! AND end of every run (load moves mid-run), with a verdict, so a table can
//! never again be read without it.
//!
//! Two platform arms, ONE normalized vocabulary (Issue 065 T3(b), the owner
//! call 2026-10-04): the macOS arm probes `pmset`/`sysctl`; the Windows arm
//! (the 4090 lane box) probes ONE PowerShell spawn — `PowerLineStatus` for
//! the power source (the .NET enum, NOT `Win32_Battery.BatteryStatus`, which
//! is a charge-LEVEL enum that cannot name the source), the active power
//! scheme via `powercfg`, CPU utilization via CIM, pagefile usage, and
//! `nvidia-smi` for GPU state (disclosure-only — a GPU lane server is
//! legitimately busy while its requests are being timed, so GPU utilization
//! must never be a refusal axis). Fields normalize into the SAME strings the
//! macOS arm emits ("AC Power"/"Battery Power"; "high"/"auto"/"low") so the
//! verdict arithmetic and every downstream consumer stay one vocabulary.
//!
//! ⚠ The Windows `load_1m` is NOT a 1-minute load average — Windows has none
//! cheaply. It is the CIM `LoadPercentage` (a short internal-window
//! utilization average) expressed in loadavg-equivalent units
//! (`pct × logical_cores / 100`), so the shared `MAX_LOAD` ceiling means the
//! same thing in runnable-thread terms; `load_source` discloses which
//! quantity the number is. Judge accordingly.
//!
//! Advisory, never refusing: a correctness-only run that does not care about
//! latency must not be blocked (T7's owner-gated question, resolved to the
//! recommended middle). Probes are subprocesses of stock OS tools; on a
//! platform where they are absent every field reads `None` and the verdict is
//! `None` — UNJUDGED, never a green.
//!
//! The parsers are pure so the thresholds are testable without a box; the
//! Windows fixtures below are pinned to REAL output captured from the 4090
//! lane box over ssh (2026-10-04, Windows PowerShell 5.1.26100.9444) — never
//! an invented shape.

use serde::Serialize;

/// Load ceiling above which a latency reading is not quotable. Mirrors
/// `bench_preflight.sh`'s `MAX_LOAD` default — change both together.
pub const MAX_LOAD: f64 = 6.0;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BoxState {
    /// `pmset -g ps` / .NET `PowerLineStatus`: "AC Power" / "Battery Power"
    /// (normalized across both platform arms).
    pub power: Option<String>,
    /// `pmset powermode` / Windows active scheme: "auto" | "low" | "high"
    /// (normalized; `unknown(...)` for anything else).
    pub power_mode: Option<String>,
    /// 1-min load average (macOS) or its loadavg-equivalent on Windows —
    /// see `load_source` and the module docs before quoting it.
    pub load_1m: Option<f64>,
    pub swap_used_mb: Option<f64>,
    /// Which quantity `load_1m` is: `"loadavg1m"` (macOS sysctl) or
    /// `"cpu_util_pct_x_cores/100"` (Windows CIM LoadPercentage). Absent on
    /// results that predate the field (read as the macOS quantity).
    pub load_source: Option<String>,
    /// NVIDIA GPU utilization % (`nvidia-smi`), Windows arm only —
    /// DISCLOSURE, never a refusal axis (a lane server is legitimately busy
    /// while serving the requests being timed).
    pub gpu_util_pct: Option<f64>,
    /// NVIDIA GPU memory in MiB — disclosure-only, same law as
    /// `gpu_util_pct`.
    pub gpu_mem_mib: Option<f64>,
    /// `Some(true)` = fit to quote latency; `Some(false)` = not (reasons
    /// below); `None` = the box could not be read — UNJUDGED.
    pub latency_quotable: Option<bool>,
    pub refusals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BoxStateSpan {
    pub start: BoxState,
    pub end: BoxState,
}

impl BoxStateSpan {
    /// Quotable only if BOTH ends are; unjudged if either end is.
    pub fn latency_quotable(&self) -> Option<bool> {
        match (self.start.latency_quotable, self.end.latency_quotable) {
            (Some(a), Some(b)) => Some(a && b),
            _ => None,
        }
    }
}

/// "Now drawing from 'AC Power'" → "AC Power".
pub fn parse_power_source(pmset_ps: &str) -> Option<String> {
    let rest = pmset_ps.split("drawing from '").nth(1)?;
    let name = rest.split('\'').next()?.trim();
    match name.is_empty() {
        true => None,
        false => Some(name.to_string()),
    }
}

/// ` powermode            2` (from `pmset -g`, the ACTIVE profile) → "high".
pub fn parse_power_mode(pmset_g: &str) -> Option<String> {
    let v = pmset_g
        .lines()
        .find_map(|l| l.trim().strip_prefix("powermode"))?
        .trim();
    let name = match v {
        "0" => "auto",
        "1" => "low",
        "2" => "high",
        other => return Some(format!("unknown({other})")),
    };
    Some(name.to_string())
}

/// `{ 6.41 16.10 18.31 }` → 6.41.
pub fn parse_loadavg(vm_loadavg: &str) -> Option<f64> {
    vm_loadavg
        .split_whitespace()
        .find(|t| *t != "{")?
        .parse()
        .ok()
}

/// `total = 4096.00M  used = 2811.94M  free = ...` → 2811.94.
pub fn parse_swap_used_mb(vm_swapusage: &str) -> Option<f64> {
    let rest = vm_swapusage.split("used =").nth(1)?;
    rest.split_whitespace().next()?.trim_end_matches('M').parse().ok()
}

/// The verdict, from parsed fields. Unreadable power → UNJUDGED.
pub fn judge(
    power: Option<&str>,
    power_mode: Option<&str>,
    load_1m: Option<f64>,
) -> (Option<bool>, Vec<String>) {
    let Some(power) = power else {
        return (None, vec!["power source unreadable — UNJUDGED".to_string()]);
    };
    let mut refusals = Vec::new();
    if power != "AC Power" {
        refusals.push(format!("on {power} — Apple Silicon sheds sustained GPU clock off AC"));
    }
    match power_mode {
        Some("low") => refusals.push("Low Power Mode (powermode=1) caps clocks".to_string()),
        Some(m) if m.starts_with("unknown") => refusals.push(format!("powermode {m}")),
        _ => {}
    }
    match load_1m {
        Some(l) if l > MAX_LOAD => {
            refusals.push(format!("load {l:.2} > {MAX_LOAD} — a sibling job is on the box"))
        }
        None => refusals.push("load unreadable".to_string()),
        _ => {}
    }
    (Some(refusals.is_empty()), refusals)
}

fn cmd(program: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(program).args(args).output().ok()?;
    match out.status.success() {
        true => Some(String::from_utf8_lossy(&out.stdout).into_owned()),
        false => None,
    }
}

/// Probe the live box. Five short subprocesses (~tens of ms); call twice per
/// run, never per case. Platform-dispatched (Issue 065 T3(b)): the Windows
/// arm is the 4090 lane box's — before it, every 4090-hosted cell read
/// UNJUDGED because `pmset` does not exist there.
pub fn capture() -> BoxState {
    match cfg!(windows) {
        true => capture_windows(),
        false => capture_macos(),
    }
}

fn capture_macos() -> BoxState {
    let power = cmd("pmset", &["-g", "ps"]).and_then(|s| parse_power_source(&s));
    let power_mode = cmd("pmset", &["-g"]).and_then(|s| parse_power_mode(&s));
    let load_1m = cmd("sysctl", &["-n", "vm.loadavg"]).and_then(|s| parse_loadavg(&s));
    let swap_used_mb = cmd("sysctl", &["-n", "vm.swapusage"]).and_then(|s| parse_swap_used_mb(&s));
    let (latency_quotable, refusals) = judge(power.as_deref(), power_mode.as_deref(), load_1m);
    BoxState {
        power,
        power_mode,
        load_1m,
        swap_used_mb,
        load_source: load_1m.map(|_| "loadavg1m".to_string()),
        gpu_util_pct: None,
        gpu_mem_mib: None,
        latency_quotable,
        refusals,
    }
}

/// The ONE PowerShell script the Windows arm runs (a single spawn: CIM
/// queries cost ~a second each, and `capture()` fires twice per run).
/// Emits `KEY=value` lines on stdout; stderr (CLIXML progress records) is
/// discarded by `Command::output`. Pinned against the 4090 lane box — see
/// the module docs.
const WINDOWS_PROBE_PS: &str = concat!(
    "$ErrorActionPreference='SilentlyContinue'\n",
    "Add-Type -AssemblyName System.Windows.Forms\n",
    "$pls=[System.Windows.Forms.SystemInformation]::PowerStatus.PowerLineStatus\n",
    "Write-Output ('POWERLINE='+$pls)\n",
    "Write-Output ('SCHEME='+((powercfg /getactivescheme) -join ' '))\n",
    "$lp=(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average\n",
    "if($null -ne $lp){Write-Output ('LOAD='+$lp)}else{Write-Output 'LOAD=NONE'}\n",
    "Write-Output ('CORES='+[Environment]::ProcessorCount)\n",
    "$pf=Get-CimInstance Win32_PageFileUsage | Select-Object -First 1\n",
    "if($pf){Write-Output ('SWAP='+$pf.CurrentUsage)}else{Write-Output 'SWAP=NONE'}\n",
    "$g=nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv,noheader\n",
    "if($g){Write-Output ('GPU='+($g -join ' '))}\n",
);

fn capture_windows() -> BoxState {
    let out = cmd("powershell", &["-NoProfile", "-Command", WINDOWS_PROBE_PS])
        .unwrap_or_default();
    let line = |key: &str| {
        out.lines().find_map(|l| l.trim().strip_prefix(key).map(str::to_string))
    };
    let power = line("POWERLINE=").and_then(|v| parse_win_powerline(&v));
    let power_mode = line("SCHEME=").and_then(|v| parse_win_scheme(&v));
    // LoadPercentage is utilization %; MAX_LOAD is a thread-count ceiling —
    // convert so the shared ceiling means the same thing (module docs).
    let load_1m = match (line("LOAD=").as_deref(), line("CORES=").as_deref()) {
        (Some(load), Some(cores)) => parse_win_load_eq(load, cores),
        _ => None,
    };
    let swap_used_mb = line("SWAP=").and_then(|v| parse_win_swap_mb(&v));
    let (gpu_util_pct, gpu_mem_mib) =
        line("GPU=").map(|v| parse_win_gpu(&v)).unwrap_or((None, None));
    let (latency_quotable, refusals) = judge(power.as_deref(), power_mode.as_deref(), load_1m);
    BoxState {
        power,
        power_mode,
        load_1m,
        swap_used_mb,
        load_source: load_1m.map(|_| "cpu_util_pct_x_cores/100".to_string()),
        gpu_util_pct,
        gpu_mem_mib,
        latency_quotable,
        refusals,
    }
}

/// `POWERLINE=Online` → "AC Power"; `Offline` → "Battery Power"; anything
/// else (incl. `Unknown`) → `None` (UNJUDGED — never guessed). The .NET
/// `PowerLineStatus` enum is the one clean source signal on Windows.
pub fn parse_win_powerline(powerline: &str) -> Option<String> {
    match powerline.trim() {
        "Online" => Some("AC Power".to_string()),
        "Offline" => Some("Battery Power".to_string()),
        _ => None,
    }
}

/// `Power Scheme GUID: 8c5e7fda-…  (High performance)` → "high". The
/// friendly name in the trailing parens is primary; the well-known scheme
/// GUID prefixes are the fallback when the name is localized away. "Power
/// saver" is the refusing class; Balanced/High/Ultimate are fit arms.
pub fn parse_win_scheme(scheme: &str) -> Option<String> {
    let name = scheme
        .contains('(')
        .then(|| {
            scheme
                .rsplit('(')
                .next()
                .map(|tail| tail.trim_end_matches(')').trim().to_string())
        })
        .flatten()
        .filter(|n| !n.is_empty());
    let by_guid = |g: &str| {
        scheme.contains(g).then_some(match g {
            "e9a42b02" => "low",       // Power saver
            "381b4222" => "auto",      // Balanced
            "8c5e7fda" | "ded574b5" => "high", // High performance / Ultimate
            _ => "auto",
        })
    };
    match name.as_deref() {
        Some("Power saver") => Some("low".to_string()),
        Some("Balanced") => Some("auto".to_string()),
        Some("High performance") | Some("Ultimate Performance") => Some("high".to_string()),
        Some(other) => Some(format!("unknown({other})")),
        None => {
            // No friendly name (localized?): fall back to the known GUIDs.
            ["e9a42b02", "381b4222", "8c5e7fda", "ded574b5"]
                .into_iter()
                .find_map(by_guid)
                .map(str::to_string)
        }
    }
}

/// `LOAD=15` + `CORES=24` → 15 × 24 / 100 = 3.6 (loadavg-equivalent; see
/// module docs). `NONE` or unparsable → `None`.
pub fn parse_win_load_eq(load_pct: &str, cores: &str) -> Option<f64> {
    let pct: f64 = load_pct.trim().parse().ok()?;
    let cores: f64 = cores.trim().parse().ok()?;
    (cores > 0.0).then(|| pct * cores / 100.0)
}

/// `SWAP=1443` (Win32_PageFileUsage CurrentUsage, MB) → 1443.0. `NONE` →
/// `None` (no pagefile is a legitimate posture, disclosed as absent).
pub fn parse_win_swap_mb(swap: &str) -> Option<f64> {
    swap.trim().parse().ok()
}

/// `GPU=19 %, 586 MiB` (nvidia-smi csv) → (19.0, 586.0). Either half may be
/// absent independently.
pub fn parse_win_gpu(gpu: &str) -> (Option<f64>, Option<f64>) {
    let util = gpu.split(',').next().and_then(|first| {
        first
            .trim()
            .trim_end_matches('%')
            .trim()
            .parse::<f64>()
            .ok()
    });
    let mem = gpu
        .split(',')
        .nth(1)
        .and_then(|second| {
            second
                .trim()
                .trim_end_matches("MiB")
                .trim()
                .parse::<f64>()
                .ok()
        });
    (util, mem)
}

/// One header line for TABLES.md.
pub fn render_line(span: &BoxStateSpan) -> String {
    let one = |b: &BoxState| {
        let base = format!(
            "power={} mode={} load={} swap={}M",
            b.power.as_deref().unwrap_or("?"),
            b.power_mode.as_deref().unwrap_or("?"),
            b.load_1m.map_or("?".to_string(), |l| format!("{l:.2}")),
            b.swap_used_mb.map_or("?".to_string(), |s| format!("{s:.0}")),
        );
        match (b.gpu_util_pct, b.gpu_mem_mib) {
            (Some(u), Some(m)) => format!("{base} gpu={u:.0}%/{m:.0}MiB"),
            _ => base,
        }
    };
    let verdict = match span.latency_quotable() {
        Some(true) => "latency QUOTABLE".to_string(),
        Some(false) => {
            let mut r: Vec<&str> = span.start.refusals.iter().map(String::as_str).collect();
            for x in &span.end.refusals {
                if !r.contains(&x.as_str()) {
                    r.push(x);
                }
            }
            format!("⛔ latency NOT QUOTABLE — {}", r.join("; "))
        }
        None => "⚠ box state UNJUDGED (probes unavailable) — latency columns carry no box state"
            .to_string(),
    };
    format!(
        "- box state (Issue 021): start {} · end {} — {}\n",
        one(&span.start),
        one(&span.end),
        verdict
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_live_macos_shapes() {
        let ps = "Now drawing from 'AC Power'\n -InternalBattery-0 (id=1)\t44%; charging;";
        assert_eq!(parse_power_source(ps).as_deref(), Some("AC Power"));
        assert_eq!(
            parse_power_source("Now drawing from 'Battery Power'\n").as_deref(),
            Some("Battery Power")
        );
        assert_eq!(parse_power_source("garbage"), None);
        let g = "System-wide power settings:\nCurrently in use:\n standby 1\n powermode            2\n";
        assert_eq!(parse_power_mode(g).as_deref(), Some("high"));
        assert_eq!(parse_power_mode(" powermode 1").as_deref(), Some("low"));
        assert_eq!(parse_power_mode(" powermode 0").as_deref(), Some("auto"));
        assert_eq!(parse_power_mode(" powermode 7").as_deref(), Some("unknown(7)"));
        assert_eq!(parse_power_mode("no such line"), None);
        assert_eq!(parse_loadavg("{ 6.41 16.10 18.31 }"), Some(6.41));
        assert_eq!(parse_loadavg(""), None);
        let sw = "total = 4096.00M  used = 2811.94M  free = 1284.06M  (encrypted)";
        assert_eq!(parse_swap_used_mb(sw), Some(2811.94));
    }

    #[test]
    fn parses_windows_shapes_captured_from_the_4090_lane_box() {
        // Verbatim from the 2026-10-04 ssh capture (Windows PowerShell
        // 5.1.26100.9444) — the fixture-pin law: never an invented shape.
        assert_eq!(parse_win_powerline("Online").as_deref(), Some("AC Power"));
        assert_eq!(parse_win_powerline("Offline").as_deref(), Some("Battery Power"));
        assert_eq!(parse_win_powerline("Unknown"), None);
        let scheme = "Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance)";
        assert_eq!(parse_win_scheme(scheme).as_deref(), Some("high"));
        assert_eq!(
            parse_win_scheme("Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)").as_deref(),
            Some("auto")
        );
        assert_eq!(
            parse_win_scheme("Power Scheme GUID: e9a42b02-d848-4be9-98ee-8c72e8edc14c  (Power saver)").as_deref(),
            Some("low")
        );
        // Localized away → the GUID fallback carries the verdict.
        assert_eq!(
            parse_win_scheme("Power Scheme GUID: e9a42b02-d848-4be9-98ee-8c72e8edc14c").as_deref(),
            Some("low")
        );
        assert_eq!(
            parse_win_scheme("Power Scheme GUID: deadbeef-0000 (Alien mode)").as_deref(),
            Some("unknown(Alien mode)")
        );
        // LOAD=15 + CORES=24 → 3.6 loadavg-equivalent.
        assert_eq!(parse_win_load_eq("15", "24"), Some(3.6));
        // 100% on 24 cores = 24.0 — the conversion saturates sanely.
        assert_eq!(parse_win_load_eq("100", "24"), Some(24.0));
        assert_eq!(parse_win_load_eq("NONE", "24"), None);
        assert_eq!(parse_win_load_eq("15", "NONE"), None);
        assert_eq!(parse_win_load_eq("15", "0"), None);
        assert_eq!(parse_win_swap_mb("1443"), Some(1443.0));
        assert_eq!(parse_win_swap_mb("NONE"), None);
        // GPU=19 %, 586 MiB (nvidia-smi csv) — the real captured line.
        assert_eq!(parse_win_gpu("19 %, 586 MiB"), (Some(19.0), Some(586.0)));
        assert_eq!(parse_win_gpu("0 %, 586 MiB"), (Some(0.0), Some(586.0)));
        assert_eq!(parse_win_gpu("19 %"), (Some(19.0), None));
        assert_eq!(parse_win_gpu("[N/A]"), (None, None));
    }

    #[test]
    fn windows_fields_feed_the_shared_judge_verdict() {
        // The normalized vocabulary means the SHARED judge decides Windows
        // readings — no platform-specific verdict path to drift.
        let power = parse_win_powerline("Online");
        let mode = parse_win_scheme(
            "Power Scheme GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (High performance)",
        );
        let load = parse_win_load_eq("15", "24");
        assert_eq!(judge(power.as_deref(), mode.as_deref(), load).0, Some(true));
        // A sibling compile at ~60% CPU on 24 cores → load_eq 14.4 > 6.0.
        let busy = parse_win_load_eq("60", "24");
        assert_eq!(judge(power.as_deref(), mode.as_deref(), busy).0, Some(false));
        // Power saver refuses; an unreadable powerline stays UNJUDGED.
        let saver = parse_win_scheme(
            "Power Scheme GUID: e9a42b02-d848-4be9-98ee-8c72e8edc14c  (Power saver)",
        );
        assert_eq!(judge(power.as_deref(), saver.as_deref(), load).0, Some(false));
        assert_eq!(judge(None, mode.as_deref(), load).0, None);
    }

    #[test]
    fn verdict_refuses_each_axis_and_only_those() {
        assert_eq!(judge(Some("AC Power"), Some("high"), Some(2.0)), (Some(true), vec![]));
        // powermode 2 (High) and 0 (Automatic) are both fit — the first
        // preflight read "not 0" as Low Power and could never pass on AC.
        assert_eq!(judge(Some("AC Power"), Some("auto"), Some(2.0)).0, Some(true));
        assert_eq!(judge(Some("Battery Power"), Some("auto"), Some(1.0)).0, Some(false));
        assert_eq!(judge(Some("AC Power"), Some("low"), Some(1.0)).0, Some(false));
        assert_eq!(judge(Some("AC Power"), Some("high"), Some(MAX_LOAD + 0.01)).0, Some(false));
        assert_eq!(judge(Some("AC Power"), Some("high"), Some(MAX_LOAD)).0, Some(true));
        assert_eq!(judge(Some("AC Power"), Some("high"), None).0, Some(false));
        // unreadable power is UNJUDGED, never a pass
        assert_eq!(judge(None, Some("high"), Some(1.0)).0, None);
        // two axes → two reasons, not one pooled
        assert_eq!(judge(Some("Battery Power"), Some("low"), Some(9.0)).1.len(), 3);
    }

    #[test]
    fn span_is_quotable_only_when_both_ends_are() {
        let ok = BoxState {
            power: Some("AC Power".into()),
            power_mode: Some("high".into()),
            load_1m: Some(1.0),
            swap_used_mb: Some(0.0),
            load_source: Some("loadavg1m".into()),
            gpu_util_pct: None,
            gpu_mem_mib: None,
            latency_quotable: Some(true),
            refusals: vec![],
        };
        let bad = BoxState {
            latency_quotable: Some(false),
            refusals: vec!["on Battery Power".into()],
            ..ok.clone()
        };
        let unk = BoxState {
            latency_quotable: None,
            ..ok.clone()
        };
        let s = |a: &BoxState, b: &BoxState| BoxStateSpan {
            start: a.clone(),
            end: b.clone(),
        };
        assert_eq!(s(&ok, &ok).latency_quotable(), Some(true));
        assert_eq!(s(&ok, &bad).latency_quotable(), Some(false));
        assert_eq!(s(&bad, &ok).latency_quotable(), Some(false));
        assert_eq!(s(&ok, &unk).latency_quotable(), None);
        assert!(render_line(&s(&ok, &bad)).contains("NOT QUOTABLE — on Battery Power"));
        assert!(render_line(&s(&ok, &ok)).contains("latency QUOTABLE"));
        // The Windows gpu disclosure rides the line without changing the
        // macOS shape (absent gpu → identical prefix).
        let mut nvidia = ok.clone();
        nvidia.gpu_util_pct = Some(19.0);
        nvidia.gpu_mem_mib = Some(586.0);
        assert!(render_line(&s(&nvidia, &nvidia)).contains("gpu=19%/586MiB"));
        assert!(!render_line(&s(&ok, &ok)).contains("gpu="));
    }
}
