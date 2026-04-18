use crate::category::Concept;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

/// DOLCE quality classification (Masolo et al. 2003 *WonderWeb D18*).
///
/// DOLCE splits qualities by the kind of being-aspect they describe:
/// physical properties (mass, color, length), temporal ones (duration,
/// instant), spatial ones (position, orientation, shape), abstract ones
/// (names, identifiers, numerical values), and social ones (roles,
/// permissions, obligations).
///
/// Each `Quality` impl declares its kind via the [`Quality::KIND`] const.
/// Default is `Abstract` — the most conservative classification for
/// qualities without an explicit physical/temporal/spatial/social
/// realisation.
///
/// # Literature
///
/// - Masolo et al. (2003) WonderWeb Deliverable D18 §4.3 — quality hierarchy
/// - Probst (2007) *Semantic Reference Systems* — quality spaces
/// - Galton (2004) *Fields and Objects in Space, Time, and Space-Time* —
///   spatial vs temporal qualities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityKind {
    /// Mass, colour, length, temperature, concentration. Inheres in
    /// physical endurants.
    Physical,
    /// Duration, instant, temporal interval. Inheres in perdurants.
    Temporal,
    /// Position, orientation, shape, extent. Inheres in physical endurants
    /// with a spatial location.
    Spatial,
    /// Names, identifiers, numerical values, abstract measures. Inheres in
    /// abstract objects.
    Abstract,
    /// Role, permission, obligation, legal status. Inheres in social objects.
    Social,
}

/// A quality — an attribute that inheres in an individual (DOLCE/BFO).
///
/// Per Aristotle's *Categories* (translated in Ackrill 1963) and DOLCE
/// (Masolo 2003 §4.3): a quality is an entity that cannot exist
/// independently — it must inhere in something else. The relation is
/// `quality hasQuality individual` / `individual hasQuality quality`.
///
/// In pr4xis, each `Quality` impl is a partial function from individuals
/// (of the associated `Concept` type) to values in a `QualitySpace`
/// (the `Value` type). The DOLCE classification is captured by
/// [`KIND`](Quality::KIND).
///
/// # Example
///
/// ```ignore
/// impl Quality for Brightness {
///     type Individual = Lamp;
///     type Value = u8;
///     const KIND: QualityKind = QualityKind::Physical;
///     fn get(&self, l: &Lamp) -> Option<u8> { Some(l.brightness) }
/// }
/// ```
///
/// # Literature
///
/// - Aristotle *Categories* §8 — quality as a primary category of being
/// - Masolo et al. (2003) WonderWeb D18 §4.3 — DOLCE's quality hierarchy
/// - Smith (2015) *Basic Formal Ontology* — quality as dependent entity
pub trait Quality: Debug + Clone {
    /// The individual type this quality applies to.
    type Individual: Concept;

    /// The value type of this quality — the "quality space" in DOLCE.
    type Value: Debug + Clone + PartialEq;

    /// DOLCE classification of this quality. Default is `Abstract`;
    /// override in impls with a specific physical / temporal / spatial /
    /// social realisation.
    const KIND: QualityKind = QualityKind::Abstract;

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
