//! Tests for the source_taxonomy ontology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    EveryAdjointEdgeTyped, HartRule, HartRuleKind, LegalAdjunctionsTerminateInLanguage,
    PrimarySecondaryDistinction, SourceTaxonomyCategory, SourceTaxonomyConcept,
    SourceTaxonomyOntology, SourceTaxonomyWellFormed, adjoint_targets, ancestors_of, concept_name,
    is_leaf, is_legal_corpus, is_lexicon, parse_concept,
};
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<SourceTaxonomyCategory>();
}

#[test]
fn ontology_validates() {
    SourceTaxonomyOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn twenty_eight_concepts() {
    // Lexicon family (9): Source, Lexicon, Language, DomainLexicon,
    //                     LegalLexicon, SchemaVocabulary,
    //                     ClosedClassLexicon, InflectionLexicon,
    //                     DerivationalLexicon.
    // LegalCorpus family (8): LegalCorpus, Statute, UsFederalStatute,
    //                         UsCodeTitle, Regulation,
    //                         ConstitutionalArticle, ProceduralRule,
    //                         CaseLaw.
    // TypographyResource family (2): TypographyResource,
    //                                TypographicGlyphSet.
    // SchemaSpec family (6): SchemaSpec, XmlSchemaDefinition,
    //                        XmlDocumentTypeDefinition, OoxmlSchemaArchive,
    //                        ConceptualSpec, OntologyVocabulary.
    // TestSuite family (3): TestSuite, XmlSchemaTestSuite,
    //                       XmlConformanceTestSuite.
    assert_eq!(SourceTaxonomyConcept::variants().len(), 28);
}

#[test]
fn root_is_source() {
    assert!(ancestors_of(SourceTaxonomyConcept::Source).is_empty());
}

#[test]
fn language_ancestors_reach_source_via_lexicon() {
    let anc = ancestors_of(SourceTaxonomyConcept::Language);
    assert!(anc.contains(&SourceTaxonomyConcept::Lexicon));
    assert!(anc.contains(&SourceTaxonomyConcept::Source));
}

#[test]
fn legal_lexicon_is_a_domain_lexicon() {
    let anc = ancestors_of(SourceTaxonomyConcept::LegalLexicon);
    assert!(anc.contains(&SourceTaxonomyConcept::DomainLexicon));
    assert!(anc.contains(&SourceTaxonomyConcept::Lexicon));
    assert!(anc.contains(&SourceTaxonomyConcept::Source));
}

#[test]
fn statute_is_a_legal_corpus() {
    let anc = ancestors_of(SourceTaxonomyConcept::Statute);
    assert!(anc.contains(&SourceTaxonomyConcept::LegalCorpus));
    assert!(anc.contains(&SourceTaxonomyConcept::Source));
}

// =============================================================================
// Family predicates
// =============================================================================

#[test]
fn is_legal_corpus_recognizes_subtree() {
    use SourceTaxonomyConcept as C;
    for c in [
        C::LegalCorpus,
        C::Statute,
        C::Regulation,
        C::ConstitutionalArticle,
        C::ProceduralRule,
        C::CaseLaw,
    ] {
        assert!(is_legal_corpus(c), "{:?} should be LegalCorpus", c);
    }
    // Lexicon side should not be LegalCorpus.
    assert!(!is_legal_corpus(C::Language));
    assert!(!is_legal_corpus(C::LegalLexicon));
}

#[test]
fn is_lexicon_recognizes_subtree() {
    use SourceTaxonomyConcept as C;
    for c in [
        C::Lexicon,
        C::Language,
        C::DomainLexicon,
        C::LegalLexicon,
        C::SchemaVocabulary,
    ] {
        assert!(is_lexicon(c), "{:?} should be Lexicon", c);
    }
    assert!(!is_lexicon(C::Statute));
}

#[test]
fn is_leaf_agrees_with_the_loaded_is_a_graph() {
    use SourceTaxonomyConcept as C;
    // STRUCTURAL invariant (audit 2026-06-12 D-18/D-21), replacing the brittle
    // hand-counted `leaves.len() == N`: `is_leaf(c)` must hold EXACTLY when no
    // other concept has `c` among its ancestors — i.e. `c` is never an `is_a`
    // parent. Since `is_leaf` is now DERIVED from the same morphisms, this is the
    // honest cross-check that the derivation means what it claims, and it never
    // needs a hand edit when a concept is added.
    for c in SourceTaxonomyConcept::variants() {
        let has_a_descendant = SourceTaxonomyConcept::variants()
            .into_iter()
            .any(|other| other != c && ancestors_of(other).contains(&c));
        assert_eq!(
            is_leaf(c),
            !has_a_descendant,
            "is_leaf({c:?}) must equal `no concept is_a {c:?}`"
        );
    }
    // Spot-check known leaves and known intermediates so a degenerate empty
    // graph can't pass vacuously.
    assert!(is_leaf(C::Language));
    assert!(is_leaf(C::ClosedClassLexicon));
    assert!(is_leaf(C::UsFederalStatute));
    assert!(is_leaf(C::OntologyVocabulary));
    assert!(!is_leaf(C::Source)); // root
    assert!(!is_leaf(C::Lexicon)); // parent of Language/DomainLexicon/ClosedClassLexicon
    assert!(!is_leaf(C::Statute)); // parent of UsFederalStatute
    assert!(!is_leaf(C::SchemaSpec)); // parent of the schema-spec leaves
}

// =============================================================================
// Adjunction graph
// =============================================================================

#[test]
fn statute_adjoins_to_legal_lexicon_regulation_procedure() {
    let targets = adjoint_targets(SourceTaxonomyConcept::Statute);
    assert!(targets.contains(&SourceTaxonomyConcept::LegalLexicon));
    assert!(targets.contains(&SourceTaxonomyConcept::Regulation));
    assert!(targets.contains(&SourceTaxonomyConcept::ProceduralRule));
}

#[test]
fn legal_lexicon_adjoins_to_language() {
    let targets = adjoint_targets(SourceTaxonomyConcept::LegalLexicon);
    assert!(targets.contains(&SourceTaxonomyConcept::Language));
}

#[test]
fn case_law_adjoins_to_statute_regulation_lexicon() {
    let targets = adjoint_targets(SourceTaxonomyConcept::CaseLaw);
    assert!(targets.contains(&SourceTaxonomyConcept::Statute));
    assert!(targets.contains(&SourceTaxonomyConcept::Regulation));
    assert!(targets.contains(&SourceTaxonomyConcept::LegalLexicon));
}

#[test]
fn constitutional_article_adjoins_to_statute_and_case_law() {
    let targets = adjoint_targets(SourceTaxonomyConcept::ConstitutionalArticle);
    assert!(targets.contains(&SourceTaxonomyConcept::Statute));
    assert!(targets.contains(&SourceTaxonomyConcept::CaseLaw));
}

#[test]
fn schema_vocabulary_adjoins_to_language() {
    // Schema-vocabulary names anchor in WordNet (via productive
    // compounds / prefixation per Huddleston & Pullum 2002 Ch. 19)
    // — the adjunction edge surfaces (a) registered names whose
    // English base lemma isn't in WordNet, and (b) WordNet lemmas
    // no schema reuses.
    let targets = adjoint_targets(SourceTaxonomyConcept::SchemaVocabulary);
    assert!(targets.contains(&SourceTaxonomyConcept::Language));
}

#[test]
fn xml_schema_definition_adjoins_to_schema_vocabulary() {
    // W3C XSD 1.1 Part 1 §3: an XSD schema declares its
    // element/attribute/type/group/model names — i.e. the schema
    // vocabulary. The adjunction surfaces XSD-declared names not
    // registered in the vocabulary and vice versa.
    let targets = adjoint_targets(SourceTaxonomyConcept::XmlSchemaDefinition);
    assert!(targets.contains(&SourceTaxonomyConcept::SchemaVocabulary));
}

#[test]
fn xml_schema_definition_adjoins_to_uscodetitle() {
    // The USLM XSD instance grounds the UsCodeTitle ontology — the
    // adjunction edge surfaces (a) titles whose XML doesn't validate
    // against the schema, and (b) schema constructs that no published
    // title exercises. Cited: W3C XSD 1.1 Part 1 §1.1 (Gao,
    // Sperberg-McQueen & Thompson 2012).
    let targets = adjoint_targets(SourceTaxonomyConcept::XmlSchemaDefinition);
    assert!(targets.contains(&SourceTaxonomyConcept::UsCodeTitle));
}

#[test]
fn schema_spec_family_subsumption() {
    // SchemaSpec descends from Source; XmlSchemaDefinition descends
    // from SchemaSpec. Both invariants checked here so the family
    // membership is testable independently of `is_leaf`.
    let xsd_anc = ancestors_of(SourceTaxonomyConcept::XmlSchemaDefinition);
    assert!(xsd_anc.contains(&SourceTaxonomyConcept::SchemaSpec));
    assert!(xsd_anc.contains(&SourceTaxonomyConcept::Source));
    let schema_anc = ancestors_of(SourceTaxonomyConcept::SchemaSpec);
    assert!(schema_anc.contains(&SourceTaxonomyConcept::Source));
}

// =============================================================================
// Qualities
// =============================================================================

#[test]
fn hart_primary_classification() {
    use SourceTaxonomyConcept as C;
    let q = HartRule;
    assert_eq!(q.get(&C::Statute), Some(HartRuleKind::Primary));
    assert_eq!(
        q.get(&C::ConstitutionalArticle),
        Some(HartRuleKind::Primary)
    );
    assert_eq!(q.get(&C::ProceduralRule), Some(HartRuleKind::Primary));
}

#[test]
fn hart_secondary_classification() {
    use SourceTaxonomyConcept as C;
    let q = HartRule;
    assert_eq!(q.get(&C::Regulation), Some(HartRuleKind::Secondary));
    assert_eq!(q.get(&C::CaseLaw), Some(HartRuleKind::Secondary));
    assert_eq!(q.get(&C::LegalLexicon), Some(HartRuleKind::Secondary));
}

#[test]
fn hart_not_applicable_for_non_legal_concepts() {
    use SourceTaxonomyConcept as C;
    let q = HartRule;
    assert_eq!(q.get(&C::Language), Some(HartRuleKind::NotApplicable));
    assert_eq!(q.get(&C::Source), Some(HartRuleKind::NotApplicable));
}

// =============================================================================
// Domain axioms
// =============================================================================

#[test]
fn axiom_source_taxonomy_well_formed() {
    assert!(SourceTaxonomyWellFormed.verify().is_ok());
}

#[test]
fn axiom_every_adjoint_edge_typed() {
    assert!(EveryAdjointEdgeTyped.verify().is_ok());
}

#[test]
fn axiom_legal_adjunctions_terminate_in_language() {
    assert!(LegalAdjunctionsTerminateInLanguage.verify().is_ok());
}

#[test]
fn axiom_primary_secondary_distinction() {
    assert!(PrimarySecondaryDistinction.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in SourceTaxonomyOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Parser boundary (string ↔ concept)
// =============================================================================

#[test]
fn parse_concept_round_trips_every_variant() {
    for c in SourceTaxonomyConcept::variants() {
        let name = concept_name(c);
        let parsed = parse_concept(name).expect("canonical name should parse");
        assert_eq!(
            parsed, c,
            "round-trip failed for {:?} (canonical name: {})",
            c, name
        );
    }
}

#[test]
fn parse_concept_rejects_unknown_names() {
    assert!(parse_concept("NotAConcept").is_none());
    assert!(parse_concept("").is_none());
    assert!(parse_concept("statute").is_none()); // case-sensitive
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = SourceTaxonomyConcept> {
    proptest::sample::select(SourceTaxonomyConcept::variants())
}

proptest! {
    #[test]
    fn prop_every_arrow_named(_seed in any::<u32>()) {
        for m in SourceTaxonomyCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Every concept is either Source, or reachable to Source via is_a.
    /// Repeats `SourceTaxonomyWellFormed` as a proptest sanity check.
    #[test]
    fn prop_every_concept_reaches_source(c in arb_concept()) {
        if c == SourceTaxonomyConcept::Source {
            prop_assert!(true);
        } else {
            prop_assert!(ancestors_of(c).contains(&SourceTaxonomyConcept::Source));
        }
    }

    /// HartRule is total over the LegalCorpus + LegalLexicon leaves and
    /// NotApplicable for non-legal concepts.
    #[test]
    fn prop_hart_rule_total_on_legal_leaves(c in arb_concept()) {
        use SourceTaxonomyConcept as C;
        let q = HartRule;
        let v = q.get(&c);
        prop_assert!(v.is_some(), "HartRule should be total");
        // Concepts where HartRule's primary/secondary classification
        // applies. Statute is the jurisdiction-agnostic parent and
        // UsFederalStatute is its leaf — both classify as Primary.
        let is_legal_leaf = matches!(c,
            C::Statute | C::UsFederalStatute | C::Regulation
            | C::ConstitutionalArticle | C::ProceduralRule
            | C::CaseLaw | C::LegalLexicon);
        if is_legal_leaf {
            prop_assert!(v != Some(HartRuleKind::NotApplicable));
        } else {
            prop_assert_eq!(v, Some(HartRuleKind::NotApplicable));
        }
    }

    /// Subsumption morphisms must point toward an ancestor (no cycles).
    #[test]
    fn prop_is_a_is_acyclic(_seed in any::<u32>()) {
        for c in SourceTaxonomyConcept::variants() {
            let anc = ancestors_of(c);
            prop_assert!(!anc.contains(&c),
                "concept {:?} should not be its own ancestor", c);
        }
    }
}
