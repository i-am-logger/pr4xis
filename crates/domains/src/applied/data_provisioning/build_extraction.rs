//! Typed outcomes of the build-time PDF text extraction.
//!
//! Each registered statute's generated codegen module
//! (`$OUT_DIR/<name>_codegen.rs`, built by `crates/domains/build.rs`)
//! emits a single `pub const PDF_EXTRACTION: PdfBuildExtraction`
//! whose variant names the exact state of the source-of-truth
//! PDF on disk at build time. Downstream code (currently
//! `canonical_audit.rs`; Phase 8 wires it) pattern-matches on
//! the variant rather than reading `Option<&str>` — every state
//! is named, no silent gaps.
//!
//! ## Literature grounding
//!
//! The "build-time extraction outcome" model is a typed
//! `prov:Activity` outcome in the W3C PROV-O sense (Lebo et al.
//! 2013, *PROV-O: The PROV Ontology*) — the build is an
//! activity acting on the PDF artifact, producing either a
//! successful derivation ([`PdfBuildExtraction::Extracted`]) or
//! one of several typed non-success states. Anchored against
//! the existing provenance subsystem at
//! `formal/information/provenance/` (cited in
//! `applied::data_provisioning` module docs).
//!
//! Per-variant spec citations:
//!
//! - [`Extracted`](PdfBuildExtraction::Extracted) — ISO
//!   32000-2:2020 §9 (Text). Successful decoding of the PDF's
//!   text-showing operators.
//! - [`NotOnDisk`](PdfBuildExtraction::NotOnDisk) — Wilkinson
//!   et al. 2016, *The FAIR Guiding Principles* (Nature Scientific
//!   Data 3:160018), Principle A1: data must be retrievable by
//!   their identifier. When the local artifact is absent, A1
//!   is unsatisfied; this variant names that fact.
//! - [`ParseFailed`](PdfBuildExtraction::ParseFailed) — ISO
//!   32000-2:2020 §7.5 (File structure). The bytes don't
//!   conform to the file-structure grammar.
//! - [`Encrypted`](PdfBuildExtraction::Encrypted) — ISO
//!   32000-2:2020 §7.6 (Encryption). The document declares
//!   `/Encrypt`; decryption is out of scope for this build path.
//! - [`UnsupportedContentType`](PdfBuildExtraction::UnsupportedContentType)
//!   — Wilkinson et al. 2016, Principle R1.3 ("data meet
//!   domain-relevant community standards") — when the registered
//!   content-type isn't one the build extractor understands,
//!   this variant surfaces the gap explicitly rather than
//!   degrading silently.
//!
//! Praxis-rule references:
//!
//! - `feedback_no_silent_failures` — every state is named;
//!   never `None` / empty string.
//! - `feedback_pdf_text_only_until_image_understanding` — the
//!   typed gap surfaces image-only / encrypted PDFs as their own
//!   variants rather than as empty text.
//!
//! The [`PdfBuildExtractionTotality`] axiom enforces structurally
//! that the enum is the *complete* response surface for the
//! build extractor: any future build-time outcome must extend
//! this enum, not return a generic `Option` or `Result` to
//! downstream consumers.

/// State of a statute's PDF text at build time.
///
/// Constructed by `crates/domains/build.rs` for each registered
/// statute. Variants are ordered by the "happy path" probability
/// (Extracted first; failures named explicitly thereafter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfBuildExtraction {
    /// Build-time extractor walked the PDF's content streams and
    /// produced the embedded text. `bytes_hash` is the SHA-256 of
    /// the source PDF bytes at extraction time; downstream tests
    /// can assert it matches `praxis.lock`'s pinned hash.
    Extracted {
        /// The decoded text. Lifetime is `'static` because the
        /// codegen emits a string literal compiled into the
        /// binary.
        text: &'static str,
        /// SHA-256 of the source PDF, as a lowercase hex string.
        bytes_hash: &'static str,
    },
    /// PDF artifact wasn't on disk at build time. Common when
    /// the workspace was built without first running
    /// `pr4xis update <name>` to fetch the source bytes. Not
    /// fatal at compile time; downstream tests can decide whether
    /// to skip or fail.
    NotOnDisk,
    /// PDF was on disk but `lopdf::Document::load_mem` rejected
    /// the bytes. The `detail` carries the lopdf diagnostic.
    /// Indicates either a corrupted artifact or an upstream-
    /// changed PDF that needs re-pinning in `praxis.lock`.
    ParseFailed {
        /// Lopdf parse-error diagnostic.
        detail: &'static str,
    },
    /// PDF declared `/Encrypt` (ISO 32000-2 §7.6). PDF
    /// encryption support is forward work; an encrypted statute
    /// PDF can't be decoded by this build path until that lands.
    Encrypted,
    /// Statute is registered as `type = "Statute"` /
    /// `"UsFederalStatute"` but its on-disk content type isn't
    /// PDF. Reserved for future content-type evolutions; today
    /// this variant doesn't occur because every statute kind
    /// resolves to `ContentType::Pdf`.
    UnsupportedContentType {
        /// The content-type name the build script saw.
        actual: &'static str,
    },
}

impl PdfBuildExtraction {
    /// Borrow the extracted text, returning `None` for every
    /// non-`Extracted` variant. Used by audit modules that
    /// need the bytes for further validation; the typed gap is
    /// still visible via the variant.
    pub fn text(&self) -> Option<&'static str> {
        match self {
            Self::Extracted { text, .. } => Some(text),
            _ => None,
        }
    }

    /// Whether extraction succeeded. Useful in tests:
    /// `assert!(PDF_EXTRACTION.is_extracted())`.
    pub fn is_extracted(&self) -> bool {
        matches!(self, Self::Extracted { .. })
    }

    /// Stable string tag for the variant name — used by
    /// downstream provenance audits to log which state was
    /// observed. Mirrors W3C PROV-O `prov:Activity` outcome
    /// naming convention.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Extracted { .. } => "Extracted",
            Self::NotOnDisk => "NotOnDisk",
            Self::ParseFailed { .. } => "ParseFailed",
            Self::Encrypted => "Encrypted",
            Self::UnsupportedContentType { .. } => "UnsupportedContentType",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Axioms — enforce the typed-completeness invariant
// ─────────────────────────────────────────────────────────────────────

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

/// Structural axiom: [`PdfBuildExtraction`] is exhaustive over
/// every build-time PDF extraction outcome. Adding a new outcome
/// state must extend this enum, not bypass it with `Option` or
/// untyped `String` markers.
///
/// W3C PROV-O (Lebo et al. 2013) models every `prov:Activity` as
/// having a determinate outcome; this axiom asserts the praxis
/// implementation enforces that at the type level.
///
/// Structural — the property is satisfied by the type being an
/// `enum` rather than a wrapper around `Option`. The axiom records
/// the invariant at the ontology layer.
pub struct PdfBuildExtractionTotality;

impl Axiom for PdfBuildExtractionTotality {
    fn verify(&self) -> Verdict {
        // Structural — enforced by the type system. Constructing a
        // `PdfBuildExtraction` requires picking one of the five
        // named variants; the compiler rejects any other shape.
        // We also exercise `variant_name` on each variant to
        // confirm the predicate is total over the surface.
        let probes = [
            PdfBuildExtraction::Extracted {
                text: "",
                bytes_hash: "",
            },
            PdfBuildExtraction::NotOnDisk,
            PdfBuildExtraction::ParseFailed { detail: "" },
            PdfBuildExtraction::Encrypted,
            PdfBuildExtraction::UnsupportedContentType { actual: "" },
        ];
        for p in &probes {
            // variant_name() is total by construction; this call
            // is the runtime witness that the predicate covers
            // every variant.
            let _ = p.variant_name();
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PdfBuildExtractionTotality",
        "PdfBuildExtraction is exhaustive over every build-time PDF extraction outcome",
        "W3C PROV-O (Lebo et al. 2013); praxis feedback_no_silent_failures"
    );
}
pr4xis::register_axiom!(
    PdfBuildExtractionTotality,
    "W3C PROV-O (Lebo et al. 2013); praxis feedback_no_silent_failures"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracted_variant_carries_text_and_hash() {
        let e = PdfBuildExtraction::Extracted {
            text: "hello",
            bytes_hash: "abc",
        };
        assert_eq!(e.text(), Some("hello"));
        assert!(e.is_extracted());
    }

    #[test]
    fn not_on_disk_variant_has_no_text() {
        let e = PdfBuildExtraction::NotOnDisk;
        assert_eq!(e.text(), None);
        assert!(!e.is_extracted());
    }

    #[test]
    fn parse_failed_variant_carries_detail_but_no_text() {
        let e = PdfBuildExtraction::ParseFailed { detail: "bad xref" };
        assert_eq!(e.text(), None);
        assert!(!e.is_extracted());
    }

    #[test]
    fn encrypted_variant_has_no_text() {
        let e = PdfBuildExtraction::Encrypted;
        assert_eq!(e.text(), None);
        assert!(!e.is_extracted());
    }

    #[test]
    fn unsupported_content_type_variant_carries_actual_kind() {
        let e = PdfBuildExtraction::UnsupportedContentType {
            actual: "Plaintext",
        };
        assert_eq!(e.text(), None);
        assert!(!e.is_extracted());
    }

    #[test]
    fn variant_name_covers_every_variant() {
        let probes = [
            (
                PdfBuildExtraction::Extracted {
                    text: "",
                    bytes_hash: "",
                },
                "Extracted",
            ),
            (PdfBuildExtraction::NotOnDisk, "NotOnDisk"),
            (
                PdfBuildExtraction::ParseFailed { detail: "" },
                "ParseFailed",
            ),
            (PdfBuildExtraction::Encrypted, "Encrypted"),
            (
                PdfBuildExtraction::UnsupportedContentType { actual: "" },
                "UnsupportedContentType",
            ),
        ];
        for (variant, expected) in probes {
            assert_eq!(variant.variant_name(), expected);
        }
    }

    #[test]
    fn axiom_pdf_build_extraction_totality_verifies() {
        assert!(PdfBuildExtractionTotality.verify().is_ok());
    }

    #[test]
    fn axiom_carries_expected_citation() {
        let cit = PdfBuildExtractionTotality.meta().citation;
        let s = cit.as_str();
        assert!(s.contains("PROV-O"), "expected PROV-O citation; got {s}");
        assert!(s.contains("feedback_no_silent_failures"));
    }

    // ── Property-based ────────────────────────────────────────────

    use proptest::prelude::*;

    fn arb_extraction() -> impl Strategy<Value = PdfBuildExtraction> {
        prop_oneof![
            Just(PdfBuildExtraction::NotOnDisk),
            Just(PdfBuildExtraction::Encrypted),
            "[a-z ]{0,16}".prop_map(|s| PdfBuildExtraction::ParseFailed {
                detail: Box::leak(s.into_boxed_str()),
            }),
            "[A-Za-z]{0,16}".prop_map(|s| PdfBuildExtraction::UnsupportedContentType {
                actual: Box::leak(s.into_boxed_str()),
            }),
            ("[A-Za-z0-9 .,!?]{0,64}", "[a-f0-9]{64}").prop_map(|(text, hash)| {
                PdfBuildExtraction::Extracted {
                    text: Box::leak(text.into_boxed_str()),
                    bytes_hash: Box::leak(hash.into_boxed_str()),
                }
            }),
        ]
    }

    proptest! {
        /// `text()` returns `Some` if and only if the variant is
        /// `Extracted`. Cross-variant invariant — no other state
        /// claims to carry text.
        #[test]
        fn prop_text_is_some_iff_extracted(e in arb_extraction()) {
            prop_assert_eq!(e.text().is_some(), e.is_extracted());
        }

        /// `is_extracted()` is a pure predicate: calling it twice
        /// returns the same answer.
        #[test]
        fn prop_is_extracted_is_pure(e in arb_extraction()) {
            prop_assert_eq!(e.is_extracted(), e.is_extracted());
        }

        /// Equality is reflexive — every variant equals itself.
        /// Catches accidental floating-point or non-deterministic
        /// field introduction.
        #[test]
        fn prop_equality_is_reflexive(e in arb_extraction()) {
            prop_assert_eq!(e.clone(), e);
        }
    }
}
