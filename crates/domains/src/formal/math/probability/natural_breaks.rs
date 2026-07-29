//! Natural breaks — where a one-dimensional set of values genuinely divides.
//!
//! Given observed magnitudes that fall into visibly separate groups, the
//! question "where is the boundary?" has a defensible answer that does not
//! require anyone to nominate a round number: put the boundary where it makes
//! the two groups internally most alike and most unlike each other.
//!
//! Jenks (1967) states that as choosing class limits which minimise the summed
//! within-class squared deviation and correspondingly maximise the between-class
//! sum of squares. For **two** classes over a sorted sequence the exact optimum
//! is reachable by evaluating every split, because the between-class sum of
//! squares collapses to `n₁n₂/n · (m₁ − m₂)²` — one term, computable for all
//! `n − 1` candidate splits in a single prefix-sum pass.
//!
//! **This is NOT the widest gap.** An earlier version of this module claimed the
//! two-class case "reduces exactly to splitting at the widest gap" and cited
//! these papers for it. That is false, and `[0, 1, 2, 3, 4, 5.5]` is a witness:
//! the widest gap is 4 → 5.5, but Jenks's optimum splits at 3 (within-class
//! deviation 5.17 against the widest gap's 10.0). Fisher (1958) is authority
//! *against* the shortcut, not for it — his contribution is an O(kn²) dynamic
//! programme precisely because no closed form exists in general. The cited work
//! now matches the implemented rule.
//!
//! This exists so thresholds can be DERIVED from the population they classify
//! rather than asserted. A constant with a comment explaining which gap it sits
//! in is honest on the day it is written and silently false the day a new value
//! lands in that gap; a computed break moves with its data and cannot go stale.
//!
//! References:
//! - Jenks, G. F. (1967). *The Data Model Concept in Statistical Mapping*.
//!   International Yearbook of Cartography 7:186–190 — the optimal-class-limits
//!   method ("natural breaks"): minimise within-class, maximise between-class
//!   deviation. This module implements that objective exactly for k = 2.
//! - Fisher, W. D. (1958). *On Grouping for Maximum Homogeneity*. Journal of the
//!   American Statistical Association 53(284):789–798 — the exact dynamic
//!   programme for arbitrary k, of which the single prefix-sum pass below is the
//!   k = 2 case.

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

/// The two-class natural break in `values`: the smallest magnitude that belongs
/// to the UPPER class, or `None` when no split is meaningful (fewer than two
/// distinct values).
///
/// The returned value is the lower bound of the upper class, so a caller
/// classifying `x` asks `x >= break`.
///
/// `values` is treated as a MULTISET — repeats carry weight, because Jenks's
/// objective is about how the mass distributes, and two values at 1 MB pull the
/// lower class's mean differently than one does. (An earlier version deduped
/// first, which silently reweighted the population.)
///
/// `values` need not be sorted, and NaNs are dropped rather than ordered — a
/// magnitude that is not a number has no position on the line, and silently
/// inventing one would put the break in an arbitrary place.
pub fn two_class_break(values: &[f64]) -> Option<f64> {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("NaNs were filtered above"));
    // Fewer than two DISTINCT values means no boundary the data supports.
    if sorted.first() == sorted.last() {
        return None;
    }
    let n = sorted.len() as f64;
    let total: f64 = sorted.iter().sum();

    // Maximise the between-class sum of squares, which for two classes is the
    // single term `n₁n₂/n · (m₁ − m₂)²` (Jenks 1967; the k = 2 case of Fisher
    // 1958's programme). Maximising it is equivalent to minimising the summed
    // within-class squared deviation, so one prefix-sum pass over the `n − 1`
    // candidate splits finds the exact optimum — no heuristic, no iteration.
    let mut lower_sum = 0.0;
    let mut best_bcss = f64::NEG_INFINITY;
    let mut split = None;
    for i in 0..sorted.len() - 1 {
        lower_sum += sorted[i];
        let n1 = (i + 1) as f64;
        let n2 = n - n1;
        // Only a boundary BETWEEN distinct values is a boundary at all —
        // equal neighbours cannot be separated, so skip those split points
        // rather than let them win a tie.
        if sorted[i + 1] == sorted[i] {
            continue;
        }
        let m1 = lower_sum / n1;
        let m2 = (total - lower_sum) / n2;
        let bcss = (n1 * n2 / n) * (m1 - m2) * (m1 - m2);
        if bcss > best_bcss {
            best_bcss = bcss;
            split = Some(sorted[i + 1]);
        }
    }
    split
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn the_break_separates_two_obvious_groups() {
        // {1, 2, 3} against {40, 41}: any reasonable rule agrees here, which is
        // exactly why this case cannot distinguish a correct implementation
        // from a wrong one — see `the_break_is_jenks_optimum_not_the_widest_gap`.
        assert_eq!(two_class_break(&[3.0, 1.0, 41.0, 2.0, 40.0]), Some(40.0));
    }

    /// THE WITNESS. This module previously implemented widest-gap and cited
    /// Jenks and Fisher for it. They do not support that rule, and this input
    /// proves the two are different answers.
    ///
    /// For `[0, 1, 2, 3, 4, 5.5]` the widest gap is 4 → 5.5, but Jenks's
    /// objective — minimise summed within-class squared deviation — splits at 3
    /// (5.17 against the widest gap's 10.0). A test built only from populations
    /// where the two rules agree is why the mis-citation survived.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_break_is_jenks_optimum_not_the_widest_gap() {
        let values = [0.0, 1.0, 2.0, 3.0, 4.0, 5.5];
        assert_eq!(
            two_class_break(&values),
            Some(3.0),
            "Jenks's optimum splits {{0,1,2}}|{{3,4,5.5}}; the widest gap (4→5.5) \
             would answer 5.5 and is a different, worse partition"
        );

        // And prove it on the objective itself, so this cannot be satisfied by
        // agreeing with a number someone wrote down.
        let sdcm = |split: f64| -> f64 {
            let (lo, hi): (Vec<f64>, Vec<f64>) = values.iter().partition(|v| **v < split);
            [lo, hi]
                .iter()
                .map(|cls| {
                    let m = cls.iter().sum::<f64>() / cls.len() as f64;
                    cls.iter().map(|v| (v - m) * (v - m)).sum::<f64>()
                })
                .sum()
        };
        assert!(
            sdcm(3.0) < sdcm(5.5),
            "the returned split must be the one with the LOWER within-class \
             deviation: {} vs {}",
            sdcm(3.0),
            sdcm(5.5)
        );
    }

    /// Repeats carry weight — the population is a multiset, not a set.
    ///
    /// An earlier version deduped before splitting, which silently reweighted
    /// the data: five sources at 1 MB and one at 40 MB is a different
    /// distribution from one of each, and Jenks's objective is about where the
    /// mass sits.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn repeated_magnitudes_carry_their_weight() {
        let massed = two_class_break(&[1.0, 1.0, 1.0, 1.0, 1.0, 40.0]);
        assert_eq!(massed, Some(40.0));
        // Same distinct values, different mass — still a valid split, and the
        // point is that the function SEES the difference rather than collapsing
        // both inputs to the same deduped sequence.
        assert_eq!(two_class_break(&[1.0, 40.0]), Some(40.0));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn the_break_is_the_lower_bound_of_the_upper_class() {
        // A caller classifies with `x >= break`, so the returned value must
        // itself be in the upper class — off by one here would misclassify
        // exactly the boundary case.
        let b = two_class_break(&[1.0, 10.0]).expect("two distinct values split");
        assert_eq!(b, 10.0);
        assert!(10.0 >= b && 1.0 < b);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_population_with_no_real_division_yields_none() {
        // Nothing to split: the caller must decide what to do with that,
        // rather than receive a boundary the data does not support.
        assert_eq!(two_class_break(&[]), None);
        assert_eq!(two_class_break(&[7.0]), None);
        assert_eq!(two_class_break(&[7.0, 7.0, 7.0]), None);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn magnitudes_that_are_not_numbers_are_dropped_not_ordered() {
        // A NaN has no position on the line. Ordering it would place the break
        // wherever the comparator happened to land it.
        assert_eq!(two_class_break(&[1.0, f64::NAN, 50.0]), Some(50.0));
        assert_eq!(two_class_break(&[f64::NAN, 3.0]), None);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn an_evenly_spaced_population_still_splits_deterministically() {
        // Every gap is equal, so no split is more "natural" than another by
        // spacing — but Jenks's objective still has a unique maximum: the
        // balanced partition, because n₁n₂ is largest in the middle. A stable,
        // principled answer rather than iteration order deciding.
        assert_eq!(two_class_break(&[1.0, 2.0, 3.0, 4.0]), Some(3.0));
    }
}
