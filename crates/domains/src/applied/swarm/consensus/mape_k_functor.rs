//! Functor: Consensus → MAPE-K.
//!
//! Each peer's participation in a consensus protocol is a local
//! autonomic loop (Kephart & Chess 2003): it *senses* its neighbours'
//! state, *analyses* how far the network is from agreement and how fast
//! it can close, *plans* by the update rule, and *executes* an exchange
//! round; what it learns about identities and trust standings persists
//! as knowledge.
//!
//! # Object mapping (Kephart & Chess 2003 phases; OSFM 2007 loop reading)
//!
//! | Consensus | MAPE-K | Why |
//! |---|---|---|
//! | `Peer`, `Neighborhood`, `Topology` | `Monitor` | Sensing neighbour state over the graph |
//! | `Disagreement`, `SpectralGap` | `Analyze` | Diagnosing distance from and rate toward agreement |
//! | `ConsensusProtocol`, `AverageConsensus`, `GossipAveraging` | `Plan` | The update rule is the plan of action |
//! | `GossipRound` | `Execute` | The exchange step acts on the network |
//! | `Convergence`, `PeerIdentity`, `TrustedNeighbor`, `DistrustedPeer`, `Equivocation` | `Knowledge` | What the loop has learned and consults: the reached agreement, who is who, who is trusted, what misbehaviour looks like |
//!
//! # Morphism-kind mapping
//!
//! MAPE-K's non-taxonomic vocabulary is two kinds: the inter-phase loop
//! arrow (`HandsOffTo`) and the substrate arrow (`Consults`). Exchange
//! dynamics ride the loop arrow; every relation that reads or derives
//! knowledge rides the substrate arrow:
//!
//! - the four canonical Relations-ontology kinds map to their namesakes;
//! - `GossipsWith`, `Reduces` → `HandsOffTo` (the peer-to-peer exchange
//!   and the round's effect on the analysis measure are the loop's
//!   hand-offs);
//! - `MemberOf`, `Governs`, `IdentifiedBy`, `Triggers`,
//!   `DistrustsAfterEquivocation` → `Consults` (structure lookups,
//!   rate knowledge, identity knowledge, and trust derivations are
//!   knowledge consultations — a documented collapse onto MAPE-K's
//!   substrate arrow).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    ConsensusCategory, ConsensusConcept, ConsensusRelation, ConsensusRelationKind,
};
use crate::formal::systems::mape_k::ontology::{
    MapeKCategory, MapeKConcept, MapeKRelation, MapeKRelationKind,
};

/// Maps each consensus concept to the MAPE-K phase (or the knowledge
/// substrate) it plays in a peer's local loop.
pub struct ConsensusToMapeK;

impl Functor for ConsensusToMapeK {
    type Source = ConsensusCategory;
    type Target = MapeKCategory;

    fn map_object(obj: &ConsensusConcept) -> MapeKConcept {
        use ConsensusConcept as C;
        match obj {
            // Sensing neighbour state over the graph.
            C::Peer | C::Neighborhood | C::Topology => MapeKConcept::Monitor,
            // Diagnosing distance from agreement and the rate bound.
            C::Disagreement | C::SpectralGap => MapeKConcept::Analyze,
            // The update rule is the plan.
            C::ConsensusProtocol | C::AverageConsensus | C::GossipAveraging => MapeKConcept::Plan,
            // The exchange step acts on the network.
            C::GossipRound => MapeKConcept::Execute,
            // What the loop has learned and consults.
            C::Convergence
            | C::PeerIdentity
            | C::TrustedNeighbor
            | C::DistrustedPeer
            | C::Equivocation => MapeKConcept::Knowledge,
        }
    }

    fn map_morphism(m: &ConsensusRelation) -> MapeKRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ConsensusRelationKind::Identity => return MapeKCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            ConsensusRelationKind::Subsumption => MapeKRelationKind::Subsumption,
            ConsensusRelationKind::Parthood => MapeKRelationKind::Parthood,
            ConsensusRelationKind::Causation => MapeKRelationKind::Causation,
            ConsensusRelationKind::Opposition => MapeKRelationKind::Opposition,
            // Exchange dynamics are the loop's hand-offs.
            ConsensusRelationKind::GossipsWith | ConsensusRelationKind::Reduces => {
                MapeKRelationKind::HandsOffTo
            }
            // Everything that reads or derives knowledge consults it —
            // MAPE-K's only substrate arrow (documented collapse).
            ConsensusRelationKind::MemberOf
            | ConsensusRelationKind::Governs
            | ConsensusRelationKind::IdentifiedBy
            | ConsensusRelationKind::Triggers
            | ConsensusRelationKind::DistrustsAfterEquivocation => MapeKRelationKind::Consults,
        };
        MapeKRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ConsensusToMapeK,
    "Kephart & Chess (2003) IEEE Computer 36(1); Olfati-Saber, Fax & Murray (2007)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn consensus_to_mape_k_functor_laws() {
        assert_functor_laws::<ConsensusToMapeK>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn phases_match_documented_table() {
        use ConsensusConcept as C;
        use MapeKConcept as M;
        assert_eq!(ConsensusToMapeK::map_object(&C::Peer), M::Monitor);
        assert_eq!(ConsensusToMapeK::map_object(&C::Disagreement), M::Analyze);
        assert_eq!(ConsensusToMapeK::map_object(&C::AverageConsensus), M::Plan);
        assert_eq!(ConsensusToMapeK::map_object(&C::GossipRound), M::Execute);
        assert_eq!(ConsensusToMapeK::map_object(&C::Equivocation), M::Knowledge);
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn trust_concepts_are_knowledge() {
        use ConsensusConcept as C;
        for c in [
            C::PeerIdentity,
            C::TrustedNeighbor,
            C::DistrustedPeer,
            C::Equivocation,
            C::Convergence,
        ] {
            assert_eq!(
                ConsensusToMapeK::map_object(&c),
                MapeKConcept::Knowledge,
                "{c:?} should live in the knowledge substrate"
            );
        }
    }
}
