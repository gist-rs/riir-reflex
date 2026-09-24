//! Near-duplicate leak report (Issue 024): does an eval row have an exact or
//! near twin in the rows the engine was fed?
//!
//! Modelless and zero-dep. Normalise (lowercase, collapse whitespace, strip
//! everything but `[a-z0-9 ]`), take char `k`-gram shingles, build an
//! inverted index over the reference rows that skips shingles held by
//! `>= rare_cap` rows, keep the `top_c` candidates by shared-shingle count,
//! and score each by Jaccard. A query row is EXACT when its normalised text
//! is a reference row's, NEAR when its best Jaccard is `>= near_j`.
//!
//! The method, every constant, and the candidate tie-break (count desc,
//! reference index asc) mirror `scripts/slice_leak_probe.py`, which is this
//! module's known-answer oracle (`tests/slice_leak_oracle.rs`, gate G1).
//! A divergence in any of them is a G1 failure, not a tuning choice.
//!
//! Build once per suite. [`ShingleIndex::classify`] allocates nothing once
//! its [`QueryScratch`] has warmed to the longest row (gate G2).

use std::collections::HashMap;

/// Shingle width (chars). The packed representation holds up to 8.
pub const DEFAULT_K: usize = 4;
/// Jaccard at or above which a row is a NEAR twin.
pub const DEFAULT_NEAR_J: f64 = 0.8;
/// Shingles held by at least this many reference rows are skipped.
pub const DEFAULT_RARE_CAP: usize = 200;
/// Candidates scored per query row.
pub const DEFAULT_TOP_C: usize = 5;
/// Query rows scanned for NEAR (the EXACT check scans every row). The
/// probe's cap; a report over the real slices may lift it.
pub const DEFAULT_NEAR_CAP: usize = 2000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeakParams {
    pub k: usize,
    pub near_j: f64,
    pub rare_cap: usize,
    pub top_c: usize,
    /// `None` scans every query row for NEAR.
    pub near_cap: Option<usize>,
}

impl Default for LeakParams {
    fn default() -> Self {
        Self {
            k: DEFAULT_K,
            near_j: DEFAULT_NEAR_J,
            rare_cap: DEFAULT_RARE_CAP,
            top_c: DEFAULT_TOP_C,
            near_cap: Some(DEFAULT_NEAR_CAP),
        }
    }
}

/// Python `str.isspace()`: Unicode `White_Space` plus the four ASCII
/// separators `\x1c..=\x1f`, which Python counts and Rust does not.
fn py_isspace(c: char) -> bool {
    c.is_whitespace() || ('\x1c'..='\x1f').contains(&c)
}

/// The probe's `norm`: `re.sub(r"[^a-z0-9 ]", "", re.sub(r"\s+", " ",
/// s.lower())).strip()`. Whitespace collapses BEFORE the strip, so a removed
/// char between two spaces leaves both (`"a é b"` → `"a  b"`), exactly as
/// the two-pass regex does.
pub fn normalize_into(s: &str, out: &mut String) {
    out.clear();
    let mut in_ws = false;
    for c in s.chars().flat_map(char::to_lowercase) {
        if py_isspace(c) {
            if !in_ws {
                out.push(' ');
            }
            in_ws = true;
            continue;
        }
        in_ws = false;
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        }
    }
    let end = out.trim_end_matches(' ').len();
    out.truncate(end);
    let start = out.len() - out.trim_start_matches(' ').len();
    out.drain(..start);
}

pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    normalize_into(s, &mut out);
    out
}

/// Pack a gram's bytes into a `u64`. Normalised text never contains a zero
/// byte, so a short gram (the probe keeps the whole string when it is
/// shorter than `k`, including the empty one) cannot collide with a full one.
fn pack(gram: &[u8]) -> u64 {
    gram.iter().fold(0u64, |acc, &b| (acc << 8) | u64::from(b))
}

/// The probe's `shingles` over already-normalised text, as a sorted,
/// deduplicated set in `out`.
pub fn shingles_into(norm: &str, k: usize, out: &mut Vec<u64>) {
    out.clear();
    let b = norm.as_bytes();
    match b.len() < k {
        true => out.push(pack(b)),
        false => out.extend(b.windows(k).map(pack)),
    }
    out.sort_unstable();
    out.dedup();
}

/// `|a ∩ b|` over two sorted, deduplicated sets.
fn intersection_len(a: &[u64], b: &[u64]) -> usize {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

/// What a query row's nearest reference row is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LeakClass {
    /// Normalised text equals reference row `row` (the FIRST such row).
    Exact { row: u32 },
    /// Best Jaccard `>= near_j`, against reference row `row`.
    Near { row: u32, jaccard: f64 },
    /// Neither. `best` is the best Jaccard seen (0.0 with no candidate).
    Clean { best: f64 },
}

/// Per-query reusable buffers. Size it with [`QueryScratch::for_index`].
pub struct QueryScratch {
    norm: String,
    grams: Vec<u64>,
    counts: Vec<u32>,
    touched: Vec<u32>,
    top: Vec<(u32, u32)>,
}

impl QueryScratch {
    pub fn for_index(index: &ShingleIndex) -> Self {
        Self {
            norm: String::with_capacity(256),
            grams: Vec::with_capacity(256),
            counts: vec![0; index.n_rows()],
            touched: Vec::with_capacity(index.n_rows().min(4096)),
            top: Vec::with_capacity(index.params.top_c),
        }
    }
}

/// The reference side: per-row shingle sets and an inverted index, both
/// flat CSR arrays, plus the exact-text map.
pub struct ShingleIndex {
    params: LeakParams,
    row_off: Vec<u32>,
    row_grams: Vec<u64>,
    keys: Vec<u64>,
    post_off: Vec<u32>,
    postings: Vec<u32>,
    exact: HashMap<String, u32>,
}

impl ShingleIndex {
    pub fn build<S: AsRef<str>>(rows: &[S], params: LeakParams) -> Self {
        assert!(
            (1..=8).contains(&params.k),
            "shingle width must be 1..=8, got {}",
            params.k
        );
        let mut row_off = Vec::with_capacity(rows.len() + 1);
        row_off.push(0u32);
        let mut row_grams = Vec::new();
        let mut exact = HashMap::with_capacity(rows.len());
        let (mut norm, mut grams) = (String::new(), Vec::new());
        for (i, row) in rows.iter().enumerate() {
            normalize_into(row.as_ref(), &mut norm);
            shingles_into(&norm, params.k, &mut grams);
            row_grams.extend_from_slice(&grams);
            row_off.push(row_grams.len() as u32);
            exact.entry(norm.clone()).or_insert(i as u32);
        }

        let mut pairs: Vec<(u64, u32)> = Vec::with_capacity(row_grams.len());
        for r in 0..rows.len() {
            let (a, b) = (row_off[r] as usize, row_off[r + 1] as usize);
            pairs.extend(row_grams[a..b].iter().map(|&g| (g, r as u32)));
        }
        pairs.sort_unstable();
        let mut keys = Vec::new();
        let mut post_off = vec![0u32];
        let mut postings = Vec::with_capacity(pairs.len());
        for (g, r) in pairs {
            if keys.last() != Some(&g) {
                if !keys.is_empty() {
                    post_off.push(postings.len() as u32);
                }
                keys.push(g);
            }
            postings.push(r);
        }
        post_off.push(postings.len() as u32);
        if keys.is_empty() {
            post_off.truncate(1);
        }

        Self {
            params,
            row_off,
            row_grams,
            keys,
            post_off,
            postings,
            exact,
        }
    }

    pub fn n_rows(&self) -> usize {
        self.row_off.len() - 1
    }

    pub fn params(&self) -> LeakParams {
        self.params
    }

    fn row(&self, r: u32) -> &[u64] {
        let r = r as usize;
        &self.row_grams[self.row_off[r] as usize..self.row_off[r + 1] as usize]
    }

    fn posting(&self, gram: u64) -> Option<&[u32]> {
        let i = self.keys.binary_search(&gram).ok()?;
        Some(&self.postings[self.post_off[i] as usize..self.post_off[i + 1] as usize])
    }

    /// EXACT only — the cheap check, over normalised text.
    pub fn exact_row(&self, text: &str, scratch: &mut QueryScratch) -> Option<u32> {
        normalize_into(text, &mut scratch.norm);
        self.exact.get(scratch.norm.as_str()).copied()
    }

    /// EXACT, else the best of the `top_c` candidates by Jaccard.
    pub fn classify(&self, text: &str, scratch: &mut QueryScratch) -> LeakClass {
        if let Some(row) = self.exact_row(text, scratch) {
            return LeakClass::Exact { row };
        }
        self.classify_near(scratch)
    }

    /// NEAR scoring over `scratch.norm` (already normalised by `exact_row`).
    fn classify_near(&self, scratch: &mut QueryScratch) -> LeakClass {
        let QueryScratch {
            norm,
            grams,
            counts,
            touched,
            top,
        } = scratch;
        shingles_into(norm, self.params.k, grams);
        for &g in grams.iter() {
            let Some(hits) = self.posting(g) else {
                continue;
            };
            if hits.len() >= self.params.rare_cap {
                continue;
            }
            for &r in hits {
                if counts[r as usize] == 0 {
                    touched.push(r);
                }
                counts[r as usize] += 1;
            }
        }

        // Top-c by (count desc, row asc): insertion into a small sorted run.
        top.clear();
        let better = |a: (u32, u32), b: (u32, u32)| a.0 > b.0 || (a.0 == b.0 && a.1 < b.1);
        for &r in touched.iter() {
            let cand = (counts[r as usize], r);
            counts[r as usize] = 0;
            if top.len() == self.params.top_c && !better(cand, top[top.len() - 1]) {
                continue;
            }
            let pos = top
                .iter()
                .position(|&t| better(cand, t))
                .unwrap_or(top.len());
            if top.len() == self.params.top_c {
                top.pop();
            }
            top.insert(pos, cand);
        }
        touched.clear();

        let (mut best, mut best_row) = (0.0f64, None);
        for &(_, r) in top.iter() {
            let row = self.row(r);
            let inter = intersection_len(grams, row);
            let union = grams.len() + row.len() - inter;
            let j = match union {
                0 => 0.0,
                u => inter as f64 / u as f64,
            };
            if j > best {
                best = j;
                best_row = Some(r);
            }
        }
        match best_row {
            Some(row) if best >= self.params.near_j => LeakClass::Near { row, jaccard: best },
            _ => LeakClass::Clean { best },
        }
    }
}

/// Per-suite tallies, the probe's output line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LeakCounts {
    pub reference: usize,
    pub query: usize,
    pub exact: usize,
    /// EXACT rows whose first reference twin carries a different label.
    pub exact_label_conflict: usize,
    pub near: usize,
    /// Non-EXACT query rows inside the NEAR cap.
    pub near_scanned: usize,
    pub near_same_label: usize,
}

/// Tally `query` against `reference`, `(text, label)` on both sides. Returns
/// `None` when either side is empty: nothing was measured, which must never
/// read as a zero leak rate.
pub fn leak_counts<S: AsRef<str>, L: PartialEq>(
    reference: &[(S, L)],
    query: &[(S, L)],
    params: LeakParams,
) -> Option<LeakCounts> {
    if reference.is_empty() || query.is_empty() {
        return None;
    }
    let texts: Vec<&str> = reference.iter().map(|(t, _)| t.as_ref()).collect();
    let index = ShingleIndex::build(&texts, params);
    let mut scratch = QueryScratch::for_index(&index);
    let cap = params.near_cap.unwrap_or(usize::MAX);
    let mut c = LeakCounts {
        reference: reference.len(),
        query: query.len(),
        ..LeakCounts::default()
    };
    for (i, (text, label)) in query.iter().enumerate() {
        if let Some(row) = index.exact_row(text.as_ref(), &mut scratch) {
            c.exact += 1;
            c.exact_label_conflict += usize::from(reference[row as usize].1 != *label);
            continue;
        }
        if i >= cap {
            continue;
        }
        c.near_scanned += 1;
        if let LeakClass::Near { row, .. } = index.classify_near(&mut scratch) {
            c.near += 1;
            c.near_same_label += usize::from(reference[row as usize].1 == *label);
        }
    }
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> LeakParams {
        LeakParams::default()
    }

    #[test]
    fn normalize_matches_the_probe_regexes() {
        assert_eq!(normalize("  Hello,\tWORLD!  "), "hello world");
        // Collapse happens before the strip, so the removed é leaves two spaces.
        assert_eq!(normalize("a é b"), "a  b");
        // Python counts \x1c..\x1f as whitespace; Rust's is_whitespace does not.
        assert_eq!(normalize("a\x1fb"), "a b");
        // Unicode lowering that lands in ASCII survives (KELVIN SIGN → k).
        assert_eq!(normalize("\u{212A}m 42"), "km 42");
        assert_eq!(normalize("!!!"), "");
    }

    #[test]
    fn short_and_empty_text_keep_one_shingle() {
        let mut g = Vec::new();
        shingles_into("ab", 4, &mut g);
        assert_eq!(g, vec![pack(b"ab")]);
        shingles_into("", 4, &mut g);
        assert_eq!(g, vec![0]);
        shingles_into("abcab", 4, &mut g);
        assert_eq!(g.len(), 2);
        // A short gram cannot collide with a full one: no zero byte, so every
        // full 4-gram packs at or above 2^24 and every shorter one below it.
        assert!(pack(b"zzz") < 1 << 24 && pack(b"0000") >= 1 << 24);
    }

    #[test]
    fn exact_path_normalises_and_reports_first_twin() {
        let idx = ShingleIndex::build(&["Turn the lights OFF.", "turn the lights off"], params());
        let mut s = QueryScratch::for_index(&idx);
        assert_eq!(
            idx.classify("turn  the lights off!", &mut s),
            LeakClass::Exact { row: 0 }
        );
    }

    #[test]
    fn planted_paraphrase_is_flagged_near() {
        let reference = [
            "What do you base your exchange rates on?",
            "How do I reset my card PIN at an ATM?",
            "My transfer to a friend has not arrived yet.",
        ];
        let idx = ShingleIndex::build(&reference, params());
        let mut s = QueryScratch::for_index(&idx);
        match idx.classify("what do you base your exchange rates on", &mut s) {
            LeakClass::Exact { row: 0 } => {}
            other => panic!("expected the exact twin, got {other:?}"),
        }
        match idx.classify("What do you base your exchange rate on?", &mut s) {
            LeakClass::Near { row: 0, jaccard } => assert!(jaccard >= DEFAULT_NEAR_J),
            other => panic!("expected a near twin of row 0, got {other:?}"),
        }
    }

    #[test]
    fn disjoint_topic_is_clean() {
        let idx = ShingleIndex::build(
            &[
                "What do you base your exchange rates on?",
                "Why was my card declined?",
            ],
            params(),
        );
        let mut s = QueryScratch::for_index(&idx);
        match idx.classify("Wall St. bears claw back into the black", &mut s) {
            LeakClass::Clean { best } => assert!(best < 0.2, "best {best}"),
            other => panic!("expected clean, got {other:?}"),
        }
    }

    #[test]
    fn rare_cap_skips_common_shingles() {
        // Every reference row shares "abcd"; only row 3 also shares "wxyz".
        let reference = ["abcd q1", "abcd q2", "abcd q3", "abcd wxyz"];
        let capped = LeakParams {
            rare_cap: 3,
            near_j: 0.0,
            ..params()
        };
        let idx = ShingleIndex::build(&reference, capped);
        let mut s = QueryScratch::for_index(&idx);
        // "abcd" is held by 4 >= 3 rows, so only "wxyz" votes: row 3 alone.
        match idx.classify("abcd wxyz!", &mut s) {
            LeakClass::Exact { row: 3 } => {}
            other => panic!("{other:?}"),
        }
        match idx.classify("abcd wxyz 9", &mut s) {
            LeakClass::Near { row: 3, .. } => {}
            other => panic!("{other:?}"),
        }
        // A query sharing ONLY the capped shingle has no candidate at all.
        assert_eq!(idx.classify("abcd", &mut s), LeakClass::Clean { best: 0.0 });
    }

    #[test]
    fn top_c_ties_break_on_lowest_row() {
        // Two identical reference rows: equal counts, equal Jaccard; the
        // strict `>` keeps the first scored, and the tie-break scores row 0 first.
        let idx = ShingleIndex::build(
            &[
                "the quick brown fox jumps over",
                "the quick brown fox jumps over",
            ],
            params(),
        );
        let mut s = QueryScratch::for_index(&idx);
        match idx.classify("the quick brown fox jumps over it", &mut s) {
            LeakClass::Near { row: 0, .. } => {}
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn scratch_is_reset_between_queries() {
        let idx = ShingleIndex::build(
            &["alpha beta gamma delta", "epsilon zeta eta theta"],
            params(),
        );
        let mut s = QueryScratch::for_index(&idx);
        let first = idx.classify("alpha beta gamma delta x", &mut s);
        let _ = idx.classify("epsilon zeta eta theta y", &mut s);
        assert_eq!(idx.classify("alpha beta gamma delta x", &mut s), first);
        assert!(s.touched.is_empty() && s.counts.iter().all(|&c| c == 0));
    }

    #[test]
    fn leak_counts_tallies_like_the_probe() {
        let reference = vec![
            ("What do you base your exchange rates on?", "rates"),
            ("Why was my card declined?", "declined"),
        ];
        let query = vec![
            ("why was my card declined", "declined"), // exact, same label
            ("What do you base your exchange rates on", "fee"), // exact, conflict
            ("What do you base your exchange rate on?", "rates"), // near, same label (J 0.816)
            ("Wall St. bears claw back into the black", "biz"), // clean
        ];
        let c = leak_counts(&reference, &query, params()).unwrap();
        assert_eq!(
            c,
            LeakCounts {
                reference: 2,
                query: 4,
                exact: 2,
                exact_label_conflict: 1,
                near: 1,
                near_scanned: 2,
                near_same_label: 1,
            }
        );
        // The NEAR cap counts query POSITIONS, exact rows included (the probe's
        // `test[:TEST_CAP]`), so a cap of 2 scans no non-exact row at all.
        let capped = LeakParams {
            near_cap: Some(2),
            ..params()
        };
        let c = leak_counts(&reference, &query, capped).unwrap();
        assert_eq!((c.exact, c.near_scanned, c.near), (2, 0, 0));
        let empty: Vec<(&str, &str)> = Vec::new();
        assert_eq!(leak_counts(&empty, &query, params()), None);
    }
}
