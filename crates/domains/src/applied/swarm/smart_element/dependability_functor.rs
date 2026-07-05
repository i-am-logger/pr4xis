//! Functor: SmartElement → Dependability.
//!
//! The dependability reading (Avizienis, Laprie, Randell & Landwehr 2004):
//! the self-* properties are the *means* by which a smart element attains
//! dependability. Two arms are faithful to Avizienis §5.2–§5.3 and the
//! tests pin them:
//!
//! - `SelfHealing → ErrorRecovery` — healing replaces erroneous state with
//!   error-free state;
//! - `SelfProtection → FaultHandling` — protection prevents located faults
//!   (here, equivocators) from being activated again by diagnosis,
//!   isolation, and reconfiguration. This is the structural half of the
//!   `SelfProtectionExcludesEquivocators` axiom.
//!
//! # Object mapping (each arm documented)
//!
//! | SmartElement | Dependability | Why |
//! |---|---|---|
//! | `SelfHealing` | `ErrorRecovery` | Avizienis §5.2: replace erroneous state with error-free state |
//! | `SelfProtection` | `FaultHandling` | Avizienis §5.2: diagnosis, isolation, reconfiguration — the equivocator exclusion |
//! | `SelfConfiguration`, `SelfOptimization`, `SelfStarProperty` | `Means` | The self-* properties are means to dependability (Avizienis §5) — the means umbrella |
//! | `AutonomicManager` | `FaultTolerance` | The manager delivers correct service despite faults (Avizienis §5.2) |
//! | `SmartElement`, `SmartSensor`, `SmartDriver`, `Transducer`, `Ncap`, `ManagedElement`, `LocalOntology`, `Teds` | `Service` | The service-delivery surface (documented collapse, following consensus→dependability's `Peer → Service`) |
//!
//! # Morphism-kind mapping
//!
//! Dependability's cross-concept vocabulary is the causal fault→error→
//! failure chain (plus the canonical structural kinds). The four canonical
//! kinds map to their namesakes; every custom smart-element edge
//! (`Carries`, `Exhibits`, `Operates`, `DescribedBy`, `Manages`) →
//! `Causation` — the same treatment consensus→dependability gives its
//! action/derivation edges (documented collapse).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SmartElementCategory, SmartElementConcept, SmartElementRelation, SmartElementRelationKind,
};
use crate::applied::dependability::ontology::{
    DependabilityCategory, DependabilityConcept, DependabilityRelation, DependabilityRelationKind,
};

/// Maps each smart-element concept to its Avizienis-taxonomy role.
pub struct SmartElementToDependability;

impl Functor for SmartElementToDependability {
    type Source = SmartElementCategory;
    type Target = DependabilityCategory;

    fn map_object(obj: &SmartElementConcept) -> DependabilityConcept {
        use SmartElementConcept as C;
        match obj {
            // Faithful (Avizienis §5.2): healing is error recovery.
            C::SelfHealing => DependabilityConcept::ErrorRecovery,
            // Faithful (Avizienis §5.2): protection is fault handling —
            // diagnosis, isolation, reconfiguration (the equivocator
            // exclusion). Checked by the SelfProtectionExcludesEquivocators
            // axiom.
            C::SelfProtection => DependabilityConcept::FaultHandling,
            // The self-* properties are means to dependability (Avizienis §5).
            C::SelfConfiguration | C::SelfOptimization | C::SelfStarProperty => {
                DependabilityConcept::Means
            }
            // The manager delivers correct service despite faults.
            C::AutonomicManager => DependabilityConcept::FaultTolerance,
            // The service-delivery surface (documented collapse).
            C::SmartElement
            | C::SmartSensor
            | C::SmartDriver
            | C::Transducer
            | C::Ncap
            | C::ManagedElement
            | C::LocalOntology
            | C::Teds => DependabilityConcept::Service,
        }
    }

    fn map_morphism(m: &SmartElementRelation) -> DependabilityRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SmartElementRelationKind::Identity => return DependabilityCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to namesakes
            // (Smith 2005 OBO-RO).
            SmartElementRelationKind::Subsumption => DependabilityRelationKind::Subsumption,
            SmartElementRelationKind::Parthood => DependabilityRelationKind::Parthood,
            SmartElementRelationKind::Causation => DependabilityRelationKind::Causation,
            SmartElementRelationKind::Opposition => DependabilityRelationKind::Opposition,
            // Every constitutive/action edge reads as a causal arrow in the
            // dependability fault→error→failure vocabulary (documented
            // collapse — the consensus→dependability treatment).
            SmartElementRelationKind::Carries
            | SmartElementRelationKind::Exhibits
            | SmartElementRelationKind::Operates
            | SmartElementRelationKind::DescribedBy
            | SmartElementRelationKind::Manages => DependabilityRelationKind::Causation,
        };
        DependabilityRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SmartElementToDependability,
    "Avizienis, Laprie, Randell & Landwehr (2004) IEEE TDSC 1(1); Kephart & Chess (2003)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn smart_element_to_dependability_functor_laws() {
        assert_functor_laws::<SmartElementToDependability>();
    }

    /// The two faithful means arms.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn healing_is_recovery_and_protection_is_fault_handling() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToDependability::map_object(&C::SelfHealing),
            DependabilityConcept::ErrorRecovery
        );
        assert_eq!(
            SmartElementToDependability::map_object(&C::SelfProtection),
            DependabilityConcept::FaultHandling
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn self_star_means_collapse() {
        use SmartElementConcept as C;
        for c in [
            C::SelfConfiguration,
            C::SelfOptimization,
            C::SelfStarProperty,
        ] {
            assert_eq!(
                SmartElementToDependability::map_object(&c),
                DependabilityConcept::Means,
                "{c:?} is a dependability means"
            );
        }
    }
}
