use crate::category::Concept;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// The DOLCE-inspired upper ontology: types of being.
///
/// DOLCE (Descriptive Ontology for Linguistic and Cognitive Engineering) classifies
/// everything that exists into fundamental categories based on how they exist in time
/// and what kind of thing they are.
///
/// Reference: Masolo et al., "WonderWeb Deliverable D18" (2003);
/// Borgo et al., "DOLCE: A Descriptive Ontology for Linguistic and Cognitive Engineering" (2022).
///
/// # Categories
///
/// ## Endurant (persists through time — has identity over time)
/// - **PhysicalEndurant**: tangible objects (a traffic light, an elevator, a circuit board)
/// - **SocialObject**: exists by agreement/convention (chess rules, XML spec, a legal system, a language)
/// - **MentalObject**: exists in cognition (a concept, a belief, an intention)
/// - **AbstractObject**: timeless, non-temporal (a number, a mathematical proof, a category)
///
/// ## Perdurant (happens over time — has temporal parts)
/// - **Event**: instantaneous or near-instantaneous (a chess move, a key press, a ruling)
/// - **Process**: extended over time (a chess game, a trial, a conversation, a computation)
///
/// ## Quality (measurable property inhering in another entity)
/// - **Quality**: a perceivable/measurable attribute (color, weight, pitch, temperature)
///
/// # Why DOLCE?
///
/// DOLCE was designed specifically for linguistic and cognitive engineering —
/// exactly what praxis does. It provides the classification that our original
/// Entity trait was missing: not just "a thing that can be enumerated" but
/// "what TYPE of thing is this?"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Being {
    // Endurant (continuant — persists through time)
    /// Tangible, physical objects. Things you can touch.
    PhysicalEndurant,
    /// Exists by social agreement: standards, rules, institutions, languages.
    SocialObject,
    /// Exists in cognition: concepts, beliefs, intentions.
    MentalObject,
    /// Timeless, non-temporal: numbers, proofs, mathematical structures.
    AbstractObject,

    // Perdurant (occurrent — happens over time)
    /// Instantaneous or near-instantaneous happening.
    Event,
    /// Extended happening with temporal parts.
    Process,

    // Quality
    /// Measurable property that inheres in another entity.
    Quality,
}

impl Concept for Being {
    fn variants() -> Vec<Self> {
        vec![
            Self::PhysicalEndurant,
            Self::SocialObject,
            Self::MentalObject,
            Self::AbstractObject,
            Self::Event,
            Self::Process,
            Self::Quality,
        ]
    }
}

impl Being {
    /// Is this an endurant (persists through time)?
    pub fn is_endurant(&self) -> bool {
        matches!(
            self,
            Self::PhysicalEndurant | Self::SocialObject | Self::MentalObject | Self::AbstractObject
        )
    }

    /// Is this a perdurant (happens over time)?
    pub fn is_perdurant(&self) -> bool {
        matches!(self, Self::Event | Self::Process)
    }

    /// Is this a quality?
    pub fn is_quality(&self) -> bool {
        matches!(self, Self::Quality)
    }

    /// Is this a non-physical endurant?
    pub fn is_non_physical(&self) -> bool {
        matches!(
            self,
            Self::SocialObject | Self::MentalObject | Self::AbstractObject
        )
    }

    /// Ontological label — the DOLCE name for this type of being.
    pub fn label(&self) -> &'static str {
        match self {
            Self::PhysicalEndurant => "Physical",
            Self::SocialObject => "Social",
            Self::MentalObject => "Mental",
            Self::AbstractObject => "Abstract",
            Self::Event => "Event",
            Self::Process => "Process",
            Self::Quality => "Quality",
        }
    }
}
