use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;

use super::graph;

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
        let adj = graph::adjacency_map(&T::relations());

        let mut morphisms = Vec::new();
        for entity in &entities {
            morphisms.push(Self::identity(entity));
            for part in graph::reachable(entity, &adj) {
                morphisms.push(HasA {
                    whole: entity.clone(),
                    part,
                });
            }
        }
        morphisms
    }
}

// ---- Query functions ----

/// All direct and transitive parts of a whole. Does not include the entity itself.
pub fn parts_of<T: MereologyDef>(whole: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::adjacency_map(&T::relations());
    graph::reachable(whole, &adj)
}

/// All wholes that transitively contain this part. Does not include the entity itself.
pub fn whole_of<T: MereologyDef>(part: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::reverse_adjacency_map(&T::relations());
    graph::reachable(part, &adj)
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
        let adj = graph::adjacency_map(&T::relations());
        T::Entity::variants()
            .iter()
            .all(|entity| !graph::has_cycle(entity, &adj))
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
        let adj = graph::adjacency_map(
            &direct
                .iter()
                .filter(|(w, p)| w != p)
                .cloned()
                .collect::<Vec<_>>(),
        );
        if adj.is_empty() {
            return false;
        }
        adj.values().all(|parts| parts.len() >= 2)
    }
}
