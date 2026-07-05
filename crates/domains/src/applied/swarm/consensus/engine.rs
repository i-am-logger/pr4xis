//! Consensus engine — three small verified fixtures over peer graphs.
//!
//! 1. **Average consensus** `x_i <- x_i + eps * sum_{j in N_i} (x_j - x_i)`
//!    with the cited step-size stability bound `eps < 1/Delta_max` —
//!    Olfati-Saber, Fax & Murray (2007) *Consensus and Cooperation in
//!    Networked Multi-Agent Systems*, Proc. IEEE 95(1) sec II.
//! 2. **Push-sum gossip** carrying `(s_i, w_i)` pairs whose mass — and
//!    therefore the ratio invariant `sum(s)/sum(w)` — is conserved by
//!    every exchange — Kempe, Dobra & Gehrke (2003) FOCS *Gossip-Based
//!    Computation of Aggregate Information*.
//! 3. **Equivocation flagging and exclusion** — a peer reporting
//!    different values to different neighbours in the same round is
//!    flagged (Lamport, Shostak & Pease 1982 ACM TOPLAS 4(3); Li, Krohn,
//!    Mazieres & Shasha 2004 OSDI, SUNDR fork-consistency) and removed
//!    from every neighbourhood *before* the aggregation step.
//!
//! The graph fixtures are the connected path `P3` and a disconnected
//! `2+1` graph, with their known algebraic connectivities as named
//! constants (Fiedler 1973). Every constant below is a documented
//! structural fixture parameter cited to its source — no free magic
//! numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

// ---------------------------------------------------------------------------
// Identities and trust states
// ---------------------------------------------------------------------------

/// A peer identity — Olfati-Saber, Fax & Murray (2007): the agent is a
/// named node of the communication graph `G = (V, E)`, never an
/// anonymous index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeerId(pub usize);

/// The binary trust standing of a peer — Lamport, Shostak & Pease
/// (1982) ACM TOPLAS 4(3): a peer either behaves consistently toward
/// every receiver (trusted, the default) or has been caught presenting
/// inconsistent claims and is excluded (distrusted). The value space of
/// the ontology's `TrustState` quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerTrust {
    /// Reported values enter neighbours' aggregation.
    Trusted,
    /// Excluded from every neighbourhood after detected misbehaviour.
    Distrusted,
}

// ---------------------------------------------------------------------------
// Topology — the communication graph G = (V, E)
// ---------------------------------------------------------------------------

/// The undirected communication graph over peers — Olfati-Saber, Fax &
/// Murray (2007) sec II: consensus dynamics are defined over `G = (V, E)`
/// through its Laplacian. Constructed only through [`SwarmTopology::from_edges`],
/// which enforces symmetry and forbids self-loops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmTopology {
    adjacency: Vec<Vec<bool>>,
}

impl SwarmTopology {
    /// Build the graph from an undirected edge list. Out-of-range and
    /// self-loop pairs are ignored (a peer is not its own neighbour in
    /// the OSFM 2007 model).
    pub fn from_edges(peer_count: usize, edges: &[(PeerId, PeerId)]) -> Self {
        let mut adjacency = vec![vec![false; peer_count]; peer_count];
        for (a, b) in edges {
            if a.0 < peer_count && b.0 < peer_count && a != b {
                adjacency[a.0][b.0] = true;
                adjacency[b.0][a.0] = true;
            }
        }
        Self { adjacency }
    }

    /// Number of peers `|V|`.
    pub fn peer_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Whether `{a, b}` is an edge of the graph.
    pub fn are_neighbors(&self, a: PeerId, b: PeerId) -> bool {
        self.adjacency
            .get(a.0)
            .and_then(|row| row.get(b.0))
            .copied()
            .unwrap_or(false)
    }

    /// The neighbourhood `N_i = { j : (i, j) in E }` — OSFM (2007).
    pub fn neighbors(&self, p: PeerId) -> Vec<PeerId> {
        match self.adjacency.get(p.0) {
            Some(row) => row
                .iter()
                .enumerate()
                .filter(|(_, connected)| **connected)
                .map(|(j, _)| PeerId(j))
                .collect(),
            None => Vec::new(),
        }
    }

    /// The degree `|N_i|` of a peer.
    pub fn degree(&self, p: PeerId) -> usize {
        self.neighbors(p).len()
    }

    /// The maximum degree `Delta_max` — the quantity the OSFM (2007)
    /// sec II step-size stability bound `eps < 1/Delta_max` is stated in.
    pub fn max_degree(&self) -> usize {
        (0..self.peer_count())
            .map(|i| self.degree(PeerId(i)))
            .max()
            .unwrap_or(0)
    }

    /// Graph connectivity by breadth-first reachability — the structural
    /// side of Fiedler (1973)'s spectral characterization (`lambda_2 > 0`
    /// iff connected).
    pub fn is_connected(&self) -> bool {
        let n = self.peer_count();
        if n == 0 {
            return true;
        }
        let mut seen = vec![false; n];
        let mut stack = vec![0usize];
        seen[0] = true;
        while let Some(i) = stack.pop() {
            for j in self.neighbors(PeerId(i)) {
                if !seen[j.0] {
                    seen[j.0] = true;
                    stack.push(j.0);
                }
            }
        }
        seen.iter().all(|reached| *reached)
    }

    /// The graph with one peer removed from every neighbourhood — the
    /// exclusion transformation applied to a distrusted peer (Li et al.
    /// 2004 SUNDR: once misbehaviour is proven, the peer leaves the
    /// pipeline).
    pub fn without_peer(&self, p: PeerId) -> SwarmTopology {
        let mut next = self.clone();
        for (i, row) in next.adjacency.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                if i == p.0 || j == p.0 {
                    *cell = false;
                }
            }
        }
        next
    }
}

// ---------------------------------------------------------------------------
// Fixture parameters (Fiedler 1973; OSFM 2007)
// ---------------------------------------------------------------------------

/// Number of peers in both graph fixtures: three is the smallest count
/// on which a path graph and a disconnected `2+1` graph are distinct and
/// a middle peer has two neighbours to equivocate between.
pub const FIXTURE_PEER_COUNT: usize = 3;

/// Initial peer values of both fixtures — three distinct values, so the
/// disagreement is non-vacuous and the convex hull `[0, 2]` is
/// non-degenerate. Their average is [`FIXTURE_INITIAL_AVERAGE`], the
/// value average consensus must converge to (Olfati-Saber, Fax & Murray
/// 2007 sec II: the agreement value is the initial average).
pub const FIXTURE_INITIAL_VALUES: [f64; FIXTURE_PEER_COUNT] = [0.0, 1.0, 2.0];

/// The average of [`FIXTURE_INITIAL_VALUES`] — the consensus target of
/// the connected fixture (OSFM 2007 sec II).
pub const FIXTURE_INITIAL_AVERAGE: f64 = 1.0;

/// Algebraic connectivity `lambda_2` of the path graph `P3` — Fiedler
/// (1973) *Algebraic connectivity of graphs*, Czech. Math. J. 23(2):
/// `lambda_2(P_n) = 2(1 - cos(pi/n))`; for `n = 3` this is
/// `2(1 - 1/2) = 1`. The full `P3` Laplacian spectrum is `{0, 1, 3}`.
pub const ALGEBRAIC_CONNECTIVITY_P3: f64 = 1.0;

/// Algebraic connectivity of the disconnected `2+1` fixture — Fiedler
/// (1973): `lambda_2 > 0` if and only if the graph is connected, so a
/// disconnected graph has `lambda_2 = 0`.
pub const ALGEBRAIC_CONNECTIVITY_DISCONNECTED: f64 = 0.0;

/// Numerator placing the step size at the midpoint of Olfati-Saber, Fax
/// & Murray (2007) sec II's open stability interval `(0, 1/Delta_max)`:
/// `eps = 1/(2 Delta_max)`. Any point of the open interval is stable;
/// the midpoint is the canonical representative.
pub const STEP_SIZE_INTERVAL_MIDPOINT: f64 = 0.5;

/// Rounds of the consensus fixture runs. With `eps = 1/(2 Delta_max) =
/// 1/4` on `P3` and Laplacian spectrum `{0, 1, 3}` (Fiedler 1973), the
/// disagreement vector contracts per round by at most
/// `max |1 - eps*lambda| = 3/4`, so the disagreement function contracts
/// by `(3/4)^2` per round (OSFM 2007 sec II-III); after 128 rounds the
/// initial disagreement of 2 shrinks below `2*(9/16)^128`, far under
/// [`DISAGREEMENT_TOLERANCE`].
pub const CONSENSUS_ROUNDS: usize = 128;

/// The convergence tolerance the connected fixture's disagreement must
/// beat after [`CONSENSUS_ROUNDS`] rounds — documented together with the
/// contraction-rate derivation above.
pub const DISAGREEMENT_TOLERANCE: f64 = 1e-9;

/// Floor the disconnected fixture's disagreement must stay above: each
/// component preserves its own average (OSFM 2007 sec II applied per
/// component), so the `2+1` run converges to `(1/2, 1/2, 2)` whose
/// disagreement from the global average `1` is `3/2`. The floor `1`
/// sits strictly between that limit and [`DISAGREEMENT_TOLERANCE`].
pub const DISCONNECTED_DISAGREEMENT_FLOOR: f64 = 1.0;

/// Floating-point slack for algebraically exact invariants (convex-hull
/// membership, mass conservation, monotone descent). The claims hold
/// with exact arithmetic; this absorbs `f64` rounding only.
pub const NUMERICAL_SLACK: f64 = 1e-12;

/// The connected fixture: the path graph `P3` (`0 - 1 - 2`) — the graph
/// whose algebraic connectivity is [`ALGEBRAIC_CONNECTIVITY_P3`]
/// (Fiedler 1973).
pub fn path_graph_p3() -> SwarmTopology {
    SwarmTopology::from_edges(
        FIXTURE_PEER_COUNT,
        &[(PeerId(0), PeerId(1)), (PeerId(1), PeerId(2))],
    )
}

/// The disconnected fixture: one edge `0 - 1` plus the isolated peer
/// `2` — the smallest graph witnessing Fiedler (1973)'s
/// `lambda_2 = 0` characterization of disconnection.
pub fn disconnected_two_plus_one() -> SwarmTopology {
    SwarmTopology::from_edges(FIXTURE_PEER_COUNT, &[(PeerId(0), PeerId(1))])
}

// ---------------------------------------------------------------------------
// Average consensus (OSFM 2007 sec II-III)
// ---------------------------------------------------------------------------

/// The stable step size `eps = 1/(2 Delta_max)` — the midpoint of the
/// OSFM (2007) sec II stability interval `(0, 1/Delta_max)`. The degree
/// is clamped to at least one so an edgeless graph yields a finite
/// (and trivially stable) step.
pub fn stable_step_size(topology: &SwarmTopology) -> f64 {
    let delta_max = topology.max_degree().max(1);
    STEP_SIZE_INTERVAL_MIDPOINT / delta_max as f64
}

/// The arithmetic mean of the peer values — the agreement value of
/// average consensus (OSFM 2007 sec II). Zero for an empty slice.
pub fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// One synchronous average-consensus step
/// `x_i <- x_i + eps * sum_{j in N_i} (x_j - x_i)` — Olfati-Saber &
/// Murray (2004); Olfati-Saber, Fax & Murray (2007) sec II.
pub fn average_consensus_step(
    values: &[f64],
    topology: &SwarmTopology,
    step_size: f64,
) -> Vec<f64> {
    values
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let neighbor_pull: f64 = topology
                .neighbors(PeerId(i))
                .iter()
                .map(|j| values[j.0] - x)
                .sum();
            x + step_size * neighbor_pull
        })
        .collect()
}

/// The quadratic disagreement (Lyapunov) function — Olfati-Saber, Fax &
/// Murray (2007) sec III: the squared norm of the disagreement vector
/// `delta = x - Ave(x) * 1`. Zero exactly at agreement; on a connected
/// graph it decays under the stable step, on a disconnected graph the
/// per-component averages differ and it stays bounded away from zero.
pub fn disagreement(values: &[f64]) -> f64 {
    let alpha = average(values);
    values.iter().map(|x| (x - alpha) * (x - alpha)).sum()
}

// ---------------------------------------------------------------------------
// Push-sum gossip (Kempe, Dobra & Gehrke 2003)
// ---------------------------------------------------------------------------

/// Kempe, Dobra & Gehrke (2003) FOCS, push-sum: the share of its
/// `(s, w)` pair a node keeps each round, sending the complement to one
/// neighbour — the protocol's "half to yourself, half to the target".
pub const PUSH_SUM_SHARE: f64 = 0.5;

/// Rounds of the push-sum fixture run — enough for the `(s, w)` pairs to
/// circulate the whole `P3` many times, so conservation is checked on a
/// genuinely mixed state, not on the initial condition.
pub const PUSH_SUM_ROUNDS: usize = 32;

/// The push-sum state: each peer carries a value mass `s_i` and a weight
/// mass `w_i`; its running estimate is `s_i / w_i` — Kempe, Dobra &
/// Gehrke (2003).
#[derive(Debug, Clone, PartialEq)]
pub struct PushSumState {
    /// Value mass per peer.
    pub s: Vec<f64>,
    /// Weight mass per peer.
    pub w: Vec<f64>,
}

/// The initial push-sum state: `s_i = x_i`, `w_i = 1` — Kempe, Dobra &
/// Gehrke (2003): with unit initial weights the mass ratio is the
/// average of the initial values.
pub fn push_sum_initial(values: &[f64]) -> PushSumState {
    PushSumState {
        s: values.to_vec(),
        w: vec![1.0; values.len()],
    }
}

/// One gossip round: every peer keeps [`PUSH_SUM_SHARE`] of its
/// `(s, w)` pair and sends the complement to one neighbour. Kempe,
/// Dobra & Gehrke (2003) draw the target uniformly at random; this
/// engine uses the deterministic rotating schedule
/// `N_i[round mod |N_i|]` — the mass-conservation invariant the axiom
/// verifies holds for *every* exchange schedule (it is a per-exchange
/// conservation law), so determinism loses nothing and keeps the
/// fixture reproducible. Isolated peers keep their whole pair.
pub fn push_sum_round(
    state: &PushSumState,
    topology: &SwarmTopology,
    round: usize,
) -> PushSumState {
    let n = state.s.len();
    let mut next = PushSumState {
        s: vec![0.0; n],
        w: vec![0.0; n],
    };
    for i in 0..n {
        let neighbors = topology.neighbors(PeerId(i));
        if neighbors.is_empty() {
            next.s[i] += state.s[i];
            next.w[i] += state.w[i];
            continue;
        }
        let target = neighbors[round % neighbors.len()];
        let kept_s = state.s[i] * PUSH_SUM_SHARE;
        let kept_w = state.w[i] * PUSH_SUM_SHARE;
        next.s[i] += kept_s;
        next.w[i] += kept_w;
        next.s[target.0] += state.s[i] - kept_s;
        next.w[target.0] += state.w[i] - kept_w;
    }
    next
}

/// The conserved ratio invariant `sum(s) / sum(w)` — Kempe, Dobra &
/// Gehrke (2003): exchanges move mass between peers but never create or
/// destroy it, so this ratio is constant across rounds (and equals the
/// initial average under unit initial weights).
pub fn ratio_invariant(state: &PushSumState) -> f64 {
    let total_w: f64 = state.w.iter().sum();
    if total_w == 0.0 {
        return 0.0;
    }
    state.s.iter().sum::<f64>() / total_w
}

/// Each peer's running estimate `s_i / w_i` — Kempe, Dobra & Gehrke
/// (2003). Peers with zero weight report zero.
pub fn push_sum_estimates(state: &PushSumState) -> Vec<f64> {
    state
        .s
        .iter()
        .zip(state.w.iter())
        .map(|(s, w)| if *w == 0.0 { 0.0 } else { s / w })
        .collect()
}

// ---------------------------------------------------------------------------
// Equivocation detection and exclusion (LSP 1982; Li et al. 2004 SUNDR)
// ---------------------------------------------------------------------------

/// Offset between the equivocator's two inconsistent same-round reports
/// in the fixture. Any non-zero offset realises Lamport, Shostak &
/// Pease (1982)'s inconsistent-claims behaviour; one keeps the fixture
/// values on the same scale as [`FIXTURE_INITIAL_VALUES`].
pub const EQUIVOCATION_OFFSET: f64 = 1.0;

/// One peer's value report to one neighbour in one exchange round.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueReport {
    /// The reporting peer.
    pub reporter: PeerId,
    /// The neighbour the report is addressed to.
    pub recipient: PeerId,
    /// The claimed local value.
    pub value: f64,
}

/// One exchange round: every report sent in it — the engine `Action`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeRound {
    /// The reports of the round.
    pub reports: Vec<ValueReport>,
}

impl Action for ExchangeRound {
    type Sit = SwarmConsensusRun;
}

/// The honest round: every peer reports its actual value to each of its
/// neighbours — the protocol-conformant behaviour every peer is assumed
/// to follow until an inconsistency proves otherwise (LSP 1982).
pub fn honest_round(values: &[f64], topology: &SwarmTopology) -> ExchangeRound {
    let mut reports = Vec::new();
    for (i, value) in values.iter().enumerate() {
        for j in topology.neighbors(PeerId(i)) {
            reports.push(ValueReport {
                reporter: PeerId(i),
                recipient: j,
                value: *value,
            });
        }
    }
    ExchangeRound { reports }
}

/// The equivocation fixture round on `P3`: the middle peer `1` reports
/// `x_1 + offset` to peer `0` and `x_1 - offset` to peer `2` — the
/// same-round inconsistent claims of Lamport, Shostak & Pease (1982);
/// the pair of conflicting reports IS the proof (Li et al. 2004 SUNDR).
/// Peers `0` and `2` report honestly.
pub fn equivocation_round_p3(values: &[f64], topology: &SwarmTopology) -> ExchangeRound {
    let mut round = ExchangeRound {
        reports: Vec::new(),
    };
    for report in honest_round(values, topology).reports {
        if report.reporter == PeerId(1) {
            let skew = if report.recipient == PeerId(0) {
                EQUIVOCATION_OFFSET
            } else {
                -EQUIVOCATION_OFFSET
            };
            round.reports.push(ValueReport {
                value: report.value + skew,
                ..report
            });
        } else {
            round.reports.push(report);
        }
    }
    round
}

/// The peers that equivocated in a round: any reporter with two reports
/// of differing value in the same round — detectable from message
/// inconsistency alone, with no appeal to ground truth (Lamport,
/// Shostak & Pease 1982; Li et al. 2004 SUNDR fork-consistency).
pub fn equivocators(round: &ExchangeRound) -> Vec<PeerId> {
    let mut out: Vec<PeerId> = Vec::new();
    for a in &round.reports {
        let inconsistent = round
            .reports
            .iter()
            .any(|b| b.reporter == a.reporter && b.value != a.value);
        if inconsistent && !out.contains(&a.reporter) {
            out.push(a.reporter);
        }
    }
    out
}

/// The joint state of a trust-aware consensus run — the engine
/// `Situation`: peer values, per-peer trust standing, and the current
/// (exclusion-pruned) topology.
#[derive(Debug, Clone, PartialEq)]
pub struct SwarmConsensusRun {
    /// Each peer's current local value.
    pub values: Vec<f64>,
    /// Each peer's trust standing.
    pub trust: Vec<PeerTrust>,
    /// The communication graph, with distrusted peers already excluded.
    pub topology: SwarmTopology,
}

impl Situation for SwarmConsensusRun {}

impl SwarmConsensusRun {
    /// A fresh run: given values, every peer trusted, given topology.
    pub fn fresh(values: &[f64], topology: SwarmTopology) -> Self {
        Self {
            values: values.to_vec(),
            trust: vec![PeerTrust::Trusted; values.len()],
            topology,
        }
    }
}

/// Apply one exchange round: **detect, exclude, then aggregate** — in
/// that order, so an equivocator's values never enter any neighbour's
/// aggregation (LSP 1982; Li et al. 2004 SUNDR gate-0 exclusion):
///
/// 1. every reporter with inconsistent same-round reports is moved to
///    [`PeerTrust::Distrusted`];
/// 2. every distrusted peer is removed from every neighbourhood;
/// 3. each still-trusted peer takes the OSFM (2007) sec II consensus
///    step over the pruned neighbourhood, using only reports from
///    trusted reporters. Distrusted peers' values are frozen.
pub fn apply_exchange_round(
    run: &SwarmConsensusRun,
    round: &ExchangeRound,
    step_size: f64,
) -> SwarmConsensusRun {
    let mut next = run.clone();
    // 1. Detection.
    for peer in equivocators(round) {
        if peer.0 < next.trust.len() {
            next.trust[peer.0] = PeerTrust::Distrusted;
        }
    }
    // 2. Exclusion before aggregation.
    for (i, trust) in next.trust.iter().enumerate() {
        if *trust == PeerTrust::Distrusted {
            next.topology = next.topology.without_peer(PeerId(i));
        }
    }
    // 3. Aggregation over the pruned graph, trusted reports only.
    let values: Vec<f64> = next
        .values
        .iter()
        .enumerate()
        .map(|(i, x)| {
            if next.trust[i] == PeerTrust::Distrusted {
                return *x;
            }
            let pull: f64 = round
                .reports
                .iter()
                .filter(|r| {
                    r.recipient == PeerId(i)
                        && next.trust[r.reporter.0] == PeerTrust::Trusted
                        && next.topology.are_neighbors(r.reporter, PeerId(i))
                })
                .map(|r| r.value - x)
                .sum();
            x + step_size * pull
        })
        .collect();
    next.values = values;
    next
}
