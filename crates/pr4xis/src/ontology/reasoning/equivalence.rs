#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::marker::PhantomData;
use hashbrown::{HashMap, HashSet};

use crate::category::Category;
use crate::category::entity::Concept;
use crate::category::arrow::Arrow;

use super::graph;

/// Domains implement this to declare equivalence (synonymy) between entities.
///
/// Equivalence is the semantic foundation for interchangeability:
/// if A ≡ B, then A and B can be used in the same contexts with the same meaning.
///
/// Equivalence forms a groupoid — every morphism is invertible.
/// The transitive closure partitions entities into equivalence classes.
pub trait EquivalenceDef {
    type Concept: Concept;
    /// Direct equivalence pairs. Order doesn't matter (symmetric).
    fn pairs() -> Vec<(Self::Concept, Self::Concept)>;
}

/// Equivalence morphism: A is equivalent to B.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Equivalent<E: Concept> {
    pub left: E,
    pub right: E,
}

impl<E: Concept> Arrow for Equivalent<E> {
    type Object = E;
    type Kind = ();
    fn source(&self) -> E {
        self.left.clone()
    }
    fn target(&self) -> E {
        self.right.clone()
    }
    fn kind(&self) {}
}

/// Category adapter for equivalence relations.
///
/// This is a groupoid: every morphism has an inverse.
/// Objects are entities. Morphisms are equivalence relations
/// (direct pairs + symmetric closure + transitive closure + identity).
pub struct EquivalenceCategory<T: EquivalenceDef> {
    _marker: PhantomData<T>,
}

/// Build the symmetric adjacency map from equivalence pairs.
fn symmetric_adj<E: Concept>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
    let mut adj: HashMap<E, Vec<E>> = HashMap::new();
    for (a, b) in pairs {
        if a != b {
            adj.entry(a.clone()).or_default().push(b.clone());
            adj.entry(b.clone()).or_default().push(a.clone());
        }
    }
    adj
}

impl<T: EquivalenceDef> Category for EquivalenceCategory<T> {
    type Object = T::Concept;
    type Morphism = Equivalent<T::Concept>;

    fn identity(obj: &T::Concept) -> Equivalent<T::Concept> {
        Equivalent {
            left: obj.clone(),
            right: obj.clone(),
        }
    }

    fn compose(
        f: &Equivalent<T::Concept>,
        g: &Equivalent<T::Concept>,
    ) -> Option<Equivalent<T::Concept>> {
        if f.right != g.left {
            return None;
        }
        Some(Equivalent {
            left: f.left.clone(),
            right: g.right.clone(),
        })
    }

    fn morphisms() -> Vec<Equivalent<T::Concept>> {
        let entities = T::Concept::variants();
        let adj = symmetric_adj(&T::pairs());

        let mut morphisms = Vec::new();
        for entity in &entities {
            morphisms.push(Self::identity(entity));
            for equiv in graph::reachable(entity, &adj) {
                morphisms.push(Equivalent {
                    left: entity.clone(),
                    right: equiv,
                });
            }
        }
        morphisms
    }
}

// ---- Query functions ----

/// All entities equivalent to this one (transitive closure). Does not include self.
pub fn equivalent_to<T: EquivalenceDef>(entity: &T::Concept) -> Vec<T::Concept> {
    let adj = symmetric_adj(&T::pairs());
    graph::reachable(entity, &adj)
        .into_iter()
        .filter(|e| e != entity)
        .collect()
}

/// The full equivalence class containing this entity (including self).
pub fn equivalence_class<T: EquivalenceDef>(entity: &T::Concept) -> Vec<T::Concept> {
    let mut class = vec![entity.clone()];
    class.extend(equivalent_to::<T>(entity));
    class
}

/// All equivalence classes in the domain.
pub fn all_classes<T: EquivalenceDef>() -> Vec<Vec<T::Concept>> {
    let entities = T::Concept::variants();
    let adj = symmetric_adj(&T::pairs());
    let mut seen = HashSet::new();
    let mut classes = Vec::new();

    for entity in &entities {
        if seen.contains(entity) {
            continue;
        }
        let mut class = vec![entity.clone()];
        for equiv in graph::reachable(entity, &adj) {
            class.push(equiv.clone());
            seen.insert(equiv);
        }
        seen.insert(entity.clone());
        classes.push(class);
    }
    classes
}

/// Check if two entities are equivalent (directly or transitively).
pub fn are_equivalent<T: EquivalenceDef>(a: &T::Concept, b: &T::Concept) -> bool {
    if a == b {
        return true;
    }
    equivalent_to::<T>(a).contains(b)
}

// Structural axioms (Symmetric, NoSelfEquivalence) moved to the
// catalog via `structural_axioms_for<C>()` + OnKind axioms in
// `reasoning::structural`. Per-def axiom types removed (#169).
