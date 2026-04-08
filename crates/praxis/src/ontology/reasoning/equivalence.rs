use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;

use super::graph;

/// Domains implement this to declare equivalence (synonymy) between entities.
///
/// Equivalence is the semantic foundation for interchangeability:
/// if A ≡ B, then A and B can be used in the same contexts with the same meaning.
///
/// Equivalence forms a groupoid — every morphism is invertible.
/// The transitive closure partitions entities into equivalence classes.
pub trait EquivalenceDef {
    type Entity: Entity;
    /// Direct equivalence pairs. Order doesn't matter (symmetric).
    fn pairs() -> Vec<(Self::Entity, Self::Entity)>;
}

/// Equivalence morphism: A is equivalent to B.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Equivalent<E: Entity> {
    pub left: E,
    pub right: E,
}

impl<E: Entity> Relationship for Equivalent<E> {
    type Object = E;
    fn source(&self) -> E {
        self.left.clone()
    }
    fn target(&self) -> E {
        self.right.clone()
    }
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
fn symmetric_adj<E: Entity>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
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
    type Object = T::Entity;
    type Morphism = Equivalent<T::Entity>;

    fn identity(obj: &T::Entity) -> Equivalent<T::Entity> {
        Equivalent {
            left: obj.clone(),
            right: obj.clone(),
        }
    }

    fn compose(
        f: &Equivalent<T::Entity>,
        g: &Equivalent<T::Entity>,
    ) -> Option<Equivalent<T::Entity>> {
        if f.right != g.left {
            return None;
        }
        Some(Equivalent {
            left: f.left.clone(),
            right: g.right.clone(),
        })
    }

    fn morphisms() -> Vec<Equivalent<T::Entity>> {
        let entities = T::Entity::variants();
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
pub fn equivalent_to<T: EquivalenceDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let adj = symmetric_adj(&T::pairs());
    graph::reachable(entity, &adj)
        .into_iter()
        .filter(|e| e != entity)
        .collect()
}

/// The full equivalence class containing this entity (including self).
pub fn equivalence_class<T: EquivalenceDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let mut class = vec![entity.clone()];
    class.extend(equivalent_to::<T>(entity));
    class
}

/// All equivalence classes in the domain.
pub fn all_classes<T: EquivalenceDef>() -> Vec<Vec<T::Entity>> {
    let entities = T::Entity::variants();
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
pub fn are_equivalent<T: EquivalenceDef>(a: &T::Entity, b: &T::Entity) -> bool {
    if a == b {
        return true;
    }
    equivalent_to::<T>(a).contains(b)
}

// ---- Axioms ----

/// Axiom: all declared pairs are symmetric (if (A,B) declared, (B,A) is implied).
/// This is automatically enforced by the symmetric adjacency map,
/// but this axiom validates the semantic intent.
pub struct Symmetric<T: EquivalenceDef> {
    _marker: PhantomData<T>,
}

impl<T: EquivalenceDef> Symmetric<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: EquivalenceDef> Default for Symmetric<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: EquivalenceDef> crate::logic::Axiom for Symmetric<T> {
    fn description(&self) -> &str {
        "equivalence is symmetric: if A ≡ B then B ≡ A"
    }

    fn holds(&self) -> bool {
        // Always true by construction (symmetric_adj), but we verify
        // that the transitive closure is also symmetric
        for entity in T::Entity::variants() {
            for equiv in equivalent_to::<T>(&entity) {
                if !are_equivalent::<T>(&equiv, &entity) {
                    return false;
                }
            }
        }
        true
    }
}

/// Axiom: no entity is equivalent to itself in the declared pairs.
/// (Reflexivity comes from identity morphisms, not from explicit declarations.)
pub struct NoSelfEquivalence<T: EquivalenceDef> {
    _marker: PhantomData<T>,
}

impl<T: EquivalenceDef> NoSelfEquivalence<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: EquivalenceDef> Default for NoSelfEquivalence<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: EquivalenceDef> crate::logic::Axiom for NoSelfEquivalence<T> {
    fn description(&self) -> &str {
        "no entity is declared equivalent to itself (reflexivity is implicit)"
    }

    fn holds(&self) -> bool {
        T::pairs().iter().all(|(a, b)| a != b)
    }
}
