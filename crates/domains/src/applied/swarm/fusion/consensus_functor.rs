//! Functor: DistributedFusion → Consensus.
//!
//! Distributed fusion IS consensus applied to information contributions
//! (Olfati-Saber 2007, 46th IEEE CDC): the fusion architectures are
//! consensus protocols, the innovation exchange is the gossip round,
//! and the agreed estimate is the convergence.
//!
//! # Object mapping (documented readings)
//!
//! | DistributedFusion | Consensus | Why |
//! |---|---|---|
//! | `DistributedKalmanFilter` | `ConsensusProtocol` | Its update rule is a consensus protocol on information contributions (Olfati-Saber 2007) |
//! | `CiOverNetwork` | `ConsensusProtocol` | Network-wide CI is likewise an agreement-driving update rule |
//! | `NetworkFusionArchitecture` | `ConsensusProtocol` | The abstract architecture collapses onto the abstract protocol |
//! | `InnovationExchange` | `GossipRound` | One exchange of contributions is one gossip step |
//! | `ConsensusEstimate` | `Convergence` | The agreed estimate is the reached agreement |
//! | `DataIncest` | `Disagreement` | Documented reading: the double count is a corruption of the agreement state — the deviation-from-truth the disagreement vocabulary names |
//!
//! # Morphism-kind mapping (documented collapses)
//!
//! `Subsumption` → `Subsumption`; `Produces` → `Reduces` (the exchange
//! produces agreement exactly by reducing disagreement — OSFM 2007 sec
//! III); `Corrupts` → `Governs` (what corrupts the agreed estimate
//! governs — biases — what the network converges to, Bar-Shalom 1981);
//! `Prevents` → `Reduces` (CI prevents the incest by keeping the
//! double-counted term out — a reduction of the corrupting
//! disagreement).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    DistributedFusionCategory, DistributedFusionConcept, DistributedFusionRelation,
    DistributedFusionRelationKind,
};
use crate::applied::swarm::consensus::ontology::{
    ConsensusCategory, ConsensusConcept, ConsensusRelation, ConsensusRelationKind,
};

/// Maps each distributed-fusion concept to the consensus concept it
/// instantiates.
pub struct DistributedFusionToConsensus;

impl Functor for DistributedFusionToConsensus {
    type Source = DistributedFusionCategory;
    type Target = ConsensusCategory;

    fn map_object(obj: &DistributedFusionConcept) -> ConsensusConcept {
        use ConsensusConcept as C;
        use DistributedFusionConcept as D;
        match obj {
            // Fusion architectures are consensus protocols on
            // information contributions (Olfati-Saber 2007).
            D::DistributedKalmanFilter | D::CiOverNetwork | D::NetworkFusionArchitecture => {
                C::ConsensusProtocol
            }
            // One exchange of contributions is one gossip step.
            D::InnovationExchange => C::GossipRound,
            // The agreed estimate is the reached agreement.
            D::ConsensusEstimate => C::Convergence,
            // Documented reading: the double count corrupts the
            // agreement state — the disagreement vocabulary names it.
            D::DataIncest => C::Disagreement,
        }
    }

    fn map_morphism(m: &DistributedFusionRelation) -> ConsensusRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            DistributedFusionRelationKind::Identity => {
                return ConsensusCategory::identity(&from);
            }
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            DistributedFusionRelationKind::Subsumption => ConsensusRelationKind::Subsumption,
            DistributedFusionRelationKind::Parthood => ConsensusRelationKind::Parthood,
            DistributedFusionRelationKind::Causation => ConsensusRelationKind::Causation,
            DistributedFusionRelationKind::Opposition => ConsensusRelationKind::Opposition,
            // Producing agreement is reducing disagreement (OSFM 2007
            // sec III); preventing the incest keeps the corrupting term
            // out — likewise a reduction (documented collapses).
            DistributedFusionRelationKind::Produces | DistributedFusionRelationKind::Prevents => {
                ConsensusRelationKind::Reduces
            }
            // What corrupts the agreed estimate governs — biases — what
            // the network converges to (Bar-Shalom 1981).
            DistributedFusionRelationKind::Corrupts => ConsensusRelationKind::Governs,
        };
        ConsensusRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    DistributedFusionToConsensus,
    "Olfati-Saber (2007) 46th IEEE CDC; Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn distributed_fusion_to_consensus_functor_laws() {
        assert_functor_laws::<DistributedFusionToConsensus>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn architectures_are_protocols() {
        use ConsensusConcept as C;
        use DistributedFusionConcept as D;
        for concept in [
            D::DistributedKalmanFilter,
            D::CiOverNetwork,
            D::NetworkFusionArchitecture,
        ] {
            assert_eq!(
                DistributedFusionToConsensus::map_object(&concept),
                C::ConsensusProtocol,
                "{concept:?} should map to the consensus protocol"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn exchange_estimate_and_incest_land_on_their_readings() {
        use ConsensusConcept as C;
        use DistributedFusionConcept as D;
        assert_eq!(
            DistributedFusionToConsensus::map_object(&D::InnovationExchange),
            C::GossipRound
        );
        assert_eq!(
            DistributedFusionToConsensus::map_object(&D::ConsensusEstimate),
            C::Convergence
        );
        assert_eq!(
            DistributedFusionToConsensus::map_object(&D::DataIncest),
            C::Disagreement
        );
    }
}
