//! Per-lane latency EXTREMES (Issue 020 T8): where the slowest sample sits.
//!
//! The harness times one sample per CASE, so a 12-case suite's "p99" lands on
//! `n - 1` — it IS the maximum (tail support 1), and a maximum on its own
//! cannot say whether it is a residual cold cost on the first case or one
//! long input. Bench 006 Addendum 3 measured `code_fixtures` at rust max
//! ~225 ms vs python ~160 ms, +43% every round, and could not attribute it
//! because nothing kept the sample order. These three numbers are the
//! cheapest record that can: `argmax_case == 0` names the cold-start reading,
//! any other index names the case to read.
//!
//! Pure over the sample slice (no clocks, no allocation) so it is armable
//! without a model.

use serde::Serialize;

/// First sample, maximum sample, and the case index of the maximum — in
/// milliseconds, whatever unit the lane timed in.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LatencyExtremes {
    pub first_ms: f64,
    pub max_ms: f64,
    /// The EARLIEST index holding the maximum, so a tie with case 0 reads as
    /// the cold-start case rather than a later one.
    pub argmax_case: usize,
}

impl LatencyExtremes {
    /// `None` on an empty lane — never a zero that reads as a fast one.
    /// `per_ms` is how many sample units make a millisecond (1000 for µs,
    /// 1 for ms).
    pub fn of(durs: &[u64], per_ms: f64) -> Option<Self> {
        let (&first, rest) = durs.split_first()?;
        let (argmax_case, max) = rest
            .iter()
            .enumerate()
            .fold((0, first), |(bi, bv), (i, &v)| match v > bv {
                true => (i + 1, v),
                false => (bi, bv),
            });
        Some(Self {
            first_ms: first as f64 / per_ms,
            max_ms: max as f64 / per_ms,
            argmax_case,
        })
    }
}

/// The `p99 (support)` table cell. When the tail support is 1 the p99 IS the
/// maximum, so the cell names the case that produced it — `max@case0` is the
/// cold-start reading. Wider tails print unchanged.
pub fn fmt_p99_cell(
    p99_ms: f64,
    support: usize,
    extremes: Option<&LatencyExtremes>,
    prec: usize,
) -> String {
    match (support, extremes) {
        (1, Some(e)) => format!("{p99_ms:.prec$} ms (1, max@case{})", e.argmax_case),
        _ => format!("{p99_ms:.prec$} ms ({support})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_lane_is_none_not_zero() {
        assert_eq!(LatencyExtremes::of(&[], 1.0), None);
    }

    #[test]
    fn cold_first_case_reads_as_case0() {
        let e = LatencyExtremes::of(&[231, 78, 80, 77], 1.0).unwrap();
        assert_eq!(e.argmax_case, 0);
        assert_eq!(e.first_ms, 231.0);
        assert_eq!(e.max_ms, 231.0);
    }

    #[test]
    fn a_later_max_names_its_case_and_units_convert() {
        let e = LatencyExtremes::of(&[80_000, 78_000, 225_000, 77_000], 1000.0).unwrap();
        assert_eq!(e.argmax_case, 2);
        assert_eq!(e.first_ms, 80.0);
        assert_eq!(e.max_ms, 225.0);
    }

    #[test]
    fn a_tie_with_case0_keeps_case0() {
        // Strict `>`: a later equal sample must not steal the cold-start label.
        let e = LatencyExtremes::of(&[90, 50, 90], 1.0).unwrap();
        assert_eq!(e.argmax_case, 0);
    }

    #[test]
    fn p99_cell_names_the_case_only_when_p99_is_the_max() {
        let e = LatencyExtremes::of(&[231, 78], 1.0).unwrap();
        assert_eq!(
            fmt_p99_cell(231.0, 1, Some(&e), 1),
            "231.0 ms (1, max@case0)"
        );
        assert_eq!(fmt_p99_cell(231.0, 6, Some(&e), 1), "231.0 ms (6)");
        assert_eq!(fmt_p99_cell(0.123, 1, None, 3), "0.123 ms (1)");
    }
}
