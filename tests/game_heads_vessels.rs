//! The heads→vessels lane (instinct Proposal 001 T4 — PUBLIC-RELEASE
//! class, lazy load).
//!
//! Pins, in order of what they protect:
//! 1. the MINT (fit → canonical payload → reflexer's public writer):
//!    fixture drift is refused BEFORE fitting; the mint is deterministic
//!    (same fixtures + key + version → byte-identical vessels);
//! 2. the ACCEPTANCE (the task's critical gate): a vessel-loaded head
//!    reproduces the fit digests — lanes `7d3f1d8e…` (the Bench 880
//!    published prefix) and flappy `c93d36dc…e3c5` (the Bench 882 full
//!    digest) hit their cross-repo anchors exactly, while tetris pins the
//!    measured v3 fit digest (a reflex-local serving serialization — see
//!    the const comment) — and its answers are BYTE-IDENTICAL to the
//!    fitted head's answers over fixture requests;
//! 3. the commitment mapping: the head digest pins stay digests of the
//!    WEIGHTS (recomputed from the loaded weights through the same
//!    bytes the fit side hashes), while the vessel's signed-region
//!    commitment pins the whole payload — mint-side == load-side, and
//!    strictly stronger (it covers the signature too);
//! 4. the fail-closed edges: a tampered vessel, a wrong pin table, and
//!    a corrupt payload all REFUSE (never install); the loud-absent
//!    posture answers `None` for everything and the healthz map reads
//!    all-false.
#![cfg(all(feature = "modelless", feature = "vessel_public_read"))]

use riir_reflex::game_heads::{
    absent_heads_message, head_vessels, GameHeads, HEAD_VESSEL_FILES,
};
use riir_reflex::serve::{LayaLane, demo_engine, serve_listener_heads};

use reflexer_vessel as vessel;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// The head anchors the loaded vessels must reproduce. Lanes is the Bench
/// 880 PUBLISHED prefix and flappy the Bench 882 FULL digest — the two
/// cross-repo anchors, hit exactly by the fit. Tetris is NOT one of them:
/// its digest is a reflex-local serving serialization (katgpt-rs
/// `.plans/609` G3: the `00aa6221…` figure was the v2-era serving digest,
/// "reflex's own serving serialization"), and the v2→v3 fixture refit
/// (Bench 892) moved it. The pin below is the MEASURED v3 fit digest —
/// deterministic (the fit is bit-deterministic, pinned in
/// `game_heads_serve.rs`), so the vessel must install exactly these
/// weights; the fitted-vs-loaded equality at the end of
/// `vessel_load_reproduces_the_published_digests` carries the rest.
const TETRIS_V3_FIT_DIGEST: &str =
    "8a1d76a05527468c5a9068b4b919685ea06a5020d3e7ca81a52d5477eb8d6032";
const LANES_HEAD_PREFIX: &str = "7d3f1d8e";
const FLAPPY_HEAD_FULL: &str =
    "c93d36dc79c0490334c20353ce5d6479eaee3448b686ac057f8a4ad4b98ae3c5";

const TETRIS_FIXTURE: &str = include_str!("../assets/game_heads/tetris_oracle_laya_en_v3.jsonl");
const LANES_FIXTURE: &str = include_str!("../assets/game_heads/lanes_oracle_laya_en_v1.jsonl");
const FLAPPY_FIXTURE: &str = include_str!("../assets/game_heads/flappy_oracle_laya_en_v3.jsonl");

fn test_key() -> vessel::ed25519_dalek::SigningKey {
    vessel::ed25519_dalek::SigningKey::from_bytes(&[42u8; 32])
}

fn test_pins() -> vessel::PinTable {
    vessel::PinTable::with_key(7, test_key().verifying_key())
}

/// Mint all three heads into a fresh temp dir; returns (dir, report).
fn mint_into(tag: &str) -> (PathBuf, Vec<head_vessels::MintedHead>) {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/game_heads");
    let out = std::env::temp_dir().join(format!("wave3_heads_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let report = head_vessels::mint_all(&fixtures, &out, &test_key(), 7, 1)
        .expect("mint from the repo fixtures");
    (out, report)
}

fn first_fixture_sentence() -> String {
    for line in TETRIS_FIXTURE.lines() {
        let v: serde_json::Value = serde_json::from_str(line).expect("fixture line parses");
        if v["state_id"] == "_meta" {
            continue;
        }
        return v["options"].as_array().expect("options")[0]["sentence"]
            .as_str()
            .expect("sentence")
            .to_string();
    }
    unreachable!("fixture has states");
}

fn first_lanes_turn() -> String {
    for line in LANES_FIXTURE.lines() {
        let v: serde_json::Value = serde_json::from_str(line).expect("fixture line parses");
        if v["state_id"] == "_meta" {
            continue;
        }
        return v["options"]
            .as_array()
            .expect("options")
            .iter()
            .map(|o| o["sentence"].as_str().expect("sentence").to_string())
            .collect::<Vec<_>>()
            .join("\n");
    }
    unreachable!("fixture has states");
}

fn first_flappy_pair() -> String {
    for line in FLAPPY_FIXTURE.lines() {
        let v: serde_json::Value = serde_json::from_str(line).expect("fixture line parses");
        if v["state_id"] == "_meta" {
            continue;
        }
        return format!(
            "{}\n{}",
            v["state_sentence"].as_str().expect("state sentence"),
            v["options"].as_array().expect("options")[0]["sentence"]
                .as_str()
                .expect("sentence")
        );
    }
    unreachable!("fixture has states");
}

/// One request per head shape (the gate corpus: a fixture-shaped request
/// for tetris / lanes / flappy each).
fn gate_requests(heads: &GameHeads) -> Vec<katgpt_core::decision_wire::DecisionRequest> {
    use katgpt_core::decision_wire::{Question, QuestionKind};
    let q = |prompt: &str| Question {
        id: "q0".into(),
        kind: QuestionKind::Noul,
        prompt: prompt.to_string(),
        options: vec![],
        criteria: None,
    };
    vec![
        katgpt_core::decision_wire::DecisionRequest {
            state: first_fixture_sentence(),
            questions: vec![q(heads.question())],
        },
        katgpt_core::decision_wire::DecisionRequest {
            state: first_lanes_turn(),
            questions: (0..3)
                .map(|i| Question {
                    id: format!("q{i}"),
                    kind: QuestionKind::Noul,
                    prompt: heads.lanes_question().to_string(),
                    options: vec![],
                    criteria: None,
                })
                .collect(),
        },
        katgpt_core::decision_wire::DecisionRequest {
            state: first_flappy_pair(),
            questions: vec![q(heads.flappy_question())],
        },
    ]
}

#[test]
fn the_mint_refuses_drifted_fixtures_before_fitting() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/game_heads");
    // A fixture dir whose tetris file is garbage: refused by BLAKE3 pin,
    // before any fit runs.
    let bad = std::env::temp_dir().join(format!("wave3_bad_fixtures_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&bad);
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("tetris_oracle_laya_en_v3.jsonl"), "{\"drifted\": true}\n").unwrap();
    for f in ["lanes_oracle_laya_en_v1.jsonl", "flappy_oracle_laya_en_v3.jsonl"] {
        std::fs::copy(fixtures.join(f), bad.join(f)).unwrap();
    }
    let out = std::env::temp_dir().join(format!("wave3_bad_out_{}", std::process::id()));
    let err = head_vessels::mint_all(&bad, &out, &test_key(), 7, 1)
        .expect_err("drifted fixture must refuse");
    assert!(
        err.contains("BLAKE3") && err.contains("refusing to mint"),
        "the refusal names the drift: {err}"
    );
    let _ = std::fs::remove_dir_all(&bad);
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn the_mint_is_deterministic_and_the_pins_hold() {
    let (dir, report) = mint_into("det_a");
    let (dir2, report2) = mint_into("det_b");
    assert_eq!(report, report2, "same fixtures + key + version → same report");
    for file in HEAD_VESSEL_FILES {
        let a = std::fs::read(dir.join(file)).unwrap();
        let b = std::fs::read(dir2.join(file)).unwrap();
        assert_eq!(a, b, "{file}: minting is deterministic (byte-identical vessels)");
    }
    // Every report row carries BOTH pins: the head digest (weights) and
    // the vessel commitment (signed region).
    for row in &report {
        assert_eq!(row.commitment_hex.len(), 64);
        assert_eq!(row.head_digest_hex.len(), 64);
        assert_ne!(row.commitment_hex, row.head_digest_hex);
    }
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn vessel_load_reproduces_the_published_digests() {
    let (dir, _report) = mint_into("digests");
    let loaded = GameHeads::from_vessel_dir(&dir, &test_pins()).expect("vessel dir loads");
    // THE ACCEPTANCE: the loaded heads carry the fit anchors.
    assert_eq!(
        loaded.digest_hex(),
        TETRIS_V3_FIT_DIGEST,
        "tetris head digest drifted from the v3 fit anchor"
    );
    let (lanes_lambda, lanes_digest, lanes_n) = loaded.lanes_fit();
    assert_eq!(lanes_lambda, 0.01, "lanes λ drifted");
    assert_eq!(lanes_n, 300, "lanes corpus size drifted");
    assert!(
        lanes_digest.starts_with(LANES_HEAD_PREFIX),
        "lanes head digest drifted: {lanes_digest}"
    );
    let (flappy_lambda, flappy_digest, flappy_n) = loaded.flappy_fit();
    assert_eq!(flappy_lambda, 1.0, "flappy λ drifted");
    assert_eq!(flappy_n, 200, "flappy corpus size drifted");
    assert_eq!(
        flappy_digest, FLAPPY_HEAD_FULL,
        "flappy v3 decoded head digest drifted from the Bench 882 full anchor"
    );
    // The digest is of the WEIGHTS — identical to the fitted head's.
    let fitted = GameHeads::build(TETRIS_FIXTURE, LANES_FIXTURE, FLAPPY_FIXTURE);
    assert_eq!(loaded.digest_hex(), fitted.digest_hex());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vessel_answers_are_byte_identical_to_the_fitted_answers() {
    let (dir, report) = mint_into("identity");
    let fitted = GameHeads::build(TETRIS_FIXTURE, LANES_FIXTURE, FLAPPY_FIXTURE);
    let loaded = GameHeads::from_vessel_dir(&dir, &test_pins()).expect("vessel dir loads");
    for req in gate_requests(&fitted) {
        let a = fitted
            .respond(&req)
            .expect("the fitted head answers the fixture shape");
        let b = loaded
            .respond(&req)
            .expect("the vessel head answers the same shape");
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap(),
            "vessel-served answer drifted from the fitted answer"
        );
    }
    // And the commitments recorded at mint are the ones the loaded
    // vessels re-derive (re-open each minted file and compare).
    let keys = ["tetris.vessel", "lanes.vessel", "flappy.vessel"];
    for (row, file) in report.iter().zip(keys) {
        let bytes = std::fs::read(dir.join(file)).unwrap();
        let verified = vessel::decode(&bytes, &test_pins()).expect("re-open");
        assert_eq!(verified.commitment_hex(), row.commitment_hex);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_wire_serves_vessel_heads_end_to_end() {
    let (dir, _report) = mint_into("wire");
    let loaded = Arc::new(GameHeads::from_vessel_dir(&dir, &test_pins()).expect("loads"));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let eng = Arc::new(Mutex::new(demo_engine()));
    let laya = Arc::new(Mutex::new(LayaLane::Off));
    std::thread::spawn(move || {
        let _ = serve_listener_heads(listener, eng, laya, vec![], loaded);
    });

    use std::io::{BufRead, BufReader, Write};
    let mut s = std::net::TcpStream::connect(&addr).unwrap();
    // healthz advertises the loaded heads.
    s.write_all(b"GET /healthz HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut body = String::new();
    for line in BufReader::new(s.try_clone().unwrap()).lines() {
        body.push_str(&line.expect("healthz body line"));
    }
    assert!(body.contains("\"tetris\":true"), "heads map: {body}");
    assert!(body.contains("\"lanes\":true"), "heads map: {body}");
    assert!(body.contains("\"flappy\":true"), "heads map: {body}");

    // A fixture spot question is answered from the VESSEL-loaded head.
    let fitted = GameHeads::build(TETRIS_FIXTURE, LANES_FIXTURE, FLAPPY_FIXTURE);
    let req = &gate_requests(&fitted)[0];
    let body_json = serde_json::to_string(req).unwrap();
    let http = format!(
        "POST /decide HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_json}",
        body_json.len()
    );
    let mut s = std::net::TcpStream::connect(&addr).unwrap();
    s.write_all(http.as_bytes()).unwrap();
    let mut reader = BufReader::new(s);
    let mut status = String::new();
    reader.read_line(&mut status).unwrap();
    assert!(status.contains("200"), "status: {status}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_tampered_or_mispinned_vessel_refuses_never_installs() {
    let (dir, _report) = mint_into("tamper");
    // Tamper one payload byte of the tetris vessel.
    let p = dir.join("tetris.vessel");
    let mut bytes = std::fs::read(&p).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    std::fs::write(&p, &bytes).unwrap();
    let err = GameHeads::from_vessel_dir(&dir, &test_pins())
        .err()
        .expect("a tampered vessel must refuse");
    assert!(err.to_string().contains("refused"), "err: {err}");
    assert!(!GameHeads::from_vessel_dir(&dir, &test_pins()).is_ok());
    // A wrong pin table refuses the untampered dir too (fail-closed on
    // unknown keys).
    let (dir2, _) = mint_into("wrongpins");
    let wrong = vessel::PinTable::with_key(99, test_key().verifying_key());
    assert!(
        GameHeads::from_vessel_dir(&dir2, &wrong).is_err(),
        "an unpinned key must refuse"
    );
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn a_corrupt_payload_refuses_even_under_a_valid_signature() {
    // A vessel signed over NON-head-payload bytes: the signature is fine
    // (our key, pinned), the payload parser must refuse — an authentic
    // file that is not a head payload never becomes a head.
    let payload = b"definitely not an RFXH head payload".to_vec();
    let minted = vessel::writer::sign_public(&test_key(), 7, 1, [0u8; 32], &payload).unwrap();
    let dir = std::env::temp_dir().join(format!("wave3_corrupt_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tetris.vessel"), &minted.bytes).unwrap();
    let err = GameHeads::from_vessel_dir(&dir, &test_pins())
        .err()
        .expect("a non-head payload must refuse");
    assert!(
        err.to_string().contains("head payload refused"),
        "err: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_lane_is_absent_others_still_serve() {
    let (dir, _report) = mint_into("partial");
    std::fs::remove_file(dir.join("lanes.vessel")).unwrap();
    let heads = GameHeads::from_vessel_dir(&dir, &test_pins()).expect("partial dir loads");
    assert!(heads.has_tetris());
    assert!(!heads.has_lanes(), "the removed lane is absent");
    assert!(heads.has_flappy());
    // The absent lane's shape falls through; the present ones answer.
    let fitted = GameHeads::build(TETRIS_FIXTURE, LANES_FIXTURE, FLAPPY_FIXTURE);
    let reqs = gate_requests(&fitted);
    assert!(heads.respond(&reqs[0]).is_some(), "tetris still serves");
    assert!(heads.respond(&reqs[1]).is_none(), "the absent lanes fall through");
    assert!(heads.respond(&reqs[2]).is_some(), "flappy still serves");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_absent_posture_answers_none_and_the_loud_line_names_the_mint() {
    let heads = GameHeads::absent();
    assert!(!heads.has_tetris() && !heads.has_lanes() && !heads.has_flappy());
    let fitted = GameHeads::build(TETRIS_FIXTURE, LANES_FIXTURE, FLAPPY_FIXTURE);
    for req in gate_requests(&fitted) {
        assert!(heads.respond(&req).is_none(), "an absent lane never answers");
    }
    let msg = absent_heads_message("RIIR_REFLEX_HEADS_DIR");
    assert!(msg.contains("RIIR_REFLEX_HEADS_DIR"), "{msg}");
    assert!(msg.contains("mint-heads"), "{msg}");
    assert!(msg.contains("RIIR_REFLEX_HEADS_PUBKEY"), "{msg}");
}
