//! Functor: Consensus → Dependability.
//!
//! The dependability reading of a consensus swarm (Avizienis, Laprie,
//! Randell & Landwehr 2004): the peers deliver an estimation *service*;
//! equivocation is the *Byzantine fault* of Lamport, Shostak & Pease
//! (1982) — inconsistent reports to different observers — and the
//! exclusion of a caught equivocator is *fault handling* (diagnosis,
//! isolation, reconfiguration).
//!
//! # Object mapping (each collapse documented)
//!
//! | Consensus | Dependability | Why |
//! |---|---|---|
//! | `Equivocation` | `ByzantineFault` | LSP (1982): inconsistent reports to different observers — Avizienis's own gloss of the Byzantine fault model |
//! | `DistrustedPeer` | `FaultHandling` | The excluded peer is the located, isolated fault (Avizienis §5.2: diagnosis, isolation, reconfiguration) |
//! | `TrustedNeighbor` | `CorrectService` | A consistent peer delivers correct service at the service boundary |
//! | `Convergence` | `CorrectService` | Reached agreement is the service delivered correctly |
//! | `Peer`, `Neighborhood`, `Topology` | `Service` | The agents and their communication structure are the service-delivery surface (umbrella collapse) |
//! | `ConsensusProtocol`, `AverageConsensus`, `GossipAveraging`, `GossipRound` | `FaultTolerance` | Redundant aggregation across peers is the means of delivering correct service despite individual faults (Avizienis §5.2) |
//! | `Disagreement` | `Error` | Deviation from agreement is erroneous system state that may propagate to failure (Avizienis §2.2) |
//! | `SpectralGap` | `Reliability` | `lambda_2` quantifies the continuity of convergence toward correct service — the reliability attribute (documented reading) |
//! | `PeerIdentity` | `Integrity` | The signing identity is what prevents improper alteration/impersonation of a peer's claims (documented reading) |
//!
//! # Morphism-kind mapping
//!
//! Dependability's cross-concept vocabulary is the causal
//! fault→error→failure chain (plus the canonical structural kinds), so:
//! the four canonical Relations-ontology kinds map to their namesakes;
//! every consensus action or derivation kind (`GossipsWith`, `MemberOf`,
//! `Reduces`, `Governs`, `IdentifiedBy`, `Triggers`,
//! `DistrustsAfterEquivocation`) → `Causation` — most faithfully
//! `Triggers`, which lands exactly on the Avizienis activation arrow
//! `ByzantineFault → FaultHandling`.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    ConsensusCategory, ConsensusConcept, ConsensusRelation, ConsensusRelationKind,
};
use crate::applied::dependability::ontology::{
    DependabilityCategory, DependabilityConcept, DependabilityRelation, DependabilityRelationKind,
};

/// Maps each consensus concept to its Avizienis-taxonomy role.
pub struct ConsensusToDependability;

impl Functor for ConsensusToDependability {
    type Source = ConsensusCategory;
    type Target = DependabilityCategory;

    fn map_object(obj: &ConsensusConcept) -> DependabilityConcept {
        use ConsensusConcept as C;
        use DependabilityConcept as D;
        match obj {
            // LSP (1982): inconsistent reports to different observers.
            C::Equivocation => D::ByzantineFault,
            // The excluded peer is the located, isolated fault.
            C::DistrustedPeer => D::FaultHandling,
            // Consistent peers and reached agreement deliver correctly.
            C::TrustedNeighbor | C::Convergence => D::CorrectService,
            // The agents and their communication structure are the
            // service-delivery surface (umbrella collapse).
            C::Peer | C::Neighborhood | C::Topology => D::Service,
            // Redundant aggregation is the fault-tolerance means.
            C::ConsensusProtocol | C::AverageConsensus | C::GossipAveraging | C::GossipRound => {
                D::FaultTolerance
            }
            // Deviation from agreement is erroneous state (Avizienis §2.2).
            C::Disagreement => D::Error,
            // lambda_2 quantifies continuity toward correct service.
            C::SpectralGap => D::Reliability,
            // The signing identity guards against improper alteration.
            C::PeerIdentity => D::Integrity,
        }
    }

    fn map_morphism(m: &ConsensusRelation) -> DependabilityRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ConsensusRelationKind::Identity => return DependabilityCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            ConsensusRelationKind::Subsumption => DependabilityRelationKind::Subsumption,
            ConsensusRelationKind::Parthood => DependabilityRelationKind::Parthood,
            ConsensusRelationKind::Causation => DependabilityRelationKind::Causation,
            ConsensusRelationKind::Opposition => DependabilityRelationKind::Opposition,
            // Every consensus action/derivation reads as a causal arrow
            // in the fault→error→failure vocabulary; Triggers lands
            // exactly on the activation arrow ByzantineFault→FaultHandling.
            ConsensusRelationKind::GossipsWith
            | ConsensusRelationKind::MemberOf
            | ConsensusRelationKind::Reduces
            | ConsensusRelationKind::Governs
            | ConsensusRelationKind::IdentifiedBy
            | ConsensusRelationKind::Triggers
            | ConsensusRelationKind::DistrustsAfterEquivocation => {
                DependabilityRelationKind::Causation
            }
        };
        DependabilityRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ConsensusToDependability,
    "Avizienis, Laprie, Randell & Landwehr (2004) IEEE TDSC 1(1); Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn consensus_to_dependability_functor_laws() {
        assert_functor_laws::<ConsensusToDependability>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn equivocation_is_the_byzantine_fault() {
        assert_eq!(
            ConsensusToDependability::map_object(&ConsensusConcept::Equivocation),
            DependabilityConcept::ByzantineFault
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn exclusion_is_fault_handling_and_trust_is_correct_service() {
        assert_eq!(
            ConsensusToDependability::map_object(&ConsensusConcept::DistrustedPeer),
            DependabilityConcept::FaultHandling
        );
        assert_eq!(
            ConsensusToDependability::map_object(&ConsensusConcept::TrustedNeighbor),
            DependabilityConcept::CorrectService
        );
    }

    /// The image of the trust taxonomy edge `TrustedNeighbor is_a Peer`
    /// is `CorrectService is_a Service` — an edge dependability itself
    /// declares (Avizienis §2.1), so the reading is structure-compatible.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn trusted_neighbor_taxonomy_lands_on_declared_dependability_edge() {
        use pr4xis::category::{Arrow, Category};
        let image_exists = DependabilityCategory::morphisms().iter().any(|m| {
            m.source() == DependabilityConcept::CorrectService
                && m.target() == DependabilityConcept::Service
                && m.kind() == DependabilityRelationKind::Subsumption
        });
        assert!(image_exists);
    }
}
