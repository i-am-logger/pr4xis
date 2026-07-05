//! DistributedFusion engine — network fusion fixtures built ON TOP of
//! the existing sensor-fusion information form.
//!
//! The estimate type here is [`InformationEstimate`] from
//! `applied::sensor_fusion::state::information` — this engine wraps it
//! and calls its `fuse()` for every additive information step; it never
//! reimplements information fusion.
//!
//! Three fixtures:
//!
//! 1. **Covariance intersection** in information form,
//!    `P_CI^{-1} = omega * P1^{-1} + (1 - omega) * P2^{-1}`, over a
//!    named omega grid — Julier & Uhlmann (1997) *A Non-divergent
//!    Estimation Algorithm in the Presence of Unknown Correlations*,
//!    American Control Conference.
//! 2. **A 3-peer ring where naive re-fusion double-counts** — each
//!    peer's posterior circulates the cycle and is re-fused additively
//!    as if independent, so the first peer's information is counted
//!    twice: the data-incest failure — Bar-Shalom (1981) *On the
//!    Track-to-Track Correlation Problem*, IEEE TAC 26(2); Mutambara
//!    (1998) Ch. 3.
//! 3. **Consensus on information contributions** — per-entry average
//!    consensus (reusing the sibling `swarm::consensus` engine) whose
//!    network-size rescaling recovers the centralized information-filter
//!    fuse — Olfati-Saber (2007) 46th IEEE CDC *Distributed Kalman
//!    Filtering for Sensor Networks*.
//!
//! Every constant below is a documented structural fixture parameter
//! cited to its source — no free magic numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::applied::sensor_fusion::state::information::InformationEstimate;
use crate::applied::swarm::consensus::engine::{
    PeerId, SwarmTopology, average_consensus_step, stable_step_size,
};
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

// ---------------------------------------------------------------------------
// Fixture parameters
// ---------------------------------------------------------------------------

/// Dimension of the planar fixture state — the smallest dimension where
/// the positive-semidefinite ordering is non-trivial. Bar-Shalom, Li &
/// Kirubarajan (2001) develop track fusion on planar position states.
pub const FUSION_STATE_DIMENSION: usize = 2;

/// Number of peers in the ring fixture: Bar-Shalom (1981) — the
/// double-counting of shared information requires a cycle in the
/// information-flow topology, and three peers form the smallest cycle.
pub const RING_PEER_COUNT: usize = 3;

/// Fixture timestamp: the three local estimates are contemporaneous, so
/// a single epoch labels every conversion back to covariance form.
pub const FIXTURE_EPOCH: f64 = 0.0;

/// The three peers' local means — distinct planar positions so fusion
/// actually mixes information (documented structural fixture values).
pub const PEER_MEANS: [[f64; FUSION_STATE_DIMENSION]; RING_PEER_COUNT] =
    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];

/// The three peers' local variances (isotropic diagonal covariances) —
/// distinct so the information weighting is non-uniform (documented
/// structural fixture values).
pub const PEER_VARIANCES: [f64; RING_PEER_COUNT] = [1.0, 2.0, 4.0];

/// The omega sample grid: Julier & Uhlmann (1997) prove consistency of
/// the CI fusion for every `omega` in the closed interval `[0, 1]`; the
/// grid samples that cited interval at its endpoints, quartiles, and
/// midpoint.
pub const OMEGA_GRID: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// First scalar CI-fixture variance — the unit-variance estimate of the
/// two-sensor demonstration setting of Julier & Uhlmann (1997).
pub const CI_VARIANCE_A: f64 = 1.0;

/// Second scalar CI-fixture variance — distinct from [`CI_VARIANCE_A`]
/// so the omega family of fused estimates is non-degenerate.
pub const CI_VARIANCE_B: f64 = 2.0;

/// The scalar CI-fixture means. The realised-error-variance computation
/// is mean-independent (the error is measured about the truth), so zero
/// means lose nothing.
pub const CI_MEAN: f64 = 0.0;

/// The cross-correlation sample grid: Julier & Uhlmann (1997) treat the
/// cross-correlation as UNKNOWN, so consistency must hold over the whole
/// admissible range `rho` in `[-1, 1]` — sampled at its endpoints,
/// midpoints, and centre.
pub const CORRELATION_GRID: [f64; 5] = [-1.0, -0.5, 0.0, 0.5, 1.0];

/// Rounds of the consensus-on-information run. On the 3-ring the
/// Laplacian spectrum is `{0, 3, 3}` and the sibling engine's stable
/// step is `1/(2 * Delta_max) = 1/4`, so the disagreement vector
/// contracts by `|1 - 3/4| = 1/4` per round; after 64 rounds the
/// per-entry deviation is far below [`CENTRAL_AGREEMENT_TOLERANCE`].
/// Cited convergence claim: Olfati-Saber (2007) 46th IEEE CDC.
pub const CONSENSUS_FUSION_ROUNDS: usize = 64;

/// Tolerance for agreement between each peer's rescaled consensus
/// estimate and the centralized information-filter fuse — documented
/// together with the contraction derivation above.
pub const CENTRAL_AGREEMENT_TOLERANCE: f64 = 1e-9;

/// Floating-point slack for algebraically exact inequalities (the CI
/// consistency bound holds with equality at the omega endpoints).
pub const NUMERICAL_SLACK: f64 = 1e-12;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// The three peers' local estimates in information form — planar
/// positions with isotropic covariances, built through the existing
/// `StateEstimate` → `InformationEstimate` conversion (code reuse, not
/// reimplementation). `None` if a fixture covariance were singular
/// (it is not: the variances are positive).
pub fn ring_local_estimates() -> Option<Vec<InformationEstimate>> {
    PEER_MEANS
        .iter()
        .zip(PEER_VARIANCES.iter())
        .map(|(mean, variance)| {
            let state = Vector::new(mean.to_vec());
            let covariance = Matrix::diagonal(&[*variance; FUSION_STATE_DIMENSION]);
            InformationEstimate::from_estimate(&StateEstimate::new(
                state,
                covariance,
                FIXTURE_EPOCH,
            ))
        })
        .collect()
}

/// The 3-ring communication graph (the triangle `0 - 1 - 2 - 0`) —
/// the smallest cyclic topology (Bar-Shalom 1981) — built on the
/// sibling consensus engine's typed topology.
pub fn ring_topology() -> SwarmTopology {
    SwarmTopology::from_edges(
        RING_PEER_COUNT,
        &[
            (PeerId(0), PeerId(1)),
            (PeerId(1), PeerId(2)),
            (PeerId(2), PeerId(0)),
        ],
    )
}

/// The two scalar CI-fixture estimates as 1x1 information forms.
pub fn ci_scalar_estimates() -> Option<(InformationEstimate, InformationEstimate)> {
    let build = |variance: f64| {
        InformationEstimate::from_estimate(&StateEstimate::new(
            Vector::new(vec![CI_MEAN]),
            Matrix::diagonal(&[variance]),
            FIXTURE_EPOCH,
        ))
    };
    Some((build(CI_VARIANCE_A)?, build(CI_VARIANCE_B)?))
}

// ---------------------------------------------------------------------------
// Covariance intersection (Julier & Uhlmann 1997)
// ---------------------------------------------------------------------------

/// An information form scaled by a CI weight — `omega * Y`, `omega * y`.
/// The scaling is the only CI-specific operation; the combination itself
/// is delegated to [`InformationEstimate::fuse`].
pub fn scaled_information(estimate: &InformationEstimate, weight: f64) -> InformationEstimate {
    InformationEstimate {
        information_matrix: estimate.information_matrix.scale(weight),
        information_vector: estimate.information_vector.scale(weight),
    }
}

/// Covariance intersection in information form — Julier & Uhlmann
/// (1997): `P_CI^{-1} = omega * P1^{-1} + (1 - omega) * P2^{-1}` (and
/// the same convex combination of information vectors). The additive
/// step is exactly [`InformationEstimate::fuse`] applied to the two
/// omega-scaled information forms.
pub fn covariance_intersection(
    a: &InformationEstimate,
    b: &InformationEstimate,
    omega: f64,
) -> InformationEstimate {
    scaled_information(a, omega).fuse(&scaled_information(b, 1.0 - omega))
}

/// The fused scalar variance the CI fixture reports for one omega —
/// `P_CI = 1 / (omega/var_a + (1-omega)/var_b)`, read back through the
/// existing information → covariance conversion. `None` if the fused
/// information matrix were singular (it is not on the grid: at every
/// grid point at least one weight is positive).
pub fn ci_fused_scalar_variance(omega: f64) -> Option<f64> {
    let (a, b) = ci_scalar_estimates()?;
    let fused = covariance_intersection(&a, &b, omega);
    let estimate = fused.to_estimate(FIXTURE_EPOCH)?;
    Some(estimate.covariance.get(0, 0))
}

/// The realised mean-square error of the scalar CI combiner when the
/// true cross-covariance is `rho * sigma_a * sigma_b` — expanding
/// `E[(x_ci - x)^2]` for
/// `x_ci = P_CI * (omega * x1 / var_a + (1 - omega) * x2 / var_b)`:
///
/// `P_actual = P_CI^2 * (omega^2/var_a + (1-omega)^2/var_b
///             + 2*omega*(1-omega)*rho/(sigma_a*sigma_b))`.
///
/// Julier & Uhlmann (1997): consistency means `P_CI >= P_actual` for
/// every admissible `rho` — the estimator never claims more confidence
/// than it can have.
pub fn ci_realised_error_variance(
    var_a: f64,
    var_b: f64,
    rho: f64,
    omega: f64,
    fused_variance: f64,
) -> f64 {
    let cross = 2.0 * omega * (1.0 - omega) * rho / (var_a.sqrt() * var_b.sqrt());
    fused_variance
        * fused_variance
        * (omega * omega / var_a + (1.0 - omega) * (1.0 - omega) / var_b + cross)
}

// ---------------------------------------------------------------------------
// Centralized fusion and the data-incest ring (Bar-Shalom 1981)
// ---------------------------------------------------------------------------

/// The centralized information-filter fuse of independent local
/// estimates: the additive combination `Y = sum(Y_i)`, `y = sum(y_i)` —
/// folded through [`InformationEstimate::fuse`] (Mutambara 1998).
/// `None` for an empty peer set.
pub fn centralized_information_fusion(
    estimates: &[InformationEstimate],
) -> Option<InformationEstimate> {
    let mut peers = estimates.iter();
    let first = peers.next()?.clone();
    Some(peers.fold(first, |acc, e| acc.fuse(e)))
}

/// Naive additive re-fusion around the ring — the data-incest fixture
/// (Bar-Shalom 1981; Mutambara 1998 Ch. 3): peer 0's posterior
/// circulates the cycle, each hop fusing it with the local estimate *as
/// if independent*; when the ring closes, peer 0 fuses the returned
/// posterior with its own local estimate again, so its own information
/// is counted twice: `Y_naive = 2*Y_0 + Y_1 + Y_2`. Every additive step
/// is [`InformationEstimate::fuse`]. `None` for an empty peer set.
pub fn naive_ring_refusion(estimates: &[InformationEstimate]) -> Option<InformationEstimate> {
    let first = estimates.first()?;
    let mut circulating = first.clone();
    for local in estimates.iter().skip(1) {
        circulating = local.fuse(&circulating);
    }
    // The ring closes: the origin peer re-fuses the returned posterior
    // — which already contains its own contribution — with its local
    // estimate. This is the double count.
    Some(first.fuse(&circulating))
}

// ---------------------------------------------------------------------------
// Consensus on information contributions (Olfati-Saber 2007)
// ---------------------------------------------------------------------------

/// Run per-entry average consensus on the peers' information
/// contributions over `topology`, then rescale by the network size —
/// Olfati-Saber (2007) 46th IEEE CDC: consensus averages the
/// contributions, the centralized information filter sums them, and
/// `N * average = sum`. The scalar consensus step is the sibling
/// `swarm::consensus` engine's — reused, not reimplemented. Returns one
/// rescaled information estimate per peer; `None` for an empty peer set
/// or mismatched dimensions.
pub fn consensus_on_information(
    estimates: &[InformationEstimate],
    topology: &SwarmTopology,
    rounds: usize,
) -> Option<Vec<InformationEstimate>> {
    let peer_count = estimates.len();
    let dim = estimates.first()?.dim();
    if estimates.iter().any(|e| e.dim() != dim) {
        return None;
    }
    let step = stable_step_size(topology);
    let scale = peer_count as f64;

    let mut fused: Vec<InformationEstimate> = estimates
        .iter()
        .map(|_| InformationEstimate {
            information_matrix: Matrix::zeros(dim, dim),
            information_vector: Vector::zeros(dim),
        })
        .collect();

    // One scalar consensus run per information-matrix entry.
    for row in 0..dim {
        for col in 0..dim {
            let mut values: Vec<f64> = estimates
                .iter()
                .map(|e| e.information_matrix.get(row, col))
                .collect();
            for _ in 0..rounds {
                values = average_consensus_step(&values, topology, step);
            }
            for (peer, value) in values.iter().enumerate() {
                fused[peer].information_matrix.set(row, col, value * scale);
            }
        }
    }

    // One scalar consensus run per information-vector entry.
    for row in 0..dim {
        let mut values: Vec<f64> = estimates
            .iter()
            .map(|e| e.information_vector.get(row))
            .collect();
        for _ in 0..rounds {
            values = average_consensus_step(&values, topology, step);
        }
        for (peer, value) in values.iter().enumerate() {
            let current = &fused[peer].information_vector;
            let entries: Vec<f64> = (0..dim)
                .map(|i| {
                    if i == row {
                        value * scale
                    } else {
                        current.get(i)
                    }
                })
                .collect();
            fused[peer].information_vector = Vector::new(entries);
        }
    }

    Some(fused)
}

/// Every fused covariance the fixtures produce, in covariance form —
/// the CI family over [`OMEGA_GRID`], the centralized fuse, the naive
/// ring re-fusion (overconfident but still a covariance), and each
/// peer's consensus limit. The positive-semidefiniteness axiom checks
/// them all. `None` if any information matrix were singular.
pub fn fixture_fused_covariances() -> Option<Vec<Matrix>> {
    let mut out = Vec::new();

    let (a, b) = ci_scalar_estimates()?;
    for omega in OMEGA_GRID {
        let fused = covariance_intersection(&a, &b, omega);
        out.push(fused.to_estimate(FIXTURE_EPOCH)?.covariance);
    }

    let locals = ring_local_estimates()?;
    out.push(
        centralized_information_fusion(&locals)?
            .to_estimate(FIXTURE_EPOCH)?
            .covariance,
    );
    out.push(
        naive_ring_refusion(&locals)?
            .to_estimate(FIXTURE_EPOCH)?
            .covariance,
    );
    for peer in consensus_on_information(&locals, &ring_topology(), CONSENSUS_FUSION_ROUNDS)? {
        out.push(peer.to_estimate(FIXTURE_EPOCH)?.covariance);
    }

    Some(out)
}
