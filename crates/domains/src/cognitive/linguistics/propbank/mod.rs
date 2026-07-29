//! PropBank (Palmer, Gildea & Kingsbury 2005; Bonial, Bonn, Conger, Hwang &
//! Palmer 2014) as loaded, queryable data — a FIFTH, independent
//! corroboration signal for the word-sense pair the two-entity relation
//! path resolves to, alongside [`crate::cognitive::linguistics::verbnet`],
//! [`crate::cognitive::linguistics::conceptnet`],
//! [`crate::cognitive::linguistics::framenet`], and
//! [`crate::cognitive::linguistics::sumo`]. See `README.md` for the full
//! picture and `citings.md` for the literature.

pub mod ontology;
pub mod reader;
#[cfg(feature = "std")]
#[cfg(test)]
mod regenerate;
pub mod store;
