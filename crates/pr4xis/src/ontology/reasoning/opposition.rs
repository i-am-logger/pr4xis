use std::collections::HashMap;

use crate::category::entity::Concept;

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
    type Concept: Concept;
    /// Direct opposition pairs. Order doesn't matter (symmetric).
    fn pairs() -> Vec<(Self::Concept, Self::Concept)>;
}

/// Build symmetric adjacency for opposition pairs.
fn symmetric_adj<E: Concept>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
    let mut adj: HashMap<E, Vec<E>> = HashMap::new();
    for (a, b) in pairs {
        adj.entry(a.clone()).or_default().push(b.clone());
        adj.entry(b.clone()).or_default().push(a.clone());
    }
    adj
}

// ---- Query functions ----

/// All direct opposites of an entity.
pub fn opposites<T: OppositionDef>(entity: &T::Concept) -> Vec<T::Concept> {
    let adj = symmetric_adj(&T::pairs());
    adj.get(entity).cloned().unwrap_or_default()
}

/// Check if two entities are opposites.
pub fn are_opposed<T: OppositionDef>(a: &T::Concept, b: &T::Concept) -> bool {
    opposites::<T>(a).contains(b)
}

// Structural axioms (Symmetric, Irreflexive, ExclusiveWithEquivalence)
// moved to the catalog via `structural_axioms_for<C>()` + OnKind axioms
// in `reasoning::structural`. Per-def axiom types removed (#169).
