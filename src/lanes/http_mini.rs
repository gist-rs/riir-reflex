//! The lanes' shared std-only HTTP/1.1 micro-client (Issue 068 — the
//! extraction the four TcpStream lanes each deferred at their own
//! landing: openthai's "isolation over DRY" T4.1, agentjev's clm-mirror
//! posture, clef's forwarder variant, clm's original). ONE `request()` +
//! ONE `parse_http_response` replace the four diverged per-lane copies;
//! the clef variant's chunked-decoding handling is folded in for EVERY
//! consumer (a forwarder is free to chunk; the content-length-only shape
//! would silently mis-truncate there).
//!
//! Laws:
//!
//! * **std-only** — the zero-dep edge is a product law (AGENTS.md). This
//!   module must never grow an HTTP crate, a TLS dep, or anything else:
//!   it is `std::net` + `std::io` and nothing more.
//! * **Per-lane knobs are parameters, not fields** — timeout and extra
//!   headers ride the call; per-lane error conversion happens at the call
//!   site (`String` for the agentjev-family lanes, `ClmError` via `From`
//!   for clm), so each lane keeps its own error surface and its own
//!   operator guidance (clef appends the forwarder hint on connect
//!   failures).
//! * **The error Display formats are the historical ones, verbatim** —
//!   `connect {host}:{port}: {e}` / `set timeout: {e}` / `write: {e}` /
//!   `read: {e}` / the parser messages. Lane errors flow into run logs
//!   and operator refusals; the extraction must not reword them.
//!
//! Deliberately NOT a consumer: `paw` — its transport is a `curl`
//! subprocess (the `-i` output parser must fold redirect chains, a
//! different shape), and `paw_local` rides a Python subprocess.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// One parsed HTTP/1.1 reply: `(status, lowercased headers, body bytes)`.
pub(crate) type HttpReply = (u16, HashMap<String, String>, Vec<u8>);

/// The micro-client's failure surface. The variants exist so a lane can
/// match on the CLASS (clef appends its forwarder hint to connect
/// failures only) while everything else renders through [`Display`].
#[derive(Debug)]
pub(crate) enum LaneHttpError {
    /// TCP connect failure (carries the target — the message names it).
    Connect {
        host: String,
        port: u16,
        source: io::Error,
    },
    /// Socket timeout installation failed.
    SetTimeout(io::Error),
    /// Request write failed.
    Write(io::Error),
    /// Response read failed.
    Read(io::Error),
    /// The response bytes are not HTTP/1.1-shaped (separator, status
    /// line, or chunk framing).
    Parse(String),
}

impl std::fmt::Display for LaneHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect { host, port, source } => {
                write!(f, "connect {host}:{port}: {source}")
            }
            Self::SetTimeout(e) => write!(f, "set timeout: {e}"),
            Self::Write(e) => write!(f, "write: {e}"),
            Self::Read(e) => write!(f, "read: {e}"),
            Self::Parse(m) => write!(f, "{m}"),
        }
    }
}

/// One HTTP/1.1 exchange over a fresh `TcpStream` (`Connection: close`,
/// read-to-end): connect → optional timeouts → write the request head +
/// body → read the whole reply → parse. A body implies
/// `Content-Type: application/json` + `Content-Length` (every lane's
/// requests are JSON); a bodyless request carries neither. `extra_headers`
/// is the per-lane knob (clef's bearer); header ORDER is not significant
/// to any of the measured servers.
pub(crate) fn request(
    host: &str,
    port: u16,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    timeout: Duration,
    extra_headers: &[(&str, &str)],
) -> Result<HttpReply, LaneHttpError> {
    let mut stream =
        TcpStream::connect((host, port)).map_err(|source| LaneHttpError::Connect {
            host: host.to_string(),
            port,
            source,
        })?;
    stream
        .set_read_timeout(Some(timeout))
        .and_then(|_| stream.set_write_timeout(Some(timeout)))
        .map_err(LaneHttpError::SetTimeout)?;
    let mut head =
        format!("{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n");
    for (k, v) in extra_headers {
        head.push_str(k);
        head.push_str(": ");
        head.push_str(v);
        head.push_str("\r\n");
    }
    if let Some(b) = body {
        head.push_str("Content-Type: application/json\r\nContent-Length: ");
        head.push_str(&b.len().to_string());
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    stream
        .write_all(head.as_bytes())
        .and_then(|_| match body {
            Some(b) => stream.write_all(b),
            None => Ok(()),
        })
        .and_then(|_| stream.flush())
        .map_err(LaneHttpError::Write)?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(LaneHttpError::Read)?;
    parse_http_response(&raw)
}

/// Split a raw HTTP/1.1 response into `(status, lowercased-headers, body)`,
/// decoding `Transfer-Encoding: chunked` when the reply is chunked (a
/// forwarder is free to chunk; a content-length-only parser would silently
/// mis-truncate there — the clef variant's law, folded in for every lane
/// by Issue 068). A neither-chunked-nor-length reply passes the body
/// through whole (the `Connection: close` read-to-end makes that the
/// complete body).
pub(crate) fn parse_http_response(raw: &[u8]) -> Result<HttpReply, LaneHttpError> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n").ok_or_else(|| {
        LaneHttpError::Parse("response has no header/body separator".into())
    })?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| LaneHttpError::Parse(format!("malformed status line: {status_line}")))?;
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let body_bytes = body.as_bytes().to_vec();
    let body_bytes = if headers
        .get("transfer-encoding")
        .is_some_and(|v| v.to_ascii_lowercase().contains("chunked"))
    {
        decode_chunked(&body_bytes).map_err(LaneHttpError::Parse)?
    } else if let Some(len) = headers.get("content-length").and_then(|v| v.parse::<usize>().ok())
    {
        let mut b = body_bytes;
        b.truncate(len);
        b
    } else {
        body_bytes
    };
    Ok((status, headers, body_bytes))
}

/// RFC 7230 chunked decoding: `size CRLF data CRLF` repeats, terminated by
/// a `0 CRLF` chunk (trailer section consumed to the end).
fn decode_chunked(body: &[u8]) -> Result<Vec<u8>, String> {
    let text = String::from_utf8_lossy(body);
    let mut out = Vec::with_capacity(body.len());
    let mut rest = text.as_ref();
    loop {
        let Some((size_line, after)) = rest.split_once("\r\n") else {
            return Err("chunked body: missing chunk-size line".into());
        };
        let size = usize::from_str_radix(
            size_line.split(';').next().unwrap_or_default().trim(),
            16,
        )
        .map_err(|_| format!("chunked body: bad chunk size {size_line:?}"))?;
        if size == 0 {
            return Ok(out);
        }
        if after.len() < size {
            return Err("chunked body: truncated chunk data".into());
        }
        let (data, tail) = after.split_at(size);
        out.extend_from_slice(data.as_bytes());
        rest = tail
            .strip_prefix("\r\n")
            .ok_or_else(|| "chunked body: chunk data not CRLF-terminated".to_string())?;
    }
}

// ───────────────────────────────────────────────────────────────── tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};

    /// The plain-reply golden (the openthai/agentjev law, carried over
    /// verbatim): head/body split, lowercased headers, content-length
    /// truncation.
    #[test]
    fn http_response_splits_head_and_body() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}";
        let (status, headers, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(
            headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(body, b"{}");
    }

    /// The chunked golden (the clef law, carried over verbatim): chunked
    /// bodies decode; plain bodies truncate at content-length (bytes a
    /// keep-alive peer would still have in flight are dropped).
    #[test]
    fn chunked_body_decodes_and_plain_truncates() {
        let raw =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n3\r\n:1}\r\n0\r\n\r\n";
        let (status, headers, body) = parse_http_response(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"{\"a\":1}");
        assert!(headers.contains_key("transfer-encoding"));
        let raw2 =
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{} plus bytes a keep-alive peer \
              would still have in flight";
        let (_, _, body2) = parse_http_response(raw2).unwrap();
        assert_eq!(body2, b"{}");
    }

    /// Every malformed-reply class refuses loudly (never a half-parsed
    /// row): no separator, no status token, a bad chunk size, and a
    /// truncated chunk.
    #[test]
    fn malformed_replies_refuse_loudly() {
        let e = parse_http_response(b"not http at all").unwrap_err();
        assert!(e.to_string().contains("separator"), "{e}");
        let e = parse_http_response(b"HTTP/1.1\r\n\r\n").unwrap_err();
        assert!(e.to_string().contains("malformed status line"), "{e}");
        let e = parse_http_response(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nZZ\r\nx\r\n0\r\n\r\n",
        )
        .unwrap_err();
        assert!(e.to_string().contains("bad chunk size"), "{e}");
        let e = parse_http_response(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n40\r\nshort\r\n0\r\n\r\n",
        )
        .unwrap_err();
        assert!(e.to_string().contains("truncated chunk data"), "{e}");
    }

    /// The REQUEST wire the micro-client writes, pinned at a stub listener
    /// (the clm/openthai stub pattern): request line, Host,
    /// `Connection: close`, the extra-header knob, Content-Type/Length on
    /// a POST — and the reply parsed back. The server drains the request
    /// BEFORE responding (a close with unread request bytes sends RST,
    /// which races the client's response read — the flake both stub tests
    /// shipped with).
    #[test]
    fn request_writes_the_wire_and_parses_the_reply() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("request line");
            let mut content_length = 0usize;
            let mut saw = std::collections::HashSet::new();
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let t = line.trim_end();
                        if t.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = t.split_once(':') {
                            let k = k.trim().to_ascii_lowercase();
                            let v = v.trim();
                            match k.as_str() {
                                "content-length" => content_length = v.parse().unwrap_or(0),
                                "authorization" => {
                                    saw.insert(format!("auth={v}"));
                                }
                                "host" => {
                                    saw.insert(format!("host={v}"));
                                }
                                "connection" => {
                                    saw.insert(format!("conn={v}"));
                                }
                                "content-type" => {
                                    saw.insert(format!("ctype={v}"));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).expect("body");
            let mut stream = stream;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .expect("respond");
            (request_line, body, saw)
        });
        let reply = request(
            "127.0.0.1",
            addr.port(),
            "POST",
            "/x",
            Some(br#"{"a":1}"#),
            Duration::from_secs(5),
            &[("Authorization", "Bearer tok")],
        )
        .expect("round trip");
        let (request_line, body, saw) = server.join().expect("server thread");
        assert_eq!(request_line, "POST /x HTTP/1.1\r\n");
        assert_eq!(body, br#"{"a":1}"#.to_vec());
        assert!(saw.contains("auth=Bearer tok"), "extra header missing: {saw:?}");
        assert!(
            saw.contains(&format!("host=127.0.0.1:{}", addr.port())),
            "host header missing: {saw:?}"
        );
        assert!(saw.contains("conn=close"), "connection header missing: {saw:?}");
        assert!(
            saw.contains("ctype=application/json"),
            "content-type on a POST missing: {saw:?}"
        );
        assert_eq!(reply.0, 200);
        assert_eq!(reply.2, b"ok".to_vec());
    }

    /// A bodyless GET carries NO content-type/content-length headers, and
    /// a chunked reply parses on that path too.
    #[test]
    fn get_carries_no_body_headers() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("request line");
            let mut saw_body_headers = false;
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let t = line.trim_end();
                        if t.is_empty() {
                            break;
                        }
                        if let Some((k, _)) = t.split_once(':') {
                            let k = k.trim().to_ascii_lowercase();
                            if k == "content-type" || k == "content-length" {
                                saw_body_headers = true;
                            }
                        }
                    }
                }
            }
            let mut stream = stream;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nok\r\n0\r\n\r\n")
                .expect("respond");
            (request_line, saw_body_headers)
        });
        let reply = request(
            "127.0.0.1",
            addr.port(),
            "GET",
            "/healthz",
            None,
            Duration::from_secs(5),
            &[],
        )
        .expect("round trip");
        let (request_line, saw_body_headers) = server.join().expect("server thread");
        assert_eq!(request_line, "GET /healthz HTTP/1.1\r\n");
        assert!(!saw_body_headers, "GET must carry no body headers");
        assert_eq!(reply.0, 200);
        assert_eq!(reply.2, b"ok".to_vec());
    }

    /// A refused port is a `Connect` failure whose Display names the
    /// target (the operator-facing format the lanes' errors have always
    /// carried).
    #[test]
    fn connect_failure_names_the_target() {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind");
        let port = l.local_addr().expect("addr").port();
        drop(l);
        let e = request(
            "127.0.0.1",
            port,
            "GET",
            "/",
            None,
            Duration::from_secs(2),
            &[],
        )
        .unwrap_err();
        assert!(
            matches!(e, LaneHttpError::Connect { .. }),
            "expected Connect, got {e:?}"
        );
        assert!(
            e.to_string().contains("connect 127.0.0.1:"),
            "display must name the target: {e}"
        );
    }
}
