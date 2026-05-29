//! USLM — United States Legislative Markup.
//!
//! USLM is an XML schema published by the U.S. House Office of
//! the Law Revision Counsel (LRC) for the United States Code. It
//! extends the [generic XML ontology](super) with **legislative**
//! meaning — `<title>`, `<section>`, `<subsection>`,
//! `<paragraph>`, `<subparagraph>`, `<clause>`, `<ref>`, etc.
//! denote the structural units of a legal text per Bluebook §3.3
//! statutory subdivision conventions.
//!
//! ## Authoritative source
//!
//! - U.S. House Office of the Law Revision Counsel, *USLM XML
//!   User Guide and Schema (USLM-1.0.15.xsd)*. Available at
//!   <https://uscode.house.gov/uslm/>.
//! - 1 U.S.C. § 204 — *Codes and Supplements; positive law
//!   titles*, the statute authorizing the U.S. Code itself.
//!
//! ## Layer position
//!
//! USLM sits between the generic XML ontology and the legal
//! Statute ontology, exactly as LMF sits between XML and the
//! lexical English ontology:
//!
//! ```text
//! generic XML  ─►  USLM (this module)  ─►  Statute  ─►  SOX 1514A instance
//! ```
//!
//! The build-time codegen path
//! ([`pr4xis::codegen::uslm`][crate-codegen]) consumes the same
//! data shape, slicing a section out and producing a `RawStatuteDoc`
//! that flows through the existing
//! [`pr4xis::codegen::statute`][crate-codegen-statute] pipeline
//! to emit static Rust modules at build time.
//!
//! [crate-codegen]: ../../../../../../../../pr4xis/codegen/uslm/index.html
//! [crate-codegen-statute]: ../../../../../../../../pr4xis/codegen/statute/index.html

pub mod corpus;
pub mod generated;
pub mod lens;

pub use corpus::*;
pub use lens::{
    UslmLensError, UslmTreeViewLens, UslmTypedTree, UslmXmlLens, read_section, read_uslm_title,
};

#[cfg(test)]
mod tests;
