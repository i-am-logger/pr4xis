//! Discourse Reference Ontology — how language tracks entities across utterances.
//!
//! Two foundational theories compose here:
//!
//! DRT (Discourse Representation Theory) — Kamp (1981), Kamp & Reyle (1993):
//!   Meaning is not static truth conditions but DYNAMIC UPDATE to a discourse model.
//!   Indefinites ("a dog") introduce new discourse referents.
//!   Pronouns ("it") resolve to existing accessible referents.
//!   Accessibility is structural: determined by DRS nesting.
//!
//! Centering Theory — Grosz, Joshi, Weinstein (1995):
//!   Local discourse coherence is tracked by salience ranking.
//!   Cf (forward-looking centers): entities in current utterance, ranked by grammar.
//!   Cb (backward-looking center): most salient entity from previous utterance.
//!   Transitions: Continue > Retain > Smooth Shift > Rough Shift.
//!
//! Together: DRT says what CAN be resolved; Centering says what SHOULD be resolved.
//!
//! References:
//! - Kamp, A Theory of Truth and Semantic Representation (1981)
//! - Kamp & Reyle, From Discourse to Logic (1993)
//! - Grosz, Joshi, Weinstein, Centering (Computational Linguistics, 1995)
//! - Van der Sandt, Presupposition Projection as Anaphora Resolution (1992)
//! - Heim, The Semantics of Definite and Indefinite Noun Phrases (1982)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

pr4xis::ontology! {
    name: "Reference",
    source: "Kamp (1981); Grosz, Joshi & Weinstein (1995)",
    being: AbstractObject,

    concepts: [
        Referent,
        DRS,
        Condition,
        Accessibility,
        CenteringState,
        Transition,
        AnaphoricExpression,
        Binding,
    ],

    labels: {
        Referent: ("en", "Discourse referent", "Abstract placeholder for an entity introduced into the discourse model."),
        DRS: ("en", "Discourse Representation Structure", "The discourse model at a point. Universe of referents + conditions."),
        Condition: ("en", "Condition", "A condition on referents within a DRS: predicates, relations, nested DRSs."),
        Accessibility: ("en", "Accessibility", "Structural context determining which referents are visible (Kamp & Reyle 1993)."),
        CenteringState: ("en", "Centering state", "Cf (forward-looking), Cp (preferred), Cb (backward-looking) (Grosz et al. 1995)."),
        Transition: ("en", "Centering transition", "Continue / Retain / Smooth Shift / Rough Shift — coherence relationship."),
        AnaphoricExpression: ("en", "Anaphoric expression", "Linguistic expression requiring resolution: pronouns, definites, demonstratives."),
        Binding: ("en", "Binding", "The resolved link between an anaphor and its antecedent referent."),
    },

    edges: [
        // DRT structure
        (DRS, Referent, Contains),
        (Condition, Referent, Constrains),
        (DRS, DRS, Subordinates),
        (Accessibility, DRS, Accessible),
        // Introduction and resolution
        (Referent, DRS, Introduces),
        (AnaphoricExpression, Referent, Resolves),
        // Binding
        (Binding, AnaphoricExpression, Binds),
        (Binding, Referent, Binds),
        // Centering
        (CenteringState, Referent, Ranks),
        (CenteringState, CenteringState, Links),
        (Transition, CenteringState, Links),
        // Update: utterance processing extends DRS
        (DRS, Condition, Updates),
    ],
}

/// Centering transition types — how topic/salience shifts between utterances.
/// Grosz, Joshi, Weinstein (1995): preference ordering Continue > Retain > Shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
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
