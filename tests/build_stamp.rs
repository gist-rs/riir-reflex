//! The build-stamp gates (Plan 606 T1.1) — the STALE marker must name the
//! missing release feature exactly when it is missing, and never when the
//! set is complete. Ungated: the stamp renders in every build, so its gates
//! run in every build too (no `#![cfg]` whole-file gate — no
//! required-features row needed).

use riir_reflex::build_stamp::{COMPILED_FEATURES, RELEASE_FEATURES, missing, stamp};

#[test]
fn release_features_are_sorted_and_complete() {
    let mut sorted = RELEASE_FEATURES.to_vec();
    sorted.sort_unstable();
    assert_eq!(
        sorted, RELEASE_FEATURES,
        "RELEASE_FEATURES must stay sorted"
    );
    assert!(
        RELEASE_FEATURES.contains(&"modelless"),
        "the engine is the product"
    );
    assert!(
        RELEASE_FEATURES.contains(&"laya-riir"),
        "the riir-owned comparison lane rides the release binary (.issues/006)"
    );
}

#[test]
fn stamp_names_version_and_compiled_set() {
    let s = stamp();
    assert!(s.contains(riir_reflex::VERSION), "got: {s}");
    assert!(s.contains("compiled features:"), "got: {s}");
    for f in COMPILED_FEATURES {
        assert!(s.contains(f), "compiled feature {f} must render — got: {s}");
    }
}

#[cfg(not(feature = "laya-riir"))]
#[test]
fn incomplete_build_prints_stale_naming_the_gap_and_rebuild() {
    assert_eq!(missing(), vec!["laya-riir"]);
    let s = stamp();
    assert!(s.contains("STALE — missing laya-riir"), "got: {s}");
    assert!(
        s.contains(riir_reflex::build_stamp::rebuild_command().as_str()),
        "got: {s}"
    );
}

#[cfg(feature = "laya-riir")]
#[test]
fn complete_release_set_never_prints_stale() {
    assert!(missing().is_empty(), "got: {:?}", missing());
    let s = stamp();
    assert!(!s.contains("STALE"), "got: {s}");
}
