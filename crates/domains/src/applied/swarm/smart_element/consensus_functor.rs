//! Functor: SmartElement → Consensus.
//!
//! The peer reading of a smart element (Olfati-Saber, Fax & Murray 2007):
//! a smart element is a `Peer` of the estimation network — it holds a
//! local estimate, gossips it, and takes part in consensus-on-information.
//! This is the **faithful anchor** the `SmartIsFusionPeer` axiom checks:
//! the three Smart* concepts map to `Peer`. Its autonomic manager drives
//! the exchange (a `ConsensusProtocol`), and its self-protection is the
//! exclusion of a caught equivocator (a `DistrustedPeer`).
//!
//! # Object mapping (each collapse documented)
//!
//! | SmartElement | Consensus | Why |
//! |---|---|---|
//! | `SmartElement`, `SmartSensor`, `SmartDriver` | `Peer` | Faithful: a smart element is a fusion peer (axiom `SmartIsFusionPeer`) |
//! | `AutonomicManager` | `ConsensusProtocol` | It drives the exchange — the update rule of the peer's loop (documented) |
//! | `SelfProtection` | `DistrustedPeer` | Self-protection IS the exclusion of a caught equivocator (documented reading); makes `SelfProtection is_a SelfStarProperty` land on the real `DistrustedPeer is_a Peer` edge |
//! | `LocalOntology` | `Peer` | Honest umbrella collapse: the local ontology has no distinct consensus image |
//! | `Transducer`, `Teds`, `Ncap`, `ManagedElement`, `SelfStarProperty`, `SelfConfiguration`, `SelfHealing`, `SelfOptimization` | `Peer` | Documented forgetful collapse onto the peer umbrella |
//!
//! # Morphism-kind mapping
//!
//! The four canonical Relations-ontology kinds map to their namesakes; the
//! five custom edges collapse into two consensus buckets:
//!
//! - `Carries`, `DescribedBy` → `IdentifiedBy` (what a peer carries / is
//!   described by is how it is known — the identity arrow);
//! - `Exhibits`, `Operates`, `Manages` → `GossipsWith` (exhibiting
//!   behaviour, operating a transducer, and driving the managed element
//!   all collapse onto the peer-to-peer exchange arrow — documented).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SmartElementCategory, SmartElementConcept, SmartElementRelation, SmartElementRelationKind,
};
use crate::applied::swarm::consensus::ontology::{
    ConsensusCategory, ConsensusConcept, ConsensusRelation, ConsensusRelationKind,
};

/// Maps each smart-element concept to its consensus-peer reading.
pub struct SmartElementToConsensus;

impl Functor for SmartElementToConsensus {
    type Source = SmartElementCategory;
    type Target = ConsensusCategory;

    fn map_object(obj: &SmartElementConcept) -> ConsensusConcept {
        use SmartElementConcept as C;
        match obj {
            // Faithful: a smart element is a fusion peer (axiom).
            C::SmartElement | C::SmartSensor | C::SmartDriver => ConsensusConcept::Peer,
            // The manager drives the exchange — the peer's update rule.
            C::AutonomicManager => ConsensusConcept::ConsensusProtocol,
            // Self-protection is the exclusion of a caught equivocator.
            C::SelfProtection => ConsensusConcept::DistrustedPeer,
            // Honest and documented forgetful collapses onto the peer
            // umbrella: no distinct consensus image exists for these.
            C::LocalOntology
            | C::Transducer
            | C::Teds
            | C::Ncap
            | C::ManagedElement
            | C::SelfStarProperty
            | C::SelfConfiguration
            | C::SelfHealing
            | C::SelfOptimization => ConsensusConcept::Peer,
        }
    }

    fn map_morphism(m: &SmartElementRelation) -> ConsensusRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SmartElementRelationKind::Identity => return ConsensusCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to namesakes
            // (Smith 2005 OBO-RO).
            SmartElementRelationKind::Subsumption => ConsensusRelationKind::Subsumption,
            SmartElementRelationKind::Parthood => ConsensusRelationKind::Parthood,
            SmartElementRelationKind::Causation => ConsensusRelationKind::Causation,
            SmartElementRelationKind::Opposition => ConsensusRelationKind::Opposition,
            // What a peer carries / is described by is how it is known.
            SmartElementRelationKind::Carries | SmartElementRelationKind::DescribedBy => {
                ConsensusRelationKind::IdentifiedBy
            }
            // Exhibiting behaviour, operating a transducer, and driving the
            // managed element collapse onto the peer-exchange arrow.
            SmartElementRelationKind::Exhibits
            | SmartElementRelationKind::Operates
            | SmartElementRelationKind::Manages => ConsensusRelationKind::GossipsWith,
        };
        ConsensusRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SmartElementToConsensus,
    "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn smart_element_to_consensus_functor_laws() {
        assert_functor_laws::<SmartElementToConsensus>();
    }

    /// The faithful anchor: the three Smart* concepts are peers — the
    /// object half of the `SmartIsFusionPeer` axiom.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn smart_concepts_are_peers() {
        use SmartElementConcept as C;
        for c in [C::SmartElement, C::SmartSensor, C::SmartDriver] {
            assert_eq!(
                SmartElementToConsensus::map_object(&c),
                ConsensusConcept::Peer,
                "{c:?} should be a Peer"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn manager_drives_and_protection_distrusts() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToConsensus::map_object(&C::AutonomicManager),
            ConsensusConcept::ConsensusProtocol
        );
        assert_eq!(
            SmartElementToConsensus::map_object(&C::SelfProtection),
            ConsensusConcept::DistrustedPeer
        );
    }

    /// `SelfProtection is_a SelfStarProperty` lands on the real consensus
    /// edge `DistrustedPeer is_a Peer` — the reading is structure-
    /// compatible, not a bare collapse.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn self_protection_taxonomy_lands_on_declared_consensus_edge() {
        let source_edge = SmartElementCategory::morphisms()
            .into_iter()
            .find(|m| {
                m.source() == SmartElementConcept::SelfProtection
                    && m.target() == SmartElementConcept::SelfStarProperty
                    && m.kind() == SmartElementRelationKind::Subsumption
            })
            .expect("the SelfProtection is_a SelfStarProperty edge is declared");
        let image = SmartElementToConsensus::map_morphism(&source_edge);
        assert_eq!(image.from, ConsensusConcept::DistrustedPeer);
        assert_eq!(image.to, ConsensusConcept::Peer);
        assert_eq!(image.kind, ConsensusRelationKind::Subsumption);
        let declared = ConsensusCategory::morphisms().into_iter().any(|m| {
            m.source() == ConsensusConcept::DistrustedPeer
                && m.target() == ConsensusConcept::Peer
                && m.kind() == ConsensusRelationKind::Subsumption
        });
        assert!(
            declared,
            "DistrustedPeer is_a Peer is a real consensus edge"
        );
    }
}
