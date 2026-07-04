//! Functor Synthesizer — meta-ontology for the bootstrapping cycle
//! by which praxis turns a
//! [`crate::formal::doctrine_discovery::DoctrineDiscovery`] into a
//! runtime functor `O → cluster index`. The synthesized functors
//! re-enter as attribute extractors for the next discovery cycle,
//! growing the knowledge graph monotonically.
//!
//! # The bootstrapping cycle
//!
//! ```text
//! Source bytes ──load──▶ Object corpus
//!                          │
//!                          ▼
//!                   FormalContext (G, M, I)
//!                          │
//!                          ▼
//!                      discover()
//!                          │
//!                          ▼
//!                  DoctrineDiscovery {
//!                      fibration,
//!                      basis,
//!                      subsumption_order,
//!                  }
//!                          │
//!                          ▼
//!                     synthesize()
//!                          │
//!                          ▼
//!                  SynthesizedFunctor {
//!                      object_map: O → cluster_idx,
//!                      morphism_map: edge → edge,
//!                      laws_verified: bool,
//!                  }
//!                          │
//!                          ├──▶ Compose with existing
//!                          │    praxis functors
//!                          │    (Causation → Derivation,
//!                          │     XsdOntology → English,
//!                          │     LMF → English, …)
//!                          │
//!                          ▼
//!                  Feed the synthesized functor's image back into
//!                  the AttributeExtractor for the next discovery
//!                  round, until the cycle reaches a fix-point.
//! ```
//!
//! The number of cycles is data-determined; convergence is detected
//! by comparing successive `SynthesizedFunctor`s for equality
//! modulo identity.
//!
//! # Composition
//!
//! This module composes:
//!
//! - [`crate::formal::doctrine_discovery::engine::DoctrineDiscovery`]
//!   — the input to synthesis.
//! - [`crate::formal::analytical_methods::fca::FormalConcept`] — each
//!   discovered cluster.
//! - [`crate::formal::rule_algebra::Implication`] — the synthesized
//!   functor's morphism image when projected through the basis.
//! - [`pr4xis::category::Functor`] — the type-level functor contract
//!   that the synthesizer's runtime output is a data-layer witness
//!   for.
//!
//! The module declares the bootstrapping vocabulary and the axioms a
//! SynthesizedFunctor must satisfy (Mac Lane §I.3 functor laws).
//!
//! # Literature
//!
//! - **Mac Lane, S. (1971)** *Categories for the Working Mathematician*,
//!   Springer GTM 5 — §I.3 functor laws (identity-preserving,
//!   composition-preserving); §IV.1 adjunctions; §IV.4 equivalence
//!   of categories.
//! - **Lambek, J. & Scott, P. J. (1986)** *Introduction to Higher-
//!   Order Categorical Logic*, Cambridge UP — the deductive-system
//!   view: functors as theory translations.
//! - **Spivak, D. I. (2013)** "Functorial Data Migration",
//!   *Information and Computation* 217: 31–51 — functors between
//!   schema categories.
//! - **Goguen, J. A. & Burstall, R. M. (1992)** "Institutions:
//!   Abstract Model Theory for Specification and Programming",
//!   *JACM* 39(1): 95–146 — institutions and theory morphisms; the
//!   classical bootstrapping framework for translating between
//!   logical systems.
//! - **Maedche, A. & Staab, S. (2001)** "Ontology Learning for the
//!   Semantic Web", *IEEE Intelligent Systems* 16(2): 72–79 — the
//!   refinement / bootstrapping loop in ontology learning.
//! - **Cimiano, P. (2006)** *Ontology Learning and Population from
//!   Text: Algorithms, Evaluation and Applications*, Springer — the
//!   incremental ontology refinement cycle that this module
//!   formalises.
//! - **Wille, R. (1982)** "Restructuring Lattice Theory", *Ordered
//!   Sets* — concept lattices as canonical-form targets for functor
//!   synthesis.
//! - **Awodey, S. (2010)** *Category Theory*, 2nd ed., Oxford UP —
//!   §7.5 natural transformations; the witnessing structure that lets
//!   us compose synthesized functors with hand-written ones.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "FunctorSynthesizer",
    source: "Mac Lane (1971) Categories for the Working Mathematician §I.3, §IV.1; Lambek & Scott (1986) Higher-Order Categorical Logic; Spivak (2013) Information and Computation 217:31-51; Goguen & Burstall (1992) JACM 39(1):95-146; Maedche & Staab (2001) IEEE Intelligent Systems 16(2):72-79; Cimiano (2006) Ontology Learning and Population from Text; Wille (1982) Ordered Sets; Awodey (2010) Category Theory §7.5",

    concepts: [
        // === Inputs ===
        DiscoveryInput,            // a DoctrineDiscovery from the doctrine-discovery engine
        SynthesisRequest,          // the (source, target) pair to synthesize a functor over

        // === Synthesized artefacts ===
        SynthesizedFunctor,        // runtime O → cluster_idx functor
        ObjectMapping,             // the object-level component
        MorphismMapping,           // the morphism-level component
        ClusterTargetCategory,     // the runtime "category of doctrine clusters"

        // === Verification ===
        FunctorLaw,                // Mac Lane §I.3 identity + composition
        IdentityPreservation,
        CompositionPreservation,

        // === Composition + bootstrap ===
        FunctorComposition,        // synth ∘ existing
        BootstrappingCycle,        // the iterative refinement loop
        ConvergenceWitness,        // the fix-point reached by repeated cycles

        // === Abstract categories ===
        SynthesisInput,            // ⊇ DiscoveryInput, SynthesisRequest
        SynthesizedArtefact,       // ⊇ SynthesizedFunctor, ObjectMapping, MorphismMapping, ClusterTargetCategory
        VerificationArtefact,      // ⊇ FunctorLaw, IdentityPreservation, CompositionPreservation
        BootstrapArtefact,         // ⊇ FunctorComposition, BootstrappingCycle, ConvergenceWitness

        // === Pipeline ===
        IngestDiscovery,
        BuildObjectMapping,
        BuildMorphismMapping,
        VerifyFunctorLaws,
        RegisterFunctor,
        AttemptComposition,
        IterateCycle,
        DetectConvergence,
    ],

    labels: {
        DiscoveryInput: ("en", "Discovery input",
            "Maedche & Staab (2001) §3: a DoctrineDiscovery produced by the doctrine-discovery engine — the substrate the synthesizer reads to derive a functor."),
        SynthesisRequest: ("en", "Synthesis request",
            "A typed pair (source-ontology, target-discovery) specifying which functor the user wants synthesized. In the bootstrap loop, the request is implicit — every fresh discovery produces a candidate functor."),

        SynthesizedFunctor: ("en", "Synthesized functor",
            "Mac Lane (1971) §I.3: the runtime functor O → cluster_idx produced from a DoctrineDiscovery. Object map sends each source object to its assigned doctrine cluster; morphism map collapses intra-cluster edges to identity and preserves inter-cluster covers."),
        ObjectMapping: ("en", "Object mapping",
            "Mac Lane §I.3: the object-level component F_0: Ob(C) → Ob(D) of the synthesized functor."),
        MorphismMapping: ("en", "Morphism mapping",
            "Mac Lane §I.3: the morphism-level component F_1: Mor(C) → Mor(D), required to preserve identity and composition."),
        ClusterTargetCategory: ("en", "Cluster target category",
            "The runtime category whose objects are doctrine clusters (from the concept-lattice fibration) and whose morphisms are the Hasse-diagram cover edges. Acts as the synthesized functor's codomain."),

        FunctorLaw: ("en", "Functor law",
            "Mac Lane (1971) §I.3 Definition 1: a functor preserves identities and composition. Two laws — IdentityPreservation and CompositionPreservation — must both hold."),
        IdentityPreservation: ("en", "Identity preservation",
            "Mac Lane §I.3 axiom 1: F(id_A) = id_{F(A)} for every object A."),
        CompositionPreservation: ("en", "Composition preservation",
            "Mac Lane §I.3 axiom 2: F(g ∘ f) = F(g) ∘ F(f) for every composable pair (f, g)."),

        FunctorComposition: ("en", "Functor composition",
            "Mac Lane §I.3 Proposition 1: functors compose. A synthesized SourceA → Cluster functor can compose with a hand-written Cluster → English projection to yield SourceA → English."),
        BootstrappingCycle: ("en", "Bootstrapping cycle",
            "Goguen & Burstall (1992) institutions / Cimiano (2006) §6: the iterative refinement loop. Each cycle consumes the previous cycle's synthesized functors as additional attribute extractors, generating richer DoctrineDiscovery outputs."),
        ConvergenceWitness: ("en", "Convergence witness",
            "A SynthesizedFunctor whose object_map agrees with the previous cycle's, modulo identity — i.e. the cycle reached a fix-point and no new structure was discovered."),

        SynthesisInput: ("en", "Synthesis input",
            "Abstract category — DiscoveryInput and SynthesisRequest classify as inputs to the synthesizer."),
        SynthesizedArtefact: ("en", "Synthesized artefact",
            "Abstract category — SynthesizedFunctor, ObjectMapping, MorphismMapping, ClusterTargetCategory classify as outputs of synthesis."),
        VerificationArtefact: ("en", "Verification artefact",
            "Abstract category — FunctorLaw and its two axioms classify as verification artefacts."),
        BootstrapArtefact: ("en", "Bootstrap artefact",
            "Abstract category — FunctorComposition, BootstrappingCycle, ConvergenceWitness classify as bootstrap artefacts."),

        IngestDiscovery: ("en", "Ingest discovery",
            "Pipeline stage 1: receive a DoctrineDiscovery from the discovery engine."),
        BuildObjectMapping: ("en", "Build object mapping",
            "Pipeline stage 2: for each object g ∈ G, determine the smallest cluster whose extent contains g; map g to that cluster's index."),
        BuildMorphismMapping: ("en", "Build morphism mapping",
            "Pipeline stage 3: define F_1 on the discrete category of source-object identity morphisms — every id_g maps to id_{F(g)}. For richer source categories (with non-identity morphisms), the mapping factors through the cluster Hasse diagram."),
        VerifyFunctorLaws: ("en", "Verify functor laws",
            "Pipeline stage 4: confirm IdentityPreservation and CompositionPreservation on the synthesized functor's image."),
        RegisterFunctor: ("en", "Register functor",
            "Pipeline stage 5: the verified SynthesizedFunctor is added to the praxis runtime registry — composes with existing functors per Mac Lane §I.3 Proposition 1."),
        AttemptComposition: ("en", "Attempt composition",
            "Pipeline stage 6: try composing the synthesized functor with every existing praxis functor whose source matches the synthesized functor's target. Successful compositions become new edges in the knowledge graph."),
        IterateCycle: ("en", "Iterate cycle",
            "Pipeline stage 7: feed the synthesized functor's image back into the next DoctrineDiscovery round as an additional attribute extractor."),
        DetectConvergence: ("en", "Detect convergence",
            "Pipeline stage 8: compare the new cycle's synthesized functor against the previous cycle's. If they agree modulo identity, emit a ConvergenceWitness; otherwise loop."),
    },

    is_a: [
        // Inputs.
        (DiscoveryInput, SynthesisInput),
        (SynthesisRequest, SynthesisInput),
        // Artefacts.
        (SynthesizedFunctor, SynthesizedArtefact),
        (ObjectMapping, SynthesizedArtefact),
        (MorphismMapping, SynthesizedArtefact),
        (ClusterTargetCategory, SynthesizedArtefact),
        // Verification.
        (FunctorLaw, VerificationArtefact),
        (IdentityPreservation, VerificationArtefact),
        (CompositionPreservation, VerificationArtefact),
        (IdentityPreservation, FunctorLaw),
        (CompositionPreservation, FunctorLaw),
        // Bootstrap.
        (FunctorComposition, BootstrapArtefact),
        (BootstrappingCycle, BootstrapArtefact),
        (ConvergenceWitness, BootstrapArtefact),
    ],

    causes: [
        // Pipeline.
        (IngestDiscovery, BuildObjectMapping),
        (BuildObjectMapping, BuildMorphismMapping),
        (BuildMorphismMapping, VerifyFunctorLaws),
        (VerifyFunctorLaws, RegisterFunctor),
        (RegisterFunctor, AttemptComposition),
        (AttemptComposition, IterateCycle),
        (IterateCycle, DetectConvergence),
    ],

    opposes: [
        // Identity and composition are the two complementary aspects
        // of the functor-law verification — neither alone suffices.
        (IdentityPreservation, CompositionPreservation),
        (CompositionPreservation, IdentityPreservation),
    ],
}

// =============================================================================
// Domain axioms — structural invariants of the synthesizer / bootstrap.
// =============================================================================

fn subsumption_pair_exists(
    child: FunctorSynthesizerConcept,
    parent: FunctorSynthesizerConcept,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    FunctorSynthesizerCategory::morphisms().iter().any(|m| {
        m.source() == child
            && m.target() == parent
            && m.kind() == FunctorSynthesizerRelationKind::Subsumption
    })
}

fn causation_pair_exists(from: FunctorSynthesizerConcept, to: FunctorSynthesizerConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    FunctorSynthesizerCategory::morphisms().iter().any(|m| {
        m.source() == from
            && m.target() == to
            && m.kind() == FunctorSynthesizerRelationKind::Causation
    })
}

/// Mac Lane (1971) §I.3: every functor must satisfy two laws —
/// identity preservation and composition preservation. Both are
/// classified as `FunctorLaw` in this ontology.
pub struct FunctorLawHasBothAxioms;

impl Axiom for FunctorLawHasBothAxioms {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = subsumption_pair_exists(
            FunctorSynthesizerConcept::IdentityPreservation,
            FunctorSynthesizerConcept::FunctorLaw,
        ) && subsumption_pair_exists(
            FunctorSynthesizerConcept::CompositionPreservation,
            FunctorSynthesizerConcept::FunctorLaw,
        );
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FunctorLawHasBothAxioms",
        "FunctorLaw is constituted by both IdentityPreservation and CompositionPreservation",
        "Mac Lane (1971) Categories for the Working Mathematician §I.3 Definition 1"
    );
}

/// Goguen & Burstall (1992) / Cimiano (2006): the bootstrap pipeline
/// terminates with a ConvergenceWitness — the head IngestDiscovery
/// transitively causes the tail DetectConvergence.
pub struct PipelineReachesConvergence;

impl Axiom for PipelineReachesConvergence {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if causation_pair_exists(
            FunctorSynthesizerConcept::IngestDiscovery,
            FunctorSynthesizerConcept::DetectConvergence,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PipelineReachesConvergence",
        "the bootstrap pipeline causally connects IngestDiscovery to DetectConvergence (transitively)",
        "Goguen & Burstall (1992) JACM 39(1):95-146 (institution morphisms terminate); Cimiano (2006) Ontology Learning §6 (refinement convergence)"
    );
}

/// Mac Lane (1971) §I.3: identity preservation and composition
/// preservation are complementary — both are required and neither
/// alone suffices.
pub struct IdentityAndCompositionAreComplementary;

impl Axiom for IdentityAndCompositionAreComplementary {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let opp_pair = (
            FunctorSynthesizerConcept::IdentityPreservation,
            FunctorSynthesizerConcept::CompositionPreservation,
        );
        let opposed = FunctorSynthesizerCategory::morphisms().iter().any(|m| {
            m.kind() == FunctorSynthesizerRelationKind::Opposition
                && m.source() == opp_pair.0
                && m.target() == opp_pair.1
        });
        if opposed {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "IdentityAndCompositionAreComplementary",
        "IdentityPreservation and CompositionPreservation are complementary FunctorLaw aspects",
        "Mac Lane (1971) Categories for the Working Mathematician §I.3"
    );
}

pr4xis::register_axiom!(
    FunctorLawHasBothAxioms,
    "Mac Lane (1971) Categories for the Working Mathematician §I.3 Definition 1"
);
pr4xis::register_axiom!(
    PipelineReachesConvergence,
    "Goguen & Burstall (1992) JACM 39(1):95-146; Cimiano (2006) Ontology Learning §6"
);
pr4xis::register_axiom!(
    IdentityAndCompositionAreComplementary,
    "Mac Lane (1971) Categories for the Working Mathematician §I.3"
);

/// The scholarly lineage that introduces each synthesizer concept — a
/// closed set of the named traditions this module composes.
///
/// A closed taxonomy drawn from the module `source:` bibliography: the
/// functor-law core is Mac Lane (1971); the bootstrap/refinement loop is
/// the Goguen–Burstall–Cimiano institution-morphism lineage; the
/// concept-lattice target category is Wille (1982); the discovery-input
/// substrate is Maedche & Staab (2001). Concepts that belong to this
/// module's own pipeline/abstract-category scaffolding, rather than to an
/// external tradition, classify as `Structural`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteratureLineage {
    /// Mac Lane (1971) *Categories for the Working Mathematician* §I.3 —
    /// the functor laws (identity/composition preservation) and their
    /// artefacts.
    MacLane,
    /// Goguen & Burstall (1992) *JACM* 39(1):95–146 (institution
    /// morphisms) + Cimiano (2006) *Ontology Learning and Population from
    /// Text* §6 — the bootstrapping / refinement-convergence loop.
    GoguenBurstallCimiano,
    /// Wille (1982) "Restructuring Lattice Theory", *Ordered Sets* — the
    /// concept-lattice fibration that supplies the cluster target category.
    Wille,
    /// Maedche & Staab (2001) *IEEE Intelligent Systems* 16(2):72–79 —
    /// the ontology-learning discovery input the synthesizer reads.
    MaedcheStaab,
    /// This module's own pipeline stages and abstract categories — no
    /// external tradition; structural scaffolding.
    Structural,
}

/// Quality: which literature [`LiteratureLineage`] introduces each concept?
#[derive(Debug, Clone)]
pub struct FunctorSynthesizerLineage;

impl Quality for FunctorSynthesizerLineage {
    type Individual = FunctorSynthesizerConcept;
    type Value = LiteratureLineage;

    fn get(&self, c: &FunctorSynthesizerConcept) -> Option<LiteratureLineage> {
        use FunctorSynthesizerConcept as C;
        use LiteratureLineage as L;
        Some(match c {
            C::SynthesizedFunctor
            | C::ObjectMapping
            | C::MorphismMapping
            | C::FunctorLaw
            | C::IdentityPreservation
            | C::CompositionPreservation
            | C::FunctorComposition => L::MacLane,
            C::BootstrappingCycle | C::ConvergenceWitness => L::GoguenBurstallCimiano,
            C::ClusterTargetCategory => L::Wille,
            C::DiscoveryInput => L::MaedcheStaab,
            _ => L::Structural,
        })
    }
}

impl Ontology for FunctorSynthesizerOntology {
    type Cat = FunctorSynthesizerCategory;
    type Qual = FunctorSynthesizerLineage;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(FunctorLawHasBothAxioms));
        axioms.push(Box::new(PipelineReachesConvergence));
        axioms.push(Box::new(IdentityAndCompositionAreComplementary));
        axioms
    }
}

#[cfg(test)]
#[path = "ontology_tests.rs"]
mod tests;
