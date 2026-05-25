//! Tests for the version adjunction ontology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    AbstractVersion, AdjunctionAppliesAcrossDomains, AdjunctionUnitReflectsInvariant,
    InvariantIsConstantComplement, LocalizeRecoversEachFiber, LocalizeVersion, VersionAdjunction,
    VersionDependence, VersionDependenceOf, VersionFiber, VersionFiberSpecialisesInvariant,
    VersionedArtifact, VersioningCategory, VersioningConcept, VersioningOntology,
    abstract_version_concept, localize_version_concept, sample_artifacts,
};
use pr4xis::category::Concept;
use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category / functor / ontology validation.
// =============================================================================

#[test]
fn versioning_category_laws() {
    assert_category_laws::<VersioningCategory>();
}

#[test]
fn localize_is_a_functor() {
    assert_functor_laws::<LocalizeVersion>();
}

#[test]
fn abstract_is_a_functor() {
    assert_functor_laws::<AbstractVersion>();
}

#[test]
fn versioning_ontology_validates() {
    VersioningOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn concept_count() {
    assert_eq!(VersioningConcept::variants().len(), 4);
}

// =============================================================================
// The adjoint endofunctor object maps.
// =============================================================================

#[test]
fn abstract_collapses_fiber_to_invariant() {
    use VersioningConcept as C;
    assert_eq!(
        abstract_version_concept(C::VersionFiber),
        C::VersionInvariant
    );
    assert_eq!(
        abstract_version_concept(C::VersionInvariant),
        C::VersionInvariant
    );
    assert_eq!(abstract_version_concept(C::Version), C::Version);
}

#[test]
fn localize_realizes_invariant_as_fiber() {
    use VersioningConcept as C;
    assert_eq!(
        localize_version_concept(C::VersionInvariant),
        C::VersionFiber
    );
    assert_eq!(localize_version_concept(C::VersionFiber), C::VersionFiber);
}

#[test]
fn reflection_round_trip_on_invariant() {
    use VersioningConcept as C;
    // AbstractVersion ∘ LocalizeVersion = id on the invariant.
    assert_eq!(
        abstract_version_concept(localize_version_concept(C::VersionInvariant)),
        C::VersionInvariant
    );
}

// =============================================================================
// Quality.
// =============================================================================

#[test]
fn version_dependence() {
    use VersioningConcept as C;
    assert_eq!(
        VersionDependenceOf.get(&C::VersionInvariant),
        Some(VersionDependence::Independent)
    );
    assert_eq!(
        VersionDependenceOf.get(&C::VersionFiber),
        Some(VersionDependence::Dependent)
    );
    assert_eq!(
        VersionDependenceOf.get(&C::Version),
        Some(VersionDependence::Dependent)
    );
    assert_eq!(VersionDependenceOf.get(&C::VersionedArtifact), None);
}

// =============================================================================
// Runtime instance level — generic over the realization type.
// =============================================================================

#[test]
fn instance_localize_and_abstract() {
    let xsd = &sample_artifacts()[0];
    assert_eq!(
        xsd.abstract_version(),
        "XML Schema Definition Language (XSD)"
    );
    assert_eq!(
        xsd.localize("1.1").map(|f| f.realization),
        Some("W3C Recommendation 2012-04-05")
    );
    assert!(xsd.localize("3.0").is_none());
}

#[test]
fn applies_to_xsd_xml_pdf_uslm_citation() {
    let arts = sample_artifacts();
    let invariants: Vec<&str> = arts.iter().map(|a| a.abstract_version()).collect();
    assert!(invariants.iter().any(|i| i.contains("XSD")));
    assert!(invariants.iter().any(|i| i.contains("XML")));
    assert!(invariants.iter().any(|i| i.contains("PDF")));
    assert!(invariants.iter().any(|i| i.contains("USLM")));
    // Each artifact has ≥2 versions sharing one invariant.
    for a in &arts {
        assert!(a.fibers.len() >= 2);
    }
}

#[test]
fn generic_over_realization_type() {
    // The realization need not be a string — here a small struct.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Feature {
        assertions: bool,
    }
    let art = VersionedArtifact {
        invariant: "complex-type assertions".to_string(),
        fibers: vec![
            VersionFiber {
                version: "1.0".to_string(),
                realization: Feature { assertions: false },
            },
            VersionFiber {
                version: "1.1".to_string(),
                realization: Feature { assertions: true },
            },
        ],
    };
    assert_eq!(art.localize("1.1"), Some(&art.fibers[1]));
    assert_eq!(art.abstract_version(), "complex-type assertions");
}

// =============================================================================
// Axiom tests.
// =============================================================================

#[test]
fn axiom_unit_reflects_invariant() {
    assert!(AdjunctionUnitReflectsInvariant.verify().is_ok());
}

#[test]
fn axiom_fiber_specialises_invariant() {
    assert!(VersionFiberSpecialisesInvariant.verify().is_ok());
}

#[test]
fn axiom_constant_complement() {
    assert!(InvariantIsConstantComplement.verify().is_ok());
}

#[test]
fn axiom_localize_recovers_each_fiber() {
    assert!(LocalizeRecoversEachFiber.verify().is_ok());
}

#[test]
fn axiom_applies_across_domains() {
    assert!(AdjunctionAppliesAcrossDomains.verify().is_ok());
}

#[test]
fn adjunction_meta_is_cited() {
    use pr4xis::category::Adjunction;
    let meta = VersionAdjunction::meta();
    assert!(!meta.citation.as_str().is_empty());
    assert!(meta.citation.as_str().contains("Mac Lane"));
}

// =============================================================================
// Property-based tests over generic versioned artifacts.
// =============================================================================

prop_compose! {
    /// A versioned artifact with a non-empty invariant and 1..6 fibers
    /// at distinct versions.
    fn arb_artifact()(
        invariant in "[a-z ]{1,20}",
        versions in prop::collection::hash_set("[0-9]{1,3}\\.[0-9]{1,3}", 1..6),
    ) -> VersionedArtifact<u32> {
        let fibers = versions
            .into_iter()
            .enumerate()
            .map(|(i, v)| VersionFiber { version: v, realization: i as u32 })
            .collect();
        VersionedArtifact { invariant, fibers }
    }
}

proptest! {
    /// The invariant is the constant complement: abstract_version is
    /// independent of which fiber is localized (Bancilhon & Spyratos
    /// 1981).
    #[test]
    fn prop_invariant_is_constant(art in arb_artifact()) {
        let inv = art.invariant.clone();
        for f in &art.fibers {
            let _ = art.localize(&f.version);
            prop_assert_eq!(art.abstract_version(), inv.as_str());
        }
    }

    /// localize recovers every registered fiber and only those
    /// (the counit is the identity on localized fibers).
    #[test]
    fn prop_localize_recovers_fibers(art in arb_artifact()) {
        for f in &art.fibers {
            prop_assert_eq!(art.localize(&f.version), Some(f));
        }
        prop_assert!(art.localize("zzz-absent").is_none());
    }

    /// The concept-level reflection round-trip holds for every concept
    /// the localize map fixes, and collapses fibers as specified.
    #[test]
    fn prop_concept_round_trip(c in proptest::sample::select(VersioningConcept::variants())) {
        use VersioningConcept as C;
        let rt = abstract_version_concept(localize_version_concept(c));
        // The round-trip is the identity except on VersionFiber, which
        // localize fixes and abstract collapses to VersionInvariant.
        let expected = if c == C::VersionFiber { C::VersionInvariant } else { c };
        prop_assert_eq!(rt, expected);
    }
}
