use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;
use crate::ontology::Quality;

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
        let direct = T::relations();

        let mut parents_of: std::collections::HashMap<T::Entity, HashSet<T::Entity>> =
            std::collections::HashMap::new();
        for (child, parent) in &direct {
            parents_of
                .entry(child.clone())
                .or_default()
                .insert(parent.clone());
        }

        let mut morphisms = Vec::new();
        for entity in &entities {
            // Identity
            morphisms.push(IsA {
                child: entity.clone(),
                parent: entity.clone(),
            });

            // Transitive closure via BFS
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            if let Some(direct_parents) = parents_of.get(entity) {
                for p in direct_parents {
                    if visited.insert(p.clone()) {
                        queue.push_back(p.clone());
                    }
                }
            }
            while let Some(current) = queue.pop_front() {
                morphisms.push(IsA {
                    child: entity.clone(),
                    parent: current.clone(),
                });
                if let Some(next_parents) = parents_of.get(&current) {
                    for p in next_parents {
                        if visited.insert(p.clone()) {
                            queue.push_back(p.clone());
                        }
                    }
                }
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
    let direct = T::relations();
    let mut parents_of: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
        std::collections::HashMap::new();
    for (child, parent) in &direct {
        parents_of
            .entry(child.clone())
            .or_default()
            .push(parent.clone());
    }

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(direct_parents) = parents_of.get(entity) {
        for p in direct_parents {
            if visited.insert(p.clone()) {
                queue.push_back(p.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(next_parents) = parents_of.get(&current) {
            for p in next_parents {
                if visited.insert(p.clone()) {
                    queue.push_back(p.clone());
                }
            }
        }
    }
    result
}

/// All descendants of an entity (transitive). Does not include the entity itself.
pub fn descendants<T: TaxonomyDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let direct = T::relations();
    let mut children_of: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
        std::collections::HashMap::new();
    for (child, parent) in &direct {
        children_of
            .entry(parent.clone())
            .or_default()
            .push(child.clone());
    }

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(direct_children) = children_of.get(entity) {
        for c in direct_children {
            if visited.insert(c.clone()) {
                queue.push_back(c.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(next_children) = children_of.get(&current) {
            for c in next_children {
                if visited.insert(c.clone()) {
                    queue.push_back(c.clone());
                }
            }
        }
    }
    result
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
        let direct = T::relations();
        let mut parents_of: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
            std::collections::HashMap::new();
        for (child, parent) in &direct {
            parents_of
                .entry(child.clone())
                .or_default()
                .push(parent.clone());
        }

        for entity in T::Entity::variants() {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            if let Some(direct_parents) = parents_of.get(&entity) {
                for p in direct_parents {
                    if visited.insert(p.clone()) {
                        queue.push_back(p.clone());
                    }
                }
            }
            while let Some(current) = queue.pop_front() {
                if current == entity {
                    return false;
                }
                if let Some(next_parents) = parents_of.get(&current) {
                    for p in next_parents {
                        if visited.insert(p.clone()) {
                            queue.push_back(p.clone());
                        }
                    }
                }
            }
        }
        true
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
