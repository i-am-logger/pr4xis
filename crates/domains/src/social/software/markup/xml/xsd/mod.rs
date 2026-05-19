//! XSD — W3C XML Schema Definition Language 1.1.
//!
//! XSD is a W3C-recommended language for declaring the grammar (structure
//! and datatypes) of XML vocabularies. This module is the praxis ontology
//! for XSD itself: the types here encode what an XSD document MEANS per
//! the W3C Recommendation, not just that it's XML.
//!
//! ## Authoritative source
//!
//! - Gao, S., Sperberg-McQueen, C. M., and Thompson, H. S. (eds.)
//!   *W3C XML Schema Definition Language (XSD) 1.1 Part 1: Structures*,
//!   W3C Recommendation 5 April 2012.
//!   <https://www.w3.org/TR/xmlschema11-1/>.
//! - Peterson, D., Gao, S., Malhotra, A., Sperberg-McQueen, C. M., and
//!   Thompson, H. S. (eds.) *W3C XML Schema Definition Language (XSD) 1.1
//!   Part 2: Datatypes*, W3C Recommendation 5 April 2012.
//!   <https://www.w3.org/TR/xmlschema11-2/>.
//!
//! ## Layer position
//!
//! XSD sits between the generic XML ontology and any content-type ontology
//! whose grammar derives from a published schema:
//!
//! ```text
//! generic XML  ─►  XSD (this module)  ─►  USLM ontology  ─►  Statute
//!                              │
//!                              └────────►  LMF ontology  ─►  WordNet
//!                              │
//!                              └────────►  OOXML ontology  ─►  DOCX
//! ```
//!
//! The W3C XSD 1.1 spec defines a much larger language than this module
//! supports. The scope below covers the *XSD subset USLM 1.0.x exercises*
//! — that's the working contract: parse what USLM uses, reject what it
//! doesn't (the parser is "fail-closed" — unknown XSD constructs return a
//! typed error, never silent passthrough). See
//! [`XsdReadError::Unsupported`] for the rejection contract.
//!
//! ## Supported subset (USLM-driven)
//!
//! Per a survey of `crates/domains/data/legal/uscode/schema/uslm-1.0.18.xsd`
//! (101 elements, 37 complexTypes, 14 simpleTypes, 47 attributeGroups,
//! 33 named groups):
//!
//! - **Top-level declarations:** `xsd:element`, `xsd:complexType`,
//!   `xsd:simpleType`, `xsd:attributeGroup`, `xsd:group`.
//! - **Content models inside complexType:** `xsd:sequence`, `xsd:choice`,
//!   `xsd:all`, with optional `minOccurs`/`maxOccurs`.
//! - **Inheritance:** `xsd:complexContent`/`xsd:simpleContent` + `xsd:extension`
//!   + `xsd:restriction` (the "Venetian Blind" pattern USLM uses).
//! - **Substitution groups:** the `substitutionGroup="head"` attribute on
//!   `xsd:element` — the mechanism USLM uses to declare `<ref>`, `<date>`,
//!   `<quotedText>` etc. as members of the `property` head, and `<column>`,
//!   `<p>` etc. as members of the `content` head.
//! - **Attribute declarations:** `xsd:attribute` (inline and ref'd via
//!   `xsd:attributeGroup`).
//! - **Wildcards:** `xsd:any` and `xsd:anyAttribute` (with `namespace=`).
//! - **Documentation:** `xsd:annotation` / `xsd:documentation`.
//! - **Simple type restrictions:** `xsd:enumeration`, `xsd:pattern`.
//!
//! Not yet supported (USLM doesn't use): `xsd:redefine`, `xsd:override`,
//! `xsd:assert`, `xsd:assertion`, `xsd:list`, `xsd:union`, `xsd:include`,
//! conditional type assignment. These return [`XsdReadError::Unsupported`]
//! when encountered.

pub mod ontology;
pub mod reader;

pub use ontology::*;
pub use reader::{XsdReadError, read_xsd};

#[cfg(test)]
mod tests;
