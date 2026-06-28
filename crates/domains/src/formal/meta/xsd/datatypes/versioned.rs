//! The XSD built-in datatype hierarchy as a versioned artifact — the
//! datatype ontology consuming the general version adjunction
//! ([`crate::formal::meta::versioning`]), companion to the XSD
//! meta-model wiring in [`super::super::versioned`].
//!
//! XSD 1.1 Part 2 adds four built-in datatypes to the 1.0 inventory:
//! `xs:anyAtomicType` (§3.2.2 — the new atomic root; in 1.0 the
//! primitives derived directly from `xs:anySimpleType`),
//! `xs:yearMonthDuration` (§3.4.26), `xs:dayTimeDuration` (§3.4.27),
//! and `xs:dateTimeStamp` (§3.4.28). The 1.0→1.1 datatype evolution is
//! additions-only (Roddick 1995), so it is a [`VersionedArtifact`]:
//! the invariant is the XSD datatype hierarchy, each fiber is the
//! datatype set available in one version.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, 2012 (the "New in
//!   XSD 1.1" markers; §3.2.2, §3.4.26-§3.4.28).
//! - **Roddick (1995)** schema versioning; **Noy & Klein (2004)**
//!   ontology evolution — additions-only evolution.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use super::super::versioned::XsdVersion;
use super::ontology::XsdDatatypeConcept;
use crate::formal::meta::versioning::ontology::{VersionFiber, VersionedArtifact};

/// The four built-in datatypes **new in XSD 1.1** (W3C XSD 1.1 Part 2
/// §3.2.2, §3.4.26, §3.4.27, §3.4.28).
pub fn datatype_1_1_additions() -> [XsdDatatypeConcept; 4] {
    [
        XsdDatatypeConcept::AnyAtomicType,
        XsdDatatypeConcept::YearMonthDuration,
        XsdDatatypeConcept::DayTimeDuration,
        XsdDatatypeConcept::DateTimeStamp,
    ]
}

/// The XSD version that *introduced* a datatype: 1.1 for the four
/// additions, 1.0 for every other built-in datatype.
pub fn introduced_in(c: XsdDatatypeConcept) -> XsdVersion {
    if datatype_1_1_additions().contains(&c) {
        XsdVersion::V1_1
    } else {
        XsdVersion::V1_0
    }
}

/// The built-in datatypes available in a given XSD version — every
/// datatype introduced in that version or earlier.
pub fn datatypes_in_version(v: XsdVersion) -> Vec<XsdDatatypeConcept> {
    XsdDatatypeConcept::variants()
        .into_iter()
        .filter(|c| introduced_in(*c) <= v)
        .collect()
}

/// The XSD datatype hierarchy as a [`VersionedArtifact`]: the invariant
/// is the datatype hierarchy, each fiber is the datatype set available
/// in one version.
pub fn xsd_datatypes_as_versioned_artifact() -> VersionedArtifact<Vec<XsdDatatypeConcept>> {
    VersionedArtifact {
        invariant: "XSD built-in datatype hierarchy (Part 2)".to_string(),
        fibers: vec![
            VersionFiber {
                version: XsdVersion::V1_0.as_str().to_string(),
                realization: datatypes_in_version(XsdVersion::V1_0),
            },
            VersionFiber {
                version: XsdVersion::V1_1.as_str().to_string(),
                realization: datatypes_in_version(XsdVersion::V1_1),
            },
        ],
    }
}

// =============================================================================
// Axioms.
// =============================================================================

/// Axiom: the datatype inventory evolves monotonically — every 1.0
/// datatype survives into 1.1, which adds exactly the four 1.1
/// datatypes (the famous XSD 1.0 "44 built-in datatypes": 2 special +
/// 19 primitive + 25 derived = 46 concepts here, vs 50 in 1.1).
pub struct DatatypeEvolutionIsMonotone;

impl Axiom for DatatypeEvolutionIsMonotone {
    fn verify(&self) -> Verdict {
        let v10 = datatypes_in_version(XsdVersion::V1_0);
        let v11 = datatypes_in_version(XsdVersion::V1_1);
        // The number of 1.1 additions is itself loaded data, not a literal 4
        // (audit 2026-06-12 D-17).
        let additions = datatype_1_1_additions().len();
        let monotone = v10.iter().all(|c| v11.contains(c));
        let strict = v11.len() == v10.len() + additions;
        let complete = v11.len() == XsdDatatypeConcept::variants().len();
        // The 1.0 fiber is the full inventory minus the 1.1 additions — DERIVED
        // from the loaded sets, not the magic literal 46 (audit D-17).
        let baseline = v10.len() == XsdDatatypeConcept::variants().len() - additions;
        if monotone && strict && complete && baseline {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DatatypeEvolutionIsMonotone",
        "the XSD datatype inventory grows additions-only: the 1.0 fiber (46 concepts) nests in the 1.1 fiber (50), which adds exactly anyAtomicType + yearMonthDuration + dayTimeDuration + dateTimeStamp",
        "W3C XSD 1.1 Part 2 §3.2.2, §3.4.26-§3.4.28 (2012); Roddick (1995); Noy & Klein (2004)"
    );
}

pr4xis::register_axiom!(
    DatatypeEvolutionIsMonotone,
    "W3C XSD 1.1 Part 2 §3.2.2, §3.4.26-§3.4.28 (2012); Roddick (1995); Noy & Klein (2004)"
);

/// Axiom: the XSD datatype hierarchy instantiates the version
/// adjunction — invariant constant across versions, `localize`
/// recovers each fiber, and the 1.0 fiber nests in the 1.1 fiber.
pub struct DatatypesConsumeVersionAdjunction;

impl Axiom for DatatypesConsumeVersionAdjunction {
    fn verify(&self) -> Verdict {
        let art = xsd_datatypes_as_versioned_artifact();
        let invariant_ok = art.abstract_version() == "XSD built-in datatype hierarchy (Part 2)";
        let (Some(f10), Some(f11)) = (art.localize("1.0"), art.localize("1.1")) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let nested = f10.realization.iter().all(|c| f11.realization.contains(c));
        if invariant_ok && nested && art.localize("0.9").is_none() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DatatypesConsumeVersionAdjunction",
        "the XSD datatype hierarchy instantiates the version adjunction: invariant constant across versions, localize recovers each fiber, 1.0 fiber nests in 1.1",
        "Mac Lane (1998) §IV.1; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
    );
}

pr4xis::register_axiom!(
    DatatypesConsumeVersionAdjunction,
    "Mac Lane (1998) §IV.1; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_additions_are_1_1() {
        for c in datatype_1_1_additions() {
            assert_eq!(introduced_in(c), XsdVersion::V1_1, "{c:?}");
        }
        assert_eq!(datatype_1_1_additions().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fibers_nest_and_baseline_is_46() {
        let v10 = datatypes_in_version(XsdVersion::V1_0);
        let v11 = datatypes_in_version(XsdVersion::V1_1);
        assert!(v10.iter().all(|c| v11.contains(c)));
        assert_eq!(v10.len(), 46);
        assert_eq!(v11.len(), 50);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn datatypes_are_a_versioned_artifact() {
        let art = xsd_datatypes_as_versioned_artifact();
        assert_eq!(
            art.abstract_version(),
            "XSD built-in datatype hierarchy (Part 2)"
        );
        assert_eq!(art.localize("1.0").map(|f| f.realization.len()), Some(46));
        assert_eq!(art.localize("1.1").map(|f| f.realization.len()), Some(50));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_monotone() {
        assert!(DatatypeEvolutionIsMonotone.verify().is_ok());
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn axiom_consumes_adjunction() {
        assert!(DatatypesConsumeVersionAdjunction.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_membership_matches_introduction(
            c in proptest::sample::select(XsdDatatypeConcept::variants())
        ) {
            let v10 = datatypes_in_version(XsdVersion::V1_0);
            let v11 = datatypes_in_version(XsdVersion::V1_1);
            prop_assert!(v11.contains(&c));
            prop_assert_eq!(v10.contains(&c), introduced_in(c) == XsdVersion::V1_0);
        }
    }

    pr4xis::register_praxis_value!(prop_membership_matches_introduction, Verifiable);
}
