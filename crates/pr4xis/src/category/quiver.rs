//! Quivers and the **free category** on a quiver.
//!
//! A quiver (directed multigraph) freely generates a category whose objects are
//! its vertices and whose morphisms are *paths* of edges, with composition =
//! path concatenation and identities = empty paths (Mac Lane 1971 CWM II.7).
//! This is the construction that makes "the runtime is determined by the
//! ontology" precise: by the free–forgetful universal property, an assignment of
//! each generating edge to a morphism of a target category extends to a **unique**
//! functor out of the free category ([`FreeExtension`]). Nothing about the
//! extension is a free choice — it is forced by the generators.
//!
//! # Finitely-generated representation
//!
//! Under a cycle (e.g. `a → b → a`) the set of paths is infinite, which praxis's
//! closed-world [`Category::morphisms`] (a finite enumeration) cannot list. So,
//! consistent with praxis's finitely-generated stance (see
//! [`FinitelyGenerated`]), [`FreeCategory`]
//! represents itself by its finite **generating set** — identities plus the
//! single edges — returned from [`Category::morphisms`]. This is sound for the
//! one thing we verify on a free category: a functor out of it is determined by,
//! and correct iff correct on, its action on generators (the universal property).
//! [`functor_law_axioms`](super::laws::functor_law_axioms) therefore check
//! [`FreeExtension`] correctly on this generating set.
//!
//! Note: [`ClosureLaw`](super::laws::ClosureLaw) does **not** apply to a free
//! category — composing two generators yields a length-2 path, deliberately
//! outside the generating set. Free categories are closed by construction; their
//! law content lives in the functors out of them. Verifying the full
//! (infinite) path category awaits praxis's planned open-world relaxation.
//!
//! Literature:
//! - Mac Lane (1971) *Categories for the Working Mathematician* II.7 — free
//!   categories on a graph; the free–forgetful adjunction Grph ⊣ Cat.
//! - Fong & Spivak (2019) *Seven Sketches in Compositionality* Ch. 3 — finitely
//!   presented categories via generators and relations.

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};
use core::hash::Hash;
use core::marker::PhantomData;

use hashbrown::{HashMap, HashSet};

use super::arrow::Arrow;
use super::category::Category;
use super::entity::{Concept, FinitelyGenerated};
use super::functor::Functor;
use super::kinds::FunctorKind;

/// A quiver: a finite directed multigraph. Vertices are a [`Concept`]'s
/// variants; [`Quiver::edges`] are the finite generating arrows.
pub trait Quiver {
    /// The vertices (objects of the free category).
    type Vertex: Concept;
    /// The generating edges (single-step arrows).
    type Edge: Arrow<Object = Self::Vertex>;

    /// All generating edges of the quiver.
    fn edges() -> Vec<Self::Edge>;
}

/// A path in quiver `Q`: a chain of edges from `source` to `target`. An empty
/// path (no edges, `source == target`) is an identity morphism.
///
/// The trait impls are hand-written (not derived) so their bounds fall on the
/// associated types actually stored (`Q::Vertex`, `Q::Edge`) rather than on the
/// quiver marker `Q` itself.
pub struct Path<Q: Quiver> {
    source: Q::Vertex,
    target: Q::Vertex,
    edges: Vec<Q::Edge>,
}

impl<Q: Quiver> Clone for Path<Q> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            target: self.target.clone(),
            edges: self.edges.clone(),
        }
    }
}

impl<Q: Quiver> core::fmt::Debug for Path<Q> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Path")
            .field("source", &self.source)
            .field("target", &self.target)
            .field("edges", &self.edges)
            .finish()
    }
}

impl<Q: Quiver> PartialEq for Path<Q> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.target == other.target && self.edges == other.edges
    }
}

impl<Q: Quiver> Eq for Path<Q> {}

impl<Q: Quiver> Path<Q> {
    /// The empty (identity) path at a vertex.
    pub fn empty(at: Q::Vertex) -> Self {
        Self {
            source: at.clone(),
            target: at,
            edges: Vec::new(),
        }
    }

    /// A single-edge path (a generator).
    pub fn edge(e: Q::Edge) -> Self {
        Self {
            source: e.source(),
            target: e.target(),
            edges: vec![e],
        }
    }

    /// The edges traversed (empty for an identity).
    pub fn edges(&self) -> &[Q::Edge] {
        &self.edges
    }

    /// The path length (number of edges).
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Whether this is an identity (empty) path.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
}

impl<Q: Quiver> Arrow for Path<Q> {
    type Object = Q::Vertex;
    type Kind = ();

    fn source(&self) -> Q::Vertex {
        self.source.clone()
    }

    fn target(&self) -> Q::Vertex {
        self.target.clone()
    }

    fn kind(&self) {}
}

/// The free category on a quiver `Q` (Mac Lane 1971 CWM II.7).
///
/// Objects are vertices, morphisms are [`Path`]s, composition is concatenation,
/// identities are empty paths. See the module docs for how the infinite path set
/// is represented by its finite generating set.
pub struct FreeCategory<Q>(PhantomData<Q>);

impl<Q: Quiver> Category for FreeCategory<Q>
where
    // The free category lists its generating set (identities at each vertex plus
    // the single edges) — `morphisms()` enumerates the vertices, which requires
    // the vertex concept to be finitely generated (closed-world). The quiver's
    // `Vertex: Concept` stays open-world (identities/composition need no
    // enumeration); only the generating-set listing needs this.
    Q::Vertex: FinitelyGenerated,
{
    type Object = Q::Vertex;
    type Morphism = Path<Q>;

    fn identity(obj: &Q::Vertex) -> Path<Q> {
        Path::empty(obj.clone())
    }

    fn compose(f: &Path<Q>, g: &Path<Q>) -> Option<Path<Q>> {
        if f.target != g.source {
            return None;
        }
        let mut edges = f.edges.clone();
        edges.extend(g.edges.iter().cloned());
        Some(Path {
            source: f.source.clone(),
            target: g.target.clone(),
            edges,
        })
    }

    /// The finite generating set: identities at each vertex plus the single
    /// edges. (The full path set is infinite under cycles — see module docs.)
    fn morphisms() -> Vec<Path<Q>> {
        let mut ms: Vec<Path<Q>> = Q::Vertex::variants().into_iter().map(Path::empty).collect();
        ms.extend(Q::edges().into_iter().map(Path::edge));
        ms
    }
}

/// A labeling of a quiver's vertices and edges into a target category —
/// equivalently a quiver morphism `Q → U(D)` into the underlying graph of `D`.
///
/// By the free–forgetful universal property it extends to a **unique** functor
/// [`FreeExtension`] `FreeCategory<Q> → D` (Mac Lane 1971 CWM II.7). The
/// edge images must compose along any path of `Q` (they do automatically when
/// `D` is total, e.g. a one-object effect category).
pub trait QuiverInterpretation {
    /// The quiver being interpreted.
    type Quiver: Quiver;
    /// The target category the generators are interpreted into.
    type Target: Category;

    /// Where a vertex goes in the target category.
    fn on_vertex(v: &<Self::Quiver as Quiver>::Vertex) -> <Self::Target as Category>::Object;

    /// Where a generating edge goes in the target category.
    fn on_edge(e: &<Self::Quiver as Quiver>::Edge) -> <Self::Target as Category>::Morphism;
}

/// The unique functor extending a [`QuiverInterpretation`] over the free
/// category — the free–forgetful universal property in action. Maps a path by
/// folding its edge images through the target's composition.
pub struct FreeExtension<I>(PhantomData<I>);

impl<I: QuiverInterpretation> Functor for FreeExtension<I>
where
    // `Source = FreeCategory<I::Quiver>` is only a `Category` when the quiver's
    // vertices are finitely generated (it lists its generating set). The functor
    // out of the free category inherits that requirement.
    <I::Quiver as Quiver>::Vertex: FinitelyGenerated,
{
    type Source = FreeCategory<I::Quiver>;
    type Target = I::Target;

    fn map_object(v: &<I::Quiver as Quiver>::Vertex) -> <I::Target as Category>::Object {
        I::on_vertex(v)
    }

    fn map_morphism(path: &Path<I::Quiver>) -> <I::Target as Category>::Morphism {
        let mut acc = I::Target::identity(&I::on_vertex(&path.source));
        for e in &path.edges {
            let img = I::on_edge(e);
            acc = I::Target::compose(&acc, &img)
                .expect("QuiverInterpretation edge images must compose along the path");
        }
        acc
    }

    const KIND: FunctorKind = FunctorKind::Free;

    crate::relationship_meta!(
        "FreeExtension",
        "the unique functor extending a quiver interpretation over the free category (free-forgetful universal property)",
        "Mac Lane (1971) Categories for the Working Mathematician II.7"
    );
}

/// The **materialized** reachability image of a directed relation over a vertex
/// type `V` — the data-structure realization of the `FreeCategory<Q> → ReachCat`
/// reflection ([`Collapse`](self) in the tests above): every generating path
/// collapses to its `(source, target)` reachability arrow, and the whole
/// reflexive-transitive image is folded ONCE, here, so that every later query is
/// an O(1) membership lookup rather than a per-query traversal.
///
/// This is the runtime/loaded analogue of the `ontology!` macro's compile-time
/// Floyd-Warshall and the shared engine behind
/// `pr4xis-runtime`'s `MaterializedClosure` (which folds it per relation-kind
/// `ConceptRef` over `ConceptRef`) and the English hypernym closure (which folds it over WordNet
/// `ConceptId`s). It is keyed by `Hash + Eq` so dense index vertices (a
/// `Reference<4>` synset id) and open-world named vertices (a `(ontology, name)`
/// pair) reuse the SAME construct without either paying for the other's identity
/// scheme.
///
/// # What is stored
///
/// For each vertex `v`, the set of vertices reachable from `v` along the relation
/// (its strict descendants under the closure), each tagged with the **minimal**
/// number of generating edges on a path to it. The grading is the path-length of
/// the free category collapsed into `ReachCat`; it is what makes a *nearest*
/// query (a lattice meet) answerable from the materialized set without re-walking
/// the generators. The reflexive arrow `v → v` (distance 0) is implicit — every
/// vertex reaches itself — and is added by [`reaches`](Self::reaches) /
/// [`reflexive_image`](Self::reflexive_image) rather than stored.
///
/// Literature:
/// - Mac Lane (1971) *CWM* II.7 — the free category on a quiver and the unique
///   functor into the thin reachability category; this closure IS that functor's
///   image, made concrete.
/// - Warshall (1962) "A Theorem on Boolean Matrices" — transitive closure as the
///   least fixpoint of "compose one more generator"; the fold below is that
///   fixpoint, carrying the shortest-path grading (Floyd 1962).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachabilityClosure<V: Eq + Hash + Clone> {
    /// `source → {target → min edges on a source⇝target path}`. The reflexive
    /// `source → source` (distance 0) arrow is implicit, never stored.
    reachable: HashMap<V, HashMap<V, u32>>,
}

// Hand-written so the empty closure is `Default` for ANY vertex type `V`, not
// only `V: Default` (a derived `Default` would wrongly require `V: Default`).
impl<V: Eq + Hash + Clone> Default for ReachabilityClosure<V> {
    fn default() -> Self {
        Self {
            reachable: HashMap::new(),
        }
    }
}

impl<V: Eq + Hash + Clone> ReachabilityClosure<V> {
    /// Fold the reflexive-transitive reachability image from the GENERATING edges
    /// `(source, target)` — the categorical free-functor image into the thin
    /// reachability category. This is **materialization**, not query: the
    /// transitive closure is saturated here, once, so every later query is an
    /// O(1) lookup. We always fold from the generators; a pre-closed input is
    /// idempotent (the closure of a closure is the same closure), so the fold is
    /// correct regardless of whether `edges` is already transitively closed.
    ///
    /// The fold is a least-fixpoint of "compose one more generator" (Warshall
    /// 1962), carrying the shortest path length to each reachable vertex (Floyd
    /// 1962) so a *nearest* query is answerable from the materialized set.
    pub fn fold(edges: impl IntoIterator<Item = (V, V)>) -> Self {
        // Direct generating image: source → {direct target: 1 edge}.
        let mut reachable: HashMap<V, HashMap<V, u32>> = HashMap::new();
        for (source, target) in edges {
            // A direct generator is one edge; a self-loop generator carries no
            // information beyond the implicit reflexive arrow, so skip it.
            if source == target {
                continue;
            }
            reachable
                .entry(source)
                .or_default()
                .entry(target)
                .or_insert(1);
        }

        // Saturate to the transitive closure. The functor's action on a composite
        // path is the composition of the targets' images; folding to a fixpoint
        // recovers the whole closure. Distances combine additively and we keep the
        // minimum (shortest path). We re-fold over a snapshot to extend the map
        // while iterating it.
        loop {
            let mut grew = false;
            let sources: Vec<V> = reachable.keys().cloned().collect();
            for source in &sources {
                // The mids reachable from `source` so far, with their distances.
                let mids: Vec<(V, u32)> = reachable
                    .get(source)
                    .map(|m| m.iter().map(|(t, &d)| (t.clone(), d)).collect())
                    .unwrap_or_default();
                for (mid, d_source_mid) in mids {
                    // One composition step further: mid's own reachable set.
                    let mid_targets: Vec<(V, u32)> = reachable
                        .get(&mid)
                        .map(|m| m.iter().map(|(t, &d)| (t.clone(), d)).collect())
                        .unwrap_or_default();
                    if mid_targets.is_empty() {
                        continue;
                    }
                    let set = reachable.entry(source.clone()).or_default();
                    for (t, d_mid_t) in mid_targets {
                        // Skip the trivial self-loop the fold would otherwise
                        // introduce; reachability here is the strict-descendant
                        // set (the reflexive arrow is implicit).
                        if &t == source {
                            continue;
                        }
                        let dist = d_source_mid.saturating_add(d_mid_t);
                        match set.get(&t) {
                            Some(&existing) if existing <= dist => {}
                            Some(_) => {
                                set.insert(t, dist);
                                grew = true;
                            }
                            None => {
                                set.insert(t, dist);
                                grew = true;
                            }
                        }
                    }
                }
            }
            if !grew {
                break;
            }
        }

        Self { reachable }
    }

    /// Does `source` reach `target` along the closure? Reflexive: every vertex
    /// reaches itself. An O(1) membership lookup, never a traversal.
    pub fn reaches(&self, source: &V, target: &V) -> bool {
        source == target
            || self
                .reachable
                .get(source)
                .is_some_and(|m| m.contains_key(target))
    }

    /// The minimal number of generating edges on a `source ⇝ target` path, or
    /// `None` if `target` is not reachable from `source`. Zero for the reflexive
    /// `source == target` case.
    pub fn distance(&self, source: &V, target: &V) -> Option<u32> {
        if source == target {
            return Some(0);
        }
        self.reachable
            .get(source)
            .and_then(|m| m.get(target))
            .copied()
    }

    /// The reflexive reachability image of `source` — `source` itself plus every
    /// strict descendant, each paired with its minimal distance (0 for `source`).
    /// A lookup over the materialized set, never a re-derivation. Order is
    /// unspecified; callers that need a deterministic order sort by the returned
    /// distance.
    pub fn reflexive_image(&self, source: &V) -> Vec<(V, u32)> {
        let mut out = vec![(source.clone(), 0u32)];
        if let Some(m) = self.reachable.get(source) {
            out.extend(m.iter().map(|(t, &d)| (t.clone(), d)));
        }
        out
    }

    /// The strict reachability image of `source` — its descendants under the
    /// closure (excluding `source`), each with its minimal distance.
    pub fn strict_image(&self, source: &V) -> Vec<(V, u32)> {
        self.reachable
            .get(source)
            .map(|m| m.iter().map(|(t, &d)| (t.clone(), d)).collect())
            .unwrap_or_default()
    }

    /// Every materialized `(source, target)` reachability pair — the closed edge
    /// set. Folding this back is idempotent (closure of a closure), so it is the
    /// canonical way to UNION two closures: chain the pairs and re-`fold`.
    pub fn edges_iter(&self) -> impl Iterator<Item = (V, V)> + '_ {
        self.reachable.iter().flat_map(|(source, targets)| {
            targets
                .keys()
                .map(move |target| (source.clone(), target.clone()))
        })
    }

    /// The **lattice meet** of `a` and `b` over this closure — the nearest vertex
    /// `m` in `strict_image(b) ∩ reflexive_image(a)`, ranked by distance from `b`
    /// (nearest first), with ties broken by `tie_key`. This is the categorical
    /// "nearest common upper bound under the relation" (a greatest-lower-bound in
    /// the dual order of the is-a poset), computed as an argmin over the
    /// materialized image — NOT a re-derivation.
    ///
    /// `b`'s strict image is used (a common ancestor sits strictly above `b`'s
    /// own level), while `a`'s reflexive image is used (so when `a` is itself an
    /// ancestor of `b`, `a` is a valid meet). This reproduces a nearest-from-`b`
    /// ascent tested against the ancestor set of `a`, without the ascent.
    pub fn meet_by<K: Ord>(&self, a: &V, b: &V, tie_key: impl Fn(&V) -> K) -> Option<V> {
        let anc_a: HashSet<V> = self
            .reflexive_image(a)
            .into_iter()
            .map(|(v, _)| v)
            .collect();
        self.strict_image(b)
            .into_iter()
            .filter(|(v, _)| anc_a.contains(v))
            .min_by(|(v1, d1), (v2, d2)| d1.cmp(d2).then_with(|| tie_key(v1).cmp(&tie_key(v2))))
            .map(|(v, _)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_functor_laws;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum V {
        A,
        B,
    }
    impl Concept for V {}
    impl FinitelyGenerated for V {
        fn variants() -> Vec<Self> {
            vec![V::A, V::B]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct E {
        from: V,
        to: V,
        label: char,
    }
    impl Arrow for E {
        type Object = V;
        type Kind = ();
        fn source(&self) -> V {
            self.from
        }
        fn target(&self) -> V {
            self.to
        }
        fn kind(&self) {}
    }

    /// The 2-cycle quiver: f: A→B, g: B→A. Its free category is infinite.
    struct TwoCycle;
    impl Quiver for TwoCycle {
        type Vertex = V;
        type Edge = E;
        fn edges() -> Vec<E> {
            vec![
                E {
                    from: V::A,
                    to: V::B,
                    label: 'f',
                },
                E {
                    from: V::B,
                    to: V::A,
                    label: 'g',
                },
            ]
        }
    }

    // Target: the thin "reachability" category on V (all pairs, transitivity).
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Reach {
        from: V,
        to: V,
    }
    impl Arrow for Reach {
        type Object = V;
        type Kind = ();
        fn source(&self) -> V {
            self.from
        }
        fn target(&self) -> V {
            self.to
        }
        fn kind(&self) {}
    }
    struct ReachCat;
    impl Category for ReachCat {
        type Object = V;
        type Morphism = Reach;
        fn identity(o: &V) -> Reach {
            Reach { from: *o, to: *o }
        }
        fn compose(f: &Reach, g: &Reach) -> Option<Reach> {
            if f.to != g.from {
                None
            } else {
                Some(Reach {
                    from: f.from,
                    to: g.to,
                })
            }
        }
        fn morphisms() -> Vec<Reach> {
            let vs = V::variants();
            vs.iter()
                .flat_map(|&a| vs.iter().map(move |&b| Reach { from: a, to: b }))
                .collect()
        }
    }

    /// The reflection: collapse each path to its (source, target) reachability
    /// arrow. The unique functor extending edge ↦ its reachability arrow.
    struct Collapse;
    impl QuiverInterpretation for Collapse {
        type Quiver = TwoCycle;
        type Target = ReachCat;
        fn on_vertex(v: &V) -> V {
            *v
        }
        fn on_edge(e: &E) -> Reach {
            Reach {
                from: e.from,
                to: e.to,
            }
        }
    }
    type Collapsed = FreeExtension<Collapse>;

    #[test]
    fn compose_concatenates_paths() {
        let f = Path::<TwoCycle>::edge(E {
            from: V::A,
            to: V::B,
            label: 'f',
        });
        let g = Path::<TwoCycle>::edge(E {
            from: V::B,
            to: V::A,
            label: 'g',
        });
        let fg = FreeCategory::<TwoCycle>::compose(&f, &g).expect("f then g composes");
        assert_eq!(fg.len(), 2);
        assert_eq!(fg.source(), V::A);
        assert_eq!(fg.target(), V::A);
        // f then f is not composable (B != A).
        assert!(FreeCategory::<TwoCycle>::compose(&f, &f).is_none());
    }

    #[test]
    fn identity_is_the_empty_path() {
        let id_a = FreeCategory::<TwoCycle>::identity(&V::A);
        assert!(id_a.is_empty());
        let f = Path::<TwoCycle>::edge(E {
            from: V::A,
            to: V::B,
            label: 'f',
        });
        let id_b = FreeCategory::<TwoCycle>::identity(&V::B);
        assert_eq!(
            FreeCategory::<TwoCycle>::compose(&id_a, &f).as_ref(),
            Some(&f)
        );
        assert_eq!(
            FreeCategory::<TwoCycle>::compose(&f, &id_b).as_ref(),
            Some(&f)
        );
    }

    #[test]
    fn free_extension_satisfies_functor_laws() {
        // Identity + composition laws, checked on the generating set — sound by
        // the universal property (module docs).
        assert_functor_laws::<Collapsed>();
    }

    #[test]
    fn free_extension_folds_paths_through_the_target() {
        let f = Path::<TwoCycle>::edge(E {
            from: V::A,
            to: V::B,
            label: 'f',
        });
        let g = Path::<TwoCycle>::edge(E {
            from: V::B,
            to: V::A,
            label: 'g',
        });
        let fg = FreeCategory::<TwoCycle>::compose(&f, &g).unwrap();
        // The path A→B→A collapses to the reachability arrow A→A.
        assert_eq!(
            Collapsed::map_morphism(&fg),
            Reach {
                from: V::A,
                to: V::A
            }
        );
    }

    // ---- ReachabilityClosure: the materialized reflection, as data ----

    #[test]
    fn closure_materializes_the_transitive_image_once() {
        // A linear taxonomy: dog → mammal → animal (is-a chain). The closure
        // folds dog ⇝ {mammal(1), animal(2)} ONCE; querying is a lookup.
        let c = ReachabilityClosure::fold([(0u32, 1u32), (1, 2)]);
        // Direct generator.
        assert!(c.reaches(&0, &1));
        // Transitive — the fold composed dog→mammal→animal into dog⇝animal.
        assert!(c.reaches(&0, &2));
        assert_eq!(c.distance(&0, &2), Some(2));
        assert_eq!(c.distance(&0, &1), Some(1));
        // Reflexive arrow is implicit, distance 0.
        assert!(c.reaches(&0, &0));
        assert_eq!(c.distance(&0, &0), Some(0));
        // Not reachable upward.
        assert!(!c.reaches(&2, &0));
        assert_eq!(c.distance(&2, &0), None);
    }

    #[test]
    fn closure_reflexive_image_includes_self_and_descendants() {
        let c = ReachabilityClosure::fold([(0u32, 1u32), (1, 2)]);
        let mut img: Vec<u32> = c.reflexive_image(&0).into_iter().map(|(v, _)| v).collect();
        img.sort_unstable();
        assert_eq!(img, vec![0, 1, 2]); // self + mammal + animal
        // A sibling/unrelated vertex is not in the image.
        assert!(!c.reflexive_image(&0).iter().any(|(v, _)| *v == 9));
    }

    #[test]
    fn closure_meet_is_the_nearest_shared_target() {
        // dog(0)→mammal(2)→animal(3); cat(1)→mammal(2)→animal(3). The meet of
        // dog and cat is mammal (distance 1 from cat), not animal (distance 2).
        let c = ReachabilityClosure::fold([(0u32, 2u32), (2, 3), (1, 2)]);
        assert_eq!(c.meet_by(&0, &1, |v| *v), Some(2));
        // When a is itself an ancestor of b, a's reflexive image makes a the meet.
        assert_eq!(c.meet_by(&2, &1, |v| *v), Some(2));
        // No shared ancestor → None.
        let d = ReachabilityClosure::fold([(0u32, 1u32), (5u32, 6u32)]);
        assert_eq!(d.meet_by(&0, &5, |v| *v), None);
    }

    #[test]
    fn closure_of_a_closure_is_set_idempotent() {
        // Folding an already-transitively-closed edge set yields the SAME
        // reachability SET as folding only the generators (closure of a closure
        // = closure). This is the idempotence law materialization relies on: a
        // `.prx` whose stored edges are already a closure re-folds to the same
        // reachable set.
        let closed = [(0u32, 1u32), (1, 2), (0, 2)];
        let c = ReachabilityClosure::fold(closed);
        let generators = [(0u32, 1u32), (1, 2)];
        let g = ReachabilityClosure::fold(generators);
        // Same reachable set, regardless of pre-closure.
        let mut cs: Vec<u32> = c.strict_image(&0).into_iter().map(|(v, _)| v).collect();
        let mut gs: Vec<u32> = g.strict_image(&0).into_iter().map(|(v, _)| v).collect();
        cs.sort_unstable();
        gs.sort_unstable();
        assert_eq!(cs, gs, "the reachable set is idempotent under re-fold");
        assert!(c.reaches(&0, &2) && g.reaches(&0, &2));

        // The shortest-path GRADING is, by definition, the shortest path in the
        // GIVEN edge set — so a pre-closure that presents the transitive edge
        // (0,2) as a length-1 generator grades it 1, while folding only the
        // generators grades the same pair 2 (via 0→1→2). The grading tracks the
        // generators it was given; only the reachable set is closure-invariant.
        assert_eq!(c.distance(&0, &2), Some(1)); // (0,2) given directly
        assert_eq!(g.distance(&0, &2), Some(2)); // (0,2) only via the path
    }
}
