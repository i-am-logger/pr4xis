// Parse ⊣ Generate — the bidirectional language pipeline as adjunction.
//
// The central theorem: parsing and generation are adjoint functors
// over the same grammar. Parse is the left adjoint (surface → meaning),
// Generate is the right adjoint (meaning → surface).
//
// de Groote ACG (2001): a lexicon IS a homomorphism L: Σ_abstract → Σ_object.
// Parsing = finding the pre-image of L (hard: proof search).
// Generation = applying L (easy: beta-reduction).
// The SAME grammar does both — the direction is the adjunction.
//
// Coecke, Sadrzadeh & Clark DisCoCat (2010): meaning IS a strong
// monoidal functor F: Grammar → Semantics. The functor maps
// grammatical composition to semantic composition.
//
// Lambek & Scott (1986): parsing IS proof search in the type logic.
// A proof term IS the parse tree. Generation = proof normalization.
//
// Levelt (1989): generation follows Conceptualizer → Formulator →
// Articulator. The parse direction reverses this: Articulator →
// Formulator → Conceptualizer (surface → syntax → semantics).
//
// Di Lavore & de Felice (2022): monoidal streams for incremental
// processing — the pipeline produces partial results over time via
// coKleisli arrows of a comonad.
//
// Source: de Groote (2001); Lambek (1958); Lambek & Scott (1986);
//         Coecke, Sadrzadeh & Clark (2010); Levelt (1989);
//         Di Lavore & de Felice (2022)

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Parse ⊣ Generate pipeline ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PipelineConcept {
    // === The adjunction (Mac Lane 1971 Ch. IV) ===
    /// The Parse functor — left adjoint. Surface → Meaning.
    /// Proof search in the type logic (Lambek 1958).
    Parse,

    /// The Generate functor — right adjoint. Meaning → Surface.
    /// Beta-reduction of the lexicon homomorphism (de Groote 2001).
    Generate,

    /// η: Id → G∘F — the unit of the adjunction.
    /// What survives the round trip: parse then generate.
    /// A meaning that can be expressed (not all meanings can).
    Unit,

    /// ε: F∘G → Id — the counit of the adjunction.
    /// What survives generating then parsing back.
    /// A surface form that parses unambiguously.
    Counit,

    // === Pipeline stages (Levelt 1989 + de Groote 2001) ===
    /// The surface form — text as it appears.
    /// NIF Context/Word layer. The object vocabulary (de Groote).
    SurfaceForm,

    /// Syntactic analysis — the proof term / parse tree.
    /// Lambek type assignment + reduction.
    SyntacticStructure,

    /// Semantic representation — the meaning.
    /// DisCoCat functor image. The abstract vocabulary (de Groote).
    SemanticRepresentation,

    /// The lexicon homomorphism — bridges abstract ↔ object.
    /// de Groote (2001): L: Σ_abstract → Σ_object.
    /// THIS is what makes parse and generate use the same grammar.
    LexiconHomomorphism,

    /// A proof term in the type logic — parsing IS proof search.
    /// Lambek & Scott (1986): proofs as programs.
    ProofTerm,

    /// The meaning functor — DisCoCat F: Grammar → Semantics.
    /// Strong monoidal: preserves composition.
    MeaningFunctor,

    // === Streaming / incremental (Di Lavore & de Felice 2022) ===
    /// A partial result in the pipeline — not yet complete.
    /// Comonadic: carries context of what came before.
    PartialResult,

    /// The stream of partial results over time.
    /// Monoidal stream: composition of incremental steps.
    Stream,
}

define_ontology! {
    /// Parse ⊣ Generate — the language pipeline adjunction.
    pub PipelineOntology for PipelineCategory {
        concepts: PipelineConcept,
        relation: PipelineRelation,

        being: Process,
        source: "de Groote (2001); Lambek (1958); Coecke et al. (2010); Levelt (1989); Di Lavore & de Felice (2022)",

        is_a: PipelineTaxonomy [
            // Unit and Counit are structural parts of the adjunction
            (Unit, Parse),
            (Counit, Generate),
            // ProofTerm is a SyntacticStructure
            (ProofTerm, SyntacticStructure),
            // PartialResult is part of a Stream
            (PartialResult, Stream),
        ],

        has_a: PipelineMereology [
            // Parse has stages: SurfaceForm → SyntacticStructure → SemanticRepresentation
            (Parse, SurfaceForm),
            (Parse, SyntacticStructure),
            (Parse, SemanticRepresentation),
            // Generate has stages in reverse
            (Generate, SemanticRepresentation),
            (Generate, SyntacticStructure),
            (Generate, SurfaceForm),
            // The lexicon homomorphism is shared
            (Parse, LexiconHomomorphism),
            (Generate, LexiconHomomorphism),
            // The meaning functor connects grammar to semantics
            (Parse, MeaningFunctor),
            // Stream contains partial results
            (Stream, PartialResult),
        ],

        causes: PipelineCausation for PipelineConcept [
            // Parse direction: surface causes syntactic, syntactic causes semantic
            // (Generation is the REVERSE adjoint, not a separate causal chain)
            (SurfaceForm, SyntacticStructure),
            (SyntacticStructure, SemanticRepresentation),
            // LexiconHomomorphism enables proof construction
            (LexiconHomomorphism, ProofTerm),
        ],

        opposes: PipelineOpposition [
            // Parse ⊣ Generate — the adjunction itself
            (Parse, Generate),
            // Surface vs Meaning — the two endpoints
            (SurfaceForm, SemanticRepresentation),
        ],
    }
}

/// Whether a concept is a pipeline stage vs structural/streaming.
#[derive(Debug, Clone)]
pub struct IsPipelineStage;

impl Quality for IsPipelineStage {
    type Individual = PipelineConcept;
    type Value = bool;

    fn get(&self, individual: &PipelineConcept) -> Option<bool> {
        Some(matches!(
            individual,
            PipelineConcept::SurfaceForm
                | PipelineConcept::SyntacticStructure
                | PipelineConcept::SemanticRepresentation
        ))
    }
}

/// Parse and Generate share the LexiconHomomorphism (de Groote 2001).
#[derive(Debug)]
pub struct SharedLexicon;

impl Axiom for SharedLexicon {
    fn description(&self) -> &str {
        "Parse and Generate share the LexiconHomomorphism (de Groote 2001: same grammar)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::mereology::MereologyDef;
        let parts = PipelineMereology::relations();
        let parse_has = parts.iter().any(|(w, p)| {
            *w == PipelineConcept::Parse && *p == PipelineConcept::LexiconHomomorphism
        });
        let gen_has = parts.iter().any(|(w, p)| {
            *w == PipelineConcept::Generate && *p == PipelineConcept::LexiconHomomorphism
        });
        parse_has && gen_has
    }
}

/// Parse and Generate are opposed (adjunction).
#[derive(Debug)]
pub struct ParseGenerateAdjoint;

impl Axiom for ParseGenerateAdjoint {
    fn description(&self) -> &str {
        "Parse ⊣ Generate: left and right adjoints are opposed"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        PipelineOpposition::pairs()
            .iter()
            .any(|(a, b)| *a == PipelineConcept::Parse && *b == PipelineConcept::Generate)
    }
}

/// Surface and Meaning are opposed endpoints.
#[derive(Debug)]
pub struct SurfaceMeaningOpposed;

impl Axiom for SurfaceMeaningOpposed {
    fn description(&self) -> &str {
        "SurfaceForm and SemanticRepresentation are opposed endpoints"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::opposition::OppositionDef;
        PipelineOpposition::pairs().iter().any(|(a, b)| {
            *a == PipelineConcept::SurfaceForm && *b == PipelineConcept::SemanticRepresentation
        })
    }
}

impl Ontology for PipelineOntology {
    type Cat = PipelineCategory;
    type Qual = IsPipelineStage;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        PipelineOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(SharedLexicon),
            Box::new(ParseGenerateAdjoint),
            Box::new(SurfaceMeaningOpposed),
        ]
    }
}
