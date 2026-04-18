use std::collections::HashMap;
use std::marker::PhantomData;

use crate::category::entity::Entity;

/// Domains implement this to declare opposition (antonymy) between entities.
///
/// Opposition is the semantic negation of concepts:
/// if A opposes B, then A ≡ NOT B in context.
///
/// Opposition is NOT a Category — composing two oppositions yields equivalence,
/// not opposition. (opposite(opposite(A)) = A, not a new opposite.)
///
/// Properties:
/// - Symmetric: if A opposes B, then B opposes A
/// - Irreflexive: nothing opposes itself
/// - Involutory: opposite(opposite(A)) = A
/// - NOT transitive: if A opposes B and B opposes C, A may equal C
pub trait OppositionDef {
    type Entity: Entity;
    /// Direct opposition pairs. Order doesn't matter (symmetric).
    fn pairs() -> Vec<(Self::Entity, Self::Entity)>;
}

/// Build symmetric adjacency for opposition pairs.
fn symmetric_adj<E: Entity>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
    let mut adj: HashMap<E, Vec<E>> = HashMap::new();
    for (a, b) in pairs {
        adj.entry(a.clone()).or_default().push(b.clone());
        adj.entry(b.clone()).or_default().push(a.clone());
    }
    adj
}

// ---- Query functions ----

/// All direct opposites of an entity.
pub fn opposites<T: OppositionDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let adj = symmetric_adj(&T::pairs());
    adj.get(entity).cloned().unwrap_or_default()
}

/// Check if two entities are opposites.
pub fn are_opposed<T: OppositionDef>(a: &T::Entity, b: &T::Entity) -> bool {
    opposites::<T>(a).contains(b)
}

// ---- Axioms ----

/// Axiom: opposition is symmetric — if A opposes B, then B opposes A.
/// Enforced by symmetric adjacency, but validated here.
pub struct Symmetric<T: OppositionDef> {
    _marker: PhantomData<T>,
}

impl<T: OppositionDef> Symmetric<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: OppositionDef> Default for Symmetric<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: OppositionDef> crate::logic::Axiom for Symmetric<T> {
    fn description(&self) -> &str {
        "opposition is symmetric: if A opposes B then B opposes A"
    }

    fn holds(&self) -> bool {
        for (a, b) in &T::pairs() {
            if !are_opposed::<T>(b, a) {
                return false;
            }
        }
        true
    }

    crate::axiom_meta!(
        "Symmetric[Opposition]",
        "Aristotle 'Peri Hermeneias' — Square of Opposition: opposition is symmetric"
    );
}

/// Axiom: opposition is irreflexive — nothing opposes itself.
pub struct Irreflexive<T: OppositionDef> {
    _marker: PhantomData<T>,
}

impl<T: OppositionDef> Irreflexive<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: OppositionDef> Default for Irreflexive<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: OppositionDef> crate::logic::Axiom for Irreflexive<T> {
    fn description(&self) -> &str {
        "opposition is irreflexive: nothing opposes itself"
    }

    fn holds(&self) -> bool {
        T::pairs().iter().all(|(a, b)| a != b)
    }

    crate::axiom_meta!(
        "Irreflexive[Opposition]",
        "Aristotle 'Peri Hermeneias' — an entity is not the opposite of itself"
    );
}

/// Axiom: opposition is exclusive with equivalence — if A opposes B,
/// then A and B must not be in the same equivalence class.
/// This connects opposition to the logical NOT: A opposes B means A ≡ NOT B.
///
/// Takes an equivalence checker function to verify against.
pub struct ExclusiveWithEquivalence<T: OppositionDef, F: Fn(&T::Entity, &T::Entity) -> bool> {
    are_equivalent: F,
    _marker: PhantomData<T>,
}

impl<T: OppositionDef, F: Fn(&T::Entity, &T::Entity) -> bool> ExclusiveWithEquivalence<T, F> {
    pub fn new(are_equivalent: F) -> Self {
        Self {
            are_equivalent,
            _marker: PhantomData,
        }
    }
}

impl<T: OppositionDef, F: Fn(&T::Entity, &T::Entity) -> bool> crate::logic::Axiom
    for ExclusiveWithEquivalence<T, F>
{
    fn description(&self) -> &str {
        "opposites cannot be equivalent (A opposes B implies A ≢ B)"
    }

    fn holds(&self) -> bool {
        for (a, b) in &T::pairs() {
            if (self.are_equivalent)(a, b) {
                return false;
            }
        }
        true
    }

    crate::axiom_meta!(
        "ExclusiveWithEquivalence[Opposition]",
        "Aristotle 'Peri Hermeneias' — A opposes B implies A ≢ B (opposition excludes equivalence)"
    );
}
