use std::collections::HashMap;
use std::marker::PhantomData;

use crate::category::entity::Entity;

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
    type Entity: Entity;
    /// Contextual signals that guide disambiguation.
    type Signal: Entity;
    /// Resolved interpretations.
    type Resolution: Entity;

    /// Resolution rules: (entity, signal) → resolution.
    fn resolutions() -> Vec<(Self::Entity, Self::Signal, Self::Resolution)>;
}

// ---- Query functions ----

/// Resolve an ambiguous entity given a contextual signal.
pub fn resolve<T: ContextDef>(entity: &T::Entity, signal: &T::Signal) -> Option<T::Resolution> {
    T::resolutions()
        .into_iter()
        .find(|(e, s, _)| e == entity && s == signal)
        .map(|(_, _, r)| r)
}

/// All possible resolutions for an ambiguous entity (across all signals).
pub fn interpretations<T: ContextDef>(entity: &T::Entity) -> Vec<(T::Signal, T::Resolution)> {
    T::resolutions()
        .into_iter()
        .filter(|(e, _, _)| e == entity)
        .map(|(_, s, r)| (s, r))
        .collect()
}

/// All signals that can disambiguate a given entity.
pub fn signals_for<T: ContextDef>(entity: &T::Entity) -> Vec<T::Signal> {
    T::resolutions()
        .into_iter()
        .filter(|(e, _, _)| e == entity)
        .map(|(_, s, _)| s)
        .collect()
}

/// All entities that are ambiguous (have more than one possible resolution).
pub fn ambiguous_entities<T: ContextDef>() -> Vec<T::Entity> {
    let mut counts: HashMap<T::Entity, usize> = HashMap::new();
    for (e, _, _) in T::resolutions() {
        *counts.entry(e).or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(e, _)| e)
        .collect()
}

// ---- Axioms ----

/// Axiom: every resolution is deterministic — no (entity, signal) pair
/// maps to more than one resolution.
pub struct Deterministic<T: ContextDef> {
    _marker: PhantomData<T>,
}

impl<T: ContextDef> Deterministic<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ContextDef> Default for Deterministic<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ContextDef> crate::logic::Axiom for Deterministic<T> {
    fn description(&self) -> &str {
        "context resolution is deterministic: each (entity, signal) has at most one resolution"
    }

    fn holds(&self) -> bool {
        let resolutions = T::resolutions();
        let mut seen: HashMap<(T::Entity, T::Signal), T::Resolution> = HashMap::new();
        for (e, s, r) in resolutions {
            if let Some(existing) = seen.get(&(e.clone(), s.clone())) {
                if *existing != r {
                    return false;
                }
            } else {
                seen.insert((e, s), r);
            }
        }
        true
    }

    crate::axiom_meta!(
        "Deterministic[Context]",
        "Carnap (1947) 'Meaning and Necessity' — intension-plus-context determines extension"
    );
}

/// Axiom: every ambiguous entity has at least two distinct resolutions.
/// (If an entity only has one resolution, it's not truly ambiguous.)
pub struct TrueAmbiguity<T: ContextDef> {
    _marker: PhantomData<T>,
}

impl<T: ContextDef> TrueAmbiguity<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ContextDef> Default for TrueAmbiguity<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ContextDef> crate::logic::Axiom for TrueAmbiguity<T> {
    fn description(&self) -> &str {
        "every entity in the context map has at least two distinct resolutions"
    }

    fn holds(&self) -> bool {
        let mut resolutions_per_entity: HashMap<T::Entity, Vec<T::Resolution>> = HashMap::new();
        for (e, _, r) in T::resolutions() {
            let rs = resolutions_per_entity.entry(e).or_default();
            if !rs.contains(&r) {
                rs.push(r);
            }
        }
        resolutions_per_entity.values().all(|rs| rs.len() >= 2)
    }

    crate::axiom_meta!(
        "TrueAmbiguity[Context]",
        "Pustejovsky (1995) 'The Generative Lexicon' — lexical ambiguity requires ≥2 distinct senses"
    );
}
