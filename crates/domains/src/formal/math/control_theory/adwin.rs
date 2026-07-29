#[allow(unused_imports)]
use alloc::{
    boxed::Box, collections::VecDeque, format, string::String, string::ToString, vec, vec::Vec,
};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// ADWIN (ADaptive WINdowing) drift detector.
///
/// Bifet, A. & Gavaldà, R. (2007). "Learning from Time-Changing Data
/// with Adaptive Windowing." *SDM 2007* (SIAM International Conference
/// on Data Mining).
///
/// Maintains an online window of recent observations and, on each new
/// sample, checks every cut of the window into a "head" (older) and
/// "tail" (newer) half. If the mean difference exceeds the Hoeffding
/// bound (`eps_cut`), the older half is dropped and a change is
/// signalled — the window adapts its own size to how fast the
/// underlying distribution is changing, hence "adaptive windowing."
/// This is the concrete detector for `ControlTheoryConcept::Disturbance`
/// (formal::systems::control): where that concept names "an external
/// perturbation acting on the plant," `Adwin` is how a controller
/// actually NOTICES one has occurred in a data stream, rather than
/// assuming the plant's dynamics are static.
///
/// The O(n) per-update scan (the paper's own exponential-histogram
/// optimization trades this for O(log n) at the cost of approximate
/// means) is exact and adequate for the bounded window sizes this
/// ontology targets.
///
/// Every public constructor/method parameter and return is a
/// [`Quantity`], never a bare `f64`/`usize`/`bool` — the window's raw
/// samples and its capacity cap are private implementation state (the
/// algorithm's own internal bookkeeping, never queried directly by a
/// caller), reached only through the typed accessors below.
#[derive(Debug, Clone, PartialEq)]
pub struct Adwin {
    /// Confidence parameter δ ∈ (0, 1) (dimensionless). Lower = more
    /// sensitive (more false positives). Bifet & Gavaldà recommend
    /// δ = 0.002.
    delta: Quantity,
    /// Hard cap on the window size — bounds memory for a stream that
    /// never drifts, at the cost of forgetting genuinely-still-relevant
    /// old samples once the cap is hit.
    max_window: usize,
    window: VecDeque<f64>,
    /// Total drift events detected over the lifetime of this detector.
    drift_count: u64,
}

/// The outcome of one [`Adwin::update`] call — a typed verdict, not a
/// bare `bool`, so "was drift detected" is a queryable/explainable fact
/// rather than an opaque flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftVerdict {
    /// The two halves of the window are statistically indistinguishable
    /// at the detector's confidence level — no change.
    Stationary,
    /// The Hoeffding bound was exceeded at some cut: the older half was
    /// dropped, and the underlying distribution is judged to have
    /// changed.
    Detected,
}

impl DriftVerdict {
    pub fn is_detected(self) -> bool {
        matches!(self, DriftVerdict::Detected)
    }
}

/// The outcome of one [`Adwin::update`] call.
#[derive(Debug, Clone, PartialEq)]
pub struct DriftUpdate {
    pub verdict: DriftVerdict,
    /// Mean of the discarded older half — only meaningful when
    /// `verdict` is [`DriftVerdict::Detected`] ("before drift"
    /// reporting).
    pub old_mean: Quantity,
    /// Mean of the retained newer half (or the whole window, when no
    /// drift was detected).
    pub new_mean: Quantity,
}

impl Adwin {
    pub fn new(delta: Quantity, max_window: Quantity) -> Self {
        Self {
            delta,
            max_window: max_window.value.max(0.0) as usize,
            window: VecDeque::new(),
            drift_count: 0,
        }
    }

    /// Restore a detector from previously-observed state — the dual of
    /// [`Adwin::new`] for a detector with history, so a caller that
    /// persists its own state across restarts (e.g. a long-running
    /// process's checkpoint) can resume a detector exactly where it left
    /// off, rather than losing the window and re-accumulating drift
    /// history from empty. `window` is oldest-first, matching
    /// [`Adwin::window_samples`]'s own order. This crate does not itself
    /// persist `Adwin` state (that is outside an ontology's scope); this
    /// constructor is the typed capability an external caller's own
    /// persistence format restores through.
    pub fn from_state(
        delta: Quantity,
        max_window: Quantity,
        window: Vec<Quantity>,
        drift_count: Quantity,
    ) -> Self {
        Self {
            delta,
            max_window: max_window.value.max(0.0) as usize,
            window: window.into_iter().map(|q| q.value).collect(),
            drift_count: drift_count.value.max(0.0) as u64,
        }
    }

    /// The detector's confidence parameter δ.
    pub fn delta(&self) -> Quantity {
        self.delta.clone()
    }

    /// The hard cap on the window size.
    pub fn max_window(&self) -> Quantity {
        Quantity::dimensionless(self.max_window as f64)
    }

    /// The number of samples currently retained in the window.
    pub fn window_len(&self) -> Quantity {
        Quantity::dimensionless(self.window.len() as f64)
    }

    /// The window's raw retained samples, oldest first — the typed
    /// accessor a caller needs to actually persist a detector's state
    /// (paired with [`Adwin::from_state`] to restore it), completing the
    /// read side of the state this struct already exposes a length and
    /// mean for.
    pub fn window_samples(&self) -> Vec<Quantity> {
        self.window
            .iter()
            .map(|&v| Quantity::from_unit(v, &unit::UNITLESS))
            .collect()
    }

    /// The total number of drift events detected over this detector's
    /// lifetime.
    pub fn drift_count(&self) -> Quantity {
        Quantity::dimensionless(self.drift_count as f64)
    }

    /// The Hoeffding bound for a cut splitting `n0` older samples from
    /// `n1` newer ones — the threshold for declaring the two halves
    /// come from different distributions at confidence `1 - delta`
    /// (Bifet & Gavaldà 2007, Theorem 1).
    fn eps_cut(&self, n0: usize, n1: usize) -> f64 {
        // Harmonic mean of the two cut sizes.
        let m = 1.0 / (1.0 / n0 as f64 + 1.0 / n1 as f64);
        let n = (n0 + n1) as f64;
        ((1.0 / (2.0 * m)) * (4.0 * n / self.delta.value).ln()).sqrt()
    }

    /// The Hoeffding bound for a cut splitting `n0` older samples from
    /// `n1` newer ones, as a typed [`Quantity`] — the public mirror of
    /// `Adwin::eps_cut` for callers reasoning about the bound itself
    /// (e.g. that a balanced cut is tightest).
    pub fn eps_cut_bound(&self, n0: Quantity, n1: Quantity) -> Quantity {
        Quantity::dimensionless(self.eps_cut(n0.value as usize, n1.value as usize))
    }

    /// Add a new sample. Scans every cut of the current window for the
    /// largest mean difference; if it exceeds `eps_cut`, drops the
    /// older half and reports [`DriftVerdict::Detected`].
    pub fn update(&mut self, x: Quantity) -> DriftUpdate {
        let x = x.value;
        self.window.push_back(x);
        if self.window.len() > self.max_window {
            self.window.pop_front();
        }
        let n = self.window.len();
        if n < 4 {
            // Need at least 2+2 samples to evaluate any cut.
            let mean = if n == 0 {
                0.0
            } else {
                self.window.iter().sum::<f64>() / n as f64
            };
            return DriftUpdate {
                verdict: DriftVerdict::Stationary,
                old_mean: Quantity::from_unit(mean, &unit::UNITLESS),
                new_mean: Quantity::from_unit(mean, &unit::UNITLESS),
            };
        }
        // Walk every possible cut [0..i] | [i..n] for i in 1..n, using
        // O(n) running prefix sums rather than an O(n^2) inner loop.
        let total: f64 = self.window.iter().sum();
        let mut prefix = 0.0;
        let mut best_change: Option<(usize, f64, f64)> = None;
        for i in 1..n {
            prefix += self.window[i - 1];
            let head_mean = prefix / i as f64;
            let tail_mean = (total - prefix) / (n - i) as f64;
            let eps = self.eps_cut(i, n - i);
            let diff = (head_mean - tail_mean).abs();
            if diff > eps {
                // Track the best (largest-margin) cut so the drop is maximal.
                let margin = diff - eps;
                if best_change.map(|(_, m, _)| margin > m).unwrap_or(true) {
                    best_change = Some((i, margin, head_mean));
                }
            }
        }
        if let Some((cut, _, head_mean)) = best_change {
            let _: VecDeque<f64> = self.window.drain(..cut).collect();
            self.drift_count += 1;
            let new_mean = if self.window.is_empty() {
                head_mean
            } else {
                self.window.iter().sum::<f64>() / self.window.len() as f64
            };
            DriftUpdate {
                verdict: DriftVerdict::Detected,
                old_mean: Quantity::from_unit(head_mean, &unit::UNITLESS),
                new_mean: Quantity::from_unit(new_mean, &unit::UNITLESS),
            }
        } else {
            let mean = total / n as f64;
            DriftUpdate {
                verdict: DriftVerdict::Stationary,
                old_mean: Quantity::from_unit(mean, &unit::UNITLESS),
                new_mean: Quantity::from_unit(mean, &unit::UNITLESS),
            }
        }
    }

    pub fn current_mean(&self) -> Quantity {
        let mean = if self.window.is_empty() {
            0.0
        } else {
            self.window.iter().sum::<f64>() / self.window.len() as f64
        };
        Quantity::from_unit(mean, &unit::UNITLESS)
    }
}

impl Default for Adwin {
    fn default() -> Self {
        // Bifet & Gavaldà recommend delta = 0.002 for ~99.8% confidence
        // cuts; 1024 is a generous window for the bounded streams this
        // ontology targets.
        Self::new(
            Quantity::dimensionless(0.002),
            Quantity::dimensionless(1024.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn delta(value: f64) -> Quantity {
        Quantity::dimensionless(value)
    }
    fn window_cap(value: usize) -> Quantity {
        Quantity::dimensionless(value as f64)
    }
    fn sample(value: f64) -> Quantity {
        Quantity::dimensionless(value)
    }

    /// A constant stream never drifts, regardless of length — there is
    /// no distribution change to detect.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_drift_on_constant_stream() {
        let mut a = Adwin::new(delta(0.002), window_cap(1024));
        for _ in 0..200 {
            let r = a.update(sample(1.0));
            assert!(!r.verdict.is_detected(), "constant stream should not drift");
        }
        assert_eq!(a.drift_count().value, 0.0);
        assert!((a.current_mean().value - 1.0).abs() < 1e-9);
    }

    /// An abrupt mean shift is detected within a few samples after it
    /// occurs — the whole point of the detector.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn detects_abrupt_shift() {
        let mut a = Adwin::new(delta(0.002), window_cap(1024));
        for _ in 0..100 {
            a.update(sample(0.0));
        }
        let mut detected = false;
        for _ in 0..50 {
            let r = a.update(sample(5.0));
            if r.verdict.is_detected() {
                detected = true;
                break;
            }
        }
        assert!(detected, "expected drift detection after a step change");
        assert_eq!(a.drift_count().value, 1.0);
        assert!(
            a.current_mean().value > 1.0,
            "after drift, mean should reflect the new regime, got {}",
            a.current_mean().value
        );
    }

    /// The window never exceeds `max_window`, regardless of stream
    /// length — the hard memory bound holds.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn window_is_capped() {
        let mut a = Adwin::new(delta(0.002), window_cap(16));
        for i in 0..100 {
            a.update(sample(i as f64));
        }
        assert!(a.window_len().value <= 16.0);
    }

    /// A detector restored via `from_state` from another detector's own
    /// `window_samples`/`drift_count`/etc. behaves identically to the
    /// original going forward — the round trip a persistence-across-
    /// restarts caller depends on preserves everything `update` reads.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn from_state_round_trips_and_continues_identically() {
        let mut original = Adwin::new(delta(0.002), window_cap(1024));
        for i in 0..50 {
            original.update(sample((i % 7) as f64));
        }

        let mut restored = Adwin::from_state(
            original.delta(),
            original.max_window(),
            original.window_samples(),
            original.drift_count(),
        );

        assert_eq!(restored.window_len().value, original.window_len().value);
        assert_eq!(restored.drift_count().value, original.drift_count().value);
        assert!((restored.current_mean().value - original.current_mean().value).abs() < 1e-9);

        // Feed the same continuation into both — behavior stays identical.
        for i in 0..50 {
            let x = sample(((i * 3) % 11) as f64);
            let r1 = original.update(x.clone());
            let r2 = restored.update(x);
            assert_eq!(r1.verdict.is_detected(), r2.verdict.is_detected());
            assert!((r1.new_mean.value - r2.new_mean.value).abs() < 1e-9);
        }
        assert_eq!(restored.drift_count().value, original.drift_count().value);
    }

    /// Fewer than 4 samples never fire detection — there is no
    /// evaluable cut yet (each half needs at least 2 samples).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_detection_below_four_samples() {
        let mut a = Adwin::new(delta(0.002), window_cap(1024));
        for _ in 0..3 {
            assert!(!a.update(sample(100.0)).verdict.is_detected());
        }
    }

    /// The Hoeffding bound is tightest (smallest) for a balanced cut —
    /// a fixed total split evenly gives the strongest statistical
    /// evidence per Bifet & Gavaldà's harmonic-mean construction.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eps_cut_balanced_is_smallest() {
        let a = Adwin::new(delta(0.002), window_cap(1024));
        let unbalanced =
            a.eps_cut_bound(Quantity::dimensionless(1.0), Quantity::dimensionless(99.0));
        let balanced =
            a.eps_cut_bound(Quantity::dimensionless(50.0), Quantity::dimensionless(50.0));
        assert!(
            balanced.value < unbalanced.value,
            "balanced cut should yield a tighter bound: balanced={}, unbalanced={}",
            balanced.value,
            unbalanced.value
        );
    }

    proptest! {
        /// Axiom: the window length after any sequence of updates never
        /// exceeds `max_window`, for ANY cap and ANY stream — not just
        /// the hand-picked case above.
        #[test]
        fn prop_window_never_exceeds_cap(
            max_window in 4usize..64,
            samples in proptest::collection::vec(-100.0f64..100.0, 0..200),
        ) {
            let mut a = Adwin::new(delta(0.002), window_cap(max_window));
            for x in samples {
                a.update(sample(x));
                prop_assert!(a.window_len().value <= max_window as f64);
            }
        }

        /// Axiom: a perfectly constant stream, for ANY constant value
        /// and ANY delta, never drifts — the Hoeffding bound is never
        /// exceeded when head and tail means are identical.
        #[test]
        fn prop_constant_stream_never_drifts(
            value in -1000.0f64..1000.0,
            delta_value in 0.0001f64..0.5,
            n in 4usize..200,
        ) {
            let mut a = Adwin::new(delta(delta_value), window_cap(1024));
            for _ in 0..n {
                let r = a.update(sample(value));
                prop_assert!(!r.verdict.is_detected());
            }
        }
    }

    pr4xis::register_praxis_value!(prop_window_never_exceeds_cap, Verifiable);
    pr4xis::register_praxis_value!(prop_constant_stream_never_drifts, Verifiable);
}
