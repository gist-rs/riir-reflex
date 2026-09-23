//! The HTTP-edge CORS seam (Plan 606 T1.2), over REAL loopback connections:
//! the allow-list opens exactly the origins it names (preflight + ACAO echo)
//! and stays closed otherwise (no ACAO anywhere, preflight 403) — the
//! drive-by posture. No process-global env mutation: the explicit allow-list
//! seam (`serve_listener_with`) is the surface under test.

use riir_reflex::serve::{demo_engine, serve_listener_with};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

fn spawn(allow: Vec<String>) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr").to_string();
    let eng = Arc::new(Mutex::new(demo_engine()));
    std::thread::spawn(move || {
        let _ = serve_listener_with(listener, eng, allow);
    });
    addr
}

fn roundtrip(addr: &str, raw: &str) -> String {
    let mut s = TcpStream::connect(addr).expect("connect");
    s.write_all(raw.as_bytes()).expect("write");
    let mut buf = String::new();
    let mut reader = BufReader::new(s);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => buf.push_str(&line),
        }
        if buf.contains("\r\n\r\n") {
            break;
        }
    }
    // Drain any declared body so the response is complete.
    if let Some(cl) = buf
        .lines()
        .find_map(|l| {
            let l = l.trim_end();
            l.strip_prefix("Content-Length:")
                .map(|v| v.trim().parse::<usize>().ok())
        })
        .flatten()
    {
        let mut body = vec![0u8; cl];
        let _ = reader.read_exact(&mut body);
        buf.push_str(&String::from_utf8_lossy(&body));
    }
    buf
}

const ARENA: &str = "https://reflex.gist.rs";
const EVIL: &str = "https://evil.example";

#[test]
fn closed_by_default_preflight_is_refused_without_acao() {
    let addr = spawn(vec![]);
    let resp = roundtrip(
        &addr,
        &format!("OPTIONS /decide HTTP/1.1\r\nHost: x\r\nOrigin: {ARENA}\r\nAccess-Control-Request-Method: POST\r\n\r\n"),
    );
    assert!(resp.starts_with("HTTP/1.1 403"), "got: {resp}");
    assert!(!resp.contains("Access-Control-Allow-Origin"), "got: {resp}");
}

#[test]
fn listed_origin_gets_preflight_204_and_full_cors_headers() {
    let addr = spawn(vec![ARENA.to_string()]);
    let resp = roundtrip(
        &addr,
        &format!("OPTIONS /decide HTTP/1.1\r\nHost: x\r\nOrigin: {ARENA}\r\nAccess-Control-Request-Method: POST\r\nAccess-Control-Request-Headers: content-type\r\n\r\n"),
    );
    assert!(resp.starts_with("HTTP/1.1 204"), "got: {resp}");
    assert!(resp.contains(&format!("Access-Control-Allow-Origin: {ARENA}")), "got: {resp}");
    assert!(resp.contains("Access-Control-Allow-Methods: GET, POST, OPTIONS"), "got: {resp}");
    assert!(resp.contains("Access-Control-Allow-Headers: Content-Type"), "got: {resp}");
    assert!(resp.contains("Vary: Origin"), "got: {resp}");
}

#[test]
fn unlisted_origin_preflight_is_refused_even_when_a_list_exists() {
    let addr = spawn(vec![ARENA.to_string()]);
    let resp = roundtrip(
        &addr,
        &format!("OPTIONS /decide HTTP/1.1\r\nHost: x\r\nOrigin: {EVIL}\r\nAccess-Control-Request-Method: POST\r\n\r\n"),
    );
    assert!(resp.starts_with("HTTP/1.1 403"), "got: {resp}");
    assert!(!resp.contains("Access-Control-Allow-Origin"), "got: {resp}");
}

#[test]
fn listed_origin_gets_acao_echo_on_post_decide() {
    let addr = spawn(vec![ARENA.to_string()]);
    let body = r#"{"state":"Deploy the server to staging","questions":[{"id":"q0","kind":"noul","prompt":"Roll back or promote?","options":[]}]}"#;
    let resp = roundtrip(
        &addr,
        &format!(
            "POST /decide HTTP/1.1\r\nHost: x\r\nOrigin: {ARENA}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(resp.contains("Access-Control-Allow-Origin: https://reflex.gist.rs"), "got: {resp}");
    assert!(resp.contains("Vary: Origin"), "got: {resp}");
    // The engine answered underneath the CORS layer.
    assert!(resp.contains("\"answers\""), "got: {resp}");
}

#[test]
fn unlisted_origin_post_gets_no_acao_and_still_serves_the_api() {
    // CORS stays closed for the stranger, but the plain-HTTP API is intact:
    // curl / localhost processes never send Origin.
    let addr = spawn(vec![ARENA.to_string()]);
    let body = r#"{"state":"Deploy the server to staging","questions":[{"id":"q0","kind":"noul","prompt":"Roll back or promote?","options":[]}]}"#;
    let resp = roundtrip(
        &addr,
        &format!(
            "POST /decide HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(!resp.contains("Access-Control-Allow-Origin"), "got: {resp}");
    assert!(resp.contains("\"answers\""), "got: {resp}");
}

#[test]
fn unlisted_origin_with_acao_attempt_is_not_echoed() {
    let addr = spawn(vec![ARENA.to_string()]);
    let body = r#"{"state":"Deploy the server to staging","questions":["Roll back or promote?"]}"#;
    let resp = roundtrip(
        &addr,
        &format!(
            "POST /decide HTTP/1.1\r\nHost: x\r\nOrigin: {EVIL}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(!resp.contains("Access-Control-Allow-Origin"), "got: {resp}");
}

#[test]
fn allow_list_parses_comma_separated_with_trim() {
    let addr = spawn(vec![ARENA.to_string(), EVIL.to_string()]);
    // Second listed origin also gets the preflight.
    let resp = roundtrip(
        &addr,
        &format!("OPTIONS /decide HTTP/1.1\r\nHost: x\r\nOrigin: {EVIL}\r\nAccess-Control-Request-Method: POST\r\n\r\n"),
    );
    assert!(resp.starts_with("HTTP/1.1 204"), "got: {resp}");
    assert!(resp.contains(&format!("Access-Control-Allow-Origin: {EVIL}")), "got: {resp}");
}
