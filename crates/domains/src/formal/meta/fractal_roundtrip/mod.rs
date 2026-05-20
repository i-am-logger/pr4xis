//! FractalRoundTrip — the signature-of-understanding for loaded
//! sources.
//!
//! A source `S` implementing [`FractalRoundTrip`] proves Praxis can
//! reconstruct it: `parse → ontology → re-emit` yields bytes whose
//! canonical form has identical SHA-256 to the input's canonical
//! form. Hash mismatch is concrete evidence of an ontology gap.
//!
//! ## Categorical statement
//!
//! Praxis declares an *adjunction* `parse ⊣ reemit` between byte
//! streams and the ontology that types them (Mac Lane *Categories
//! for the Working Mathematician*, 2nd ed., §IV.1). The minimum
//! guarantee an adjunction gives is naturality of unit and counit
//! — not that they are identities. When the adjunction reaches
//! the stronger *equivalence-of-categories* status (Mac Lane §IV.4
//! Theorem 1), unit and counit compose to identity up to natural
//! isomorphism, and the round-trip is faithful.
//!
//! `FractalRoundTrip` is the runtime witness of equivalence. For
//! every byte stream `b`:
//!
//!   `sig(b) == sig(reemit(parse(b)))`     where     `sig = SHA-256 ∘ canonical`
//!
//! The [`canonical`] submodule supplies the per-source-kind
//! canonicalization needed to compute `sig`.
//!
//! ## Module layout
//!
//! - [`roundtrip_trait`] — the [`FractalRoundTrip`] trait + the
//!   [`FractalRoundTripFailure`] error.
//! - [`canonical`] — per-source-kind canonical forms (XML, JSON,
//!   plain text, TOML; RDF stubbed).
//!
//! Tests for the trait laws + the canonical-form library live in
//! [`tests`].
//!
//! ## Citations
//!
//! - **W3C XML Canonicalization 1.1** — Boyer & Marcy 2008, W3C
//!   Recommendation, <https://www.w3.org/TR/xml-c14n11/>.
//! - **RFC 8785 JSON Canonicalization Scheme** — Rundgren, Jordan &
//!   Erdtman 2020, IETF,
//!   <https://www.rfc-editor.org/rfc/rfc8785.html>.
//! - **Unicode TR #15 Normalization Forms** — Davis & Whistler
//!   2024, Unicode Consortium,
//!   <https://www.unicode.org/reports/tr15/>.
//! - **RFC 9595 RDF Dataset Canonicalization** — Longley & Sporny
//!   2024, IETF, <https://www.rfc-editor.org/rfc/rfc9595.html>
//!   (registered; implementation deferred).
//! - **Mac Lane** *Categories for the Working Mathematician* §IV.1
//!   (Adjunctions) and §IV.4 (Equivalence of categories), Springer
//!   GTM 5, 2nd ed. 1998.

pub mod canonical;
pub mod roundtrip_trait;

pub use roundtrip_trait::{FailureStage, FractalRoundTrip, FractalRoundTripFailure};

#[cfg(test)]
mod tests;
