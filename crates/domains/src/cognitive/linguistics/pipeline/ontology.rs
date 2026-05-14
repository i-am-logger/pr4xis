//! Parse ⊣ Generate — the bidirectional language pipeline as adjunction.
//!
//! The central theorem: parsing and generation are adjoint functors
//! over the same grammar. Parse is the left adjoint (surface → meaning),
//! Generate is the right adjoint (meaning → surface).
//!
//! de Groote ACG (2001): a lexicon IS a homomorphism L: Σ_abstract → Σ_object.
//! Parsing = finding the pre-image of L (hard: proof search).
//! Generation = applying L (easy: beta-reduction).
//! The SAME grammar does both — the direction is the adjunction.
//!
//! Coecke, Sadrzadeh & Clark DisCoCat (2010): meaning IS a strong
//! monoidal functor F: Grammar → Semantics.
//!
//! Lambek & Scott (1986): parsing IS proof search in the type logic.
//!
//! Levelt (1989): generation follows Conceptualizer → Formulator → Articulator.
//!
//! Di Lavore & de Felice (2022): monoidal streams for incremental processing.
//!
//! Source: de Groote (2001); Lambek (1958); Lambek & Scott (1986);
//!         Coecke, Sadrzadeh & Clark (2010); Levelt (1989);
//!         Di Lavore & de Felice (2022)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Pipeline",
    source: "de Groote (2001); Lambek (1958); Coecke et al. (2010); Levelt (1989); Di Lavore & de Felice (2022)",

    concepts: [
        // The adjunction (Mac Lane 1971 Ch. IV)
        Parse,
        Generate,
        Unit,
        Counit,
        // Pipeline stages (Levelt 1989 + de Groote 2001)
        SurfaceForm,
        SyntacticStructure,
        SemanticRepresentation,
        LexiconHomomorphism,
        ProofTerm,
        MeaningFunctor,
        // Streaming / incremental (Di Lavore & de Felice 2022)
        PartialResult,
        Stream,
    ],

    labels: {
        Parse: ("en", "Parse", "The Parse functor — left adjoint. Surface → Meaning. Proof search in the type logic (Lambek 1958)."),
        Generate: ("en", "Generate", "The Generate functor — right adjoint. Meaning → Surface. Beta-reduction of the lexicon homomorphism (de Groote 2001)."),
        Unit: ("en", "Unit", "η: Id → G∘F — the unit of the adjunction. What survives the round trip: parse then generate."),
        Counit: ("en", "Counit", "ε: F∘G → Id — the counit of the adjunction. What survives generating then parsing back."),
        SurfaceForm: ("en", "Surface form", "Text as it appears. NIF Context/Word layer. The object vocabulary (de Groote)."),
        SyntacticStructure: ("en", "Syntactic structure", "The proof term / parse tree. Lambek type assignment + reduction."),
        SemanticRepresentation: ("en", "Semantic representation", "The meaning. DisCoCat functor image. The abstract vocabulary (de Groote)."),
        LexiconHomomorphism: ("en", "Lexicon homomorphism", "L: Σ_abstract → Σ_object (de Groote 2001). Bridges abstract ↔ object."),
        ProofTerm: ("en", "Proof term", "A proof term in the type logic — parsing IS proof search (Lambek & Scott 1986)."),
        MeaningFunctor: ("en", "Meaning functor", "DisCoCat F: Grammar → Semantics. Strong monoidal: preserves composition."),
        PartialResult: ("en", "Partial result", "A partial result in the pipeline — not yet complete. Comonadic: carries context."),
        Stream: ("en", "Stream", "The stream of partial results over time. Monoidal stream: composition of incremental steps."),
    },

    is_a: [
        (Unit, Parse),
        (Counit, Generate),
        (ProofTerm, SyntacticStructure),
        (PartialResult, Stream),
    ],

    has_a: [
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

    causes: [
        // Parse direction: surface causes syntactic, syntactic causes semantic
        (SurfaceForm, SyntacticStructure),
        (SyntacticStructure, SemanticRepresentation),
        // LexiconHomomorphism enables proof construction
        (LexiconHomomorphism, ProofTerm),
    ],

    opposes: [
        // Parse ⊣ Generate — the adjunction itself
        (Parse, Generate),
        // Surface vs Meaning — the two endpoints
        (SurfaceForm, SemanticRepresentation),
    ],
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
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts: Vec<_> = PipelineCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == PipelineRelationKind::Parthood)
            .collect();
        let parse_has = parts.iter().any(|m| {
            m.source() == PipelineConcept::Parse
                && m.target() == PipelineConcept::LexiconHomomorphism
        });
        let gen_has = parts.iter().any(|m| {
            m.source() == PipelineConcept::Generate
                && m.target() == PipelineConcept::LexiconHomomorphism
        });
        if parse_has && gen_has {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SharedLexicon",
        "Parse and Generate share the LexiconHomomorphism (de Groote 2001: same grammar)",
        "de Groote (2001) Towards Abstract Categorial Grammars, ACL 2001"
    );
}
pr4xis::register_axiom!(
    SharedLexicon,
    "de Groote (2001) Towards Abstract Categorial Grammars, ACL 2001"
);

/// Parse and Generate are opposed (adjunction).
#[derive(Debug)]
pub struct ParseGenerateAdjoint;

impl Axiom for ParseGenerateAdjoint {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if PipelineCategory::morphisms().iter().any(|m| {
            m.kind() == PipelineRelationKind::Opposition
                && m.source() == PipelineConcept::Parse
                && m.target() == PipelineConcept::Generate
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ParseGenerateAdjoint",
        "Parse ⊣ Generate: left and right adjoints are opposed",
        "de Groote (2001) Towards Abstract Categorial Grammars; Lambek & Scott (1986) Introduction to Higher Order Categorical Logic"
    );
}
pr4xis::register_axiom!(
    ParseGenerateAdjoint,
    "de Groote (2001) Towards Abstract Categorial Grammars; Lambek & Scott (1986) Introduction to Higher Order Categorical Logic"
);

/// Surface and Meaning are opposed endpoints.
#[derive(Debug)]
pub struct SurfaceMeaningOpposed;

impl Axiom for SurfaceMeaningOpposed {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if PipelineCategory::morphisms().iter().any(|m| {
            m.kind() == PipelineRelationKind::Opposition
                && m.source() == PipelineConcept::SurfaceForm
                && m.target() == PipelineConcept::SemanticRepresentation
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SurfaceMeaningOpposed",
        "SurfaceForm and SemanticRepresentation are opposed endpoints",
        "Levelt (1989) Speaking: From Intention to Articulation; de Groote (2001) Towards Abstract Categorial Grammars"
    );
}
pr4xis::register_axiom!(
    SurfaceMeaningOpposed,
    "Levelt (1989) Speaking: From Intention to Articulation; de Groote (2001) Towards Abstract Categorial Grammars"
);

impl Ontology for PipelineOntology {
    type Cat = PipelineCategory;
    type Qual = IsPipelineStage;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SharedLexicon));
        axioms.push(Box::new(ParseGenerateAdjoint));
        axioms.push(Box::new(SurfaceMeaningOpposed));
        axioms
    }
}
