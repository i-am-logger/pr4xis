//! Consensus — distributed agreement over a peer graph, with the trust
//! wiring that excludes protocol-detectable misbehaviour.
//!
//! The estimation mathematics here is established literature — this
//! ontology encodes it, it does not extend it:
//!
//! - **Olfati-Saber, Fax & Murray (2007)** *Consensus and Cooperation in
//!   Networked Multi-Agent Systems*, Proc. IEEE 95(1):215-233 — peers,
//!   neighbourhoods, the graph-Laplacian consensus dynamics, the
//!   disagreement Lyapunov function, and the step-size stability bound.
//! - **Xiao & Boyd (2004)** *Fast linear iterations for distributed
//!   averaging*, Systems & Control Letters 53(1):65-78 — the spectral
//!   view of distributed averaging.
//! - **Kempe, Dobra & Gehrke (2003)** FOCS *Gossip-Based Computation of
//!   Aggregate Information* — push-sum gossip and its mass conservation.
//! - **Fiedler (1973)** *Algebraic connectivity of graphs*, Czechoslovak
//!   Mathematical Journal 23(2):298-305 — `lambda_2` and the spectral
//!   characterization of connectivity.
//! - **Lamport, Shostak & Pease (1982)** ACM TOPLAS 4(3); **Li, Krohn,
//!   Mazieres & Shasha (2004)** OSDI (SUNDR) — equivocation as the
//!   protocol-detectable misbehaviour, and exclusion as its consequence.
//!
//! The five domain axioms are discharged against the small verified
//! fixtures in [`super::engine`]: the path graph `P3`, a disconnected
//! `2+1` graph, a deterministic push-sum schedule, and an equivocating
//! middle peer.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::engine::{
    ALGEBRAIC_CONNECTIVITY_DISCONNECTED, ALGEBRAIC_CONNECTIVITY_P3, CONSENSUS_ROUNDS,
    DISAGREEMENT_TOLERANCE, DISCONNECTED_DISAGREEMENT_FLOOR, FIXTURE_INITIAL_AVERAGE,
    FIXTURE_INITIAL_VALUES, NUMERICAL_SLACK, PUSH_SUM_ROUNDS, PeerTrust, SwarmConsensusRun,
    apply_exchange_round, average_consensus_step, disagreement, disconnected_two_plus_one,
    equivocation_round_p3, equivocators, honest_round, path_graph_p3, push_sum_initial,
    push_sum_round, ratio_invariant, stable_step_size,
};

pr4xis::ontology! {
    name: "Consensus",
    source: "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1); Xiao & Boyd (2004) Systems & Control Letters 53(1); Kempe, Dobra & Gehrke (2003) FOCS; Fiedler (1973) Czechoslovak Mathematical Journal 23(2)",

    concepts: [
        // === The networked multi-agent system (OSFM 2007) ===
        Peer,
        Neighborhood,
        Topology,

        // === Protocols and their steps (OSFM 2007; Kempe et al. 2003) ===
        GossipRound,
        ConsensusProtocol,
        AverageConsensus,
        GossipAveraging,

        // === Convergence analysis (OSFM 2007; Fiedler 1973) ===
        Disagreement,
        Convergence,
        SpectralGap,

        // === Trust wiring (Lamport 1979; LSP 1982; SUNDR) ===
        PeerIdentity,
        TrustedNeighbor,
        DistrustedPeer,
        Equivocation,
    ],

    labels: {
        Peer: ("en", "Peer", "Olfati-Saber, Fax & Murray (2007) 'Consensus and Cooperation in Networked Multi-Agent Systems' Proc. IEEE 95(1):215-233: an agent holding a local value and exchanging it with its neighbours."),
        Neighborhood: ("en", "Neighborhood", "OSFM (2007), graph-Laplacian model: the set of peers a peer communicates with - N_i = { j : (i, j) in E }."),
        Topology: ("en", "Topology", "OSFM (2007): the communication graph G = (V, E) over the peers, whose Laplacian defines the consensus dynamics."),
        GossipRound: ("en", "Gossip round", "Kempe, Dobra & Gehrke (2003) FOCS 'Gossip-Based Computation of Aggregate Information' (push-sum): one randomized pairwise exchange-and-update step."),
        ConsensusProtocol: ("en", "Consensus protocol", "OSFM (2007): abstract - the update rule driving peers toward agreement on a common value."),
        AverageConsensus: ("en", "Average consensus", "x_i <- x_i + eps * sum over neighbours j of (x_j - x_i) - Olfati-Saber & Murray (2004); OSFM (2007) sec II."),
        GossipAveraging: ("en", "Gossip averaging", "Randomized push-sum averaging - Kempe et al. (2003) FOCS; Xiao & Boyd (2004) 'Fast linear iterations for distributed averaging' Systems & Control Letters 53(1):65-78."),
        Disagreement: ("en", "Disagreement", "OSFM (2007) sec III: the quadratic disagreement (Lyapunov) function - the squared norm of the disagreement vector delta = x - Ave(x)1; in graph form the Laplacian potential, the sum over edges of (x_i - x_j)^2."),
        Convergence: ("en", "Convergence", "OSFM (2007): asymptotic agreement - every peer's value tends to a common value (the initial average, for average consensus on a connected graph)."),
        SpectralGap: ("en", "Spectral gap", "Fiedler (1973) 'Algebraic connectivity of graphs' Czech. Math. J. 23(2):298-305; Xiao & Boyd (2004): the algebraic connectivity lambda_2 of the graph Laplacian - the Fiedler value governing the convergence rate."),
        PeerIdentity: ("en", "Peer identity", "Lamport (1979) SRI CSL-98: the signing identity a peer acts under - to be able to sign is what an identity is. The bridge concept to the constitutive protocol."),
        TrustedNeighbor: ("en", "Trusted neighbor", "A neighbour whose reported values enter the aggregation step - the default standing of a peer whose reports have shown no protocol-detectable inconsistency (Lamport, Shostak & Pease 1982, contrapositive)."),
        DistrustedPeer: ("en", "Distrusted peer", "A peer excluded from every neighbourhood after protocol-detectable misbehaviour - Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3); Li, Krohn, Mazieres & Shasha (2004) OSDI SUNDR."),
        Equivocation: ("en", "Equivocation", "Lamport, Shostak & Pease (1982) 'The Byzantine Generals Problem' ACM TOPLAS 4(3); Li, Krohn, Mazieres & Shasha (2004) OSDI (SUNDR fork-consistency): a peer presenting inconsistent values/claims to different neighbours."),
    },

    is_a: [
        // The two concrete protocols specialise the abstract update rule.
        (AverageConsensus, ConsensusProtocol),
        (GossipAveraging, ConsensusProtocol),
        // Trust standings are standings OF peers.
        (TrustedNeighbor, Peer),
        (DistrustedPeer, Peer),
    ],

    has_a: [
        // The graph carries a neighbourhood per peer (OSFM 2007).
        (Topology, Neighborhood),
    ],

    edges: [
        // OSFM (2007); Kempe et al. (2003): peers exchange pairwise.
        (Peer, Peer, GossipsWith),
        // OSFM (2007): a peer belongs to its neighbours' neighbourhoods.
        (Peer, Neighborhood, MemberOf),
        // OSFM (2007) sec III: each round reduces the disagreement.
        (GossipRound, Disagreement, Reduces),
        // Fiedler (1973); Xiao & Boyd (2004): lambda_2 governs the rate.
        (SpectralGap, Convergence, Governs),
        // Lamport (1979): a peer acts under its signing identity.
        (Peer, PeerIdentity, IdentifiedBy),
        // LSP (1982); SUNDR: detected equivocation triggers distrust.
        (Equivocation, DistrustedPeer, Triggers),
        // The exclusion relation: peers distrust an equivocator.
        (Peer, DistrustedPeer, DistrustsAfterEquivocation),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Whether a consensus protocol requires a connected communication graph
/// to converge to global agreement — Fiedler (1973): `lambda_2 > 0` iff
/// connected; Xiao & Boyd (2004): the averaging iteration converges iff
/// the graph (sequence) is connected. `Some(true)` for both concrete
/// protocols; `None` for concepts that are not protocols (including the
/// abstract `ConsensusProtocol` parent, which fixes no graph model).
#[derive(Debug, Clone)]
pub struct RequiresConnectivity;

impl Quality for RequiresConnectivity {
    type Individual = ConsensusConcept;
    type Value = bool;

    fn get(&self, c: &ConsensusConcept) -> Option<bool> {
        use ConsensusConcept as C;
        match c {
            C::AverageConsensus | C::GossipAveraging => Some(true),
            _ => None,
        }
    }
}

/// The trust standing a trust-concept denotes — Lamport, Shostak &
/// Pease (1982): consistent-toward-everyone vs caught-inconsistent.
/// Value space is the engine's [`PeerTrust`]; `None` for every concept
/// that is not a trust standing (including the plain `Peer` parent).
#[derive(Debug, Clone)]
pub struct TrustState;

impl Quality for TrustState {
    type Individual = ConsensusConcept;
    type Value = PeerTrust;

    fn get(&self, c: &ConsensusConcept) -> Option<PeerTrust> {
        use ConsensusConcept as C;
        match c {
            C::TrustedNeighbor => Some(PeerTrust::Trusted),
            C::DistrustedPeer => Some(PeerTrust::Distrusted),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn kinded_edge_exists(
    from: ConsensusConcept,
    to: ConsensusConcept,
    kind: ConsensusRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    ConsensusCategory::morphisms()
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

/// Run [`CONSENSUS_ROUNDS`] average-consensus steps on a fixture,
/// returning every iterate (including the initial one).
fn consensus_trajectory(topology: &super::engine::SwarmTopology, initial: &[f64]) -> Vec<Vec<f64>> {
    let step = stable_step_size(topology);
    let mut trajectory = vec![initial.to_vec()];
    for _ in 0..CONSENSUS_ROUNDS {
        let next = average_consensus_step(trajectory.last().expect("non-empty"), topology, step);
        trajectory.push(next);
    }
    trajectory
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Olfati-Saber, Fax & Murray (2007) sec II: with `eps` inside the
/// stability interval, every consensus iterate is a convex combination
/// of the previous values — so every iterate stays within the convex
/// hull `[min, max]` of the initial values, and the limit is the
/// initial average.
pub struct ConsensusValueInConvexHull;

impl Axiom for ConsensusValueInConvexHull {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let topology = path_graph_p3();
        let initial = FIXTURE_INITIAL_VALUES.to_vec();
        let lo = initial.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = initial.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let trajectory = consensus_trajectory(&topology, &initial);
        let within_hull = trajectory.iter().all(|values| {
            values
                .iter()
                .all(|v| *v >= lo - NUMERICAL_SLACK && *v <= hi + NUMERICAL_SLACK)
        });
        let final_values = trajectory.last().expect("non-empty trajectory");
        let at_initial_average = final_values
            .iter()
            .all(|v| (v - FIXTURE_INITIAL_AVERAGE).abs() <= DISAGREEMENT_TOLERANCE);
        // Non-vacuity: the hull is non-degenerate and the run moved.
        let non_vacuous = hi > lo && final_values != &initial;
        verdict_from(self, within_hull && at_initial_average && non_vacuous)
    }

    pr4xis::axiom_meta!(
        "ConsensusValueInConvexHull",
        "on the P3 fixture every average-consensus iterate stays within [min, max] of the initial values and the limit is the initial average",
        "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1) sec II"
    );
}
pr4xis::register_axiom!(
    ConsensusValueInConvexHull,
    "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1) sec II"
);

/// Fiedler (1973); Xiao & Boyd (2004): connectivity is exactly what
/// convergence to global agreement needs. On the connected fixture the
/// disagreement decays below the documented tolerance; on the
/// disconnected fixture it does not; the fixtures' cited `lambda_2`
/// constants are respectively positive and zero — the spectral
/// characterization of connectivity.
pub struct ConnectedTopologyConverges;

impl Axiom for ConnectedTopologyConverges {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let connected = path_graph_p3();
        let disconnected = disconnected_two_plus_one();
        let initial = FIXTURE_INITIAL_VALUES.to_vec();

        let connected_final = consensus_trajectory(&connected, &initial)
            .last()
            .expect("non-empty trajectory")
            .clone();
        let disconnected_final = consensus_trajectory(&disconnected, &initial)
            .last()
            .expect("non-empty trajectory")
            .clone();

        let connected_converges = disagreement(&connected_final) < DISAGREEMENT_TOLERANCE;
        let disconnected_stuck =
            disagreement(&disconnected_final) >= DISCONNECTED_DISAGREEMENT_FLOOR;
        let spectral_characterization = connected.is_connected()
            && ALGEBRAIC_CONNECTIVITY_P3 > 0.0
            && !disconnected.is_connected()
            && ALGEBRAIC_CONNECTIVITY_DISCONNECTED == 0.0;
        let governs_edge = kinded_edge_exists(
            ConsensusConcept::SpectralGap,
            ConsensusConcept::Convergence,
            ConsensusRelationKind::Governs,
        );
        verdict_from(
            self,
            connected_converges && disconnected_stuck && spectral_characterization && governs_edge,
        )
    }

    pr4xis::axiom_meta!(
        "ConnectedTopologyConverges",
        "disagreement decays below tolerance on the connected fixture and stays above the floor on the disconnected one; the cited lambda_2 constants are > 0 and = 0 respectively",
        "Fiedler (1973) Czech. Math. J. 23(2); Xiao & Boyd (2004) Systems & Control Letters 53(1)"
    );
}
pr4xis::register_axiom!(
    ConnectedTopologyConverges,
    "Fiedler (1973) Czech. Math. J. 23(2); Xiao & Boyd (2004) Systems & Control Letters 53(1)"
);

/// Kempe, Dobra & Gehrke (2003): push-sum conserves mass — the ratio
/// invariant `sum(s)/sum(w)` is constant across rounds (and equals the
/// initial average under unit initial weights).
pub struct GossipMassConservation;

impl Axiom for GossipMassConservation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let topology = path_graph_p3();
        let mut state = push_sum_initial(&FIXTURE_INITIAL_VALUES);
        let expected = ratio_invariant(&state);
        let starts_at_average = (expected - FIXTURE_INITIAL_AVERAGE).abs() <= NUMERICAL_SLACK;
        let mut conserved = true;
        let mut mixed = false;
        for round in 0..PUSH_SUM_ROUNDS {
            let next = push_sum_round(&state, &topology, round);
            mixed |= next != state;
            conserved &= (ratio_invariant(&next) - expected).abs() <= NUMERICAL_SLACK;
            state = next;
        }
        verdict_from(self, starts_at_average && conserved && mixed)
    }

    pr4xis::axiom_meta!(
        "GossipMassConservation",
        "across every push-sum round on the P3 fixture, sum(s)/sum(w) stays equal to the initial average",
        "Kempe, Dobra & Gehrke (2003) FOCS"
    );
}
pr4xis::register_axiom!(GossipMassConservation, "Kempe, Dobra & Gehrke (2003) FOCS");

/// Olfati-Saber, Fax & Murray (2007) sec III: under the cited step-size
/// bound, the disagreement function is a Lyapunov function of the
/// consensus iteration — non-increasing along the run on the connected
/// fixture (and strictly decreased overall, the non-vacuity witness).
pub struct DisagreementMonotoneNonIncreasing;

impl Axiom for DisagreementMonotoneNonIncreasing {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        let topology = path_graph_p3();
        let trajectory = consensus_trajectory(&topology, &FIXTURE_INITIAL_VALUES);
        let potentials: Vec<f64> = trajectory.iter().map(|v| disagreement(v)).collect();
        let monotone = potentials
            .windows(2)
            .all(|pair| pair[1] <= pair[0] + NUMERICAL_SLACK);
        let initial = *potentials.first().expect("non-empty trajectory");
        let final_potential = *potentials.last().expect("non-empty trajectory");
        let strictly_decreased = initial > 0.0 && final_potential < initial;
        let reduces_edge = kinded_edge_exists(
            ConsensusConcept::GossipRound,
            ConsensusConcept::Disagreement,
            ConsensusRelationKind::Reduces,
        );
        verdict_from(self, monotone && strictly_decreased && reduces_edge)
    }

    pr4xis::axiom_meta!(
        "DisagreementMonotoneNonIncreasing",
        "under the cited step-size bound the disagreement function is non-increasing along the consensus iteration on the connected fixture",
        "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1) sec III"
    );
}
pr4xis::register_axiom!(
    DisagreementMonotoneNonIncreasing,
    "Olfati-Saber, Fax & Murray (2007) Proc. IEEE 95(1) sec III"
);

/// Lamport, Shostak & Pease (1982); Li et al. (2004) SUNDR: a peer
/// flagged for equivocation is moved to distrusted and its values do not
/// enter any neighbour's next aggregation — detection and exclusion
/// happen *before* the aggregation step. The category carries the
/// matching `Triggers` and `DistrustsAfterEquivocation` edges.
pub struct EquivocatorExcludedBeforeAggregation;

impl Axiom for EquivocatorExcludedBeforeAggregation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use super::engine::PeerId;

        // Structural half: the trust-wiring edges.
        let edges = kinded_edge_exists(
            ConsensusConcept::Equivocation,
            ConsensusConcept::DistrustedPeer,
            ConsensusRelationKind::Triggers,
        ) && kinded_edge_exists(
            ConsensusConcept::Peer,
            ConsensusConcept::DistrustedPeer,
            ConsensusRelationKind::DistrustsAfterEquivocation,
        );
        if !edges {
            return verdict_from(self, false);
        }

        // Operational half: the engine fixture. Middle peer 1 of P3
        // reports inconsistent values to peers 0 and 2 in the same round.
        let topology = path_graph_p3();
        let step = stable_step_size(&topology);
        let run = SwarmConsensusRun::fresh(&FIXTURE_INITIAL_VALUES, topology.clone());
        let round = equivocation_round_p3(&run.values, &topology);
        let flagged = equivocators(&round) == vec![PeerId(1)];
        let next = apply_exchange_round(&run, &round, step);

        // Moved to distrusted, and removed from every neighbourhood.
        let distrusted = next.trust[1] == PeerTrust::Distrusted;
        let excluded = !next.topology.are_neighbors(PeerId(0), PeerId(1))
            && !next.topology.are_neighbors(PeerId(1), PeerId(2));
        // Its values entered no aggregation: in P3 the equivocator was
        // the only neighbour of peers 0 and 2, so their values are
        // untouched; the equivocator's own value is frozen.
        let no_influence = next.values == run.values;
        // Non-vacuity: the honest round DOES move values, so exclusion
        // (not inertness) is what kept them fixed.
        let honest = apply_exchange_round(&run, &honest_round(&run.values, &topology), step);
        let honest_moves = honest.values != run.values;

        verdict_from(
            self,
            flagged && distrusted && excluded && no_influence && honest_moves,
        )
    }

    pr4xis::axiom_meta!(
        "EquivocatorExcludedBeforeAggregation",
        "a peer flagged for same-round inconsistent reports is moved to distrusted and its values enter no neighbour's next aggregation; the Triggers and DistrustsAfterEquivocation edges are present",
        "Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3); Li, Krohn, Mazieres & Shasha (2004) OSDI"
    );
}
pr4xis::register_axiom!(
    EquivocatorExcludedBeforeAggregation,
    "Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3); Li, Krohn, Mazieres & Shasha (2004) OSDI"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for ConsensusOntology {
    type Cat = ConsensusCategory;
    type Qual = TrustState;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ConsensusValueInConvexHull));
        axioms.push(Box::new(ConnectedTopologyConverges));
        axioms.push(Box::new(GossipMassConservation));
        axioms.push(Box::new(DisagreementMonotoneNonIncreasing));
        axioms.push(Box::new(EquivocatorExcludedBeforeAggregation));
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
        assert_category_laws::<ConsensusCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ConsensusOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn consensus_value_in_convex_hull_holds() {
        assert!(ConsensusValueInConvexHull.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn connected_topology_converges_holds() {
        assert!(ConnectedTopologyConverges.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gossip_mass_conservation_holds() {
        assert!(GossipMassConservation.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn disagreement_monotone_non_increasing_holds() {
        assert!(DisagreementMonotoneNonIncreasing.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn equivocator_excluded_before_aggregation_holds() {
        assert!(EquivocatorExcludedBeforeAggregation.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn connectivity_requirement_classification() {
        let q = RequiresConnectivity;
        assert_eq!(q.get(&ConsensusConcept::AverageConsensus), Some(true));
        assert_eq!(q.get(&ConsensusConcept::GossipAveraging), Some(true));
        assert_eq!(
            q.get(&ConsensusConcept::ConsensusProtocol),
            None,
            "the abstract protocol fixes no graph model"
        );
        assert_eq!(q.get(&ConsensusConcept::Peer), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn trust_state_classification() {
        let q = TrustState;
        assert_eq!(
            q.get(&ConsensusConcept::TrustedNeighbor),
            Some(PeerTrust::Trusted)
        );
        assert_eq!(
            q.get(&ConsensusConcept::DistrustedPeer),
            Some(PeerTrust::Distrusted)
        );
        assert_eq!(
            q.get(&ConsensusConcept::Peer),
            None,
            "the plain peer parent carries no fixed standing"
        );
    }
}
