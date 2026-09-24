//! `reflex` — the localhost decision-engine binary (Plan 603 T1.3; the
//! `riir-reflex` crate's serve bin, renamed from `riir-reflex` 2026-09-23
//! so users type the short name).
//!
//! Bare invocation = serve (the just-works posture). `--version`/`-V` prints
//! the build stamp (`build_stamp`): version + compiled feature set + the
//! loud `STALE` line and rebuild command when the set is incomplete against
//! the shipped release set.
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
        Some(other) => {
            eprintln!(
                "error: unknown argument {other:?} — bare invocation serves; --version prints the build stamp"
            );
            eprintln!("{}", riir_reflex::build_stamp::stamp());
            std::process::exit(2);
        }
    }
}
