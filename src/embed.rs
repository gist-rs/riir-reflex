//! Hashed-feature text embedding — the modelless head.
//!
//! Deterministic, allocation-free, fixed width: words (lowercased ASCII,
//! punctuation-trimmed) + word bigrams are FNV-1a-hashed into `EMBED_DIM`
//! buckets with the sign trick, then L2-normalized (a zero vector passes
//! through — "no direction" reads as no signal, never NaN; the
//! `distance_abstain::unit` law). No learned state: the embedding is a
//! pure function, bit-identical across runs and platforms — the modelless
//! lane's determinism claim (Plan 603 caveat 3) starts HERE.

/// Latent width for the day-one engine. Sized for DOCUMENT-scale hashed
/// bags: a 50-word doc draws ~75 unigram+bigram hashes, and 256 buckets
/// keeps collisions sub-dominant (measured at birth: 64 buckets made every
/// dense doc vector near-random — the abstain geometry washed out to
/// ±0.02 confidence margin; 256 separates cleanly). Fixed-size arrays end
/// to end: the distance gate, the routing directions and the scratch all
/// carry `[f32; EMBED_DIM]`.
pub const EMBED_DIM: usize = 256;

/// Word-hash salt (domain separation — bigram hashes never collide with
/// word hashes by construction, not by luck).
const WORD_SALT: u64 = 0x3456_7890_1234_5678;
/// Bigram-hash salt.
const BIGRAM_SALT: u64 = 0x0f1e_2d3c_4b5a_6978;
const TRIGRAM_SALT: u64 = 0x7a3c_51e9_0d24_86bf;
/// Bigram weight (the pair feature is evidence, not identity).
const BIGRAM_WEIGHT: f32 = 0.5;

/// FNV-1a 64 over ASCII-lowercased bytes. Lowercasing happens in-hash (no
/// allocation on the hot path; non-ASCII bytes pass through deterministically).
#[inline]
pub(crate) fn fnv1a_word(bytes: &[u8], salt: u64) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ salt;
    for &b in bytes {
        let b = if b.is_ascii_uppercase() { b + 32 } else { b };
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Trim non-alphanumeric ASCII edges. Returns an empty slice for tokens
/// that are pure punctuation.
#[inline]
pub(crate) fn token(raw: &[u8]) -> &[u8] {
    // Treat non-ASCII bytes as part of the word, otherwise a Thai (or any
    // non-Latin) token trims to nothing and only its digits are kept.
    let is_sep = |b: u8| !(b.is_ascii_alphanumeric() || b >= 0x80);
    let mut a = 0usize;
    let mut b = raw.len();
    while a < b && is_sep(raw[a]) {
        a += 1;
    }
    while b > a && is_sep(raw[b - 1]) {
        b -= 1;
    }
    &raw[a..b]
}

#[inline]
fn add_hash(out: &mut [f32], h: u64, w: f32) {
    let bucket = (h as usize) % out.len();
    let sign = if h >> 63 == 1 { -1.0 } else { 1.0 };
    out[bucket] += w * sign;
}

/// The embedder. Zero-state on purpose: there is nothing to freeze, calibrate
/// or persist — the corpus IS the model (the engine's scoring head), this is
/// only the projection.
#[derive(Clone, Copy, Debug, Default)]
pub struct Embedder;

impl Embedder {
    /// Embed `text` into `out` (overwritten, then L2-normalized in place).
    /// Allocation-free.
    pub fn embed_into(&self, text: &[u8], out: &mut [f32]) {
        for v in out.iter_mut() {
            *v = 0.0;
        }
        let mut prev: Option<u64> = None;
        for raw in text.split(|b: &u8| b.is_ascii_whitespace()) {
            let t = token(raw);
            if t.is_empty() {
                continue;
            }
            let h = fnv1a_word(t, WORD_SALT);
            add_hash(out, h, 1.0);
            if let Some(p) = prev {
                let mut pair = [0u8; 16];
                pair[..8].copy_from_slice(&p.to_le_bytes());
                pair[8..].copy_from_slice(&h.to_le_bytes());
                add_hash(out, fnv1a_word(&pair, BIGRAM_SALT), BIGRAM_WEIGHT);
            }
            prev = Some(h);
        }
        // L2-normalize in place; zero passes through (never NaN).
        let n = out.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n > 0.0 {
            let inv = 1.0 / n;
            for v in out.iter_mut() {
                *v *= inv;
            }
        }
    }
}

/// Hashed token ids for count tables (issue 038, the `nb_scope` lane): the
/// SAME tokenizer and FNV-1a word/bigram hashes as [`Embedder::embed_into`]
/// (one lexicon, two projections), reduced `mod vocab` into `out`
/// (cleared first; capacity reused — zero-alloc once warm). Unsigned ids,
/// no weights: a count table wants integer events, and bigram evidence is
/// one event like a word.
pub fn hashed_tokens_into(text: &[u8], vocab: usize, out: &mut Vec<u32>) {
    debug_assert!(vocab > 0 && vocab <= u32::MAX as usize);
    out.clear();
    let mut prev: Option<u64> = None;
    for raw in text.split(|b: &u8| b.is_ascii_whitespace()) {
        let t = token(raw);
        if t.is_empty() {
            continue;
        }
        let h = fnv1a_word(t, WORD_SALT);
        out.push((h % vocab as u64) as u32);
        if let Some(p) = prev {
            let mut pair = [0u8; 16];
            pair[..8].copy_from_slice(&p.to_le_bytes());
            pair[8..].copy_from_slice(&h.to_le_bytes());
            out.push((fnv1a_word(&pair, BIGRAM_SALT) % vocab as u64) as u32);
        }
        prev = Some(h);
        if t.iter().any(|&b| b >= 0x80) {
            push_char_trigrams(t, vocab, out);
        }
    }
}

/// Character trigrams of a non-ASCII token.
/// Thai writes no spaces between words, so a whitespace token is a whole
/// phrase that never recurs; its trigrams do. ASCII-only tokens never get
/// here, so English features are unchanged.
fn push_char_trigrams(t: &[u8], vocab: usize, out: &mut Vec<u32>) {
    let Ok(s) = std::str::from_utf8(t) else {
        return;
    };
    let mut starts = [0usize; 3];
    for (n, (i, _)) in s.char_indices().enumerate() {
        if n >= 3 {
            let g = &t[starts[n % 3]..i];
            out.push((fnv1a_word(g, TRIGRAM_SALT) % vocab as u64) as u32);
        }
        starts[n % 3] = i;
    }
    if s.chars().count() >= 3 {
        let g = &t[starts[s.chars().count() % 3]..];
        out.push((fnv1a_word(g, TRIGRAM_SALT) % vocab as u64) as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thai_tokens_share_char_trigrams_and_ascii_is_unchanged() {
        let ids = |s: &str| {
            let mut v = Vec::new();
            hashed_tokens_into(s.as_bytes(), 1 << 20, &mut v);
            v
        };
        let tri = |s: &str| (fnv1a_word(s.as_bytes(), TRIGRAM_SALT) % (1 << 20)) as u32;
        let a = ids("ผ้าอนามัยแบบเย็น");
        let b = ids("ซื้อผ้าอนามัย");
        assert!(a.contains(&tri("อนา")) && b.contains(&tri("อนา")));
        assert!(a.contains(&tri("ย็น")), "last trigram emitted");
        assert_eq!(
            ids("hello world").len(),
            3,
            "ascii: 2 words + 1 bigram, no trigrams"
        );
    }

    #[test]
    fn thai_words_survive_tokenizing() {
        assert_eq!(token("ลด".as_bytes()), "ลด".as_bytes());
        assert_eq!(token("(ราคาสมาชิก)".as_bytes()), "ราคาสมาชิก".as_bytes());
        assert_eq!(token(b"\"hello,"), b"hello");
        let (mut a, mut b) = (Vec::new(), Vec::new());
        hashed_tokens_into("ลด 40%".as_bytes(), 1 << 20, &mut a);
        hashed_tokens_into("ราคาสมาชิก 40".as_bytes(), 1 << 20, &mut b);
        assert_ne!(a, b);
    }

    fn embed(s: &str) -> [f32; EMBED_DIM] {
        let mut v = [0.0f32; EMBED_DIM];
        Embedder.embed_into(s.as_bytes(), &mut v);
        v
    }

    #[test]
    fn deterministic_bit_identical() {
        let a = embed("deploy the server to staging and verify the rollout");
        let b = embed("deploy the server to staging and verify the rollout");
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(
                x.to_bits(),
                y.to_bits(),
                "same text must embed bit-identically"
            );
        }
    }

    #[test]
    fn unit_norm_and_zero_passes_through() {
        let a = embed("refund the customer invoice balance");
        let n: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((n - 1.0).abs() < 1e-5, "unit norm, got {n}");
        let z = embed("...");
        assert!(
            z.iter().all(|x| x.to_bits() == 0f32.to_bits()),
            "zero text stays zero (never NaN)"
        );
    }

    #[test]
    fn different_topics_land_differently() {
        let a = embed("deploy server staging rollout verify");
        let b = embed("refund customer invoice billing account");
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!(
            dot < 0.9,
            "disjoint topics must not collapse together (cos {dot})"
        );
        let a2 = embed("verify the staging rollout of the server deploy");
        let dot_same: f32 = a.iter().zip(a2.iter()).map(|(x, y)| x * y).sum();
        assert!(
            dot_same > dot,
            "same-topic variants must sit closer than disjoint topics"
        );
    }

    #[test]
    fn case_and_punctuation_insensitive() {
        let a = embed("Deploy: the server, NOW!");
        let b = embed("deploy the server now");
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(
                x.to_bits(),
                y.to_bits(),
                "case/punctuation must not move the embedding"
            );
        }
    }
}
