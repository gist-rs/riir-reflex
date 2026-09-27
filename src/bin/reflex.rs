//! `reflex` — the localhost decision-engine binary (Plan 603 T1.3; the
//! `riir-reflex` crate's serve bin, renamed from `riir-reflex` 2026-09-23
//! so users type the short name).
//!
//! Bare invocation = serve (the just-works posture). `--version`/`-V` prints
//! the build stamp (`build_stamp`): version + compiled feature set + the
//! loud `STALE` line and rebuild command when the set is incomplete against
//! the shipped release set.
//!
//! `mint-heads` (instinct Proposal 001 T4, feature `vessel_public_read`):
//! fit the three arena game heads from the repo fixture files, verify each
//! against its published BLAKE3 pin, sign them as PUBLIC-RELEASE vessels
//! through reflexer-vessel's public writer, and write
//! `{tetris,lanes,flappy}.vessel` to `--out`. The serve binary loads them
//! from `RIIR_REFLEX_HEADS_DIR` (trusted via `RIIR_REFLEX_HEADS_PUBKEY` —
//! the verifying key hex printed here). Vessels are committed NOWHERE.
//!
//! Loopback-only by default (`RIIR_REFLEX_BIND` overrides). The modelless
//! lane is ns–µs tier; the HTTP edge is the cold path. No daemon framework,
//! one process, std networking only.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => {
            let bind = riir_reflex::serve::bind_addr();
            if let Err(e) = riir_reflex::serve::run() {
                eprintln!("[reflex] serve on {bind} failed: {e}");
                std::process::exit(1);
            }
        }
        Some("-V" | "--version") => {
            print!("{}", riir_reflex::build_stamp::stamp());
        }
        #[cfg(feature = "vessel_public_read")]
        Some("mint-heads") => {
            mint_heads(&args[1..]);
        }
        #[cfg(not(feature = "vessel_public_read"))]
        Some("mint-heads") => {
            eprintln!(
                "error: mint-heads needs the vessel_public_read feature — rebuild with: cargo build --features vessel_public_read --bin reflex"
            );
            std::process::exit(2);
        }
        Some(other) => {
            eprintln!(
                "error: unknown argument {other:?} — bare invocation serves; --version prints the build stamp; mint-heads mints the game-head vessels"
            );
            eprintln!("{}", riir_reflex::build_stamp::stamp());
            std::process::exit(2);
        }
    }
}

/// `reflex mint-heads` — the heads→vessels mint path (the MINT INPUTS are
/// the repo fixture files; the OUTPUTS are signed vessels, committed
/// nowhere). Key handling is fail-closed: `--key` (64-hex seed) →
/// `--key-file` (64-hex text or 32 raw bytes) → env `REFLEXER_SIGN_KEY`
/// (the format repo's env name — one key surface across both bins).
#[cfg(feature = "vessel_public_read")]
fn mint_heads(args: &[String]) {
    let flag = |name: &str| -> Option<String> {
        let prefix = format!("{name}=");
        for (i, a) in args.iter().enumerate() {
            if a == name {
                return args.get(i + 1).cloned();
            }
            if let Some(v) = a.strip_prefix(&prefix) {
                return Some(v.to_string());
            }
        }
        None
    };
    let has = |name: &str| args.iter().any(|a| a == name);
    if has("--help") || has("-h") {
        eprintln!(
            "usage: reflex mint-heads --out <dir> [--fixtures <dir>=assets/game_heads] \\\n             --key-id <u32> [--artifact-version <u64>=1] \\\n             [--key <64-hex-seed> | --key-file <path>]   (env REFLEXER_SIGN_KEY)"
        );
        return;
    }
    let Some(out) = flag("--out") else {
        eprintln!("reflex mint-heads: --out <dir> is required (the heads dir RIIR_REFLEX_HEADS_DIR will point at)");
        std::process::exit(2);
    };
    let fixtures = flag("--fixtures").unwrap_or_else(|| "assets/game_heads".to_string());
    let key_id: u32 = match flag("--key-id") {
        Some(v) => v.parse().unwrap_or_else(|_| {
            eprintln!("reflex mint-heads: --key-id must be a u32, got {v:?}");
            std::process::exit(2);
        }),
        None => {
            eprintln!("reflex mint-heads: --key-id <u32> is required (the PinTable identity of the minting key)");
            std::process::exit(2);
        }
    };
    let artifact_version: u64 = match flag("--artifact-version") {
        Some(v) => v.parse().unwrap_or_else(|_| {
            eprintln!("reflex mint-heads: --artifact-version must be a u64, got {v:?}");
            std::process::exit(2);
        }),
        None => 1,
    };
    let key = if let Some(hex) = flag("--key") {
        reflexer_vessel::writer::signing_key_from_seed_hex(hex.trim())
    } else if let Some(path) = flag("--key-file") {
        reflexer_vessel::writer::signing_key_from_file(std::path::Path::new(&path))
    } else if let Ok(hex) = std::env::var("REFLEXER_SIGN_KEY") {
        reflexer_vessel::writer::signing_key_from_seed_hex(hex.trim())
    } else {
        Err("no signing key: pass --key <64-hex-seed>, --key-file <path> (64-hex text or 32 raw bytes), or set REFLEXER_SIGN_KEY".to_string())
    }
    .unwrap_or_else(|e| {
        eprintln!("reflex mint-heads: {e}");
        std::process::exit(2);
    });

    let report = riir_reflex::game_heads::head_vessels::mint_all(
        std::path::Path::new(&fixtures),
        std::path::Path::new(&out),
        &key,
        key_id,
        artifact_version,
    )
    .unwrap_or_else(|e| {
        eprintln!("reflex mint-heads: {e}");
        std::process::exit(1);
    });

    let vk = reflexer_vessel::writer::verifying_key_hex(&key);
    eprintln!(
        "reflex mint-heads: wrote {} head vessels to {out} (deterministic — re-running refreshes nothing)",
        report.len()
    );
    for row in &report {
        eprintln!(
            "  {:>7}: {} → head digest {}, commitment {}, λ {}, {} options",
            row.name, row.file, row.head_digest_hex, row.commitment_hex, row.lambda, row.n_options
        );
    }
    println!(
        "{{\"out\":{:?},\"key_id\":{},\"artifact_version\":{},\"verifying_key\":{:?},\"heads\":{}}}",
        out,
        key_id,
        artifact_version,
        vk,
        serde_json::to_string(&report).unwrap_or_default(),
    );
    eprintln!(
        "serve posture: RIIR_REFLEX_HEADS_DIR={out} RIIR_REFLEX_HEADS_PUBKEY={vk} (the verifying key IS the trust anchor — pin it beside the release)"
    );
}
