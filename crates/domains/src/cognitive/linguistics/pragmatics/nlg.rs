//! Natural Language Generation pipeline ontology.
//!
//! The Reiter & Dale (2000) four-stage pipeline, driven by metacognition
//! (what do I know?) and the Levelt production model.
//!
//! The pipeline:
//!   ContentDetermination → DocumentPlanning → Microplanning → Realization
//!
//! Each stage is a functor: transforms one representation into the next.
//! Content determination is driven by the epistemics ontology (KK/KU/UK/UU).
//! Document planning organizes using RST (rhetorical structure theory).
//! Microplanning selects words and referring expressions.
//! Realization produces surface text through the SVO grammar.
//!
//! References:
//! - Reiter & Dale, "Building Natural Language Generation Systems" (2000)
//! - Levelt, "Speaking: From Intention to Articulation" (1989)
//! - Mann & Thompson, "Rhetorical Structure Theory" (1988) — RST
//! - Appelt, "Planning English Sentences" (1985) — speech act planning
//! - McKeown, "Text Generation" (1985) — rhetorical schemata

pr4xis::ontology! {
    name: "Nlg",
    source: "Reiter & Dale (2000); Levelt (1989); Mann & Thompson (1988); Appelt (1985)",
    being: AbstractObject,

    concepts: [
        CommunicativeGoal,
        ContentDetermination,
        Message,
        DocumentPlanning,
        RhetoricalRelation,
        Microplanning,
        ReferringExpression,
        Realization,
        SurfaceText,
        KnowledgeGathering,
        Monitor,
    ],

    labels: {
        CommunicativeGoal: ("en", "Communicative goal", "Appelt (1985): a goal in the hearer's mental state. Driven by the epistemic state from metacognition."),
        ContentDetermination: ("en", "Content determination", "Reiter & Dale Stage 1: gather relevant knowledge from ontologies."),
        Message: ("en", "Message", "A fact selected from the knowledge base for expression. The atomic unit of communicable content."),
        DocumentPlanning: ("en", "Document planning", "Reiter & Dale Stage 2: organize messages using rhetorical structure. Mann & Thompson RST (1988): nucleus/satellite tree."),
        RhetoricalRelation: ("en", "Rhetorical relation", "RST: elaboration, evidence, contrast, sequence, etc."),
        Microplanning: ("en", "Microplanning", "Reiter & Dale Stage 3: select words, build referring expressions. Includes lexicalization, aggregation, REG."),
        ReferringExpression: ("en", "Referring expression", "Dale & Reiter (1995) incremental algorithm for RE generation."),
        Realization: ("en", "Realization", "Reiter & Dale Stage 4: produce actual text through grammar. de Groote (2001): beta-reduction in Lambek grammar."),
        SurfaceText: ("en", "Surface text", "Levelt (1989): the articulated utterance — the output."),
        KnowledgeGathering: ("en", "Knowledge gathering", "A structured collection of ontological facts gathered during content determination."),
        Monitor: ("en", "Monitor", "Levelt (1989): the inner speech loop. Parse back the generated text and compare to intent."),
    },

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<NlgCategory>().unwrap();
    }

    #[test]
    fn has_eleven_concepts() {
        assert_eq!(NlgConcept::variants().len(), 11);
    }

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

    #[test]
    fn goal_drives_content_determination() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::CommunicativeGoal
            && r.to == NlgConcept::ContentDetermination
            && r.kind == NlgRelationKind::Drives));
    }

    #[test]
    fn goal_reaches_surface_text() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::CommunicativeGoal
            && r.to == NlgConcept::SurfaceText));
    }

    #[test]
    fn realization_generates_text() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::Realization
            && r.to == NlgConcept::SurfaceText
            && r.kind == NlgRelationKind::Generates));
    }

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

    #[test]
    fn content_gathers_knowledge() {
        let m = NlgCategory::morphisms();
        assert!(m.iter().any(|r| r.from == NlgConcept::ContentDetermination
            && r.to == NlgConcept::KnowledgeGathering
            && r.kind == NlgRelationKind::Gathers));
    }

    #[test]
    fn document_planning_uses_rst() {
        let m = NlgCategory::morphisms();
        assert!(
            m.iter().any(|r| r.from == NlgConcept::DocumentPlanning
                && r.to == NlgConcept::RhetoricalRelation)
        );
    }
}
