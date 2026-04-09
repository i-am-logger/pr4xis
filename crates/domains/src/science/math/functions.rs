/// Real-valued function primitives.
///
/// Sources:
/// - Piecewise functions: standard real analysis
/// - Linear combination: linear algebra fundamentals
/// - Interval clamping: order theory on ℝ
///
/// These are building blocks for domain-specific formulas
/// (color science, physics, signal processing).
use praxis::ontology::Axiom;

/// A piecewise function on ℝ: applies different rules based on a threshold.
///
/// f(x) = { below(x)  if x <= threshold
///         { above(x)  if x > threshold
///
/// Continuity at the threshold is an axiom, not a type constraint —
/// domain-specific piecewise functions may or may not be continuous.
#[derive(Debug, Clone)]
pub struct Piecewise {
    pub threshold: f64,
    pub below: fn(f64) -> f64,
    pub above: fn(f64) -> f64,
}

impl Piecewise {
    pub fn eval(&self, x: f64) -> f64 {
        if x <= self.threshold {
            (self.below)(x)
        } else {
            (self.above)(x)
        }
    }

    /// Check continuity at the threshold (left limit ≈ right limit).
    pub fn is_continuous(&self, epsilon: f64) -> bool {
        let left = (self.below)(self.threshold);
        let right = (self.above)(self.threshold);
        (left - right).abs() < epsilon
    }
}

/// A weighted linear combination: Σ wᵢ · xᵢ
///
/// Source: linear algebra — the most fundamental operation
/// in vector spaces over ℝ.
#[derive(Debug, Clone)]
pub struct LinearCombination {
    pub weights: Vec<f64>,
}

impl LinearCombination {
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }

    /// Evaluate: Σ wᵢ · xᵢ
    pub fn eval(&self, values: &[f64]) -> f64 {
        assert_eq!(
            self.weights.len(),
            values.len(),
            "weights and values must have same length"
        );
        self.weights
            .iter()
            .zip(values.iter())
            .map(|(w, x)| w * x)
            .sum()
    }

    /// Weights sum to 1.0 (convex combination).
    pub fn is_convex(&self) -> bool {
        (self.weights.iter().sum::<f64>() - 1.0).abs() < 1e-10
    }

    /// All weights non-negative.
    pub fn is_non_negative(&self) -> bool {
        self.weights.iter().all(|w| *w >= 0.0)
    }
}

/// A closed interval [a, b] on ℝ.
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    pub const UNIT: Interval = Interval { lo: 0.0, hi: 1.0 };

    pub fn new(lo: f64, hi: f64) -> Self {
        assert!(lo <= hi, "interval lo must be <= hi");
        Self { lo, hi }
    }

    pub fn contains(&self, x: f64) -> bool {
        x >= self.lo && x <= self.hi
    }

    /// Clamp a value to this interval.
    pub fn clamp(&self, x: f64) -> f64 {
        x.clamp(self.lo, self.hi)
    }

    pub fn width(&self) -> f64 {
        self.hi - self.lo
    }
}

/// A ratio with offset: (a + k) / (b + k) where a >= b.
///
/// Used for WCAG contrast ratio where k = 0.05 (viewing flare).
/// Source: WCAG 2.1 "contrast ratio" definition.
#[derive(Debug, Clone, Copy)]
pub struct OffsetRatio {
    pub offset: f64,
}

impl OffsetRatio {
    pub fn eval(&self, a: f64, b: f64) -> f64 {
        let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
        (lighter + self.offset) / (darker + self.offset)
    }
}

// ── Axioms ──

/// Piecewise continuity: left and right limits agree at threshold.
pub struct PiecewiseContinuity {
    pub function: Piecewise,
    pub epsilon: f64,
}

impl Axiom for PiecewiseContinuity {
    fn description(&self) -> &str {
        "piecewise function is continuous at threshold"
    }
    fn holds(&self) -> bool {
        self.function.is_continuous(self.epsilon)
    }
}

/// Linear combination weights sum to 1 (convex combination).
pub struct ConvexWeights {
    pub combination: LinearCombination,
}

impl Axiom for ConvexWeights {
    fn description(&self) -> &str {
        "weights form a convex combination (sum to 1)"
    }
    fn holds(&self) -> bool {
        self.combination.is_convex() && self.combination.is_non_negative()
    }
}

/// Offset ratio is always >= 1.0 (by definition, lighter/darker).
pub struct RatioBounded {
    pub ratio: OffsetRatio,
}

impl Axiom for RatioBounded {
    fn description(&self) -> &str {
        "offset ratio is >= 1.0"
    }
    fn holds(&self) -> bool {
        // For any a, b in [0, 1]: ratio(a, b) >= 1.0
        // Test with extremes
        let r = &self.ratio;
        r.eval(0.0, 0.0) >= 1.0
            && r.eval(1.0, 0.0) >= 1.0
            && r.eval(0.0, 1.0) >= 1.0
            && r.eval(1.0, 1.0) >= 1.0
            && r.eval(0.5, 0.3) >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piecewise_linear() {
        // f(x) = { 2x     if x <= 0.5
        //        { x + 0.5 if x > 0.5
        let f = Piecewise {
            threshold: 0.5,
            below: |x| 2.0 * x,
            above: |x| x + 0.5,
        };
        assert!((f.eval(0.25) - 0.5).abs() < 1e-10);
        assert!((f.eval(0.5) - 1.0).abs() < 1e-10);
        assert!((f.eval(0.75) - 1.25).abs() < 1e-10);
        assert!(f.is_continuous(1e-10));
    }

    #[test]
    fn test_piecewise_discontinuous() {
        let f = Piecewise {
            threshold: 0.5,
            below: |_| 0.0,
            above: |_| 1.0,
        };
        assert!(!f.is_continuous(0.1));
    }

    #[test]
    fn test_linear_combination() {
        // BT.709: 0.2126R + 0.7152G + 0.0722B
        let lc = LinearCombination::new(vec![0.2126, 0.7152, 0.0722]);
        assert!(lc.is_convex()); // sums to ~1.0
        assert!(lc.is_non_negative());

        // Pure green → luminance = 0.7152
        let result = lc.eval(&[0.0, 1.0, 0.0]);
        assert!((result - 0.7152).abs() < 1e-10);

        // Pure white → luminance = 1.0
        let result = lc.eval(&[1.0, 1.0, 1.0]);
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_interval() {
        let unit = Interval::UNIT;
        assert!(unit.contains(0.0));
        assert!(unit.contains(0.5));
        assert!(unit.contains(1.0));
        assert!(!unit.contains(-0.1));
        assert!(!unit.contains(1.1));
        assert!((unit.clamp(1.5) - 1.0).abs() < 1e-10);
        assert!((unit.clamp(-0.5) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_offset_ratio() {
        // WCAG contrast ratio with 0.05 offset
        let r = OffsetRatio { offset: 0.05 };

        // White (1.0) vs black (0.0): (1.05) / (0.05) = 21.0
        assert!((r.eval(1.0, 0.0) - 21.0).abs() < 1e-10);

        // Same color: ratio = 1.0
        assert!((r.eval(0.5, 0.5) - 1.0).abs() < 1e-10);

        // Always >= 1.0
        assert!(RatioBounded { ratio: r }.holds());
    }

    #[test]
    fn test_convex_weights_axiom() {
        let lc = LinearCombination::new(vec![0.2126, 0.7152, 0.0722]);
        assert!(ConvexWeights { combination: lc }.holds());
    }

    #[test]
    fn test_non_convex_rejected() {
        let lc = LinearCombination::new(vec![0.5, 0.5, 0.5]); // sums to 1.5
        assert!(!ConvexWeights { combination: lc }.holds());
    }

    // ── Property-based tests ──
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_piecewise_output_in_range(x in 0.0f64..=1.0) {
            // sRGB linearize maps [0,1] → [0,1]
            let f = Piecewise {
                threshold: 0.04045,
                below: |c| c / 12.92,
                above: |c| ((c + 0.055) / 1.055).powf(2.4),
            };
            let y = f.eval(x);
            prop_assert!(y >= 0.0 && y <= 1.0, "f({}) = {} not in [0,1]", x, y);
        }

        #[test]
        fn prop_piecewise_monotone(a in 0.0f64..=1.0, b in 0.0f64..=1.0) {
            // sRGB linearize is monotone: a <= b → f(a) <= f(b)
            let f = Piecewise {
                threshold: 0.04045,
                below: |c| c / 12.92,
                above: |c| ((c + 0.055) / 1.055).powf(2.4),
            };
            if a <= b {
                prop_assert!(f.eval(a) <= f.eval(b) + 1e-10, "not monotone: f({}) > f({})", a, b);
            }
        }

        #[test]
        fn prop_linear_combination_homogeneous(k in 0.1f64..=10.0, x in 0.0f64..=1.0) {
            // L(k*x) = k * L(x) when applied to uniform input
            let lc = LinearCombination::new(vec![0.2126, 0.7152, 0.0722]);
            let vals = vec![x, x, x];
            let scaled_vals = vec![k * x, k * x, k * x];
            let l = lc.eval(&vals);
            let l_scaled = lc.eval(&scaled_vals);
            prop_assert!((l_scaled - k * l).abs() < 1e-10);
        }

        #[test]
        fn prop_interval_clamp_idempotent(x in -10.0f64..=10.0) {
            let unit = Interval::UNIT;
            let clamped = unit.clamp(x);
            prop_assert_eq!(unit.clamp(clamped), clamped);
        }

        #[test]
        fn prop_offset_ratio_symmetric(a in 0.0f64..=1.0, b in 0.0f64..=1.0) {
            // ratio(a, b) == ratio(b, a) — commutative
            let r = OffsetRatio { offset: 0.05 };
            prop_assert!((r.eval(a, b) - r.eval(b, a)).abs() < 1e-10);
        }

        #[test]
        fn prop_offset_ratio_always_ge_one(a in 0.0f64..=1.0, b in 0.0f64..=1.0) {
            let r = OffsetRatio { offset: 0.05 };
            prop_assert!(r.eval(a, b) >= 1.0 - 1e-10);
        }
    }
}
