//! XSD built-in datatypes — W3C XML Schema 1.1 Part 2 as a Praxis
//! ontology.
//!
//! XSD 1.1 Part 2 (Peterson, Gao, Akhmedov, Malhotra, Biron &
//! Sperberg-McQueen 2012) defines a fixed inventory of built-in
//! datatypes — the value spaces that every `<xs:simpleType>`
//! ultimately restricts. The inventory is *closed*: it is enumerated
//! exhaustively in the Recommendation (§3.2 special, §3.3 the 19
//! primitives, §3.4 the 28 derived datatypes — 25 from XSD 1.0 plus
//! `yearMonthDuration` / `dayTimeDuration` / `dateTimeStamp` new in
//! 1.1). A closed published inventory is exactly the case the
//! hand-authored [`pr4xis::ontology!`] macro is for.
//!
//! ## Module layout
//!
//! - [`ontology`] — the [`XsdDatatypeConcept`](ontology::XsdDatatypeConcept)
//!   inventory: 50 concepts (3 special + 19 primitive + 28 derived),
//!   the `is_a` = {base type definition} derivation lattice (Part 2
//!   §3.4 hierarchy diagram), the [`Variety`](ontology::Variety)
//!   quality (atomic / list / union), and the lattice axioms (single
//!   root `anyType`, derivation acyclic, primitives base on
//!   `anyAtomicType`, the XSD 1.1 additions present).
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. <https://www.w3.org/TR/xmlschema11-2/>

pub mod binary;
pub mod floating;
pub mod numeric;
pub mod ontology;
pub mod strings;
pub mod temporal;
pub mod versioned;

#[cfg(test)]
mod tests;
