//! Citation-quality ontology — concepts, is_a, severity, axioms.
//!
//! Models the assessed quality of a literature citation as a small set
//! of *independently-verifiable dimensions* rather than a single
//! boolean. Each dimension carries a severity (how badly a defect on it
//! undermines the citation), which partitions the dimensions into a
//! **sound-gate** pair (existence, claim-support — a defect here makes
//! the citation invalid) and lower dimensions whose defects are
//! recorded but non-blocking (locator, bibliographic, format).
//!
//! This is the hand-authored core of the citation-quality model. The
//! *relationship* axis (quotation vs paraphrase, cites-as-evidence, …)
//! is grounded separately in the loaded CiTO/SPAR vocabularies; the
//! verification *provenance* (who/what/when/how/why) in W3C PROV-O;
//! the per-defect typing + the English / Communication functors wire
//! on top of these dimensions. See `mod.rs` for the layering.
//!
//! # Literature
//!
//! - **ISO/IEC 25012:2008** *Software engineering — Software product
//!   Quality Requirements and Evaluation (SQuaRE) — Data quality
//!   model.* Decomposes data quality into independent inherent
//!   characteristics (accuracy, completeness, credibility, …); the
//!   inherent/system-dependent split grounds "is the source sound" vs
//!   "is our record of it clean".
//! - **Wang, R. Y. & Strong, D. M. (1996)** "Beyond Accuracy: What
//!   Data Quality Means to Data Consumers", *Journal of Management
//!   Information Systems* 12(4):5–33 — intrinsic data-quality
//!   dimensions (accuracy, believability).
//! - **Sarol, M. J. et al. (2024)** "Assessing citation integrity in
//!   biomedical publications: corpus annotation and NLP models",
//!   *Bioinformatics* 40(7):btae420 — the major (CONTRADICT,
//!   NOT_SUBSTANTIATE, IRRELEVANT) vs minor (OVERSIMPLIFY, MISQUOTE,
//!   INDIRECT) content-error severity ordering, distinguished from the
//!   reference (bibliographic) error class.
//! - **Guyatt, G. H. et al. (2008)** "GRADE: an emerging consensus on
//!   rating quality of evidence and strength of recommendations",
//!   *BMJ* 336(7650):924–926 — graded certainty with downgrade logic;
//!   grounds the severity ordering.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "CitationQuality",
    source: "ISO/IEC 25012:2008 (Data quality model); Wang & Strong (1996) J. MIS 12(4):5-33; Sarol et al. (2024) Bioinformatics 40(7):btae420; Guyatt et al. (2008) BMJ 336(7650):924-926",

    concepts: [
        // Root
        CitationQuality,

        // The verifiable dimensions, highest severity first.
        Existence,
        ClaimSupport,
        LocatorAccuracy,
        BibliographicAccuracy,
        FormatConformance,
    ],

    labels: {
        CitationQuality: ("en", "Citation quality",
            "The assessed quality of a literature citation, decomposed into independently-verifiable dimensions rather than a single boolean (ISO/IEC 25012:2008 data-quality model; Wang & Strong 1996)."),
        Existence: ("en", "Existence",
            "Whether the cited work exists at all, as opposed to being fabricated. ISO/IEC 25012 credibility. Sound-gate: a defect here makes the citation invalid."),
        ClaimSupport: ("en", "Claim support",
            "Whether the cited work actually supports the asserted claim. Sarol et al. 2024 (ACCURATE vs CONTRADICT/NOT_SUBSTANTIATE/IRRELEVANT); ISO/IEC 25012 accuracy. Sound-gate."),
        LocatorAccuracy: ("en", "Locator accuracy",
            "Whether the pinpoint locator (section/paragraph/page/line) resolves to the right place in the work. Wang & Strong 1996 intrinsic accuracy. Non-blocking when the source + claim are sound."),
        BibliographicAccuracy: ("en", "Bibliographic accuracy",
            "Whether author, title, edition, and year are correct. The reference-error class distinguished from content errors by Sarol et al. 2024. Non-blocking."),
        FormatConformance: ("en", "Format conformance",
            "Whether the citation conforms to the required style. Sarol et al. 2024 ETIQUETTE class; lowest severity (informational)."),
    },

    is_a: [
        (Existence, CitationQuality),
        (ClaimSupport, CitationQuality),
        (LocatorAccuracy, CitationQuality),
        (BibliographicAccuracy, CitationQuality),
        (FormatConformance, CitationQuality),
    ],
}

/// The five verifiable dimensions (every concept except the root).
pub fn dimensions() -> [CitationQualityConcept; 5] {
    [
        CitationQualityConcept::Existence,
        CitationQualityConcept::ClaimSupport,
        CitationQualityConcept::LocatorAccuracy,
        CitationQualityConcept::BibliographicAccuracy,
        CitationQualityConcept::FormatConformance,
    ]
}

/// True for the leaf dimensions, false for the `CitationQuality` root.
pub fn is_dimension(c: CitationQualityConcept) -> bool {
    !matches!(c, CitationQualityConcept::CitationQuality)
}

// ---------------------------------------------------------------------------
// Quality: Severity — how badly a defect on a dimension undermines the cite.
//
// 2 = Blocking  (sound-gate: existence, claim_support)
// 1 = Warning   (locator, bibliographic)
// 0 = Info      (format)
//
// The Blocking set is exactly the two dimensions whose failure means the
// citation cannot stand at all; Warning/Info dimensions carry defects
// that are recorded but do not invalidate a sound source + claim. The
// ordering follows GRADE's downgrade logic (Guyatt et al. 2008).
// ---------------------------------------------------------------------------

/// Severity ordinal of a citation-quality dimension. Info(0) < Warning(1)
/// < Blocking(2).
#[derive(Debug, Clone)]
pub struct Severity;

/// Highest severity: a defect here invalidates the citation. The
/// sound-gate dimensions map to this.
pub const SEVERITY_BLOCKING: u8 = 2;
/// A defect here is recorded as a non-blocking issue.
pub const SEVERITY_WARNING: u8 = 1;
/// A defect here is informational only.
pub const SEVERITY_INFO: u8 = 0;

impl Quality for Severity {
    type Individual = CitationQualityConcept;
    type Value = u8;

    fn get(&self, c: &CitationQualityConcept) -> Option<u8> {
        use CitationQualityConcept as C;
        match c {
            C::Existence | C::ClaimSupport => Some(SEVERITY_BLOCKING),
            C::LocatorAccuracy | C::BibliographicAccuracy => Some(SEVERITY_WARNING),
            C::FormatConformance => Some(SEVERITY_INFO),
            C::CitationQuality => None,
        }
    }
}

/// True iff a defect on `dim` invalidates the citation (severity
/// Blocking). These are exactly the two sound-gate dimensions.
pub fn is_sound_gate(dim: CitationQualityConcept) -> bool {
    Severity.get(&dim) == Some(SEVERITY_BLOCKING)
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

impl Ontology for CitationQualityOntology {
    type Cat = CitationQualityCategory;
    type Qual = Severity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SeverityPartitionsDimensions));
        axioms.push(Box::new(SoundGateIsExactlyExistenceAndClaimSupport));
        axioms
    }
}

/// Axiom: every dimension has a defined severity in {Info, Warning,
/// Blocking}, and the root has none. No dimension is left ungraded.
pub struct SeverityPartitionsDimensions;

impl Axiom for SeverityPartitionsDimensions {
    fn verify(&self) -> Verdict {
        let q = Severity;
        for dim in dimensions() {
            match q.get(&dim) {
                Some(SEVERITY_INFO | SEVERITY_WARNING | SEVERITY_BLOCKING) => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        if q.get(&CitationQualityConcept::CitationQuality).is_some() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SeverityPartitionsDimensions",
        "Every citation-quality dimension is graded Info/Warning/Blocking; the root is ungraded",
        "ISO/IEC 25012:2008 (data-quality dimensions); Guyatt et al. (2008) GRADE, BMJ 336(7650):924-926"
    );
}

pr4xis::register_axiom!(
    SeverityPartitionsDimensions,
    "ISO/IEC 25012:2008; Guyatt et al. (2008) GRADE BMJ 336(7650):924-926"
);

/// Axiom: the Blocking (sound-gate) dimensions are exactly existence and
/// claim-support. A citation with a sound source and a supported claim
/// is valid even if lower dimensions carry defects; conversely a defect
/// on either of these two invalidates it.
pub struct SoundGateIsExactlyExistenceAndClaimSupport;

impl Axiom for SoundGateIsExactlyExistenceAndClaimSupport {
    fn verify(&self) -> Verdict {
        use CitationQualityConcept as C;
        for dim in dimensions() {
            let blocking = is_sound_gate(dim);
            let expected = matches!(dim, C::Existence | C::ClaimSupport);
            if blocking != expected {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SoundGateIsExactlyExistenceAndClaimSupport",
        "The Blocking dimensions are exactly {Existence, ClaimSupport}; locator/bibliographic/format are non-blocking",
        "Sarol et al. (2024) Bioinformatics 40(7):btae420 (major vs minor citation errors); ISO/IEC 25012:2008"
    );
}

pr4xis::register_axiom!(
    SoundGateIsExactlyExistenceAndClaimSupport,
    "Sarol et al. (2024) Bioinformatics 40(7):btae420"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ontology_validates() {
        for axiom in CitationQualityOntology::axioms() {
            assert!(
                axiom.verify().is_ok(),
                "axiom failed: {}",
                axiom.meta().name
            );
        }
    }

    #[test]
    fn five_dimensions_all_graded() {
        assert_eq!(dimensions().len(), 5);
        for dim in dimensions() {
            assert!(Severity.get(&dim).is_some(), "{dim:?} ungraded");
            assert!(is_dimension(dim));
        }
        assert!(!is_dimension(CitationQualityConcept::CitationQuality));
        assert!(
            Severity
                .get(&CitationQualityConcept::CitationQuality)
                .is_none()
        );
    }

    #[test]
    fn sound_gate_is_existence_and_claim_support() {
        use CitationQualityConcept as C;
        assert!(is_sound_gate(C::Existence));
        assert!(is_sound_gate(C::ClaimSupport));
        assert!(!is_sound_gate(C::LocatorAccuracy));
        assert!(!is_sound_gate(C::BibliographicAccuracy));
        assert!(!is_sound_gate(C::FormatConformance));
    }

    #[test]
    fn severity_strictly_decreases_across_tiers() {
        use CitationQualityConcept as C;
        let blocking = Severity.get(&C::Existence).expect("existence graded");
        let warning = Severity.get(&C::LocatorAccuracy).expect("locator graded");
        let info = Severity.get(&C::FormatConformance).expect("format graded");
        assert!(blocking > warning, "sound-gate must outrank locator");
        assert!(warning > info, "locator must outrank format");
    }

    // ── Property-based ─────────────────────────────────────────────
    use pr4xis::category::Concept;
    use proptest::prelude::*;

    fn arb_concept() -> impl Strategy<Value = CitationQualityConcept> {
        proptest::sample::select(CitationQualityConcept::variants())
    }

    proptest! {
        /// Grading is total on dimensions and undefined on the root,
        /// and every defined severity is one of the three tiers
        /// (ISO/IEC 25012; GRADE).
        #[test]
        fn prop_grading_total_on_dimensions(c in arb_concept()) {
            match Severity.get(&c) {
                Some(v) => {
                    prop_assert!(matches!(
                        v,
                        SEVERITY_INFO | SEVERITY_WARNING | SEVERITY_BLOCKING
                    ));
                    prop_assert!(is_dimension(c));
                }
                None => prop_assert_eq!(c, CitationQualityConcept::CitationQuality),
            }
        }

        /// A dimension is a sound-gate iff it is graded Blocking.
        #[test]
        fn prop_sound_gate_iff_blocking(c in arb_concept()) {
            prop_assert_eq!(is_sound_gate(c), Severity.get(&c) == Some(SEVERITY_BLOCKING));
        }
    }
}
