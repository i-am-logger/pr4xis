use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;

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
        let direct = T::relations();

        let mut effects_map: std::collections::HashMap<T::Entity, HashSet<T::Entity>> =
            std::collections::HashMap::new();
        for (cause, effect) in &direct {
            effects_map
                .entry(cause.clone())
                .or_default()
                .insert(effect.clone());
        }

        let mut morphisms = Vec::new();
        for entity in &entities {
            // Identity
            morphisms.push(Causes {
                cause: entity.clone(),
                effect: entity.clone(),
            });

            // Transitive closure
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            if let Some(direct_effects) = effects_map.get(entity) {
                for e in direct_effects {
                    if visited.insert(e.clone()) {
                        queue.push_back(e.clone());
                    }
                }
            }
            while let Some(current) = queue.pop_front() {
                morphisms.push(Causes {
                    cause: entity.clone(),
                    effect: current.clone(),
                });
                if let Some(next_effects) = effects_map.get(&current) {
                    for e in next_effects {
                        if visited.insert(e.clone()) {
                            queue.push_back(e.clone());
                        }
                    }
                }
            }
        }

        morphisms
    }
}

// ---- Query functions ----

/// All direct and transitive effects of a cause. Does not include the entity itself.
pub fn effects_of<T: CausalDef>(cause: &T::Entity) -> Vec<T::Entity> {
    let direct = T::relations();
    let mut effects_map: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
        std::collections::HashMap::new();
    for (c, e) in &direct {
        effects_map.entry(c.clone()).or_default().push(e.clone());
    }

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(direct_effects) = effects_map.get(cause) {
        for e in direct_effects {
            if visited.insert(e.clone()) {
                queue.push_back(e.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(next_effects) = effects_map.get(&current) {
            for e in next_effects {
                if visited.insert(e.clone()) {
                    queue.push_back(e.clone());
                }
            }
        }
    }
    result
}

/// All direct and transitive causes of an effect. Does not include the entity itself.
pub fn causes_of<T: CausalDef>(effect: &T::Entity) -> Vec<T::Entity> {
    let direct = T::relations();
    let mut causes_map: std::collections::HashMap<T::Entity, Vec<T::Entity>> =
        std::collections::HashMap::new();
    for (c, e) in &direct {
        causes_map.entry(e.clone()).or_default().push(c.clone());
    }

    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(direct_causes) = causes_map.get(effect) {
        for c in direct_causes {
            if visited.insert(c.clone()) {
                queue.push_back(c.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(next_causes) = causes_map.get(&current) {
            for c in next_causes {
                if visited.insert(c.clone()) {
                    queue.push_back(c.clone());
                }
            }
        }
    }
    result
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
        let direct = T::relations();
        direct.iter().all(|(cause, effect)| cause != effect)
    }
}
