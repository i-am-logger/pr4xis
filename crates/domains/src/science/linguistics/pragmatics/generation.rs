use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Speech production ontology — the generation pipeline.
//
// This is the RIGHT ADJOINT of the parsing pipeline.
// Parse: Text → Syntax → Semantics (left adjoint, F)
// Generate: Semantics → Syntax → Text (right adjoint, G)
// Together: Parse ⊣ Generate (adjunction)
//
// The pipeline follows Levelt's speech production model (1989):
//   Conceptualizer → PreverbalMessage → Formulator → SurfaceForm
//
// Enriched by:
// - Reiter & Dale (2000): content determination → document planning → microplanning → realization
// - Appelt (1985): speech acts as plan operators with preconditions and effects
// - Pogodalla (2000): generation in Lambek calculus = proof search with semantic constraint fixed
// - de Groote (2001): ACG generation = beta-reduction of lexicon homomorphism (trivial!)
// - McKeown (1985): rhetorical schemata for content organization
//
// The key theorem (Pogodalla/de Groote): the SAME grammar does both parsing and
// generation. Generation is the easy direction (beta-reduce). Parsing is the hard
// direction (find pre-image). Our Lambek grammar already has everything needed.

/// Concepts in the speech production pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductionConcept {
    /// What the system wants to achieve by speaking.
    /// Appelt (1985): a goal in the hearer's mental state.
    CommunicativeGoal,

    /// The pre-linguistic representation of what to say.
    /// Levelt (1989): output of the Conceptualizer.
    /// Contains: speech act type, propositional content, topic/focus, mood.
    PreverbalMessage,

    /// The grammatical structure being built.
    /// Levelt (1989): output of the Formulator's grammatical encoder.
    /// = Vec<(word, LambekType)> — a typed sequence ready for realization.
    SentencePlan,

    /// The final surface string — the realized utterance.
    /// de Groote (2001): L(abstract_term) = beta-reduce = surface string.
    SurfaceForm,

    /// The self-monitoring loop — parse back and compare to intention.
    /// Levelt (1989): the speech comprehension system applied to own output.
    /// = Metacognition applied to generation.
    Monitor,

    /// A fact selected from the knowledge base for expression.
    /// Reiter & Dale (2000): the atomic unit of communicable content.
    Message,

    /// The rhetorical structure organizing multiple messages.
    /// Mann & Thompson RST (1988): nucleus/satellite tree.
    /// McKeown (1985): rhetorical schemata.
    DocumentPlan,

    /// A word selected from the lexicon to express a concept.
    /// Levelt (1989): lemma retrieval from mental lexicon.
    LexicalChoice,
}

impl Entity for ProductionConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::CommunicativeGoal,
            Self::PreverbalMessage,
            Self::SentencePlan,
            Self::SurfaceForm,
            Self::Monitor,
            Self::Message,
            Self::DocumentPlan,
            Self::LexicalChoice,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductionRelation {
    pub from: ProductionConcept,
    pub to: ProductionConcept,
    pub kind: ProductionRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductionRelationKind {
    Identity,
    /// CommunicativeGoal conceptualized into PreverbalMessage (Levelt macro+micro planning).
    Conceptualizes,
    /// PreverbalMessage formulated into SentencePlan (Levelt grammatical encoding).
    Formulates,
    /// SentencePlan realized as SurfaceForm (de Groote ACG beta-reduction).
    Realizes,
    /// Monitor checks SurfaceForm against PreverbalMessage (Levelt inner speech loop).
    Monitors,
    /// CommunicativeGoal selects Messages from knowledge base (Reiter-Dale content det.).
    Selects,
    /// Messages organized into DocumentPlan (RST / McKeown schemata).
    Organizes,
    /// DocumentPlan elaborated into PreverbalMessage (Levelt micro-planning per clause).
    Elaborates,
    /// SentencePlan uses LexicalChoice (Levelt lemma access).
    UsesLexicon,
    Composed,
}

impl Relationship for ProductionRelation {
    type Object = ProductionConcept;
    fn source(&self) -> ProductionConcept {
        self.from
    }
    fn target(&self) -> ProductionConcept {
        self.to
    }
}

pub struct ProductionCategory;

impl Category for ProductionCategory {
    type Object = ProductionConcept;
    type Morphism = ProductionRelation;

    fn identity(obj: &ProductionConcept) -> ProductionRelation {
        ProductionRelation {
            from: *obj,
            to: *obj,
            kind: ProductionRelationKind::Identity,
        }
    }

    fn compose(f: &ProductionRelation, g: &ProductionRelation) -> Option<ProductionRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ProductionRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ProductionRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ProductionRelation {
            from: f.from,
            to: g.to,
            kind: ProductionRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ProductionRelation> {
        use ProductionConcept as C;
        use ProductionRelationKind as R;
        let mut m = Vec::new();

        for c in ProductionConcept::variants() {
            m.push(ProductionRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // The Levelt pipeline: Goal → PreverbalMessage → SentencePlan → SurfaceForm
        m.push(ProductionRelation {
            from: C::CommunicativeGoal,
            to: C::PreverbalMessage,
            kind: R::Conceptualizes,
        });
        m.push(ProductionRelation {
            from: C::PreverbalMessage,
            to: C::SentencePlan,
            kind: R::Formulates,
        });
        m.push(ProductionRelation {
            from: C::SentencePlan,
            to: C::SurfaceForm,
            kind: R::Realizes,
        });

        // Monitor loop: SurfaceForm → Monitor → PreverbalMessage (repair)
        m.push(ProductionRelation {
            from: C::Monitor,
            to: C::SurfaceForm,
            kind: R::Monitors,
        });
        m.push(ProductionRelation {
            from: C::Monitor,
            to: C::PreverbalMessage,
            kind: R::Monitors,
        });

        // Content determination: Goal → Messages → DocumentPlan → PreverbalMessage
        m.push(ProductionRelation {
            from: C::CommunicativeGoal,
            to: C::Message,
            kind: R::Selects,
        });
        m.push(ProductionRelation {
            from: C::Message,
            to: C::DocumentPlan,
            kind: R::Organizes,
        });
        m.push(ProductionRelation {
            from: C::DocumentPlan,
            to: C::PreverbalMessage,
            kind: R::Elaborates,
        });

        // Lexical choice: SentencePlan uses LexicalChoice
        m.push(ProductionRelation {
            from: C::SentencePlan,
            to: C::LexicalChoice,
            kind: R::UsesLexicon,
        });

        // Transitive compositions
        // Goal → SurfaceForm (full pipeline)
        m.push(ProductionRelation {
            from: C::CommunicativeGoal,
            to: C::SurfaceForm,
            kind: R::Composed,
        });
        // Goal → SentencePlan (through PreverbalMessage)
        m.push(ProductionRelation {
            from: C::CommunicativeGoal,
            to: C::SentencePlan,
            kind: R::Composed,
        });
        // Goal → DocumentPlan (through Messages)
        m.push(ProductionRelation {
            from: C::CommunicativeGoal,
            to: C::DocumentPlan,
            kind: R::Composed,
        });
        // PreverbalMessage → SurfaceForm (formulate then realize)
        m.push(ProductionRelation {
            from: C::PreverbalMessage,
            to: C::SurfaceForm,
            kind: R::Composed,
        });
        // DocumentPlan → SurfaceForm (through PreverbalMessage)
        m.push(ProductionRelation {
            from: C::DocumentPlan,
            to: C::SurfaceForm,
            kind: R::Composed,
        });

        for c in ProductionConcept::variants() {
            m.push(ProductionRelation {
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
    use praxis::category::Category;
    use praxis::category::entity::Entity;

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
        // Goal → PreverbalMessage → SentencePlan → SurfaceForm
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
        // Goal → PreverbalMessage → SentencePlan composes
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
        // The full composed path should exist
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
        // Monitor checks both SurfaceForm and PreverbalMessage
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
        // Goal → Message → DocumentPlan → PreverbalMessage
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
