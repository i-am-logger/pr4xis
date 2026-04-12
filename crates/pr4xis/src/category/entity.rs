use std::fmt::Debug;
use std::hash::Hash;

/// An entity is a thing that exists in an ontology — an object in the category.
///
/// Entities must be finite and enumerable. Every entity can list all possible
/// values of its type, enabling exhaustive validation of ontology properties.
pub trait Entity: Sized + Clone + Eq + Hash + Debug {
    /// All possible entities of this type.
    fn variants() -> Vec<Self>;
}
