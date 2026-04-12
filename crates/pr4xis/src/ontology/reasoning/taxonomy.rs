use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;
use crate::ontology::Quality;

use super::graph;

/// Domains implement this to declare their is-a taxonomy.
///
/// A taxonomy is a directed acyclic graph (DAG) of subsumption relationships.
/// If A is-a B, then A inherits all qualities of B.
pub trait TaxonomyDef {
    type Entity: Entity;
    /// Direct is-a pairs: (child, parent).
    fn relations() -> Vec<(Self::Entity, Self::Entity)>;
}

/// Is-a relationship morphism: child is-a parent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IsA<E: Entity> {
    pub child: E,
    pub parent: E,
}

impl<E: Entity> Relationship for IsA<E> {
    type Object = E;
    fn source(&self) -> E {
        self.child.clone()
    }
    fn target(&self) -> E {
        self.parent.clone()
    }
}

/// Category adapter for a taxonomy.
///
/// Objects are the entities. Morphisms are is-a relationships
/// (direct relations + identity + transitive closure).
pub struct TaxonomyCategory<T: TaxonomyDef> {
    _marker: PhantomData<T>,
}

impl<T: TaxonomyDef> Category for TaxonomyCategory<T> {
    type Object = T::Entity;
    type Morphism = IsA<T::Entity>;

    fn identity(obj: &T::Entity) -> IsA<T::Entity> {
        IsA {
            child: obj.clone(),
            parent: obj.clone(),
        }
    }

    fn compose(f: &IsA<T::Entity>, g: &IsA<T::Entity>) -> Option<IsA<T::Entity>> {
        if f.parent != g.child {
            return None;
        }
        Some(IsA {
            child: f.child.clone(),
            parent: g.parent.clone(),
        })
    }

    fn morphisms() -> Vec<IsA<T::Entity>> {
        let entities = T::Entity::variants();
        let adj = graph::adjacency_map(&T::relations());

        let mut morphisms = Vec::new();
        for entity in &entities {
            morphisms.push(Self::identity(entity));
            for ancestor in graph::reachable(entity, &adj) {
                morphisms.push(IsA {
                    child: entity.clone(),
                    parent: ancestor,
                });
            }
        }
        morphisms
    }
}

// ---- Query functions ----

/// Check if `child` is-a `ancestor` (transitively).
pub fn is_a<T: TaxonomyDef>(child: &T::Entity, ancestor: &T::Entity) -> bool {
    if child == ancestor {
        return true;
    }
    ancestors::<T>(child).contains(ancestor)
}

/// All ancestors of an entity (transitive). Does not include the entity itself.
pub fn ancestors<T: TaxonomyDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::adjacency_map(&T::relations());
    graph::reachable(entity, &adj)
}

/// All descendants of an entity (transitive). Does not include the entity itself.
pub fn descendants<T: TaxonomyDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::reverse_adjacency_map(&T::relations());
    graph::reachable(entity, &adj)
}

/// Inherit a quality from an ancestor: if the entity doesn't have the quality directly,
/// walk up the taxonomy until an ancestor has it.
pub fn inherit_quality<T, Q>(entity: &T::Entity, quality: &Q) -> Option<Q::Value>
where
    T: TaxonomyDef,
    Q: Quality<Individual = T::Entity>,
{
    if let Some(v) = quality.get(entity) {
        return Some(v);
    }
    for ancestor in ancestors::<T>(entity) {
        if let Some(v) = quality.get(&ancestor) {
            return Some(v);
        }
    }
    None
}

// ---- Axioms ----

/// Axiom: the taxonomy has no cycles (it is a DAG).
pub struct NoCycles<T: TaxonomyDef> {
    _marker: PhantomData<T>,
}

impl<T: TaxonomyDef> NoCycles<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TaxonomyDef> Default for NoCycles<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TaxonomyDef> crate::logic::Axiom for NoCycles<T> {
    fn description(&self) -> &str {
        "taxonomy has no cycles (is a DAG)"
    }

    fn holds(&self) -> bool {
        let adj = graph::adjacency_map(&T::relations());
        T::Entity::variants()
            .iter()
            .all(|entity| !graph::has_cycle(entity, &adj))
    }
}

/// Axiom: antisymmetry — if A is-a B (and A != B), then B is NOT a A.
pub struct Antisymmetric<T: TaxonomyDef> {
    _marker: PhantomData<T>,
}

impl<T: TaxonomyDef> Antisymmetric<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TaxonomyDef> Default for Antisymmetric<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TaxonomyDef> crate::logic::Axiom for Antisymmetric<T> {
    fn description(&self) -> &str {
        "taxonomy is antisymmetric: if A is-a B then B is not a A"
    }

    fn holds(&self) -> bool {
        let direct = T::relations();
        for (child, parent) in &direct {
            if child != parent && is_a::<T>(parent, child) {
                return false;
            }
        }
        true
    }
}
