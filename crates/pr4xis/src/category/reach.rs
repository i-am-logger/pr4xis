//! The ONE hop-graded reachability kernel — free functions, generic over the
//! vertex, with the adjacency INJECTED.
//!
//! Every reachability engine in the workspace answers the same four questions
//! over a directed generating graph: the graded image of a vertex (its strict
//! reachable set, each member with its minimal hop count), a membership probe,
//! the lattice meet of two vertices, and the ordered evidence chain between
//! two vertices. Before this module those answers were hand-copied per engine
//! (the runtime's `MaterializedClosure`, English's `TaxonomyStore`) and
//! hand-synchronized — same algorithm token-for-token, divergent tie-breaks.
//! This module is the single home: one cycle-safe breadth-first walk, one
//! grading, ONE deterministic output order — plus the ONE generic engine over
//! it ([`ReachSubstrate`] + [`ImageMemo`] + [`ReachView`]) both those types
//! now instantiate instead of each holding a hand-rolled shell.
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
//! oracle the full-corpus `english_taxonomy_bfs` gate compares the
//! `TaxonomyStore` engine against (the runtime's `MaterializedClosure` is
//! oracle-checked at unit scale — no full-corpus materialize+closure run
//! exists since the B1-bridge deletion), and an oracle must not share its
//! subject's implementation.

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::cell::RefCell;
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

// ── the generic engine: substrate + memo policy + per-call view ──────────────
//
// The kernel above is the ALGORITHM; the three items below are the ONE generic
// ENGINE every reachability surface in the workspace instantiates. A substrate
// exposes its per-kind generating adjacency ([`ReachSubstrate`]); a memo policy
// decides whether computed images are cached ([`ImageMemo`]: [`Cached`] for the
// runtime's interior-mutable memo, [`Uncached`] for English, which must stay
// `Sync`); and a [`ReachView`] — MINTED PER CALL, never stored (a stored view
// would borrow its own engine) — binds one `(substrate, memo, kind)` triple and
// answers the graded queries by delegating to the kernel.

/// The minimal READ interface the graded-reach engine needs from an engine's
/// representation: the per-kind generating adjacency, enumerated one vertex at
/// a time. Everything else (representation, mutation, persistence) stays the
/// implementor's business — the runtime's `BTreeMap` adjacency and English's
/// zero-copy packed CSR column both answer this and nothing more.
///
/// The neighbor iterator deliberately captures ONLY the substrate borrow
/// (`use<'s, Self>`), never the `kind` / `vertex` argument lifetimes: the
/// kernel's injected `neighbors` closure must have one return type independent
/// of the per-call query vertex.
pub trait ReachSubstrate {
    /// The relation-kind axis the adjacency is partitioned by (the runtime's
    /// `ConceptRef` into the Relations vocabulary; English's `Direction`).
    type Kind: Ord + Clone;
    /// The vertex. Its total order IS the kernel's determinism tie-break
    /// (`(hops, V::Ord)` — see the module docs).
    type Vertex: Ord + Clone;
    /// The direct successors of `vertex` along `kind` — the generating edges,
    /// not any closure. An unknown kind or vertex yields the empty iterator.
    fn neighbors<'s>(
        &'s self,
        kind: &Self::Kind,
        vertex: &Self::Vertex,
    ) -> impl Iterator<Item = Self::Vertex> + use<'s, Self>;
}

/// The typed MEMO POLICY of the graded-reach engine — whether a computed
/// strict image is cached for later queries. Two implementations, one per
/// concurrency stance:
///
/// - [`Cached`] — interior-mutable (`RefCell`), `!Sync`; the runtime's policy
///   (chat / wasm are single-threaded and a queried vertex repeats).
/// - [`Uncached`] — stateless ZST, `Sync`; English's policy (`English` lives
///   in a shared `OnceLock`, and its shallow DAG makes a per-query walk as
///   cheap as a lookup).
///
/// A policy is semantically TRANSPARENT: for the same adjacency both answer
/// every query identically (the memo stores exactly the kernel's canonical
/// output); the choice is a footprint/`Sync` trade, never a semantics one.
pub trait ImageMemo<K: Ord + Clone, V: Ord + Clone> {
    /// The strict graded image of `(kind, source)` — a memo hit, or
    /// `compute()` (the kernel walk), stored per policy.
    fn image(&self, kind: &K, source: &V, compute: impl FnOnce() -> Vec<(V, u32)>)
    -> Vec<(V, u32)>;

    /// Does `source` strictly reach `target`? — [`Cached`] scans its stored
    /// image without cloning it out (computing + storing on a miss, so the
    /// probe warms the memo); [`Uncached`] delegates to `probe` (the kernel's
    /// early-exit walk, [`graded_reaches`] — no image is materialized).
    fn reaches(
        &self,
        kind: &K,
        source: &V,
        target: &V,
        compute: impl FnOnce() -> Vec<(V, u32)>,
        probe: impl FnOnce() -> bool,
    ) -> bool;
}

/// The CACHING memo policy — computed strict images are stored and every later
/// query for the same `(kind, source)` hits the memo with no re-walk.
///
/// The memo is NESTED PER KIND (`kind → source → image`), NOT keyed by a flat
/// `(kind, source)` pair: the kind key is cloned once when its per-kind map is
/// first created, never once per memoized source — the runtime's kind is a
/// `ConceptRef` (two owned strings), and a per-source clone of it would tax
/// every first query.
///
/// `RefCell` interior mutability keeps the query surface `&self` and makes any
/// holder `!Sync` — the runtime's DELIBERATE single-threaded invariant (see
/// `MaterializedClosure`); an engine that must stay `Sync` uses [`Uncached`].
/// The cache is a DERIVED view of the substrate's adjacency: a holder that
/// mutates its adjacency (the runtime's `union`) MUST [`clear`](Self::clear)
/// it, and it never takes part in the holder's identity/equality.
#[derive(Debug, Clone)]
pub struct Cached<K: Ord + Clone, V: Ord + Clone> {
    /// `kind → source → [(descendant, min hops)]` — exactly the kernel's
    /// canonical [`graded_image`] output, per queried source.
    images: RefCell<MemoImages<K, V>>,
}

/// The nested per-kind memo shape — `kind → source → [(descendant, min hops)]`,
/// each image being exactly the kernel's canonical [`graded_image`] output.
type MemoImages<K, V> = BTreeMap<K, BTreeMap<V, Vec<(V, u32)>>>;

/// Manual (a derive would demand `K: Default + V: Default`, which map KEYS
/// never need): the default is simply the empty memo.
impl<K: Ord + Clone, V: Ord + Clone> Default for Cached<K, V> {
    fn default() -> Self {
        Self {
            images: RefCell::new(BTreeMap::new()),
        }
    }
}

impl<K: Ord + Clone, V: Ord + Clone> Cached<K, V> {
    /// Drop every memoized image — REQUIRED after any adjacency mutation
    /// (the memo is derived from the adjacency; stale entries would answer
    /// the pre-mutation graph).
    pub fn clear(&self) {
        self.images.borrow_mut().clear();
    }

    /// Store `image` under `(kind, source)`, cloning the kind key only when
    /// its per-kind map does not exist yet.
    fn store(&self, kind: &K, source: &V, image: Vec<(V, u32)>) {
        let mut memo = self.images.borrow_mut();
        if !memo.contains_key(kind) {
            memo.insert(kind.clone(), BTreeMap::new());
        }
        memo.get_mut(kind)
            .expect("per-kind memo map was just ensured present")
            .insert(source.clone(), image);
    }
}

impl<K: Ord + Clone, V: Ord + Clone> ImageMemo<K, V> for Cached<K, V> {
    fn image(
        &self,
        kind: &K,
        source: &V,
        compute: impl FnOnce() -> Vec<(V, u32)>,
    ) -> Vec<(V, u32)> {
        if let Some(hit) = self.images.borrow().get(kind).and_then(|m| m.get(source)) {
            return hit.clone();
        }
        let image = compute();
        self.store(kind, source, image.clone());
        image
    }

    fn reaches(
        &self,
        kind: &K,
        source: &V,
        target: &V,
        compute: impl FnOnce() -> Vec<(V, u32)>,
        _probe: impl FnOnce() -> bool,
    ) -> bool {
        // Membership only: borrow the cached image and scan WITHOUT cloning —
        // a full-image clone on the hot is-a path would be waste.
        if let Some(hit) = self.images.borrow().get(kind).and_then(|m| m.get(source)) {
            return hit.iter().any(|(v, _)| v == target);
        }
        let image = compute();
        let found = image.iter().any(|(v, _)| v == target);
        self.store(kind, source, image);
        found
    }
}

/// The STATELESS memo policy — nothing is stored, every query re-walks the
/// generators. A ZST, trivially `Sync`: the policy for an engine held in a
/// shared `OnceLock` (English), where interior mutability is not an option and
/// the graph is shallow enough that a walk costs what a lookup would.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Uncached;

impl<K: Ord + Clone, V: Ord + Clone> ImageMemo<K, V> for Uncached {
    fn image(
        &self,
        _kind: &K,
        _source: &V,
        compute: impl FnOnce() -> Vec<(V, u32)>,
    ) -> Vec<(V, u32)> {
        compute()
    }

    fn reaches(
        &self,
        _kind: &K,
        _source: &V,
        _target: &V,
        _compute: impl FnOnce() -> Vec<(V, u32)>,
        probe: impl FnOnce() -> bool,
    ) -> bool {
        probe()
    }
}

/// One `(substrate, memo, kind)` binding of the generic engine — the object
/// that answers the graded queries. MINTED PER CALL by its engine (a struct
/// field would borrow the engine's own adjacency and memo — a self-borrow);
/// it is three shared references, free to construct.
pub struct ReachView<'a, S: ReachSubstrate, M: ImageMemo<S::Kind, S::Vertex>> {
    substrate: &'a S,
    memo: &'a M,
    kind: &'a S::Kind,
}

impl<'a, S: ReachSubstrate, M: ImageMemo<S::Kind, S::Vertex>> ReachView<'a, S, M> {
    /// Bind a substrate, a memo policy and one relation kind for the duration
    /// of a query.
    pub fn new(substrate: &'a S, memo: &'a M, kind: &'a S::Kind) -> Self {
        Self {
            substrate,
            memo,
            kind,
        }
    }

    /// The kernel walk over the substrate's adjacency — the pure computation
    /// the memo policy stores or passes through.
    fn compute_image(&self, source: &S::Vertex) -> Vec<(S::Vertex, u32)> {
        graded_image(source, |v: &S::Vertex| {
            self.substrate.neighbors(self.kind, v)
        })
    }

    /// The STRICT graded image of `source` along the bound kind
    /// ([`graded_image`], via the memo policy).
    pub fn strict_image(&self, source: &S::Vertex) -> Vec<(S::Vertex, u32)> {
        self.memo
            .image(self.kind, source, || self.compute_image(source))
    }

    /// The reflexive image — `source` at hop 0 plus its strict image.
    pub fn reflexive_image(&self, source: &S::Vertex) -> Vec<(S::Vertex, u32)> {
        let mut out = alloc::vec![(source.clone(), 0u32)];
        out.extend(self.strict_image(source));
        out
    }

    /// Does `source` STRICTLY reach `target`? The reflexive `source == target`
    /// case is the CALLER's per-relation decision, exactly as in the kernel.
    pub fn reaches(&self, source: &S::Vertex, target: &S::Vertex) -> bool {
        self.memo.reaches(
            self.kind,
            source,
            target,
            || self.compute_image(source),
            || {
                graded_reaches(source, target, |v: &S::Vertex| {
                    self.substrate.neighbors(self.kind, v)
                })
            },
        )
    }

    /// The lattice meet of `a` and `b` — the kernel's argmin formula
    /// ([`graded_meet_of`]) over this view's (possibly memoized) images.
    pub fn meet(&self, a: &S::Vertex, b: &S::Vertex) -> Option<S::Vertex> {
        graded_meet_of(&self.reflexive_image(a), &self.strict_image(b))
    }

    /// The ordered evidence chain `[child, …, ancestor]` when `child`
    /// (reflexively) reaches `ancestor`, else `None` — [`graded_chain`]'s
    /// contract, evaluated through the memo policy so a caching engine's chain
    /// keeps hitting (and warming) its memo.
    pub fn chain(&self, child: &S::Vertex, ancestor: &S::Vertex) -> Option<Vec<S::Vertex>> {
        if child != ancestor && !self.reaches(child, ancestor) {
            return None;
        }
        // Reflexive ancestors of `child` that themselves (reflexively) reach
        // `ancestor` lie on a child ⇝ ancestor path; canonical order.
        let mut chain: Vec<(S::Vertex, u32)> = self
            .reflexive_image(child)
            .into_iter()
            .filter(|(x, _)| x == ancestor || self.reaches(x, ancestor))
            .collect();
        chain.sort_unstable_by(graded_cmp);
        Some(chain.into_iter().map(|(v, _)| v).collect())
    }
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

    // ── the generic engine (substrate + memo policy + view) ──────────────────

    /// A witness substrate: `kind → vertex → neighbors`, insertion-ordered
    /// neighbor lists (so a witness can enumerate against `V::Ord`).
    struct MapSubstrate {
        adj: BTreeMap<u8, Adj>,
    }

    impl ReachSubstrate for MapSubstrate {
        type Kind = u8;
        type Vertex = u8;
        fn neighbors<'s>(&'s self, kind: &u8, vertex: &u8) -> impl Iterator<Item = u8> + use<'s> {
            self.adj
                .get(kind)
                .and_then(|per_kind| per_kind.get(vertex))
                .map(|targets| targets.as_slice())
                .unwrap_or(&[])
                .iter()
                .copied()
        }
    }

    /// The MEMO-TRANSPARENCY pin: over the same substrate, a [`Cached`] view
    /// and an [`Uncached`] view answer every graded query identically, and
    /// identically to the kernel free functions — the policy is a
    /// footprint/`Sync` trade, never a semantics one. Includes a repeat query
    /// (the memoized second answer must equal the first) and the DAG tie case.
    #[crate::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn memo_policy_is_semantically_transparent() {
        // The diamond with anti-Ord enumeration plus a second, disjoint kind —
        // so the per-kind partitioning is exercised too.
        let mut adj = BTreeMap::new();
        adj.insert(0u8, adjacency(&[(0, 7), (0, 3), (7, 9), (3, 9)]));
        adj.insert(1u8, adjacency(&[(0, 5)]));
        let substrate = MapSubstrate { adj };

        let cached: Cached<u8, u8> = Cached::default();
        for kind in [0u8, 1u8] {
            let warm = ReachView::new(&substrate, &cached, &kind);
            let cold = ReachView::new(&substrate, &Uncached, &kind);
            let kernel_neighbors = |v: &u8| substrate.neighbors(&kind, v);
            for v in 0u8..10 {
                // Image: cached == uncached == kernel, and the memoized repeat
                // answer is identical to the first.
                let want = graded_image(&v, kernel_neighbors);
                assert_eq!(warm.strict_image(&v), want, "cached image, kind {kind}");
                assert_eq!(warm.strict_image(&v), want, "memoized repeat, kind {kind}");
                assert_eq!(cold.strict_image(&v), want, "uncached image, kind {kind}");
                for w in 0u8..10 {
                    assert_eq!(
                        warm.reaches(&v, &w),
                        graded_reaches(&v, &w, kernel_neighbors),
                        "cached probe ({v} ⇝ {w}), kind {kind}"
                    );
                    assert_eq!(warm.reaches(&v, &w), cold.reaches(&v, &w));
                    assert_eq!(warm.meet(&v, &w), graded_meet(&v, &w, kernel_neighbors));
                    assert_eq!(warm.meet(&v, &w), cold.meet(&v, &w));
                    assert_eq!(
                        warm.chain(&v, &w),
                        graded_chain(&v, &w, kernel_neighbors),
                        "cached chain ({v} ⇝ {w}), kind {kind}"
                    );
                    assert_eq!(warm.chain(&v, &w), cold.chain(&v, &w));
                }
            }
        }
    }

    /// The DERIVED-CACHE pin: after the substrate's adjacency grows, a
    /// [`Cached`] memo still answers the OLD graph until [`Cached::clear`] —
    /// which is exactly why an engine that mutates its adjacency (the
    /// runtime's `MaterializedClosure::union`) MUST invalidate. Asserts both
    /// halves: the staleness (honest — the memo does not watch the substrate)
    /// and the recovery after `clear()`.
    #[crate::praxis_value(Honest, Verifiable)]
    #[test]
    fn cached_clear_drops_stale_images_after_adjacency_growth() {
        let kind = 0u8;
        let mut adj = BTreeMap::new();
        adj.insert(kind, adjacency(&[(0, 1)]));
        let mut substrate = MapSubstrate { adj };
        let memo: Cached<u8, u8> = Cached::default();

        // Warm the memo on the small graph: 0 reaches only 1.
        assert_eq!(
            ReachView::new(&substrate, &memo, &kind).strict_image(&0),
            vec![(1, 1)]
        );

        // The adjacency grows: 1 → 2.
        substrate
            .adj
            .get_mut(&kind)
            .expect("kind 0 present")
            .entry(1)
            .or_default()
            .push(2);

        // WITHOUT clear(): the memo honestly answers the pre-growth graph.
        assert_eq!(
            ReachView::new(&substrate, &memo, &kind).strict_image(&0),
            vec![(1, 1)],
            "a derived cache does not watch its substrate — this is the staleness clear() exists for"
        );

        // WITH clear(): the next query re-walks the enlarged generators.
        memo.clear();
        assert_eq!(
            ReachView::new(&substrate, &memo, &kind).strict_image(&0),
            vec![(1, 1), (2, 2)],
            "clear() must drop the stale image so the union'd graph is seen"
        );
    }
}
