use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;

use super::graph;

/// Domains implement this to declare their causal relationships.
///
/// A causal graph is a directed acyclic graph where edges represent
/// "A causes B" relationships.
pub trait CausalDef {
    type Entity: Entity;
    /// Direct causal pairs: (cause, effect).
    fn relations() -> Vec<(Self::Entity, Self::Entity)>;
}

/// Causal relationship morphism: cause causes effect.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Causes<E: Entity> {
    pub cause: E,
    pub effect: E,
}

impl<E: Entity> Relationship for Causes<E> {
    type Object = E;
    fn source(&self) -> E {
        self.cause.clone()
    }
    fn target(&self) -> E {
        self.effect.clone()
    }
}

/// Category adapter for a causal graph.
///
/// Objects are the entities. Morphisms are causal relationships
/// (direct + identity + transitive closure).
pub struct CausalCategory<T: CausalDef> {
    _marker: PhantomData<T>,
}

impl<T: CausalDef> Category for CausalCategory<T> {
    type Object = T::Entity;
    type Morphism = Causes<T::Entity>;

    fn identity(obj: &T::Entity) -> Causes<T::Entity> {
        Causes {
            cause: obj.clone(),
            effect: obj.clone(),
        }
    }

    fn compose(f: &Causes<T::Entity>, g: &Causes<T::Entity>) -> Option<Causes<T::Entity>> {
        if f.effect != g.cause {
            return None;
        }
        Some(Causes {
            cause: f.cause.clone(),
            effect: g.effect.clone(),
        })
    }

    fn morphisms() -> Vec<Causes<T::Entity>> {
        let entities = T::Entity::variants();
        let adj = graph::adjacency_map(&T::relations());

        let mut morphisms = Vec::new();
        for entity in &entities {
            morphisms.push(Self::identity(entity));
            for effect in graph::reachable(entity, &adj) {
                morphisms.push(Causes {
                    cause: entity.clone(),
                    effect,
                });
            }
        }
        morphisms
    }
}

// ---- Query functions ----

/// All direct and transitive effects of a cause. Does not include the entity itself.
pub fn effects_of<T: CausalDef>(cause: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::adjacency_map(&T::relations());
    graph::reachable(cause, &adj)
}

/// All direct and transitive causes of an effect. Does not include the entity itself.
pub fn causes_of<T: CausalDef>(effect: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::reverse_adjacency_map(&T::relations());
    graph::reachable(effect, &adj)
}

// ---- Axioms ----

/// Axiom: asymmetry — if A causes B (and A != B), then B does NOT cause A.
pub struct Asymmetric<T: CausalDef> {
    _marker: PhantomData<T>,
}

impl<T: CausalDef> Asymmetric<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: CausalDef> Default for Asymmetric<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CausalDef> crate::logic::Axiom for Asymmetric<T> {
    fn description(&self) -> &str {
        "causation is asymmetric: if A causes B then B does not cause A"
    }

    fn holds(&self) -> bool {
        let direct = T::relations();
        for (cause, effect) in &direct {
            if cause != effect && effects_of::<T>(effect).contains(cause) {
                return false;
            }
        }
        true
    }
}

/// Axiom: no self-causation — no entity directly causes itself.
pub struct NoSelfCausation<T: CausalDef> {
    _marker: PhantomData<T>,
}

impl<T: CausalDef> NoSelfCausation<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: CausalDef> Default for NoSelfCausation<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CausalDef> crate::logic::Axiom for NoSelfCausation<T> {
    fn description(&self) -> &str {
        "no entity directly causes itself"
    }

    fn holds(&self) -> bool {
        T::relations().iter().all(|(cause, effect)| cause != effect)
    }
}
