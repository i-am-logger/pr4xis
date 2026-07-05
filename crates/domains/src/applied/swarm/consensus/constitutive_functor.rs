//! Functor: Consensus → ConstitutiveProtocol.
//!
//! The constitutive reading of a consensus swarm: peers act under
//! signing identities (Lamport 1979 — to be able to sign is what an
//! identity is), same-round inconsistent claims are the protocol's own
//! `Equivocation` (Li et al. 2004 SUNDR fork-consistency), and the
//! exclusion of a caught equivocator is `Slashing` — the structural
//! penalty whose proof is a constructable artifact (Buterin & Griffith
//! 2017).
//!
//! # Object mapping (each collapse documented)
//!
//! | Consensus | ConstitutiveProtocol | Why |
//! |---|---|---|
//! | `PeerIdentity` | `Identity` | Lamport (1979): the signing identity, concept for concept |
//! | `Peer` | `Identity` | Documented collapse: the protocol sees a peer only through the identity it signs under |
//! | `Equivocation` | `Equivocation` | Same concept, same sources (LSP 1982; SUNDR) |
//! | `DistrustedPeer` | `Slashing` | Exclusion after constructable proof is the slashing penalty (Buterin & Griffith 2017) |
//! | `TrustedNeighbor` | `Membership` | A trusted neighbour is a peer in good standing of the exchange relation — membership |
//! | `Neighborhood`, `Topology` | `Membership` | The communication structure is, constitutively, who is a member of whose exchange set (documented collapse) |
//! | `GossipRound`, `ConsensusProtocol`, `AverageConsensus`, `GossipAveraging` | `PraxisEvent` | The protocol's activity is lived praxis under the constitution (umbrella collapse) |
//! | `Disagreement`, `Convergence`, `SpectralGap` | `PraxisEvent` | Estimation-level quantities have no constitutive image; they collapse forgetfully onto the activity umbrella (documented) |
//!
//! # Morphism-kind mapping
//!
//! `Subsumption` → `Subsumption`, `Parthood` → `Parthood`;
//! `GossipsWith` → `Authors` (an exchange is each identity authoring
//! signed events the other verifies); `MemberOf` → `Establishes` (the
//! membership relation is established by the join declaration);
//! `Reduces` → `OrderedInto` (successive rounds are ordered into the
//! channel's chains); `Governs` → `Gates` (what governs the dynamics
//! reads as what gates the praxis); `IdentifiedBy` → `NamedBy` (the
//! protocol's external-naming arrow); `Triggers` → `Triggers` (the
//! slashing chain's own kind); `DistrustsAfterEquivocation` →
//! `ExcludesFrom` (slashing excludes from future praxis).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    ConsensusCategory, ConsensusConcept, ConsensusRelation, ConsensusRelationKind,
};
use crate::social::protocols::constitutive::ontology::{
    ConstitutiveProtocolCategory, ConstitutiveProtocolConcept, ConstitutiveProtocolRelation,
    ConstitutiveProtocolRelationKind,
};

/// Maps each consensus concept to its constitutive-protocol reading.
pub struct ConsensusToConstitutiveProtocol;

impl Functor for ConsensusToConstitutiveProtocol {
    type Source = ConsensusCategory;
    type Target = ConstitutiveProtocolCategory;

    fn map_object(obj: &ConsensusConcept) -> ConstitutiveProtocolConcept {
        use ConsensusConcept as C;
        use ConstitutiveProtocolConcept as P;
        match obj {
            // Lamport (1979): the signing identity, concept for concept;
            // the protocol sees a peer only through its identity.
            C::PeerIdentity | C::Peer => P::Identity,
            // Same concept, same sources (LSP 1982; SUNDR).
            C::Equivocation => P::Equivocation,
            // Exclusion after constructable proof is the slashing penalty.
            C::DistrustedPeer => P::Slashing,
            // Good standing in the exchange relation is membership; the
            // communication structure is who is a member of whose set.
            C::TrustedNeighbor | C::Neighborhood | C::Topology => P::Membership,
            // The protocol's activity — and its estimation-level
            // quantities, which have no constitutive image — collapse
            // onto the lived-praxis umbrella (documented).
            C::GossipRound
            | C::ConsensusProtocol
            | C::AverageConsensus
            | C::GossipAveraging
            | C::Disagreement
            | C::Convergence
            | C::SpectralGap => P::PraxisEvent,
        }
    }

    fn map_morphism(m: &ConsensusRelation) -> ConstitutiveProtocolRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ConsensusRelationKind::Identity => {
                return ConstitutiveProtocolCategory::identity(&from);
            }
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            ConsensusRelationKind::Subsumption => ConstitutiveProtocolRelationKind::Subsumption,
            ConsensusRelationKind::Parthood => ConstitutiveProtocolRelationKind::Parthood,
            ConsensusRelationKind::Causation => ConstitutiveProtocolRelationKind::Causation,
            ConsensusRelationKind::Opposition => ConstitutiveProtocolRelationKind::Opposition,
            // An exchange is each identity authoring signed events.
            ConsensusRelationKind::GossipsWith => ConstitutiveProtocolRelationKind::Authors,
            // Membership is established by the join declaration.
            ConsensusRelationKind::MemberOf => ConstitutiveProtocolRelationKind::Establishes,
            // Successive rounds are ordered into the channel's chains.
            ConsensusRelationKind::Reduces => ConstitutiveProtocolRelationKind::OrderedInto,
            // What governs the dynamics gates the praxis.
            ConsensusRelationKind::Governs => ConstitutiveProtocolRelationKind::Gates,
            // The protocol's external-naming arrow.
            ConsensusRelationKind::IdentifiedBy => ConstitutiveProtocolRelationKind::NamedBy,
            // The slashing chain's own kind.
            ConsensusRelationKind::Triggers => ConstitutiveProtocolRelationKind::Triggers,
            // Slashing excludes from future praxis.
            ConsensusRelationKind::DistrustsAfterEquivocation => {
                ConstitutiveProtocolRelationKind::ExcludesFrom
            }
        };
        ConstitutiveProtocolRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ConsensusToConstitutiveProtocol,
    "Lamport (1979) SRI CSL-98; Buterin & Griffith (2017) arXiv:1710.09437; Li et al. (2004) OSDI"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn consensus_to_constitutive_functor_laws() {
        assert_functor_laws::<ConsensusToConstitutiveProtocol>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn trust_wiring_lands_on_the_slashing_cluster() {
        use ConsensusConcept as C;
        use ConstitutiveProtocolConcept as P;
        assert_eq!(
            ConsensusToConstitutiveProtocol::map_object(&C::Equivocation),
            P::Equivocation
        );
        assert_eq!(
            ConsensusToConstitutiveProtocol::map_object(&C::DistrustedPeer),
            P::Slashing
        );
        assert_eq!(
            ConsensusToConstitutiveProtocol::map_object(&C::TrustedNeighbor),
            P::Membership
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn peers_are_identities() {
        use ConsensusConcept as C;
        use ConstitutiveProtocolConcept as P;
        assert_eq!(
            ConsensusToConstitutiveProtocol::map_object(&C::PeerIdentity),
            P::Identity
        );
        // Documented collapse: the protocol sees a peer only through the
        // identity it signs under.
        assert_eq!(
            ConsensusToConstitutiveProtocol::map_object(&C::Peer),
            P::Identity
        );
    }

    /// The consensus `Triggers` edge (`Equivocation → DistrustedPeer`)
    /// maps onto the constitutive slashing chain's own `Triggers` kind,
    /// landing on `Equivocation → Slashing` — the same shape the target
    /// declares via `ForkProof` (Equivocation `ProvenBy` ForkProof
    /// `Triggers` Slashing).
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn equivocation_triggers_slashing_in_the_image() {
        use pr4xis::category::{Arrow, Category};
        let source_edge = ConsensusCategory::morphisms()
            .into_iter()
            .find(|m| {
                m.source() == ConsensusConcept::Equivocation
                    && m.target() == ConsensusConcept::DistrustedPeer
                    && m.kind() == ConsensusRelationKind::Triggers
            })
            .expect("the Triggers edge is declared");
        let image = ConsensusToConstitutiveProtocol::map_morphism(&source_edge);
        assert_eq!(image.from, ConstitutiveProtocolConcept::Equivocation);
        assert_eq!(image.to, ConstitutiveProtocolConcept::Slashing);
        assert_eq!(image.kind, ConstitutiveProtocolRelationKind::Triggers);
    }
}
