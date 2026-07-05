//! Functor: SmartElement → MAPE-K.
//!
//! A smart element *is* an autonomic element (Kephart & Chess 2003): its
//! self-* properties are the loop's phases, its local ontology and TEDS
//! are the knowledge substrate, and its transducer/managed side is what
//! the loop monitors. This functor witnesses that reading, and its image
//! is what the `SmartClosesMapeKLoop` axiom checks covers every MAPE-K
//! phase.
//!
//! # Object mapping (Kephart & Chess 2003 §2–§3; Table 1)
//!
//! | SmartElement | MAPE-K | Why |
//! |---|---|---|
//! | `LocalOntology`, `Teds` | `Knowledge` | Faithful: the local ontology IS the loop's knowledge; a TEDS is its standardized self-description (IEEE 1451.0-2007 §5) |
//! | `SelfHealing` | `Analyze` | K&C Table 1: healing detects + diagnoses — the Analyze function |
//! | `SelfOptimization` | `Plan` | K&C Table 1: optimization decides how to retune — the Plan function |
//! | `SelfConfiguration` | `Execute` | K&C Table 1: configuration installs/adapts — the Execute function |
//! | `SelfProtection` | `Monitor` | K&C Table 1: protection continuously watches for attacks — the Monitor function |
//! | `Transducer`, `ManagedElement` | `Monitor` | The sensed / managed side the loop observes |
//! | `Ncap` | `Execute` | IEEE 1451: the NCAP operates the transducer — the acting side |
//! | `AutonomicManager`, `SmartElement`, `SmartSensor`, `SmartDriver`, `SelfStarProperty` | `MapeKPhase` | Documented collapse onto the abstract phase parent: the manager drives the phase cycle and the elements/self-* umbrella are characterised by running it |
//!
//! # Morphism-kind mapping
//!
//! The four canonical Relations-ontology kinds map to their namesakes;
//! the five custom edges ride MAPE-K's two non-taxonomic arrows —
//! `HandsOffTo` (the loop's operative hand-off) and `Consults` (a
//! knowledge lookup):
//!
//! - `Exhibits`, `Operates`, `Manages` → `HandsOffTo` (the element engages
//!   its phases, the driver acts on the transducer, the manager acts on
//!   the managed element — all loop hand-offs);
//! - `Carries`, `DescribedBy` → `Consults` (carrying the local ontology
//!   and reading the TEDS self-description are knowledge consultations —
//!   a documented collapse onto MAPE-K's substrate arrow).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SmartElementCategory, SmartElementConcept, SmartElementRelation, SmartElementRelationKind,
};
use crate::formal::systems::mape_k::ontology::{
    MapeKCategory, MapeKConcept, MapeKRelation, MapeKRelationKind,
};

/// Maps each smart-element concept to the MAPE-K phase (or the knowledge
/// substrate, or the abstract phase parent) it plays in the element's
/// autonomic loop.
pub struct SmartElementToMapeK;

impl Functor for SmartElementToMapeK {
    type Source = SmartElementCategory;
    type Target = MapeKCategory;

    fn map_object(obj: &SmartElementConcept) -> MapeKConcept {
        use SmartElementConcept as C;
        match obj {
            // Faithful: the local ontology / TEDS is the loop's Knowledge.
            C::LocalOntology | C::Teds => MapeKConcept::Knowledge,
            // The self-* properties are the loop's four phases (K&C Table 1).
            C::SelfHealing => MapeKConcept::Analyze,
            C::SelfOptimization => MapeKConcept::Plan,
            C::SelfConfiguration => MapeKConcept::Execute,
            C::SelfProtection => MapeKConcept::Monitor,
            // The sensed / managed side the loop observes.
            C::Transducer | C::ManagedElement => MapeKConcept::Monitor,
            // IEEE 1451: the NCAP operates the transducer — the acting side.
            C::Ncap => MapeKConcept::Execute,
            // Documented collapse onto the abstract phase parent: the manager
            // drives the phase cycle, and the elements and self-* umbrella
            // are characterised by running it.
            C::AutonomicManager
            | C::SmartElement
            | C::SmartSensor
            | C::SmartDriver
            | C::SelfStarProperty => MapeKConcept::MapeKPhase,
        }
    }

    fn map_morphism(m: &SmartElementRelation) -> MapeKRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SmartElementRelationKind::Identity => return MapeKCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to namesakes
            // (Smith 2005 OBO-RO).
            SmartElementRelationKind::Subsumption => MapeKRelationKind::Subsumption,
            SmartElementRelationKind::Parthood => MapeKRelationKind::Parthood,
            SmartElementRelationKind::Causation => MapeKRelationKind::Causation,
            SmartElementRelationKind::Opposition => MapeKRelationKind::Opposition,
            // Loop hand-offs: engaging phases, operating the transducer,
            // acting on the managed element.
            SmartElementRelationKind::Exhibits
            | SmartElementRelationKind::Operates
            | SmartElementRelationKind::Manages => MapeKRelationKind::HandsOffTo,
            // Knowledge consultations: carrying the local ontology, reading
            // the TEDS self-description (documented collapse).
            SmartElementRelationKind::Carries | SmartElementRelationKind::DescribedBy => {
                MapeKRelationKind::Consults
            }
        };
        MapeKRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SmartElementToMapeK,
    "Kephart & Chess (2003) IEEE Computer 36(1)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn smart_element_to_mape_k_functor_laws() {
        assert_functor_laws::<SmartElementToMapeK>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn local_ontology_and_teds_are_knowledge() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToMapeK::map_object(&C::LocalOntology),
            MapeKConcept::Knowledge
        );
        assert_eq!(
            SmartElementToMapeK::map_object(&C::Teds),
            MapeKConcept::Knowledge
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn self_star_properties_map_to_the_four_phases() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToMapeK::map_object(&C::SelfProtection),
            MapeKConcept::Monitor
        );
        assert_eq!(
            SmartElementToMapeK::map_object(&C::SelfHealing),
            MapeKConcept::Analyze
        );
        assert_eq!(
            SmartElementToMapeK::map_object(&C::SelfOptimization),
            MapeKConcept::Plan
        );
        assert_eq!(
            SmartElementToMapeK::map_object(&C::SelfConfiguration),
            MapeKConcept::Execute
        );
    }

    /// The image covers every phase and the knowledge substrate — the
    /// structural half of the `SmartClosesMapeKLoop` axiom.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn image_covers_all_five_mape_k_concepts() {
        use pr4xis::category::FinitelyGenerated;
        let image: Vec<MapeKConcept> = SmartElementConcept::variants()
            .into_iter()
            .map(|c| SmartElementToMapeK::map_object(&c))
            .collect();
        for phase in [
            MapeKConcept::Monitor,
            MapeKConcept::Analyze,
            MapeKConcept::Plan,
            MapeKConcept::Execute,
            MapeKConcept::Knowledge,
        ] {
            assert!(image.contains(&phase), "image should cover {phase:?}");
        }
    }
}
