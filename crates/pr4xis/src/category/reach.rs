//! The ONE hop-graded reachability kernel — free functions, generic over the
//! vertex, with the adjacency INJECTED.
//!
//! Every reachability engine in the workspace answers the same four questions
//! over a directed generating graph: the graded image of a vertex (its strict
//! reachable set, each member with its minimal hop count), a membership probe,
//! the lattice meet of two vertices, and the ordered evidence chain between
//! two vertices. Before this module those answers were hand-copied per engine
//! (the runtime's `LazyKindReach`, English's `TaxonomyStore`) and
//! hand-synchronized — same algorithm token-for-token, divergent tie-breaks.
//! This module is the single home: one cycle-safe breadth-first walk, one
//! grading, ONE deterministic output order.
//!
//! # The algorithm
//!
//! Breadth-first search over the injected `neighbors` adjacency, guarded by a
//! seen-set: the first time a vertex is enqueued is along a fewest-hop path,
//! so its recorded hop count is minimal, and each vertex is enqueued at most
//! once, so the walk terminates even on cyclic generators — bounded by the
//! reachable-set size, never divergent.
//!
//! Literature: Moore (1959) "The shortest path through a maze", *Proceedings
//! of an International Symposium on the Theory of Switching*, Harvard
//! University Press — unit-weight breadth-first search grades every reachable
//! vertex at its minimal hop count.
//!
//! # The determinism contract — `(hops, V::Ord)`
//!
//! BFS discovery order is hop-monotone but NOT canonical within a hop level:
//! it leaks the adjacency's enumeration order. Every image-shaped answer this
//! kernel returns is therefore explicitly sorted by the TOTAL order
//! `(hops, V::Ord)` ([`graded_cmp`]), and every argmin ([`graded_meet`],
//! [`graded_meet_of`]) breaks distance ties by `V::Ord`. Because the vertices
//! of an image are distinct, `(hops, V::Ord)` is a total order on the image —
//! the output is a canonical function of the GRAPH, independent of how the
//! injected adjacency enumerates its neighbors. Callers pick the tie-break by
//! picking `V`'s `Ord` (English: `ConceptId`'s `value()` order; the runtime:
//! `ConceptRef`'s derived `(ontology, name)` order).
//!
//! # What is injected
//!
//! `neighbors: impl Fn(&V) -> I, I: IntoIterator<Item = V>` — the generating
//! adjacency, borrowed from whatever representation the engine holds (a CSR
//! column, a `BTreeMap`, an archived buffer). The kernel is STATELESS: no
//! memo, no interior mutability, `no_std + alloc`. An engine that memoizes
//! (the runtime's `RefCell` memo) stores this kernel's output; an engine that
//! must stay `Sync` (English) calls it per query.
//!
//! NOTE: the eager `ReachabilityClosure` in [`quiver`](super::quiver) is
//! deliberately NOT delegated to this kernel — it is the independent test
//! oracle the full-corpus equivalence gates compare the kernel-backed engines
//! against, and an oracle must not share its subject's implementation.

use alloc::collections::{BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// THE canonical order on graded vertices — `(hops, V::Ord)`, nearest first,
/// distance ties broken by the vertex's own total order. This single
/// comparator IS the determinism contract: every sorted image ([`graded_image`],
/// [`graded_chain`]) sorts by it and every argmin ([`graded_meet`],
/// [`graded_meet_of`]) minimizes by it, so all kernel answers agree on one
/// ordering.
pub fn graded_cmp<V: Ord>(a: &(V, u32), b: &(V, u32)) -> Ordering {
    a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))
}

/// The STRICT hop-graded reachable image of `source` — every vertex reachable
/// along one or more `neighbors` edges (excluding `source` itself; a cycle
/// back to `source` does not re-admit it), each paired with its minimal hop
/// count (Moore 1959), sorted by the canonical `(hops, V::Ord)` order
/// ([`graded_cmp`]). Cycle-safe: the seen-set admits each vertex once, so the
/// walk is bounded by the reachable-set size.
pub fn graded_image<V, I>(source: &V, neighbors: impl Fn(&V) -> I) -> Vec<(V, u32)>
where
    V: Ord + Clone,
    I: IntoIterator<Item = V>,
{
    let mut image: Vec<(V, u32)> = Vec::new();
    let mut seen: BTreeSet<V> = BTreeSet::new();
    seen.insert(source.clone());
    let mut queue: VecDeque<(V, u32)> = VecDeque::new();
    queue.push_back((source.clone(), 0));
    while let Some((vertex, hops)) = queue.pop_front() {
        for next in neighbors(&vertex) {
            if seen.insert(next.clone()) {
                queue.push_back((next.clone(), hops + 1));
                image.push((next, hops + 1));
            }
        }
    }
    // BFS discovery order is hop-monotone but leaks the adjacency's neighbor
    // enumeration within a hop level — canonicalize to (hops, V::Ord).
    image.sort_unstable_by(graded_cmp);
    image
}

/// Does `source` STRICTLY reach `target`? — membership of `target` in
/// `source`'s strict image, computed with an early exit (the full image is not
/// materialized). The reflexive `source == target` case is the CALLER's
/// per-relation decision, never assumed here: exactly like [`graded_image`],
/// a cycle back to `source` does not place `source` in its own strict image,
/// so `graded_reaches(v, v, ..)` is `false` even on cyclic generators.
pub fn graded_reaches<V, I>(source: &V, target: &V, neighbors: impl Fn(&V) -> I) -> bool
where
    V: Ord + Clone,
    I: IntoIterator<Item = V>,
{
    let mut seen: BTreeSet<V> = BTreeSet::new();
    seen.insert(source.clone());
    let mut queue: VecDeque<V> = VecDeque::new();
    queue.push_back(source.clone());
    while let Some(vertex) = queue.pop_front() {
        for next in neighbors(&vertex) {
            if seen.insert(next.clone()) {
                if &next == target {
                    return true;
                }
                queue.push_back(next);
            }
        }
    }
    false
}

/// The lattice MEET of `a` and `b` — the nearest vertex in
/// `strict_image(b) ∩ reflexive_image(a)`, ranked by hops from `b` (nearest
/// first), distance ties broken by `V::Ord` ([`graded_cmp`]). `b`'s strict
/// image (a common ancestor sits strictly above `b`'s own level) against `a`'s
/// REFLEXIVE image (so when `a` is itself an ancestor of `b`, `a` is a valid
/// meet).
pub fn graded_meet<V, I>(a: &V, b: &V, neighbors: impl Fn(&V) -> I) -> Option<V>
where
    V: Ord + Clone,
    I: IntoIterator<Item = V>,
{
    let mut reflexive_a = alloc::vec![(a.clone(), 0u32)];
    reflexive_a.extend(graded_image(a, &neighbors));
    graded_meet_of(&reflexive_a, &graded_image(b, &neighbors))
}

/// The meet FORMULA over already-graded images — the argmin
/// [`graded_meet`] delegates to, exposed for an engine that holds (memoized)
/// images and must not re-walk the adjacency to combine them. `reflexive_a` is
/// `a`'s reflexive image (`a` at hop 0 plus its strict image), `strict_b` is
/// `b`'s strict image graded by hops FROM `b`; the result is the
/// `(hops, V::Ord)`-minimal member of `strict_b` also present in
/// `reflexive_a`.
pub fn graded_meet_of<V: Ord + Clone>(
    reflexive_a: &[(V, u32)],
    strict_b: &[(V, u32)],
) -> Option<V> {
    let anc_a: BTreeSet<&V> = reflexive_a.iter().map(|(v, _)| v).collect();
    strict_b
        .iter()
        .filter(|(v, _)| anc_a.contains(v))
        .min_by(|x, y| graded_cmp(x, y))
        .map(|(v, _)| v.clone())
}

/// The ordered EVIDENCE chain `[child, …, ancestor]` when `child` reaches
/// `ancestor` (reflexively — `child == ancestor` counts), else `None`: every
/// member of `child`'s reflexive image that itself (reflexively) reaches
/// `ancestor` lies on a `child ⇝ ancestor` path, sorted by the canonical
/// `(hops from child, V::Ord)` order ([`graded_cmp`]).
pub fn graded_chain<V, I>(child: &V, ancestor: &V, neighbors: impl Fn(&V) -> I) -> Option<Vec<V>>
where
    V: Ord + Clone,
    I: IntoIterator<Item = V>,
{
    let on_path = |x: &V| x == ancestor || graded_reaches(x, ancestor, &neighbors);
    if !on_path(child) {
        return None;
    }
    let mut chain: Vec<(V, u32)> = alloc::vec![(child.clone(), 0u32)];
    chain.extend(
        graded_image(child, &neighbors)
            .into_iter()
            .filter(|(x, _)| on_path(x)),
    );
    // `graded_image` is already canonically sorted and `child` sits at hop 0,
    // but sort explicitly: the chain's order is a CONTRACT, not an accident of
    // its construction.
    chain.sort_unstable_by(graded_cmp);
    Some(chain.into_iter().map(|(v, _)| v).collect())
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::vec;
    use proptest::prelude::*;

    /// Adjacency for the tests: vertex → its neighbor list, in INSERTION order
    /// (so a test can deliberately enumerate neighbors against `V::Ord`).
    type Adj = BTreeMap<u8, Vec<u8>>;

    fn adjacency(edges: &[(u8, u8)]) -> Adj {
        let mut adj = Adj::new();
        for &(s, t) in edges {
            adj.entry(s).or_default().push(t);
        }
        adj
    }

    fn fwd(adj: &Adj) -> impl Fn(&u8) -> Vec<u8> + '_ {
        move |v| adj.get(v).cloned().unwrap_or_default()
    }

    /// The same adjacency with every neighbor list REVERSED — the
    /// enumeration-order counterfactual the determinism contract must erase.
    fn rev(adj: &Adj) -> impl Fn(&u8) -> Vec<u8> + '_ {
        move |v| {
            let mut n = adj.get(v).cloned().unwrap_or_default();
            n.reverse();
            n
        }
    }

    /// An INDEPENDENT minimal-hop oracle: iterate edge relaxation to fixpoint
    /// (Bellman–Ford shape, deliberately not a BFS), yielding every strictly
    /// reachable vertex's minimal hop count from `source`.
    fn relaxation_distances(adj: &Adj, source: u8) -> BTreeMap<u8, u32> {
        let mut dist: BTreeMap<u8, u32> = BTreeMap::new();
        dist.insert(source, 0);
        loop {
            let mut grew = false;
            for (&s, targets) in adj.iter() {
                let Some(&ds) = dist.get(&s) else { continue };
                for &t in targets {
                    let candidate = ds + 1;
                    if dist.get(&t).is_none_or(|&dt| candidate < dt) {
                        dist.insert(t, candidate);
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        dist.remove(&source);
        dist
    }

    proptest! {
        /// Over ARBITRARY directed graphs (cycles and self-loops admitted): the
        /// graded image terminates, excludes the source, carries the minimal
        /// hop counts of an independent relaxation oracle, is sorted by the
        /// canonical `(hops, V::Ord)` order, and is IDENTICAL under reversed
        /// neighbor enumeration (the determinism contract). The membership
        /// probe agrees with the image on every vertex.
        #[test]
        fn graded_image_is_canonical_minimal_and_cycle_safe(
            edges in prop::collection::vec((0u8..12, 0u8..12), 0..48),
            source in 0u8..12,
        ) {
            let adj = adjacency(&edges);
            let image = graded_image(&source, fwd(&adj));

            // Strict: the source is never in its own image, cycles included.
            prop_assert!(image.iter().all(|(v, _)| *v != source));

            // Canonically sorted — the LITERAL `(hops ascending, V ascending)`
            // order, spelled out independently of `graded_cmp` so a broken
            // comparator cannot certify itself.
            let literally_sorted = image.windows(2).all(|w| {
                let ((v1, d1), (v2, d2)) = (&w[0], &w[1]);
                d1 < d2 || (d1 == d2 && v1 < v2)
            });
            prop_assert!(literally_sorted, "image not in literal (hops, V::Ord) order");
            // …and `graded_cmp` IS that order (ties impossible on distinct vertices).
            prop_assert!(
                image.windows(2).all(|w| graded_cmp(&w[0], &w[1]) == Ordering::Less)
            );

            // Deterministic: reversing every neighbor list changes NOTHING.
            prop_assert_eq!(&image, &graded_image(&source, rev(&adj)));

            // Hop-graded: exactly the reachable set of, at the minimal hop
            // counts of, the independent relaxation oracle.
            let want = relaxation_distances(&adj, source);
            let got: BTreeMap<u8, u32> = image.iter().copied().collect();
            prop_assert_eq!(got.len(), image.len(), "image vertices are distinct");
            prop_assert_eq!(&got, &want);

            // The early-exit probe agrees with image membership everywhere.
            for v in 0u8..12 {
                prop_assert_eq!(
                    graded_reaches(&source, &v, fwd(&adj)),
                    want.contains_key(&v),
                    "probe/image divergence at {}", v
                );
            }
        }
    }
    crate::register_praxis_value!(
        graded_image_is_canonical_minimal_and_cycle_safe,
        Deterministic,
        Verifiable
    );

    /// The DAG-tie pins: a diamond with two equal-distance mid vertices whose
    /// neighbor enumeration order OPPOSES `V::Ord` — the meet must pick the
    /// `V::Ord`-minimal one and the chain must order the tied pair by `V::Ord`,
    /// never by discovery order.
    #[crate::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn meet_and_chain_break_equal_distance_ties_by_vertex_order() {
        // 0 → 7 → 9, 0 → 3 → 9 — neighbors of 0 enumerate as [7, 3], i.e.
        // AGAINST u8::Ord, so a discovery-order tie-break would answer 7.
        let adj = adjacency(&[(0, 7), (0, 3), (7, 9), (3, 9)]);

        // meet(0, 0): both mids are common ancestors at hops 1 — the tie.
        assert_eq!(
            graded_meet(&0, &0, fwd(&adj)),
            Some(3),
            "V::Ord wins the tie"
        );
        assert_eq!(
            graded_meet(&0, &0, rev(&adj)),
            Some(3),
            "the winner is enumeration-order independent"
        );

        // chain(0, 9): the tied mids order as [3, 7] within hop level 1.
        assert_eq!(graded_chain(&0, &9, fwd(&adj)), Some(vec![0, 3, 7, 9]));
        assert_eq!(graded_chain(&0, &9, rev(&adj)), Some(vec![0, 3, 7, 9]));

        // The meet formula over pre-graded images is the same argmin.
        let mut reflexive = vec![(0u8, 0u32)];
        reflexive.extend(graded_image(&0, fwd(&adj)));
        assert_eq!(
            graded_meet_of(&reflexive, &graded_image(&0, fwd(&adj))),
            Some(3)
        );
    }

    /// Reflexivity and unreachability contracts: the probe never invents a
    /// reflexive arrow (even through a cycle), the chain answers the reflexive
    /// case as the singleton, and an unreachable pair is an honest `None`.
    #[crate::praxis_value(Honest, Verifiable)]
    #[test]
    fn reflexive_and_unreachable_cases_are_honest() {
        // 0 → 1 → 2 → 0: a 3-cycle, plus an isolated 5.
        let adj = adjacency(&[(0, 1), (1, 2), (2, 0)]);

        // A cycle back to the source does not re-admit it.
        assert!(!graded_reaches(&0, &0, fwd(&adj)));
        assert!(graded_reaches(&0, &2, fwd(&adj)));
        assert!(graded_image(&0, fwd(&adj)).iter().all(|(v, _)| *v != 0));

        // The chain treats `child == ancestor` reflexively; in this CYCLE the
        // other members also (reflexively) reach 0, so they are evidence too.
        assert_eq!(graded_chain(&0, &0, fwd(&adj)), Some(vec![0, 1, 2]));

        // Unreachable: honest None, not an empty chain.
        assert_eq!(graded_chain(&5, &0, fwd(&adj)), None);
        assert_eq!(graded_meet(&5, &0, fwd(&adj)), None);
    }
}
