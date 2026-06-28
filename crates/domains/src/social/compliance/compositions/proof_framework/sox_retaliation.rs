//! The SOX whistleblower-retaliation proof framework — typed
//! composition of `sox_1514a@2002` (the substantive prohibition and
//! procedural posture) with `air21_42121@2010` (the burden-shifting
//! framework imported by reference per § 1514A(b)(2)(C)).
//!
//! Three statutory cross-references connect SOX's terms to AIR21's
//! framework. Each is grounded in the express cross-reference
//! language of § 1514A(b)(2)(A) or (C). CURIEs follow the USLM URN
//! path convention (slashes → underscores):
//!
//! 1. `sox_1514a:b_2_A` Requires `air21_42121:b_2` — § 1514A(b)(2)(A)
//!    governs SOX actions by the AIR21 procedures.
//! 2. `sox_1514a:b_2_C` Requires `air21_42121:b_2_B` — § 1514A(b)(2)(C)
//!    governs the burdens of proof in SOX actions by the AIR21
//!    four-clause framework.
//! 3. `sox_1514a:a` Implies `air21_42121:b_2_B_iii` — SOX's
//!    "because of" causation element in § 1514A(a) is substantively
//!    realized through AIR21's merits contributing-factor demonstration.
//!
//! When the PDF loader lands and case-law decisions become loadable
//! (Murray v. UBS, Lawson v. FMR, Sylvester v. Parexel, Marano v.
//! DOT, Burlington v. White), additional cross-references will be
//! added connecting their holdings to the SOX/AIR21 statutory terms.
//! Until then, this framework captures the statute-only synthesis.

use std::sync::OnceLock;

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use hashbrown::HashMap;

use super::{CrossReference, CrossReferenceKind, ProofFramework};
use crate::formal::meta::identifier_format::Identifier;
use crate::social::compliance::statutes::{Statute, air21_42121, sox_1514a};
use crate::social::judicial::authority_strength::ontology::AuthorityStrengthConcept;
use crate::social::judicial::source_text::SourceTextRef;

/// The SOX whistleblower-retaliation proof framework. Composes
/// `sox_1514a@2002` and `air21_42121@2010` per the cross-reference
/// language at 18 U.S.C. § 1514A(b)(2)(A)–(C).
///
/// Lazily constructed on first access; cached for the process
/// lifetime. Panics on construction error — every cross-reference
/// endpoint must resolve in the bundled statutes, which is guaranteed
/// by the structural data shipped in `praxis.lock`.
pub fn framework() -> &'static ProofFramework {
    static INSTANCE: OnceLock<ProofFramework> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let statutes: Vec<&'static Statute> =
            vec![sox_1514a::statute(), air21_42121::statute()];

        let mut authority_strengths = HashMap::new();
        authority_strengths.insert(
            "sox_1514a".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );
        authority_strengths.insert(
            "air21_42121".to_string(),
            AuthorityStrengthConcept::FederalStatute,
        );

        let cross_references = vec![
            CrossReference {
                from_source: "sox_1514a".to_string(),
                from_term: Identifier::curie("sox_1514a:b_2_A")
                    .expect("valid CURIE"),
                kind: CrossReferenceKind::Requires,
                to_source: "air21_42121".to_string(),
                to_term: Identifier::curie("air21_42121:b_2")
                    .expect("valid CURIE"),
                rationale: SourceTextRef::with_context(
                    "18 U.S.C. § 1514A(b)(2)(A): SOX actions \"shall be governed by\" the rules and procedures set forth in 49 U.S.C. § 42121(b).",
                    "uslm-crossref://sox_1514a@2002+air21_42121@2010",
                ),
            },
            CrossReference {
                from_source: "sox_1514a".to_string(),
                from_term: Identifier::curie("sox_1514a:b_2_C")
                    .expect("valid CURIE"),
                kind: CrossReferenceKind::Requires,
                to_source: "air21_42121".to_string(),
                to_term: Identifier::curie("air21_42121:b_2_B")
                    .expect("valid CURIE"),
                rationale: SourceTextRef::with_context(
                    "18 U.S.C. § 1514A(b)(2)(C): SOX district-court actions \"shall be governed by the legal burdens of proof set forth in section 42121(b) of title 49\" — the four-clause framework.",
                    "uslm-crossref://sox_1514a@2002+air21_42121@2010",
                ),
            },
            CrossReference {
                from_source: "sox_1514a".to_string(),
                from_term: Identifier::curie("sox_1514a:a")
                    .expect("valid CURIE"),
                kind: CrossReferenceKind::Implies,
                to_source: "air21_42121".to_string(),
                to_term: Identifier::curie("air21_42121:b_2_B_iii")
                    .expect("valid CURIE"),
                rationale: SourceTextRef::with_context(
                    "SOX § 1514A(a)'s \"because of [protected activity]\" causation element is substantively realized by AIR21 § 42121(b)(2)(B)(iii)'s merits contributing-factor demonstration — the cross-reference at § 1514A(b)(2)(C) routes proof of causation through the contributing-factor standard.",
                    "uslm-crossref://sox_1514a@2002+air21_42121@2010",
                ),
            },
        ];

        ProofFramework::new(
            "sox_retaliation",
            SourceTextRef::with_context(
                "The SOX whistleblower-retaliation proof framework: substantive prohibition and procedural posture from 18 U.S.C. § 1514A composed with the four-clause burden-shifting framework from 49 U.S.C. § 42121(b)(2)(B) imported by reference per § 1514A(b)(2)(A)-(C).",
                "uslm-crossref://sox_1514a@2002+air21_42121@2010",
            ),
            statutes,
            cross_references,
            authority_strengths,
        )
        .expect("sox_retaliation framework's cross-references must resolve in praxis.lock data")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn framework_constructs() {
        let fw = framework();
        assert_eq!(fw.name(), "sox_retaliation");
        assert_eq!(fw.statutes().len(), 2);
        assert_eq!(fw.cross_references().len(), 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bundles_sox_and_air21() {
        let fw = framework();
        assert!(fw.statute_by_name("sox_1514a").is_some());
        assert!(fw.statute_by_name("air21_42121").is_some());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn both_statutes_tagged_federal_statute() {
        let fw = framework();
        assert_eq!(
            fw.authority_strength("sox_1514a"),
            Some(AuthorityStrengthConcept::FederalStatute)
        );
        assert_eq!(
            fw.authority_strength("air21_42121"),
            Some(AuthorityStrengthConcept::FederalStatute)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_cross_references_from_sox_to_air21() {
        let fw = framework();
        let from_sox = fw.cross_references_from("sox_1514a").count();
        let to_air21 = fw.cross_references_to("air21_42121").count();
        assert_eq!(from_sox, 3);
        assert_eq!(to_air21, 3);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_cross_references_from_air21() {
        // SOX imports AIR21, not vice versa.
        let fw = framework();
        assert_eq!(fw.cross_references_from("air21_42121").count(), 0);
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn b2c_requires_burden_framework() {
        let fw = framework();
        let cr = fw
            .cross_references()
            .iter()
            .find(|c| c.from_term.value() == "sox_1514a:b_2_C")
            .expect("(b)(2)(C) cross-reference exists");
        assert_eq!(cr.kind, CrossReferenceKind::Requires);
        assert_eq!(cr.to_term.value(), "air21_42121:b_2_B");
        assert!(cr.rationale.text.contains("1514A(b)(2)(C)"));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn b2a_requires_investigation() {
        let fw = framework();
        let cr = fw
            .cross_references()
            .iter()
            .find(|c| c.from_term.value() == "sox_1514a:b_2_A")
            .expect("(b)(2)(A) cross-reference exists");
        assert_eq!(cr.kind, CrossReferenceKind::Requires);
        assert_eq!(cr.to_term.value(), "air21_42121:b_2");
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn causation_implies_merits_contributing_factor() {
        let fw = framework();
        let cr = fw
            .cross_references()
            .iter()
            .find(|c| c.from_term.value() == "sox_1514a:a")
            .expect("(a) cross-reference exists");
        assert_eq!(cr.kind, CrossReferenceKind::Implies);
        assert_eq!(cr.to_term.value(), "air21_42121:b_2_B_iii");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_cross_reference_resolves() {
        // Property: framework().new() already enforces this, but the
        // assertion guards against a future change to the bundled
        // statutes' term names that would silently break a
        // cross-reference.
        let fw = framework();
        for cr in fw.cross_references() {
            let from_statute = fw.statute_by_name(&cr.from_source).unwrap();
            let to_statute = fw.statute_by_name(&cr.to_source).unwrap();
            assert!(
                from_statute.term_by_id(&cr.from_term).is_some(),
                "from-term {} not in {}",
                cr.from_term.value(),
                cr.from_source
            );
            assert!(
                to_statute.term_by_id(&cr.to_term).is_some(),
                "to-term {} not in {}",
                cr.to_term.value(),
                cr.to_source
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn description_cites_cross_reference_statutes() {
        let desc = &framework().description().text;
        assert!(desc.contains("1514A"));
        assert!(desc.contains("42121"));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn framework_is_idempotent() {
        let a = framework() as *const _;
        let b = framework() as *const _;
        assert!(core::ptr::eq(a, b));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_cross_reference_carries_rationale() {
        for cr in framework().cross_references() {
            assert!(!cr.rationale.text.is_empty());
            assert_eq!(
                cr.rationale.context_uri.as_deref(),
                Some("uslm-crossref://sox_1514a@2002+air21_42121@2010")
            );
        }
    }
}
