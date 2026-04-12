use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

// Discourse Reference Ontology — how language tracks entities across utterances.
//
// Two foundational theories compose here:
//
// DRT (Discourse Representation Theory) — Kamp (1981), Kamp & Reyle (1993):
//   Meaning is not static truth conditions but DYNAMIC UPDATE to a discourse model.
//   Indefinites ("a dog") introduce new discourse referents.
//   Pronouns ("it") resolve to existing accessible referents.
//   Accessibility is structural: determined by DRS nesting.
//
// Centering Theory — Grosz, Joshi, Weinstein (1995):
//   Local discourse coherence is tracked by salience ranking.
//   Cf (forward-looking centers): entities in current utterance, ranked by grammar.
//   Cb (backward-looking center): most salient entity from previous utterance.
//   Transitions: Continue > Retain > Smooth Shift > Rough Shift.
//
// Together: DRT says what CAN be resolved; Centering says what SHOULD be resolved.
//
// References:
// - Kamp, A Theory of Truth and Semantic Representation (1981)
// - Kamp & Reyle, From Discourse to Logic (1993)
// - Grosz, Joshi, Weinstein, Centering (Computational Linguistics, 1995)
// - Van der Sandt, Presupposition Projection as Anaphora Resolution (1992)
// - Heim, The Semantics of Definite and Indefinite Noun Phrases (1982)

/// Core concepts of discourse reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceConcept {
    /// A discourse referent — an abstract placeholder for an entity
    /// introduced into the discourse model. NOT the real-world entity;
    /// a mediating representation that accumulates conditions.
    Referent,
    /// A Discourse Representation Structure — the discourse model at a point.
    /// Contains a universe of referents and conditions on them.
    DRS,
    /// A condition on referents within a DRS: predicates, relations, nested DRSs.
    Condition,
    /// The structural context determining which referents are visible.
    /// DRS nesting defines accessibility (Kamp & Reyle 1993).
    Accessibility,
    /// The salience state of an utterance (Grosz, Joshi, Weinstein 1995).
    /// Contains Cf (forward-looking centers), Cp (preferred), Cb (backward-looking).
    CenteringState,
    /// The coherence relationship between adjacent utterances.
    /// Continue, Retain, Smooth Shift, Rough Shift.
    Transition,
    /// A linguistic expression requiring resolution: pronouns, definites, demonstratives.
    AnaphoricExpression,
    /// The resolved link between an anaphor and its antecedent referent.
    Binding,
}

impl Entity for ReferenceConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Referent,
            Self::DRS,
            Self::Condition,
            Self::Accessibility,
            Self::CenteringState,
            Self::Transition,
            Self::AnaphoricExpression,
            Self::Binding,
        ]
    }
}

/// Centering transition types — how topic/salience shifts between utterances.
/// Grosz, Joshi, Weinstein (1995): preference ordering Continue > Retain > Shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CenteringTransition {
    /// Same topic, expected to persist. Cb(U_n) = Cb(U_{n-1}) and Cb(U_n) = Cp(U_n).
    Continue,
    /// Same topic, but a new entity becoming more salient. Cb persists but ≠ Cp.
    Retain,
    /// New topic, cleanly established. Cb changes, Cb = Cp.
    SmoothShift,
    /// New topic, not yet clearly established. Cb changes, Cb ≠ Cp.
    RoughShift,
}

impl Entity for CenteringTransition {
    fn variants() -> Vec<Self> {
        vec![
            Self::Continue,
            Self::Retain,
            Self::SmoothShift,
            Self::RoughShift,
        ]
    }
}

/// Relationships in the discourse reference category.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceRelation {
    pub from: ReferenceConcept,
    pub to: ReferenceConcept,
    pub kind: ReferenceRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceRelationKind {
    Identity,
    /// NP introduces a new discourse referent into the DRS.
    Introduces,
    /// Anaphor resolves to an existing referent.
    Resolves,
    /// Condition constrains what a referent can denote.
    Constrains,
    /// DRS contains referents in its universe.
    Contains,
    /// DRS nesting: sub-DRS for negation, conditionals, quantifiers.
    Subordinates,
    /// Referents in source DRS are visible from target DRS.
    Accessible,
    /// Processing an utterance extends the DRS.
    Updates,
    /// Centering state ranks referents by salience.
    Ranks,
    /// Centering links adjacent utterance states.
    Links,
    /// Binding connects anaphor to resolved referent.
    Binds,
    Composed,
}

impl Relationship for ReferenceRelation {
    type Object = ReferenceConcept;
    fn source(&self) -> ReferenceConcept {
        self.from
    }
    fn target(&self) -> ReferenceConcept {
        self.to
    }
}

pub struct ReferenceCategory;

impl Category for ReferenceCategory {
    type Object = ReferenceConcept;
    type Morphism = ReferenceRelation;

    fn identity(obj: &ReferenceConcept) -> ReferenceRelation {
        ReferenceRelation {
            from: *obj,
            to: *obj,
            kind: ReferenceRelationKind::Identity,
        }
    }

    fn compose(f: &ReferenceRelation, g: &ReferenceRelation) -> Option<ReferenceRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ReferenceRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ReferenceRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ReferenceRelation {
            from: f.from,
            to: g.to,
            kind: ReferenceRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ReferenceRelation> {
        use ReferenceConcept::*;
        use ReferenceRelationKind::*;

        let mut m = Vec::new();

        for c in ReferenceConcept::variants() {
            m.push(ReferenceRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // DRT structure
        m.push(ReferenceRelation {
            from: DRS,
            to: Referent,
            kind: Contains,
        });
        m.push(ReferenceRelation {
            from: Condition,
            to: Referent,
            kind: Constrains,
        });
        m.push(ReferenceRelation {
            from: DRS,
            to: DRS,
            kind: Subordinates,
        });
        m.push(ReferenceRelation {
            from: Accessibility,
            to: DRS,
            kind: Accessible,
        });

        // Introduction and resolution
        m.push(ReferenceRelation {
            from: Referent,
            to: DRS,
            kind: Introduces,
        });
        m.push(ReferenceRelation {
            from: AnaphoricExpression,
            to: Referent,
            kind: Resolves,
        });

        // Binding
        m.push(ReferenceRelation {
            from: Binding,
            to: AnaphoricExpression,
            kind: Binds,
        });
        m.push(ReferenceRelation {
            from: Binding,
            to: Referent,
            kind: Binds,
        });

        // Centering
        m.push(ReferenceRelation {
            from: CenteringState,
            to: Referent,
            kind: Ranks,
        });
        m.push(ReferenceRelation {
            from: CenteringState,
            to: CenteringState,
            kind: Links,
        });
        m.push(ReferenceRelation {
            from: Transition,
            to: CenteringState,
            kind: Links,
        });

        // Update: utterance processing extends DRS
        m.push(ReferenceRelation {
            from: DRS,
            to: Condition,
            kind: Updates,
        });

        // Transitive
        m.push(ReferenceRelation {
            from: AnaphoricExpression,
            to: DRS,
            kind: Composed,
        });
        m.push(ReferenceRelation {
            from: DRS,
            to: Condition,
            kind: Composed,
        });
        m.push(ReferenceRelation {
            from: Accessibility,
            to: Referent,
            kind: Composed,
        });

        // Self-composed closure
        for c in ReferenceConcept::variants() {
            m.push(ReferenceRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ReferenceCategory>().unwrap();
    }

    #[test]
    fn eight_concepts() {
        assert_eq!(ReferenceConcept::variants().len(), 8);
    }

    #[test]
    fn four_centering_transitions() {
        assert_eq!(CenteringTransition::variants().len(), 4);
    }

    #[test]
    fn drs_contains_referents() {
        let morphisms = ReferenceCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ReferenceConcept::DRS
            && m.to == ReferenceConcept::Referent
            && m.kind == ReferenceRelationKind::Contains));
    }

    #[test]
    fn anaphor_resolves_to_referent() {
        let morphisms = ReferenceCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ReferenceConcept::AnaphoricExpression
                    && m.to == ReferenceConcept::Referent
                    && m.kind == ReferenceRelationKind::Resolves)
        );
    }

    #[test]
    fn centering_links_states() {
        let morphisms = ReferenceCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ReferenceConcept::CenteringState
                    && m.to == ReferenceConcept::CenteringState
                    && m.kind == ReferenceRelationKind::Links)
        );
    }

    #[test]
    fn accessibility_reaches_referents() {
        // Accessibility → DRS → Referent (transitive through composition)
        let morphisms = ReferenceCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ReferenceConcept::Accessibility
                    && m.to == ReferenceConcept::Referent
                    && m.kind == ReferenceRelationKind::Composed)
        );
    }
}
