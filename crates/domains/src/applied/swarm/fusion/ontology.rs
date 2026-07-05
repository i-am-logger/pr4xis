//! DistributedFusion — fusing state estimates across a network of
//! peers.
//!
//! The estimation mathematics here is established literature — this
//! ontology encodes it, it does not extend it:
//!
//! - **Olfati-Saber (2005)** 44th IEEE CDC *Distributed Kalman Filter
//!   with Embedded Consensus Filters* and **Olfati-Saber (2007)** 46th
//!   IEEE CDC *Distributed Kalman Filtering for Sensor Networks* —
//!   local Kalman filters plus consensus on information contributions.
//! - **Julier & Uhlmann (1997)** American Control Conference *A
//!   Non-divergent Estimation Algorithm in the Presence of Unknown
//!   Correlations* — covariance intersection.
//! - **Bar-Shalom (1981)** *On the Track-to-Track Correlation Problem*,
//!   IEEE TAC 26(2), and **Mutambara (1998)** *Decentralized Estimation
//!   and Control for Multisensor Systems* — the correlation problem and
//!   the data-incest failure of naive additive re-fusion.
//!
//! The four domain axioms are discharged against the fixtures in
//! [`super::engine`], which wrap the existing sensor-fusion
//! `InformationEstimate` (its `fuse()` is every additive step) and the
//! sibling `swarm::consensus` engine (its step is the consensus run).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::engine::{
    CENTRAL_AGREEMENT_TOLERANCE, CI_VARIANCE_A, CI_VARIANCE_B, CONSENSUS_FUSION_ROUNDS,
    CORRELATION_GRID, FIXTURE_EPOCH, NUMERICAL_SLACK, OMEGA_GRID, centralized_information_fusion,
    ci_fused_scalar_variance, ci_realised_error_variance, consensus_on_information,
    fixture_fused_covariances, naive_ring_refusion, ring_local_estimates, ring_topology,
};
use crate::applied::sensor_fusion::state::covariance;
use crate::formal::math::linear_algebra::matrix::approx_eq;
use crate::formal::math::linear_algebra::positive_definite;

pr4xis::ontology! {
    name: "DistributedFusion",
    source: "Olfati-Saber (2007) 46th IEEE CDC; Julier & Uhlmann (1997) American Control Conference; Bar-Shalom (1981) IEEE TAC 26(2); Mutambara (1998) Decentralized Estimation and Control for Multisensor Systems",

    concepts: [
        NetworkFusionArchitecture,
        DistributedKalmanFilter,
        CiOverNetwork,
        ConsensusEstimate,
        InnovationExchange,
        DataIncest,
    ],

    labels: {
        NetworkFusionArchitecture: ("en", "Network fusion architecture", "Mutambara (1998) Ch. 1: abstract - how estimates are fused across a network of peers."),
        DistributedKalmanFilter: ("en", "Distributed Kalman filter", "Local Kalman filters plus consensus on information contributions - Olfati-Saber (2005) 44th IEEE CDC 'Distributed Kalman Filter with Embedded Consensus Filters'; Olfati-Saber (2007) 46th IEEE CDC 'Distributed Kalman Filtering for Sensor Networks'."),
        CiOverNetwork: ("en", "CI over network", "Network-wide covariance intersection: consistent fusion without cross-correlation knowledge - Julier & Uhlmann (1997) 'A Non-divergent Estimation Algorithm in the Presence of Unknown Correlations', American Control Conference."),
        ConsensusEstimate: ("en", "Consensus estimate", "Olfati-Saber (2007) 46th IEEE CDC: a state estimate agreed across peers - the limit of the information-consensus iteration."),
        InnovationExchange: ("en", "Innovation exchange", "Olfati-Saber (2007) 46th IEEE CDC, consensus filters: peers exchange measurement innovations / information contributions."),
        DataIncest: ("en", "Data incest", "Double-counting of shared information when the topology has cycles - Bar-Shalom (1981) 'On the Track-to-Track Correlation Problem' IEEE TAC 26(2); Mutambara (1998)."),
    },

    is_a: [
        (DistributedKalmanFilter, NetworkFusionArchitecture),
        (CiOverNetwork, NetworkFusionArchitecture),
    ],

    edges: [
        // Olfati-Saber (2007): exchanging contributions yields agreement.
        (InnovationExchange, ConsensusEstimate, Produces),
        // Bar-Shalom (1981): the double count biases what peers agree on.
        (DataIncest, ConsensusEstimate, Corrupts),
        // Julier & Uhlmann (1997): CI's conservatism blocks the incest.
        (CiOverNetwork, DataIncest, Prevents),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether a network fusion architecture stays consistent (never
/// over-confident) when the peers' estimates are correlated with each
/// other — Julier & Uhlmann (1997): covariance intersection is
/// non-divergent for every admissible cross-correlation; Mutambara
/// (1998): naive additive information fusion assumes independence and
/// is over-confident when it fails. `None` for concepts that are not
/// concrete architectures (including the abstract parent).
#[derive(Debug, Clone)]
pub struct ConsistentUnderInterPeerCorrelation;

impl Quality for ConsistentUnderInterPeerCorrelation {
    type Individual = DistributedFusionConcept;
    type Value = bool;

    fn get(&self, c: &DistributedFusionConcept) -> Option<bool> {
        use DistributedFusionConcept as D;
        match c {
            D::CiOverNetwork => Some(true),
            D::DistributedKalmanFilter => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn kinded_edge_exists(
    from: DistributedFusionConcept,
    to: DistributedFusionConcept,
    kind: DistributedFusionRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    DistributedFusionCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

fn verdict_from(axiom: &dyn Axiom, ok: bool) -> pr4xis::logic::proof::Verdict {
    use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
    if ok {
        Ok(Box::new(SimpleProof::new(axiom.meta())))
    } else {
        Err(Box::new(SimpleCounterexample::new(axiom.meta())))
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Julier & Uhlmann (1997): CI of two estimates with unknown
/// correlation remains conservative — for every omega in the cited grid
/// and every admissible cross-correlation, the fused variance is at
/// least the realised error variance (1D ordering, which is the PSD
/// ordering in one dimension).
pub struct CiFusionConsistentAcrossPeers;

impl Axiom for CiFusionConsistentAcrossPeers {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let mut consistent = true;
        let mut strict_somewhere = false;
        for omega in OMEGA_GRID {
            let Some(fused) = ci_fused_scalar_variance(omega) else {
                return verdict_from(self, false);
            };
            for rho in CORRELATION_GRID {
                let realised =
                    ci_realised_error_variance(CI_VARIANCE_A, CI_VARIANCE_B, rho, omega, fused);
                consistent &= fused + NUMERICAL_SLACK >= realised;
                strict_somewhere |= fused > realised + NUMERICAL_SLACK;
            }
        }
        // The category carries the matching Prevents edge.
        let prevents = kinded_edge_exists(
            DistributedFusionConcept::CiOverNetwork,
            DistributedFusionConcept::DataIncest,
            DistributedFusionRelationKind::Prevents,
        );
        verdict_from(self, consistent && strict_somewhere && prevents)
    }

    pr4xis::axiom_meta!(
        "CiFusionConsistentAcrossPeers",
        "for every omega in the grid and every cross-correlation in [-1, 1], the CI-fused variance is at least the realised error variance",
        "Julier & Uhlmann (1997) American Control Conference"
    );
}
pr4xis::register_axiom!(
    CiFusionConsistentAcrossPeers,
    "Julier & Uhlmann (1997) American Control Conference"
);

/// HONEST NEGATIVE AXIOM — Bar-Shalom (1981); Mutambara (1998) Ch. 3:
/// additive information fusion around the 3-peer cycle re-counts shared
/// information and yields a covariance strictly smaller than the correct
/// one. The demonstrated mechanism: the naive information matrix exceeds
/// the centralized one by exactly the origin peer's own contribution.
pub struct NaiveInformationFusionOverconfidentUnderCycles;

impl Axiom for NaiveInformationFusionOverconfidentUnderCycles {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let Some(locals) = ring_local_estimates() else {
            return verdict_from(self, false);
        };
        let Some(central) = centralized_information_fusion(&locals) else {
            return verdict_from(self, false);
        };
        let Some(naive) = naive_ring_refusion(&locals) else {
            return verdict_from(self, false);
        };
        let Some(central_estimate) = central.to_estimate(FIXTURE_EPOCH) else {
            return verdict_from(self, false);
        };
        let Some(naive_estimate) = naive.to_estimate(FIXTURE_EPOCH) else {
            return verdict_from(self, false);
        };

        // Overconfidence: the correct covariance strictly dominates the
        // naive one in the PSD ordering (difference PSD, trace positive).
        let difference = central_estimate.covariance.sub(&naive_estimate.covariance);
        let strictly_smaller =
            positive_definite::is_positive_semidefinite(&difference) && difference.trace() > 0.0;

        // The mechanism, verified exactly: the surplus information is
        // the origin peer's own contribution, counted a second time.
        let surplus = naive.information_matrix.sub(&central.information_matrix);
        let is_first_peer_contribution =
            approx_eq(&surplus, &locals[0].information_matrix, NUMERICAL_SLACK);

        // The category carries the matching Corrupts edge.
        let corrupts = kinded_edge_exists(
            DistributedFusionConcept::DataIncest,
            DistributedFusionConcept::ConsensusEstimate,
            DistributedFusionRelationKind::Corrupts,
        );
        verdict_from(
            self,
            strictly_smaller && is_first_peer_contribution && corrupts,
        )
    }

    pr4xis::axiom_meta!(
        "NaiveInformationFusionOverconfidentUnderCycles",
        "additive re-fusion around the 3-peer cycle double-counts the origin peer's information and yields a covariance strictly smaller than the correct one",
        "Bar-Shalom (1981) IEEE TAC 26(2); Mutambara (1998) Ch. 3"
    );
}
pr4xis::register_axiom!(
    NaiveInformationFusionOverconfidentUnderCycles,
    "Bar-Shalom (1981) IEEE TAC 26(2); Mutambara (1998) Ch. 3"
);

/// Olfati-Saber (2007) 46th IEEE CDC: as consensus iterations grow on a
/// connected fixture, every peer's rescaled information-consensus
/// estimate matches the centralized information-filter fusion within the
/// documented tolerance.
pub struct DistributedFilterAgreesWithCentralInTheLimit;

impl Axiom for DistributedFilterAgreesWithCentralInTheLimit {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let Some(locals) = ring_local_estimates() else {
            return verdict_from(self, false);
        };
        let Some(central) = centralized_information_fusion(&locals) else {
            return verdict_from(self, false);
        };
        let Some(per_peer) =
            consensus_on_information(&locals, &ring_topology(), CONSENSUS_FUSION_ROUNDS)
        else {
            return verdict_from(self, false);
        };
        let agrees = !per_peer.is_empty()
            && per_peer.iter().all(|peer| {
                let matrices_close = approx_eq(
                    &peer.information_matrix,
                    &central.information_matrix,
                    CENTRAL_AGREEMENT_TOLERANCE,
                );
                let vectors_close = (0..peer.dim()).all(|i| {
                    (peer.information_vector.get(i) - central.information_vector.get(i)).abs()
                        <= CENTRAL_AGREEMENT_TOLERANCE
                });
                matrices_close && vectors_close
            });
        // The category carries the matching Produces edge.
        let produces = kinded_edge_exists(
            DistributedFusionConcept::InnovationExchange,
            DistributedFusionConcept::ConsensusEstimate,
            DistributedFusionRelationKind::Produces,
        );
        verdict_from(self, agrees && produces)
    }

    pr4xis::axiom_meta!(
        "DistributedFilterAgreesWithCentralInTheLimit",
        "each peer's rescaled information-consensus estimate matches the centralized information-filter fuse within the documented tolerance",
        "Olfati-Saber (2007) 46th IEEE CDC"
    );
}
pr4xis::register_axiom!(
    DistributedFilterAgreesWithCentralInTheLimit,
    "Olfati-Saber (2007) 46th IEEE CDC"
);

/// Bar-Shalom, Li & Kirubarajan (2001): a covariance is a covariance —
/// every fused covariance the fixtures produce stays symmetric positive
/// semidefinite, checked with the existing sensor-fusion validity
/// predicate (code reuse, not a new PSD check).
pub struct FusedCovarianceRemainsPsd;

impl Axiom for FusedCovarianceRemainsPsd {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let Some(covariances) = fixture_fused_covariances() else {
            return verdict_from(self, false);
        };
        let all_valid = !covariances.is_empty() && covariances.iter().all(covariance::is_valid);
        verdict_from(self, all_valid)
    }

    pr4xis::axiom_meta!(
        "FusedCovarianceRemainsPsd",
        "every fused covariance in the fixtures (CI grid, centralized, naive ring, consensus limits) is symmetric positive semidefinite",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation"
    );
}
pr4xis::register_axiom!(
    FusedCovarianceRemainsPsd,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for DistributedFusionOntology {
    type Cat = DistributedFusionCategory;
    type Qual = ConsistentUnderInterPeerCorrelation;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CiFusionConsistentAcrossPeers));
        axioms.push(Box::new(NaiveInformationFusionOverconfidentUnderCycles));
        axioms.push(Box::new(DistributedFilterAgreesWithCentralInTheLimit));
        axioms.push(Box::new(FusedCovarianceRemainsPsd));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<DistributedFusionCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        DistributedFusionOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ci_fusion_consistent_across_peers_holds() {
        assert!(CiFusionConsistentAcrossPeers.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn naive_information_fusion_overconfident_under_cycles_holds() {
        assert!(
            NaiveInformationFusionOverconfidentUnderCycles
                .verify()
                .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn distributed_filter_agrees_with_central_in_the_limit_holds() {
        assert!(
            DistributedFilterAgreesWithCentralInTheLimit
                .verify()
                .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fused_covariance_remains_psd_holds() {
        assert!(FusedCovarianceRemainsPsd.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn correlation_consistency_classification() {
        let q = ConsistentUnderInterPeerCorrelation;
        assert_eq!(
            q.get(&DistributedFusionConcept::CiOverNetwork),
            Some(true),
            "CI is non-divergent under unknown correlation (Julier & Uhlmann 1997)"
        );
        assert_eq!(
            q.get(&DistributedFusionConcept::DistributedKalmanFilter),
            Some(false),
            "naive additive information fusion is over-confident when peer estimates are correlated (Mutambara 1998)"
        );
        assert_eq!(
            q.get(&DistributedFusionConcept::NetworkFusionArchitecture),
            None,
            "the abstract parent fixes no correlation discipline"
        );
        assert_eq!(q.get(&DistributedFusionConcept::DataIncest), None);
    }
}
