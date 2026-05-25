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
//! [`FinitelyGenerated`](super::entity::FinitelyGenerated)), [`FreeCategory`]
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
use core::marker::PhantomData;

use super::arrow::Arrow;
use super::category::Category;
use super::entity::Concept;
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

impl<Q: Quiver> Category for FreeCategory<Q> {
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

impl<I: QuiverInterpretation> Functor for FreeExtension<I> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_functor_laws;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum V {
        A,
        B,
    }
    impl Concept for V {
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
}
