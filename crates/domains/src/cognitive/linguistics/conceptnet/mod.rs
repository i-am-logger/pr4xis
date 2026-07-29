//! ConceptNet (Speer, Chin & Havasi 2017) as loaded, queryable data — a third,
//! independent corroboration signal for the word-sense pair the two-entity
//! relation path resolves to, alongside
//! [`crate::cognitive::linguistics::verbnet`]. See `README.md` for the full
//! picture and `citings.md` for the literature.

pub mod ontology;
pub mod reader;
#[cfg(feature = "std")]
#[cfg(test)]
mod regenerate;
pub mod store;
