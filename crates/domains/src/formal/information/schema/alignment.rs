use pr4xis::category::Entity;
use pr4xis::define_ontology;

// Ontology Alignment ontology — discovering connections between ontologies.
//
// An alignment finds correspondences between entities in different ontologies.
// Categorically: an alignment is a SPAN O1 ← A → O2, not a functor.
// This captures partiality (not every entity need map) and multiplicity.
//
// When the system encounters concepts from disconnected ontologies
// (UnknownUnknown in the epistemic model), alignment is the metacognitive
// process of discovering the missing functor.
//
// References:
// - Euzenat & Shvaiko, "Ontology Matching" (2007, 2nd ed. 2013, Springer)
//   Chapter 2: correspondences, relation algebra, confidence
// - Zimmermann, Krötzsch, Euzenat & Hitzler, "Formalizing Ontology
//   Alignment and its Operations with Category Theory" (FOIS 2006)
//   Alignment as span, merge as pushout, composition via pullback
// - Kalfoglou & Schorlemmer, "Ontology Mapping: The State of the Art"
//   (2003, Knowledge Engineering Review) — mapping/alignment/merging distinctions
// - OAEI (Ontology Alignment Evaluation Initiative) — evaluation metrics

/// Concepts in the alignment ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AlignmentConcept {
    /// A set of correspondences between two ontologies.
    /// Euzenat & Shvaiko (2013): A ⊆ C (set of correspondences).
    /// Zimmermann (2006): a span O1 ← A → O2 in Cat.
    Alignment,

    /// A single correspondence: (entity1, entity2, relation, confidence).
    /// Euzenat & Shvaiko (2013), Definition 3.1.
    Correspondence,

    /// The semantic relation between aligned entities.
    /// From the relation algebra: ≡, ⊑, ⊒, ⊥, ∩.
    CorrespondenceRelation,

    /// Confidence value in [0,1] for a correspondence.
    /// Euzenat & Shvaiko (2013), Section 2.3.
    /// Enrichment over the monoidal category ([0,1], ×, 1).
    Confidence,

    /// Matching technique used to discover correspondences.
    /// OAEI taxonomy: string-based, structural, semantic, extensional.
    MatchingTechnique,

    /// Discovery phase: find candidate correspondences.
    /// Generate morphism candidates between ontologies.
    Discovery,

    /// Evaluation phase: score correspondences with confidence.
    /// Enrichment functor to [0,1].
    Evaluation,

    /// Refinement phase: filter, compose, negotiate.
    /// Kan extension or colimit.
    Refinement,

    /// Execution phase: apply alignment to transform data.
    /// Pushforward functor (Spivak ΣF).
    Execution,

    /// Merge: create new ontology unifying both via alignment.
    /// Zimmermann (2006): pushout of the alignment span.
    Merge,

    /// Coherence check: alignment must not create unsatisfiable concepts.
    /// Meilicke et al. (2007): Mod(O1 + A + O2) must be non-empty.
    Coherence,
}

/// Semantic relations between aligned entities.
/// Euzenat & Shvaiko (2013), Chapter 2 — a relation algebra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRelation {
    /// ≡ — entities denote the same concept. Categorical: isomorphism.
    Equivalence,
    /// ⊑ — entity1 is more specific than entity2. Categorical: monomorphism.
    SubsumedBy,
    /// ⊒ — entity1 is more general than entity2. Categorical: epimorphism.
    Subsumes,
    /// ⊥ — entities share no instances. Categorical: zero morphism.
    Disjoint,
    /// ∩ — entities share some but not all instances. Categorical: pullback exists.
    Overlap,
}

impl SemanticRelation {
    /// Inverse the relation direction.
    /// Zimmermann (2006), Proposition 2: swap legs of the span.
    pub fn inverse(&self) -> Self {
        match self {
            Self::Equivalence => Self::Equivalence, // symmetric
            Self::SubsumedBy => Self::Subsumes,
            Self::Subsumes => Self::SubsumedBy,
            Self::Disjoint => Self::Disjoint, // symmetric
            Self::Overlap => Self::Overlap,   // symmetric
        }
    }

    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::Equivalence | Self::Disjoint | Self::Overlap)
    }
}

/// Matching technique types from OAEI taxonomy.
/// Euzenat & Shvaiko (2013), Chapter 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchingType {
    /// Compare entity labels using string similarity.
    /// Edit distance, n-gram, Jaro-Winkler.
    StringBased,
    /// Use linguistic resources (WordNet synonyms, morphology).
    LanguageBased,
    /// Compare graph topology: shared neighbors, path length.
    /// Melnik et al. (2002): similarity flooding.
    Structural,
    /// Compare overlapping instances (Jaccard index).
    Extensional,
    /// Use logical axioms: SAT-based, model-theoretic.
    /// Giunchiglia & Shvaiko (2003): S-Match.
    Semantic,
    /// Compose through intermediate ontology: O1→Obridge→O2.
    /// Functor composition.
    Compositional,
}

define_ontology! {
    pub AlignmentOntology for AlignmentCategory {
        concepts: AlignmentConcept,
        relation: AlignmentRelation,
        kind: AlignmentRelationKind,
        kinds: [
            /// Alignment contains Correspondences.
            Contains,
            /// Correspondence has a CorrespondenceRelation and Confidence.
            HasRelation,
            HasConfidence,
            /// Discovery produces Alignment.
            Produces,
            /// MatchingTechnique drives Discovery.
            Drives,
            /// Lifecycle: Discovery → Evaluation → Refinement → Execution.
            Precedes,
            /// Merge consumes Alignment (pushout of span).
            Consumes,
            /// Coherence validates Alignment.
            Validates,
        ],
        edges: [
            // Alignment contains Correspondences
            (Alignment, Correspondence, Contains),
            // Correspondence has relation and confidence
            (Correspondence, CorrespondenceRelation, HasRelation),
            (Correspondence, Confidence, HasConfidence),
            // MatchingTechnique drives Discovery
            (MatchingTechnique, Discovery, Drives),
            // Discovery produces Alignment
            (Discovery, Alignment, Produces),
            // Lifecycle: Discovery → Evaluation → Refinement → Execution
            (Discovery, Evaluation, Precedes),
            (Evaluation, Refinement, Precedes),
            (Refinement, Execution, Precedes),
            // Merge consumes Alignment (pushout)
            (Merge, Alignment, Consumes),
            // Coherence validates Alignment
            (Coherence, Alignment, Validates),
        ],
        composed: [
            // Discovery → Alignment → Correspondence
            (Discovery, Correspondence),
            // MatchingTechnique → Alignment
            (MatchingTechnique, Alignment),
            // Discovery → Execution (full lifecycle)
            (Discovery, Execution),
        ],
        being: AbstractObject,
        source: "Spivak (2012); Euzenat & Shvaiko (2013)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<AlignmentCategory>().unwrap();
    }

    #[test]
    fn has_eleven_concepts() {
        assert_eq!(AlignmentConcept::variants().len(), 11);
    }

    // --- Euzenat & Shvaiko: Alignment contains Correspondences ---

    #[test]
    fn alignment_contains_correspondences() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Alignment
            && r.to == AlignmentConcept::Correspondence
            && r.kind == AlignmentRelationKind::Contains));
    }

    // --- Euzenat & Shvaiko: Correspondence has relation and confidence ---

    #[test]
    fn correspondence_has_relation_and_confidence() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Correspondence
            && r.to == AlignmentConcept::CorrespondenceRelation));
        assert!(
            m.iter().any(|r| r.from == AlignmentConcept::Correspondence
                && r.to == AlignmentConcept::Confidence)
        );
    }

    // --- Zimmermann (2006): Alignment lifecycle ---

    #[test]
    fn lifecycle_order() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Discovery
            && r.to == AlignmentConcept::Evaluation
            && r.kind == AlignmentRelationKind::Precedes));
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Evaluation
            && r.to == AlignmentConcept::Refinement
            && r.kind == AlignmentRelationKind::Precedes));
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Refinement
            && r.to == AlignmentConcept::Execution
            && r.kind == AlignmentRelationKind::Precedes));
    }

    #[test]
    fn discovery_reaches_execution() {
        let m = AlignmentCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == AlignmentConcept::Discovery
                    && r.to == AlignmentConcept::Execution)
        );
    }

    // --- Zimmermann (2006): Merge is pushout of alignment span ---

    #[test]
    fn merge_consumes_alignment() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Merge
            && r.to == AlignmentConcept::Alignment
            && r.kind == AlignmentRelationKind::Consumes));
    }

    // --- Meilicke (2007): Coherence validates alignment ---

    #[test]
    fn coherence_validates_alignment() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Coherence
            && r.to == AlignmentConcept::Alignment
            && r.kind == AlignmentRelationKind::Validates));
    }

    // --- Semantic relation algebra ---

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
    fn disjointness_is_symmetric() {
        assert!(SemanticRelation::Disjoint.is_symmetric());
        assert_eq!(
            SemanticRelation::Disjoint.inverse(),
            SemanticRelation::Disjoint
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

    // --- OAEI: Six matching technique types ---

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

    // --- Discovery produces Alignment ---

    #[test]
    fn discovery_produces_alignment() {
        let m = AlignmentCategory::morphisms();
        assert!(m.iter().any(|r| r.from == AlignmentConcept::Discovery
            && r.to == AlignmentConcept::Alignment
            && r.kind == AlignmentRelationKind::Produces));
    }

    // --- MatchingTechnique drives Discovery ---

    #[test]
    fn technique_drives_discovery() {
        let m = AlignmentCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == AlignmentConcept::MatchingTechnique
                    && r.to == AlignmentConcept::Discovery
                    && r.kind == AlignmentRelationKind::Drives)
        );
    }
}
