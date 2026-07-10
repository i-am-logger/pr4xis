use super::arrow::Arrow;
use super::category::Category;
use super::entity::FinitelyGenerated;
use super::reach::graded_image;
use super::terminal::TerminalTarget;
use crate::logic::Axiom;
use crate::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};

/// Every object has at least one outgoing morphism (no dead states).
pub struct NoDeadStates<C: Category> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: Category> NoDeadStates<C> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<C: Category> Default for NoDeadStates<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Category> Axiom for NoDeadStates<C>
where
    // Enumerates every object to check each has an outgoing morphism — a
    // closed-world (finite) check.
    C::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        if C::Object::variants()
            .iter()
            .all(|obj| !C::morphisms_from(obj).is_empty())
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    crate::axiom_meta!(
        "NoDeadStates",
        "every object has at least one outgoing morphism",
        "Mac Lane (1971) 'Categories for the Working Mathematician' Ch. I"
    );
}

/// Every object is reachable from every other object.
pub struct FullyConnected<C: Category> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: Category> FullyConnected<C> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<C: Category> Default for FullyConnected<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Category> Axiom for FullyConnected<C>
where
    // Enumerates every object to seed the reachability probe — a closed-world
    // (finite) check.
    C::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        let variants = C::Object::variants();
        let morphisms = C::morphisms();
        // Delegate to the ONE graded-reach kernel (`category::reach`) instead
        // of a hand-rolled seen-set BFS. The kernel's vertex must carry a
        // total order; `Concept` only guarantees `Eq + Hash`, so the vertex is
        // the object's INDEX in the finite generator list (`variants()`), which
        // is canonically ordered. The adjacency is ALL-KINDS: every morphism of
        // the category, whatever relation kind it carries — full connectivity
        // is a property of the whole generating graph, not of one kind's slice.
        let neighbors = |i: &usize| -> Vec<usize> {
            let from = &variants[*i];
            morphisms
                .iter()
                .filter(|m| m.source() == *from)
                .filter_map(|m| {
                    let target = m.target();
                    variants.iter().position(|v| *v == target)
                })
                .collect()
        };
        // The kernel's image is STRICT (the source itself is excluded even on a
        // cycle), so "every object reaches every other object" is: each start's
        // strict image covers all `n - 1` other objects. Vacuously true on the
        // empty category (no starts to check).
        let connected =
            (0..variants.len()).all(|i| graded_image(&i, neighbors).len() == variants.len() - 1);
        if connected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    crate::axiom_meta!(
        "FullyConnected",
        "every object is reachable from every other object",
        "Graph connectivity invariant on a category"
    );
}

/// A designated object `T` is **terminal**: every object has exactly one
/// morphism to `T`.
///
/// In a thin category this is "everything reaches a single sink." For an
/// interaction statechart whose objects are modes, `T` = the root mode and this
/// axiom is the **no-stuck guarantee**: from every mode there is exactly one way
/// back to root, so no input can strand the user (a terminal object is the limit
/// of the empty diagram — its defining universal property is the right adjoint to
/// the unique functor `! : C → 1`).
///
/// Literature:
/// - Awodey (2010) *Category Theory* Ch. 2 — initial and terminal objects.
/// - Mac Lane (1971) CWM III.4 — terminal object as the limit of the empty
///   diagram; right adjoint to `!`.
pub struct TerminalObject<C, T> {
    _marker: core::marker::PhantomData<(C, T)>,
}

impl<C, T> TerminalObject<C, T> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<C, T> Default for TerminalObject<C, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, T> Axiom for TerminalObject<C, T>
where
    C: Category,
    T: TerminalTarget<Category = C>,
    <C as Category>::Morphism: PartialEq,
    // Enumerates every object to check each has exactly one morphism to the
    // terminal — a closed-world (finite) check.
    C::Object: FinitelyGenerated,
{
    fn verify(&self) -> Verdict {
        let t = T::target();
        let terminal = <C::Object as FinitelyGenerated>::variants()
            .iter()
            .all(|a| {
                C::morphisms_from(a)
                    .into_iter()
                    .filter(|m| m.target() == t)
                    .count()
                    == 1
            });
        if terminal {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    crate::axiom_meta!(
        "TerminalObject",
        "every object has exactly one morphism to the terminal object (no-stuck root)",
        "Awodey (2010) Category Theory Ch. 2; Mac Lane (1971) CWM III.4 — terminal object as limit of the empty diagram"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::{Concept, FinitelyGenerated};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Tri {
        A,
        B,
        C,
    }
    impl Concept for Tri {}
    impl FinitelyGenerated for Tri {
        fn variants() -> Vec<Self> {
            vec![Tri::A, Tri::B, Tri::C]
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TriEdge {
        from: Tri,
        to: Tri,
    }
    impl Arrow for TriEdge {
        type Object = Tri;
        type Kind = ();
        fn source(&self) -> Tri {
            self.from
        }
        fn target(&self) -> Tri {
            self.to
        }
        fn kind(&self) {}
    }

    /// A generating-edge witness category over [`Tri`], its edge set injected
    /// by the marker type — so one fixture serves the connected 3-cycle, the
    /// broken 2-cycle (isolated `C`), and the one-way chain (directionality).
    trait EdgeSet {
        fn edges() -> Vec<TriEdge>;
    }
    struct WitnessCat<E: EdgeSet>(core::marker::PhantomData<E>);
    impl<E: EdgeSet + 'static> Category for WitnessCat<E> {
        type Object = Tri;
        type Morphism = TriEdge;
        fn identity(obj: &Tri) -> TriEdge {
            TriEdge {
                from: *obj,
                to: *obj,
            }
        }
        fn compose(f: &TriEdge, g: &TriEdge) -> Option<TriEdge> {
            (f.to == g.from).then_some(TriEdge {
                from: f.from,
                to: g.to,
            })
        }
        fn morphisms() -> Vec<TriEdge> {
            E::edges()
        }
    }

    fn edge(from: Tri, to: Tri) -> TriEdge {
        TriEdge { from, to }
    }

    /// The 3-cycle `A → B → C → A` — every object reaches every other.
    struct Cycle;
    impl EdgeSet for Cycle {
        fn edges() -> Vec<TriEdge> {
            vec![
                edge(Tri::A, Tri::B),
                edge(Tri::B, Tri::C),
                edge(Tri::C, Tri::A),
            ]
        }
    }

    /// `A ⇄ B` with `C` isolated — NOT fully connected.
    struct Isolated;
    impl EdgeSet for Isolated {
        fn edges() -> Vec<TriEdge> {
            vec![edge(Tri::A, Tri::B), edge(Tri::B, Tri::A)]
        }
    }

    /// The one-way chain `A → B → C` — A reaches all, but C reaches nothing:
    /// reachability is DIRECTED, so this must fail (a symmetric-closure bug —
    /// or a neighbor filter that walked edges backwards — would pass it).
    struct OneWay;
    impl EdgeSet for OneWay {
        fn edges() -> Vec<TriEdge> {
            vec![edge(Tri::A, Tri::B), edge(Tri::B, Tri::C)]
        }
    }

    #[test]
    fn a_cycle_is_fully_connected() {
        assert!(
            FullyConnected::<WitnessCat<Cycle>>::new().verify().is_ok(),
            "the 3-cycle reaches everywhere from everywhere"
        );
    }

    #[test]
    fn an_isolated_object_fails_full_connectivity() {
        assert!(
            FullyConnected::<WitnessCat<Isolated>>::new()
                .verify()
                .is_err(),
            "an isolated object must fail full connectivity"
        );
    }

    #[test]
    fn a_one_way_chain_fails_full_connectivity() {
        assert!(
            FullyConnected::<WitnessCat<OneWay>>::new()
                .verify()
                .is_err(),
            "reachability is directed — the chain's sink reaches nothing"
        );
    }
}
