//! The circle-group ontology for [`Angle`] — S¹ = ℝ/2πℤ ≅ SO(2),
//! with the group laws discharged as computed, falsifiable axioms.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use core::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::formal::math::angle::Angle;

pr4xis::ontology! {
    name: "Angle",
    source: "The circle group S¹ = ℝ/2πℤ ≅ SO(2), a compact abelian Lie group (Stillwell 2008, Naive Lie Theory §1–2); shortest-arc difference from directional statistics (Mardia & Jupp 2000, Directional Statistics §1–2); the plane-angle dimensionality problem (Quincey 2016; Krylov 2019, Metrologia 56, 065009)",

    concepts: [Zero, RightAngle, StraightAngle, FullTurn],

    labels: {
        Zero: ("en", "Zero angle", "0 rad = 0° — the additive identity of the circle group."),
        RightAngle: ("en", "Right angle", "π/2 rad = 90°."),
        StraightAngle: ("en", "Straight angle", "π rad = 180°."),
        FullTurn: ("en", "Full turn", "2π rad = 360°, which is congruent to 0 in ℝ/2πℤ."),
    },
}

/// Quality: the canonical radian measure of each notable angle.
#[derive(Debug, Clone)]
pub struct RadianMeasure;

impl Quality for RadianMeasure {
    type Individual = AngleConcept;
    type Value = f64;

    fn get(&self, a: &AngleConcept) -> Option<f64> {
        Some(match a {
            AngleConcept::Zero => 0.0,
            AngleConcept::RightAngle => FRAC_PI_2,
            AngleConcept::StraightAngle => PI,
            AngleConcept::FullTurn => TAU,
        })
    }
}

impl AngleConcept {
    /// The notable angle as an [`Angle`] value.
    pub fn angle(&self) -> Angle {
        Angle::from_radians(RadianMeasure.get(self).unwrap_or(0.0))
    }
}

impl Ontology for AngleOntology {
    type Cat = AngleCategory;
    type Qual = RadianMeasure;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(FullTurnIsIdentity));
        axioms.push(Box::new(AdditionIsAbelianModTwoPi));
        axioms.push(Box::new(NormalizationIsIdempotent));
        axioms.push(Box::new(DifferenceIsShortestArc));
        axioms.push(Box::new(RightAngleIsHalfStraight));
        axioms
    }
}

/// A small spread of fixture angles (degrees) for the computed group laws.
fn fixtures() -> [Angle; 6] {
    [
        Angle::from_degrees(0.0),
        Angle::from_degrees(30.0),
        Angle::from_degrees(170.0),
        Angle::from_degrees(-100.0),
        Angle::from_degrees(250.0),
        Angle::from_degrees(359.0),
    ]
}

/// Axiom: a full turn is the group identity — `2π ≡ 0` in ℝ/2πℤ.
pub struct FullTurnIsIdentity;

impl Axiom for FullTurnIsIdentity {
    fn verify(&self) -> Verdict {
        let ok = Angle::from_turns(1.0).circle_eq(&Angle::ZERO, 1e-12)
            && AngleConcept::FullTurn
                .angle()
                .circle_eq(&AngleConcept::Zero.angle(), 1e-12);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FullTurnIsIdentity",
        "a full turn (2π) is congruent to the zero angle in the circle group ℝ/2πℤ",
        "The circle group S¹ = ℝ/2πℤ (Stillwell 2008, Naive Lie Theory §1)"
    );
}
pr4xis::register_axiom!(
    FullTurnIsIdentity,
    "The circle group S¹ = ℝ/2πℤ (Stillwell 2008, Naive Lie Theory §1)"
);

/// Axiom: addition is abelian with identity `0` and inverse `−a`, modulo 2π.
pub struct AdditionIsAbelianModTwoPi;

impl Axiom for AdditionIsAbelianModTwoPi {
    fn verify(&self) -> Verdict {
        let fs = fixtures();
        let ok = fs.iter().all(|a| {
            // identity and inverse
            a.add(&Angle::ZERO).circle_eq(a, 1e-12)
                && a.add(&a.negate()).circle_eq(&Angle::ZERO, 1e-12)
                && fs.iter().all(|b| {
                    // commutativity
                    a.add(b).circle_eq(&b.add(a), 1e-12)
                        && fs.iter().all(|c| {
                            // associativity
                            a.add(b).add(c).circle_eq(&a.add(&b.add(c)), 1e-12)
                        })
                })
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AdditionIsAbelianModTwoPi",
        "angle addition is associative, commutative, has identity 0 and inverse −a, modulo 2π (abelian group)",
        "The circle group S¹ = ℝ/2πℤ ≅ SO(2) (Stillwell 2008, Naive Lie Theory §2)"
    );
}
pr4xis::register_axiom!(
    AdditionIsAbelianModTwoPi,
    "The circle group S¹ = ℝ/2πℤ ≅ SO(2) (Stillwell 2008, Naive Lie Theory §2)"
);

/// Axiom: normalisation is idempotent and lands in `[-π, π)`.
pub struct NormalizationIsIdempotent;

impl Axiom for NormalizationIsIdempotent {
    fn verify(&self) -> Verdict {
        let ok = fixtures()
            .iter()
            .chain(&[Angle::from_degrees(720.0), Angle::from_degrees(-540.0)])
            .all(|a| {
                let n = a.normalized_signed();
                let r = n.radians();
                (-PI..PI).contains(&r)
                    && n.normalized_signed().radians() == r
                    && n.circle_eq(a, 1e-12)
            });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NormalizationIsIdempotent",
        "signed normalisation maps into [-π, π), is idempotent, and preserves the circle-group class",
        "The circle group S¹ = ℝ/2πℤ (Stillwell 2008, Naive Lie Theory §1)"
    );
}
pr4xis::register_axiom!(
    NormalizationIsIdempotent,
    "The circle group S¹ = ℝ/2πℤ (Stillwell 2008, Naive Lie Theory §1)"
);

/// Axiom: the difference between two angles is the shortest arc — never
/// exceeding π in magnitude.
pub struct DifferenceIsShortestArc;

impl Axiom for DifferenceIsShortestArc {
    fn verify(&self) -> Verdict {
        let fs = fixtures();
        let ok = fs.iter().all(|a| {
            fs.iter().all(|b| {
                let d = a.difference(b);
                // shortest arc: |d| ≤ π, and a + d ≡ b
                d.radians().abs() <= PI + 1e-12 && a.add(&d).circle_eq(b, 1e-9)
            })
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DifferenceIsShortestArc",
        "the signed difference of two angles is the shortest arc (|Δ| ≤ π) and satisfies a + (b − a) ≡ b",
        "Directional statistics — circular difference / shortest arc (Mardia & Jupp 2000, Directional Statistics §1–2)"
    );
}
pr4xis::register_axiom!(
    DifferenceIsShortestArc,
    "Directional statistics — circular difference / shortest arc (Mardia & Jupp 2000, Directional Statistics §1–2)"
);

/// Axiom: two right angles make a straight angle (`2 · π/2 = π`).
pub struct RightAngleIsHalfStraight;

impl Axiom for RightAngleIsHalfStraight {
    fn verify(&self) -> Verdict {
        let right = AngleConcept::RightAngle.angle();
        let straight = AngleConcept::StraightAngle.angle();
        if right.add(&right).circle_eq(&straight, 1e-12) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RightAngleIsHalfStraight",
        "two right angles compose to a straight angle (π/2 + π/2 = π)",
        "Euclid, Elements, Book I, Definition 10 (right angle)"
    );
}
pr4xis::register_axiom!(
    RightAngleIsHalfStraight,
    "Euclid, Elements, Book I, Definition 10 (right angle)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<AngleCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AngleOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn all_group_axioms_hold() {
        assert!(FullTurnIsIdentity.verify().is_ok());
        assert!(AdditionIsAbelianModTwoPi.verify().is_ok());
        assert!(NormalizationIsIdempotent.verify().is_ok());
        assert!(DifferenceIsShortestArc.verify().is_ok());
        assert!(RightAngleIsHalfStraight.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axioms_can_fail_on_a_broken_group() {
        // The group laws are computed, not asserted: a non-shortest "difference"
        // (naive subtraction, no normalisation) would exceed π and be caught.
        let a = Angle::from_degrees(350.0);
        let b = Angle::from_degrees(10.0);
        let naive = a.sub(&b); // 340° — NOT the shortest arc
        assert!(naive.radians().abs() > PI);
        assert!(a.difference(&b).radians().abs() <= PI);
    }
}
