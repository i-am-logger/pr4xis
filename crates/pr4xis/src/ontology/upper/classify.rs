use super::being::Being;

/// Trait that domains implement to declare their ontological classification.
///
/// Every domain in praxis models some aspect of reality. The `Classified` trait
/// declares WHAT TYPE of being that domain models, according to the DOLCE upper ontology.
///
/// This classification enables:
/// - Correct namespace placement (social objects group together, abstracts group together)
/// - Functor-based ontology evolution (mappings between classification schemes)
/// - Self-knowledge (praxis knows what type of thing each of its domains is)
pub trait Classified {
    /// What type of being does this domain primarily model?
    fn being() -> Being;

    /// Human-readable explanation of the classification.
    fn classification_reason() -> &'static str;
}
