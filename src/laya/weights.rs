//! Checkpoint file resolution: locate → verify → (download).
//!
//! Pins (`.docs/laya_reference_pin.md`): weights are verified by the
//! SHA-256 full-file digests (the HF LFS oids — an external fact, so the
//! house blake3 rule yields to the pin's own hash family); the 12 small
//! files are blake3-pinned. Files are RUNTIME-DOWNLOADED from the hub,
//! never bundled.
//!
//! Hub layout (`convaiinnovations/laya`, the reference's own subfolder
//! packaging): the ENGLISH checkpoint sits at the repo ROOT; the other two
//! live under `multilingual/` and `typed-decisions/`. Within a checkpoint:
//! `model.safetensors`, `rl_agent_config.json`,
//! `tokenizer/tokenizer.json`, `tokenizer/tokenizer_config.json`, and
//! `encoder/config.json` (the hub names the encoder config `config.json`;
//! the pin doc's `encoder_config.json` is the flat local name).
//!
//! Local layout: `<root>/<local-sub>/` with FLAT names (the pin layout —
//! `.raw/laya-hf`). Two locals are readable without copying:
//! 1. the flat cache (`~/.cache/riir-reflex/laya`, `LAYA_HOME`);
//! 2. an HF SNAPSHOT dir handed via `LAYA_WEIGHTS_DIR` — its hub-shaped
//!    relative paths resolve directly (symlinks into the blob store).
//!
//! A missing file downloads to the flat local name. A PRESENT file that
//! fails its pin is a hard error — never a silent re-download (a wrong
//! local file is a fact to surface, not to paper over).
//!
//! Transport: `curl` subprocess (follows the HF resolve redirect chain).
//! Deliberate for this lane — the download runs once per machine, curl is
//! ubiquitous, and it keeps rustls/reqwest out of the dep tree; a
//! pure-Rust transport is a Phase-2 serving concern, not a gate concern.
//! Downloads go to `<name>.part` and rename only after the hash verifies —
//! a killed download can never leave a plausible-looking wrong file.

use std::path::{Path, PathBuf};
use std::process::Command;

use blake3::Hasher;

use super::config::Checkpoint;
use super::{LayaError, Result};

/// The hub repo the three checkpoints ship in (the reference's own default
/// `model_id`).
pub const HF_REPO: &str = "convaiinnovations/laya";

/// The SHA-256 pin per checkpoint weight file (full-file digest = the HF
/// LFS oid; verified from the tree API at pin time).
pub const WEIGHT_SHA256: [(Checkpoint, &str); 3] = [
    (
        Checkpoint::English,
        "891102d372688fc2a094dac56a384bc537b87c63f21f9f3dac0be2b7cbc8d86c",
    ),
    (
        Checkpoint::Multilingual,
        "9d628fd971b700382ac6f65920a86f149777b2e748e0c955fb3b19695aa8f204",
    ),
    (
        Checkpoint::TypedDecisions,
        "4fa56de72383a9d3efa9cfa78955733c81b9fc8067a587ca4beb82c78107a24e",
    ),
];

/// The blake3 pin per small file, keyed `(local subfolder, local name,
/// blake3 hex)` — the pin doc's table verbatim.
#[rustfmt::skip]
pub const SMALL_BLAKE3: [(&str, &str, &str); 12] = [
    ("english",      "tokenizer.json",        "d8d4c8b30443535d6416e4f542d0a77e7f10e2809df063d11f7642a667037030"),
    ("english",      "tokenizer_config.json", "8b275b6f13f464ceb2e6268c566ab3b40c8bfdcd6af7a5a299539194450083f0"),
    ("english",      "encoder_config.json",   "3b1e7e84b9d90a3835a55ed2a81395698e1d143f3553123c1e498649c7464098"),
    ("english",      "rl_agent_config.json",  "dbc03fdb75def9d9ea5218dd6df470d4bb5842584ea372bd9232ded658ed0f8f"),
    ("multilingual", "tokenizer.json",        "01e0f0d31015fa48f99c5e7aea639fbf136181e632cfbf40156862aced7b20b1"),
    ("multilingual", "tokenizer_config.json", "8486197d7556f869092d214857a10fc9b75f8108250197ba08e7e9df8dba6637"),
    ("multilingual", "encoder_config.json",   "a831925d30809ab5bb2ad5424eff016a422310a8989758c44631748276eee06a"),
    ("multilingual", "rl_agent_config.json",  "dd2da6b7f43afd2babffa290bb1857784e49d825e6f76c5c0ab1d0fbdf441714"),
    ("typed",        "tokenizer.json",        "d8d4c8b30443535d6416e4f542d0a77e7f10e2809df063d11f7642a667037030"),
    ("typed",        "tokenizer_config.json", "66b37468dfcfbc4b9022c2026d602bbd3dac39903fccd4361d81caa5e3ab4056"),
    ("typed",        "encoder_config.json",   "e724310ec3024cb84e2d7ad571f1ff20f2f3d4c1021b18cc43140a51a0a3a02e"),
    ("typed",        "rl_agent_config.json",  "4fbe491144bb1b3119a7ad308b6942523e85b9b35d554329b8d95d17e71e51d7"),
];

/// The files one checkpoint needs: `(local flat name, hub-relative path)`.
const FILES: [(&str, &str); 5] = [
    ("model.safetensors", "model.safetensors"),
    ("rl_agent_config.json", "rl_agent_config.json"),
    ("tokenizer.json", "tokenizer/tokenizer.json"),
    ("tokenizer_config.json", "tokenizer/tokenizer_config.json"),
    ("encoder_config.json", "encoder/config.json"),
];

/// The hub repo prefix for a checkpoint (english IS the repo root).
fn hub_prefix(ckpt: Checkpoint) -> &'static str {
    match ckpt {
        Checkpoint::English => "",
        Checkpoint::Multilingual => "multilingual",
        Checkpoint::TypedDecisions => "typed-decisions",
    }
}

/// Resolve (or create) the weights root.
pub fn weights_root() -> PathBuf {
    if let Ok(dir) = std::env::var("LAYA_WEIGHTS_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("LAYA_HOME") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".cache")
        .join("riir-reflex")
        .join("laya")
}

/// The sha256 pin for one checkpoint's weights.
fn weight_pin(ckpt: Checkpoint) -> &'static str {
    WEIGHT_SHA256
        .iter()
        .find(|(c, _)| *c == ckpt)
        .map(|(_, pin)| *pin)
        .expect("all three checkpoints pinned")
}

fn small_pin(local_sub: &str, local_name: &str) -> &'static str {
    SMALL_BLAKE3
        .iter()
        .find(|(s, f, _)| *s == local_sub && *f == local_name)
        .map_or_else(
            || panic!("small file {local_sub}/{local_name} pinned"),
            |(_, _, pin)| *pin,
        )
}

/// Ensure every file of `ckpt` exists under `root` (either layout) and
/// matches its pin, downloading what is missing. Returns the local
/// (flat-named) checkpoint directory.
pub fn ensure_checkpoint(root: &Path, ckpt: Checkpoint) -> Result<PathBuf> {
    let local_sub = ckpt.subfolder();
    let dir = root.join(local_sub);
    if !dir.is_dir() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| LayaError::Runtime(format!("create {}: {e}", dir.display())))?;
    }
    for (local_name, hub_rel) in FILES {
        let kind = if local_name == "model.safetensors" {
            HashKind::Sha256
        } else {
            HashKind::Blake3
        };
        let pin = if local_name == "model.safetensors" {
            weight_pin(ckpt)
        } else {
            small_pin(local_sub, local_name)
        };
        let flat = dir.join(local_name);
        if flat.exists() {
            verify(&flat, pin, kind, ckpt, local_name)?;
            continue;
        }
        // Hub-shaped candidate beside the flat name (an HF snapshot root
        // passed via LAYA_WEIGHTS_DIR): readable in place, no copy.
        let hub_shaped = dir.join(hub_rel);
        if hub_shaped.exists() {
            verify(&hub_shaped, pin, kind, ckpt, local_name)?;
            // The agent reads FLAT names — link the hub-shaped file into
            // place (symlink, copy as the portable fallback).
            link_or_copy(&hub_shaped, &flat)?;
            continue;
        }
        let sub = hub_prefix(ckpt);
        let url = match sub {
            "" => format!("https://huggingface.co/{HF_REPO}/resolve/main/{hub_rel}"),
            p => format!("https://huggingface.co/{HF_REPO}/resolve/main/{p}/{hub_rel}"),
        };
        fetch_to_flat(&url, &flat, pin, kind, ckpt, local_name)?;
    }
    Ok(dir)
}

/// Link `src` to `dst` (symlink on unix; byte copy where symlinks fail).
fn link_or_copy(src: &Path, dst: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink(src, dst).is_ok() {
            return Ok(());
        }
    }
    std::fs::copy(src, dst)
        .map(|_| ())
        .map_err(|e| LayaError::Runtime(format!("link {}: {e}", dst.display())))
}

enum HashKind {
    Blake3,
    Sha256,
}

impl HashKind {
    fn name(self) -> &'static str {
        match self {
            Self::Blake3 => "blake3",
            Self::Sha256 => "sha256",
        }
    }
}

/// Stream a URL to `dest.part`, verify, then rename into place.
fn fetch_to_flat(
    url: &str,
    dest: &Path,
    pin: &str,
    kind: HashKind,
    ckpt: Checkpoint,
    local_name: &str,
) -> Result<()> {
    let tmp = dest.with_extension("part");
    let status = Command::new("curl")
        .args([
            "--location",
            "--silent",
            "--show-error",
            "--fail",
            "--output",
        ])
        .arg(&tmp)
        .arg(url)
        .status()
        .map_err(|e| LayaError::Runtime(format!("spawn curl: {e}")))?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return Err(LayaError::Runtime(format!(
            "download failed ({url}): {status}"
        )));
    }
    if let Err(e) = verify(&tmp, pin, kind, ckpt, local_name) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, dest)
        .map_err(|e| LayaError::Runtime(format!("rename {}: {e}", dest.display())))?;
    Ok(())
}

/// Hash `path` and compare against `pin`.
fn verify(path: &Path, pin: &str, kind: HashKind, ckpt: Checkpoint, file: &str) -> Result<()> {
    let bytes = std::fs::read(path)
        .map_err(|e| LayaError::Runtime(format!("read {}: {e}", path.display())))?;
    let got = match kind {
        HashKind::Blake3 => {
            let mut h = Hasher::new();
            h.update(&bytes);
            h.finalize().to_hex().to_string()
        }
        HashKind::Sha256 => {
            use sha2::Digest;
            let mut h = sha2::Sha256::new();
            h.update(&bytes);
            format!("{:x}", h.finalize())
        }
    };
    if got != pin {
        return Err(LayaError::Pin {
            checkpoint: ckpt.subfolder(),
            file: file.to_string(),
            detail: format!("{} {} != pinned {pin}", kind.name(), got),
        });
    }
    Ok(())
}
