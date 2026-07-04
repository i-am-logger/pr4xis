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

/// The four structural aspects of the Kripke apparatus. Every Kripke
/// concept belongs to exactly one: the frame (W, R) skeleton, the semantic
/// machinery that evaluates truth, the modal operators, or the frame
/// conditions that constrain accessibility. This partition is the standard
/// presentation of Kripke semantics (Kripke 1963; Hughes & Cresswell 1996,
/// New Introduction to Modal Logic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KripkeAspect {
    /// The frame skeleton: worlds W and the accessibility relation R.
    Frame,
    /// The semantic apparatus: valuation and the forcing relation ⊩.
    Semantics,
    /// The modal operators □ and ◇ that quantify over accessibility.
    ModalOperator,
    /// The frame conditions (reflexive, symmetric, …) on R.
    FrameCondition,
}

/// Which aspect of the Kripke apparatus each concept belongs to.
#[derive(Debug, Clone)]
pub struct KripkeFamily;

impl Quality for KripkeFamily {
    type Individual = KripkeConcept;
    type Value = KripkeAspect;

    fn get(&self, c: &KripkeConcept) -> Option<KripkeAspect> {
        use KripkeConcept as K;
        Some(match c {
            K::KripkeFrame | K::PossibleWorld | K::AccessibilityRelation => KripkeAspect::Frame,
            K::Valuation | K::ForcingRelation => KripkeAspect::Semantics,
            K::ModalOperator | K::Necessity | K::Possibility => KripkeAspect::ModalOperator,
            K::FrameCondition | K::Reflexive | K::Symmetric | K::Transitive | K::Euclidean => {
                KripkeAspect::FrameCondition
            }
        })
    }
}

/// Direct subsumption children of `parent`. Filters
/// `KripkeCategory::morphisms()` by the `Subsumption` kind, per the
/// kinded-morphism canonical pattern (per_def `TaxonomyDef` is gone).
fn direct_children_of(parent: KripkeConcept) -> Vec<KripkeConcept> {
    use pr4xis::category::{Arrow, Category};
    KripkeCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == KripkeRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

/// Axiom: the two modal operators are exactly `{Necessity, Possibility}`.
pub struct TwoModalOperators;

impl Axiom for TwoModalOperators {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let actual = direct_children_of(KripkeConcept::ModalOperator);
        let expected = [KripkeConcept::Necessity, KripkeConcept::Possibility];
        if actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "TwoModalOperators",
        "the direct children of ModalOperator are exactly {Necessity, Possibility} (Kripke 1963)",
        "Kripke, S. (1959). A Completeness Theorem in Modal Logic. JSL 24(1)."
    );
}
pr4xis::register_axiom!(
    TwoModalOperators,
    "Kripke, S. (1959). A Completeness Theorem in Modal Logic. JSL 24(1)."
);

/// Axiom: the four standard frame conditions are all direct children of
/// FrameCondition. (S4 needs reflexive + transitive; S5 needs equivalence =
/// reflexive + symmetric + transitive; Kripke's original paper discussed
/// each.)
pub struct StandardFrameConditions;

impl Axiom for StandardFrameConditions {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let actual = direct_children_of(KripkeConcept::FrameCondition);
        let expected = [
            KripkeConcept::Reflexive,
            KripkeConcept::Symmetric,
            KripkeConcept::Transitive,
            KripkeConcept::Euclidean,
        ];
        if actual.len() == expected.len() && expected.iter().all(|c| actual.contains(c)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "StandardFrameConditions",
        "FrameCondition has {Reflexive, Symmetric, Transitive, Euclidean} as direct children (Kripke 1963; Hughes & Cresswell 1996)",
        "Kripke (1963) Semantical Analysis of Modal Logic I; Hughes & Cresswell (1996) A New Introduction to Modal Logic"
    );
}
pr4xis::register_axiom!(
    StandardFrameConditions,
    "Kripke (1963) Semantical Analysis of Modal Logic I; Hughes & Cresswell (1996) A New Introduction to Modal Logic"
);

/// Axiom: the Kripke frame mereologically contains both `PossibleWorld` and
/// `AccessibilityRelation` as its constitutive parts. Without this, the
/// (W, R) pair definition doesn't hold.
pub struct FrameContainsWorldsAndRelation;

impl Axiom for FrameContainsWorldsAndRelation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // Direct mereological parts of KripkeFrame — filter morphisms by the
        // Parthood kind (per_def `MereologyDef` is gone).
        let parts: Vec<KripkeConcept> = KripkeCategory::morphisms()
            .iter()
            .filter(|m| {
                m.kind() == KripkeRelationKind::Parthood && m.target() == KripkeConcept::KripkeFrame
            })
            .map(|m| m.source())
            .collect();
        if parts.contains(&KripkeConcept::PossibleWorld)
            && parts.contains(&KripkeConcept::AccessibilityRelation)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FrameContainsWorldsAndRelation",
        "KripkeFrame contains {PossibleWorld, AccessibilityRelation} as mereological parts (Kripke 1963: a frame IS the (W, R) pair)",
        "Kripke (1963) Semantical Analysis of Modal Logic I"
    );
}
pr4xis::register_axiom!(
    FrameContainsWorldsAndRelation,
    "Kripke (1963) Semantical Analysis of Modal Logic I"
);

impl Ontology for KripkeOntology {
    type Cat = KripkeCategory;
    type Qual = KripkeFamily;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = KripkeOntology::generated_structural_axioms();
        axioms.push(Box::new(TwoModalOperators));
        axioms.push(Box::new(StandardFrameConditions));
        axioms.push(Box::new(FrameContainsWorldsAndRelation));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<KripkeCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        KripkeOntology::validate().unwrap();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn two_modal_operators_holds() {
        assert!(
            TwoModalOperators.verify().is_ok(),
            "{}",
            TwoModalOperators.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn standard_frame_conditions_holds() {
        assert!(
            StandardFrameConditions.verify().is_ok(),
            "{}",
            StandardFrameConditions.description().as_str()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn frame_contains_worlds_and_relation_holds() {
        assert!(
            FrameContainsWorldsAndRelation.verify().is_ok(),
            "{}",
            FrameContainsWorldsAndRelation.description().as_str()
        );
    }
}
