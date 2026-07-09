//! Canonical-form library — per-source-kind canonicalizers used by
//! [`WellBehavedLens::canonical`](super::lens_trait::WellBehavedLens::canonical).
//!
//! Each submodule implements its source kind's *published* canonical
//! form. Two byte streams of the same kind are taken to represent
//! the same source iff their canonical forms are byte-identical.
//! The canonical form is what makes the PutGet law (Foster et al.
//! 2007 §3, Definition 3.2) checkable: `canonical(put(get(s))) == canonical(s)`.
//!
//! ## Spec coverage
//!
//! - [`xml`] — W3C XML Canonicalization 1.1 (Boyer & Marcy 2008,
//!   W3C Recommendation, <https://www.w3.org/TR/xml-c14n11/>). No
//!   maintained Rust crate at the time of writing; we walk the
//!   document with `quick-xml` and emit per §3 of the spec.
//!   Implements the core subset required for the lens-law gate;
//!   does *not* yet implement the inclusive-namespace prefix list
//!   from XML C14N 1.1 §2.3 (only relevant when canonicalizing
//!   document subsets, which the lens-law gate does not).
//! - [`json`] — RFC 8785 JSON Canonicalization Scheme (Rundgren,
//!   Jordan & Erdtman 2020, IETF,
//!   <https://www.rfc-editor.org/rfc/rfc8785.html>). Implemented
//!   by walking `serde_json::Value`, emitting per RFC 8785 §3.2
//!   (numbers per JSON.stringify of ECMA-262 §7.1.12.1; sorted
//!   keys; UTF-8; no insignificant whitespace).
//! - [`plain_text`] — Unicode Technical Report #15 Normalization
//!   Forms (Davis & Whistler 2024, Unicode Consortium,
//!   <https://www.unicode.org/reports/tr15/>), specifically NFKC
//!   per §6, plus LF normalization (CRLF/CR → LF) and BOM strip.
//! - [`rdf`] — W3C Recommendation REC-rdf-canon-20240521 "RDF Dataset
//!   Canonicalization" (RDFC-1.0; Longley, Kellogg & Yamamoto 2024,
//!   <https://www.w3.org/TR/rdf-canon/>). Routes RDF/XML bytes through
//!   the in-house, W3C-suite-conformant implementation at
//!   [`crate::social::software::markup::xml::rdf::canon`]
//!   (`bytes → read_owl_to_quads → rdf::canonicalize → canonical
//!   N-Quads`). Two RDF graphs canonicalize to byte-identical output iff
//!   they are RDF-isomorphic (RDF 1.1 §3.6) — graph identity, the OWL
//!   lens's `[canonical_signatures]` form.
//! - [`toml`] — TOML has no canonical-form RFC. We document our
//!   canonical form: parse into [`::toml::Value`], walk with
//!   sorted-table-keys + consistent quoting (always double-quoted
//!   strings, no trailing commas, one entry per line for tables).
//!   This is *Praxis's* canonical form for TOML, not a published
//!   IETF standard; new versions of `toml-rs` could change the
//!   emitter output, in which case we re-canonicalize.

pub mod json;
pub mod plain_text;
pub mod rdf;
pub mod toml;
pub mod xml;

use alloc::string::String;
use core::fmt;

/// Error returned by every per-source canonicalizer in this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalizationError {
    /// Which canonical form failed.
    pub form: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

impl CanonicalizationError {
    pub fn new(form: &'static str, message: impl Into<String>) -> Self {
        Self {
            form,
            message: message.into(),
        }
    }
}

impl fmt::Display for CanonicalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} canonicalization failed: {}", self.form, self.message)
    }
}
