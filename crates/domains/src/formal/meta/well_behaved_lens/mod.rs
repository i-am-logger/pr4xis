//! WellBehavedLens — the bidirectional-transformation pattern with
//! round-trip preservation, à la Foster, Greenwald, Moore, Pierce &
//! Schmitt 2007.
//!
//! A source `S` implementing [`WellBehavedLens`] proves Praxis can
//! reconstruct it: `get → ontology → put` yields bytes whose
//! canonical form has an identical content digest to the input's canonical
//! form. Hash mismatch is concrete evidence of an ontology gap.
//!
//! ## Lens laws
//!
//! A [`WellBehavedLens`] declares two operations between byte
//! streams and the ontology target type `T`:
//!
//!   - `get : &[u8] → T`        (parse the source into the target representation)
//!   - `put : &T   → Vec<u8>`   (re-emit the target back to source bytes)
//!
//! Foster et al. 2007 §2.2 define three well-behaved-lens laws:
//!
//!   - **GetPut:** `get(put(t)) = t` — what's put round-trips through get.
//!   - **PutGet:** `canonical(put(get(s))) = canonical(s)` — round-trip
//!     preserves the source up to canonical form.
//!   - **PutPut:** `put(t')` after `put(t)` equals `put(t')` directly —
//!     successive puts are idempotent in source space.
//!
//! `WellBehavedLens` is the runtime witness of PutGet. For every byte
//! stream `b`:
//!
//!   `sig(b) == sig(put(get(b)))`     where     `sig = address ∘ canonical`
//!
//! The [`canonical`] submodule supplies the per-source-kind
//! canonicalization needed to compute `sig`.
//!
//! ## Categorical statement
//!
//! Foster et al.'s lenses are equivalent to the *adjunction* `get ⊣ put`
//! between byte streams and the ontology that types them (Mac Lane
//! *Categories for the Working Mathematician*, 2nd ed., §IV.1). The
//! PutGet law is the unit/counit naturality condition reaching the
//! stronger *equivalence-of-categories* status (Mac Lane §IV.4
//! Theorem 1).
//!
//! ## Module layout
//!
//! - [`lens_trait`] — the [`WellBehavedLens`] trait, the
//!   [`RoundTripFidelity`] level (canonical FLOOR vs byte-exact
//!   graph-faithful, M4.ι / #186), and the [`LensLawFailure`] error.
//! - [`canonical`] — per-source-kind canonical forms (XML, JSON,
//!   plain text, TOML; RDF stubbed).
//!
//! Tests for the trait laws + the canonical-form library live in the
//! `tests` submodule (test-only, not part of the public API).
//!
//! ## Citations
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt 2007** —
//!   "Combinators for Bidirectional Tree Transformations: A
//!   Linguistic Approach to the View Update Problem", *ACM TOPLAS*
//!   29(3) Article 17, §2.2 (well-behaved lens laws).
//! - **W3C XML Canonicalization 1.1** — Boyer & Marcy 2008, W3C
//!   Recommendation, <https://www.w3.org/TR/xml-c14n11/>.
//! - **RFC 8785 JSON Canonicalization Scheme** — Rundgren, Jordan &
//!   Erdtman 2020, IETF,
//!   <https://www.rfc-editor.org/rfc/rfc8785.html>.
//! - **Unicode TR #15 Normalization Forms** — Davis & Whistler
//!   2024, Unicode Consortium,
//!   <https://www.unicode.org/reports/tr15/>.
//! - **W3C RDF Dataset Canonicalization (REC-rdf-canon-20240521)** —
//!   Longley, Kellogg & Yamamoto 2024, W3C Recommendation,
//!   <https://www.w3.org/TR/rdf-canon/>
//!   (registered; implementation deferred).
//! - **Mac Lane** *Categories for the Working Mathematician* §IV.1
//!   (Adjunctions) and §IV.4 (Equivalence of categories), Springer
//!   GTM 5, 2nd ed. 1998.

pub mod canonical;
// The completeness meter reuses the harness verdict + `RoundTripFidelity` and
// the decompile leaf's `DecompileKind`, so it is gated with `decompile` (which
// links the `prx` load path).
#[cfg(feature = "prx")]
pub mod completeness;
// The uniform `.prx → source` decompile op routes to the per-leaf reconstruct
// in the `prx`-gated markup modules (rkyv + gzip), so it is `prx`-only — the
// same gate the OWL/USC/WordNet `.prx` load paths carry.
#[cfg(feature = "prx")]
pub mod decompile;
pub mod harness;
pub mod lens_trait;

pub use harness::{
    HarnessOutcome, HarnessResult, LensRegistration, RoundTripHarnessAllVerified, lens_by_name,
    lens_registrations, run_round_trip_harness,
};
// linkme's distributed_slice is native-only; the raw slice is absent on
// wasm32. Use `lens_registrations()` for a target-portable accessor.
#[cfg(not(target_arch = "wasm32"))]
pub use harness::LENS_REGISTRATIONS;
pub use lens_trait::{FailureStage, LensLawFailure, RoundTripFidelity, WellBehavedLens};

#[cfg(feature = "prx")]
pub use completeness::{
    CompletenessReport, completeness_meter, declared_matches_achieved, floor_source_count,
    print_completeness_meter,
};
#[cfg(feature = "prx")]
pub use decompile::{DecompileError, DecompileKind, decompile};

#[cfg(test)]
mod tests;
