use praxis_category::Entity;
use std::fmt::Debug;

/// A quality that individuals can have — attributes, capabilities, metadata.
///
/// Qualities describe WHAT an individual IS beyond just its identity.
/// For example, a hardware device individual might have qualities like
/// "supports static color", "has 24 LEDs", "max brightness 100".
///
/// Named after the BFO/DOLCE ontology term "Quality" — an entity that
/// inheres in another entity.
pub trait Quality: Debug + Clone {
    /// The individual type this quality applies to.
    type Individual: Entity;

    /// The value type of this quality.
    type Value: Debug + Clone + PartialEq;

    /// Get the value of this quality for a given individual.
    /// Returns None if the quality doesn't apply to this individual.
    fn get(&self, individual: &Self::Individual) -> Option<Self::Value>;

    /// All individuals that have this quality.
    fn individuals_with(&self) -> Vec<Self::Individual> {
        Self::Individual::variants()
            .into_iter()
            .filter(|e| self.get(e).is_some())
            .collect()
    }
}
