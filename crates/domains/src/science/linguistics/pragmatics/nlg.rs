use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for NlgConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::CommunicativeGoal,
            Self::ContentDetermination,
            Self::Message,
            Self::DocumentPlanning,
            Self::RhetoricalRelation,
            Self::Microplanning,
            Self::ReferringExpression,
            Self::Realization,
            Self::SurfaceText,
            Self::KnowledgeGathering,
            Self::Monitor,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NlgRelation {
    pub from: NlgConcept,
    pub to: NlgConcept,
    pub kind: NlgRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NlgRelationKind {
    Identity,
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
    Composed,
}

impl Relationship for NlgRelation {
    type Object = NlgConcept;
    fn source(&self) -> NlgConcept {
        self.from
    }
    fn target(&self) -> NlgConcept {
        self.to
    }
}

pub struct NlgCategory;

impl Category for NlgCategory {
    type Object = NlgConcept;
    type Morphism = NlgRelation;

    fn identity(obj: &NlgConcept) -> NlgRelation {
        NlgRelation {
            from: *obj,
            to: *obj,
            kind: NlgRelationKind::Identity,
        }
    }

    fn compose(f: &NlgRelation, g: &NlgRelation) -> Option<NlgRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == NlgRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == NlgRelationKind::Identity {
            return Some(f.clone());
        }
        Some(NlgRelation {
            from: f.from,
            to: g.to,
            kind: NlgRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<NlgRelation> {
        use NlgConcept as C;
        use NlgRelationKind as R;
        let mut m = Vec::new();

        for c in NlgConcept::variants() {
            m.push(NlgRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // CommunicativeGoal drives ContentDetermination
        m.push(NlgRelation {
            from: C::CommunicativeGoal,
            to: C::ContentDetermination,
            kind: R::Drives,
        });

        // ContentDetermination gathers knowledge and selects messages
        m.push(NlgRelation {
            from: C::ContentDetermination,
            to: C::KnowledgeGathering,
            kind: R::Gathers,
        });
        m.push(NlgRelation {
            from: C::ContentDetermination,
            to: C::Message,
            kind: R::Selects,
        });

        // DocumentPlanning organizes using RhetoricalRelation
        m.push(NlgRelation {
            from: C::DocumentPlanning,
            to: C::Message,
            kind: R::Organizes,
        });
        m.push(NlgRelation {
            from: C::DocumentPlanning,
            to: C::RhetoricalRelation,
            kind: R::Organizes,
        });

        // Microplanning produces ReferringExpressions
        m.push(NlgRelation {
            from: C::Microplanning,
            to: C::ReferringExpression,
            kind: R::Produces,
        });

        // Realization generates SurfaceText
        m.push(NlgRelation {
            from: C::Realization,
            to: C::SurfaceText,
            kind: R::Generates,
        });

        // Monitor checks SurfaceText against CommunicativeGoal
        m.push(NlgRelation {
            from: C::Monitor,
            to: C::SurfaceText,
            kind: R::Checks,
        });
        m.push(NlgRelation {
            from: C::Monitor,
            to: C::CommunicativeGoal,
            kind: R::Checks,
        });

        // Pipeline: CD → DP → MP → R
        m.push(NlgRelation {
            from: C::ContentDetermination,
            to: C::DocumentPlanning,
            kind: R::Precedes,
        });
        m.push(NlgRelation {
            from: C::DocumentPlanning,
            to: C::Microplanning,
            kind: R::Precedes,
        });
        m.push(NlgRelation {
            from: C::Microplanning,
            to: C::Realization,
            kind: R::Precedes,
        });

        // Composed: Goal → SurfaceText (full pipeline)
        m.push(NlgRelation {
            from: C::CommunicativeGoal,
            to: C::SurfaceText,
            kind: R::Composed,
        });
        m.push(NlgRelation {
            from: C::ContentDetermination,
            to: C::SurfaceText,
            kind: R::Composed,
        });
        m.push(NlgRelation {
            from: C::ContentDetermination,
            to: C::Realization,
            kind: R::Composed,
        });

        for c in NlgConcept::variants() {
            m.push(NlgRelation {
                from: c,
                to: c,
                kind: R::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_category_laws;

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
