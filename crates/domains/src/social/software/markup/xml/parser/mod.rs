//! Literature-grounded XML 1.0 parser and serializer pair.
//!
//! Implements a recursive-descent parser over the W3C XML 1.0 Fifth
//! Edition grammar (Bray, Paoli, Sperberg-McQueen, Maler & Yergeau
//! 2008, W3C Recommendation 26 November 2008), paired with a
//! symmetric serializer. The pair is wrapped as an
//! [`XmlLens`](lens::XmlLens) implementing [`WellBehavedLens`](
//! crate::formal::meta::well_behaved_lens::WellBehavedLens) per
//! Foster, Greenwald, Moore, Pierce & Schmitt (2007, ACM TOPLAS
//! 29(3) §2.2): the parser is `get`, the serializer is `put`, and
//! W3C XML Canonicalization 1.1 (Boyer & Marcy 2008) is the
//! canonical-form witness for the PutGet law.
//!
//! # Citations
//!
//! - **Bray, Paoli, Sperberg-McQueen, Maler & Yergeau (2008)** XML
//!   1.0 Fifth Edition, W3C Recommendation.
//!   <https://www.w3.org/TR/xml/> — the EBNF productions
//!   implemented in [`grammar`].
//! - **Cowan & Tobin (2004)** XML Information Set, Second Edition,
//!   W3C Recommendation. <https://www.w3.org/TR/xml-infoset/> —
//!   the eleven information items that the parser emits, surfaced
//!   through the typed [`XmlDocument`](super::ontology::XmlDocument)
//!   / [`XmlElement`](super::ontology::XmlElement) /
//!   [`XmlNode`](super::ontology::XmlNode) ontology types.
//! - **Bray, Hollander, Layman & Tobin (2009)** Namespaces in XML
//!   1.0, Third Edition, W3C Recommendation.
//!   <https://www.w3.org/TR/xml-names/> — the rules used to
//!   discriminate `xmlns` / `xmlns:prefix` attributes from regular
//!   attributes.
//! - **Boyer & Marcy (2008)** XML Canonicalization Version 1.1,
//!   W3C Recommendation 2 May 2008.
//!   <https://www.w3.org/TR/xml-c14n11/> — the canonical-form
//!   specification used by [`XmlLens::canonical`](lens::XmlLens).
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)**
//!   "Combinators for bidirectional tree transformations: A
//!   linguistic approach to the view-update problem", ACM TOPLAS
//!   29(3) §2.2 — the well-behaved lens laws (GetPut, PutGet,
//!   PutPut) that bind the parser to the serializer.
//! - **Aho, Sethi & Ullman (1986)** *Compilers: Principles,
//!   Techniques, and Tools* (Dragon Book), Chapter 4 §4.4 —
//!   recursive-descent parsing strategy. The XML 1.0 grammar is
//!   LL(1) on the prolog and structurally LL(1) on element bodies
//!   (with predictable lookahead via `<`, `<!`, `<?`, `]]>`), so
//!   straight recursive descent suffices without backtracking.
//!
//! # Layering
//!
//! ```text
//!   bytes  <─XmlLens─>  XmlDocument  ← W3C Infoset (Cowan & Tobin 2004)
//!     │
//!     │  put = serialize(target)
//!     │  get = parse(bytes)
//!     │  canonical = c14n_1_1(bytes)
//!     ▼
//!   address(canonical(s)) is the source's stable signature.
//! ```
//!
//! Each submodule covers one layer of the parser pipeline:
//!
//! - [`grammar`] — W3C XML 1.0 §2–§3 productions as recursive-descent
//!   functions, byte-level (no allocations beyond the emitted typed
//!   tree).
//! - [`serializer`] — the symmetric inverse of [`grammar`]:
//!   `XmlDocument` → bytes. Preserves the byte-level form modulo C14N.
//! - [`lens`] — the [`XmlLens`](lens::XmlLens) [`WellBehavedLens`](
//!   crate::formal::meta::well_behaved_lens::WellBehavedLens) impl
//!   binding `get`/`put`/`canonical` together.

pub mod axioms;
pub mod conformance;
pub mod grammar;
pub mod lens;
pub mod serializer;
pub mod source_syntax;
pub mod xmlconf_audit;

#[cfg(test)]
mod tests;
