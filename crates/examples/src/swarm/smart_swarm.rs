//! Smart-swarm end-to-end demo — a ring of smart sensors that converge by
//! consensus-on-information, survive one equivocating peer, and converge
//! again among the trusted remainder.
//!
//! This is the finale of the smart-edge program: it wires together three
//! established pieces (none extended here) and the one synthesis
//! (`SmartElement`) that composes them.
//!
//! - Five smart sensors sit on a connected ring, each holding a local
//!   state estimate in the additive information form
//!   (`applied::sensor_fusion::state::information::InformationEstimate`).
//! - They gossip information contributions by average consensus — the
//!   `applied::swarm::fusion` engine's `consensus_on_information`, whose
//!   network-size rescaling recovers the centralized information-filter
//!   fuse (Olfati-Saber 2007). **(a) they converge to the centralized
//!   estimate within a documented tolerance.**
//! - One peer then equivocates. Every neighbour's *self-protection* — the
//!   `applied::swarm::smart_element` engine's `ObserveEquivocation` —
//!   distrusts and excludes it *before* the next aggregation (Lamport,
//!   Shostak & Pease 1982; Li et al. 2004 SUNDR). **(b) the equivocator is
//!   excluded by every neighbour.**
//! - The trusted four, whose ring minus the excluded peer is still a
//!   connected path, run consensus again and **(c) converge to the
//!   centralized fuse of the trusted remainder.**
//!
//! The estimation math and the autonomic loop are established literature;
//! the only novelty is the ontological synthesis that lets one element be
//! at once a MAPE-K manager and a signed-estimate fusion peer.

#[cfg(test)]
mod tests {
    use pr4xis_domains::applied::sensor_fusion::state::estimate::StateEstimate;
    use pr4xis_domains::applied::sensor_fusion::state::information::InformationEstimate;
    use pr4xis_domains::applied::swarm::consensus::engine::{PeerId, PeerTrust, SwarmTopology};
    use pr4xis_domains::applied::swarm::fusion::engine::{
        centralized_information_fusion, consensus_on_information,
    };
    use pr4xis_domains::applied::swarm::smart_element::engine::{
        ElementKnowledge, FusionNeighbor, MapePhase, SmartElementAction, SmartElementSituation,
        apply, trusts,
    };
    use pr4xis_domains::formal::math::linear_algebra::matrix::Matrix;
    use pr4xis_domains::formal::math::linear_algebra::vector_space::Vector;
    use pr4xis_domains::formal::math::temporal::instant::Instant;
    use pr4xis_domains::formal::math::temporal::time_system::TimeSystem;

    // === Documented, cited fixture parameters — no magic numbers ===

    /// Number of smart sensors in the swarm ring — Olfati-Saber, Fax &
    /// Murray (2007): a connected ring is the canonical peer topology. Five
    /// is the smallest odd ring larger than the fusion module's 3-cycle on
    /// which removing one peer still leaves a connected path (peers 0–3),
    /// so the trusted remainder can re-converge.
    const SWARM_SIZE: usize = 5;

    /// Dimension of the planar fixture state — Bar-Shalom, Li & Kirubarajan
    /// (2001): the smallest dimension with a non-trivial covariance
    /// ordering; the same dimension the fusion engine uses.
    const STATE_DIM: usize = 2;

    /// The five peers' local means — distinct planar positions so fusion
    /// actually mixes information (documented structural fixture values).
    const PEER_MEANS: [[f64; STATE_DIM]; SWARM_SIZE] =
        [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [1.0, 1.0]];

    /// The five peers' local variances (isotropic diagonal covariances) —
    /// distinct so the information weighting is non-uniform.
    const PEER_VARIANCES: [f64; SWARM_SIZE] = [1.0, 2.0, 3.0, 4.0, 5.0];

    /// The epoch labelling every conversion back to covariance form — the
    /// estimates are contemporaneous.
    const FIXTURE_EPOCH: Instant = Instant::new(0.0, TimeSystem::GPS);

    /// The peer that equivocates mid-run. Its ring neighbours (peers 3 and
    /// 0) are the ones whose self-protection excludes it.
    const EQUIVOCATOR: usize = 4;

    /// Consensus rounds per run. On the 5-ring the slowest disagreement
    /// mode contracts by ≈ 0.655 per round under the stable step
    /// (Olfati-Saber, Fax & Murray 2007 §II–III), so 256 rounds drive the
    /// per-entry deviation far below [`AGREEMENT_TOLERANCE`].
    const CONSENSUS_ROUNDS: usize = 256;

    /// Agreement tolerance between each peer's rescaled consensus estimate
    /// and the centralized information-filter fuse — documented together
    /// with the contraction rate above.
    const AGREEMENT_TOLERANCE: f64 = 1e-6;

    /// Build a planar isotropic estimate in information form through the
    /// existing `StateEstimate` → `InformationEstimate` conversion (reuse,
    /// not reimplementation).
    fn build_estimate(mean: &[f64], variance: f64) -> InformationEstimate {
        let state = Vector::new(mean.to_vec());
        let covariance = Matrix::diagonal(&[variance; STATE_DIM]);
        InformationEstimate::from_estimate(&StateEstimate::new(state, covariance, FIXTURE_EPOCH))
            .expect("fixture covariance is positive-definite")
    }

    /// The five peers' local estimates.
    fn swarm_estimates() -> Vec<InformationEstimate> {
        PEER_MEANS
            .iter()
            .zip(PEER_VARIANCES.iter())
            .map(|(mean, variance)| build_estimate(mean, *variance))
            .collect()
    }

    /// The 5-ring communication graph (0-1-2-3-4-0).
    fn ring_topology() -> SwarmTopology {
        SwarmTopology::from_edges(
            SWARM_SIZE,
            &[
                (PeerId(0), PeerId(1)),
                (PeerId(1), PeerId(2)),
                (PeerId(2), PeerId(3)),
                (PeerId(3), PeerId(4)),
                (PeerId(4), PeerId(0)),
            ],
        )
    }

    /// The 4-peer path (0-1-2-3) — the ring with the equivocator (peer 4)
    /// removed; still connected, so the trusted four re-converge.
    fn trusted_path_topology() -> SwarmTopology {
        SwarmTopology::from_edges(
            SWARM_SIZE - 1,
            &[
                (PeerId(0), PeerId(1)),
                (PeerId(1), PeerId(2)),
                (PeerId(2), PeerId(3)),
            ],
        )
    }

    /// The worst per-component deviation of any peer's estimate mean from a
    /// reference estimate's mean — the convergence residual.
    fn max_mean_deviation(peers: &[InformationEstimate], reference: &InformationEstimate) -> f64 {
        let ref_mean = reference
            .to_estimate(FIXTURE_EPOCH)
            .expect("reference is invertible")
            .state;
        let mut worst = 0.0_f64;
        for peer in peers {
            let mean = peer
                .to_estimate(FIXTURE_EPOCH)
                .expect("peer estimate is invertible")
                .state;
            for i in 0..mean.dim() {
                worst = worst.max((mean.get(i) - ref_mean.get(i)).abs());
            }
        }
        worst
    }

    /// One smart sensor's situation, holding its own estimate and its two
    /// ring neighbours as its fusion neighbourhood — used to drive the
    /// real `smart_element` engine's self-protection.
    fn smart_sensor(
        estimates: &[InformationEstimate],
        me: usize,
        left: usize,
        right: usize,
    ) -> SmartElementSituation {
        let neighbor = |peer: usize| FusionNeighbor {
            peer: PeerId(peer),
            trust: PeerTrust::Trusted,
            contribution: estimates[peer].clone(),
        };
        SmartElementSituation {
            local_estimate: estimates[me].clone(),
            neighborhood: vec![neighbor(left), neighbor(right)],
            knowledge: ElementKnowledge {
                configured: true,
                healthy: true,
            },
            phase: MapePhase::Monitor,
        }
    }

    /// A typed trace event from the smart-swarm run — the load-bearing
    /// verdicts the run establishes, carried as *data* rather than derived by
    /// substring-matching prose. String rendering ([`SwarmTraceEvent::render`])
    /// is purely presentational; the test asserts on these fields.
    #[derive(Debug, Clone, PartialEq)]
    enum SwarmTraceEvent {
        /// A consensus cycle converged: `peer_count` peers gossiped for
        /// `rounds` rounds; `residual` is the worst per-component deviation of
        /// any peer from the centralized fuse, which must fall within
        /// `tolerance`.
        Convergence {
            cycle: char,
            peer_count: usize,
            rounds: usize,
            residual: f64,
            tolerance: f64,
        },
        /// One neighbour observed the equivocation: whether it
        /// `trusted_before` and whether it `excluded_after` (before the next
        /// aggregation).
        ExclusionObserved {
            observer: PeerId,
            equivocator: PeerId,
            trusted_before: bool,
            excluded_after: bool,
        },
        /// Every neighbour of `equivocator` excluded it before aggregation.
        ExclusionSummary {
            equivocator: PeerId,
            all_excluded: bool,
        },
    }

    impl SwarmTraceEvent {
        /// Human-readable rendering — data, not marketing. Presentational
        /// only; the test asserts on the typed fields, never on this string.
        fn render(&self) -> String {
            match self {
                SwarmTraceEvent::Convergence {
                    cycle,
                    peer_count,
                    rounds,
                    residual,
                    tolerance,
                } => format!(
                    "cycle {cycle}: {peer_count} peers gossip for {rounds} rounds; \
                     worst deviation from the centralized fuse = {residual:.2e} (tol {tolerance:.0e})"
                ),
                SwarmTraceEvent::ExclusionObserved {
                    observer,
                    equivocator,
                    trusted_before,
                    excluded_after,
                } => format!(
                    "cycle B: peer {} observed peer {}'s equivocation \
                     (trusted before = {trusted_before}, excluded after = {excluded_after})",
                    observer.0, equivocator.0
                ),
                SwarmTraceEvent::ExclusionSummary {
                    equivocator,
                    all_excluded,
                } => format!(
                    "cycle B: peer {} excluded by every neighbour before aggregation = {all_excluded}",
                    equivocator.0
                ),
            }
        }
    }

    /// The full smart-swarm run, returning a trace of **typed** events
    /// (convergence residuals and exclusion facts) — the load-bearing
    /// verdicts as data, not prose to be matched. Rendering to human-readable
    /// lines is presentational ([`SwarmTraceEvent::render`]).
    fn run_smart_swarm() -> Vec<SwarmTraceEvent> {
        let mut trace = Vec::new();
        let estimates = swarm_estimates();
        let ring = ring_topology();

        // (a) Pre-exclusion: consensus-on-information over the whole ring.
        let central_all = centralized_information_fusion(&estimates).expect("non-empty peer set");
        let converged_all =
            consensus_on_information(&estimates, &ring, CONSENSUS_ROUNDS).expect("dims agree");
        let residual_all = max_mean_deviation(&converged_all, &central_all);
        trace.push(SwarmTraceEvent::Convergence {
            cycle: 'A',
            peer_count: SWARM_SIZE,
            rounds: CONSENSUS_ROUNDS,
            residual: residual_all,
            tolerance: AGREEMENT_TOLERANCE,
        });

        // (b) Equivocation event: peer 4 equivocates; its ring neighbours
        // (3 and 0) exclude it by self-protection, before the next
        // aggregation — driven through the real smart_element engine.
        let neighbours_of_equivocator =
            [(3usize, 2usize, EQUIVOCATOR), (0usize, EQUIVOCATOR, 1usize)];
        let mut all_excluded = true;
        for (me, left, right) in neighbours_of_equivocator {
            let sit = smart_sensor(&estimates, me, left, right);
            let before = trusts(&sit, PeerId(EQUIVOCATOR));
            let after = apply(
                &sit,
                &SmartElementAction::ObserveEquivocation {
                    peer: PeerId(EQUIVOCATOR),
                },
            );
            let excluded = !trusts(&after, PeerId(EQUIVOCATOR));
            all_excluded &= before && excluded;
            trace.push(SwarmTraceEvent::ExclusionObserved {
                observer: PeerId(me),
                equivocator: PeerId(EQUIVOCATOR),
                trusted_before: before,
                excluded_after: excluded,
            });
        }
        trace.push(SwarmTraceEvent::ExclusionSummary {
            equivocator: PeerId(EQUIVOCATOR),
            all_excluded,
        });

        // (c) Post-exclusion: the trusted four re-converge on the path.
        let trusted: Vec<InformationEstimate> = estimates[..EQUIVOCATOR].to_vec();
        let central_trusted = centralized_information_fusion(&trusted).expect("non-empty peer set");
        let converged_trusted =
            consensus_on_information(&trusted, &trusted_path_topology(), CONSENSUS_ROUNDS)
                .expect("dims agree");
        let residual_trusted = max_mean_deviation(&converged_trusted, &central_trusted);
        trace.push(SwarmTraceEvent::Convergence {
            cycle: 'C',
            peer_count: SWARM_SIZE - 1,
            rounds: CONSENSUS_ROUNDS,
            residual: residual_trusted,
            tolerance: AGREEMENT_TOLERANCE,
        });

        trace
    }

    #[test]
    fn swarm_converges_survives_equivocation_and_reconverges() {
        let estimates = swarm_estimates();
        let ring = ring_topology();

        // (a) The whole ring converges to the centralized fuse.
        let central_all = centralized_information_fusion(&estimates).expect("non-empty peer set");
        let converged_all =
            consensus_on_information(&estimates, &ring, CONSENSUS_ROUNDS).expect("dims agree");
        assert!(
            max_mean_deviation(&converged_all, &central_all) < AGREEMENT_TOLERANCE,
            "(a) the swarm must converge to the centralized information-filter estimate"
        );

        // (b) Every ring neighbour of the equivocator excludes it.
        for (me, left, right) in [(3usize, 2usize, EQUIVOCATOR), (0usize, EQUIVOCATOR, 1usize)] {
            let sit = smart_sensor(&estimates, me, left, right);
            assert!(trusts(&sit, PeerId(EQUIVOCATOR)), "starts trusted");
            let after = apply(
                &sit,
                &SmartElementAction::ObserveEquivocation {
                    peer: PeerId(EQUIVOCATOR),
                },
            );
            assert!(
                !trusts(&after, PeerId(EQUIVOCATOR)),
                "(b) neighbour {me} must exclude the equivocator before aggregation"
            );
        }

        // (c) The trusted four re-converge among themselves.
        let trusted: Vec<InformationEstimate> = estimates[..EQUIVOCATOR].to_vec();
        let central_trusted = centralized_information_fusion(&trusted).expect("non-empty peer set");
        let converged_trusted =
            consensus_on_information(&trusted, &trusted_path_topology(), CONSENSUS_ROUNDS)
                .expect("dims agree");
        assert!(
            max_mean_deviation(&converged_trusted, &central_trusted) < AGREEMENT_TOLERANCE,
            "(c) the trusted remainder must still converge to their centralized fuse"
        );
    }

    #[test]
    fn narrated_trace_reports_convergence_and_exclusion() {
        let trace = run_smart_swarm();

        // The trace covers all three cycles plus the per-neighbour and
        // summary exclusion events.
        assert!(trace.len() >= SWARM_SIZE);

        // Data, not marketing: render each typed event for the human reader.
        // The assertions below read the typed fields, never this string.
        for event in &trace {
            println!("{}", event.render());
        }

        // (a)/(c) Both consensus cycles report a residual within tolerance.
        let mut convergences = 0usize;
        for event in &trace {
            if let SwarmTraceEvent::Convergence {
                residual,
                tolerance,
                ..
            } = event
            {
                convergences += 1;
                assert!(
                    *residual < *tolerance,
                    "each consensus cycle must converge within tolerance"
                );
            }
        }
        assert_eq!(convergences, 2, "cycles A and C each report a convergence");

        // (b) Every neighbour that observed the equivocation trusted the peer
        // before and excluded it after — the typed facts from the engine.
        let mut observed = 0usize;
        for event in &trace {
            if let SwarmTraceEvent::ExclusionObserved {
                trusted_before,
                excluded_after,
                ..
            } = event
            {
                observed += 1;
                assert!(*trusted_before, "the neighbour trusted the peer before");
                assert!(*excluded_after, "the neighbour excluded the peer after");
            }
        }
        assert!(
            observed > 0,
            "at least one neighbour observes the equivocation"
        );

        // The summary event asserts the exclusion held for every neighbour —
        // the typed `all_excluded` fact, not a substring of narration.
        let all_excluded = trace
            .iter()
            .find_map(|event| match event {
                SwarmTraceEvent::ExclusionSummary { all_excluded, .. } => Some(*all_excluded),
                _ => None,
            })
            .expect("the run emits an exclusion summary");
        assert!(
            all_excluded,
            "the equivocator must be excluded by every neighbour"
        );
    }
}
