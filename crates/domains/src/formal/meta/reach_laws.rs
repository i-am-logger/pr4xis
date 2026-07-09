//! Runnable, cited engine-law axiom for the ONE graded-reach kernel + generic
//! engine ([`pr4xis::category::reach`]) — the reachability machinery the
//! runtime's `MaterializedClosure` and English's `TaxonomyStore` both
//! instantiate.
//!
//! The kernel's contract is DETERMINISM BY TOTAL ORDER: every graded query
//! (image, membership probe, lattice meet, evidence chain) is a function of
//! exactly `(the generating adjacency, the vertex type's total order)` — never
//! of the order in which the injected adjacency happens to enumerate its
//! neighbors. BFS discovery order is hop-monotone but NOT canonical within a
//! hop level (it leaks enumeration order), so the kernel canonicalizes every
//! image-shaped answer by the total order `(hops, V::Ord)` and breaks every
//! argmin tie by `V::Ord`. The falsifiable claim is NEIGHBOR-ORDER
//! INVARIANCE: reverse every neighbor list and nothing changes. This module
//! lifts that claim into a registered, discoverable `Axiom`,
//! `GradedReachDeterminism`, verifying over witness graphs with teeth
//! (an anti-`Ord` enumeration that a discovery-order tie-break would answer
//! differently, a cycle, and both memo policies of the generic engine),
//! mirroring the `packed_csr_laws` shape.
//!
//! # Literature
//!
//! - **Moore, E. F. (1959)** "The shortest path through a maze", *Proceedings
//!   of an International Symposium on the Theory of Switching, Part II*,
//!   Harvard University Press, 285–292 — unit-weight breadth-first search
//!   grades every reachable vertex at its minimal hop count; the hop grading
//!   this law fixes.
//!
//! The determinism half is the standard canonicalization move: a set-valued
//! answer is made a FUNCTION of the graph by sorting under a total order on
//! vertices, so two runs (or two engines) can be compared for equality at all.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use pr4xis::category::reach::{
    Cached, ReachSubstrate, ReachView, Uncached, graded_chain, graded_image, graded_meet,
    graded_reaches,
};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

// ── witness graphs ───────────────────────────────────────────────────────────

/// A witness adjacency: vertex → neighbor list in INSERTION order, so a
/// witness can deliberately enumerate neighbors AGAINST `u8::Ord` (the case a
/// discovery-order tie-break gets wrong).
type Adj = BTreeMap<u8, Vec<u8>>;

fn adjacency(edges: &[(u8, u8)]) -> Adj {
    let mut adj = Adj::new();
    for &(s, t) in edges {
        adj.entry(s).or_default().push(t);
    }
    adj
}

/// Forward enumeration — the neighbor lists as declared.
fn fwd(adj: &Adj) -> impl Fn(&u8) -> Vec<u8> + '_ {
    move |v| adj.get(v).cloned().unwrap_or_default()
}

/// The COUNTERFACTUAL: the same graph with every neighbor list reversed. The
/// determinism contract says this must change NOTHING.
fn rev(adj: &Adj) -> impl Fn(&u8) -> Vec<u8> + '_ {
    move |v| {
        let mut n = adj.get(v).cloned().unwrap_or_default();
        n.reverse();
        n
    }
}

/// The witness graphs the law ranges over:
/// - the DAG diamond `0 → {7, 3} → 9` whose neighbor enumeration OPPOSES
///   `u8::Ord` (declared `[7, 3]`) — the equal-distance tie a discovery-order
///   tie-break answers `7`, the total order answers `3`;
/// - a 3-cycle with an isolated vertex — termination + honest unreachability;
/// - a multi-level lattice with a redundant shortcut — minimal-hop grading
///   (the shortcut `0 → 9` must grade 9 at hop 1, not the long way's 3).
fn witness_graphs() -> Vec<Adj> {
    alloc::vec![
        adjacency(&[(0, 7), (0, 3), (7, 9), (3, 9)]),
        adjacency(&[(0, 1), (1, 2), (2, 0)]),
        adjacency(&[(0, 4), (0, 9), (4, 6), (6, 9), (9, 11)]),
    ]
}

/// One witness graph, checked exhaustively over all vertex pairs: every graded
/// query answers IDENTICALLY under forward and reversed neighbor enumeration,
/// every image is sorted by the literal `(hops ascending, vertex ascending)`
/// total order (spelled out independently of the kernel's `graded_cmp`, so a
/// broken comparator cannot certify itself), and the early-exit probe agrees
/// with image membership.
fn kernel_deterministic_on(adj: &Adj) -> bool {
    for v in 0u8..12 {
        let image = graded_image(&v, fwd(adj));
        // Neighbor-order invariance — the falsifiable core of the law.
        if image != graded_image(&v, rev(adj)) {
            return false;
        }
        // Canonical total order, literally.
        let sorted = image.windows(2).all(|w| {
            let ((v1, d1), (v2, d2)) = (&w[0], &w[1]);
            d1 < d2 || (d1 == d2 && v1 < v2)
        });
        if !sorted {
            return false;
        }
        for w in 0u8..12 {
            if graded_reaches(&v, &w, fwd(adj)) != image.iter().any(|(x, _)| *x == w) {
                return false;
            }
            if graded_meet(&v, &w, fwd(adj)) != graded_meet(&v, &w, rev(adj)) {
                return false;
            }
            if graded_chain(&v, &w, fwd(adj)) != graded_chain(&v, &w, rev(adj)) {
                return false;
            }
        }
    }
    true
}

// ── the engine leg: both memo policies answer as the kernel does ─────────────

/// A witness [`ReachSubstrate`]: one relation kind (`0`) over a witness
/// adjacency — the minimal engine instantiation, local to the law (no domain
/// store imported), exactly as `packed_csr_laws` mints local witness columns.
struct WitnessSubstrate {
    adj: Adj,
}

impl ReachSubstrate for WitnessSubstrate {
    type Kind = u8;
    type Vertex = u8;
    fn neighbors<'s>(&'s self, kind: &u8, vertex: &u8) -> impl Iterator<Item = u8> + use<'s> {
        let known = *kind == 0;
        known
            .then(|| self.adj.get(vertex))
            .flatten()
            .map(|targets| targets.as_slice())
            .unwrap_or(&[])
            .iter()
            .copied()
    }
}

/// The generic engine is the kernel under EITHER memo policy: over the same
/// substrate, a [`Cached`] view and an [`Uncached`] view answer every graded
/// query identically to the kernel free functions — including the memoized
/// REPEAT query, which must equal the first (a memo that stored a
/// non-canonical image would diverge here).
fn engine_agrees_with_kernel(adj: &Adj) -> bool {
    let substrate = WitnessSubstrate { adj: adj.clone() };
    let kind = 0u8;
    let cached: Cached<u8, u8> = Cached::default();
    let warm = ReachView::new(&substrate, &cached, &kind);
    let cold = ReachView::new(&substrate, &Uncached, &kind);
    for v in 0u8..12 {
        let want = graded_image(&v, fwd(adj));
        if warm.strict_image(&v) != want
            || warm.strict_image(&v) != want // the memo hit
            || cold.strict_image(&v) != want
        {
            return false;
        }
        for w in 0u8..12 {
            if warm.reaches(&v, &w) != graded_reaches(&v, &w, fwd(adj))
                || cold.reaches(&v, &w) != graded_reaches(&v, &w, fwd(adj))
                || warm.meet(&v, &w) != graded_meet(&v, &w, fwd(adj))
                || cold.meet(&v, &w) != graded_meet(&v, &w, fwd(adj))
                || warm.chain(&v, &w) != graded_chain(&v, &w, fwd(adj))
                || cold.chain(&v, &w) != graded_chain(&v, &w, fwd(adj))
            {
                return false;
            }
        }
    }
    true
}

// ── the axiom ────────────────────────────────────────────────────────────────

/// DETERMINISM BY TOTAL ORDER: every graded-reach query (image / probe / meet
/// / chain) is a deterministic function of `(the generating adjacency, the
/// vertex total order)` — invariant under the adjacency's neighbor enumeration
/// order, canonically sorted by `(hops, V::Ord)`, minimal-hop graded (Moore
/// 1959) — and the generic engine preserves it under BOTH memo policies.
///
/// The verification's TEETH: the diamond witness enumerates its equal-distance
/// tie pair AGAINST `u8::Ord`, so an engine that broke ties by BFS discovery
/// order (the pre-kernel behavior this law forbids returning to) fails the
/// forward/reversed equality; the tie itself is additionally PINNED to the
/// `Ord`-minimal vertex.
pub struct GradedReachDeterminism;

impl Axiom for GradedReachDeterminism {
    fn verify(&self) -> Verdict {
        let kernel_ok = witness_graphs().iter().all(kernel_deterministic_on);
        let engine_ok = witness_graphs().iter().all(engine_agrees_with_kernel);

        // The pinned tie: both diamond mids sit at hop 1; the meet MUST be the
        // Ord-minimal `3` (enumeration declares 7 first — discovery order
        // would answer 7), under both enumeration orders.
        let diamond = adjacency(&[(0, 7), (0, 3), (7, 9), (3, 9)]);
        let tie_ok = graded_meet(&0, &0, fwd(&diamond)) == Some(3)
            && graded_meet(&0, &0, rev(&diamond)) == Some(3);

        if kernel_ok && engine_ok && tie_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GradedReachDeterminism",
        "every graded-reach query is a deterministic function of (adjacency, vertex total order): neighbor-enumeration invariant, (hops, V::Ord)-canonical, minimal-hop graded, under both engine memo policies",
        "Moore (1959) The shortest path through a maze, Proceedings of an International Symposium on the Theory of Switching Part II, Harvard University Press, 285-292"
    );
}

pr4xis::register_axiom!(GradedReachDeterminism, constructor);

// ── laws-hold + discoverability (the packed_csr_laws shape) ──────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use pr4xis::ontology::registry::axiom_by_name;

    /// The engine law holds over its witnesses — kernel invariance, engine
    /// agreement under both memo policies, and the pinned anti-`Ord` tie.
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn graded_reach_determinism_holds() {
        assert!(
            GradedReachDeterminism.verify().is_ok(),
            "the graded queries must be deterministic functions of (adjacency, V::Ord)"
        );
    }

    /// The axiom re-binds by name through the registry — discoverable exactly
    /// as any statute's law is (the load-time rebind gate).
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn graded_reach_law_discoverable_via_registry() {
        assert!(
            axiom_by_name("GradedReachDeterminism").is_some(),
            "GradedReachDeterminism must re-bind through the registry"
        );
    }
}
