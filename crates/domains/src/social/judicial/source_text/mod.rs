//! Source-text reference — the typed verbatim-quote citation that
//! appears throughout legal-domain value types (`LegalTerm`,
//! `Obligation`, `Deadline`, `BurdenOfProof`, `Remedy`, `Exception`,
//! etc.).
//!
//! Conceptually a [`SourceTextRef`] is a *span over a legal-text
//! context* — the verbatim words from the statute, regulation, or case
//! that justify the surrounding legal-reasoning structure. The praxis
//! `cognitive::linguistics::text::ontology::TextConcept::Span` is the
//! conceptual parent; this module specializes it with legal-domain
//! semantics (the citing must be verbatim and attributable).
//!
//! # Why a dedicated typed wrapper
//!
//! Bare `String` cannot carry the semantic claim "this is a verbatim
//! legal citation". A `SourceTextRef` does — praxis can reason
//! "this field is a citing, it must be verbatim, it should be
//! locatable in the source corpus". The wrapper is the praxis-way
//! antidote to opaque-`String` legal value types.
//!
//! Optional [`SourceTextRef::context_uri`] points to the originating
//! corpus (e.g., `"praxis-lock://sox_1514a@2002"`). Future M-tier work
//! resolves the verbatim text against the loaded Context to produce a
//! typed [`Span`][text-span] with verified byte offsets.
//!
//! [text-span]: crate::cognitive::linguistics::text::ontology::TextConcept
//!
//! # Literature
//!
//! - **Hellmann, Lehmann, Auer, Brümmer (2013)** "Integrating NLP using
//!   Linked Data", *Proc. ISWC 2013* — NIF (NLP Interchange Format)
//!   defines the `nif:String` / `nif:Context` / `nif:Span` model that
//!   grounds typed text references.
//! - **RFC 5147 (Wilde & Dürst 2008)** "URI Fragment Identifiers for
//!   the text/plain Media Type", IETF — defines `char=N,M` and
//!   `line=N,M` fragment syntax for citing into plain-text resources.
//! - **The Bluebook: A Uniform System of Citation, 21st ed. (2020)**
//!   §1.2 — verbatim-quotation conventions in legal writing
//!   (parallel-citing, ellipses, brackets).

#[allow(unused_imports)]
use alloc::{string::String, string::ToString};

/// Typed verbatim citation from a legal source. Replaces bare `String`
/// in every `source_text` / `name` / `description` / `definition` /
/// quote-bearing field across the legal-domain value types.
///
/// The contained `text` is the verbatim words as they appear in the
/// source (with the quotation conventions of Bluebook §1.2 — ellipses
/// allowed for non-substantive elisions, square brackets for
/// alterations). `context_uri` optionally identifies the parent
/// corpus (e.g., a `praxis-lock://<source>@<version>` URI) so that
/// future Span resolution against a loaded Context can verify the
/// citation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SourceTextRef {
    /// Verbatim words from the source.
    pub text: String,
    /// Optional pointer to the originating corpus / Context.
    pub context_uri: Option<String>,
}

impl SourceTextRef {
    /// Construct a `SourceTextRef` with verbatim text and no context.
    /// Most legal-data ingestion paths use this — the Context binding
    /// is layered on later.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            context_uri: None,
        }
    }

    /// Construct a `SourceTextRef` bound to a specific context URI
    /// (typically `"praxis-lock://<source>@<version>"`).
    pub fn with_context(text: impl Into<String>, context_uri: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            context_uri: Some(context_uri.into()),
        }
    }

    /// True iff the citation has a context binding.
    pub fn is_bound(&self) -> bool {
        self.context_uri.is_some()
    }

    /// True iff the citation is empty (no text). Empty citations are
    /// usually a parser bug — every Obligation, Deadline, etc. should
    /// carry verbatim text.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// View the verbatim text.
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl From<&str> for SourceTextRef {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SourceTextRef {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn new_constructs_unbound_ref() {
        let r = SourceTextRef::new("the employee may file");
        assert_eq!(r.as_str(), "the employee may file");
        assert!(!r.is_bound());
        assert!(!r.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn with_context_binds_to_uri() {
        let r =
            SourceTextRef::with_context("the employee may file", "praxis-lock://sox_1514a@2002");
        assert!(r.is_bound());
        assert_eq!(
            r.context_uri.as_deref(),
            Some("praxis-lock://sox_1514a@2002")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn empty_is_detected() {
        assert!(SourceTextRef::new("").is_empty());
        assert!(!SourceTextRef::new("a").is_empty());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn from_str_round_trips() {
        let r: SourceTextRef = "shall not retaliate".into();
        assert_eq!(r.as_str(), "shall not retaliate");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn from_owned_string() {
        let r: SourceTextRef = String::from("180 days").into();
        assert_eq!(r.as_str(), "180 days");
    }
}
