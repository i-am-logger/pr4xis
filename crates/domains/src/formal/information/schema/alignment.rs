//! Ontology alignment — discovering correspondences between ontologies.
//!
//! An alignment finds correspondences between entities in different
//! ontologies. Categorically, an alignment is a span `O1 ← A → O2`
//! (Zimmermann et al. 2006), not a functor — capturing partiality and
//! multiplicity. The pushout of the span IS the merge.
//!
//! # Literature
//!
//! - **Euzenat & Shvaiko (2013)** *Ontology Matching*, 2nd ed., Springer
//!   — correspondences, relation algebra (≡, ⊑, ⊒, ⊥, ∩), confidence,
//!   matching-technique taxonomy (string / language / structural /
//!   extensional / semantic / compositional).
//! - **Zimmermann, Krötzsch, Euzenat & Hitzler (2006)** "Formalizing
//!   Ontology Alignment and its Operations with Category Theory",
//!   *FOIS 2006* — alignment as span; merge as pushout; composition
//!   via pullback.
//! - **Kalfoglou & Schorlemmer (2003)** "Ontology Mapping: The State of
//!   the Art", *Knowledge Engineering Review* 18(1):1-31 — mapping /
//!   alignment / merging distinctions.
//! - **Meilicke, Stuckenschmidt & Tamilin (2007)** "Repairing
//!   Ontology Mappings", *AAAI 2007* — coherence: Mod(O1 ∪ A ∪ O2)
//!   must be non-empty.
//! - **Melnik, Garcia-Molina & Rahm (2002)** "Similarity Flooding",
//!   *ICDE 2002* — structural matching via fixpoint iteration.
//! - **Giunchiglia & Shvaiko (2003)** "Semantic Matching: Algorithms
//!   and Implementation", *Journal on Data Semantics IX* — S-Match.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Alignment",
    source: "Euzenat & Shvaiko (2013) Ontology Matching 2nd ed., Springer; Zimmermann, Krötzsch, Euzenat & Hitzler (2006) Formalizing Ontology Alignment and its Operations with Category Theory, FOIS; Kalfoglou & Schorlemmer (2003) Ontology Mapping: The State of the Art, KER 18(1); Meilicke, Stuckenschmidt & Tamilin (2007) Repairing Ontology Mappings, AAAI; Melnik, Garcia-Molina & Rahm (2002) Similarity Flooding, ICDE",

    concepts: [
        Alignment,
        Correspondence,
        CorrespondenceRelation,
        Confidence,
        MatchingTechnique,
        Discovery,
        Evaluation,
        Refinement,
        Execution,
        Merge,
        Coherence,
    ],

    labels: {
        Alignment: ("en", "Alignment",
            "Euzenat & Shvaiko (2013) Def. 2.1: a set of correspondences between two ontologies. Zimmermann (2006): a span O1 ← A → O2 in Cat."),
        Correspondence: ("en", "Correspondence",
            "Euzenat & Shvaiko (2013) Def. 3.1: a single (entity1, entity2, relation, confidence) tuple."),
        CorrespondenceRelation: ("en", "Correspondence relation",
            "Euzenat & Shvaiko (2013) Ch. 2: the semantic relation between aligned entities (≡, ⊑, ⊒, ⊥, ∩)."),
        Confidence: ("en", "Confidence",
            "Euzenat & Shvaiko (2013) §2.3: a value in [0,1] indicating belief in a correspondence; enrichment over the monoidal category ([0,1], ×, 1)."),
        MatchingTechnique: ("en", "Matching technique",
            "Euzenat & Shvaiko (2013) Ch. 4: a method to discover correspondences - OAEI taxonomy (string / language / structural / extensional / semantic / compositional)."),
        Discovery: ("en", "Discovery",
            "Generate candidate correspondences between two ontologies."),
        Evaluation: ("en", "Evaluation",
            "Score candidate correspondences with confidence values - enrichment functor to [0,1]."),
        Refinement: ("en", "Refinement",
            "Filter, compose, and negotiate the candidate alignment - Kan extension or colimit."),
        Execution: ("en", "Execution",
            "Apply the alignment to transform data - pushforward functor (Spivak ΣF)."),
        Merge: ("en", "Merge",
            "Zimmermann (2006): pushout of the alignment span - the new ontology unifying both."),
        Coherence: ("en", "Coherence",
            "Meilicke et al. (2007): alignment must not create unsatisfiable concepts; Mod(O1 ∪ A ∪ O2) must be non-empty."),
    },

    edges: [
        // Euzenat & Shvaiko (2013): alignment contains correspondences.
        (Alignment, Correspondence, Contains),
        // Each correspondence has a relation and a confidence.
        (Correspondence, CorrespondenceRelation, HasRelation),
        (Correspondence, Confidence, HasConfidence),
        // MatchingTechnique drives Discovery (Euzenat & Shvaiko Ch. 4).
        (MatchingTechnique, Discovery, Drives),
        // Discovery produces Alignment.
        (Discovery, Alignment, Produces),
        // Lifecycle: Discovery → Evaluation → Refinement → Execution.
        (Discovery, Evaluation, Precedes),
        (Evaluation, Refinement, Precedes),
        (Refinement, Execution, Precedes),
        // Zimmermann (2006): Merge consumes Alignment (pushout).
        (Merge, Alignment, Consumes),
        // Meilicke (2007): Coherence validates Alignment.
        (Coherence, Alignment, Validates),
    ],
}

/// Semantic relations between aligned entities. Euzenat & Shvaiko (2013)
/// Ch. 2 — a relation algebra adapted from description logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRelation {
    /// ≡ — entities denote the same concept (categorical: isomorphism).
    Equivalence,
    /// ⊑ — entity1 is more specific (categorical: monomorphism).
    SubsumedBy,
    /// ⊒ — entity1 is more general (categorical: epimorphism).
    Subsumes,
    /// ⊥ — disjoint (categorical: zero morphism).
    Disjoint,
    /// ∩ — overlap (categorical: pullback exists).
    Overlap,
}

impl SemanticRelation {
    /// Zimmermann (2006) Proposition 2: swap legs of the span.
    pub fn inverse(&self) -> Self {
        match self {
            Self::Equivalence => Self::Equivalence,
            Self::SubsumedBy => Self::Subsumes,
            Self::Subsumes => Self::SubsumedBy,
            Self::Disjoint => Self::Disjoint,
            Self::Overlap => Self::Overlap,
        }
    }

    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::Equivalence | Self::Disjoint | Self::Overlap)
    }
}

/// OAEI matching-technique taxonomy. Euzenat & Shvaiko (2013) Ch. 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchingType {
    /// Edit distance, n-gram, Jaro-Winkler.
    StringBased,
    /// WordNet synonyms, morphology.
    LanguageBased,
    /// Melnik et al. (2002) similarity flooding.
    Structural,
    /// Compare overlapping instances (Jaccard index).
    Extensional,
    /// Giunchiglia & Shvaiko (2003) S-Match.
    Semantic,
    /// Compose through intermediate ontology O1 → Obridge → O2.
    Compositional,
}

/// Quality: whether a concept is on the alignment lifecycle (Discovery
/// → Evaluation → Refinement → Execution).
#[derive(Debug, Clone)]
pub struct IsLifecycleStage;

impl Quality for IsLifecycleStage {
    type Individual = AlignmentConcept;
    type Value = bool;

    fn get(&self, c: &AlignmentConcept) -> Option<bool> {
        use AlignmentConcept as A;
        Some(matches!(
            c,
            A::Discovery | A::Evaluation | A::Refinement | A::Execution
        ))
    }
}

impl Ontology for AlignmentOntology {
    type Cat = AlignmentCategory;
    type Qual = IsLifecycleStage;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<AlignmentCategory>();
    }

    #[test]
    fn ontology_validates() {
        AlignmentOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn eleven_concepts() {
        assert_eq!(AlignmentConcept::variants().len(), 11);
    }

    #[test]
    fn alignment_contains_correspondences() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == AlignmentConcept::Alignment
            && r.target() == AlignmentConcept::Correspondence
            && r.kind() == AlignmentRelationKind::Contains));
    }

    #[test]
    fn lifecycle_order() {
        let m = AlignmentCategory::morphisms();
        use AlignmentConcept as A;
        for (from, to) in [
            (A::Discovery, A::Evaluation),
            (A::Evaluation, A::Refinement),
            (A::Refinement, A::Execution),
        ] {
            assert!(m.iter().any(|r| r.source() == from
                && r.target() == to
                && r.kind() == AlignmentRelationKind::Precedes));
        }
    }

    #[test]
    fn merge_consumes_alignment() {
        // Zimmermann (2006) pushout of alignment span.
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == AlignmentConcept::Merge
            && r.target() == AlignmentConcept::Alignment
            && r.kind() == AlignmentRelationKind::Consumes));
    }

    #[test]
    fn coherence_validates_alignment() {
        // Meilicke et al. (2007).
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == AlignmentConcept::Coherence
            && r.target() == AlignmentConcept::Alignment
            && r.kind() == AlignmentRelationKind::Validates));
    }

    #[test]
    fn equivalence_is_symmetric() {
        assert!(SemanticRelation::Equivalence.is_symmetric());
    }

    #[test]
    fn subsumption_inverse() {
        assert_eq!(
            SemanticRelation::SubsumedBy.inverse(),
            SemanticRelation::Subsumes
        );
        assert_eq!(
            SemanticRelation::Subsumes.inverse(),
            SemanticRelation::SubsumedBy
        );
    }

    #[test]
    fn inverse_of_inverse_is_identity() {
        for rel in [
            SemanticRelation::Equivalence,
            SemanticRelation::SubsumedBy,
            SemanticRelation::Subsumes,
            SemanticRelation::Disjoint,
            SemanticRelation::Overlap,
        ] {
            assert_eq!(rel.inverse().inverse(), rel);
        }
    }

    #[test]
    fn six_matching_types() {
        let types = [
            MatchingType::StringBased,
            MatchingType::LanguageBased,
            MatchingType::Structural,
            MatchingType::Extensional,
            MatchingType::Semantic,
            MatchingType::Compositional,
        ];
        assert_eq!(types.len(), 6);
    }

    fn arb_concept() -> impl Strategy<Value = AlignmentConcept> {
        proptest::sample::select(AlignmentConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in AlignmentCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in AlignmentOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_lifecycle_total(c in arb_concept()) {
            prop_assert!(IsLifecycleStage.get(&c).is_some());
        }

        #[test]
        fn prop_semantic_inverse_involutive(_seed in any::<u32>()) {
            for r in [SemanticRelation::Equivalence, SemanticRelation::SubsumedBy,
                      SemanticRelation::Subsumes, SemanticRelation::Disjoint,
                      SemanticRelation::Overlap] {
                prop_assert_eq!(r.inverse().inverse(), r);
            }
        }
    }
}
