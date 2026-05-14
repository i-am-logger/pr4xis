//! Analytical methods — formalises structural analysis methodology, its
//! components, its outputs, and the canonical analysis pipeline.
//!
//! This is a PURE-SCIENCE ontology of analysis methods — not an
//! implementation of analysis itself. It supplies the vocabulary the
//! diagnostic layer reasons over when it inspects other ontologies.
//!
//! # Literature
//!
//! - **Wille (1982)** "Restructuring Lattice Theory: An Approach Based on
//!   Hierarchies of Concepts", in *Ordered Sets* (Reidel) — Formal Concept
//!   Analysis: formal contexts, derivation operators, concept lattices.
//! - **Ganter & Wille (1999)** *Formal Concept Analysis: Mathematical
//!   Foundations*, Springer — the textbook treatment of FCA including
//!   Galois connections, concept lattice construction, and attribute
//!   exploration.
//! - **Birkhoff (1940)** *Lattice Theory*, AMS Colloquium Publications 25 —
//!   algebraic structure of partial orders that underwrites FCA.
//! - **Tukey (1977)** *Exploratory Data Analysis*, Addison-Wesley — the
//!   methodological distinction between exploratory (structural / pattern)
//!   analysis and confirmatory (statistical) analysis.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "AnalyticalMethods",
    source: "Wille (1982) Restructuring Lattice Theory; Ganter & Wille (1999) Formal Concept Analysis: Mathematical Foundations; Birkhoff (1940) Lattice Theory; Tukey (1977) Exploratory Data Analysis",

    concepts: [
        // === Methods (how you analyse) ===
        StructuralAnalysis,
        PatternAnalysis,
        StatisticalAnalysis,
        ComparativeAnalysis,
        AbsorptionAnalysis,
        ClusterAnalysis,

        // === Components (what you work with) ===
        FormalContext,
        ConceptLattice,
        GaloisConnection,
        ObjectSet,
        AttributeSet,
        BinaryRelation,

        // === Outputs (what you produce) ===
        Pattern,
        Cluster,
        Anomaly,
        Invariant,

        // === Abstract categories ===
        AnalysisMethod,
        AnalysisComponent,
        AnalysisOutput,

        // === Pipeline stages (Wille 1982 §2; Ganter & Wille 1999 Ch. 1) ===
        DataCollection,
        ContextFormation,
        DerivationComputation,
        LatticeConstruction,
        PatternExtraction,
        AnomalyDetection,
        ResultInterpretation,
        KnowledgeUpdate,
    ],

    labels: {
        StructuralAnalysis: ("en", "Structural analysis",
            "Wille (1982): analysis that recovers the lattice structure implicit in a formal context — concept lattice construction."),
        PatternAnalysis: ("en", "Pattern analysis",
            "Identification of recurring sub-structures (implications, association rules) in a formal context. Ganter & Wille (1999) §3."),
        StatisticalAnalysis: ("en", "Statistical analysis",
            "Tukey (1977): summarisation of data via distributions, moments, and confidence intervals — the confirmatory complement to structural / exploratory analysis."),
        ComparativeAnalysis: ("en", "Comparative analysis",
            "Analysis that contrasts two or more formal contexts (or sub-lattices) — requires human judgment on what to compare."),
        AbsorptionAnalysis: ("en", "Absorption analysis",
            "Birkhoff (1940) Ch. I: lattice-theoretic absorption laws used to identify redundant concepts."),
        ClusterAnalysis: ("en", "Cluster analysis",
            "Grouping of objects by attribute similarity — the FCA equivalent is concept clustering on the lattice."),
        FormalContext: ("en", "Formal context",
            "Wille (1982): a triple (G, M, I) of objects, attributes, and an incidence relation I subset of G x M."),
        ConceptLattice: ("en", "Concept lattice",
            "Ganter & Wille (1999) Theorem 3: the set of formal concepts (extent, intent) of a context, ordered by extent inclusion, forms a complete lattice."),
        GaloisConnection: ("en", "Galois connection",
            "Birkhoff (1940) Ch. V: the antitone pair of derivation operators ' between 2^G and 2^M that establishes the concept lattice."),
        ObjectSet: ("en", "Object set",
            "Wille (1982): the set G of formal-context objects (rows of the incidence table)."),
        AttributeSet: ("en", "Attribute set",
            "Wille (1982): the set M of formal-context attributes (columns of the incidence table)."),
        BinaryRelation: ("en", "Binary relation",
            "Wille (1982): the incidence relation I subset of G x M of a formal context."),
        Pattern: ("en", "Pattern",
            "A recurring sub-structure detected in the data — an FCA implication, an association rule, or a sub-lattice."),
        Cluster: ("en", "Cluster",
            "A group of objects sharing a defining intent — a concept's extent."),
        Anomaly: ("en", "Anomaly",
            "An object or attribute whose participation in the context deviates from the dominant pattern — Tukey (1977) outlier."),
        Invariant: ("en", "Invariant",
            "A property that holds across all instances of a concept — an implication A => B that holds in every model of the context."),
        AnalysisMethod: ("en", "Analysis method",
            "Abstract category for analysis methods — supertype of StructuralAnalysis, PatternAnalysis, etc."),
        AnalysisComponent: ("en", "Analysis component",
            "Abstract category for the structural pieces an analysis works with (contexts, lattices, sets, relations)."),
        AnalysisOutput: ("en", "Analysis output",
            "Abstract category for what an analysis produces (patterns, clusters, anomalies, invariants)."),

        DataCollection: ("en", "Data collection",
            "Pipeline stage 1: gather the raw observations that will populate a formal context."),
        ContextFormation: ("en", "Context formation",
            "Pipeline stage 2: form the formal context (G, M, I) from the collected data. Wille (1982) §2."),
        DerivationComputation: ("en", "Derivation computation",
            "Pipeline stage 3: compute the derivation operators ' (the Galois connection). Birkhoff (1940) Ch. V."),
        LatticeConstruction: ("en", "Lattice construction",
            "Pipeline stage 4: build the concept lattice from the derivation operators. Ganter & Wille (1999) Algorithm Next-Closure."),
        PatternExtraction: ("en", "Pattern extraction",
            "Pipeline stage 5: extract recurring sub-structures (implications, association rules) from the lattice."),
        AnomalyDetection: ("en", "Anomaly detection",
            "Pipeline stage 6: flag deviations from the dominant patterns. Tukey (1977)."),
        ResultInterpretation: ("en", "Result interpretation",
            "Pipeline stage 7: interpret patterns and anomalies in domain terms — requires human judgment."),
        KnowledgeUpdate: ("en", "Knowledge update",
            "Pipeline stage 8: integrate the interpretation back into the knowledge base."),
    },

    is_a: [
        // Methods classify under AnalysisMethod.
        (StructuralAnalysis, AnalysisMethod),
        (PatternAnalysis, AnalysisMethod),
        (StatisticalAnalysis, AnalysisMethod),
        (ComparativeAnalysis, AnalysisMethod),
        (AbsorptionAnalysis, AnalysisMethod),
        (ClusterAnalysis, AnalysisMethod),
        // Components classify under AnalysisComponent.
        (FormalContext, AnalysisComponent),
        (ConceptLattice, AnalysisComponent),
        (GaloisConnection, AnalysisComponent),
        (ObjectSet, AnalysisComponent),
        (AttributeSet, AnalysisComponent),
        (BinaryRelation, AnalysisComponent),
        // Outputs classify under AnalysisOutput.
        (Pattern, AnalysisOutput),
        (Cluster, AnalysisOutput),
        (Anomaly, AnalysisOutput),
        (Invariant, AnalysisOutput),
    ],

    causes: [
        // Canonical FCA analysis pipeline — each stage causally precedes
        // the next. Wille (1982) §2; Ganter & Wille (1999) Ch. 1.
        (DataCollection, ContextFormation),
        (ContextFormation, DerivationComputation),
        (DerivationComputation, LatticeConstruction),
        (LatticeConstruction, PatternExtraction),
        (PatternExtraction, AnomalyDetection),
        (AnomalyDetection, ResultInterpretation),
        (ResultInterpretation, KnowledgeUpdate),
    ],

    opposes: [
        // Structure vs distribution — two complementary lenses on data
        // (Tukey 1977 distinguishes exploratory vs confirmatory analysis).
        (StructuralAnalysis, StatisticalAnalysis),
        (StatisticalAnalysis, StructuralAnalysis),
        // Regularity vs deviation — what an analysis finds in the data.
        (Pattern, Anomaly),
        (Anomaly, Pattern),
    ],
}

/// Computational complexity class of an analysis method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityClass {
    Linear,
    Quadratic,
    Exponential,
}

/// Quality: whether a method admits full automation.
///
/// Ganter & Wille (1999) §3: Next-Closure et al. are total algorithms — no
/// human judgment required. Comparative and absorption analyses involve
/// domain choices about what to compare or what to absorb.
#[derive(Debug, Clone)]
pub struct IsAutomatable;

impl Quality for IsAutomatable {
    type Individual = AnalyticalMethodsConcept;
    type Value = bool;

    fn get(&self, c: &AnalyticalMethodsConcept) -> Option<bool> {
        use AnalyticalMethodsConcept as C;
        match c {
            C::StructuralAnalysis
            | C::PatternAnalysis
            | C::StatisticalAnalysis
            | C::ClusterAnalysis => Some(true),
            C::ComparativeAnalysis | C::AbsorptionAnalysis => Some(false),
            _ => None,
        }
    }
}

/// Quality: whether a method requires human judgment for interpretation.
#[derive(Debug, Clone)]
pub struct RequiresHumanJudgment;

impl Quality for RequiresHumanJudgment {
    type Individual = AnalyticalMethodsConcept;
    type Value = bool;

    fn get(&self, c: &AnalyticalMethodsConcept) -> Option<bool> {
        use AnalyticalMethodsConcept as C;
        match c {
            C::StructuralAnalysis
            | C::PatternAnalysis
            | C::StatisticalAnalysis
            | C::ClusterAnalysis => Some(false),
            C::ComparativeAnalysis | C::AbsorptionAnalysis => Some(true),
            _ => None,
        }
    }
}

/// Quality: worst-case complexity class. Ganter & Wille (1999) §3 — the
/// concept lattice has up to 2^min(|G|,|M|) concepts, hence exponential.
#[derive(Debug, Clone)]
pub struct Complexity;

impl Quality for Complexity {
    type Individual = AnalyticalMethodsConcept;
    type Value = ComplexityClass;

    fn get(&self, c: &AnalyticalMethodsConcept) -> Option<ComplexityClass> {
        use AnalyticalMethodsConcept as C;
        match c {
            C::StatisticalAnalysis => Some(ComplexityClass::Linear),
            C::ClusterAnalysis
            | C::PatternAnalysis
            | C::ComparativeAnalysis
            | C::AbsorptionAnalysis => Some(ComplexityClass::Quadratic),
            C::StructuralAnalysis => Some(ComplexityClass::Exponential),
            _ => None,
        }
    }
}

impl Ontology for AnalyticalMethodsOntology {
    type Cat = AnalyticalMethodsCategory;
    type Qual = IsAutomatable;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SomeMethodsAutomatableSomeNot));
        axioms.push(Box::new(GaloisConnectionIsComponent));
        axioms.push(Box::new(PatternAndAnomalyAreOutputs));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Axiom: the set of analysis methods is split — some admit full automation
/// (Ganter & Wille's Next-Closure et al.) while others require human
/// judgment on what to compare or what to absorb.
pub struct SomeMethodsAutomatableSomeNot;

impl Axiom for SomeMethodsAutomatableSomeNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AnalyticalMethodsConcept as C;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let methods = [
            C::StructuralAnalysis,
            C::PatternAnalysis,
            C::StatisticalAnalysis,
            C::ComparativeAnalysis,
            C::AbsorptionAnalysis,
            C::ClusterAnalysis,
        ];
        let auto = methods
            .iter()
            .filter(|m| IsAutomatable.get(m) == Some(true))
            .count();
        if auto > 0 && auto < methods.len() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SomeMethodsAutomatableSomeNot",
        "the set of analysis methods is split between automatable and human-judgment-requiring",
        "Ganter & Wille (1999) Formal Concept Analysis: Mathematical Foundations §3"
    );
}

pr4xis::register_axiom!(
    SomeMethodsAutomatableSomeNot,
    "Ganter & Wille (1999) Formal Concept Analysis: Mathematical Foundations §3"
);

/// Axiom: the Galois connection is classified as an analysis component
/// (Birkhoff 1940 Ch. V — the antitone pair of derivation operators).
pub struct GaloisConnectionIsComponent;

impl Axiom for GaloisConnectionIsComponent {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let target = AnalyticalMethodsConcept::AnalysisComponent;
        let found = AnalyticalMethodsCategory::morphisms().iter().any(|m| {
            m.kind() == AnalyticalMethodsRelationKind::Subsumption
                && m.source() == AnalyticalMethodsConcept::GaloisConnection
                && m.target() == target
        });
        if found {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GaloisConnectionIsComponent",
        "GaloisConnection is classified as an AnalysisComponent under the Subsumption relation",
        "Birkhoff (1940) Lattice Theory, AMS Colloquium 25, Ch. V"
    );
}

pr4xis::register_axiom!(
    GaloisConnectionIsComponent,
    "Birkhoff (1940) Lattice Theory, AMS Colloquium 25, Ch. V"
);

/// Axiom: Pattern and Anomaly both classify as AnalysisOutput
/// (Tukey 1977 — patterns and outliers are co-output of EDA).
pub struct PatternAndAnomalyAreOutputs;

impl Axiom for PatternAndAnomalyAreOutputs {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let target = AnalyticalMethodsConcept::AnalysisOutput;
        let sub: Vec<_> = AnalyticalMethodsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AnalyticalMethodsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        let ok = sub.contains(&(AnalyticalMethodsConcept::Pattern, target))
            && sub.contains(&(AnalyticalMethodsConcept::Anomaly, target));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PatternAndAnomalyAreOutputs",
        "Pattern and Anomaly both classify as AnalysisOutput",
        "Tukey (1977) Exploratory Data Analysis, Addison-Wesley"
    );
}

pr4xis::register_axiom!(
    PatternAndAnomalyAreOutputs,
    "Tukey (1977) Exploratory Data Analysis, Addison-Wesley"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<AnalyticalMethodsCategory>();
    }

    #[test]
    fn ontology_validates() {
        AnalyticalMethodsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn pipeline_stages_form_causal_chain() {
        // Wille (1982) §2: the 8 pipeline stages form a linear causal chain.
        let causation: Vec<_> = AnalyticalMethodsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AnalyticalMethodsRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        use AnalyticalMethodsConcept as C;
        let pipeline = [
            (C::DataCollection, C::ContextFormation),
            (C::ContextFormation, C::DerivationComputation),
            (C::DerivationComputation, C::LatticeConstruction),
            (C::LatticeConstruction, C::PatternExtraction),
            (C::PatternExtraction, C::AnomalyDetection),
            (C::AnomalyDetection, C::ResultInterpretation),
            (C::ResultInterpretation, C::KnowledgeUpdate),
        ];
        for edge in pipeline {
            assert!(causation.contains(&edge), "missing causation {:?}", edge);
        }
    }

    #[test]
    fn pipeline_transitively_reaches_knowledge_update() {
        // Causation is transitive (OBO-RO `transitive_over`) — the causal
        // closure includes (DataCollection, KnowledgeUpdate).
        let causation: Vec<_> = AnalyticalMethodsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AnalyticalMethodsRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(causation.contains(&(
            AnalyticalMethodsConcept::DataCollection,
            AnalyticalMethodsConcept::KnowledgeUpdate
        )));
    }

    #[test]
    fn methods_subsume_analysis_method() {
        let sub: Vec<_> = AnalyticalMethodsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AnalyticalMethodsRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        use AnalyticalMethodsConcept as C;
        for method in [
            C::StructuralAnalysis,
            C::PatternAnalysis,
            C::StatisticalAnalysis,
            C::ComparativeAnalysis,
            C::AbsorptionAnalysis,
            C::ClusterAnalysis,
        ] {
            assert!(sub.contains(&(method, C::AnalysisMethod)));
        }
    }

    #[test]
    fn structural_opposes_statistical() {
        let opp: Vec<_> = AnalyticalMethodsCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == AnalyticalMethodsRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            AnalyticalMethodsConcept::StructuralAnalysis,
            AnalyticalMethodsConcept::StatisticalAnalysis
        )));
    }

    #[test]
    fn automatability_split_holds() {
        assert!(SomeMethodsAutomatableSomeNot.verify().is_ok());
    }

    #[test]
    fn galois_is_component_holds() {
        assert!(GaloisConnectionIsComponent.verify().is_ok());
    }

    #[test]
    fn pattern_and_anomaly_outputs_holds() {
        assert!(PatternAndAnomalyAreOutputs.verify().is_ok());
    }

    #[test]
    fn complexity_quality_total_on_methods() {
        let q = Complexity;
        use AnalyticalMethodsConcept as C;
        for m in [
            C::StructuralAnalysis,
            C::PatternAnalysis,
            C::StatisticalAnalysis,
            C::ComparativeAnalysis,
            C::AbsorptionAnalysis,
            C::ClusterAnalysis,
        ] {
            assert!(q.get(&m).is_some(), "no complexity for {:?}", m);
        }
    }

    fn arb_concept() -> impl Strategy<Value = AnalyticalMethodsConcept> {
        proptest::sample::select(AnalyticalMethodsConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in AnalyticalMethodsCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in AnalyticalMethodsOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_automatability_total_on_methods(c in arb_concept()) {
            use AnalyticalMethodsConcept as C;
            let v = IsAutomatable.get(&c);
            let is_method = matches!(c,
                C::StructuralAnalysis | C::PatternAnalysis | C::StatisticalAnalysis
                | C::ComparativeAnalysis | C::AbsorptionAnalysis | C::ClusterAnalysis
            );
            prop_assert_eq!(v.is_some(), is_method);
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = AnalyticalMethodsConcept::variants();
            for m in AnalyticalMethodsCategory::morphisms() {
                if m.kind() == AnalyticalMethodsRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = AnalyticalMethodsCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == AnalyticalMethodsRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} -> {:?} but not back", a, b);
            }
        }
    }
}
