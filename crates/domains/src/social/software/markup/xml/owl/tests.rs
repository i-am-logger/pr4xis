#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use super::reader;
use pr4xis::category::Category;
use pr4xis::category::entity::Concept;
use pr4xis::logic::Axiom;

const SAMPLE_OWL: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns="http://example.org/test#">
  <owl:Ontology rdf:about="http://example.org/test"/>
  <owl:Class rdf:about="http://example.org/test#Animal">
    <rdfs:label>animal</rdfs:label>
    <rdfs:comment>a living organism</rdfs:comment>
  </owl:Class>
  <owl:Class rdf:about="http://example.org/test#Mammal">
    <rdfs:label>mammal</rdfs:label>
    <rdfs:subClassOf rdf:resource="http://example.org/test#Animal"/>
  </owl:Class>
  <owl:Class rdf:about="http://example.org/test#Dog">
    <rdfs:label>dog</rdfs:label>
    <rdfs:subClassOf rdf:resource="http://example.org/test#Mammal"/>
  </owl:Class>
  <owl:ObjectProperty rdf:about="http://example.org/test#hasPart">
    <rdfs:label>has part</rdfs:label>
    <rdfs:domain rdf:resource="http://example.org/test#Animal"/>
    <rdfs:range rdf:resource="http://example.org/test#Animal"/>
  </owl:ObjectProperty>
</rdf:RDF>"#;

#[test]
fn read_sample_owl() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    assert_eq!(ont.class_count(), 3);
}

#[test]
fn owl_class_has_label() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    let dog = ont.find_class("http://example.org/test#Dog").unwrap();
    assert_eq!(dog.label.as_deref(), Some("dog"));
}

#[test]
fn owl_subclass_taxonomy() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    assert_eq!(ont.taxonomy.len(), 2); // Dog→Mammal, Mammal→Animal
}

#[test]
fn owl_subclasses_of() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    let mammal_subs = ont.subclasses_of("http://example.org/test#Mammal");
    assert_eq!(mammal_subs.len(), 1);
    assert_eq!(mammal_subs[0].label.as_deref(), Some("dog"));
}

#[test]
fn owl_superclasses_of() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    let dog_supers = ont.superclasses_of("http://example.org/test#Dog");
    assert_eq!(dog_supers.len(), 1);
    assert!(dog_supers[0].contains("Mammal"));
}

#[test]
fn owl_property() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    assert_eq!(ont.properties.len(), 1);
    assert_eq!(ont.properties[0].label.as_deref(), Some("has part"));
}

// =============================================================================
// OLiA test — load the real linguistic ontology
// =============================================================================

#[test]
fn load_olia() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/papers/olia-reference-model.owl"
    );

    if !std::path::Path::new(path).exists() {
        eprintln!("SKIP: OLiA OWL not found");
        return;
    }

    let xml = std::fs::read_to_string(path).unwrap();

    let start = std::time::Instant::now();
    let ont = reader::read_owl(&xml).unwrap();
    let load_time = start.elapsed();

    eprintln!("=== OLiA Ontology ===");
    eprintln!("  Load time:     {:?}", load_time);
    eprintln!("  Classes:       {}", ont.class_count());
    eprintln!("  Properties:    {}", ont.properties.len());
    eprintln!("  Taxonomy:      {} relations", ont.taxonomy.len());

    // Should have substantial content
    assert!(
        ont.class_count() > 100,
        "expected 100+ classes, got {}",
        ont.class_count()
    );

    // Should have key linguistic classes
    let copula = ont.classes.iter().find(|c| c.iri.contains("Copula"));
    assert!(copula.is_some(), "OLiA should define Copula");

    let determiner = ont
        .classes
        .iter()
        .find(|c| c.iri.ends_with("#Determiner") || c.iri.ends_with("Determiner"));
    assert!(determiner.is_some(), "OLiA should define Determiner");

    let pronoun = ont
        .classes
        .iter()
        .find(|c| c.iri.ends_with("Pronoun") || c.iri.ends_with("PronounOrDeterminer"));
    assert!(pronoun.is_some(), "OLiA should define Pronoun");

    eprintln!("  Copula:        {:?}", copula.map(|c| &c.iri));
    eprintln!("  Determiner:    {:?}", determiner.map(|c| &c.iri));

    // List some subclasses of Determiner
    if let Some(det) = determiner {
        let det_subs = ont.subclasses_of(&det.iri);
        let sub_labels: Vec<&str> = det_subs.iter().filter_map(|c| c.label.as_deref()).collect();
        eprintln!("  Det subtypes:  {:?}", sub_labels);
    }

    // Verify AuxiliaryVerb and InterrogativePronoun exist (OLiA-specific classes)
    let aux = ont
        .classes
        .iter()
        .find(|c| c.iri.ends_with("#AuxiliaryVerb"));
    assert!(aux.is_some(), "OLiA should define AuxiliaryVerb");

    let interr = ont
        .classes
        .iter()
        .find(|c| c.iri.ends_with("#InterrogativePronoun"));
    assert!(interr.is_some(), "OLiA should define InterrogativePronoun");

    // Explore key POS categories
    let pos_keywords = [
        "Noun",
        "Verb",
        "Adjective",
        "Adverb",
        "Pronoun",
        "Determiner",
        "Preposition",
        "Conjunction",
        "Copula",
        "Auxiliary",
        "Article",
        "Interjection",
        "Particle",
        "Numeral",
        "Interrogative",
    ];
    eprintln!("\n=== Key POS Classes ===");
    for kw in pos_keywords {
        let matches: Vec<&str> = ont
            .classes
            .iter()
            .filter(|c| {
                let frag = c.iri.rsplit_once('#').map(|(_, f)| f).unwrap_or(&c.iri);
                frag == kw
            })
            .map(|c| c.iri.as_str())
            .collect();
        if !matches.is_empty() {
            eprintln!("  {}: {:?}", kw, matches);
        } else {
            eprintln!("  {}: NOT FOUND (exact)", kw);
        }
    }
}

// =============================================================================
// OWL category law tests
// =============================================================================

#[test]
fn owl_identity_law() {
    for obj in OwlConcept::variants() {
        let id = OwlCategory::identity(&obj);
        assert_eq!(id.source, obj);
        assert_eq!(id.target, obj);
    }
}

#[test]
fn owl_composition_with_identity() {
    let morphisms = OwlCategory::morphisms();
    for m in &morphisms {
        let id_src = OwlCategory::identity(&m.source);
        let composed = OwlCategory::compose(&id_src, m);
        assert_eq!(composed.as_ref(), Some(m));

        let id_tgt = OwlCategory::identity(&m.target);
        let composed = OwlCategory::compose(m, &id_tgt);
        assert_eq!(composed.as_ref(), Some(m));
    }
}

#[test]
fn owl_associativity() {
    let morphisms = OwlCategory::morphisms();
    for f in &morphisms {
        for g in morphisms.iter().filter(|g| g.source == f.target) {
            for h in morphisms.iter().filter(|h| h.source == g.target) {
                let fg = OwlCategory::compose(f, g);
                let gh = OwlCategory::compose(g, h);
                if let (Some(fg), Some(gh)) = (&fg, &gh) {
                    let f_gh = OwlCategory::compose(f, &gh);
                    let fg_h = OwlCategory::compose(&fg, h);
                    assert_eq!(f_gh, fg_h, "associativity: (f∘g)∘h = f∘(g∘h)");
                }
            }
        }
    }
}

// =============================================================================
// OWL vocabulary / concept lookup tests
// =============================================================================

#[test]
fn owl_from_iri_known() {
    assert_eq!(
        OwlVocabulary::from_iri(OwlVocabulary::OWL_CLASS),
        Some(OwlConcept::Class)
    );
    assert_eq!(
        OwlVocabulary::from_iri(OwlVocabulary::OWL_OBJECT_PROPERTY),
        Some(OwlConcept::ObjectProperty)
    );
    assert_eq!(
        OwlVocabulary::from_iri(OwlVocabulary::OWL_NAMED_INDIVIDUAL),
        Some(OwlConcept::NamedIndividual)
    );
}

#[test]
fn owl_from_iri_unknown() {
    assert_eq!(OwlVocabulary::from_iri("http://example.org/foo"), None);
}

#[test]
fn owl_from_local_name() {
    assert_eq!(
        OwlVocabulary::from_local_name("Class"),
        Some(OwlConcept::Class)
    );
    assert_eq!(
        OwlVocabulary::from_local_name("ObjectProperty"),
        Some(OwlConcept::ObjectProperty)
    );
    assert_eq!(OwlVocabulary::from_local_name("UnknownThing"), None);
}

#[test]
fn owl_concept_classification() {
    assert!(OwlConcept::Class.is_class_expression());
    assert!(OwlConcept::Restriction.is_class_expression());
    assert!(!OwlConcept::ObjectProperty.is_class_expression());

    assert!(OwlConcept::ObjectProperty.is_property());
    assert!(OwlConcept::DatatypeProperty.is_property());
    assert!(!OwlConcept::Class.is_property());

    assert!(OwlConcept::TransitiveProperty.is_property_characteristic());
    assert!(!OwlConcept::Class.is_property_characteristic());
}

#[test]
fn owl_restriction_needs_property_axiom() {
    assert!(RestrictionNeedsProperty.holds());
}

#[test]
fn category_laws() {
    use pr4xis::category::validate::check_category_laws;
    check_category_laws::<OwlCategory>().unwrap();
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_owl() -> impl Strategy<Value = OwlConcept> {
        prop_oneof![
            Just(OwlConcept::Class),
            Just(OwlConcept::Restriction),
            Just(OwlConcept::UnionOf),
            Just(OwlConcept::IntersectionOf),
            Just(OwlConcept::ComplementOf),
            Just(OwlConcept::OneOf),
            Just(OwlConcept::ObjectProperty),
            Just(OwlConcept::DatatypeProperty),
            Just(OwlConcept::AnnotationProperty),
            Just(OwlConcept::FunctionalProperty),
            Just(OwlConcept::InverseFunctionalProperty),
            Just(OwlConcept::TransitiveProperty),
            Just(OwlConcept::SymmetricProperty),
            Just(OwlConcept::AsymmetricProperty),
            Just(OwlConcept::ReflexiveProperty),
            Just(OwlConcept::IrreflexiveProperty),
            Just(OwlConcept::NamedIndividual),
            Just(OwlConcept::SomeValuesFrom),
            Just(OwlConcept::AllValuesFrom),
            Just(OwlConcept::HasValue),
            Just(OwlConcept::MinCardinality),
            Just(OwlConcept::MaxCardinality),
            Just(OwlConcept::ExactCardinality),
            Just(OwlConcept::Ontology),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_owl()) {
            let id = OwlCategory::identity(&c);
            prop_assert_eq!(OwlCategory::compose(&id, &id), Some(id));
        }

        /// OWL 2 §8: class expressions are class expressions.
        #[test]
        fn prop_class_expression_classification(c in arb_owl()) {
            if c.is_class_expression() {
                prop_assert!(!c.is_property());
            }
        }

        /// OWL 2 §9: properties are properties.
        #[test]
        fn prop_property_classification(c in arb_owl()) {
            if c.is_property() {
                prop_assert!(!c.is_class_expression());
            }
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_owl()) {
            let m = OwlCategory::morphisms();
            let id = OwlCategory::identity(&c);
            for morph in m.iter().filter(|r| r.source == c) {
                let composed = OwlCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.source, r.target)), Some((morph.source, morph.target)));
            }
        }
    }
}
