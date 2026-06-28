//! Ontology diagnostics — the meta-ontology of ontology engineering.
//!
//! Formalises the gap-detection methodology as a domain: components of
//! ontological analysis, the detection → resolution pipeline, qualities
//! of gaps and their severity, and axioms of the methodology.
//!
//! # Literature
//!
//! - **Spivak & Kent (2012)** "Ologs: A Categorical Framework for
//!   Knowledge Representation", *PLOS ONE* 7(1):e24274 — ologs as
//!   categorical ontologies.
//! - **Spivak (2014)** *Category Theory for the Sciences*, MIT Press
//!   — functors for cross-domain mapping.
//! - **Mac Lane (1971)** *Categories for the Working Mathematician*,
//!   Springer — adjunctions; unit and counit.
//! - **Euzenat & Shvaiko (2013)** *Ontology Matching*, 2nd ed.,
//!   Springer — ontology alignment (matching, distinct from
//!   gap-detection).
//! - **Schlobach & Cornet (2003)** "Non-Standard Reasoning Services for
//!   the Debugging of Description Logic Terminologies", *IJCAI 2003* —
//!   ontology debugging (consistency).
//! - **Herre & Loebe (2005)** *A Meta-Ontological Architecture for
//!   Foundational Ontologies* — meta-ontological framing.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Meta",
    source: "Spivak & Kent (2012) Ologs, PLOS ONE 7(1):e24274; Spivak (2014) Category Theory for the Sciences, MIT Press; Mac Lane (1971) Categories for the Working Mathematician, Springer; Euzenat & Shvaiko (2013) Ontology Matching 2nd ed., Springer; Schlobach & Cornet (2003) Non-Standard Reasoning Services for the Debugging of DL Terminologies, IJCAI; Herre & Loebe (2005) A Meta-Ontological Architecture for Foundational Ontologies",

    concepts: [
        // === Ontological structures ===
        DomainOntology,
        CategoryStructure,
        TaxonomyStructure,
        CausalStructure,
        QualityStructure,
        AxiomSet,
        // === Cross-domain connections ===
        Functor,
        Adjunction,
        UnitMorphism,
        CounitMorphism,
        NaturalTransformation,
        // === Gap-analysis findings ===
        UnitGap,
        CounitGap,
        GranularityMismatch,
        MissingDistinction,
        InformationLoss,
        CanonicalRepresentative,
        // === Resolution mechanisms ===
        ContextResolution,
        OntologyEnrichment,
        IntermediateDomain,
        GranularityRefinement,
        // === Verification ===
        LiteratureVerification,
        MachineProof,
        PropertyTest,
        // === Abstract categories ===
        Structure,
        Connection,
        Gap,
        Resolution,
        Verification,
        // === Methodology pipeline stages ===
        FormalizeDomains,
        ConstructFunctors,
        VerifyFunctorLaws,
        ConstructAdjunction,
        ComputeUnit,
        ComputeCounit,
        DetectGaps,
        ClassifyGaps,
        ComputeLossRatios,
        ProposeResolution,
        VerifyAgainstLiterature,
        ImplementResolution,
        RunMachineProofs,
        AssessImprovement,
    ],

    labels: {
        DomainOntology: ("en", "Domain ontology", "Spivak & Kent (2012): an ontology of a single subject domain."),
        CategoryStructure: ("en", "Category structure", "Mac Lane (1971): objects + morphisms + composition + identity."),
        TaxonomyStructure: ("en", "Taxonomy structure", "Gruber (1993): the is-a subsumption layer of an ontology."),
        CausalStructure: ("en", "Causal structure", "The cause→effect layer of an ontology."),
        QualityStructure: ("en", "Quality structure", "Property/value assignments over concepts."),
        AxiomSet: ("en", "Axiom set", "The set of axioms an ontology asserts."),
        Functor: ("en", "Functor", "Mac Lane (1971): a structure-preserving map between categories."),
        Adjunction: ("en", "Adjunction", "Mac Lane (1971) Ch. IV: F ⊣ G with unit η: 1 → GF and counit ε: FG → 1."),
        UnitMorphism: ("en", "Unit morphism", "Mac Lane (1971): η: 1 → GF - the unit of an adjunction."),
        CounitMorphism: ("en", "Counit morphism", "Mac Lane (1971): ε: FG → 1 - the counit of an adjunction."),
        NaturalTransformation: ("en", "Natural transformation", "Mac Lane (1971): a morphism between functors."),
        UnitGap: ("en", "Unit gap", "An entity for which the adjunction unit η is not identity - a forward round-trip loss."),
        CounitGap: ("en", "Counit gap", "An entity for which the counit ε is not identity - a backward round-trip loss."),
        GranularityMismatch: ("en", "Granularity mismatch", "Source and target ontologies operate at different conceptual scales."),
        MissingDistinction: ("en", "Missing distinction", "A distinction present in one ontology absent in the other."),
        InformationLoss: ("en", "Information loss", "The functor identifies entities that should remain distinct."),
        CanonicalRepresentative: ("en", "Canonical representative", "The entity chosen by a many-to-one functor as the canonical image."),
        ContextResolution: ("en", "Context resolution", "Add distinctions WITHOUT changing the category structure - non-destructive fix."),
        OntologyEnrichment: ("en", "Ontology enrichment", "Add new entities to one of the ontologies - may break existing functors."),
        IntermediateDomain: ("en", "Intermediate domain", "Add a third domain between the two with finer granularity."),
        GranularityRefinement: ("en", "Granularity refinement", "Refine within existing structure without adding entities."),
        LiteratureVerification: ("en", "Literature verification", "Manual verification against published sources - requires human reading."),
        MachineProof: ("en", "Machine proof", "Automated proof of structural properties (category laws, functor laws)."),
        PropertyTest: ("en", "Property test", "Property-based testing - statistically samples the search space."),
        Structure: ("en", "Structure", "Abstract category for ontological structures."),
        Connection: ("en", "Connection", "Abstract category for cross-domain connections."),
        Gap: ("en", "Gap", "Abstract category for gap-analysis findings."),
        Resolution: ("en", "Resolution", "Abstract category for resolution mechanisms."),
        Verification: ("en", "Verification", "Abstract category for verification approaches."),

        FormalizeDomains: ("en", "Formalize domains", "Methodology stage 1: formalise two scientific domains as categories."),
        ConstructFunctors: ("en", "Construct functors", "Methodology stage 2: build functors in both directions."),
        VerifyFunctorLaws: ("en", "Verify functor laws", "Methodology stage 3: verify identity + composition preservation."),
        ConstructAdjunction: ("en", "Construct adjunction", "Methodology stage 4: pair functors with unit and counit."),
        ComputeUnit: ("en", "Compute unit", "Methodology stage 5a: compute unit for every source entity."),
        ComputeCounit: ("en", "Compute counit", "Methodology stage 5b: compute counit for every target entity."),
        DetectGaps: ("en", "Detect gaps", "Methodology stage 6: identify gaps (unit ≠ id or counit ≠ id)."),
        ClassifyGaps: ("en", "Classify gaps", "Methodology stage 7a: classify gaps (mismatch, missing, ...)."),
        ComputeLossRatios: ("en", "Compute loss ratios", "Methodology stage 7b: fraction of entities that collapse."),
        ProposeResolution: ("en", "Propose resolution", "Methodology stage 8: propose ContextDef / enrichment / intermediate."),
        VerifyAgainstLiterature: ("en", "Verify against literature", "Methodology stage 9: confirm proposal in published sources."),
        ImplementResolution: ("en", "Implement resolution", "Methodology stage 10: implement the proposed resolution."),
        RunMachineProofs: ("en", "Run machine proofs", "Methodology stage 11: run automated tests."),
        AssessImprovement: ("en", "Assess improvement", "Methodology stage 12: assess whether loss ratio improved."),
    },

    is_a: [
        (DomainOntology, Structure),
        (CategoryStructure, Structure),
        (TaxonomyStructure, Structure),
        (CausalStructure, Structure),
        (QualityStructure, Structure),
        (AxiomSet, Structure),
        (Functor, Connection),
        (Adjunction, Connection),
        (UnitMorphism, Connection),
        (CounitMorphism, Connection),
        (NaturalTransformation, Connection),
        (UnitGap, Gap),
        (CounitGap, Gap),
        (GranularityMismatch, Gap),
        (MissingDistinction, Gap),
        (InformationLoss, Gap),
        (CanonicalRepresentative, Gap),
        (ContextResolution, Resolution),
        (OntologyEnrichment, Resolution),
        (IntermediateDomain, Resolution),
        (GranularityRefinement, Resolution),
        (LiteratureVerification, Verification),
        (MachineProof, Verification),
        (PropertyTest, Verification),
    ],

    has_a: [
        // Adjunction components.
        (Adjunction, UnitMorphism),
        (Adjunction, CounitMorphism),
    ],

    causes: [
        (FormalizeDomains, ConstructFunctors),
        (ConstructFunctors, VerifyFunctorLaws),
        (VerifyFunctorLaws, ConstructAdjunction),
        (ConstructAdjunction, ComputeUnit),
        (ConstructAdjunction, ComputeCounit),
        (ComputeUnit, DetectGaps),
        (ComputeCounit, DetectGaps),
        (DetectGaps, ClassifyGaps),
        (DetectGaps, ComputeLossRatios),
        (ClassifyGaps, ProposeResolution),
        (ComputeLossRatios, ProposeResolution),
        (ProposeResolution, VerifyAgainstLiterature),
        (VerifyAgainstLiterature, ImplementResolution),
        (ImplementResolution, RunMachineProofs),
        (RunMachineProofs, AssessImprovement),
    ],

    opposes: [
        // Gap vs Resolution (problem vs solution).
        (Gap, Resolution),
        (Resolution, Gap),
        // Forward vs backward round-trip.
        (UnitGap, CounitGap),
        (CounitGap, UnitGap),
        // Automated vs manual verification.
        (MachineProof, LiteratureVerification),
        (LiteratureVerification, MachineProof),
        // Context preserves functors; enrichment may break them.
        (ContextResolution, OntologyEnrichment),
        (OntologyEnrichment, ContextResolution),
    ],
}

/// Loss-ratio threshold categories. Based on empirical findings across
/// three adjunctions in pr4xis: >80% needs IntermediateDomain;
/// 40-80% needs ContextResolution; <40% needs GranularityRefinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LossThreshold {
    Low,
    Moderate,
    High,
}

/// Quality: is this gap type detectable automatically by adjunction
/// analysis?
#[derive(Debug, Clone)]
pub struct IsAutoDetectable;

impl Quality for IsAutoDetectable {
    type Individual = MetaConcept;
    type Value = bool;

    fn get(&self, c: &MetaConcept) -> Option<bool> {
        use MetaConcept as M;
        match c {
            M::UnitGap
            | M::CounitGap
            | M::InformationLoss
            | M::CanonicalRepresentative
            | M::GranularityMismatch => Some(true),
            M::MissingDistinction => Some(false),
            _ => None,
        }
    }
}

/// Quality: does this resolution type preserve functor validity?
/// ContextResolution adds distinctions without changing category structure;
/// OntologyEnrichment may break existing functors.
#[derive(Debug, Clone)]
pub struct PreservesFunctorValidity;

impl Quality for PreservesFunctorValidity {
    type Individual = MetaConcept;
    type Value = bool;

    fn get(&self, c: &MetaConcept) -> Option<bool> {
        use MetaConcept as M;
        match c {
            M::ContextResolution | M::GranularityRefinement => Some(true),
            M::OntologyEnrichment | M::IntermediateDomain => Some(false),
            _ => None,
        }
    }
}

/// Quality: which loss-ratio threshold suggests this resolution type?
#[derive(Debug, Clone)]
pub struct SuggestedForLossLevel;

impl Quality for SuggestedForLossLevel {
    type Individual = MetaConcept;
    type Value = LossThreshold;

    fn get(&self, c: &MetaConcept) -> Option<LossThreshold> {
        use MetaConcept as M;
        match c {
            M::GranularityRefinement => Some(LossThreshold::Low),
            M::ContextResolution | M::OntologyEnrichment => Some(LossThreshold::Moderate),
            M::IntermediateDomain => Some(LossThreshold::High),
            _ => None,
        }
    }
}

/// Quality: is this verification type automated?
#[derive(Debug, Clone)]
pub struct IsAutomated;

impl Quality for IsAutomated {
    type Individual = MetaConcept;
    type Value = bool;

    fn get(&self, c: &MetaConcept) -> Option<bool> {
        use MetaConcept as M;
        match c {
            M::MachineProof | M::PropertyTest => Some(true),
            M::LiteratureVerification => Some(false),
            _ => None,
        }
    }
}

// Re-export the legacy `MetaEntity` name as an alias.
pub type MetaEntity = MetaConcept;

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Gap detection requires both unit and counit computation. Mac Lane
/// (1971): an adjunction has both η and ε.
pub struct GapDetectionRequiresBothDirections;

impl Axiom for GapDetectionRequiresBothDirections {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let causation: Vec<_> = MetaCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MetaRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        let ok = causation.contains(&(MetaConcept::ComputeUnit, MetaConcept::DetectGaps))
            && causation.contains(&(MetaConcept::ComputeCounit, MetaConcept::DetectGaps));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GapDetectionRequiresBothDirections",
        "DetectGaps is caused by both ComputeUnit and ComputeCounit",
        "Mac Lane (1971) Categories for the Working Mathematician Ch. IV"
    );
}

pr4xis::register_axiom!(
    GapDetectionRequiresBothDirections,
    "Mac Lane (1971) Categories for the Working Mathematician Ch. IV"
);

/// ContextResolution preserves functor validity (Spivak 2014: structure-
/// preserving refinement).
pub struct ContextResolutionPreservesFunctors;

impl Axiom for ContextResolutionPreservesFunctors {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if PreservesFunctorValidity.get(&MetaConcept::ContextResolution) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ContextResolutionPreservesFunctors",
        "ContextResolution preserves existing functor validity (non-destructive fix)",
        "Spivak (2014) Category Theory for the Sciences, MIT Press"
    );
}

pr4xis::register_axiom!(
    ContextResolutionPreservesFunctors,
    "Spivak (2014) Category Theory for the Sciences, MIT Press"
);

/// High loss (>80%) suggests an intermediate domain. Empirical finding
/// from three pr4xis adjunctions.
pub struct HighLossSuggestsIntermediateDomain;

impl Axiom for HighLossSuggestsIntermediateDomain {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if SuggestedForLossLevel.get(&MetaConcept::IntermediateDomain) == Some(LossThreshold::High)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "HighLossSuggestsIntermediateDomain",
        ">80% loss suggests an IntermediateDomain resolution",
        "Spivak & Kent (2012) Ologs, PLOS ONE 7(1):e24274"
    );
}

pr4xis::register_axiom!(
    HighLossSuggestsIntermediateDomain,
    "Spivak & Kent (2012) Ologs, PLOS ONE 7(1):e24274"
);

impl Ontology for MetaOntology {
    type Cat = MetaCategory;
    type Qual = IsAutoDetectable;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(GapDetectionRequiresBothDirections));
        axioms.push(Box::new(ContextResolutionPreservesFunctors));
        axioms.push(Box::new(HighLossSuggestsIntermediateDomain));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<MetaCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MetaOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gap_detection_requires_both_directions_holds() {
        assert!(GapDetectionRequiresBothDirections.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn context_resolution_preserves_functors_holds() {
        assert!(ContextResolutionPreservesFunctors.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn high_loss_suggests_intermediate_domain_holds() {
        assert!(HighLossSuggestsIntermediateDomain.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_pipeline_path() {
        // Causation transitive closure: FormalizeDomains reaches AssessImprovement.
        let causation: Vec<_> = MetaCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == MetaRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(causation.contains(&(
            MetaConcept::FormalizeDomains,
            MetaConcept::AssessImprovement
        )));
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MetaCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MetaOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = MetaConcept::variants();
            for m in MetaCategory::morphisms() {
                if m.kind() == MetaRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
}
