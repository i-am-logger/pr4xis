//! Speech production ontology — the generation pipeline.
//!
//! This is the RIGHT ADJOINT of the parsing pipeline.
//! Parse: Text → Syntax → Semantics (left adjoint, F)
//! Generate: Semantics → Syntax → Text (right adjoint, G)
//! Together: Parse ⊣ Generate (adjunction)
//!
//! The pipeline follows Levelt's speech production model (1989):
//!   Conceptualizer → PreverbalMessage → Formulator → SurfaceForm
//!
//! Enriched by:
//! - Reiter & Dale (2000): content determination → document planning → microplanning → realization
//! - Appelt (1985): speech acts as plan operators with preconditions and effects
//! - Pogodalla (2000): generation in Lambek calculus = proof search with semantic constraint fixed
//! - de Groote (2001): ACG generation = beta-reduction of lexicon homomorphism (trivial!)
//! - McKeown (1985): rhetorical schemata for content organization

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pr4xis::ontology! {
    name: "Production",
    source: "Levelt (1989); de Groote (2001); Reiter & Dale (2000); Appelt (1985); McKeown (1985)",
    being: Process,

    concepts: [
        CommunicativeGoal,
        PreverbalMessage,
        SentencePlan,
        SurfaceForm,
        Monitor,
        Message,
        DocumentPlan,
        LexicalChoice,
    ],

    labels: {
        CommunicativeGoal: ("en", "Communicative goal", "What the system wants to achieve by speaking. Appelt (1985): a goal in the hearer's mental state."),
        PreverbalMessage: ("en", "Preverbal message", "The pre-linguistic representation of what to say. Levelt (1989): output of the Conceptualizer."),
        SentencePlan: ("en", "Sentence plan", "The grammatical structure being built. Levelt (1989): output of the Formulator's grammatical encoder."),
        SurfaceForm: ("en", "Surface form", "The final realized utterance. de Groote (2001): L(abstract_term) = beta-reduce = surface string."),
        Monitor: ("en", "Monitor", "The self-monitoring loop — parse back and compare to intention. Levelt (1989): inner speech loop."),
        Message: ("en", "Message", "A fact selected from the knowledge base for expression. Reiter & Dale (2000): the atomic unit of communicable content."),
        DocumentPlan: ("en", "Document plan", "The rhetorical structure organizing multiple messages. Mann & Thompson RST (1988); McKeown (1985) schemata."),
        LexicalChoice: ("en", "Lexical choice", "A word selected from the lexicon to express a concept. Levelt (1989): lemma retrieval."),
    },

    edges: [
        // The Levelt pipeline: Goal → PreverbalMessage → SentencePlan → SurfaceForm
        (CommunicativeGoal, PreverbalMessage, Conceptualizes),
        (PreverbalMessage, SentencePlan, Formulates),
        (SentencePlan, SurfaceForm, Realizes),
        // Monitor loop: SurfaceForm → Monitor → PreverbalMessage (repair)
        (Monitor, SurfaceForm, Monitors),
        (Monitor, PreverbalMessage, Monitors),
        // Content determination: Goal → Messages → DocumentPlan → PreverbalMessage
        (CommunicativeGoal, Message, Selects),
        (Message, DocumentPlan, Organizes),
        (DocumentPlan, PreverbalMessage, Elaborates),
        // Lexical choice: SentencePlan uses LexicalChoice
        (SentencePlan, LexicalChoice, UsesLexicon),
    ],
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Concept;

    #[test]
    fn category_identity_law() {
        for obj in ProductionConcept::variants() {
            let id = ProductionCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
        }
    }

    #[test]
    fn category_composition_with_identity() {
        for m in &ProductionCategory::morphisms() {
            let left =
                ProductionCategory::compose(&ProductionCategory::identity(&m.from), m).unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);
        }
    }

    #[test]
    fn has_eight_concepts() {
        assert_eq!(ProductionConcept::variants().len(), 8);
    }

    #[test]
    fn levelt_pipeline_exists() {
        let m = ProductionCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == ProductionConcept::CommunicativeGoal
                    && r.to == ProductionConcept::PreverbalMessage
                    && r.kind == ProductionRelationKind::Conceptualizes)
        );
        assert!(
            m.iter()
                .any(|r| r.from == ProductionConcept::PreverbalMessage
                    && r.to == ProductionConcept::SentencePlan
                    && r.kind == ProductionRelationKind::Formulates)
        );
        assert!(m.iter().any(|r| r.from == ProductionConcept::SentencePlan
            && r.to == ProductionConcept::SurfaceForm
            && r.kind == ProductionRelationKind::Realizes));
    }

    #[test]
    fn full_pipeline_composes() {
        let conceptualize = ProductionRelation {
            from: ProductionConcept::CommunicativeGoal,
            to: ProductionConcept::PreverbalMessage,
            kind: ProductionRelationKind::Conceptualizes,
        };
        let formulate = ProductionRelation {
            from: ProductionConcept::PreverbalMessage,
            to: ProductionConcept::SentencePlan,
            kind: ProductionRelationKind::Formulates,
        };
        let composed = ProductionCategory::compose(&conceptualize, &formulate).unwrap();
        assert_eq!(composed.from, ProductionConcept::CommunicativeGoal);
        assert_eq!(composed.to, ProductionConcept::SentencePlan);
    }

    #[test]
    fn goal_reaches_surface_form() {
        assert!(
            ProductionCategory::morphisms()
                .iter()
                .any(|r| r.from == ProductionConcept::CommunicativeGoal
                    && r.to == ProductionConcept::SurfaceForm
                    && r.kind == ProductionRelationKind::Composed)
        );
    }

    #[test]
    fn monitor_exists() {
        let m = ProductionCategory::morphisms();
        assert!(m.iter().any(|r| r.from == ProductionConcept::Monitor
            && r.to == ProductionConcept::SurfaceForm
            && r.kind == ProductionRelationKind::Monitors));
        assert!(m.iter().any(|r| r.from == ProductionConcept::Monitor
            && r.to == ProductionConcept::PreverbalMessage
            && r.kind == ProductionRelationKind::Monitors));
    }

    #[test]
    fn content_determination_path() {
        let m = ProductionCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == ProductionConcept::CommunicativeGoal
                    && r.to == ProductionConcept::Message
                    && r.kind == ProductionRelationKind::Selects)
        );
        assert!(m.iter().any(|r| r.from == ProductionConcept::Message
            && r.to == ProductionConcept::DocumentPlan
            && r.kind == ProductionRelationKind::Organizes));
        assert!(m.iter().any(|r| r.from == ProductionConcept::DocumentPlan
            && r.to == ProductionConcept::PreverbalMessage
            && r.kind == ProductionRelationKind::Elaborates));
    }
}
