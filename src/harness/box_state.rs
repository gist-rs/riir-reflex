//! Box-state provenance for a harness run (Issue 021 T7).
//!
//! A latency number without its box state is not a measurement, and on a
//! laptop the POWER SOURCE is a first-order arm: every Issue 020 A/B was taken
//! on battery and nothing recorded it. `scripts/bench_preflight.sh` is the
//! refusal gate a human runs first; this module is the half that cannot be
//! forgotten — the state is stamped into `results.json`'s `meta` at the start
//! AND end of every run (load moves mid-run), with a verdict, so a table can
//! never again be read without it.
//!
//! Advisory, never refusing: a correctness-only run that does not care about
//! latency must not be blocked (T7's owner-gated question, resolved to the
//! recommended middle). Probes are subprocesses of stock macOS tools; on a
//! platform where they are absent every field reads `None` and the verdict is
//! `None` — UNJUDGED, never a green.
//!
//! The parsers are pure so the thresholds are testable without a box.

use serde::Serialize;

/// Load ceiling above which a latency reading is not quotable. Mirrors
/// `bench_preflight.sh`'s `MAX_LOAD` default — change both together.
pub const MAX_LOAD: f64 = 6.0;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BoxState {
    /// `pmset -g ps` source: "AC Power" / "Battery Power" / "UPS Power".
    pub power: Option<String>,
    /// `pmset powermode`: 0 = auto, 1 = low, 2 = high (3-state, not a bool).
    pub power_mode: Option<String>,
    pub load_1m: Option<f64>,
    pub swap_used_mb: Option<f64>,
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
/// run, never per case.
pub fn capture() -> BoxState {
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
        latency_quotable,
        refusals,
    }
}

/// One header line for TABLES.md.
pub fn render_line(span: &BoxStateSpan) -> String {
    let one = |b: &BoxState| {
        format!(
            "power={} mode={} load={} swap={}M",
            b.power.as_deref().unwrap_or("?"),
            b.power_mode.as_deref().unwrap_or("?"),
            b.load_1m.map_or("?".to_string(), |l| format!("{l:.2}")),
            b.swap_used_mb.map_or("?".to_string(), |s| format!("{s:.0}")),
        )
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
    }
}
