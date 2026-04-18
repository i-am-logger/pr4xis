//! Kripke semantics — possible-worlds semantics for modal logic and
//! aspect-relative truth.
//!
//! Saul Kripke's semantic analysis (1959, 1963) is the standard frame
//! for modal logic: necessity / possibility / accessibility between
//! possible worlds. Heim's *Aspektrelativität* (aspect-relative truth)
//! is structurally the same idea applied to syntrometric Aspekts —
//! different observer-aspects see different facets of the underlying
//! distinction-system, and the relation-between-aspects is an
//! accessibility relation between Kripke frames.
//!
//! References:
//! - Kripke, S. (1959). *A Completeness Theorem in Modal Logic*. JSL 24(1).
//! - Kripke, S. (1963). *Semantical Analysis of Modal Logic I: Normal
//!   Propositional Calculi*. Zeitschrift für mathematische Logik 9.
//! - Hughes, G. E., & Cresswell, M. J. (1996). *A New Introduction to
//!   Modal Logic*. Routledge.
//! - van Benthem, J. (2010). *Modal Logic for Open Minds*. CSLI.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Kripke",
    source: "Kripke (1959, 1963); Hughes & Cresswell (1996)",
    being: AbstractObject,

    concepts: [
        // === Frames and worlds ===
        KripkeFrame,
        PossibleWorld,
        AccessibilityRelation,

        // === Semantic apparatus ===
        Valuation,
        ForcingRelation,

        // === Modal operators ===
        ModalOperator,
        Necessity,
        Possibility,

        // === Frame conditions (constraints on accessibility) ===
        FrameCondition,
        Reflexive,
        Symmetric,
        Transitive,
        Euclidean,
    ],

    labels: {
        KripkeFrame: ("en", "Kripke frame", "A pair (W, R) of possible worlds W and an accessibility relation R on W (Kripke 1963)."),
        PossibleWorld: ("en", "Possible world", "A single point in a Kripke frame — a way the world could be, at which propositions are evaluated."),
        AccessibilityRelation: ("en", "Accessibility relation", "The binary relation R on possible worlds — w R v means v is accessible from w. The modal operators quantify over accessible worlds."),

        Valuation: ("en", "Valuation", "The function V : Prop × W → {true, false} assigning truth values to atomic propositions at each world."),
        ForcingRelation: ("en", "Forcing relation ⊩", "The truth-at-a-world relation. w ⊩ φ iff φ is true at w given the valuation and accessibility."),

        ModalOperator: ("en", "Modal operator", "A unary operator on formulas whose semantics depend on the accessibility relation — □ and ◇."),
        Necessity: ("en", "Necessity □", "□φ is true at w iff φ is true at every v accessible from w."),
        Possibility: ("en", "Possibility ◇", "◇φ is true at w iff φ is true at some v accessible from w."),

        FrameCondition: ("en", "Frame condition", "A property required of the accessibility relation — reflexivity, symmetry, transitivity, etc. Different modal logics correspond to different frame conditions."),
        Reflexive: ("en", "Reflexive", "∀w. w R w. Corresponds to axiom T: □φ → φ."),
        Symmetric: ("en", "Symmetric", "∀w,v. w R v → v R w. Corresponds to axiom B: φ → □◇φ."),
        Transitive: ("en", "Transitive", "∀w,v,u. w R v ∧ v R u → w R u. Corresponds to axiom 4: □φ → □□φ."),
        Euclidean: ("en", "Euclidean", "∀w,v,u. w R v ∧ w R u → v R u. Corresponds to axiom 5: ◇φ → □◇φ."),
    },

    is_a: [
        // Every concrete modal operator is a ModalOperator.
        (Necessity, ModalOperator),
        (Possibility, ModalOperator),
        // Every concrete frame condition is a FrameCondition.
        (Reflexive, FrameCondition),
        (Symmetric, FrameCondition),
        (Transitive, FrameCondition),
        (Euclidean, FrameCondition),
    ],

    has_a: [
        // A Kripke frame contains worlds and an accessibility relation.
        (KripkeFrame, PossibleWorld),
        (KripkeFrame, AccessibilityRelation),
    ],

    edges: [
        // The accessibility relation holds between possible worlds.
        (AccessibilityRelation, PossibleWorld, RelatesWorlds),

        // The valuation + accessibility define the forcing relation.
        (Valuation, ForcingRelation, Determines),
        (AccessibilityRelation, ForcingRelation, Constrains),

        // Modal operators quantify over accessibility.
        (Necessity, AccessibilityRelation, QuantifiesOver),
        (Possibility, AccessibilityRelation, QuantifiesOver),

        // Frame conditions constrain the accessibility relation.
        (Reflexive, AccessibilityRelation, Constrains),
        (Symmetric, AccessibilityRelation, Constrains),
        (Transitive, AccessibilityRelation, Constrains),
        (Euclidean, AccessibilityRelation, Constrains),
    ],
}

/// Which aspect of the Kripke apparatus each concept belongs to.
#[derive(Debug, Clone)]
pub struct KripkeFamily;

impl Quality for KripkeFamily {
    type Individual = KripkeConcept;
    type Value = &'static str;

    fn get(&self, c: &KripkeConcept) -> Option<&'static str> {
        use KripkeConcept as K;
        Some(match c {
            K::KripkeFrame | K::PossibleWorld | K::AccessibilityRelation => "frame",
            K::Valuation | K::ForcingRelation => "semantics",
            K::ModalOperator | K::Necessity | K::Possibility => "modal-operator",
            K::FrameCondition | K::Reflexive | K::Symmetric | K::Transitive | K::Euclidean => {
                "frame-condition"
            }
        })
    }
}

fn direct_children_of(parent: KripkeConcept) -> Vec<KripkeConcept> {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    KripkeTaxonomy::relations()
        .into_iter()
        .filter_map(|(child, p)| if p == parent { Some(child) } else { None })
        .collect()
}

/// Axiom: the two modal operators are exactly `{Necessity, Possibility}`.
pub struct TwoModalOperators;

impl Axiom for TwoModalOperators {
    fn description(&self) -> &str {
        "the direct children of ModalOperator are exactly {Necessity, Possibility} (Kripke 1963)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(KripkeConcept::ModalOperator);
        let expected = [KripkeConcept::Necessity, KripkeConcept::Possibility];
        actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
    }
}
pr4xis::register_axiom!(
    TwoModalOperators,
    "- Kripke, S. (1959). *A Completeness Theorem in Modal Logic*. JSL 24(1)."
);

/// Axiom: the four standard frame conditions are all direct children of
/// FrameCondition. (S4 needs reflexive + transitive; S5 needs equivalence =
/// reflexive + symmetric + transitive; Kripke's original paper discussed
/// each.)
pub struct StandardFrameConditions;

impl Axiom for StandardFrameConditions {
    fn description(&self) -> &str {
        "FrameCondition has {Reflexive, Symmetric, Transitive, Euclidean} as direct children (Kripke 1963; Hughes & Cresswell 1996)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(KripkeConcept::FrameCondition);
        let expected = [
            KripkeConcept::Reflexive,
            KripkeConcept::Symmetric,
            KripkeConcept::Transitive,
            KripkeConcept::Euclidean,
        ];
        actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c))
    }
}
pr4xis::register_axiom!(
    StandardFrameConditions,
    "- Kripke, S. (1959). *A Completeness Theorem in Modal Logic*. JSL 24(1)."
);

/// Axiom: the Kripke frame mereologically contains both `PossibleWorld` and
/// `AccessibilityRelation` as its constitutive parts. Without this, the
/// (W, R) pair definition doesn't hold.
pub struct FrameContainsWorldsAndRelation;

impl Axiom for FrameContainsWorldsAndRelation {
    fn description(&self) -> &str {
        "KripkeFrame contains {PossibleWorld, AccessibilityRelation} as mereological parts (Kripke 1963: a frame IS the (W, R) pair)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::mereology::MereologyDef;
        let parts: Vec<_> = KripkeMereology::relations()
            .into_iter()
            .filter_map(|(w, p)| {
                if w == KripkeConcept::KripkeFrame {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        parts.contains(&KripkeConcept::PossibleWorld)
            && parts.contains(&KripkeConcept::AccessibilityRelation)
    }
}
pr4xis::register_axiom!(
    FrameContainsWorldsAndRelation,
    "- Kripke, S. (1959). *A Completeness Theorem in Modal Logic*. JSL 24(1)."
);

impl Ontology for KripkeOntology {
    type Cat = KripkeCategory;
    type Qual = KripkeFamily;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        KripkeOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TwoModalOperators),
            Box::new(StandardFrameConditions),
            Box::new(FrameContainsWorldsAndRelation),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<KripkeCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        KripkeOntology::validate().unwrap();
    }

    #[test]
    fn two_modal_operators_holds() {
        assert!(
            TwoModalOperators.holds(),
            "{}",
            TwoModalOperators.description()
        );
    }

    #[test]
    fn standard_frame_conditions_holds() {
        assert!(
            StandardFrameConditions.holds(),
            "{}",
            StandardFrameConditions.description()
        );
    }

    #[test]
    fn frame_contains_worlds_and_relation_holds() {
        assert!(
            FrameContainsWorldsAndRelation.holds(),
            "{}",
            FrameContainsWorldsAndRelation.description()
        );
    }
}
