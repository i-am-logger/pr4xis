//! Single-target tracking — target state-component ontology.
//!
//! Models the kinematic state components a tracker estimates for a
//! single target: position, velocity, acceleration, and turn-rate
//! (for maneuvering models). The causal-derivation chain
//! `Position → Velocity → Acceleration` is recorded as `causes:` edges,
//! reflecting that each higher derivative is the time-derivative of the
//! one below.
//!
//! # Literature
//!
//! - **Bar-Shalom, Li & Kirubarajan (2001)** *Estimation with
//!   Applications to Tracking and Navigation*, Ch. 6 — the canonical
//!   single-target kinematic state-vector decomposition (position /
//!   velocity / acceleration / turn-rate) used in constant-velocity,
//!   constant-acceleration, and coordinated-turn motion models.
//! - **Li & Jilkov (2003)** "Survey of Maneuvering Target Tracking. Part I:
//!   Dynamic Models", *IEEE Transactions on Aerospace and Electronic
//!   Systems* 39(4) — the multi-model dynamic survey establishing the
//!   turn-rate component for maneuvering-target trackers.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SingleTarget",
    source: "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation Ch. 6; Li & Jilkov (2003) Survey of Maneuvering Target Tracking. Part I: Dynamic Models, IEEE Transactions on Aerospace and Electronic Systems 39(4)",

    concepts: [
        // Bar-Shalom (2001) §6.2 single-target kinematic state components.
        Position,
        Velocity,
        Acceleration,
        // Li & Jilkov (2003): turn-rate for the coordinated-turn model.
        TurnRate,
    ],

    labels: {
        Position: ("en", "Position",
            "Bar-Shalom (2001) §6.2: target Cartesian position (x, y, z)."),
        Velocity: ("en", "Velocity",
            "Bar-Shalom (2001) §6.2: target velocity (vx, vy, vz) — first time-derivative of position."),
        Acceleration: ("en", "Acceleration",
            "Bar-Shalom (2001) §6.2: target acceleration (ax, ay, az) — second time-derivative of position."),
        TurnRate: ("en", "Turn rate",
            "Li & Jilkov (2003): angular turn rate ω — used in the coordinated-turn model for maneuvering targets."),
    },

    // Bar-Shalom (2001) §6.2 — the kinematic differentiation chain.
    causes: [
        (Position, Velocity),
        (Velocity, Acceleration),
    ],
}

/// Quality: dimensionality of each state component in the standard 3D
/// Cartesian formulation.
///
/// Bar-Shalom (2001) §6.2 — Position / Velocity / Acceleration are
/// 3-vectors in the ECEF / ENU frame; TurnRate is a scalar in the
/// coordinated-turn model.
#[derive(Debug, Clone)]
pub struct ComponentDimension;

impl Quality for ComponentDimension {
    type Individual = SingleTargetConcept;
    type Value = usize;

    fn get(&self, c: &SingleTargetConcept) -> Option<usize> {
        Some(match c {
            SingleTargetConcept::Position => 3,
            SingleTargetConcept::Velocity => 3,
            SingleTargetConcept::Acceleration => 3,
            SingleTargetConcept::TurnRate => 1,
        })
    }
}

impl Ontology for SingleTargetOntology {
    type Cat = SingleTargetCategory;
    type Qual = ComponentDimension;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(VelocityDerivesFromPosition));
        axioms.push(Box::new(AccelerationDerivesFromVelocity));
        axioms
    }
}

/// Axiom: velocity is the time derivative of position — recorded as a
/// Causation edge in the ontology.
///
/// Bar-Shalom, Li & Kirubarajan (2001) §6.2 — in the constant-velocity
/// state-space model `x_dot = [v; 0]`, position differentiates to
/// velocity by construction.
pub struct VelocityDerivesFromPosition;

impl Axiom for VelocityDerivesFromPosition {
    fn verify(&self) -> Verdict {
        let has_causation = SingleTargetCategory::morphisms().iter().any(|m| {
            m.kind() == SingleTargetRelationKind::Causation
                && m.source() == SingleTargetConcept::Position
                && m.target() == SingleTargetConcept::Velocity
        });
        if has_causation {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VelocityDerivesFromPosition",
        "velocity is the time derivative of position",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2"
    );
}

pr4xis::register_axiom!(
    VelocityDerivesFromPosition,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2"
);

/// Axiom: acceleration is the time derivative of velocity — recorded as
/// a Causation edge in the ontology.
///
/// Bar-Shalom (2001) §6.2 — in the constant-acceleration state-space
/// model `v_dot = a`.
pub struct AccelerationDerivesFromVelocity;

impl Axiom for AccelerationDerivesFromVelocity {
    fn verify(&self) -> Verdict {
        let has_causation = SingleTargetCategory::morphisms().iter().any(|m| {
            m.kind() == SingleTargetRelationKind::Causation
                && m.source() == SingleTargetConcept::Velocity
                && m.target() == SingleTargetConcept::Acceleration
        });
        if has_causation {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AccelerationDerivesFromVelocity",
        "acceleration is the time derivative of velocity",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2"
    );
}

pr4xis::register_axiom!(
    AccelerationDerivesFromVelocity,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SingleTargetCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SingleTargetOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_state_components() {
        assert_eq!(SingleTargetConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn position_3d() {
        assert_eq!(
            ComponentDimension.get(&SingleTargetConcept::Position),
            Some(3)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn turnrate_scalar() {
        assert_eq!(
            ComponentDimension.get(&SingleTargetConcept::TurnRate),
            Some(1)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn position_causes_velocity() {
        let caus: Vec<_> = SingleTargetCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SingleTargetRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(caus.contains(&(SingleTargetConcept::Position, SingleTargetConcept::Velocity)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn velocity_derives_axiom_holds() {
        assert!(VelocityDerivesFromPosition.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn acceleration_derives_axiom_holds() {
        assert!(AccelerationDerivesFromVelocity.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = SingleTargetConcept> {
        proptest::sample::select(SingleTargetConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SingleTargetCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SingleTargetOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_dimension_total(c in arb_concept()) {
            prop_assert!(ComponentDimension.get(&c).is_some());
        }

        #[test]
        fn prop_dimension_positive(c in arb_concept()) {
            // Every component has at least one dimension.
            let d = ComponentDimension.get(&c).unwrap();
            prop_assert!(d >= 1);
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_dimension_total, Verifiable);
    pr4xis::register_praxis_value!(prop_dimension_positive, Verifiable);
}
