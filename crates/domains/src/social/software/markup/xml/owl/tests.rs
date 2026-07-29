#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use super::reader;
use pr4xis::category::Category;
use pr4xis::category::entity::FinitelyGenerated;

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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn read_sample_owl() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    assert_eq!(ont.class_count().value, 3.0);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_class_has_label() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    let dog = ont.find_class("http://example.org/test#Dog").unwrap();
    assert_eq!(dog.label.as_deref(), Some("dog"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_subclass_taxonomy() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    assert_eq!(ont.taxonomy.len(), 2); // Dog→Mammal, Mammal→Animal
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_subclasses_of() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    let mammal_subs = ont.subclasses_of("http://example.org/test#Mammal");
    assert_eq!(mammal_subs.len(), 1);
    assert_eq!(mammal_subs[0].label.as_deref(), Some("dog"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_superclasses_of() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    let dog_supers = ont.superclasses_of("http://example.org/test#Dog");
    assert_eq!(dog_supers.len(), 1);
    assert!(dog_supers[0].contains("Mammal"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_property() {
    let ont = reader::read_owl(SAMPLE_OWL).unwrap();
    assert_eq!(ont.properties.len(), 1);
    assert_eq!(ont.properties[0].label.as_deref(), Some("has part"));
}

// A CiTO-shaped fragment: object properties arranged in an
// `rdfs:subPropertyOf` hierarchy (the structure CiTO uses throughout —
// e.g. citesAsEvidence ⊑ cites). Exercises the property-hierarchy
// extraction added for loading the SPAR vocabularies.
const SAMPLE_PROPERTY_HIERARCHY_OWL: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns="http://example.org/cito#">
  <owl:Ontology rdf:about="http://example.org/cito"/>
  <owl:ObjectProperty rdf:about="http://example.org/cito#cites">
    <rdfs:label>cites</rdfs:label>
    <rdfs:comment>the citing entity cites the cited entity</rdfs:comment>
  </owl:ObjectProperty>
  <owl:ObjectProperty rdf:about="http://example.org/cito#citesAsEvidence">
    <rdfs:label>cites as evidence</rdfs:label>
    <rdfs:subPropertyOf rdf:resource="http://example.org/cito#cites"/>
  </owl:ObjectProperty>
  <owl:ObjectProperty rdf:about="http://example.org/cito#includesQuotationFrom">
    <rdfs:label>includes quotation from</rdfs:label>
    <rdfs:subPropertyOf rdf:resource="http://example.org/cito#cites"/>
  </owl:ObjectProperty>
</rdf:RDF>"#;

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_property_comment_read() {
    let ont = reader::read_owl(SAMPLE_PROPERTY_HIERARCHY_OWL).unwrap();
    let cites = ont.find_property("http://example.org/cito#cites").unwrap();
    assert_eq!(
        cites.comment.as_deref(),
        Some("the citing entity cites the cited entity")
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_subproperty_taxonomy() {
    let ont = reader::read_owl(SAMPLE_PROPERTY_HIERARCHY_OWL).unwrap();
    assert_eq!(ont.property_count().value, 3.0);
    // citesAsEvidence→cites, includesQuotationFrom→cites
    assert_eq!(ont.property_taxonomy.len(), 2);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_subproperties_of() {
    let ont = reader::read_owl(SAMPLE_PROPERTY_HIERARCHY_OWL).unwrap();
    let subs = ont.subproperties_of("http://example.org/cito#cites");
    assert_eq!(subs.len(), 2);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_superproperties_of() {
    let ont = reader::read_owl(SAMPLE_PROPERTY_HIERARCHY_OWL).unwrap();
    let supers = ont.superproperties_of("http://example.org/cito#citesAsEvidence");
    assert_eq!(supers.len(), 1);
    assert!(supers[0].contains("cites"));
}

// =============================================================================
// CiTO bundle audit — load the registered SPAR Citation Typing Ontology
// =============================================================================

/// Walk the bundled, hash-pinned CiTO OWL vocabulary
/// (`crates/domains/data/ontologies/cito-2.8.1.owl`, registered in
/// praxis.toml as `[sources.cito]` and pinned in praxis.lock) through
/// `read_owl` and assert it loads the way the registry expects.
///
/// CiTO (Peroni & Shotton 2012, J. Web Semantics 17:33-43) is a
/// vocabulary of object properties: `cito:cites` and its ~40
/// sub-properties (e.g. `citesAsEvidence`, `includesQuotationFrom`,
/// `agreesWith`, `disputes`) plus their `cito:isCitedBy` inverses, all
/// arranged in an `rdfs:subPropertyOf` hierarchy. The bundled file MUST
/// exist (it is committed under `data/ontologies/`), so this test reads
/// it directly rather than skipping when absent.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn load_bundled_cito() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/cito-2.8.1.owl"
    );
    let xml = std::fs::read_to_string(path)
        .expect("bundled CiTO OWL must exist at data/ontologies/cito-2.8.1.owl");

    let ont = reader::read_owl(&xml).expect("bundled CiTO must parse through read_owl");

    // CiTO declares well over 30 object properties (the cito:cites /
    // cito:isCitedBy families together run to ~90). Assert a generous
    // lower bound so the test is robust to point-release churn.
    assert!(
        ont.property_count().value > 30.0,
        "expected CiTO to declare >30 object properties, got {}",
        ont.property_count().value
    );

    // Key CiTO citation-type properties resolve by IRI suffix. These
    // are the load-bearing relations downstream CitationQuality work
    // grounds in (Peroni & Shotton 2012 Table 1).
    for suffix in [
        "citesAsEvidence",
        "includesQuotationFrom",
        "agreesWith",
        "disputes",
    ] {
        let found = ont
            .properties
            .iter()
            .find(|p| p.iri.ends_with(suffix))
            .unwrap_or_else(|| panic!("CiTO must declare cito:{suffix}"));
        // Every CiTO sub-property carries a human label (rdfs:label).
        assert!(
            found.label.is_some(),
            "cito:{suffix} should carry an rdfs:label"
        );
    }

    // The CiTO property hierarchy is non-empty: its sub-properties roll
    // up to `cito:cites` / `cito:isCitedBy` via rdfs:subPropertyOf
    // edges, which `read_owl` records in `property_taxonomy`.
    assert!(
        !ont.property_taxonomy.is_empty(),
        "CiTO must record rdfs:subPropertyOf edges in property_taxonomy"
    );

    // citesAsEvidence ⊑ cites: the canonical CiTO sub-property edge.
    let cites_evidence = ont
        .properties
        .iter()
        .find(|p| p.iri.ends_with("citesAsEvidence"))
        .expect("cito:citesAsEvidence");
    assert!(
        cites_evidence
            .superproperties
            .iter()
            .any(|s| s.ends_with("cites")),
        "cito:citesAsEvidence must be a sub-property of cito:cites"
    );
}

// =============================================================================
// Entity-order determinism — reproducible parses (#264)
// =============================================================================

/// Two independent `read_owl` parses of the same bundled OWL file MUST
/// yield identical class and property IRI sequences.
///
/// `deduplicate_classes` / `deduplicate_properties` merge duplicate IRIs
/// (OWL reopens entities). They preserve first-occurrence document order
/// so the resulting `classes` / `properties` Vecs are deterministic across
/// processes — a hash-map iteration order (ahash, per-process seed) would
/// vary. Determinism here is the precondition for byte-reproducible
/// `.prx.gz` artifacts (the `prx` reproducibility test), since rkyv
/// serialises the Vecs in their stored order.
#[pr4xis::praxis_value(Deterministic)]
#[test]
fn read_owl_entity_order_is_deterministic() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/cito-2.8.1.owl"
    );
    let xml = std::fs::read_to_string(path)
        .expect("bundled CiTO OWL must exist at data/ontologies/cito-2.8.1.owl");

    let first = reader::read_owl(&xml).expect("first parse");
    let second = reader::read_owl(&xml).expect("second parse");

    let first_classes: Vec<&str> = first.classes.iter().map(|c| c.iri.as_str()).collect();
    let second_classes: Vec<&str> = second.classes.iter().map(|c| c.iri.as_str()).collect();
    assert_eq!(
        first_classes, second_classes,
        "class IRI order must be identical across independent parses"
    );

    let first_props: Vec<&str> = first.properties.iter().map(|p| p.iri.as_str()).collect();
    let second_props: Vec<&str> = second.properties.iter().map(|p| p.iri.as_str()).collect();
    assert_eq!(
        first_props, second_props,
        "property IRI order must be identical across independent parses"
    );

    // The merged-edge collections are sorted+deduped, so they are
    // order-stable too — verify alongside the entity Vecs.
    assert_eq!(
        first.taxonomy, second.taxonomy,
        "taxonomy order must be identical across independent parses"
    );
    assert_eq!(
        first.property_taxonomy, second.property_taxonomy,
        "property_taxonomy order must be identical across independent parses"
    );
}

// =============================================================================
// DoCO bundle audit — load the registered Document Components Ontology
// =============================================================================

/// Walk the bundled, hash-pinned DoCO OWL vocabulary
/// (`crates/domains/data/ontologies/doco-1.3.owl`, registered in
/// praxis.toml as `[sources.doco]` and pinned in praxis.lock) through
/// `read_owl` and assert it loads the way the registry expects.
///
/// DoCO (Constantin, Peroni, Pettifer, Shotton & Vitali 2016, Semantic
/// Web 7(2):167-181) is a vocabulary of document-component classes:
/// `doco:Paragraph`, `doco:Sentence`, `doco:Section`, `doco:Footnote`,
/// `doco:Title`, `doco:Figure`, `doco:Table`, … — the structural units a
/// document is decomposed into. The classes serialise in the RDF/XML
/// typed-node form (`<rdf:Description>` + `<rdf:type
/// rdf:resource=".../owl#Class"/>`). The bundled file MUST exist (it is
/// committed under `data/ontologies/`), so this test reads it directly
/// rather than skipping when absent.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn load_bundled_doco() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/ontologies/doco-1.3.owl");
    let xml = std::fs::read_to_string(path)
        .expect("bundled DoCO OWL must exist at data/ontologies/doco-1.3.owl");

    let ont = reader::read_owl(&xml).expect("bundled DoCO must parse through read_owl");

    // DoCO declares well over 50 named document-component classes (the
    // 2016 publication carries ~80). Assert a generous lower bound so
    // the test is robust to point-release churn.
    assert!(
        ont.class_count().value > 50.0,
        "expected DoCO to declare >50 classes, got {}",
        ont.class_count().value
    );

    // Load-bearing document-component classes resolve by IRI suffix.
    // These are the structural units downstream document-decomposition
    // grounds in (Constantin et al. 2016 §3).
    for suffix in [
        "doco/Paragraph",
        "doco/Sentence",
        "doco/Section",
        "doco/Footnote",
    ] {
        let found = ont
            .classes
            .iter()
            .find(|c| c.iri.ends_with(suffix))
            .unwrap_or_else(|| panic!("DoCO must declare {suffix}"));
        // Every DoCO component class carries a human label (rdfs:label).
        assert!(
            found.label.is_some(),
            "DoCO {suffix} should carry an rdfs:label"
        );
    }

    // The DoCO class hierarchy is non-empty: its component classes roll
    // up via rdfs:subClassOf edges, which `read_owl` records in
    // `taxonomy`.
    assert!(
        !ont.taxonomy.is_empty(),
        "DoCO must record rdfs:subClassOf edges in taxonomy"
    );
}

// =============================================================================
// C4O bundle audit — Citation Counting and Context Characterisation Ontology
// =============================================================================

/// Walk the bundled, hash-pinned C4O OWL vocabulary
/// (`crates/domains/data/ontologies/c4o-1.2.owl`, registered in
/// praxis.toml as `[sources.c4o]` and pinned in praxis.lock) through
/// `read_owl` and assert it loads the way the registry expects.
///
/// C4O (Di Iorio, Nuzzolese, Peroni, Shotton & Vitali 2014, SePublica;
/// part of the SPAR suite, Peroni & Shotton 2018) describes the number
/// and context of citations: object properties `c4o:denotes` /
/// `c4o:isDenotedBy`, `c4o:hasContent`, `c4o:hasContext`, … plus classes
/// such as `c4o:InTextReferencePointer` and `c4o:GlobalCitationCount`.
/// Serialised in the typed-node RDF/XML form. The bundled file MUST
/// exist, so this test reads it directly rather than skipping.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn load_bundled_c4o() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/ontologies/c4o-1.2.owl");
    let xml = std::fs::read_to_string(path)
        .expect("bundled C4O OWL must exist at data/ontologies/c4o-1.2.owl");

    let ont = reader::read_owl(&xml).expect("bundled C4O must parse through read_owl");

    // C4O declares its citation-context object properties (denotes,
    // isDenotedBy, hasContent, hasContext, isRelevantTo, pertainsTo).
    // Assert a generous lower bound on the property count.
    assert!(
        ont.property_count().value >= 4.0,
        "expected C4O to declare >=4 object properties, got {}",
        ont.property_count().value
    );

    // Load-bearing C4O citation-context relations resolve by IRI suffix
    // (Di Iorio et al. 2014).
    for suffix in ["c4o/denotes", "c4o/isDenotedBy", "c4o/hasContext"] {
        let found = ont
            .properties
            .iter()
            .find(|p| p.iri.ends_with(suffix))
            .unwrap_or_else(|| panic!("C4O must declare {suffix}"));
        assert!(
            found.label.is_some(),
            "C4O {suffix} should carry an rdfs:label"
        );
    }

    // C4O's `InTextReferencePointer` class — the in-text pointer that a
    // `c4o:denotes` edge originates from — resolves as a class.
    assert!(
        ont.classes
            .iter()
            .any(|c| c.iri.ends_with("c4o/InTextReferencePointer")),
        "C4O must declare c4o:InTextReferencePointer"
    );
}

// =============================================================================
// BiRO bundle audit — load the Bibliographic Reference Ontology
// =============================================================================

/// Walk the bundled, hash-pinned BiRO OWL vocabulary
/// (`crates/domains/data/ontologies/biro-1.1.1.owl`, registered in
/// praxis.toml as `[sources.biro]` and pinned in praxis.lock) through
/// `read_owl` and assert it loads the way the registry expects.
///
/// BiRO (Di Iorio, Nuzzolese, Peroni, Shotton & Vitali 2014, SePublica;
/// part of the SPAR suite, Peroni & Shotton 2018) models bibliographic
/// records and references: classes `biro:BibliographicRecord`,
/// `biro:BibliographicReference`, `biro:ReferenceList`,
/// `biro:BibliographicCollection`, … plus `biro:references` /
/// `biro:isReferencedBy` object properties. Serialised in the typed-node
/// RDF/XML form. The bundled file MUST exist, so this test reads it
/// directly rather than skipping.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn load_bundled_biro() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/biro-1.1.1.owl"
    );
    let xml = std::fs::read_to_string(path)
        .expect("bundled BiRO OWL must exist at data/ontologies/biro-1.1.1.owl");

    let ont = reader::read_owl(&xml).expect("bundled BiRO must parse through read_owl");

    // BiRO declares its bibliographic-record / reference classes. Assert
    // a generous lower bound on the class count.
    assert!(
        ont.class_count().value >= 4.0,
        "expected BiRO to declare >=4 classes, got {}",
        ont.class_count().value
    );

    // Load-bearing BiRO bibliographic classes resolve by IRI suffix
    // (Di Iorio et al. 2014).
    for suffix in [
        "biro/BibliographicRecord",
        "biro/BibliographicReference",
        "biro/ReferenceList",
    ] {
        let found = ont
            .classes
            .iter()
            .find(|c| c.iri.ends_with(suffix))
            .unwrap_or_else(|| panic!("BiRO must declare {suffix}"));
        assert!(
            found.comment.is_some() || found.label.is_some(),
            "BiRO {suffix} should carry an rdfs:label or rdfs:comment"
        );
    }
}

// =============================================================================
// PROV-O bundle audit — load the W3C PROV Ontology
// =============================================================================

/// Walk the bundled, hash-pinned PROV-O OWL vocabulary
/// (`crates/domains/data/ontologies/prov_o-2013-04-30.owl`, registered
/// in praxis.toml as `[sources.prov_o]` and pinned in praxis.lock)
/// through `read_owl` and assert it loads the way the registry expects.
///
/// PROV-O (Lebo, Sahoo & McGuinness eds. 2013, W3C Recommendation 30
/// April 2013) is the provenance interchange vocabulary: the core
/// classes `prov:Entity` / `prov:Activity` / `prov:Agent` and the
/// provenance object properties `prov:used`, `prov:wasGeneratedBy`,
/// `prov:wasAttributedTo`, `prov:wasDerivedFrom`, … . Unlike the SPAR
/// vocabularies, PROV-O serialises in the *striped* RDF/XML form
/// (`<owl:Class>` / `<owl:ObjectProperty>` typed-node elements). The
/// bundled file MUST exist, so this test reads it directly rather than
/// skipping.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn load_bundled_prov_o() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/prov_o-2013-04-30.owl"
    );
    let xml = std::fs::read_to_string(path)
        .expect("bundled PROV-O OWL must exist at data/ontologies/prov_o-2013-04-30.owl");

    let ont = reader::read_owl(&xml).expect("bundled PROV-O must parse through read_owl");

    // PROV-O declares ~30 classes and ~40 object properties. Assert
    // generous lower bounds on both axes (striped-form serialisation).
    assert!(
        ont.class_count().value > 20.0,
        "expected PROV-O to declare >20 classes, got {}",
        ont.class_count().value
    );
    assert!(
        ont.property_count().value > 30.0,
        "expected PROV-O to declare >30 object properties, got {}",
        ont.property_count().value
    );

    // The three core PROV-O classes (Lebo et al. 2013 §3, the starting
    // point of any provenance description) resolve by IRI suffix.
    for suffix in ["prov#Entity", "prov#Activity", "prov#Agent"] {
        assert!(
            ont.classes.iter().any(|c| c.iri.ends_with(suffix)),
            "PROV-O must declare {suffix}"
        );
    }

    // The load-bearing PROV-O provenance relations (Lebo et al. 2013
    // §2 "Starting Point" terms) resolve as object properties.
    for suffix in [
        "prov#used",
        "prov#wasGeneratedBy",
        "prov#wasAttributedTo",
        "prov#wasDerivedFrom",
    ] {
        assert!(
            ont.properties.iter().any(|p| p.iri.ends_with(suffix)),
            "PROV-O must declare {suffix}"
        );
    }
}

// =============================================================================
// OLiA test — load the real linguistic ontology
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn load_olia() {
    // Source the OLiA OWL from the registered, CI-fetched `olia` corpus
    // (`[sources.olia]`, bundled at data/ontologies/olia-2026-04-09.owl) —
    // not the stale `docs/papers/olia-reference-model.owl` path that no
    // longer exists. No graceful skip: an absent bundled vocab is a real
    // failure (mirrors `rdf_triple_reader_structural_content_audit`).
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/ontologies/olia-2026-04-09.owl");

    let xml = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "run `pr4xis update olia` to fetch the bundled OLiA OWL at {}; tests do not skip",
            path.display()
        )
    });

    let start = std::time::Instant::now();
    let ont = reader::read_owl(&xml).unwrap();
    let load_time = start.elapsed();

    eprintln!("=== OLiA Ontology ===");
    eprintln!("  Load time:     {:?}", load_time);
    eprintln!("  Classes:       {}", ont.class_count().value);
    eprintln!("  Properties:    {}", ont.properties.len());
    eprintln!("  Taxonomy:      {} relations", ont.taxonomy.len());

    // Should have substantial content
    assert!(
        ont.class_count().value > 100.0,
        "expected 100+ classes, got {}",
        ont.class_count().value
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
// Phase 1 derisk-gate audit — RDF-triple-based reader structural content
// =============================================================================

/// Walk every bundled OWL vocabulary through the new triple-based
/// `read_owl` and report the structural-content counts. Asserts that
/// each vocabulary produces a non-empty typed view — the per-vocab
/// counts are printed to stderr per `feedback_no_bounded_discovery_
/// counts` (the data determines the numbers, not the test). The new
/// fields (`labels`/`comments`/`annotations`/class expressions) get
/// audited alongside the classic counts.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rdf_triple_reader_structural_content_audit() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = root.join("data/ontologies");

    let vocabs = [
        "cito-2.8.1.owl",
        "doco-1.3.owl",
        "c4o-1.2.owl",
        "biro-1.1.1.owl",
        "prov_o-2013-04-30.owl",
        "olia-2026-04-09.owl",
    ];

    for name in &vocabs {
        let path = data_dir.join(name);
        let xml = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("bundled vocab must exist: {}", path.display()));
        let ont = reader::read_owl(&xml)
            .unwrap_or_else(|e| panic!("{name} must parse through new read_owl: {e}"));

        let total_labels: usize = ont
            .classes
            .iter()
            .map(|c| c.labels.len())
            .chain(ont.properties.iter().map(|p| p.labels.len()))
            .sum();
        let total_comments: usize = ont
            .classes
            .iter()
            .map(|c| c.comments.len())
            .chain(ont.properties.iter().map(|p| p.comments.len()))
            .sum();
        let total_annotations: usize = ont
            .classes
            .iter()
            .map(|c| c.annotations.len())
            .chain(ont.properties.iter().map(|p| p.annotations.len()))
            .sum();
        let total_class_exprs: usize = ont
            .classes
            .iter()
            .map(|c| {
                c.superclass_expressions.len()
                    + c.equivalent_classes.len()
                    + c.disjoint_classes.len()
            })
            .sum();
        let inverses = ont
            .properties
            .iter()
            .filter(|p| p.inverse_of.is_some())
            .count();

        eprintln!(
            "phase-1-audit: {name} \
             classes={} \
             props={} \
             individuals={} \
             taxonomy={} \
             prop_taxonomy={} \
             labels={total_labels} \
             comments={total_comments} \
             annotations={total_annotations} \
             class_exprs={total_class_exprs} \
             inverses={inverses} \
             header_annotations={}",
            ont.classes.len(),
            ont.properties.len(),
            ont.individuals.len(),
            ont.taxonomy.len(),
            ont.property_taxonomy.len(),
            ont.ontology_annotations.len(),
        );

        // A vocabulary that ships in this repo MUST surface at least
        // one structural fact through the new reader — either a class,
        // a property, or a header annotation. A bundle that produced
        // zero would mean the triple → OWL projection silently
        // dropped everything.
        assert!(
            !ont.classes.is_empty()
                || !ont.properties.is_empty()
                || !ont.ontology_annotations.is_empty(),
            "{name} produced an empty OwlOntology through the triple-based reader"
        );
    }
}

// =============================================================================
// Phase 2 derisk-gate audit — categorical round-trip on all 6 vocabs
// =============================================================================

/// W3C OWL 2 RDF Mapping (Patel-Schneider & Motik 2012) is a
/// set-theoretic projection. The praxis `read_owl` / `write_owl`
/// pair is well-behaved iff:
///
/// `read_owl(write_owl(read_owl(b))) ≡_graph read_owl(b)`
///
/// — the `read_owl` of the lens output is graph-equivalent to the
/// `read_owl` of the input bytes. (Byte-identical round-trip is the
/// stronger PutGet law verified at Phase 3, against `canonical`-
/// normalised bytes.)
///
/// This test walks every bundled OWL vocabulary through the
/// categorical round-trip and reports per-vocab equivalence.
#[pr4xis::praxis_value(Deterministic)]
#[test]
fn categorical_round_trip_six_owl_vocabularies() {
    use crate::social::software::markup::xml::owl::reader::owl_equivalent;
    use crate::social::software::markup::xml::owl::writer::write_owl;

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = root.join("data/ontologies");

    let vocabs = [
        "cito-2.8.1.owl",
        "doco-1.3.owl",
        "c4o-1.2.owl",
        "biro-1.1.1.owl",
        "prov_o-2013-04-30.owl",
        "olia-2026-04-09.owl",
    ];

    let mut failures: Vec<String> = Vec::new();

    for name in &vocabs {
        let path = data_dir.join(name);
        let xml = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("bundled vocab must exist: {}", path.display()));

        let ont1 = match reader::read_owl(&xml) {
            Ok(o) => o,
            Err(e) => {
                failures.push(format!("{name}: initial read_owl failed: {e}"));
                continue;
            }
        };
        let bytes = write_owl(&ont1);
        let text = match core::str::from_utf8(&bytes) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("{name}: write_owl emitted non-UTF-8: {e}"));
                continue;
            }
        };
        let ont2 = match reader::read_owl(text) {
            Ok(o) => o,
            Err(e) => {
                failures.push(format!("{name}: round-trip read_owl failed: {e}"));
                continue;
            }
        };

        if owl_equivalent(&ont1, &ont2) {
            eprintln!("phase-2-audit: {name} round-trip OK");
        } else {
            // Detailed diagnostic: which dimension drifted?
            let cls1: std::collections::HashSet<&str> =
                ont1.classes.iter().map(|c| c.iri.as_str()).collect();
            let cls2: std::collections::HashSet<&str> =
                ont2.classes.iter().map(|c| c.iri.as_str()).collect();
            let cls_diff_lr: Vec<&str> = cls1.difference(&cls2).copied().collect();
            let cls_diff_rl: Vec<&str> = cls2.difference(&cls1).copied().collect();

            let prp1: std::collections::HashSet<&str> =
                ont1.properties.iter().map(|p| p.iri.as_str()).collect();
            let prp2: std::collections::HashSet<&str> =
                ont2.properties.iter().map(|p| p.iri.as_str()).collect();
            let prp_diff_lr: Vec<&str> = prp1.difference(&prp2).copied().collect();
            let prp_diff_rl: Vec<&str> = prp2.difference(&prp1).copied().collect();

            eprintln!(
                "phase-2-audit: {name} round-trip NOT equivalent — \
                 c1={} c2={} p1={} p2={} i1={} i2={} tax1={} tax2={} ptax1={} ptax2={}\n  \
                 classes-only-in-input: {:?}\n  classes-only-in-output: {:?}\n  \
                 props-only-in-input: {:?}\n  props-only-in-output: {:?}",
                ont1.classes.len(),
                ont2.classes.len(),
                ont1.properties.len(),
                ont2.properties.len(),
                ont1.individuals.len(),
                ont2.individuals.len(),
                ont1.taxonomy.len(),
                ont2.taxonomy.len(),
                ont1.property_taxonomy.len(),
                ont2.property_taxonomy.len(),
                cls_diff_lr,
                cls_diff_rl,
                prp_diff_lr,
                prp_diff_rl,
            );

            // Detail per-class drift: find first class whose
            // literal/expression sets differ.
            for c1 in &ont1.classes {
                if let Some(c2) = ont2.classes.iter().find(|c| c.iri == c1.iri) {
                    if c1.labels.len() != c2.labels.len() {
                        eprintln!(
                            "  class {} labels drift: {} → {}",
                            c1.iri,
                            c1.labels.len(),
                            c2.labels.len()
                        );
                    }
                    if c1.comments.len() != c2.comments.len() {
                        eprintln!(
                            "  class {} comments drift: {} → {}",
                            c1.iri,
                            c1.comments.len(),
                            c2.comments.len()
                        );
                    }
                    if c1.superclass_expressions.len() != c2.superclass_expressions.len() {
                        eprintln!(
                            "  class {} superclass_exprs drift: {} → {}",
                            c1.iri,
                            c1.superclass_expressions.len(),
                            c2.superclass_expressions.len()
                        );
                    }
                }
            }
            for p1 in &ont1.properties {
                if let Some(p2) = ont2.properties.iter().find(|p| p.iri == p1.iri)
                    && (p1.labels.len() != p2.labels.len()
                        || p1.comments.len() != p2.comments.len()
                        || p1.inverse_of != p2.inverse_of
                        || p1.superproperties.len() != p2.superproperties.len())
                {
                    eprintln!(
                        "  prop {} drift: labels {}/{} comments {}/{} inv {:?}/{:?} sup {}/{}",
                        p1.iri,
                        p1.labels.len(),
                        p2.labels.len(),
                        p1.comments.len(),
                        p2.comments.len(),
                        p1.inverse_of,
                        p2.inverse_of,
                        p1.superproperties.len(),
                        p2.superproperties.len(),
                    );
                }
            }

            failures.push(format!("{name}: categorical round-trip not equivalent"));
        }
    }

    if !failures.is_empty() {
        panic!(
            "phase-2 round-trip failures:\n  - {}",
            failures.join("\n  - ")
        );
    }
}

// =============================================================================
// OWL category law tests
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn owl_identity_law() {
    for obj in OwlConcept::variants() {
        let id = OwlCategory::identity(&obj);
        assert_eq!(id.source, obj);
        assert_eq!(id.target, obj);
    }
}

#[pr4xis::praxis_value(Deterministic)]
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

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn owl_associativity() {
    let morphisms = OwlCategory::morphisms();
    for f in &morphisms {
        for g in morphisms.iter().filter(|g| g.source == f.target) {
            for h in morphisms.iter().filter(|h| h.source == g.target) {
                let fg = OwlCategory::compose(f, g);
                let gh = OwlCategory::compose(g, h);
                if let (Some(fg), Some(gh)) = (&fg, &gh) {
                    let f_gh = OwlCategory::compose(f, gh);
                    let fg_h = OwlCategory::compose(fg, h);
                    assert_eq!(f_gh, fg_h, "associativity: (f∘g)∘h = f∘(g∘h)");
                }
            }
        }
    }
}

// =============================================================================
// OWL vocabulary / concept lookup tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Honest)]
#[test]
fn owl_from_iri_unknown() {
    assert_eq!(OwlVocabulary::from_iri("http://example.org/foo"), None);
}

#[pr4xis::praxis_value(Verifiable, Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn owl_restriction_needs_property_axiom() {
    use pr4xis::ontology::Axiom;
    assert!(RestrictionNeedsProperty.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    use pr4xis::category::laws::assert_category_laws;
    assert_category_laws::<OwlCategory>();
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

    pr4xis::register_praxis_value!(prop_identity_idempotent, Deterministic);
    pr4xis::register_praxis_value!(prop_class_expression_classification, Verifiable);
    pr4xis::register_praxis_value!(prop_property_classification, Verifiable);
    pr4xis::register_praxis_value!(prop_left_identity, Deterministic);
}
