//! Doctrine Discovery — meta-ontology for the FCA-driven extraction of
//! doctrinal patterns from a corpus of conditional rules / statute
//! sections / concept-tagged objects.
//!
//! This is a PURE-SCIENCE ontology. The runtime engine lives in
//! [`super::engine`]. The concepts here name the *outputs* of the
//! engine (DoctrineCluster, DoctrineHierarchy, AttributeClosureImplication)
//! and the *pipeline stages* that produce them.
//!
//! # Composition
//!
//! Doctrine Discovery composes:
//!
//! - [`crate::formal::causation::derivation_functor::CausationToDerivation`]
//!   — lifts causal patterns to abductive hypothesis schemata
//!   (Peirce 1903; Lewis 1973).
//! - [`crate::formal::analytical_methods::fca`] — supplies the
//!   formal-context machinery and the NextClosure algorithm
//!   (Wille 1982; Ganter 1984; Ganter & Wille 1999).
//! - [`crate::formal::analytical_methods::classification_fibration`]
//!   — projects each formal concept onto a Linnaean rank from the
//!   Classification ontology (Grothendieck 1971; Jacobs 1999;
//!   Linnaeus 1735).
//! - [`crate::formal::rule_algebra`] — supplies subsumption,
//!   normalization, and conflict detection over the implications
//!   extracted by the engine.
//!
//! The pipeline:
//!
//! ```text
//! Object corpus + Attribute extractor
//!   ──FCA──▶ ConceptLattice
//!   ──Fibration──▶ ConceptLatticeFibration (Linnaean ranks)
//!   ──Singleton-attribute closure──▶ RuleSet<Attribute>
//!   ──Rule Algebra──▶ Canonical basis + Subsumption order
//!   ──CausationToDerivation──▶ Abductive hypothesis schemata
//! ```
//!
//! The number of doctrines the engine emits is not pre-listed — it
//! is whatever the FCA lattice cardinality is, bounded by
//! `2^min(|G|, |M|)` per Ganter & Wille (1999) §2.3 and in practice
//! exponentially smaller for sparse legal contexts.
//!
//! # Literature
//!
//! - **Wille, R. (1982)** "Restructuring Lattice Theory", in *Ordered
//!   Sets*, Reidel — FCA as a knowledge-representation framework.
//! - **Wolff, K. E. (1994)** "An Introduction to Formal Concept
//!   Analysis", *Proceedings of the 4th International Conference on
//!   Conceptual Structures* — FCA applications to legal knowledge.
//! - **Cimiano, P., Hotho, A. & Staab, S. (2005)** "Learning Concept
//!   Hierarchies from Text Corpora using Formal Concept Analysis",
//!   *Journal of Artificial Intelligence Research* 24: 305–339 —
//!   ontology learning from text via FCA.
//! - **Yannouli, M. & Triantafyllou, V. (1998)** "Formal Concept
//!   Analysis as a Way of Representing Legal Knowledge", *Proceedings
//!   of JURIX 1998* — FCA on statute corpora.
//! - **Maedche, A. & Staab, S. (2001)** "Ontology Learning for the
//!   Semantic Web", *IEEE Intelligent Systems* 16(2): 72–79 — the
//!   discovery-engine architecture template.
//! - **Buitelaar, P., Cimiano, P. & Magnini, B. (eds.) (2005)**
//!   *Ontology Learning from Text*, IOS Press — taxonomy of
//!   ontology-from-corpus extraction.
//! - **Peirce, C. S. (1903)** *Harvard Lectures on Pragmatism*,
//!   Lecture VII — abductive inference (the discovered patterns'
//!   hypothesis-generation reading).
//! - **Ganter, B. & Wille, R. (1999)** *Formal Concept Analysis:
//!   Mathematical Foundations*, Springer — Theorem 3 (basic theorem);
//!   §2.3 lattice-size bound.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "DoctrineDiscovery",
    source: "Wille (1982) Ordered Sets; Wolff (1994) ICCS 1994; Cimiano-Hotho-Staab (2005) JAIR 24:305-339; Yannouli & Triantafyllou (1998) JURIX 1998; Maedche & Staab (2001) IEEE IS 16(2); Buitelaar-Cimiano-Magnini (eds.) (2005) Ontology Learning from Text; Peirce (1903) Harvard Lectures; Ganter & Wille (1999) Formal Concept Analysis",

    concepts: [
        // === Inputs ===
        ObjectCorpus,                 // the set of items being analysed
        AttributeExtractor,           // pluggable feature extractor
        FormalContextInput,           // the (G, M, I) triple

        // === Outputs ===
        DoctrineCluster,              // a formal concept's intent — a co-occurring attribute set
        DoctrineHierarchy,            // the Hasse diagram of DoctrineClusters
        AttributeClosureImplication,  // `{m} → closure({m})` for each attribute m
        CanonicalDoctrineBasis,       // subsumption-reduced RuleSet of closures
        DoctrineDiscovery,            // the engine's full output: fibration + RuleSet

        // === Abstract categories ===
        DiscoveryInput,               // ⊇ ObjectCorpus, AttributeExtractor, FormalContextInput
        DiscoveryOutput,              // ⊇ DoctrineCluster, DoctrineHierarchy, AttributeClosureImplication, CanonicalDoctrineBasis, DoctrineDiscovery

        // === Pipeline (Wille 1982 §2; Maedche-Staab 2001) ===
        CorpusLoad,
        AttributeExtraction,
        ContextAssembly,
        LatticeBuild,
        FibrationLift,
        ClosureExtraction,
        BasisNormalization,
        SubsumptionOrdering,
        AbductiveLift,
        OutputAssembly,
    ],

    labels: {
        ObjectCorpus: ("en", "Object corpus",
            "Wille (1982) §2: the set G of objects analysed by FCA. In the doctrine-discovery setting, typically a corpus of statute sections (UscSection), regulations, or annotated clauses."),
        AttributeExtractor: ("en", "Attribute extractor",
            "Maedche & Staab (2001): the pluggable component that maps each object to its attribute set. In legal-doctrine discovery, attributes range from explicit metadata (e.g., RelationType edges incident on a term) to derived features (e.g., presence of burden-shifting language)."),
        FormalContextInput: ("en", "Formal-context input",
            "Wille (1982): the triple (G, M, I) the extractor produces. The runtime counterpart is `formal::analytical_methods::fca::FormalContext<O, A>`."),

        DoctrineCluster: ("en", "Doctrine cluster",
            "Ganter & Wille (1999) Definition 3: a formal concept's intent (B) — the maximal attribute set shared by some subset of the corpus. Each doctrine cluster is a candidate 'doctrine' — a body of co-occurring legal features that act together across the corpus."),
        DoctrineHierarchy: ("en", "Doctrine hierarchy",
            "Davey & Priestley (2002) §1.4: the Hasse diagram of the doctrine clusters under intent inclusion. More-specific doctrines have larger intents."),
        AttributeClosureImplication: ("en", "Attribute-closure implication",
            "Ganter & Wille (1999) §2: the implication `{m} → closure({m})` extracted from a context — the set of attributes invariably co-occurring with attribute m. The non-trivial closures are the candidate doctrinal rules of the corpus."),
        CanonicalDoctrineBasis: ("en", "Canonical doctrine basis",
            "Duquenne & Guigues (1986); Robinson (1965): the subsumption-reduced canonical form of the extracted attribute-closure implications, computed by `rule_algebra::RuleSet::canonical_basis`."),
        DoctrineDiscovery: ("en", "Doctrine discovery",
            "Cimiano-Hotho-Staab (2005): the full output of running FCA + rule-algebra + classification fibration over the corpus. Carries the concept lattice, its Linnaean fibration, and the canonical implication basis."),

        DiscoveryInput: ("en", "Discovery input",
            "Abstract category — ObjectCorpus, AttributeExtractor, FormalContextInput fall under it."),
        DiscoveryOutput: ("en", "Discovery output",
            "Abstract category — DoctrineCluster, DoctrineHierarchy, AttributeClosureImplication, CanonicalDoctrineBasis, DoctrineDiscovery fall under it."),

        CorpusLoad: ("en", "Corpus load",
            "Pipeline stage 1: materialise the object corpus (e.g., load UsCode sections from the registered USLM titles)."),
        AttributeExtraction: ("en", "Attribute extraction",
            "Pipeline stage 2: run the AttributeExtractor over each object to produce its attribute set."),
        ContextAssembly: ("en", "Context assembly",
            "Pipeline stage 3: combine objects and attributes into a FormalContext (G, M, I) per Wille (1982)."),
        LatticeBuild: ("en", "Lattice build",
            "Pipeline stage 4: run Ganter's NextClosure to compute the complete concept lattice (Ganter 1984; Ganter & Wille 1999 §2.1.3)."),
        FibrationLift: ("en", "Fibration lift",
            "Pipeline stage 5: project the lattice onto Linnaean ranks via the Classification fibration (Grothendieck 1971)."),
        ClosureExtraction: ("en", "Closure extraction",
            "Pipeline stage 6: emit `{m} → closure({m})` for every attribute m whose closure is non-trivial (i.e. adds attributes)."),
        BasisNormalization: ("en", "Basis normalization",
            "Pipeline stage 7: normalize the implications via Tarski (1956) closure semantics — sort, dedupe, canonical form. Implemented by `rule_algebra::RuleSet::normalize`."),
        SubsumptionOrdering: ("en", "Subsumption ordering",
            "Pipeline stage 8: compute the Plotkin (1970) θ-subsumption order over the implications. Implemented by `rule_algebra::RuleSet::subsumption_order`."),
        AbductiveLift: ("en", "Abductive lift",
            "Pipeline stage 9: each DoctrineCluster admits an abductive reading (Peirce 1903) — 'the corpus exhibits cluster B because some underlying cause produces all attributes in B simultaneously'. Implemented via `causation::CausationToDerivation`."),
        OutputAssembly: ("en", "Output assembly",
            "Pipeline stage 10: package the lattice, fibration, canonical basis, and abductive schemata as a DoctrineDiscovery."),
    },

    is_a: [
        // Inputs.
        (ObjectCorpus, DiscoveryInput),
        (AttributeExtractor, DiscoveryInput),
        (FormalContextInput, DiscoveryInput),
        // Outputs.
        (DoctrineCluster, DiscoveryOutput),
        (DoctrineHierarchy, DiscoveryOutput),
        (AttributeClosureImplication, DiscoveryOutput),
        (CanonicalDoctrineBasis, DiscoveryOutput),
        (DoctrineDiscovery, DiscoveryOutput),
    ],

    causes: [
        // The discovery pipeline — Wille (1982) §2 + Maedche-Staab (2001)
        // ontology-learning architecture.
        (CorpusLoad, AttributeExtraction),
        (AttributeExtraction, ContextAssembly),
        (ContextAssembly, LatticeBuild),
        (LatticeBuild, FibrationLift),
        (FibrationLift, ClosureExtraction),
        (ClosureExtraction, BasisNormalization),
        (BasisNormalization, SubsumptionOrdering),
        (SubsumptionOrdering, AbductiveLift),
        (AbductiveLift, OutputAssembly),
    ],

}

// =============================================================================
// Domain axioms — structural invariants of the discovery pipeline.
// =============================================================================

fn subsumption_pair_exists(
    child: DoctrineDiscoveryConcept,
    parent: DoctrineDiscoveryConcept,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    DoctrineDiscoveryCategory::morphisms().iter().any(|m| {
        m.source() == child
            && m.target() == parent
            && m.kind() == DoctrineDiscoveryRelationKind::Subsumption
    })
}

fn causation_pair_exists(from: DoctrineDiscoveryConcept, to: DoctrineDiscoveryConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    DoctrineDiscoveryCategory::morphisms().iter().any(|m| {
        m.source() == from
            && m.target() == to
            && m.kind() == DoctrineDiscoveryRelationKind::Causation
    })
}

/// Wille (1982) / Cimiano-Hotho-Staab (2005): the engine's outputs
/// form a coherent classification — DoctrineCluster, DoctrineHierarchy,
/// and AttributeClosureImplication all classify as DiscoveryOutput.
pub struct OutputsClassifyAsDiscoveryOutput;

impl Axiom for OutputsClassifyAsDiscoveryOutput {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DoctrineDiscoveryConcept as C;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let outputs = [
            C::DoctrineCluster,
            C::DoctrineHierarchy,
            C::AttributeClosureImplication,
            C::CanonicalDoctrineBasis,
            C::DoctrineDiscovery,
        ];
        for o in outputs {
            if !subsumption_pair_exists(o, C::DiscoveryOutput) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "OutputsClassifyAsDiscoveryOutput",
        "every named discovery output is-a DiscoveryOutput",
        "Cimiano-Hotho-Staab (2005) JAIR 24:305-339 (FCA ontology-learning outputs)"
    );
}

/// Maedche-Staab (2001): the discovery pipeline is a *linear* causal
/// chain — CorpusLoad eventually causes OutputAssembly via the
/// transitive closure of Causation edges.
pub struct PipelineIsLinearChain;

impl Axiom for PipelineIsLinearChain {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DoctrineDiscoveryConcept as C;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // Transitive closure of canonical Causation kind is computed
        // by the praxis structural-axiom layer. We check the head
        // and tail of the chain are connected via the transitive
        // closure (CorpusLoad → OutputAssembly).
        if causation_pair_exists(C::CorpusLoad, C::OutputAssembly) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PipelineIsLinearChain",
        "CorpusLoad transitively causes OutputAssembly via the discovery pipeline",
        "Maedche & Staab (2001) IEEE Intelligent Systems 16(2):72-79"
    );
}

pr4xis::register_axiom!(
    OutputsClassifyAsDiscoveryOutput,
    "Cimiano-Hotho-Staab (2005) JAIR 24:305-339"
);
pr4xis::register_axiom!(
    PipelineIsLinearChain,
    "Maedche & Staab (2001) IEEE Intelligent Systems 16(2):72-79"
);

/// The scholarly lineage (author-year tradition) that introduces a
/// doctrine-discovery concept. A closed set of named FCA / ontology-learning
/// / abduction / fibration traditions, one variant per source:
///
/// - [`DoctrineLineage::Wille`] — Wille (1982) "Restructuring Lattice Theory",
///   in *Ordered Sets*, Reidel (FCA foundations: G, M, I).
/// - [`DoctrineLineage::GanterWille`] — Ganter & Wille (1999) *Formal Concept
///   Analysis: Mathematical Foundations*, Springer (formal concepts / intents).
/// - [`DoctrineLineage::DuquenneGuigues`] — Duquenne & Guigues (1986) "Familles
///   minimales d'implications informatives", *Math. Sci. Hum.* 95 (canonical
///   implication basis).
/// - [`DoctrineLineage::CimianoHothoStaab`] — Cimiano, Hotho & Staab (2005)
///   *JAIR* 24:305-339 (FCA ontology-learning outputs).
/// - [`DoctrineLineage::Peirce`] — Peirce (1903) *Harvard Lectures on
///   Pragmatism*, Lecture VII (abductive inference).
/// - [`DoctrineLineage::GrothendieckJacobs`] — Grothendieck (1971) SGA 1;
///   Jacobs (1999) *Categorical Logic and Type Theory* (fibrations).
/// - [`DoctrineLineage::MaedcheStaab`] — Maedche & Staab (2001) *IEEE
///   Intelligent Systems* 16(2):72-79 (discovery-engine / pipeline architecture).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctrineLineage {
    /// Wille (1982) — FCA foundations.
    Wille,
    /// Ganter & Wille (1999) — formal concepts / intents.
    GanterWille,
    /// Duquenne & Guigues (1986) — canonical implication basis.
    DuquenneGuigues,
    /// Cimiano, Hotho & Staab (2005) — FCA ontology-learning outputs.
    CimianoHothoStaab,
    /// Peirce (1903) — abductive inference.
    Peirce,
    /// Grothendieck (1971); Jacobs (1999) — fibrations.
    GrothendieckJacobs,
    /// Maedche & Staab (2001) — discovery-engine / pipeline architecture.
    MaedcheStaab,
}

/// Quality: which literature lineage introduces each concept?
#[derive(Debug, Clone)]
pub struct DoctrineDiscoveryLineage;

impl Quality for DoctrineDiscoveryLineage {
    type Individual = DoctrineDiscoveryConcept;
    type Value = DoctrineLineage;

    fn get(&self, c: &DoctrineDiscoveryConcept) -> Option<DoctrineLineage> {
        use DoctrineDiscoveryConcept as C;
        Some(match c {
            C::ObjectCorpus | C::AttributeExtractor | C::FormalContextInput => {
                DoctrineLineage::Wille
            }
            C::DoctrineCluster | C::DoctrineHierarchy => DoctrineLineage::GanterWille,
            C::AttributeClosureImplication | C::CanonicalDoctrineBasis => {
                DoctrineLineage::DuquenneGuigues
            }
            C::DoctrineDiscovery => DoctrineLineage::CimianoHothoStaab,
            C::AbductiveLift => DoctrineLineage::Peirce,
            C::FibrationLift => DoctrineLineage::GrothendieckJacobs,
            _ => DoctrineLineage::MaedcheStaab,
        })
    }
}

impl Ontology for DoctrineDiscoveryOntology {
    type Cat = DoctrineDiscoveryCategory;
    type Qual = DoctrineDiscoveryLineage;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(OutputsClassifyAsDiscoveryOutput));
        axioms.push(Box::new(PipelineIsLinearChain));
        axioms
    }
}

#[cfg(test)]
#[path = "ontology_tests.rs"]
mod tests;
