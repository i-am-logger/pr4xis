//! Angle — a first-class type for angles, grounded in the circle group.
//!
//! An angle is **not** a bare `f64` and not a dimensionless [`Quantity`]: it is
//! an element of the circle group **S¹ = ℝ/2πℤ ≅ SO(2)**. Modelling it as its
//! own type buys real safety a `Quantity(ANGLE)` cannot: radians and degrees
//! are set by explicit constructors (never confused), you cannot add a length
//! to an angle, and the circle-group operations (normalisation, shortest-arc
//! difference) are correct by construction rather than open-coded at each site.
//!
//! # Literature
//!
//! - The circle group **S¹ = ℝ/2πℤ**, isomorphic to the rotation group
//!   **SO(2)** — a 1-dimensional compact abelian Lie group with product
//!   `(θ)·(φ) = (θ+φ)`. Stillwell (2008), *Naive Lie Theory*, §1–2.
//! - Shortest-arc difference and wrap-around are the founding concern of
//!   **directional (circular) statistics** — "180° is not a sensible mean of
//!   2° and 358°". Mardia & Jupp (2000), *Directional Statistics*, §1–2.
//! - Why a dedicated type rather than `Quantity(ANGLE)`: under the SI a plane
//!   angle is a ratio of two lengths (L/L), so `Dimension::ANGLE ==
//!   DIMENSIONLESS` and a `Quantity(ANGLE)` carries no dimensional protection.
//!   That the numerical value is a pure number does not make the quantity
//!   itself dimensionless — an open problem in metrology (Quincey (2016),
//!   *A proposal to classify the radian as a base unit in the SI*; Krylov
//!   (2019), *On the status of plane and solid angles in the SI*, Metrologia
//!   56, 065009). A distinct `Angle` type restores the safety the SI dimension
//!   vector cannot.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use core::f64::consts::{PI, TAU};

pub mod ontology;

/// An angle — an element of the circle group `S¹ = ℝ/2πℤ`, stored canonically
/// in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    /// The additive identity of the circle group — 0 rad.
    pub const ZERO: Angle = Angle { radians: 0.0 };

    /// Construct from a value in **radians**.
    pub fn from_radians(radians: f64) -> Self {
        Self { radians }
    }

    /// Construct from a value in **degrees**.
    pub fn from_degrees(degrees: f64) -> Self {
        Self {
            radians: degrees.to_radians(),
        }
    }

    /// Construct from a number of **turns** (1 turn = 2π rad = 360°).
    pub fn from_turns(turns: f64) -> Self {
        Self {
            radians: turns * TAU,
        }
    }

    /// The angle in radians.
    pub fn radians(&self) -> f64 {
        self.radians
    }

    /// The angle in degrees.
    pub fn degrees(&self) -> f64 {
        self.radians.to_degrees()
    }

    /// Sine of the angle.
    pub fn sin(&self) -> f64 {
        self.radians.sin()
    }

    /// Cosine of the angle.
    pub fn cos(&self) -> f64 {
        self.radians.cos()
    }

    /// Tangent of the angle.
    pub fn tan(&self) -> f64 {
        self.radians.tan()
    }

    /// Group operation: `self + other` (before normalisation).
    pub fn add(&self, other: &Angle) -> Angle {
        Angle {
            radians: self.radians + other.radians,
        }
    }

    /// Group inverse composed with addition: `self - other`.
    pub fn sub(&self, other: &Angle) -> Angle {
        Angle {
            radians: self.radians - other.radians,
        }
    }

    /// Group inverse: `-self`.
    pub fn negate(&self) -> Angle {
        Angle {
            radians: -self.radians,
        }
    }

    /// Canonical representative in `[-π, π)` — the signed normalisation.
    pub fn normalized_signed(&self) -> Angle {
        let mut r = self.radians % TAU;
        if r >= PI {
            r -= TAU;
        } else if r < -PI {
            r += TAU;
        }
        Angle { radians: r }
    }

    /// Canonical representative in `[0, 2π)` — the positive normalisation.
    pub fn normalized_positive(&self) -> Angle {
        let mut r = self.radians % TAU;
        if r < 0.0 {
            r += TAU;
        }
        Angle { radians: r }
    }

    /// The shortest signed arc from `self` to `other`, in `[-π, π)`.
    pub fn difference(&self, other: &Angle) -> Angle {
        other.sub(self).normalized_signed()
    }

    /// Circle-group equality: equal modulo full turns (`a ≡ a + 2πk`).
    pub fn circle_eq(&self, other: &Angle, tol: f64) -> bool {
        self.difference(other).radians.abs() < tol
    }
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    /// Any angle built from a finite radian value across many turns.
    fn any_angle() -> impl Strategy<Value = Angle> {
        (-100.0f64..100.0).prop_map(Angle::from_radians)
    }

    proptest! {
        /// Circle-group addition is commutative (mod 2π).
        #[test]
        fn addition_commutes(a in any_angle(), b in any_angle()) {
            prop_assert!(a.add(&b).circle_eq(&b.add(&a), 1e-9));
        }

        /// Circle-group addition is associative (mod 2π).
        #[test]
        fn addition_associates(a in any_angle(), b in any_angle(), c in any_angle()) {
            prop_assert!(a.add(&b).add(&c).circle_eq(&a.add(&b.add(&c)), 1e-9));
        }

        /// Zero is the identity and negation is the inverse (mod 2π).
        #[test]
        fn identity_and_inverse(a in any_angle()) {
            prop_assert!(a.add(&Angle::ZERO).circle_eq(&a, 1e-12));
            prop_assert!(a.add(&a.negate()).circle_eq(&Angle::ZERO, 1e-9));
        }

        /// Signed normalisation lands in [-π, π) and is idempotent, preserving the class.
        #[test]
        fn normalization_in_range_idempotent(a in any_angle()) {
            let n = a.normalized_signed();
            prop_assert!(n.radians() >= -PI && n.radians() < PI);
            prop_assert_eq!(n.normalized_signed().radians(), n.radians());
            prop_assert!(n.circle_eq(&a, 1e-9));
        }

        /// The signed difference is the shortest arc (|Δ| ≤ π) and closes: a + Δ ≡ b.
        #[test]
        fn difference_is_shortest_arc(a in any_angle(), b in any_angle()) {
            let d = a.difference(&b);
            prop_assert!(d.radians().abs() <= PI + 1e-9);
            prop_assert!(a.add(&d).circle_eq(&b, 1e-9));
        }

        /// Degrees and radians are consistent — the two constructors never disagree.
        #[test]
        fn degree_radian_consistent(deg in -1000.0f64..1000.0) {
            let a = Angle::from_degrees(deg);
            prop_assert!((a.degrees() - deg).abs() < 1e-6);
            prop_assert!((a.radians() - deg.to_radians()).abs() < 1e-9);
        }

        /// An angle's sine and cosine lie on the unit circle.
        #[test]
        fn sin_cos_on_unit_circle(a in any_angle()) {
            prop_assert!((a.sin().powi(2) + a.cos().powi(2) - 1.0).abs() < 1e-9);
        }
    }

    pr4xis::register_praxis_value!(addition_commutes, Verifiable);
    pr4xis::register_praxis_value!(addition_associates, Verifiable);
    pr4xis::register_praxis_value!(identity_and_inverse, Verifiable);
    pr4xis::register_praxis_value!(normalization_in_range_idempotent, Verifiable);
    pr4xis::register_praxis_value!(difference_is_shortest_arc, Verifiable);
    pr4xis::register_praxis_value!(degree_radian_consistent, Verifiable);
    pr4xis::register_praxis_value!(sin_cos_on_unit_circle, Verifiable);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn degree_radian_constructors_distinct() {
        // 180 degrees is π radians — the whole point is you cannot confuse them.
        assert!((Angle::from_degrees(180.0).radians() - PI).abs() < 1e-12);
        assert!((Angle::from_radians(PI).degrees() - 180.0).abs() < 1e-12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_turn_is_identity() {
        assert!(Angle::from_turns(1.0).circle_eq(&Angle::ZERO, 1e-12));
        assert!(Angle::from_degrees(360.0).circle_eq(&Angle::ZERO, 1e-12));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn shortest_arc_never_exceeds_pi() {
        let a = Angle::from_degrees(350.0);
        let b = Angle::from_degrees(10.0);
        // Naive difference is 340°, but the shortest arc is +20°.
        assert!((a.difference(&b).degrees() - 20.0).abs() < 1e-9);
        assert!(a.difference(&b).radians().abs() <= PI + 1e-12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn signed_normalization_in_range() {
        for deg in [-720.0, -190.0, 0.0, 179.0, 181.0, 540.0] {
            let n = Angle::from_degrees(deg).normalized_signed().radians();
            assert!((-PI..PI).contains(&n), "{deg} -> {n}");
        }
    }
}
