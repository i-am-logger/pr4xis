use super::category::Category;
use super::entity::Entity;

// Yoneda lemma — the deepest result in category theory.
//
// For any functor F: C → Set and any object A in C:
//   Nat(Hom(A, -), F) ≅ F(A)
//
// In English: to know everything about an object A, it suffices to know
// all morphisms out of A. An object IS its relationships.
//
// In pr4xis, the Yoneda lemma is the formal basis for:
//   - morphisms_from(A): the representable functor Hom(A, -)
//   - morphisms_to(A): the co-representable functor Hom(-, A)
//   - Understanding an entity through its connections
//   - The "ontological identity" principle: an entity IS defined by
//     how it relates to everything else
//
// The Yoneda embedding: C → [C^op, Set] is full and faithful.
// This means no information is lost by viewing objects through morphisms.
//
// References:
// - Yoneda, "On the Theory of Modules" (1954, unpublished, communicated by Mac Lane)
// - Mac Lane, "Categories for the Working Mathematician" (1971), Ch. III §2
//   — "every concept is a Kan extension" (but Yoneda is the foundation)
// - Riehl, "Category Theory in Context" (2016), Ch. 2
//   https://math.jhu.edu/~eriehl/context.pdf
// - Awodey, "Category Theory" (2010, Oxford), Ch. 8

/// The Yoneda embedding of an object: all morphisms from that object.
///
/// `Yoneda<C>(A)` = `{ f : A → B | for all B in C }` = `Hom(A, -)`
///
/// This IS the representable functor. An object is fully determined
/// by its outgoing morphisms (Yoneda lemma).
pub struct Yoneda<C: Category> {
    /// The represented object.
    pub object: C::Object,
    /// All morphisms from this object (the representable presheaf).
    pub morphisms: Vec<C::Morphism>,
}

impl<C: Category> Yoneda<C>
where
    C::Object: Clone + PartialEq,
    C::Morphism: Clone,
{
    /// Embed an object via Yoneda: collect all morphisms from it.
    pub fn embed(obj: &C::Object) -> Self {
        Self {
            object: obj.clone(),
            morphisms: C::morphisms_from(obj),
        }
    }

    /// The number of outgoing morphisms (the "degree" of this object).
    pub fn degree(&self) -> usize {
        self.morphisms.len()
    }

    /// Two objects are isomorphic iff their Yoneda embeddings are naturally isomorphic.
    /// Simplified: same number of outgoing morphisms to each target.
    pub fn structurally_equivalent(a: &C::Object, b: &C::Object) -> bool {
        C::morphisms_from(a).len() == C::morphisms_from(b).len()
            && C::morphisms_to(a).len() == C::morphisms_to(b).len()
    }
}

/// The co-Yoneda embedding: all morphisms TO an object.
///
/// `CoYoneda<C>(A)` = `{ f : B → A | for all B in C }` = `Hom(-, A)`
pub struct CoYoneda<C: Category> {
    pub object: C::Object,
    pub morphisms: Vec<C::Morphism>,
}

impl<C: Category> CoYoneda<C>
where
    C::Object: Clone + PartialEq,
    C::Morphism: Clone,
{
    /// Embed an object via co-Yoneda: collect all morphisms to it.
    pub fn embed(obj: &C::Object) -> Self {
        Self {
            object: obj.clone(),
            morphisms: C::morphisms_to(obj),
        }
    }

    pub fn degree(&self) -> usize {
        self.morphisms.len()
    }
}

/// Full Yoneda profile: both outgoing and incoming morphisms.
/// The complete "identity" of an object in its category.
pub struct YonedaProfile<C: Category> {
    pub object: C::Object,
    pub outgoing: Vec<C::Morphism>,
    pub incoming: Vec<C::Morphism>,
}

impl<C: Category> YonedaProfile<C>
where
    C::Object: Clone + PartialEq,
    C::Morphism: Clone,
{
    pub fn of(obj: &C::Object) -> Self {
        Self {
            object: obj.clone(),
            outgoing: C::morphisms_from(obj),
            incoming: C::morphisms_to(obj),
        }
    }

    /// The total connectivity of this object.
    pub fn total_degree(&self) -> usize {
        self.outgoing.len() + self.incoming.len()
    }
}

/// Compute the Yoneda profile for every object in a category.
pub fn full_yoneda<C: Category>() -> Vec<YonedaProfile<C>>
where
    C::Object: Clone + PartialEq,
    C::Morphism: Clone,
{
    C::Object::variants()
        .iter()
        .map(|obj| YonedaProfile::of(obj))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::entity::Entity as EntityTrait;
    use crate::category::relationship::Relationship;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Node {
        A,
        B,
        C,
    }
    impl EntityTrait for Node {
        fn variants() -> Vec<Self> {
            vec![Self::A, Self::B, Self::C]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Edge {
        from: Node,
        to: Node,
    }
    impl Relationship for Edge {
        type Object = Node;
        fn source(&self) -> Node {
            self.from
        }
        fn target(&self) -> Node {
            self.to
        }
    }

    struct Graph;
    impl Category for Graph {
        type Object = Node;
        type Morphism = Edge;
        fn identity(obj: &Node) -> Edge {
            Edge {
                from: *obj,
                to: *obj,
            }
        }
        fn compose(f: &Edge, g: &Edge) -> Option<Edge> {
            if f.to == g.from {
                Some(Edge {
                    from: f.from,
                    to: g.to,
                })
            } else {
                None
            }
        }
        fn morphisms() -> Vec<Edge> {
            vec![
                Edge {
                    from: Node::A,
                    to: Node::B,
                },
                Edge {
                    from: Node::B,
                    to: Node::C,
                },
                // identities
                Edge {
                    from: Node::A,
                    to: Node::A,
                },
                Edge {
                    from: Node::B,
                    to: Node::B,
                },
                Edge {
                    from: Node::C,
                    to: Node::C,
                },
            ]
        }
    }

    #[test]
    fn yoneda_embed_collects_outgoing() {
        let y = Yoneda::<Graph>::embed(&Node::A);
        // A has: A→A (identity), A→B
        assert_eq!(y.degree(), 2);
    }

    #[test]
    fn coyoneda_embed_collects_incoming() {
        let cy = CoYoneda::<Graph>::embed(&Node::C);
        // C has incoming: C→C (identity), B→C
        assert_eq!(cy.degree(), 2);
    }

    #[test]
    fn yoneda_profile_total_degree() {
        let p = YonedaProfile::<Graph>::of(&Node::B);
        // B outgoing: B→B, B→C = 2
        // B incoming: B→B, A→B = 2
        assert_eq!(p.total_degree(), 4);
    }

    #[test]
    fn full_yoneda_covers_all_objects() {
        let profiles = full_yoneda::<Graph>();
        assert_eq!(profiles.len(), 3); // A, B, C
    }

    #[test]
    fn yoneda_identity_principle() {
        // Yoneda lemma: an object IS its morphisms.
        // A and C have the same out-degree (2) and in-degree (1 and 2).
        // A: out=[A→A, A→B], in=[A→A]
        // C: out=[C→C], in=[C→C, B→C]
        // They are NOT structurally equivalent.
        assert!(!Yoneda::<Graph>::structurally_equivalent(
            &Node::A,
            &Node::C
        ));
        // A object is equivalent to itself.
        assert!(Yoneda::<Graph>::structurally_equivalent(&Node::A, &Node::A));
    }
}
