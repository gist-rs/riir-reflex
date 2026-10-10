//! Issue 063: the user-corpus loader behind `RIIR_REFLEX_CORPUS` — the
//! shipped binary's answer to "your first corpus in 5 minutes" (reflex-site
//! Issue 006 T4).
//!
//! Shape: one `<domain>.md` file (the file is one document) or one
//! `<domain>/` directory (each `.md` inside is one document, filename-sorted)
//! per domain. Domain names sort — the engine's build order (and every
//! routing array) is deterministic for the same directory. Non-`.md` files
//! (README notes, editor scratch) are not corpus and are ignored; a `.md`
//! file and a directory claiming the SAME domain name is a collision and
//! refuses.
//!
//! A malformed directory refuses the boot — never a silent demo fallback
//! (the per-lane-claims law: an engine that was asked to serve a corpus and
//! cannot is a boot failure, not a demo swap). The dispatch ceiling is
//! [`MAX_DOMAINS`]: the shipped binary monomorphises the serve loop over
//! `N ∈ 1..=8` (option 1 — the hot path stays compile-time-shaped; the
//! dispatch happens once per process, never per request).

use crate::engine::ExpertSpec;
use std::path::Path;

/// The closed dispatch enum's ceiling. Over this the loader refuses loud
/// (the in-repo harness lane is the over-8 posture).
pub const MAX_DOMAINS: usize = 8;

/// A loaded corpus: the engine input plus the disclosure material the boot
/// line and `/healthz` render.
#[derive(Debug)]
pub struct CorpusSpecs {
    /// Domain names in build order (sorted).
    pub domains: Vec<String>,
    /// One document count per domain, same order.
    pub docs_per_domain: Vec<usize>,
    /// The engine's builder input, same order.
    pub specs: Vec<ExpertSpec>,
}

/// Load a corpus directory. Errors name the file that refused — the boot
/// prints them and exits, it never falls back to the demo engine.
pub fn load_dir(dir: &Path) -> Result<CorpusSpecs, String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    // (domain, [(source path, text)]) — insertion order, sorted below.
    let mut found: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = entry.path();
        // The domain name is the entry's name VERBATIM for a directory
        // (issue 081 T2c: a domain named after a wire option string — prose
        // that may end in `.` — must survive byte-intact for by-name option
        // routing; `file_stem` strips a trailing extension separator and
        // silently renamed `"…help."` to `"…help"`). Files keep the stem
        // (the `<domain>.md` single-doc form strips its extension).
        let name_os = if path.is_dir() {
            path.file_name()
        } else {
            path.file_stem()
        };
        let Some(name) = name_os.and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            let inner =
                std::fs::read_dir(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut inner_paths: Vec<std::path::PathBuf> = Vec::new();
            for e in inner {
                inner_paths.push(e.map_err(|er| format!("{}: {er}", path.display()))?.path());
            }
            inner_paths.sort();
            let mut docs = Vec::new();
            for p in inner_paths {
                if p.extension().and_then(|e| e.to_str()) == Some("md") {
                    let text = std::fs::read_to_string(&p)
                        .map_err(|e| format!("{}: {e}", p.display()))?;
                    docs.push((p.display().to_string(), text));
                }
            }
            if docs.is_empty() {
                return Err(format!(
                    "domain `{name}` ({}) carries no .md documents — put one or more .md files inside it",
                    path.display()
                ));
            }
            found.push((name.to_string(), docs));
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            found.push((name.to_string(), vec![(path.display().to_string(), text)]));
        }
        // Anything else (notes, non-.md files) is not corpus — ignored.
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    if found.is_empty() {
        return Err(format!(
            "no domains found in {} — expected `<domain>.md` files or `<domain>/` directories of .md files",
            dir.display()
        ));
    }
    if found.len() > MAX_DOMAINS {
        return Err(format!(
            "{} carries {} domains — the shipped binary serves up to {MAX_DOMAINS} \
             (split the corpus or use the in-repo harness lane)",
            dir.display(),
            found.len()
        ));
    }
    let mut domains = Vec::with_capacity(found.len());
    let mut docs_per_domain = Vec::with_capacity(found.len());
    let mut specs = Vec::with_capacity(found.len());
    for (name, docs) in found {
        if domains.iter().any(|d| d == &name) {
            return Err(format!(
                "duplicate domain name `{name}` — a `<name>.md` file and a `<name>/` \
                 directory cannot both claim the domain"
            ));
        }
        let mut texts = Vec::with_capacity(docs.len());
        for (src, text) in docs {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Err(format!("domain `{name}`: {src} is empty"));
            }
            texts.push(trimmed.to_string());
        }
        docs_per_domain.push(texts.len());
        domains.push(name.clone());
        specs.push(ExpertSpec::new(name, &texts));
    }
    Ok(CorpusSpecs {
        domains,
        docs_per_domain,
        specs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "riir_reflex_corpus_loader_{}_{}",
            tag,
            std::process::id()
        ))
    }

    fn write(path: &std::path::Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdirs");
        std::fs::write(path, text).expect("write");
    }

    #[test]
    fn file_and_dir_domains_load_sorted_with_counts() {
        let dir = temp_dir("ok");
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir.join("billing.md"), "refund the duplicate charge.\n");
        write(&dir.join("deploy/rollback.md"), "rollback is one command.\n");
        write(&dir.join("deploy/staging.md"), "verify the staging rollout.\n");
        // A non-.md note is ignored, not corpus.
        write(&dir.join("deploy/README.txt"), "notes only");
        let loaded = load_dir(&dir).expect("loads");
        assert_eq!(loaded.domains, vec!["billing", "deploy"]);
        assert_eq!(loaded.docs_per_domain, vec![1, 2]);
        assert_eq!(loaded.specs.len(), 2);
        assert_eq!(loaded.specs[1].docs.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg_attr(windows, ignore = "Win32 strips trailing dots from the last path component at dir creation - the trailing-dot fixture is unrepresentable on NTFS; the byte-intact property stays guarded by the interior-dot companion below on Windows and by this test on the macOS/Linux deployment platforms (issue 084)")]
    fn directory_names_load_verbatim_including_dots() {
        // Issue 081 T2c: a domain named after a wire option string (prose
        // that ends in `.`) must load byte-intact — by-name option routing
        // compares the option string to the domain name with `==`.
        let dir = temp_dir("verbatim");
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir.join("No further round helps./d0.md"), "state depth 0 doc\n");
        write(&dir.join("Another round would help./d1.md"), "state depth 1 doc\n");
        write(&dir.join("plain/plain.md"), "plain domain keeps working\n");
        let loaded = load_dir(&dir).expect("loads");
        assert!(loaded.domains.contains(&"No further round helps.".to_string()));
        assert!(loaded.domains.contains(&"Another round would help.".to_string()));
        assert!(loaded.domains.contains(&"plain".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn directory_names_load_verbatim_interior_dots() {
        // Issue 084 companion: Windows cannot create the trailing-dot
        // fixture at all, so the loader's byte-intact-name property needs a
        // fixture every platform can represent — dots in NON-terminal
        // positions. Still catches a loader that trims at the first dot or
        // normalizes domain names.
        let dir = temp_dir("interior");
        let _ = std::fs::remove_dir_all(&dir);
        write(&dir.join("state.depth.0/d0.md"), "state depth 0 doc\n");
        write(&dir.join("v1.2 release notes/d1.md"), "state depth 1 doc\n");
        let loaded = load_dir(&dir).expect("loads");
        assert!(loaded.domains.contains(&"state.depth.0".to_string()));
        assert!(loaded.domains.contains(&"v1.2 release notes".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_dirs_refuse_with_the_reason() {
        // Missing directory.
        let missing = temp_dir("missing").join("nope");
        assert!(load_dir(&missing).is_err());
        // Empty directory.
        let empty = temp_dir("empty");
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).unwrap();
        assert!(load_dir(&empty).is_err());
        // A domain dir without any .md inside.
        let no_docs = temp_dir("nodoc");
        let _ = std::fs::remove_dir_all(&no_docs);
        std::fs::create_dir_all(no_docs.join("deploy")).unwrap();
        assert!(load_dir(&no_docs).is_err());
        // An empty document.
        let blank = temp_dir("blank");
        let _ = std::fs::remove_dir_all(&blank);
        write(&blank.join("deploy.md"), "   \n");
        assert!(load_dir(&blank).is_err());
        // A file/dir collision on one domain name.
        let dup = temp_dir("dup");
        let _ = std::fs::remove_dir_all(&dup);
        write(&dup.join("deploy.md"), "a doc");
        write(&dup.join("deploy/rollback.md"), "another doc");
        assert!(load_dir(&dup).is_err());
        for d in [&no_docs, &blank, &dup] {
            let _ = std::fs::remove_dir_all(d);
        }
    }

    #[test]
    fn over_the_ceiling_refuses_naming_the_count() {
        let dir = temp_dir("many");
        let _ = std::fs::remove_dir_all(&dir);
        for i in 0..=MAX_DOMAINS {
            write(&dir.join(format!("d{i:02}.md")), "doc text");
        }
        let err = load_dir(&dir).expect_err("refuses");
        assert!(err.contains(&format!("{}", MAX_DOMAINS + 1)), "got: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
