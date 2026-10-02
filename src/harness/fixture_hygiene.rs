//! The ONE fixture-hygiene checker — shared by every in-process fixture
//! authoring (ungated + pure).
//!
//! WHY THIS EXISTS (the recurring-bug root cause, 2026-10-02): the
//! workspace measured the same contamination class four times before
//! landing one shared instrument —
//!
//! 1. the DATASET suites' near-duplicate train/test leakage (Issue 024,
//!    the `slice_leak` probe);
//! 2. the frozen t20k sst5 pool's cross-split + train-internal duplicates
//!    (the `slice_guard` KNOWN_DIRTY pin — published rows measured on
//!    dirty bytes);
//! 3. `code_fixtures`' template-shared eval (the specialist lane's
//!    unfalsifiable-memorization refusal, instinct issue 008 T8);
//! 4. the six harness families' evals — measured at the 059 wide-eval
//!    landing: 0.68–0.78 mean unigram overlap against corpus∪cal,
//!    per-case maxes at 1.00, 2–7 trigram-hit cases each — the published
//!    family accuracies were corpus-overlap inflation.
//!
//! Each instance got its OWN private instrument after the fact, and the
//! asserts that DID exist were structurally blind: exact-equality
//! disjointness (`assert_ne!` on whole texts) cannot see two texts
//! sharing 90% of their vocabulary; accuracy floors gate DOWNWARD only
//! (they catch a systematically wrong engine, never an inflated one).
//! This module is the shared vocabulary layer so the next fixture set
//! starts from a checker instead of from raw prose — and the fleet audit
//! (`tests/fixture_fleet_hygiene.rs`) derives its population from the
//! registry, so a new synthetic suite cannot land without hygiene facts.
//!
//! The property it measures — bounded STATISTICAL overlap between the
//! eval slice and the corpus/cal slices — is deliberately stricter than
//! exact disjointness: the memorization path lives in the gap between
//! "not the same string" and "not the same vocabulary."

use std::collections::HashSet;

/// Lowercased alphanumeric word tokens (apostrophes stripped — `tree's`
/// reads as one token `trees`, matching the digest/hygiene convention
/// every consumer pins against).
#[must_use]
pub fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .replace('\'', "")
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// The set of contiguous word 3-grams (case/punctuation-insensitive).
#[must_use]
pub fn trigrams(text: &str) -> HashSet<[String; 3]> {
    let t = tokens(text);
    let mut out = HashSet::new();
    if t.len() >= 3 {
        for w in t.windows(3) {
            out.insert([w[0].clone(), w[1].clone(), w[2].clone()]);
        }
    }
    out
}

/// One fixture slice's hygiene facts. `mean_unigram_overlap` is the mean
/// share of an eval case's tokens that appear in the corpus∪cal token
/// pool; `max_unigram_overlap` is the worst single case;
/// `trigram_hit_cases` counts eval cases sharing at least one word
/// 3-gram with any corpus or cal text (the memorization-path wall — the
/// wide authoring law holds it at ZERO); `trigram_examples` carries up to
/// three offending eval texts for the failure message.
#[derive(Debug, Clone)]
pub struct SliceHygiene {
    pub eval_n: usize,
    pub mean_unigram_overlap: f64,
    pub max_unigram_overlap: f64,
    pub trigram_hit_cases: usize,
    pub trigram_examples: Vec<String>,
}

/// Audit one fixture's three slices: eval against corpus∪cal. Any slice
/// may be empty (an eval-only fixture reports zero overlaps honestly).
#[must_use]
pub fn audit_slices(corpus: &[&str], cal: &[&str], eval: &[&str]) -> SliceHygiene {
    let mut pool: HashSet<String> = HashSet::new();
    for ct in corpus {
        pool.extend(tokens(ct));
    }
    for c in cal {
        pool.extend(tokens(c));
    }
    let mut wall: HashSet<[String; 3]> = HashSet::new();
    for ct in corpus {
        wall.extend(trigrams(ct));
    }
    for c in cal {
        wall.extend(trigrams(c));
    }
    let mut sum = 0.0f64;
    let mut max = 0.0f64;
    let mut hit_cases = 0usize;
    let mut examples: Vec<String> = Vec::new();
    for e in eval {
        let toks = tokens(e);
        let hits = toks.iter().filter(|w| pool.contains(*w)).count();
        let o = if toks.is_empty() {
            0.0
        } else {
            hits as f64 / toks.len() as f64
        };
        sum += o;
        max = max.max(o);
        let et = trigrams(e);
        if et.intersection(&wall).next().is_some() {
            hit_cases += 1;
            if examples.len() < 3 {
                examples.push((*e).to_string());
            }
        }
    }
    SliceHygiene {
        eval_n: eval.len(),
        mean_unigram_overlap: if eval.is_empty() {
            0.0
        } else {
            sum / eval.len() as f64
        },
        max_unigram_overlap: max,
        trigram_hit_cases: hit_cases,
        trigram_examples: examples,
    }
}

/// Label-token violations: the (index, offending token) pairs where an
/// eval text carries one of the banned tokens as a whole word. The ban
/// list is the FAMILY's law (its labels, or the tool names an intent
/// must never name) — the caller owns it.
#[must_use]
pub fn label_token_violations(bans: &[&str], texts: &[&str]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, t) in texts.iter().enumerate() {
        for w in tokens(t) {
            if bans.iter().any(|b| *b == w) {
                out.push((i, w));
            }
        }
    }
    out
}

/// The canonical fixture digest: the fixture's name, then per case
/// `gold 0x1F text 0x1E`, BLAKE3-hexed. ANY fixture edit (text, gold,
/// order, insertion, deletion) moves the digest — the consumer pins it
/// so an edit reds its gate until consciously re-pinned in the same
/// change.
#[must_use]
pub fn fixture_digest(name: &str, eval: &[(usize, &str)]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0x1E);
    for (gold, text) in eval {
        bytes.extend_from_slice(gold.to_string().as_bytes());
        bytes.push(0x1F);
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(0x1E);
    }
    blake3::hash(&bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_lowercase_words_with_apostrophes_stripped() {
        assert_eq!(
            tokens("The tree's Bark, and 42 camels!"),
            vec!["the", "trees", "bark", "and", "42", "camels"]
        );
    }

    #[test]
    fn trigrams_are_case_and_punctuation_insensitive() {
        let a = trigrams("The quick brown fox.");
        let b = trigrams("the QUICK, brown FOX");
        assert_eq!(a, b);
        let expected: HashSet<[String; 3]> = [[
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
        ]]
        .into_iter()
        .collect();
        assert!(expected.is_subset(&a));
    }

    #[test]
    fn audit_detects_vocabulary_sharing_that_exact_disjointness_misses() {
        let corpus = ["Rust systems must compile deterministically across machines."];
        let eval = ["Rust systems must compile deterministically across clusters."];
        // Exact-equality disjointness would pass; the trigram wall must not.
        let h = audit_slices(&corpus, &[], &eval);
        assert_eq!(h.trigram_hit_cases, 1);
        assert!(h.mean_unigram_overlap > 0.8);
        assert_eq!(h.trigram_examples.len(), 1);
    }

    #[test]
    fn audit_clean_slices_report_zero() {
        let corpus = ["Alpha beta gamma delta epsilon."];
        let eval = ["Completely different vocabulary appears here now."];
        let h = audit_slices(&corpus, &[], &eval);
        assert_eq!(h.trigram_hit_cases, 0);
        assert!(h.mean_unigram_overlap < 0.2);
    }

    #[test]
    fn label_violations_carry_index_and_token() {
        let v = label_token_violations(&["deny"], &["clean text", "never deny this"]);
        assert_eq!(v, vec![(1, "deny".to_string())]);
    }

    #[test]
    fn digest_moves_on_any_edit() {
        let a = fixture_digest("f", &[(0, "one text")]);
        let b = fixture_digest("f", &[(0, "one text")]);
        let c = fixture_digest("f", &[(1, "one text")]);
        let d = fixture_digest("g", &[(0, "one text")]);
        assert_eq!(a, b, "same bytes → same digest");
        assert_ne!(a, c, "gold edit moves the digest");
        assert_ne!(a, d, "name edit moves the digest");
    }
}
