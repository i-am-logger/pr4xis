use std::collections::HashMap;

use crate::category::entity::Concept;

/// Domains implement this to declare context-dependent disambiguation.
///
/// Context is the semantic mechanism for resolving ambiguity:
/// given an ambiguous entity and a contextual signal, produce
/// the correct interpretation.
///
/// This is NOT a Category — it's a ternary relation:
/// (ambiguous entity, signal) → resolution.
///
/// Example: ("bank", MoneyContext) → FinancialInstitution
///          ("bank", RiverContext) → Riverbank
pub trait ContextDef {
    /// The ambiguous entities that need disambiguation.
    type Concept: Concept;
    /// Contextual signals that guide disambiguation.
    type Signal: Concept;
    /// Resolved interpretations.
    type Resolution: Concept;

    /// Resolution rules: (entity, signal) → resolution.
    fn resolutions() -> Vec<(Self::Concept, Self::Signal, Self::Resolution)>;
}

// ---- Query functions ----

/// Resolve an ambiguous entity given a contextual signal.
pub fn resolve<T: ContextDef>(entity: &T::Concept, signal: &T::Signal) -> Option<T::Resolution> {
    T::resolutions()
        .into_iter()
        .find(|(e, s, _)| e == entity && s == signal)
        .map(|(_, _, r)| r)
}

/// All possible resolutions for an ambiguous entity (across all signals).
pub fn interpretations<T: ContextDef>(entity: &T::Concept) -> Vec<(T::Signal, T::Resolution)> {
    T::resolutions()
        .into_iter()
        .filter(|(e, _, _)| e == entity)
        .map(|(_, s, r)| (s, r))
        .collect()
}

/// All signals that can disambiguate a given entity.
pub fn signals_for<T: ContextDef>(entity: &T::Concept) -> Vec<T::Signal> {
    T::resolutions()
        .into_iter()
        .filter(|(e, _, _)| e == entity)
        .map(|(_, s, _)| s)
        .collect()
}

/// All entities that are ambiguous (have more than one possible resolution).
pub fn ambiguous_entities<T: ContextDef>() -> Vec<T::Concept> {
    let mut counts: HashMap<T::Concept, usize> = HashMap::new();
    for (e, _, _) in T::resolutions() {
        *counts.entry(e).or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(e, _)| e)
        .collect()
}

// Context-specific axioms (Deterministic, TrueAmbiguity) were per-def
// structural axioms. Context doesn't have a Relations-ontology kind in
// the catalog — these would be best rewritten as domain-level axioms
// in ontologies that use contexts. Removed for now (#169).
