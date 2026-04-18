#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Concept;
use crate::category::relationship::Relationship;

use super::graph;

/// Domains implement this to declare their causal relationships.
///
/// A causal graph is a directed acyclic graph where edges represent
/// "A causes B" relationships.
pub trait CausalDef {
    type Concept: Concept;
    /// Direct causal pairs: (cause, effect).
    fn relations() -> Vec<(Self::Concept, Self::Concept)>;
}

/// Causal relationship morphism: cause causes effect.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Causes<E: Concept> {
    pub cause: E,
    pub effect: E,
}

impl<E: Concept> Relationship for Causes<E> {
    type Object = E;
    type Kind = ();
    fn source(&self) -> E {
        self.cause.clone()
    }
    fn target(&self) -> E {
        self.effect.clone()
    }
    fn kind(&self) {}
}

/// Category adapter for a causal graph.
///
/// Objects are the entities. Morphisms are causal relationships
/// (direct + identity + transitive closure).
pub struct CausalCategory<T: CausalDef> {
    _marker: PhantomData<T>,
}

impl<T: CausalDef> Category for CausalCategory<T> {
    type Object = T::Concept;
    type Morphism = Causes<T::Concept>;

    fn identity(obj: &T::Concept) -> Causes<T::Concept> {
        Causes {
            cause: obj.clone(),
            effect: obj.clone(),
        }
    }

    fn compose(f: &Causes<T::Concept>, g: &Causes<T::Concept>) -> Option<Causes<T::Concept>> {
        if f.effect != g.cause {
            return None;
        }
        Some(Causes {
            cause: f.cause.clone(),
            effect: g.effect.clone(),
        })
    }

    fn morphisms() -> Vec<Causes<T::Concept>> {
        let entities = T::Concept::variants();
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
pub fn effects_of<T: CausalDef>(cause: &T::Concept) -> Vec<T::Concept> {
    let adj = graph::adjacency_map(&T::relations());
    graph::reachable(cause, &adj)
}

/// All direct and transitive causes of an effect. Does not include the entity itself.
pub fn causes_of<T: CausalDef>(effect: &T::Concept) -> Vec<T::Concept> {
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

    crate::axiom_meta!(
        "Asymmetric[Causation]",
        "causation is asymmetric: if A causes B then B does not cause A",
        "Lewis (1973) 'Causation'; Reichenbach (1956) 'The Direction of Time'"
    );
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

    crate::axiom_meta!(
        "NoSelfCausation[Causation]",
        "no entity directly causes itself",
        "Lewis (1973) 'Causation' — Humean causation"
    );
}

// ---- Algebraic structure integrations ----

/// Unfold a causal chain from a root cause using anamorphism.
///
/// Produces a Cofree tree where each node is a causal event
/// and children are its direct effects.
///
/// Reference: Meijer, Fokkinga & Paterson (1991)
pub fn unfold_causal<T: CausalDef + 'static>()
-> crate::category::algebra::Coalgebra<T::Concept, T::Concept>
where
    T::Concept: Clone + core::fmt::Debug,
{
    let relations = T::relations();
    crate::category::algebra::Coalgebra::new(move |cause: &T::Concept| {
        let effects: Vec<T::Concept> = relations
            .iter()
            .filter(|(c, _)| c == cause)
            .map(|(_, effect)| effect.clone())
            .collect();
        (cause.clone(), effects)
    })
}

/// Kleisli morphism for causal reasoning: cause → Maybe(effect).
///
/// Composition may fail if the causal chain is broken.
/// This IS the Kleisli category for Maybe applied to causation.
///
/// Reference: Kleisli (1965)
pub fn causal_kleisli<T: CausalDef>(
    cause: &T::Concept,
) -> crate::category::kleisli::KleisliMorphism<CausalCategory<T>>
where
    T::Concept: Clone + PartialEq,
{
    let effects = effects_of::<T>(cause);
    if let Some(first_effect) = effects.first() {
        crate::category::kleisli::KleisliMorphism::total(Causes {
            cause: cause.clone(),
            effect: first_effect.clone(),
        })
    } else {
        crate::category::kleisli::KleisliMorphism::zero(cause.clone(), cause.clone())
    }
}

/// Yoneda profile for causation.
pub fn yoneda_profile<T: CausalDef>(
    entity: &T::Concept,
) -> crate::category::yoneda::YonedaProfile<CausalCategory<T>> {
    crate::category::yoneda::YonedaProfile::of(entity)
}
