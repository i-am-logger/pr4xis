use pr4xis::category::Entity;
use pr4xis::define_ontology;

// Natural Language Generation pipeline ontology.
//
// The Reiter & Dale (2000) four-stage pipeline, driven by
// metacognition (what do I know?) and the Levelt production model.
//
// The pipeline:
//   ContentDetermination → DocumentPlanning → Microplanning → Realization
//
// Each stage is a functor: transforms one representation into the next.
// Content determination is driven by the epistemics ontology (KK/KU/UK/UU).
// Document planning organizes using RST (rhetorical structure theory).
// Microplanning selects words and referring expressions.
// Realization produces surface text through the SVO grammar.
//
// References:
// - Reiter & Dale, "Building Natural Language Generation Systems" (2000)
// - Levelt, "Speaking: From Intention to Articulation" (1989)
// - Mann & Thompson, "Rhetorical Structure Theory" (1988) — RST
// - Appelt, "Planning English Sentences" (1985) — speech act planning
// - McKeown, "Text Generation" (1985) — rhetorical schemata

/// Concepts in the NLG pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum NlgConcept {
    /// The communicative goal — what the system wants to achieve.
    /// Appelt (1985): a goal in the hearer's mental state.
    /// Driven by the epistemic state from metacognition.
    CommunicativeGoal,

    /// Content determination: gather relevant knowledge from ontologies.
    /// Reiter & Dale (2000), Stage 1.
    /// Metacognition: traverse associations, find relationships.
    ContentDetermination,

    /// A fact selected from the knowledge base for expression.
    /// Reiter & Dale (2000): the atomic unit of communicable content.
    Message,

    /// Document planning: organize messages using rhetorical structure.
    /// Reiter & Dale (2000), Stage 2.
    /// Mann & Thompson RST (1988): nucleus/satellite tree.
    DocumentPlanning,

    /// A rhetorical relation organizing multiple messages.
    /// RST: elaboration, evidence, contrast, sequence, etc.
    RhetoricalRelation,

    /// Microplanning: select words, build referring expressions.
    /// Reiter & Dale (2000), Stage 3.
    /// Includes: lexicalization, aggregation, referring expression generation.
    Microplanning,

    /// A referring expression for an entity (definite, indefinite, pronoun).
    /// Dale & Reiter (1995): incremental algorithm for RE generation.
    ReferringExpression,

    /// Surface realization: produce the actual text through grammar.
    /// Reiter & Dale (2000), Stage 4.
    /// de Groote (2001): beta-reduction in the Lambek grammar.
    Realization,

    /// The final surface text — the output.
    /// Levelt (1989): the articulated utterance.
    SurfaceText,

    /// The knowledge gathered during content determination.
    /// A structured collection of ontological facts.
    KnowledgeGathering,

    /// Self-monitoring: parse back the generated text and compare to intent.
    /// Levelt (1989): the inner speech loop.
    Monitor,
}

define_ontology! {
    /// NLG Pipeline — four-stage generation (Reiter & Dale 2000).
    pub NlgOntology for NlgCategory {
        concepts: NlgConcept,
        relation: NlgRelation,
        kind: NlgRelationKind,
        kinds: [
            /// CommunicativeGoal drives ContentDetermination.
            Drives,
            /// ContentDetermination gathers KnowledgeGathering.
            Gathers,
            /// ContentDetermination selects Messages.
            Selects,
            /// DocumentPlanning organizes Messages using RhetoricalRelation.
            Organizes,
            /// Microplanning produces ReferringExpressions from Messages.
            Produces,
            /// Realization generates SurfaceText from the plan.
            Generates,
            /// Monitor checks SurfaceText against CommunicativeGoal.
            Checks,
            /// Pipeline stages: each precedes the next.
            Precedes,
        ],
        edges: [
            // CommunicativeGoal drives ContentDetermination
            (CommunicativeGoal, ContentDetermination, Drives),
            // ContentDetermination gathers knowledge and selects messages
            (ContentDetermination, KnowledgeGathering, Gathers),
            (ContentDetermination, Message, Selects),
            // DocumentPlanning organizes using RhetoricalRelation
            (DocumentPlanning, Message, Organizes),
            (DocumentPlanning, RhetoricalRelation, Organizes),
            // Microplanning produces ReferringExpressions
            (Microplanning, ReferringExpression, Produces),
            // Realization generates SurfaceText
            (Realization, SurfaceText, Generates),
            // Monitor checks SurfaceText against CommunicativeGoal
            (Monitor, SurfaceText, Checks),
            (Monitor, CommunicativeGoal, Checks),
            // Pipeline: CD → DP → MP → R
            (ContentDetermination, DocumentPlanning, Precedes),
            (DocumentPlanning, Microplanning, Precedes),
            (Microplanning, Realization, Precedes),
        ],
        composed: [
            // Composed: Goal → SurfaceText (full pipeline)
            (CommunicativeGoal, SurfaceText),
            (ContentDetermination, SurfaceText),
            (ContentDetermination, Realization),
        ],

        being: AbstractObject,
        source: "Reiter & Dale (2000)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<NlgCategory>().unwrap();
    }

    #[test]
    fn has_eleven_concepts() {
        assert_eq!(NlgConcept::variants().len(), 11);
    }

    // --- Reiter & Dale (2000): four-stage pipeline ---

    #[test]
    fn pipeline_order() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::ContentDetermination
            && r.to == NlgConcept::DocumentPlanning
            && r.kind == NlgRelationKind::Precedes));
        assert!(m.iter().any(|r| r.from == NlgConcept::DocumentPlanning
            && r.to == NlgConcept::Microplanning
            && r.kind == NlgRelationKind::Precedes));
        assert!(m.iter().any(|r| r.from == NlgConcept::Microplanning
            && r.to == NlgConcept::Realization
            && r.kind == NlgRelationKind::Precedes));
    }

    // --- Appelt (1985): goal drives content ---

    #[test]
    fn goal_drives_content_determination() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::CommunicativeGoal
            && r.to == NlgConcept::ContentDetermination
            && r.kind == NlgRelationKind::Drives));
    }

    // --- Full pipeline: Goal → SurfaceText ---

    #[test]
    fn goal_reaches_surface_text() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::CommunicativeGoal
            && r.to == NlgConcept::SurfaceText));
    }

    // --- Realization generates SurfaceText ---

    #[test]
    fn realization_generates_text() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::Realization
            && r.to == NlgConcept::SurfaceText
            && r.kind == NlgRelationKind::Generates));
    }

    // --- Levelt (1989): Monitor loop ---

    #[test]
    fn monitor_checks_output_against_goal() {
        let m = NlgCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == NlgConcept::Monitor && r.to == NlgConcept::SurfaceText)
        );
        assert!(
            m.iter()
                .any(|r| r.from == NlgConcept::Monitor && r.to == NlgConcept::CommunicativeGoal)
        );
    }

    // --- Content determination gathers knowledge ---

    #[test]
    fn content_gathers_knowledge() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::ContentDetermination
            && r.to == NlgConcept::KnowledgeGathering
            && r.kind == NlgRelationKind::Gathers));
    }

    // --- RST: DocumentPlanning uses RhetoricalRelations ---

    #[test]
    fn document_planning_uses_rst() {
        let m = NlgCategory::morphisms();
        assert!(
            m.iter().any(|r| r.from == NlgConcept::DocumentPlanning
                && r.to == NlgConcept::RhetoricalRelation)
        );
    }
}
