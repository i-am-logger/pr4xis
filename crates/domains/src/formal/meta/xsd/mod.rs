//! XSD — W3C XML Schema 1.1 as a Praxis ontology.
//!
//! XSD is a meta-language for describing XML document structure.
//! Any XML schema (USLM, LMF, OOXML, ...) is itself an XSD-described
//! ontology. By declaring XSD as a Praxis ontology, every loaded
//! schema becomes a Praxis ontology instance through a single
//! [xsd-parser AST] → [XsdOntology] functor — no per-schema hand-
//! coding of the concept inventory.
//!
//! ## Why XSD lives in `formal/meta`
//!
//! `formal/meta` is the home for ontologies *about* ontologies and
//! about the syntactic substrate over which ontologies are stated:
//! [`identifier_format`](super::identifier_format),
//! [`source_taxonomy`](super::source_taxonomy),
//! [`gap_analysis`](super::gap_analysis). XSD is a meta-language for
//! schemas, so it belongs here next to identifier-format — not in
//! `social/software/markup`, where the *XML serialisation* and the
//! *USLM schema instance* live.
//!
//! ## Module layout
//!
//! - [`ontology`] — the XSD concept inventory + relationship axioms
//!   (W3C XSD 1.1 Part 1 / Part 2 cited section-by-section).
//! - [`from_xsd_parser`] — the functor that turns xsd-parser's typed
//!   AST into instances of the XSD ontology (Mac Lane §I.3).
//! - [`english_projection`] — the functor `XsdOntology → English`
//!   that projects every schema-component name's localName and every
//!   `<xs:documentation>` prose block through the WordNet-backed
//!   English pipeline (Mac Lane §I.3 functor + Fellbaum 1998 +
//!   Bauer 1983 compound decomposition + Spivak 2014 §5 functorial
//!   structure preservation).
//! - [`schema_vocabulary`] — runtime loader for the
//!   `schema_vocabulary@2026` source, the bundled LMF-encoded
//!   closed-class enumeration of schema-vocabulary names (WHATWG HTML
//!   element/attribute names, W3C XML 1.0 attribute-type names, LRC
//!   USLM XML User Guide element/attribute/type/group/model names,
//!   productive sub-* hierarchical forms per Huddleston & Pullum
//!   2002 Ch. 19 §1.2). Consulted by
//!   [`english_projection::is_schema_vocabulary`] when projecting an
//!   XSD-declared name fails to resolve through WordNet AND is not
//!   classifiable as a statutory-term-of-art.
//! - [`uslm_vocabulary`] — runtime loader that scans the bundled
//!   `uslm-1.0.18.xsd` for every schema-component declaration whose
//!   own `<xsd:annotation><xsd:documentation>` block is non-empty.
//!   Per LRC USLM XML User Guide §V, every USLM element / attribute
//!   carries inline documentation; this loader recognises those
//!   names from the schema's own self-documentation rather than a
//!   hand-curated list. Consulted by
//!   [`english_projection::is_schema_vocabulary`] before falling
//!   back to the [`schema_vocabulary`] bundle.
//!
//! ## Citations
//!
//! - **W3C XML Schema 1.1 Part 1: Structures**, Gao, Sperberg-McQueen
//!   & Thompson 2012, W3C Recommendation 2012-04-05.
//!   <https://www.w3.org/TR/xmlschema11-1/>
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05.
//!   <https://www.w3.org/TR/xmlschema11-2/>
//! - **Mac Lane** *Categories for the Working Mathematician* §I.3
//!   (Functors), Springer GTM 5, 2nd ed. 1998.
//! - **Bergmann, S.** *xsd-parser: Rust code generator for XML
//!   schema files*, v1.5.2, MIT-licensed.
//!   <https://github.com/Bergmann89/xsd-parser>.

pub mod english_projection;
pub mod from_xsd_parser;
pub mod ontology;
pub mod schema_vocabulary;
pub mod uslm_vocabulary;

#[cfg(test)]
mod tests;
