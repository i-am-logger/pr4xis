//! Citation quality — a graded, multi-dimensional model of how good a
//! literature citation is.
//!
//! Correctness of a citation is not a boolean. A citation can name a
//! real work that genuinely supports the claim, yet point at the wrong
//! section, misspell the author, or break the required style. Praxis
//! should be able to say "valid, with these issues" rather than collapse
//! all of that to pass/fail. This module is the hand-authored core that
//! makes that possible: the set of independently-verifiable *dimensions*
//! a citation is judged on, and the *severity* each carries.
//!
//! # Layering
//!
//! The dimensions here are the spine; the rest of the model wires onto
//! them and lives in (or will live in) adjacent modules:
//!
//! - **Relationship** — how the citing text uses the source (quotation,
//!   paraphrase, cites-as-evidence, agrees-with, disputes). Grounded in
//!   the loaded CiTO/SPAR vocabularies (Shotton & Peroni), not here.
//! - **Locator** — the pinpoint structure (title/section/paragraph/
//!   page/line) the `LocatorAccuracy` dimension grades. Grounded in the
//!   [`crate::social::judicial::citation`] `PinpointCitation` ontology
//!   over the USLM subdivision hierarchy.
//! - **Provenance** — who/what/when/how/why a citation was verified.
//!   Grounded in W3C PROV-O plus a Toulmin (1958) warrant for the *why*.
//! - **Defects** — the typed, per-dimension issues (wrong section,
//!   misquote, reference error, style break), each carrying the
//!   severity of the dimension it lands on.
//!
//! Functors out of this ontology (to English for explanation, to a
//! Communication ontology for chat surfaces) and the lens between this
//! model and `citations.toml` are built on top of the dimensions
//! defined here.
//!
//! # Literature
//!
//! See [`ontology`] for the full citation list (ISO/IEC 25012:2008;
//! Wang & Strong 1996; Sarol et al. 2024; Guyatt et al. 2008 GRADE).

pub mod assessment;
pub mod cito_functors;
pub mod english_projection;
pub mod ontology;
pub mod record_lens;
pub mod registry_projection;

pub use assessment::{
    CitationVerdict, DimensionStatus, VerdictMeetIsBoundedSemilattice, VerificationMethod, assess,
    dimension_verdict,
};
pub use ontology::{
    CitationQualityConcept, CitationQualityOntology, SEVERITY_BLOCKING, SEVERITY_INFO,
    SEVERITY_WARNING, Severity, SeverityPartitionsDimensions,
    SoundGateIsExactlyExistenceAndClaimSupport, dimensions, is_dimension, is_sound_gate,
};
pub use record_lens::CitationAssessment;
pub use registry_projection::{
    EntryFields, VersionFiber, parse_verification_method, project_entry,
};
