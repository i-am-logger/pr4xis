//! XSD as a versioned artifact — the XSD meta-model consuming the
//! general version adjunction ([`crate::formal::meta::versioning`]).
//!
//! XSD is published in two versions, 1.0 (W3C Recommendation 2004) and
//! 1.1 (2012). XSD 1.1 is a strict superset of the 1.0 schema-component
//! inventory: every 1.0 construct survives, and 1.1 *adds* six —
//! `<xs:override>` (§4.2.5), `<xs:explicitTimezone>` (§4.3.14),
//! `<xs:assertion>` (§4.3.13), `<xs:assert>` (§3.13), `<xs:openContent>`
//! (§3.4.2.2), and `<xs:defaultOpenContent>` (§3.16.2). That
//! monotone, additions-only evolution (Roddick 1995; Noy & Klein 2004)
//! is exactly a [`VersionedArtifact`]: the **invariant** is XSD itself,
//! each **fiber** is the construct set available in one version, and
//! `LocalizeVersion ⊣ AbstractVersion` moves between them.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 1: Structures** / **Part 2: Datatypes**,
//!   2012 (the "New in XSD 1.1" markers on each construct).
//! - **Roddick (1995)** schema versioning; **Noy & Klein (2004)**
//!   ontology evolution — additions-only schema evolution.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::ontology::XsdConcept;
use crate::formal::meta::versioning::ontology::{VersionFiber, VersionedArtifact};

/// A published version of XSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum XsdVersion {
    /// XSD 1.0 — W3C Recommendation, 2004-10-28.
    V1_0,
    /// XSD 1.1 — W3C Recommendation, 2012-04-05.
    V1_1,
}

impl XsdVersion {
    /// The version label used as a [`VersionFiber::version`].
    pub fn as_str(self) -> &'static str {
        match self {
            XsdVersion::V1_0 => "1.0",
            XsdVersion::V1_1 => "1.1",
        }
    }
}

/// The six schema-component constructs **new in XSD 1.1** (W3C XSD 1.1
/// Part 1 §4.2.5, §3.13, §3.4.2.2, §3.16.2; Part 2 §4.3.13, §4.3.14).
pub fn xsd_1_1_additions() -> [XsdConcept; 6] {
    [
        XsdConcept::SchemaOverride,
        XsdConcept::ExplicitTimezoneFacet,
        XsdConcept::AssertionFacet,
        XsdConcept::Assert,
        XsdConcept::OpenContent,
        XsdConcept::DefaultOpenContent,
    ]
}

/// The XSD version that *introduced* a concept: 1.1 for the six
/// additions, 1.0 for every other schema component (the 1.0 inventory
/// that 1.1 inherits unchanged).
pub fn introduced_in(c: XsdConcept) -> XsdVersion {
    if xsd_1_1_additions().contains(&c) {
        XsdVersion::V1_1
    } else {
        XsdVersion::V1_0
    }
}

/// The schema components available in a given XSD version — every
/// concept introduced in that version or earlier.
pub fn concepts_in_version(v: XsdVersion) -> Vec<XsdConcept> {
    XsdConcept::variants()
        .into_iter()
        .filter(|c| introduced_in(*c) <= v)
        .collect()
}

/// XSD as a [`VersionedArtifact`]: the invariant is XSD, each fiber is
/// the construct set available in one version. Consuming the general
/// version adjunction makes XSD's 1.0→1.1 evolution a first-class
/// instance of `LocalizeVersion ⊣ AbstractVersion`.
pub fn xsd_as_versioned_artifact() -> VersionedArtifact<Vec<XsdConcept>> {
    VersionedArtifact {
        invariant: "XML Schema Definition Language (XSD)".to_string(),
        fibers: vec![
            VersionFiber {
                version: XsdVersion::V1_0.as_str().to_string(),
                realization: concepts_in_version(XsdVersion::V1_0),
            },
            VersionFiber {
                version: XsdVersion::V1_1.as_str().to_string(),
                realization: concepts_in_version(XsdVersion::V1_1),
            },
        ],
    }
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: XSD's version evolution is monotone — every construct
/// available in 1.0 is still available in 1.1 (additions only, no
/// removals). The 1.1 fiber strictly contains the 1.0 fiber.
pub struct XsdVersionEvolutionIsMonotone;

impl Axiom for XsdVersionEvolutionIsMonotone {
    fn verify(&self) -> Verdict {
        let v10 = concepts_in_version(XsdVersion::V1_0);
        let v11 = concepts_in_version(XsdVersion::V1_1);
        // 1.0 ⊆ 1.1, and 1.1 strictly larger (the six additions).
        let monotone = v10.iter().all(|c| v11.contains(c));
        let strict = v11.len() == v10.len() + 6;
        // 1.1 covers the whole inventory.
        let complete = v11.len() == XsdConcept::variants().len();
        if monotone && strict && complete {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdVersionEvolutionIsMonotone",
        "XSD 1.1 is a strict superset of the 1.0 schema-component inventory — additions only, no removals; the 1.1 fiber contains the 1.0 fiber plus exactly the six 1.1 additions",
        "W3C XSD 1.1 Part 1/Part 2 (2012); Roddick (1995) schema versioning; Noy & Klein (2004) ontology evolution"
    );
}

pr4xis::register_axiom!(
    XsdVersionEvolutionIsMonotone,
    "W3C XSD 1.1 Part 1/Part 2 (2012); Roddick (1995); Noy & Klein (2004)"
);

/// Axiom: the new-in-1.1 set is exactly the difference between the 1.1
/// and 1.0 fibers — the six §-marked additions and no others.
pub struct XsdAdditionsAreFiberDifference;

impl Axiom for XsdAdditionsAreFiberDifference {
    fn verify(&self) -> Verdict {
        let v10 = concepts_in_version(XsdVersion::V1_0);
        let additions = xsd_1_1_additions();
        // Each addition is absent from 1.0 and introduced in 1.1.
        let added_not_in_10 = additions
            .iter()
            .all(|c| !v10.contains(c) && introduced_in(*c) == XsdVersion::V1_1);
        // Every concept NOT among the additions is a 1.0 concept.
        let rest_are_10 = XsdConcept::variants()
            .into_iter()
            .filter(|c| !additions.contains(c))
            .all(|c| introduced_in(c) == XsdVersion::V1_0);
        if added_not_in_10 && rest_are_10 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdAdditionsAreFiberDifference",
        "the XSD 1.1 additions (override/explicitTimezone/assertion/assert/openContent/defaultOpenContent) are exactly the 1.1-fiber-minus-1.0-fiber difference; every other concept belongs to the 1.0 inventory",
        "W3C XSD 1.1 Part 1 §4.2.5, §3.13, §3.4.2.2, §3.16.2; Part 2 §4.3.13, §4.3.14 (2012)"
    );
}

pr4xis::register_axiom!(
    XsdAdditionsAreFiberDifference,
    "W3C XSD 1.1 Part 1 §4.2.5, §3.13, §3.4.2.2, §3.16.2; Part 2 §4.3.13, §4.3.14"
);

/// Axiom: XSD instantiates the version adjunction — the
/// [`xsd_as_versioned_artifact`]'s invariant is constant across
/// versions, `localize` recovers each version's fiber, and the fibers
/// nest 1.0 ⊆ 1.1 (the localize/abstract round-trip).
pub struct XsdConsumesVersionAdjunction;

impl Axiom for XsdConsumesVersionAdjunction {
    fn verify(&self) -> Verdict {
        let art = xsd_as_versioned_artifact();
        let invariant_ok = art.abstract_version() == "XML Schema Definition Language (XSD)";
        let f10 = art.localize("1.0");
        let f11 = art.localize("1.1");
        let (Some(f10), Some(f11)) = (f10, f11) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        // The version adjunction's fibers nest by version order.
        let nested = f10.realization.iter().all(|c| f11.realization.contains(c));
        if invariant_ok && nested && art.localize("9.9").is_none() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "XsdConsumesVersionAdjunction",
        "XSD instantiates the version adjunction: the artifact's invariant is XSD across versions, localize recovers each fiber, and the 1.0 fiber nests in the 1.1 fiber",
        "Mac Lane (1998) §IV.1; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
    );
}

pr4xis::register_axiom!(
    XsdConsumesVersionAdjunction,
    "Mac Lane (1998) §IV.1; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn six_additions_are_1_1() {
        for c in xsd_1_1_additions() {
            assert_eq!(introduced_in(c), XsdVersion::V1_1, "{c:?}");
        }
        assert_eq!(xsd_1_1_additions().len(), 6);
    }

    #[test]
    fn other_concepts_are_1_0() {
        let adds = xsd_1_1_additions();
        for c in XsdConcept::variants() {
            if !adds.contains(&c) {
                assert_eq!(introduced_in(c), XsdVersion::V1_0, "{c:?}");
            }
        }
    }

    #[test]
    fn fibers_nest() {
        let v10 = concepts_in_version(XsdVersion::V1_0);
        let v11 = concepts_in_version(XsdVersion::V1_1);
        assert!(v10.iter().all(|c| v11.contains(c)));
        assert_eq!(v11.len(), v10.len() + 6);
        assert_eq!(v11.len(), XsdConcept::variants().len());
    }

    #[test]
    fn xsd_is_a_versioned_artifact() {
        let art = xsd_as_versioned_artifact();
        assert_eq!(
            art.abstract_version(),
            "XML Schema Definition Language (XSD)"
        );
        assert!(art.localize("1.0").is_some());
        assert!(art.localize("1.1").is_some());
        assert!(art.localize("2.0").is_none());
    }

    #[test]
    fn axiom_monotone() {
        assert!(XsdVersionEvolutionIsMonotone.verify().is_ok());
    }

    #[test]
    fn axiom_additions_difference() {
        assert!(XsdAdditionsAreFiberDifference.verify().is_ok());
    }

    #[test]
    fn axiom_consumes_adjunction() {
        assert!(XsdConsumesVersionAdjunction.verify().is_ok());
    }

    proptest! {
        /// Every concept is in the 1.1 fiber, and in the 1.0 fiber iff
        /// it was introduced in 1.0.
        #[test]
        fn prop_membership_matches_introduction(
            c in proptest::sample::select(XsdConcept::variants())
        ) {
            let v10 = concepts_in_version(XsdVersion::V1_0);
            let v11 = concepts_in_version(XsdVersion::V1_1);
            prop_assert!(v11.contains(&c));
            prop_assert_eq!(v10.contains(&c), introduced_in(c) == XsdVersion::V1_0);
        }
    }
}
