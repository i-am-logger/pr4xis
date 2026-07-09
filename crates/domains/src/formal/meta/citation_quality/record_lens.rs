//! Lens: `CitationRecord ⇄ CitationAssessment` (Foster et al. 2007).
//!
//! A citation registry entry carries two separable parts: its
//! *bibliographic complement* (the slug + author/title/year/locator
//! data identifying the cited work) and its *quality assessment* (the
//! per-dimension [`DimensionStatus`] the CitationQuality model grades).
//! This lens **focuses the assessment**: `get` reads it; `put` writes an
//! updated assessment back while preserving the complement untouched
//! (Bancilhon & Spyratos 1981 *constant complement*).
//!
//! Why a lens and not a one-way function: the audit gate reads the
//! assessment (`get`) to compute a verdict, while the migration writes a
//! human/machine assessment into an entry (`put`) without disturbing its
//! bibliography. The well-behaved-lens laws (Foster et al. 2007 §3, Definition 3.2)
//! guarantee those two directions stay consistent — reading back a
//! just-written assessment yields it (PutGet), and writing back an
//! unchanged assessment is a no-op (GetPut). The gate then *composes*
//! through the lens — `lens.get(record).verdict()` — instead of
//! inspecting raw fields.
//!
//! This is an instance of the general [`Lens`] trait, so it composes
//! (via [`crate::formal::meta::lens_composition::lens::Compose`]) with a
//! future `toml-bytes ⇄ CitationRecord` parse hop and with the verdict
//! fold, forming one `bytes ⇄ record ⇄ assessment` chain.
//!
//! ## Citation
//!
//! - Foster, J. N., Greenwald, M. B., Moore, J. T., Pierce, B. C. &
//!   Schmitt, A. (2007) "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3) Art. 17, §3, Definition 3.2 (lens laws).
//! - Bancilhon, F. & Spyratos, N. (1981) "Update Semantics of
//!   Relational Views", *ACM TODS* 6(4) (constant complement).

#[allow(unused_imports)]
use alloc::{collections::BTreeMap, string::String, vec::Vec};

use super::assessment::{CitationVerdict, DimensionStatus, VerificationMethod, assess};
use super::ontology::CitationQualityConcept;
use crate::formal::meta::lens_composition::lens::Lens;

/// The quality assessment of a citation — one [`DimensionStatus`] per
/// CitationQuality dimension, plus the method by which the assessment
/// was established. The *view* focused by [`CitationAssessmentLens`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationAssessment {
    pub existence: DimensionStatus,
    pub claim_support: DimensionStatus,
    pub locator_accuracy: DimensionStatus,
    pub bibliographic_accuracy: DimensionStatus,
    pub format_conformance: DimensionStatus,
    /// How the assessment was established (Daubert reliability order).
    pub method: VerificationMethod,
}

impl CitationAssessment {
    /// The per-dimension statuses keyed by their ontology concept — the
    /// input to [`assess`].
    pub fn statuses(&self) -> [(CitationQualityConcept, DimensionStatus); 5] {
        use CitationQualityConcept as C;
        [
            (C::Existence, self.existence),
            (C::ClaimSupport, self.claim_support),
            (C::LocatorAccuracy, self.locator_accuracy),
            (C::BibliographicAccuracy, self.bibliographic_accuracy),
            (C::FormatConformance, self.format_conformance),
        ]
    }

    /// The composed verdict for this assessment (the monoid fold of its
    /// per-dimension verdicts).
    pub fn verdict(&self) -> CitationVerdict {
        assess(&self.statuses())
    }
}

/// A citation registry entry: the bibliographic complement (slug +
/// arbitrary identifying key/value fields) plus its quality assessment.
/// The complement is opaque to the lens — it is preserved verbatim
/// across `put` (Bancilhon & Spyratos 1981).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationRecord {
    /// The registry slug (e.g. `mac_lane_1971_...`).
    pub slug: String,
    /// Bibliographic + provenance fields identifying the cited work
    /// (author, title, year, section_or_page, …). Opaque complement.
    pub bibliography: BTreeMap<String, String>,
    /// The per-dimension quality assessment — the lens's focus.
    pub assessment: CitationAssessment,
}

/// The lens focusing a [`CitationRecord`]'s assessment. `get` reads the
/// assessment; `put` replaces it, keeping the slug + bibliography
/// complement (Foster et al. 2007 §3, Definition 3.2; Bancilhon & Spyratos 1981).
#[derive(Debug, Clone, Copy, Default)]
pub struct CitationAssessmentLens;

impl Lens for CitationAssessmentLens {
    type Source = CitationRecord;
    type View = CitationAssessment;
    type Error = core::convert::Infallible;

    fn get(&self, source: &CitationRecord) -> Result<CitationAssessment, Self::Error> {
        Ok(source.assessment.clone())
    }

    fn put(
        &self,
        view: &CitationAssessment,
        source: &CitationRecord,
    ) -> Result<CitationRecord, Self::Error> {
        Ok(CitationRecord {
            slug: source.slug.clone(),
            bibliography: source.bibliography.clone(),
            assessment: view.clone(),
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::meta::lens_composition::lens::{
        get_put_holds, put_get_holds, put_put_holds,
    };

    fn verified_assessment() -> CitationAssessment {
        CitationAssessment {
            existence: DimensionStatus::Verified,
            claim_support: DimensionStatus::Verified,
            locator_accuracy: DimensionStatus::Verified,
            bibliographic_accuracy: DimensionStatus::Verified,
            format_conformance: DimensionStatus::Verified,
            method: VerificationMethod::MachineChecked,
        }
    }

    fn sample_record() -> CitationRecord {
        let mut bib = BTreeMap::new();
        bib.insert(
            "title".into(),
            "Categories for the Working Mathematician".into(),
        );
        bib.insert("year".into(), "1971".into());
        CitationRecord {
            slug: "mac_lane_1971".into(),
            bibliography: bib,
            assessment: verified_assessment(),
        }
    }

    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn lens_is_well_behaved_on_sample() {
        let r = sample_record();
        let other = CitationAssessment {
            claim_support: DimensionStatus::Unverified,
            ..verified_assessment()
        };
        assert!(get_put_holds(&CitationAssessmentLens, &r));
        assert!(put_get_holds(&CitationAssessmentLens, &other, &r));
        assert!(put_put_holds(
            &CitationAssessmentLens,
            &other,
            &verified_assessment(),
            &r
        ));
    }

    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn put_preserves_complement() {
        let r = sample_record();
        let degraded = CitationAssessment {
            locator_accuracy: DimensionStatus::Unverified,
            ..verified_assessment()
        };
        let r2 = CitationAssessmentLens.put(&degraded, &r).unwrap();
        // The bibliography complement is untouched; only the assessment changed.
        assert_eq!(r2.slug, r.slug);
        assert_eq!(r2.bibliography, r.bibliography);
        assert_eq!(r2.assessment, degraded);
    }

    #[pr4xis::praxis_value(Extensible, Verifiable)]
    #[test]
    fn gate_composes_through_the_lens() {
        // All-verified record reads through to a Valid verdict.
        let r = sample_record();
        assert_eq!(
            CitationAssessmentLens.get(&r).unwrap().verdict(),
            CitationVerdict::Valid
        );
        // A record whose claim-support is unconfirmed reads Invalid —
        // without the gate inspecting any raw field.
        let bad = CitationRecord {
            assessment: CitationAssessment {
                claim_support: DimensionStatus::Unverified,
                ..verified_assessment()
            },
            ..sample_record()
        };
        assert_eq!(
            CitationAssessmentLens.get(&bad).unwrap().verdict(),
            CitationVerdict::Invalid
        );
    }

    // ── Property-based laws (Foster et al. 2007 §3, Definition 3.2) ──────────────
    use proptest::prelude::*;

    fn arb_status() -> impl Strategy<Value = DimensionStatus> {
        prop_oneof![
            Just(DimensionStatus::Verified),
            Just(DimensionStatus::Unverified),
        ]
    }

    fn arb_method() -> impl Strategy<Value = VerificationMethod> {
        prop_oneof![
            Just(VerificationMethod::Unverified),
            Just(VerificationMethod::HumanAttested),
            Just(VerificationMethod::MachineChecked),
        ]
    }

    fn arb_assessment() -> impl Strategy<Value = CitationAssessment> {
        (
            arb_status(),
            arb_status(),
            arb_status(),
            arb_status(),
            arb_status(),
            arb_method(),
        )
            .prop_map(|(e, c, l, b, f, m)| CitationAssessment {
                existence: e,
                claim_support: c,
                locator_accuracy: l,
                bibliographic_accuracy: b,
                format_conformance: f,
                method: m,
            })
    }

    fn arb_record() -> impl Strategy<Value = CitationRecord> {
        (
            "[a-z][a-z0-9_]{0,12}",
            proptest::collection::btree_map("[a-z]{1,6}", "[a-z0-9 ]{0,12}", 0..4),
            arb_assessment(),
        )
            .prop_map(|(slug, bibliography, assessment)| CitationRecord {
                slug,
                bibliography,
                assessment,
            })
    }

    proptest! {
        #[test]
        fn prop_get_put(r in arb_record()) {
            prop_assert!(get_put_holds(&CitationAssessmentLens, &r));
        }

        #[test]
        fn prop_put_get(r in arb_record(), v in arb_assessment()) {
            prop_assert!(put_get_holds(&CitationAssessmentLens, &v, &r));
        }

        #[test]
        fn prop_put_put(r in arb_record(), v1 in arb_assessment(), v2 in arb_assessment()) {
            prop_assert!(put_put_holds(&CitationAssessmentLens, &v1, &v2, &r));
        }

        /// The verdict read through the lens equals the direct fold of
        /// the focused assessment's statuses — the gate's composition.
        #[test]
        fn prop_gate_composition(r in arb_record()) {
            let v = CitationAssessmentLens.get(&r).unwrap();
            prop_assert_eq!(v.verdict(), assess(&v.statuses()));
        }
    }

    pr4xis::register_praxis_value!(prop_get_put, Deterministic);
    pr4xis::register_praxis_value!(prop_put_get, Deterministic);
    pr4xis::register_praxis_value!(prop_put_put, Deterministic);
    pr4xis::register_praxis_value!(prop_gate_composition, Extensible, Deterministic);
}
