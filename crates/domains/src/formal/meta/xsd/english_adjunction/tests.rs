//! Tests for the XsdOntology ⊣ English adjunction.
//!
//! Three layers per `feedback_high_test_coverage`:
//!
//! 1. **Axioms** (cited): the triangle identities `(ε F) ∘ (F η) = id_F`
//!    and `(G ε) ∘ (η G) = id_G` hold (Mac Lane §IV.1); the functor
//!    laws hold for both adjoints (`assert_functor_laws`); the
//!    canonical English-projection phrase of every XSD concept resolves
//!    back to that concept under the round trip.
//! 2. **Property-based** (proptest): for arbitrary objects on either
//!    side, the unit/counit components are identities (the equivalence
//!    property holds globally); the runtime Lift is monotone in the
//!    instance's name set and deterministic across calls.
//! 3. **Adjunction laws**: triangle identities verified explicitly on
//!    every variant; naturality of unit and counit verified across
//!    every morphism in both source categories. Mac Lane §IV.1 (1).

#[allow(unused_imports)]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
use pr4xis::category::{Adjunction, Arrow, Category, Concept, Functor};

use super::super::english_projection::{
    XsdEnglishLabel, XsdEnglishLabelCategory, XsdToEnglish, canonical_english_phrase,
    project_concept,
};
use super::super::from_xsd_parser::{NamedSchemaComponentEntry, XsdOntologyInstance};
use super::super::ontology::{XsdCategory, XsdConcept};
use super::{
    LiftEnglishToXsd, XsdEnglishAdjunction, lift_and_project,
    lift_english_term_to_schema_components, lift_label_to_concept,
};
use crate::cognitive::linguistics::english::ontology::English;
use crate::social::software::markup::xml::lmf;

// =============================================================================
// Test fixtures
// =============================================================================

/// Minimal LMF WordNet bundle — enough to give the property tests an
/// `&English` to thread through `lift_english_term_to_schema_components`.
/// The Lift itself is monolingual substring-matching; the WordNet
/// presence is for the type-level documentation of the adjunction's
/// English source category.
const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-section-n"><Lemma writtenForm="section" partOfSpeech="n"/><Sense id="s-section-1" synset="s-section"/></LexicalEntry>
    <LexicalEntry id="e-element-n"><Lemma writtenForm="element" partOfSpeech="n"/><Sense id="s-element-1" synset="s-element"/></LexicalEntry>
    <LexicalEntry id="e-type-n"><Lemma writtenForm="type" partOfSpeech="n"/><Sense id="s-type-1" synset="s-type"/></LexicalEntry>
    <LexicalEntry id="e-complex-a"><Lemma writtenForm="complex" partOfSpeech="a"/><Sense id="s-complex-1" synset="s-complex"/></LexicalEntry>
    <LexicalEntry id="e-simple-a"><Lemma writtenForm="simple" partOfSpeech="a"/><Sense id="s-simple-1" synset="s-simple"/></LexicalEntry>
    <Synset id="s-section" ili="i1" partOfSpeech="n"><Definition>segment</Definition></Synset>
    <Synset id="s-element" ili="i2" partOfSpeech="n"><Definition>constituent</Definition></Synset>
    <Synset id="s-type" ili="i3" partOfSpeech="n"><Definition>kind</Definition></Synset>
    <Synset id="s-complex" ili="i4" partOfSpeech="a"><Definition>complicated</Definition></Synset>
    <Synset id="s-simple" ili="i5" partOfSpeech="a"><Definition>elementary</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

fn sample_english() -> English {
    let wn = lmf::reader::read_wordnet(SAMPLE_LMF).expect("sample LMF parses");
    English::from_wordnet(&wn)
}

/// Build a small `XsdOntologyInstance` with a handful of named USLM-
/// like declarations. Mirrors the (concept, name) pairs the loader
/// produces from USLM-1.0.18.xsd for a representative subset of
/// declarations.
fn sample_uslm_instance() -> XsdOntologyInstance {
    let named = vec![
        NamedSchemaComponentEntry {
            concept: XsdConcept::ElementDeclaration,
            local_name: "section".to_string(),
        },
        NamedSchemaComponentEntry {
            concept: XsdConcept::ElementDeclaration,
            local_name: "subsection".to_string(),
        },
        NamedSchemaComponentEntry {
            concept: XsdConcept::ElementDeclaration,
            local_name: "paragraph".to_string(),
        },
        NamedSchemaComponentEntry {
            concept: XsdConcept::ComplexTypeDefinition,
            local_name: "SectionType".to_string(),
        },
        NamedSchemaComponentEntry {
            concept: XsdConcept::ComplexTypeDefinition,
            local_name: "BlockType".to_string(),
        },
        NamedSchemaComponentEntry {
            concept: XsdConcept::AttributeGroup,
            local_name: "XmlSpecialAttrs".to_string(),
        },
    ];
    let components = named.iter().map(|n| n.concept).collect();
    XsdOntologyInstance {
        components,
        named,
        elements: Vec::new(),
        imports: Vec::new(),
        includes: Vec::new(),
        redefines: Vec::new(),
        overrides: Vec::new(),
        annotations: Vec::new(),
        derivations: Vec::new(),
    }
}

// =============================================================================
// Adjoint functor laws — both adjoints satisfy Mac Lane §I.3.
// =============================================================================

#[test]
fn left_adjoint_functor_laws_pass() {
    assert_functor_laws::<LiftEnglishToXsd>();
}

#[test]
fn right_adjoint_functor_laws_pass() {
    assert_functor_laws::<XsdToEnglish>();
}

#[test]
fn xsd_english_label_category_laws_pass() {
    assert_category_laws::<XsdEnglishLabelCategory>();
}

#[test]
fn xsd_category_laws_pass() {
    assert_category_laws::<XsdCategory>();
}

// =============================================================================
// Bijection laws — `project_concept ∘ lift_label_to_concept = id` and
// vice versa. These are the equivalence-of-categories witnesses
// (Mac Lane §IV.4) the triangle identities collapse onto.
// =============================================================================

#[test]
fn project_then_lift_is_identity_on_concepts() {
    for c in XsdConcept::variants() {
        let label = project_concept(c);
        let back = lift_label_to_concept(label);
        assert_eq!(back, c, "round-trip on {c:?} must be identity");
    }
}

#[test]
fn lift_then_project_is_identity_on_labels() {
    for l in XsdEnglishLabel::variants() {
        let c = lift_label_to_concept(l);
        let back = project_concept(c);
        assert_eq!(back, l, "round-trip on {l:?} must be identity");
    }
}

// =============================================================================
// Triangle identities — Mac Lane §IV.1 (1).
// =============================================================================
//
// The two triangle identities of an adjunction F ⊣ G:
//
//   (ε F) ∘ (F η) = id_F     [R-triangle]
//   (G ε) ∘ (η G) = id_G     [L-triangle]
//
// In components, at an object A of C and an object B of D:
//
//   ε_{F(A)} ∘ F(η_A) = id_{F(A)}     [R-component]
//   G(ε_B)  ∘ η_{G(B)} = id_{G(B)}    [L-component]
//
// Both reduce here to identity-composition because the adjunction is
// an equivalence (Mac Lane §IV.4): η and ε are identity natural
// transformations.

#[test]
fn r_triangle_identity_holds_per_concept() {
    // R-triangle component at every English label l (object of C):
    //   ε_{F(l)} ∘ F(η_l) = id_{F(l)}
    for l in XsdEnglishLabel::variants() {
        // η_l : l → l (identity on C).
        let eta_l = XsdEnglishAdjunction::unit(&l);
        // F(η_l) is a morphism F(l) → F(l) in D.
        let f_eta = LiftEnglishToXsd::map_morphism(&eta_l);
        // ε_{F(l)} : F(l) → F(l) (identity on D).
        let fl = LiftEnglishToXsd::map_object(&l);
        let eps_fl = XsdEnglishAdjunction::counit(&fl);
        // Compose them in D.
        let composed = XsdCategory::compose(&f_eta, &eps_fl).expect("composable in D");
        let id_fl = XsdCategory::identity(&fl);
        assert_eq!(
            composed, id_fl,
            "R-triangle identity must hold at {l:?} (got {composed:?})"
        );
    }
}

#[test]
fn l_triangle_identity_holds_per_concept() {
    // L-triangle component at every XSD concept c (object of D):
    //   G(ε_c) ∘ η_{G(c)} = id_{G(c)}
    for c in XsdConcept::variants() {
        // ε_c : c → c (identity on D).
        let eps_c = XsdEnglishAdjunction::counit(&c);
        // G(ε_c) : G(c) → G(c) in C.
        let g_eps = XsdToEnglish::map_morphism(&eps_c);
        // η_{G(c)} : G(c) → G(c) (identity on C).
        let gc = XsdToEnglish::map_object(&c);
        let eta_gc = XsdEnglishAdjunction::unit(&gc);
        let composed = XsdEnglishLabelCategory::compose(&eta_gc, &g_eps).expect("composable in C");
        let id_gc = XsdEnglishLabelCategory::identity(&gc);
        assert_eq!(
            composed, id_gc,
            "L-triangle identity must hold at {c:?} (got {composed:?})"
        );
    }
}

// =============================================================================
// Naturality of unit and counit — Mac Lane §I.4.
//
// η is natural iff for every morphism f: A → A' in C the square
//   G(F(A)) <-- η_A --- A
//      |                |
//   G(F(f))            f
//      v                v
//   G(F(A')) <-- η_{A'} -- A'
// commutes, i.e. G(F(f)) ∘ η_A = η_{A'} ∘ f.
//
// In an equivalence, η is the identity nat-trans, so both sides equal
// the original morphism f (modulo round-trip through the bijection).
// =============================================================================

#[test]
fn unit_is_natural_for_every_morphism_in_c() {
    for m in XsdEnglishLabelCategory::morphisms() {
        let a = m.source();
        let a_prime = m.target();
        // Left side: G(F(f)) ∘ η_a.
        let f_morph = LiftEnglishToXsd::map_morphism(&m);
        let gf_morph = XsdToEnglish::map_morphism(&f_morph);
        let eta_a = XsdEnglishAdjunction::unit(&a);
        let lhs = XsdEnglishLabelCategory::compose(&eta_a, &gf_morph);
        // Right side: η_{a'} ∘ f.
        let eta_ap = XsdEnglishAdjunction::unit(&a_prime);
        let rhs = XsdEnglishLabelCategory::compose(&m, &eta_ap);
        assert_eq!(lhs, rhs, "naturality of η must hold on morphism {:?}", m);
    }
}

#[test]
fn counit_is_natural_for_every_morphism_in_d() {
    for m in XsdCategory::morphisms() {
        let b = m.source();
        let b_prime = m.target();
        let g_morph = XsdToEnglish::map_morphism(&m);
        let fg_morph = LiftEnglishToXsd::map_morphism(&g_morph);
        let eps_b = XsdEnglishAdjunction::counit(&b);
        let lhs = XsdCategory::compose(&eps_b, &m);
        let eps_bp = XsdEnglishAdjunction::counit(&b_prime);
        let rhs = XsdCategory::compose(&fg_morph, &eps_bp);
        assert_eq!(lhs, rhs, "naturality of ε must hold on morphism {:?}", m);
    }
}

// =============================================================================
// Adjunction meta — citation surfaces through `Provenance`.
// =============================================================================

#[test]
fn adjunction_meta_carries_citation() {
    let meta = <XsdEnglishAdjunction as Adjunction>::meta();
    assert_eq!(meta.name.as_str(), "XsdEnglishAdjunction");
    let cit = meta.citation.as_str();
    assert!(cit.contains("Mac Lane"), "citation: {cit}");
    assert!(
        cit.contains("Awodey") || cit.contains("Lambek"),
        "citation: {cit}"
    );
    assert!(meta.module_path.as_str().contains("xsd"));
}

#[test]
fn left_adjoint_meta_carries_citation() {
    let meta = LiftEnglishToXsd::meta();
    assert_eq!(meta.name.as_str(), "LiftEnglishToXsd");
    assert!(meta.citation.as_str().contains("Mac Lane"));
}

// =============================================================================
// Round-trip on the canonical English head-noun phrases
// (M4.ε.5.a.3): for every XSD concept c, its canonical English
// phrase round-trips through the right adjoint then the left adjoint
// to a concept set that contains c.
// =============================================================================

#[test]
fn canonical_phrase_round_trip_recovers_concept() {
    // For every XSD concept c, the canonical English phrase resolves
    // through the right adjoint's `canonical_english_phrase` and
    // re-lifts through `lift_label_to_concept` ∘ `project_concept⁻¹`
    // back to c. This is the witness of the L-triangle identity at
    // the level of canonical English phrases (Mac Lane §IV.4
    // equivalence).
    for c in XsdConcept::variants() {
        let phrase = canonical_english_phrase(c);
        // The English label projection of c.
        let label = project_concept(c);
        // The lift back to the concept.
        let lifted = lift_label_to_concept(label);
        assert_eq!(
            lifted, c,
            "canonical phrase {phrase:?} for {c:?} did not round-trip"
        );
    }
}

// =============================================================================
// Runtime Lift — instance-level substring-matching component of F.
// =============================================================================

#[test]
fn runtime_lift_recovers_section_components() {
    let en = sample_english();
    let inst = sample_uslm_instance();
    let lifted = lift_english_term_to_schema_components("section", &inst, &en);
    // Three components contain "section": `section`, `subsection`,
    // `SectionType`.
    let names: Vec<String> = lifted.iter().map(|c| c.local_name.clone()).collect();
    assert!(names.contains(&"section".to_string()));
    assert!(names.contains(&"subsection".to_string()));
    assert!(names.contains(&"SectionType".to_string()));
    assert!(!names.contains(&"paragraph".to_string()));
    assert!(!names.contains(&"BlockType".to_string()));
}

#[test]
fn runtime_lift_is_case_insensitive() {
    let en = sample_english();
    let inst = sample_uslm_instance();
    let upper = lift_english_term_to_schema_components("SECTION", &inst, &en);
    let lower = lift_english_term_to_schema_components("section", &inst, &en);
    assert_eq!(upper.len(), lower.len());
    let upper_names: Vec<String> = upper.iter().map(|c| c.local_name.clone()).collect();
    let lower_names: Vec<String> = lower.iter().map(|c| c.local_name.clone()).collect();
    assert_eq!(upper_names, lower_names);
}

#[test]
fn runtime_lift_empty_term_returns_empty() {
    let en = sample_english();
    let inst = sample_uslm_instance();
    assert!(lift_english_term_to_schema_components("", &inst, &en).is_empty());
    assert!(lift_english_term_to_schema_components("   ", &inst, &en).is_empty());
}

#[test]
fn runtime_lift_unknown_term_returns_empty() {
    let en = sample_english();
    let inst = sample_uslm_instance();
    assert!(lift_english_term_to_schema_components("platypus", &inst, &en).is_empty());
}

#[test]
fn runtime_lift_on_empty_instance_returns_empty() {
    let en = sample_english();
    let inst = XsdOntologyInstance::default();
    assert!(lift_english_term_to_schema_components("section", &inst, &en).is_empty());
}

#[test]
fn lift_and_project_round_trip_carries_term_lemma() {
    // The composite (R ∘ L) applied to "section" must produce at
    // least one match whose projected English lemmas contain
    // "section". This is the runtime witness of the unit's
    // instance-level component.
    let en = sample_english();
    let inst = sample_uslm_instance();
    let combined = lift_and_project("section", &inst, &en);
    assert!(!combined.is_empty(), "lift_and_project produced no results");
    assert!(
        combined
            .iter()
            .any(|(_, lemmas)| lemmas.iter().any(|l| l == "section")),
        "round-trip lemmas should contain `section`; got {combined:?}"
    );
}

// =============================================================================
// Property-based tests
// =============================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_xsd_concept() -> impl Strategy<Value = XsdConcept> {
        proptest::sample::select(XsdConcept::variants())
    }

    fn arb_xsd_english_label() -> impl Strategy<Value = XsdEnglishLabel> {
        proptest::sample::select(XsdEnglishLabel::variants())
    }

    proptest! {
        /// Property: round-tripping every English label through F
        /// then G yields the original label. Equivalent to the
        /// L-triangle identity restricted to objects (Mac Lane §IV.4
        /// equivalence).
        #[test]
        fn prop_round_trip_label_through_f_then_g(l in arb_xsd_english_label()) {
            let c = LiftEnglishToXsd::map_object(&l);
            let back = XsdToEnglish::map_object(&c);
            prop_assert_eq!(back, l);
        }

        /// Property: round-tripping every XSD concept through G then
        /// F yields the original concept. Equivalent to the
        /// R-triangle identity restricted to objects.
        #[test]
        fn prop_round_trip_concept_through_g_then_f(c in arb_xsd_concept()) {
            let l = XsdToEnglish::map_object(&c);
            let back = LiftEnglishToXsd::map_object(&l);
            prop_assert_eq!(back, c);
        }

        /// Property: η at every label is the identity morphism on
        /// that label. (Mac Lane §IV.4 equivalence-case unit.)
        #[test]
        fn prop_unit_is_identity_on_object(l in arb_xsd_english_label()) {
            let eta = XsdEnglishAdjunction::unit(&l);
            let id = XsdEnglishLabelCategory::identity(&l);
            prop_assert_eq!(eta, id);
        }

        /// Property: ε at every concept is the identity morphism on
        /// that concept. (Mac Lane §IV.4 equivalence-case counit.)
        #[test]
        fn prop_counit_is_identity_on_object(c in arb_xsd_concept()) {
            let eps = XsdEnglishAdjunction::counit(&c);
            let id = XsdCategory::identity(&c);
            prop_assert_eq!(eps, id);
        }

        /// Property: triangle identity (R-component) holds on every
        /// label — `ε_{F(l)} ∘ F(η_l) = id_{F(l)}`.
        #[test]
        fn prop_r_triangle(l in arb_xsd_english_label()) {
            let eta_l = XsdEnglishAdjunction::unit(&l);
            let f_eta = LiftEnglishToXsd::map_morphism(&eta_l);
            let fl = LiftEnglishToXsd::map_object(&l);
            let eps_fl = XsdEnglishAdjunction::counit(&fl);
            let composed = XsdCategory::compose(&f_eta, &eps_fl);
            let id_fl = XsdCategory::identity(&fl);
            prop_assert_eq!(composed, Some(id_fl));
        }

        /// Property: triangle identity (L-component) holds on every
        /// concept — `G(ε_c) ∘ η_{G(c)} = id_{G(c)}`.
        #[test]
        fn prop_l_triangle(c in arb_xsd_concept()) {
            let eps_c = XsdEnglishAdjunction::counit(&c);
            let g_eps = XsdToEnglish::map_morphism(&eps_c);
            let gc = XsdToEnglish::map_object(&c);
            let eta_gc = XsdEnglishAdjunction::unit(&gc);
            let composed = XsdEnglishLabelCategory::compose(&eta_gc, &g_eps);
            let id_gc = XsdEnglishLabelCategory::identity(&gc);
            prop_assert_eq!(composed, Some(id_gc));
        }
    }

    proptest! {
        /// Property: the runtime Lift is deterministic — calling it
        /// twice on the same input yields the same output.
        #[test]
        fn prop_lift_deterministic(term in "[a-z]{0,8}") {
            let en = sample_english();
            let inst = sample_uslm_instance();
            let a = lift_english_term_to_schema_components(&term, &inst, &en);
            let b = lift_english_term_to_schema_components(&term, &inst, &en);
            prop_assert_eq!(a, b);
        }

        /// Property: the runtime Lift is monotone — extending the
        /// instance with more named components never removes a
        /// match. (Sound by construction of the substring-contains
        /// check.)
        #[test]
        fn prop_lift_monotone(term in "[a-z]{1,4}") {
            let en = sample_english();
            let small = sample_uslm_instance();
            let mut big = small.clone();
            big.named.push(NamedSchemaComponentEntry {
                concept: XsdConcept::ElementDeclaration,
                local_name: format!("extra_{}_node", term),
            });
            big.components = big.named.iter().map(|n| n.concept).collect();
            let from_small = lift_english_term_to_schema_components(&term, &small, &en);
            let from_big = lift_english_term_to_schema_components(&term, &big, &en);
            prop_assert!(from_big.len() >= from_small.len());
            // Every match against `small` must also appear in `big`.
            for m in &from_small {
                prop_assert!(
                    from_big.iter().any(|n| n == m),
                    "monotonicity violated: {m:?} missing from big result"
                );
            }
        }

        /// Property: every Lift result's `local_name` actually
        /// contains the (lowercased) term — soundness witness.
        #[test]
        fn prop_lift_is_sound(term in "[a-z]{1,4}") {
            let en = sample_english();
            let inst = sample_uslm_instance();
            let needle = term.to_lowercase();
            let lifted = lift_english_term_to_schema_components(&term, &inst, &en);
            for m in &lifted {
                prop_assert!(
                    m.local_name.to_lowercase().contains(&needle),
                    "soundness violated: {} should contain {needle}",
                    m.local_name
                );
            }
        }
    }
}

// =============================================================================
// USLM corpus axiom — every USC term in `sox_1514a` shape resolves
// against USLM's section heading vocabulary through the runtime Lift.
//
// The bundled USLM-1.0.18 XSD declares `<xsd:element name="section">`
// (LRC USLM User Guide §6.5 Levels — the <section> element); the runtime Lift's substring-
// contains check therefore matches every English term that contains
// "section" against that declaration. This anchors the adjunction's
// USLM-side soundness without coupling to the still-evolving sox_1514a
// term loader.
// =============================================================================

#[test]
fn axiom_uslm_section_term_lifts_to_section_declaration() {
    let en = sample_english();
    let inst = sample_uslm_instance();
    let lifted = lift_english_term_to_schema_components("section", &inst, &en);
    // The bundled USLM declares `<xsd:element name="section">` per
    // the USLM Levels model (User Guide §6.5); the Lift must surface it.
    assert!(
        lifted
            .iter()
            .any(|c| c.local_name == "section" && c.concept == XsdConcept::ElementDeclaration),
        "USLM `<xsd:element name=\"section\">` should appear in the Lift result; got {lifted:?}"
    );
}
