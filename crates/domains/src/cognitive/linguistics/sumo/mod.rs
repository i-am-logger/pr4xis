//! SUMO (Niles & Pease 2001) WordNet↔SUMO mappings as loaded, queryable data —
//! a FOURTH, independent corroboration signal for the word-sense pair the
//! two-entity relation path resolves to, alongside
//! [`crate::cognitive::linguistics::verbnet`],
//! [`crate::cognitive::linguistics::conceptnet`], and
//! [`crate::cognitive::linguistics::framenet`]. See `README.md` for the full
//! picture and `citings.md` for the literature.

pub mod ontology;
pub mod reader;
#[cfg(feature = "std")]
#[cfg(test)]
mod regenerate;
#[cfg(feature = "std")]
pub mod sssom;
pub mod store;
