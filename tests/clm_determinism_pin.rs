//! Issue 027 amendment 1 — the CLM determinism pin (the 4090-window form).
//!
//! A comparison lane that cannot repeat byte-identically produces no
//! cells: every number the harness publishes over `/v1/systemone` rests
//! on the same request answering with the same BYTES. The pin sends ONE
//! fixed mixed-kind request (choice + score + noul — every question kind
//! the harness serves) N times and requires:
//!
//!   1. N byte-identical raw HTTP response bodies;
//!   2. the trained head file's mtime UNCHANGED across the run (a
//!      hot-reload path that rewrites its own checkpoint would be a
//!      provenance hole — the measured artifact must be the mounted one).
//!
//! UNSEEN, never a pass: without `CLM_SERVE_URL` (their stack serving in
//! the 4090 window — `scripts/clm_serve_4090.sh start`) this prints a
//! loud skip and exits 0, the `slice_leak_oracle` law. A served stack
//! that wobbles fails hard.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, SystemTime};

use riir_reflex::lanes::clm::{build_body, SERVE_URL_ENV};

/// The fixed request: one case-shaped state + one question of every kind,
/// the same shapes the harness serves (their rendering law applies inside
/// [`build_body`]).
fn fixed_request() -> katgpt_core::decision_wire::DecisionRequest {
    use katgpt_core::decision_wire::{DecisionRequest, Question};
    DecisionRequest {
        state: "article: The regulator opened a probe into the exchange's \
                listing disclosures.\n\nprior: two earlier probes closed without findings"
            .to_string(),
        questions: vec![
            Question::choice(
                "topic",
                "What is the topic of `article`?",
                vec![
                    "world news and international politics".to_string(),
                    "business and economy".to_string(),
                    "science and technology".to_string(),
                ],
                None,
            ),
            Question::score(
                "severity",
                "How severe is the disclosure lapse?",
                vec![
                    "routine paperwork".to_string(),
                    "material omission".to_string(),
                    "systemic fraud".to_string(),
                ],
            ),
            Question::noul("holds", "The prior probes were closed WITH findings."),
        ],
    }
}

/// One raw HTTP/1.1 POST returning the response BODY bytes (the lane's
/// own `post` posture — std only; we need the raw bytes, not the mapped
/// response).
fn raw_post(url: &str, body: &str) -> Result<Vec<u8>, String> {
    let rest = url
        .strip_prefix("http://")
        .unwrap_or_else(|| url.strip_prefix("https://").unwrap_or(url));
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().map_err(|_| "bad port")?),
        None => (authority.to_string(), 80),
    };
    let mut stream =
        TcpStream::connect((host.as_str(), port)).map_err(|e| format!("connect: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(|e| e.to_string())?;
    let head = format!(
        "POST /v1/systemone HTTP/1.1\r\nHost: {host}:{port}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body.as_bytes()))
        .and_then(|_| stream.flush())
        .map_err(|e| format!("write: {e}"))?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| format!("read: {e}"))?;
    let text = String::from_utf8_lossy(&raw);
    let (headers, body) = text
        .split_once("\r\n\r\n")
        .ok_or("no header/body separator")?;
    let status: u16 = headers
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .ok_or("malformed status line")?;
    if status != 200 {
        return Err(format!("HTTP {status}: {}", &body[..body.len().min(300)]));
    }
    let mut body_bytes = body.as_bytes().to_vec();
    for line in headers.lines().skip(1) {
        if let Some((k, v)) = line.split_once(':')
            && k.trim().eq_ignore_ascii_case("content-length")
            && let Ok(n) = v.trim().parse::<usize>()
        {
            body_bytes.truncate(n);
        }
    }
    Ok(body_bytes)
}

fn head_mtime() -> Option<SystemTime> {
    // The mounted reference head — the artifact the server must be
    // measuring (.raw is gitignored local state; absence skips the mtime
    // arm LOUD, it does not fail it).
    Path::new(".raw/CLM/checkpoints/CLM_v0.1-8B.pt")
        .metadata()
        .and_then(|m| m.modified())
        .ok()
}

#[test]
fn clm_repeat_byte_identity_and_head_untouched() {
    let Ok(url) = std::env::var(SERVE_URL_ENV) else {
        eprintln!(
            "UNSEEN — CLM_SERVE_URL is not set (their stack is not serving; \
             scripts/clm_serve_4090.sh start in the 4090 window, .issues/027). \
             This is a loud skip, never a pass."
        );
        return;
    };
    let req = fixed_request();
    let body = build_body(&req).to_string();

    let mtime_before = head_mtime();
    if mtime_before.is_none() {
        eprintln!(
            "UNSEEN — the mounted head .raw/CLM/checkpoints/CLM_v0.1-8B.pt is \
             absent on this box; the mtime arm cannot run (the byte-identity \
             arm still does)."
        );
    }

    // WARMUP (measured 2026-09-25): the very FIRST request after a
    // clm-serve boot answers byte-identically but reports
    // usage.input_tokens = 0 — a server-side first-request accounting
    // quirk, not answer nondeterminism. One throwaway request absorbs
    // it; the pin then holds the FULL body to byte-identity.
    raw_post(&url, &body).expect("warmup request");

    let lane_url = url.clone();
    const N: usize = 8;
    let mut bodies: Vec<Vec<u8>> = Vec::with_capacity(N);
    for i in 0..N {
        match raw_post(&lane_url, &body) {
            Ok(b) => bodies.push(b),
            Err(e) => panic!("repeat {i}: {e}"),
        }
    }

    let first = &bodies[0];
    for (i, b) in bodies.iter().enumerate().skip(1) {
        assert_eq!(
            first, b,
            "repeat {i} is NOT byte-identical to repeat 0 — the served lane is \
             nondeterministic (a lane that cannot repeat produces no cells; \
             .issues/027 amendment 1)"
        );
    }

    if let (Some(before), Some(after)) = (mtime_before, head_mtime()) {
        assert_eq!(
            before, after,
            "the head file's mtime MOVED across the pin — the server rewrote \
             its own checkpoint (a provenance hole; the measured artifact must \
             be the mounted one)"
        );
    }
    eprintln!(
        "clm determinism pin: {N}/{N} byte-identical repeats over one \
         mixed-kind request{}",
        if mtime_before.is_some() {
            " · head mtime unchanged"
        } else {
            ""
        }
    );
}
