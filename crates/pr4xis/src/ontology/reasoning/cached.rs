#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::hash::Hash;
use hashbrown::HashMap;

use super::graph;

/// A pre-computed taxonomy — built once, queried many times.
///
/// This is the "compute once, query many" pattern (the cache ontology).
/// Instead of rebuilding adjacency maps on every query, we build them
/// once during construction and return references thereafter.
#[derive(Debug, Clone)]
pub struct CachedTaxonomy<E: Clone + Eq + Hash> {
    forward: HashMap<E, Vec<E>>,
    reverse: HashMap<E, Vec<E>>,
}

impl<E: Clone + Eq + Hash + core::fmt::Debug> CachedTaxonomy<E> {
    /// Build from relation pairs (child, parent).
    pub fn new(relations: &[(E, E)]) -> Self {
        Self {
            forward: graph::adjacency_map(relations),
            reverse: graph::reverse_adjacency_map(relations),
        }
    }

    /// All ancestors (transitive). Does not include entity itself.
    pub fn ancestors(&self, entity: &E) -> Vec<E> {
        graph::reachable(entity, &self.forward)
    }

    /// All descendants (transitive). Does not include entity itself.
    pub fn descendants(&self, entity: &E) -> Vec<E> {
        graph::reachable(entity, &self.reverse)
    }

    /// Is child a descendant of ancestor (transitively)?
    pub fn is_a(&self, child: &E, ancestor: &E) -> bool {
        if child == ancestor {
            return true;
        }
        self.ancestors(child).contains(ancestor)
    }

    /// Has cycles?
    pub fn has_cycles(&self) -> bool {
        self.forward
            .keys()
            .any(|k| graph::has_cycle(k, &self.forward))
    }

    /// Direct parents of an entity.
    pub fn parents(&self, entity: &E) -> &[E] {
        self.forward
            .get(entity)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Direct children of an entity.
    pub fn children(&self, entity: &E) -> &[E] {
        self.reverse
            .get(entity)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// A pre-computed equivalence — built once, queried many times.
#[derive(Debug, Clone)]
pub struct CachedEquivalence<E: Clone + Eq + Hash> {
    adj: HashMap<E, Vec<E>>,
}

impl<E: Clone + Eq + Hash + core::fmt::Debug> CachedEquivalence<E> {
    /// Build from equivalence pairs (a, b). Symmetric adjacency.
    pub fn new(pairs: &[(E, E)]) -> Self {
        let mut adj: HashMap<E, Vec<E>> = HashMap::new();
        for (a, b) in pairs {
            if a != b {
                adj.entry(a.clone()).or_default().push(b.clone());
                adj.entry(b.clone()).or_default().push(a.clone());
            }
        }
        Self { adj }
    }

    /// All entities equivalent to this one (transitive).
    pub fn equivalent_to(&self, entity: &E) -> Vec<E> {
        graph::reachable(entity, &self.adj)
            .into_iter()
            .filter(|e| e != entity)
            .collect()
    }

    /// Are two entities equivalent?
    pub fn are_equivalent(&self, a: &E, b: &E) -> bool {
        if a == b {
            return true;
        }
        self.equivalent_to(a).contains(b)
    }
}

/// A pre-computed opposition — built once, queried many times.
#[derive(Debug, Clone)]
pub struct CachedOpposition<E: Clone + Eq + Hash> {
    adj: HashMap<E, Vec<E>>,
}

impl<E: Clone + Eq + Hash> CachedOpposition<E> {
    /// Build from opposition pairs (a, b). Symmetric.
    pub fn new(pairs: &[(E, E)]) -> Self {
        let mut adj: HashMap<E, Vec<E>> = HashMap::new();
        for (a, b) in pairs {
            adj.entry(a.clone()).or_default().push(b.clone());
            adj.entry(b.clone()).or_default().push(a.clone());
        }
        Self { adj }
    }

    /// All opposites of an entity.
    pub fn opposites(&self, entity: &E) -> &[E] {
        self.adj.get(entity).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Are two entities opposed?
    pub fn are_opposed(&self, a: &E, b: &E) -> bool {
        self.opposites(a).contains(b)
    }
}

/// A pre-computed mereology — built once, queried many times.
#[derive(Debug, Clone)]
pub struct CachedMereology<E: Clone + Eq + Hash> {
    parts: HashMap<E, Vec<E>>,
    wholes: HashMap<E, Vec<E>>,
}

impl<E: Clone + Eq + Hash + core::fmt::Debug> CachedMereology<E> {
    /// Build from (whole, part) pairs.
    pub fn new(relations: &[(E, E)]) -> Self {
        Self {
            parts: graph::adjacency_map(relations),
            wholes: graph::reverse_adjacency_map(relations),
        }
    }

    /// All parts of a whole (transitive).
    pub fn parts_of(&self, whole: &E) -> Vec<E> {
        graph::reachable(whole, &self.parts)
    }

    /// All wholes containing this part (transitive).
    pub fn whole_of(&self, part: &E) -> Vec<E> {
        graph::reachable(part, &self.wholes)
    }

    /// Direct parts.
    pub fn direct_parts(&self, whole: &E) -> &[E] {
        self.parts.get(whole).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Animal {
        Dog,
        Cat,
        Mammal,
        Animal,
    }

    #[test]
    fn cached_taxonomy_is_a() {
        let t = CachedTaxonomy::new(&[
            (Animal::Dog, Animal::Mammal),
            (Animal::Cat, Animal::Mammal),
            (Animal::Mammal, Animal::Animal),
        ]);
        assert!(t.is_a(&Animal::Dog, &Animal::Animal));
        assert!(t.is_a(&Animal::Dog, &Animal::Mammal));
        assert!(!t.is_a(&Animal::Dog, &Animal::Cat));
        assert!(t.is_a(&Animal::Dog, &Animal::Dog)); // reflexive
    }

    #[test]
    fn cached_taxonomy_parents_children() {
        let t =
            CachedTaxonomy::new(&[(Animal::Dog, Animal::Mammal), (Animal::Cat, Animal::Mammal)]);
        assert_eq!(t.parents(&Animal::Dog), &[Animal::Mammal]);
        assert_eq!(t.children(&Animal::Mammal).len(), 2);
    }

    #[test]
    fn cached_taxonomy_no_cycles() {
        let t = CachedTaxonomy::new(&[
            (Animal::Dog, Animal::Mammal),
            (Animal::Mammal, Animal::Animal),
        ]);
        assert!(!t.has_cycles());
    }

    #[test]
    fn cached_equivalence() {
        let e = CachedEquivalence::new(&[
            (Animal::Dog, Animal::Cat), // pretend they're equivalent for testing
        ]);
        assert!(e.are_equivalent(&Animal::Dog, &Animal::Cat));
        assert!(e.are_equivalent(&Animal::Cat, &Animal::Dog)); // symmetric
        assert!(!e.are_equivalent(&Animal::Dog, &Animal::Mammal));
    }

    #[test]
    fn cached_opposition() {
        let o = CachedOpposition::new(&[(Animal::Dog, Animal::Cat)]);
        assert!(o.are_opposed(&Animal::Dog, &Animal::Cat));
        assert!(o.are_opposed(&Animal::Cat, &Animal::Dog)); // symmetric
        assert_eq!(o.opposites(&Animal::Dog), &[Animal::Cat]);
    }

    #[test]
    fn cached_mereology() {
        let m = CachedMereology::new(&[
            (Animal::Animal, Animal::Mammal),
            (Animal::Mammal, Animal::Dog),
        ]);
        assert_eq!(m.direct_parts(&Animal::Animal), &[Animal::Mammal]);
        let all_parts = m.parts_of(&Animal::Animal);
        assert!(all_parts.contains(&Animal::Mammal));
        assert!(all_parts.contains(&Animal::Dog)); // transitive
    }

    // ---- Property-based tests ----

    mod prop {
        use super::*;
        use proptest::prelude::*;

        fn arb_animal() -> impl Strategy<Value = Animal> {
            prop_oneof![
                Just(Animal::Dog),
                Just(Animal::Cat),
                Just(Animal::Mammal),
                Just(Animal::Animal),
            ]
        }

        fn taxonomy() -> CachedTaxonomy<Animal> {
            CachedTaxonomy::new(&[
                (Animal::Dog, Animal::Mammal),
                (Animal::Cat, Animal::Mammal),
                (Animal::Mammal, Animal::Animal),
            ])
        }

        proptest! {
            /// Taxonomy is reflexive: everything is-a itself.
            #[test]
            fn prop_taxonomy_reflexive(a in arb_animal()) {
                let t = taxonomy();
                prop_assert!(t.is_a(&a, &a));
            }

            /// Taxonomy is transitive: if A is-a B and B is-a C, then A is-a C.
            #[test]
            fn prop_taxonomy_transitive(a in arb_animal(), b in arb_animal(), c in arb_animal()) {
                let t = taxonomy();
                if t.is_a(&a, &b) && t.is_a(&b, &c) {
                    prop_assert!(t.is_a(&a, &c));
                }
            }

            /// Taxonomy is antisymmetric: if A is-a B and B is-a A, then A == B.
            #[test]
            fn prop_taxonomy_antisymmetric(a in arb_animal(), b in arb_animal()) {
                let t = taxonomy();
                if t.is_a(&a, &b) && t.is_a(&b, &a) {
                    prop_assert_eq!(a, b);
                }
            }

            /// Ancestors never include self.
            #[test]
            fn prop_ancestors_exclude_self(a in arb_animal()) {
                let t = taxonomy();
                prop_assert!(!t.ancestors(&a).contains(&a));
            }

            /// Descendants never include self.
            #[test]
            fn prop_descendants_exclude_self(a in arb_animal()) {
                let t = taxonomy();
                prop_assert!(!t.descendants(&a).contains(&a));
            }

            /// If A is a parent of B, then B is a child of A.
            #[test]
            fn prop_parent_child_inverse(a in arb_animal()) {
                let t = taxonomy();
                for parent in t.parents(&a) {
                    prop_assert!(t.children(parent).contains(&a));
                }
            }

            /// Equivalence is reflexive.
            #[test]
            fn prop_equivalence_reflexive(a in arb_animal()) {
                let e = CachedEquivalence::new(&[(Animal::Dog, Animal::Cat)]);
                prop_assert!(e.are_equivalent(&a, &a));
            }

            /// Equivalence is symmetric.
            #[test]
            fn prop_equivalence_symmetric(a in arb_animal(), b in arb_animal()) {
                let e = CachedEquivalence::new(&[(Animal::Dog, Animal::Cat)]);
                if e.are_equivalent(&a, &b) {
                    prop_assert!(e.are_equivalent(&b, &a));
                }
            }

            /// Opposition is symmetric.
            #[test]
            fn prop_opposition_symmetric(a in arb_animal(), b in arb_animal()) {
                let o = CachedOpposition::new(&[(Animal::Dog, Animal::Cat)]);
                if o.are_opposed(&a, &b) {
                    prop_assert!(o.are_opposed(&b, &a));
                }
            }

            /// Opposition is irreflexive: nothing opposes itself.
            #[test]
            fn prop_opposition_irreflexive(a in arb_animal()) {
                let o = CachedOpposition::new(&[(Animal::Dog, Animal::Cat)]);
                prop_assert!(!o.are_opposed(&a, &a));
            }

            /// Parts never include the whole itself.
            #[test]
            fn prop_parts_exclude_self(a in arb_animal()) {
                let m = CachedMereology::new(&[
                    (Animal::Animal, Animal::Mammal),
                    (Animal::Mammal, Animal::Dog),
                ]);
                prop_assert!(!m.parts_of(&a).contains(&a));
            }

            /// Cached taxonomy gives same results as non-cached.
            #[test]
            fn prop_cached_matches_original(a in arb_animal(), b in arb_animal()) {
                let relations = vec![
                    (Animal::Dog, Animal::Mammal),
                    (Animal::Cat, Animal::Mammal),
                    (Animal::Mammal, Animal::Animal),
                ];
                let cached = CachedTaxonomy::new(&relations);
                let original = crate::ontology::reasoning::taxonomy::is_a::<TestTaxonomy>(&a, &b);
                prop_assert_eq!(cached.is_a(&a, &b), original);
            }
        }

        // Helper for comparing cached vs original
        use crate::category::Concept;
        use crate::ontology::reasoning::taxonomy::TaxonomyDef;

        impl Concept for Animal {
            fn variants() -> Vec<Self> {
                vec![Animal::Dog, Animal::Cat, Animal::Mammal, Animal::Animal]
            }
        }

        struct TestTaxonomy;
        impl TaxonomyDef for TestTaxonomy {
            type Concept = Animal;
            fn relations() -> Vec<(Animal, Animal)> {
                vec![
                    (Animal::Dog, Animal::Mammal),
                    (Animal::Cat, Animal::Mammal),
                    (Animal::Mammal, Animal::Animal),
                ]
            }
        }
    }
}
