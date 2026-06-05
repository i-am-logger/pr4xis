//! RDF canonical form — **W3C RDF Dataset Canonicalization (RDFC-1.0)**,
//! REC-rdf-canon-20240521 (Longley, Kellogg & Yamamoto 2024,
//! <https://www.w3.org/TR/rdf-canon/>).
//!
//! RDFC-1.0 maps any RDF dataset to a single *serialized canonical form*
//! (sorted canonical N-Quads) by assigning deterministic canonical
//! blank-node identifiers via iterative hashing. Two byte streams of the
//! same RDF source are taken to denote the same graph iff their canonical
//! N-Quads are byte-identical — i.e. iff the graphs are RDF-isomorphic
//! (RDF 1.1 Concepts §3.6).
//!
//! This is the in-house, W3C-conformant implementation that lives at
//! [`crate::social::software::markup::xml::rdf::canon`] (the four
//! sub-algorithms of REC §4, DoS-capped per REC §"Dataset Poisoning",
//! exercised against the official W3C test suite). The generic
//! canonical-form surface here routes to it:
//!
//! ```text
//!   bytes  ──read_owl_to_quads──▶  Vec<Quad>  ──rdf::canonicalize──▶  N-Quads
//! ```
//!
//! [`read_owl_to_quads`] is the RDF/XML-bytes → `Vec<Quad>` path (XML 1.0
//! well-formedness, then the RDF/XML triple reader of Gandon & Schreiber
//! 2014, lifted into the RDF 1.1 default graph). A single RDF/XML document
//! denotes one graph, so every quad is in the default graph.
//!
//! ## Citations
//!
//! - **Longley, D.; Kellogg, G.; Yamamoto, D. (eds.) (2024)** *RDF Dataset
//!   Canonicalization* (RDFC-1.0), W3C Recommendation
//!   REC-rdf-canon-20240521, §4.4.3 / §"A Canonical form of N-Quads".
//! - **Cyganiak, R.; Wood, D.; Lanthaler, M. (eds.) (2014)** *RDF 1.1
//!   Concepts and Abstract Syntax*, §3.6 (graph isomorphism), §4 (RDF
//!   datasets).
//! - **Gandon, F. & Schreiber, G. (eds.) (2014)** *RDF 1.1 XML Syntax*,
//!   W3C Recommendation — the RDF/XML grammar `read_owl_to_quads` parses.
//!
//! [`read_owl_to_quads`]:
//!     crate::social::software::markup::xml::owl::reader::read_owl_to_quads

use alloc::string::ToString;
use alloc::vec::Vec;

use super::CanonicalizationError;
use crate::social::software::markup::xml::owl::reader::read_owl_to_quads;
use crate::social::software::markup::xml::rdf::canonicalize as rdfc_canonicalize;

/// The canonical-form discriminator recorded in `citations.toml` and in
/// any [`CanonicalizationError`] this module raises.
const FORM: &str = "rdf-canon-rec-20240521";

/// Canonicalize an RDF/XML byte stream into its **RDFC-1.0 serialized
/// canonical form** (sorted canonical N-Quads, REC §4.4.3 ca.7).
///
/// Pipeline: parse the RDF/XML bytes into the raw RDF graph
/// ([`read_owl_to_quads`]), then run W3C RDF Dataset Canonicalization
/// over the resulting default-graph dataset
/// ([`rdfc_canonicalize`](crate::social::software::markup::xml::rdf::canonicalize)).
/// The algorithm is reused, never duplicated.
///
/// # Errors
///
/// Returns a [`CanonicalizationError`] (form `rdf-canon-rec-20240521`)
/// when the bytes are not UTF-8, are not well-formed XML / RDF/XML, or
/// when canonicalization trips the DoS complexity cap on a poison dataset
/// (REC §"Dataset Poisoning").
///
/// [`read_owl_to_quads`]:
///     crate::social::software::markup::xml::owl::reader::read_owl_to_quads
pub fn canonicalize(bytes: &[u8]) -> Result<Vec<u8>, CanonicalizationError> {
    let text =
        core::str::from_utf8(bytes).map_err(|e| CanonicalizationError::new(FORM, e.to_string()))?;
    // Strip an optional BOM (W3C XML 1.0 §F.1) before parsing.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let quads =
        read_owl_to_quads(text).map_err(|e| CanonicalizationError::new(FORM, e.to_string()))?;
    let nquads =
        rdfc_canonicalize(&quads).map_err(|e| CanonicalizationError::new(FORM, e.to_string()))?;
    Ok(nquads.into_bytes())
}
