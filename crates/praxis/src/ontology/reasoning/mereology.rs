use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;

/// Domains implement this to declare their part-whole relationships.
///
/// A mereology is a DAG of has-a relationships.
/// If A has-a B, then B is a part of A.
pub trait MereologyDef {
    type Entity: Entity;
    /// Direct has-a pairs: (whole, part).
    fn relations() -> Vec<(Self::Entity, Self::Entity)>;
}

/// Has-a relationship morphism: whole has-a part.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HasA<E: Entity> {
    pub whole: E,
    pub part: E,
}

impl<E: Entity> Relationship for HasA<E> {
    type Object = E;
    fn source(&self) -> E {
        self.whole.clone()
    }
    fn target(&self) -> E {
        self.part.clone()
    }
}

/// Category adapter for a mereology.
///
/// Objects are the entities. Morphisms are has-a relationships
/// (direct + identity + transitive closure).
pub struct MereologyCategory<T: MereologyDef> {
    _marker: PhantomData<T>,
}

impl<T: MereologyDef> Category for MereologyCategory<T> {
    type Object = T::Entity;
    type Morphism = HasA<T::Entity>;

    fn identity(obj: &T::Entity) -> HasA<T::Entity> {
        HasA {
            whole: obj.clone(),
            part: obj.clone(),
        }
    }

    fn compose(f: &HasA<T::Entity>, g: &HasA<T::Entity>) -> Option<HasA<T::Entity>> {
        if f.part != g.whole {
            return None;
        }
        Some(HasA {
            whole: f.whole.clone(),
            part: g.part.clone(),
        })
    }

    fn morphisms() -> Vec<HasA<T::Entity>> {
        let entities = T::Entity::variants();
        let direct = T::relations();

        let mut parts_map: std::collections::HashMap<T::Entity, HashSet<T::Entity>> =
            std::collections::HashMap::new();
        for (whole, part) in &direct {
            parts_map
                .entry(whole.clone())
                .or_default()
                .insert(part.clone());
        }

        let mut morphisms = Vec::new();
        for entity in &entities {
            // Identity
            morphisms.push(HasA {
                whole: entity.clone(),
                part: entity.clone(),
            });

            // Transitive closure
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            if let Some(direct_parts) = parts_map.get(entity) {
                for p in direct_parts {
                    if visited.insert(p.clone()) {
                        queue.push_back(p.clone());
                    }
                }
            }
            while let Some(current) = queue.pop_front() {
                morphisms.push(HasA {
                    whole: entity.clone(),
                    part: current.clone(),
                });
                if let Some(next_parts) = parts_map.get(&current) {
                    for p in next_parts {
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

/// All direct and transitive parts of a whole. Does not include the entity itself.
pub fn parts_of<T: MereologyDef>(whole: &T::Entity) -> Vec<T::Entity> {
    let direct = T::relations();
    let mut parts_map: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
        std::collections::HashMap::new();
    for (w, p) in &direct {
        parts_map.entry(w.clone()).or_default().push(p.clone());
    }

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(direct_parts) = parts_map.get(whole) {
        for p in direct_parts {
            if visited.insert(p.clone()) {
                queue.push_back(p.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(next_parts) = parts_map.get(&current) {
            for p in next_parts {
                if visited.insert(p.clone()) {
                    queue.push_back(p.clone());
                }
            }
        }
    }
    result
}

/// All wholes that transitively contain this part. Does not include the entity itself.
pub fn whole_of<T: MereologyDef>(part: &T::Entity) -> Vec<T::Entity> {
    let direct = T::relations();
    let mut wholes_map: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
        std::collections::HashMap::new();
    for (w, p) in &direct {
        wholes_map.entry(p.clone()).or_default().push(w.clone());
    }

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(direct_wholes) = wholes_map.get(part) {
        for w in direct_wholes {
            if visited.insert(w.clone()) {
                queue.push_back(w.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(next_wholes) = wholes_map.get(&current) {
            for w in next_wholes {
                if visited.insert(w.clone()) {
                    queue.push_back(w.clone());
                }
            }
        }
    }
    result
}

// ---- Axioms ----

/// Axiom: the mereology has no cycles (it is a DAG).
pub struct NoCycles<T: MereologyDef> {
    _marker: PhantomData<T>,
}

impl<T: MereologyDef> NoCycles<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: MereologyDef> Default for NoCycles<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: MereologyDef> crate::logic::Axiom for NoCycles<T> {
    fn description(&self) -> &str {
        "mereology has no cycles (part-whole is a DAG)"
    }

    fn holds(&self) -> bool {
        let direct = T::relations();
        let mut parts_map: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
            std::collections::HashMap::new();
        for (whole, part) in &direct {
            parts_map
                .entry(whole.clone())
                .or_default()
                .push(part.clone());
        }

        for entity in T::Entity::variants() {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            if let Some(direct_parts) = parts_map.get(&entity) {
                for p in direct_parts {
                    if visited.insert(p.clone()) {
                        queue.push_back(p.clone());
                    }
                }
            }
            while let Some(current) = queue.pop_front() {
                if current == entity {
                    return false;
                }
                if let Some(next_parts) = parts_map.get(&current) {
                    for p in next_parts {
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

/// Axiom: weak supplementation — if A has-a B (and A != B),
/// then A has at least one other direct part C != B.
pub struct WeakSupplementation<T: MereologyDef> {
    _marker: PhantomData<T>,
}

impl<T: MereologyDef> WeakSupplementation<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: MereologyDef> Default for WeakSupplementation<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: MereologyDef> crate::logic::Axiom for WeakSupplementation<T> {
    fn description(&self) -> &str {
        "weak supplementation: every proper whole has at least two direct parts"
    }

    fn holds(&self) -> bool {
        let direct = T::relations();
        let mut parts_map: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
            std::collections::HashMap::new();
        for (whole, part) in &direct {
            if whole != part {
                parts_map
                    .entry(whole.clone())
                    .or_default()
                    .push(part.clone());
            }
        }

        for parts in parts_map.values() {
            if parts.len() < 2 {
                return false;
            }
        }
        true
    }
}
