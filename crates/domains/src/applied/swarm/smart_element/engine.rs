//! SmartElement engine — the autonomic edge-element loop, pure `no_std`.
//!
//! A smart element runs a local MAPE-K loop (Kephart & Chess 2003 §3)
//! over a queryable local knowledge base, holding a state estimate in the
//! additive information form and fusing trusted neighbours' contributions.
//! Two facts about this engine are load-bearing and honestly framed:
//!
//! - the estimate math is *reused, never reimplemented*: the local
//!   estimate is the existing [`InformationEstimate`] from
//!   `applied::sensor_fusion::state::information`, and every aggregation
//!   step is its additive `fuse()` (Maybeck 1979; Mutambara 1998);
//! - the trust discipline is the *same* protocol-detectable exclusion the
//!   sibling `swarm::consensus` engine uses — a peer caught equivocating
//!   is distrusted and drops out of the fusion neighbourhood *before* the
//!   next aggregation (Lamport, Shostak & Pease 1982; Li, Krohn, Mazieres
//!   & Shasha 2004 SUNDR). This engine reuses its `PeerId` / `PeerTrust`.
//!
//! The novelty this file carries is only the *synthesis*: one element that
//! is simultaneously a MAPE-K autonomic manager and a signed-estimate
//! fusion peer. Every constant below is a documented, cited fixture
//! parameter — no free magic numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::applied::sensor_fusion::state::information::InformationEstimate;
use crate::applied::swarm::consensus::engine::{PeerId, PeerTrust};
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

// ---------------------------------------------------------------------------
// The four MAPE-K phases and their canonical order (Kephart & Chess 2003)
// ---------------------------------------------------------------------------

/// The phase a smart element's autonomic manager is currently in —
/// Kephart & Chess (2003) *The Vision of Autonomic Computing*, IEEE
/// Computer 36(1) §3: Monitor, Analyze, Plan, Execute over a shared
/// Knowledge base. The typed loop position, never a bare index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapePhase {
    /// Observe the managed element and update local knowledge.
    Monitor,
    /// Diagnose what (if anything) needs to change.
    Analyze,
    /// Decide on a plan of action.
    Plan,
    /// Carry out the plan; loop closes back to Monitor.
    Execute,
}

impl MapePhase {
    /// The successor phase in the closed cycle — Kephart & Chess (2003)
    /// §3: Monitor → Analyze → Plan → Execute → Monitor. The wrap from
    /// Execute back to Monitor is the loop closure that makes MAPE-K a
    /// cycle, not a linear pipeline.
    pub fn next(self) -> MapePhase {
        match self {
            MapePhase::Monitor => MapePhase::Analyze,
            MapePhase::Analyze => MapePhase::Plan,
            MapePhase::Plan => MapePhase::Execute,
            MapePhase::Execute => MapePhase::Monitor,
        }
    }
}

/// Number of MAPE phases in one autonomic cycle — Kephart & Chess (2003)
/// §3: the loop is exactly four-phase (Monitor / Analyze / Plan /
/// Execute).
pub const MAPE_PHASE_COUNT: usize = 4;

/// The canonical order the four phases are exercised in one cycle —
/// Kephart & Chess (2003) §3. The loop-closure witness the engine
/// asserts: a full cycle visits the phases in exactly this sequence and
/// wraps back to Monitor.
pub const MAPE_PHASE_ORDER: [MapePhase; MAPE_PHASE_COUNT] = [
    MapePhase::Monitor,
    MapePhase::Analyze,
    MapePhase::Plan,
    MapePhase::Execute,
];

// ---------------------------------------------------------------------------
// The planar estimate fixture (Bar-Shalom, Li & Kirubarajan 2001)
// ---------------------------------------------------------------------------

/// Dimension of the planar fixture state — the smallest dimension where
/// the positive-semidefinite ordering of covariances is non-trivial.
/// Bar-Shalom, Li & Kirubarajan (2001) develop track fusion on planar
/// position states; the sibling `swarm::fusion` engine uses the same
/// dimension.
pub const SMART_ELEMENT_STATE_DIMENSION: usize = 2;

/// Fixture timestamp: the element and its neighbours hold contemporaneous
/// estimates, so a single epoch labels every conversion between covariance
/// and information form.
pub const SMART_FIXTURE_EPOCH: f64 = 0.0;

/// The smart element's own local mean — a documented structural fixture
/// value (distinct from its neighbours so fusion actually mixes
/// information).
pub const SELF_ESTIMATE_MEAN: [f64; SMART_ELEMENT_STATE_DIMENSION] = [0.0, 0.0];

/// The smart element's own local variance (isotropic diagonal
/// covariance) — a documented structural fixture value.
pub const SELF_ESTIMATE_VARIANCE: f64 = 1.0;

/// The honest neighbour's local mean — a documented structural fixture
/// value distinct from the element's own so its contribution is
/// observable in the fused posterior.
pub const HONEST_NEIGHBOR_MEAN: [f64; SMART_ELEMENT_STATE_DIMENSION] = [1.0, 0.0];

/// The honest neighbour's local variance — distinct from the element's so
/// the information weighting is non-uniform.
pub const HONEST_NEIGHBOR_VARIANCE: f64 = 2.0;

/// The equivocating neighbour's local mean — a documented structural
/// fixture value; its exclusion changes the fused posterior (the axiom's
/// non-vacuity witness).
pub const EQUIVOCATOR_NEIGHBOR_MEAN: [f64; SMART_ELEMENT_STATE_DIMENSION] = [0.0, 1.0];

/// The equivocating neighbour's local variance.
pub const EQUIVOCATOR_NEIGHBOR_VARIANCE: f64 = 4.0;

/// The smart element itself, as a named peer of the fusion network
/// (Olfati-Saber, Fax & Murray 2007: peers are named graph nodes, never
/// anonymous indices).
pub const SELF_PEER: PeerId = PeerId(0);

/// The honest neighbour peer.
pub const HONEST_PEER: PeerId = PeerId(1);

/// The equivocating neighbour peer — the one the self-protection property
/// excludes.
pub const EQUIVOCATOR_PEER: PeerId = PeerId(2);

// ---------------------------------------------------------------------------
// Situation — the joint state of one smart element
// ---------------------------------------------------------------------------

/// The knowledge flags a smart element consults and updates over its MAPE
/// cycle — Kephart & Chess (2003) §3: the local Knowledge base. Kept
/// deliberately small (configuration and health), the two self-* concerns
/// the fixture exercises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementKnowledge {
    /// Set by the Monitor phase once the managed element is sensed —
    /// the self-configuration flag.
    pub configured: bool,
    /// Set by the Analyze phase once health is diagnosed — the
    /// self-healing flag.
    pub healthy: bool,
}

/// A neighbour in a smart element's fusion neighbourhood: the peer's
/// identity, its current trust standing, and the information contribution
/// it most recently gossiped. Reuses the sibling consensus engine's
/// [`PeerId`] and [`PeerTrust`] — the trust standing is not redefined
/// here.
#[derive(Debug, Clone)]
pub struct FusionNeighbor {
    /// The neighbour's peer identity.
    pub peer: PeerId,
    /// Trusted (its contribution enters aggregation) or Distrusted
    /// (excluded after detected equivocation).
    pub trust: PeerTrust,
    /// The neighbour's gossiped information contribution.
    pub contribution: InformationEstimate,
}

impl PartialEq for FusionNeighbor {
    fn eq(&self, other: &Self) -> bool {
        self.peer == other.peer
            && self.trust == other.trust
            && information_eq(&self.contribution, &other.contribution)
    }
}

/// Structural equality of two information forms — `InformationEstimate`
/// carries `Matrix`/`Vector` (both `PartialEq`) but does not itself derive
/// `PartialEq`, so this compares the two components directly. Exact
/// equality is the right notion here: the engine builds and copies these
/// values, it does not perturb them.
pub fn information_eq(a: &InformationEstimate, b: &InformationEstimate) -> bool {
    a.information_matrix == b.information_matrix && a.information_vector == b.information_vector
}

/// The joint state of one smart element — the engine [`Situation`]: its
/// local estimate in information form, its fusion neighbourhood with trust
/// standings, its local knowledge flags, and the MAPE phase its autonomic
/// manager is currently in.
#[derive(Debug, Clone)]
pub struct SmartElementSituation {
    /// The element's own state estimate, in the additive information form
    /// (reused from `applied::sensor_fusion`, never reimplemented).
    pub local_estimate: InformationEstimate,
    /// The peers this element fuses with, each with its trust standing.
    pub neighborhood: Vec<FusionNeighbor>,
    /// The local knowledge base the MAPE loop consults and updates.
    pub knowledge: ElementKnowledge,
    /// The phase the autonomic manager is in.
    pub phase: MapePhase,
}

impl PartialEq for SmartElementSituation {
    fn eq(&self, other: &Self) -> bool {
        information_eq(&self.local_estimate, &other.local_estimate)
            && self.neighborhood == other.neighborhood
            && self.knowledge == other.knowledge
            && self.phase == other.phase
    }
}

impl Situation for SmartElementSituation {}

// ---------------------------------------------------------------------------
// Actions — the MAPE cycle steps plus the two fusion-peer actions
// ---------------------------------------------------------------------------

/// A smart element's engine action: the four MAPE-K phase steps (Kephart &
/// Chess 2003), plus the two actions that make the element a fusion peer —
/// gossiping an estimate with a trusted neighbour (Olfati-Saber 2007) and
/// observing a neighbour's equivocation (Lamport, Shostak & Pease 1982;
/// Li et al. 2004 SUNDR).
#[derive(Debug, Clone)]
pub enum SmartElementAction {
    /// Monitor: sense the managed element; update the knowledge base.
    Sense,
    /// Analyze: diagnose health from the sensed knowledge.
    Analyze,
    /// Plan: decide on a plan of action.
    Plan,
    /// Execute: aggregate over the trusted neighbourhood; loop back to
    /// Monitor.
    Act,
    /// Fuse in one trusted neighbour's contribution — a pairwise
    /// consensus-on-information step (Olfati-Saber 2007).
    GossipEstimate {
        /// The peer to gossip with.
        with: PeerId,
    },
    /// Self-protection: record a neighbour as an equivocator, moving it to
    /// Distrusted so it is excluded from the next aggregation.
    ObserveEquivocation {
        /// The peer caught presenting inconsistent claims.
        peer: PeerId,
    },
}

impl Action for SmartElementAction {
    type Sit = SmartElementSituation;
}

// ---------------------------------------------------------------------------
// Fixture construction
// ---------------------------------------------------------------------------

/// Build a planar isotropic estimate in information form through the
/// existing `StateEstimate` → `InformationEstimate` conversion (code
/// reuse, not reimplementation). `None` only if the covariance were
/// singular — it is not, the variances are positive.
fn build_estimate(mean: &[f64], variance: f64) -> Option<InformationEstimate> {
    let state = Vector::new(mean.to_vec());
    let covariance = Matrix::diagonal(&[variance; SMART_ELEMENT_STATE_DIMENSION]);
    InformationEstimate::from_estimate(&StateEstimate::new(state, covariance, SMART_FIXTURE_EPOCH))
}

/// The smart-element fixture: one element with a two-peer fusion
/// neighbourhood — an honest neighbour and (initially trusted) an
/// equivocator. Every estimate is built through the existing
/// information-form conversion. `None` only on a singular fixture
/// covariance (there is none).
pub fn smart_element_fixture() -> Option<SmartElementSituation> {
    let local = build_estimate(&SELF_ESTIMATE_MEAN, SELF_ESTIMATE_VARIANCE)?;
    let honest = FusionNeighbor {
        peer: HONEST_PEER,
        trust: PeerTrust::Trusted,
        contribution: build_estimate(&HONEST_NEIGHBOR_MEAN, HONEST_NEIGHBOR_VARIANCE)?,
    };
    let equivocator = FusionNeighbor {
        peer: EQUIVOCATOR_PEER,
        trust: PeerTrust::Trusted,
        contribution: build_estimate(&EQUIVOCATOR_NEIGHBOR_MEAN, EQUIVOCATOR_NEIGHBOR_VARIANCE)?,
    };
    Some(SmartElementSituation {
        local_estimate: local,
        neighborhood: vec![honest, equivocator],
        knowledge: ElementKnowledge {
            configured: false,
            healthy: false,
        },
        phase: MapePhase::Monitor,
    })
}

// ---------------------------------------------------------------------------
// Aggregation and trust queries
// ---------------------------------------------------------------------------

/// Whether a peer is a *trusted* member of the fusion neighbourhood — the
/// standing whose contributions enter aggregation (Lamport, Shostak &
/// Pease 1982, contrapositive: trusted until proven inconsistent).
pub fn trusts(sit: &SmartElementSituation, peer: PeerId) -> bool {
    sit.neighborhood
        .iter()
        .any(|n| n.peer == peer && n.trust == PeerTrust::Trusted)
}

/// The Execute-phase aggregation: additively fuse the local estimate with
/// every *trusted* neighbour's information contribution — Mutambara
/// (1998): the information form is additive, and the fuse is
/// [`InformationEstimate::fuse`], never reimplemented. Distrusted
/// (excluded) neighbours contribute nothing.
pub fn aggregate_trusted(sit: &SmartElementSituation) -> InformationEstimate {
    sit.neighborhood
        .iter()
        .filter(|n| n.trust == PeerTrust::Trusted)
        .fold(sit.local_estimate.clone(), |acc, n| {
            acc.fuse(&n.contribution)
        })
}

// ---------------------------------------------------------------------------
// The MAPE step function
// ---------------------------------------------------------------------------

/// Apply one engine action, returning the next situation. The four MAPE
/// actions each advance the phase along the closed cycle (Kephart & Chess
/// 2003 §3); `Act` performs the aggregation and closes the loop back to
/// Monitor. `ObserveEquivocation` moves the named peer to Distrusted —
/// detection-and-exclusion happening *before* any later aggregation reads
/// the neighbourhood (Lamport, Shostak & Pease 1982; Li et al. 2004
/// SUNDR).
pub fn apply(sit: &SmartElementSituation, action: &SmartElementAction) -> SmartElementSituation {
    let mut next = sit.clone();
    match action {
        SmartElementAction::Sense => {
            // Monitor: the managed element is sensed and configured.
            next.knowledge.configured = true;
            next.phase = next.phase.next();
        }
        SmartElementAction::Analyze => {
            // Analyze: diagnose health from the sensed knowledge.
            next.knowledge.healthy = next.knowledge.configured;
            next.phase = next.phase.next();
        }
        SmartElementAction::Plan => {
            next.phase = next.phase.next();
        }
        SmartElementAction::Act => {
            // Execute: the local estimate becomes the trusted-neighbourhood
            // fused posterior; the loop closes back to Monitor.
            next.local_estimate = aggregate_trusted(sit);
            next.phase = next.phase.next();
        }
        SmartElementAction::GossipEstimate { with } => {
            // A pairwise consensus-on-information step: fold in one trusted
            // peer's contribution. Untrusted / unknown peers are ignored.
            if let Some(n) = sit
                .neighborhood
                .iter()
                .find(|n| n.peer == *with && n.trust == PeerTrust::Trusted)
            {
                next.local_estimate = sit.local_estimate.fuse(&n.contribution);
            }
        }
        SmartElementAction::ObserveEquivocation { peer } => {
            // Self-protection: the caught equivocator is distrusted. It
            // stays in the neighbourhood as a recorded standing (mirroring
            // the consensus engine's trust vector) but no longer enters
            // aggregation.
            for n in next.neighborhood.iter_mut() {
                if n.peer == *peer {
                    n.trust = PeerTrust::Distrusted;
                }
            }
        }
    }
    next
}

/// Run one full autonomic cycle — Monitor → Analyze → Plan → Execute in
/// order (Kephart & Chess 2003 §3) — returning the resulting situation and
/// the ordered phases the four MAPE actions acted in. The returned phase
/// sequence is the loop-closure witness: it equals [`MAPE_PHASE_ORDER`]
/// and the final situation's phase has wrapped back to Monitor.
pub fn run_mape_cycle(sit: &SmartElementSituation) -> (SmartElementSituation, Vec<MapePhase>) {
    let steps = [
        SmartElementAction::Sense,
        SmartElementAction::Analyze,
        SmartElementAction::Plan,
        SmartElementAction::Act,
    ];
    let mut current = sit.clone();
    let mut visited = Vec::with_capacity(MAPE_PHASE_COUNT);
    for step in &steps {
        visited.push(current.phase);
        current = apply(&current, step);
    }
    (current, visited)
}
