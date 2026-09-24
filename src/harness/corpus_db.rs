//! `corpus_db` — the harness's Warm-tier persistence via the released `ndb`
//! binary (Issue 007 P1). **VALUE-ONLY law:** the row VALUES are 100% ours
//! (harness JSON + BLAKE3 digests); the CLI's `--json` grammar, key grammar,
//! and exit-code taxonomy are ndb-owned wires, pinned here by golden tests
//! against the REAL binary (`binary_wire_golden_round_trip`) so a
//! producer-side change reds on the consumer side before any run trusts it.
//!
//! Consumption law (Issue 007, the ndb-611 / deployer posture):
//! - **Zero cargo deps on the storage leaf.** The store is a subprocess
//!   (`std::process`); the structural no-source-leak posture — a released
//!   reflex binary can never carry compiled db code, by construction.
//! - **`--json`-only calls.** One JSON object per line on stdout; classified
//!   failures as `{"v":1,"ok":false,"code":"…","error":"…"}` with the stable
//!   code taxonomy (usage=2, not_found=3, denied=4, store_fault=5).
//! - **Writes through stdin, never argv** — payloads in `ps`/argv leak on
//!   multi-agent boxes and are ARG_MAX-bounded.
//! - **One corpus = ONE row** — a suite's dataset is a single digest-pinned
//!   JSON blob; `scan` is enumeration/inventory only.
//! - **Agents/CI consent:** the child always gets `NDB_ASSUME_YES=1` (the
//!   CLI-side bypass; public rows never prompt anyway — belt and braces).
//! - **Passphrase posture:** `NDB_PASSPHRASE` env (inherited by the child)
//!   selects the encrypted posture; absent/empty → `--insecure-plaintext`
//!   is passed EXPLICITLY (the store's own dev posture, acknowledged —
//!   local benchmark data, never chain-committed state).
//!
//! Binary resolution: `NDB_BIN` override → `ndb` on PATH → loud refuse
//! naming the local-build one-liner and the `NDB_BIN` escape.
//!
//! NATIVE-ONLY by construction (`std::process` does not exist on
//! wasm32-unknown-unknown): the module is cfg'd out there and the feature
//! must never join a wasm32 combo (the `trace_index` rule).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The table one run-history row lands in (value = the exact results.json
/// bytes the harness bin writes; no dual schema).
pub const RUNS_TABLE: &str = "harness_runs";
/// The table digest-pinned corpus rows land in.
pub const CORPUS_TABLE: &str = "harness_corpus";

/// The `ndb` store the harness writes run history + corpora into.
pub struct NdbRunStore {
    binary: PathBuf,
    store_dir: PathBuf,
}

/// The `ndb --json version` probe answer (the consumer version pin).
#[derive(Debug, Clone)]
pub struct NdbVersion {
    pub version: String,
    pub compiled_features: Vec<String>,
    pub shippable: bool,
}

/// A `put` acknowledgement.
#[derive(Debug, Clone)]
pub struct PutAck {
    pub store_key: String,
    pub shard: u8,
    pub bytes: usize,
    pub visibility: String,
}

/// One `scan` row (keys + lengths only — `scan` is inventory, never bulk read).
#[derive(Debug, Clone)]
pub struct RowMeta {
    pub table: String,
    pub key: String,
    pub bytes: usize,
    pub visibility: String,
    pub readable: bool,
}

/// A classified ndb failure: the stable `code` string + the CLI exit code.
#[derive(Debug, Clone)]
pub struct NdbError {
    pub code: String,
    pub exit: u8,
    pub message: String,
}

impl NdbError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            code: "usage".into(),
            exit: 2,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for NdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ndb {} (exit {}): {}",
            self.code, self.exit, self.message
        )
    }
}

impl NdbRunStore {
    /// Resolve the binary from the env value of `NDB_BIN` (a `None` = unset),
    /// falling back to `ndb` on PATH, refusing LOUD with the local-build
    /// one-liner and the `NDB_BIN` escape. Split from `resolve()` for
    /// testability (no env mutation in tests).
    pub fn resolve_with(
        ndb_bin_env: Option<&std::ffi::OsStr>,
        store_dir: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        let binary = match ndb_bin_env {
            Some(v) if !v.is_empty() => PathBuf::from(v),
            _ => {
                let path = std::env::var_os("PATH").unwrap_or_default();
                let found = std::env::split_paths(&path)
                    .map(|dir| dir.join("ndb"))
                    .find(|p| is_executable(p));
                match found {
                    Some(p) => p,
                    None => {
                        return Err(
                            "ndb binary not found — the harness store needs the released \
                             neuron-db CLI. Build it locally:\n\
                             \x20 cargo build --release -p neuron-db-cli\n\
                             \x20 (run in ../riir-neuron-db), then point NDB_BIN at \
                             target/release/ndb (or install it on PATH)."
                                .to_string(),
                        );
                    }
                }
            }
        };
        Ok(Self {
            binary,
            store_dir: store_dir.into(),
        })
    }

    /// `resolve_with` over the live `NDB_BIN` env.
    pub fn resolve(store_dir: impl Into<PathBuf>) -> Result<Self, String> {
        Self::resolve_with(std::env::var_os("NDB_BIN").as_deref(), store_dir)
    }

    pub fn store_dir(&self) -> &Path {
        &self.store_dir
    }

    /// The consumer version probe: `ndb --json version` (clap's `--version`
    /// short-circuits before dispatch — the SUBCOMMAND form is the probe).
    pub fn probe(&self) -> Result<NdbVersion, NdbError> {
        let out = self.run_json(&["version"], false, None)?;
        Ok(NdbVersion {
            version: out
                .get("version")
                .and_then(|v| v.as_str())
                .ok_or_else(|| NdbError::usage("version probe: no version field"))?
                .to_string(),
            compiled_features: out
                .get("compiled_features")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            shippable: out
                .get("shippable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    /// Write one row; the VALUE rides stdin (never argv).
    pub fn put_row(&self, table: &str, key: &str, value: &[u8]) -> Result<PutAck, NdbError> {
        let out = self.run_json(&["put", table, key], true, Some(value))?;
        Ok(PutAck {
            store_key: out
                .get("store_key")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            shard: out.get("shard").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            bytes: out.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            visibility: out
                .get("visibility")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
        })
    }

    /// Read one row. The harness only writes utf-8 JSON values; a `base64`
    /// encoding means FOREIGN data in this table — refused loud, never
    /// silently decoded (fail-closed over convenience).
    pub fn get_row(&self, table: &str, key: &str) -> Result<Vec<u8>, NdbError> {
        let out = self.run_json(&["get", table, key], true, None)?;
        match out.get("encoding").and_then(|v| v.as_str()) {
            Some("utf8") => Ok(out
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .as_bytes()
                .to_vec()),
            Some(other) => Err(NdbError::usage(format!(
                "get {table}/{key}: non-utf8 row encoding `{other}` — the harness \
                 writes utf-8 JSON only; foreign data in a harness table is a \
                 fail-loud condition"
            ))),
            None => Err(NdbError::usage(format!(
                "get {table}/{key}: no encoding field in --json output"
            ))),
        }
    }

    /// Inventory: keys + lengths (the one-corpus-one-row law keeps this cheap).
    pub fn scan(&self) -> Result<Vec<RowMeta>, NdbError> {
        let out = self.run_json(&["scan"], true, None)?;
        let rows = out
            .get("rows")
            .and_then(|v| v.as_array())
            .ok_or_else(|| NdbError::usage("scan: no rows array in --json output"))?;
        Ok(rows
            .iter()
            .map(|r| RowMeta {
                table: r
                    .get("table")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .into(),
                key: r.get("key").and_then(|v| v.as_str()).unwrap_or("?").into(),
                bytes: r.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                visibility: r
                    .get("visibility")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .into(),
                readable: r.get("readable").and_then(|v| v.as_bool()).unwrap_or(false),
            })
            .collect())
    }

    /// Save ONE corpus row: `corpus/<name>@<digest16>` — the whole suite
    /// blob (test+train envelopes) as a single value, BLAKE3-digested in the
    /// key (the dataset_manifest law). Returns the full 64-hex digest.
    pub fn put_corpus(&self, name: &str, blob: &[u8]) -> Result<(PutAck, String), NdbError> {
        let digest = blake3::hash(blob).to_hex().to_string();
        let key = corpus_key(name, &digest);
        let ack = self.put_row(CORPUS_TABLE, &key, blob)?;
        Ok((ack, digest))
    }

    /// Read a corpus row by name + expected digest — a digest mismatch (the
    /// row is not the corpus this run was pinned to) is REFUSED, never
    /// loaded-with-a-warning.
    pub fn get_corpus(&self, name: &str, digest: &str) -> Result<Vec<u8>, NdbError> {
        let key = corpus_key(name, digest16(digest));
        let blob = self.get_row(CORPUS_TABLE, &key)?;
        let got = blake3::hash(&blob).to_hex().to_string();
        if got != digest {
            return Err(NdbError {
                code: "store_fault".into(),
                exit: 5,
                message: format!(
                    "corpus {name}: stored bytes digest {got} != pinned {digest} — \
                     refusing to load a corpus that is not the pinned one"
                ),
            });
        }
        Ok(blob)
    }

    /// One `--json` call. `store_opening` adds `--dir <store>` plus the
    /// plaintext ack; `stdin_value` pipes the payload (the write law).
    fn run_json(
        &self,
        sub_args: &[&str],
        store_opening: bool,
        stdin_value: Option<&[u8]>,
    ) -> Result<serde_json::Value, NdbError> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("--json").args(sub_args);
        if store_opening {
            cmd.arg("--dir").arg(&self.store_dir);
            // The plaintext posture: NDB_PASSPHRASE env (inherited) selects
            // encryption; absent/empty → pass the ack flag explicitly.
            let passphrase_set = std::env::var_os("NDB_PASSPHRASE")
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            if !passphrase_set {
                cmd.arg("--insecure-plaintext");
            }
        }
        cmd.env("NDB_ASSUME_YES", "1");
        cmd.stdin(if stdin_value.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| {
            NdbError::usage(format!(
                "spawn {} failed: {e} — is NDB_BIN pointing at the ndb binary?",
                self.binary.display()
            ))
        })?;
        if let Some(value) = stdin_value
            && let Some(mut stdin) = child.stdin.take()
        {
            stdin
                .write_all(value)
                .map_err(|e| NdbError::usage(format!("ndb stdin write: {e}")))?;
            // Dropping the handle closes stdin — put reads to EOF.
        }
        let out = child
            .wait_with_output()
            .map_err(|e| NdbError::usage(format!("ndb wait: {e}")))?;
        let parsed = parse_json_line(&out.stdout).ok_or_else(|| {
            NdbError::usage(format!(
                "ndb {:?} produced no --json line (exit {:?}, stderr: {})",
                sub_args,
                out.status.code(),
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        })?;
        // The classified-failure shape: `{"ok":false,"code":…}` + non-zero
        // exit — surfaced as the typed error, never as a success value.
        if parsed.get("ok").and_then(|v| v.as_bool()) == Some(false) {
            return Err(NdbError {
                code: parsed
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                exit: out.status.code().unwrap_or(1).clamp(0, 255) as u8,
                message: parsed
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no error field)")
                    .to_string(),
            });
        }
        Ok(parsed)
    }
}

/// The corpus key grammar: `corpus/<name>@<digest16>` — one corpus, one row,
/// the digest IS the version.
pub fn corpus_key(name: &str, digest: &str) -> String {
    format!("corpus/{name}@{}", digest16(digest))
}

fn digest16(digest: &str) -> &str {
    &digest[..digest.len().min(16)]
}

/// Extract the LAST `{"v":1,...}` object line from stdout (errors print the
/// JSON object then a stderr line, and exit non-zero).
fn parse_json_line(stdout: &[u8]) -> Option<serde_json::Value> {
    let text = String::from_utf8_lossy(stdout);
    let mut found = None;
    for line in text.lines().rev() {
        let line = line.trim();
        if line.starts_with('{')
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(line)
            && v.get("v").and_then(|v| v.as_u64()) == Some(1)
            && v.get("ok").is_some()
        {
            found = Some(v);
            break;
        }
    }
    found
}

fn is_executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        p.is_file()
            && std::fs::metadata(p)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── wire goldens (consumer-side pins over the ndb-owned grammar) ────

    #[test]
    fn version_wire_golden() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"v":1,"ok":true,"version":"0.1.0","compiled_features":["a","b"],"shippable":true,"forbidden_present":[]}"#,
        )
        .unwrap();
        assert_eq!(v.get("version").unwrap().as_str().unwrap(), "0.1.0");
        assert_eq!(
            v.get("compiled_features")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(v.get("shippable").unwrap().as_bool().unwrap());
    }

    #[test]
    fn put_get_delete_scan_wire_goldens() {
        let put: serde_json::Value = serde_json::from_str(
            r#"{"v":1,"ok":true,"store_key":"k","shard":7,"bytes":42,"visibility":"pub"}"#,
        )
        .unwrap();
        assert_eq!(put.get("shard").unwrap().as_u64().unwrap(), 7);
        assert_eq!(put.get("visibility").unwrap().as_str().unwrap(), "pub");

        let get: serde_json::Value = serde_json::from_str(
            r#"{"v":1,"ok":true,"table":"t","key":"k","encoding":"utf8","value":"{}"}"#,
        )
        .unwrap();
        assert_eq!(get.get("encoding").unwrap().as_str().unwrap(), "utf8");

        let scan: serde_json::Value = serde_json::from_str(
            r#"{"v":1,"ok":true,"rows":[{"store_key":"k","visibility":"pub","shard":0,"owner":null,"table":"t","key":"k","bytes":3,"readable":true}]}"#,
        )
        .unwrap();
        assert_eq!(scan.get("rows").unwrap().as_array().unwrap().len(), 1);
    }

    #[test]
    fn error_wire_golden_maps_code_and_exit() {
        // The classified-failure path run_json implements: ok:false + the
        // child's exit code surface together as one typed error.
        let parsed: serde_json::Value =
            serde_json::from_str(r#"{"v":1,"ok":false,"code":"not_found","error":"no row t/k"}"#)
                .unwrap();
        assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(
            parsed.get("code").and_then(|v| v.as_str()),
            Some("not_found")
        );
        // An ok answer is NOT an error.
        let ok: serde_json::Value =
            serde_json::from_str(r#"{"v":1,"ok":true,"version":"x"}"#).unwrap();
        assert_eq!(ok.get("ok").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn parse_json_line_takes_the_last_v1_object() {
        let stdout = b"noise\n{\"v\":1,\"ok\":true,\"n\":1}\n{\"v\":2,\"ok\":true}\n{\"v\":1,\"ok\":true,\"n\":2}\n";
        let v = parse_json_line(stdout).unwrap();
        assert_eq!(v.get("n").unwrap().as_u64().unwrap(), 2);
        assert!(parse_json_line(b"no json at all").is_none());
    }

    // ── resolution (no env mutation — the env value is injected) ────────

    #[test]
    fn resolve_prefers_the_ndb_bin_override() {
        let store =
            NdbRunStore::resolve_with(Some(std::ffi::OsStr::new("/opt/ndb")), "/tmp/kv").unwrap();
        assert_eq!(store.binary, PathBuf::from("/opt/ndb"));
        assert_eq!(store.store_dir(), Path::new("/tmp/kv"));
    }

    #[test]
    fn corpus_key_pins_name_and_digest() {
        assert_eq!(
            corpus_key("ag_news", "0123456789abcdef0123456789abcdef"),
            "corpus/ag_news@0123456789abcdef"
        );
        // Short digests are used verbatim (never truncated below what's given).
        assert_eq!(corpus_key("x", "abc"), "corpus/x@abc");
    }

    // ── the REAL-binary golden pin (Issue-084 discipline) ───────────────
    // Loud-skip without the binary (the G5-weights convention: a skip names
    // what was skipped, never a green zero over an unrun gate).

    #[test]
    fn binary_wire_golden_round_trip() {
        let store_dir =
            std::env::temp_dir().join(format!("reflex_corpus_db_pin_{}", std::process::id()));
        let Ok(store) = NdbRunStore::resolve(&store_dir) else {
            eprintln!(
                "SKIP: ndb binary not found (NDB_BIN unset, not on PATH) — build it: \
                 cargo build --release -p neuron-db-cli in ../riir-neuron-db, or set NDB_BIN"
            );
            return;
        };
        let version = store.probe().expect("version probe");
        eprintln!(
            "ndb version {} (features: {:?})",
            version.version, version.compiled_features
        );
        assert!(
            !version.version.is_empty(),
            "version stamp must be non-empty"
        );

        // put → get round trip through stdin (the write law).
        let value = br#"{"runs":1,"suite":"pin","acc":0.5}"#;
        let ack = store
            .put_row(RUNS_TABLE, "pin/round_trip", value)
            .expect("put");
        assert_eq!(ack.visibility, "pub", "harness rows are public");
        assert_eq!(ack.bytes, value.len());
        let got = store.get_row(RUNS_TABLE, "pin/round_trip").expect("get");
        assert_eq!(got, value, "round trip must be byte-exact");

        // scan enumerates the row (keys + lengths only).
        let rows = store.scan().expect("scan");
        assert!(
            rows.iter()
                .any(|r| r.table == RUNS_TABLE && r.key == "pin/round_trip"),
            "scan must list the pinned row"
        );

        // The corpus half: put_corpus digests, get_corpus verifies, a wrong
        // digest is refused.
        let corpus = br#"{"test_rows":{"rows":[]},"train_rows":{"rows":[]}}"#;
        let (ack, digest) = store.put_corpus("pin_suite", corpus).expect("put_corpus");
        assert_eq!(ack.visibility, "pub");
        let read_back = store.get_corpus("pin_suite", &digest).expect("get_corpus");
        assert_eq!(read_back, corpus, "corpus round trip must be byte-exact");
        let err = store
            .get_corpus(
                "pin_suite",
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap_err();
        assert!(
            err.message.contains("digest") || err.code == "not_found",
            "wrong digest must refuse or miss: {err}"
        );

        // not_found has its own code — a get of an absent row is classified,
        // never a generic failure.
        let err = store.get_row(RUNS_TABLE, "pin/absent").unwrap_err();
        assert_eq!(
            err.code, "not_found",
            "absent row must classify as not_found: {err}"
        );
    }
}
