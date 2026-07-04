//! Lambek pregroup grammar as a pr4xis ontology.
//!
//! Joachim Lambek's pregroup grammar formalism — the foundation of
//! pr4xis's Parse step. Encodes the structural vocabulary (types,
//! contractions, expansions) so that `PipelineStep::Parse` can resolve
//! to `LambekOntology::meta().name` rather than a hardcoded string.
//!
//! References:
//! - Lambek, J. (1958). *The Mathematics of Sentence Structure*. American
//!   Mathematical Monthly 65(3).
//! - Lambek, J. (1999). *Type grammars revisited*. Lecture Notes in
//!   Computer Science 1582.
//! - Coecke, B., Sadrzadeh, M., Clark, S. (2010). *Mathematical
//!   foundations for a compositional distributional model of meaning*.
//!   Linguistic Analysis 36 — DisCoCat over pregroup grammar.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Lambek",
    source: "Lambek (1958) American Mathematical Monthly 65(3); Lambek (1999)",

    concepts: [
        LambekType,
        LeftAdjoint,
        RightAdjoint,
        Contraction,
        Expansion,
        Reduction,
    ],

    labels: {
        LambekType: ("en", "Lambek type", "A pregroup-algebra type — the grammatical category of a word or phrase. Lambek (1958)."),
        LeftAdjoint: ("en", "Left adjoint", "The left-adjoint operator on types: ᴸa · a → 1. Lambek (1999)."),
        RightAdjoint: ("en", "Right adjoint", "The right-adjoint operator on types: a · aᴿ → 1. Lambek (1999)."),
        Contraction: ("en", "Contraction", "The reduction rule that cancels adjacent type with its adjoint: a · aᴿ → 1 or ᴸa · a → 1. The syntactic content of Lambek's proof system."),
        Expansion: ("en", "Expansion", "The reverse rule: 1 → a · aᴿ (or ᴸa · a). Rarely used in parsing; present in the full calculus."),
        Reduction: ("en", "Reduction", "A sequence of contractions (and/or expansions) reducing a sentence's type string to the target sentence type s."),
    },

    is_a: [
        (LeftAdjoint, LambekType),
        (RightAdjoint, LambekType),
    ],

    edges: [
        (Contraction, LambekType, Cancels),
        (Expansion, LambekType, Produces),
        (Reduction, Contraction, ComposesOf),
    ],
}

/// The two structural roles a Lambek concept can play: forming a grammatical
/// type versus performing a reduction move.
///
/// A closed classification following Lambek's own separation of the pregroup
/// *algebra of types* (types and their left/right adjoints) from the *proof
/// system* of reduction rules (contraction and expansion) that acts on type
/// strings. Lambek (1999) *Type Grammars Revisited*, LNCS 1582.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LambekRoleKind {
    /// A type-forming construct: a type or one of its adjoint operators.
    TypeForming,
    /// A reduction move: a contraction, expansion, or a composed reduction.
    ReductionMove,
}

/// Quality: the structural [`LambekRoleKind`] each Lambek concept plays.
#[derive(Debug, Clone)]
pub struct LambekRole;

impl Quality for LambekRole {
    type Individual = LambekConcept;
    type Value = LambekRoleKind;

    fn get(&self, c: &LambekConcept) -> Option<LambekRoleKind> {
        use LambekConcept as L;
        Some(match c {
            L::LambekType | L::LeftAdjoint | L::RightAdjoint => LambekRoleKind::TypeForming,
            L::Contraction | L::Expansion | L::Reduction => LambekRoleKind::ReductionMove,
        })
    }
}

/// Axiom: Lambek types include both left and right adjoints — the defining
/// property of a *pregroup* (non-commutative residuated monoid).
pub struct LambekHasBothAdjoints;

impl Axiom for LambekHasBothAdjoints {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let subs: Vec<_> = LambekCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == LambekRelationKind::Subsumption)
            .collect();
        let left = subs.iter().any(|m| {
            m.source() == LambekConcept::LeftAdjoint && m.target() == LambekConcept::LambekType
        });
        let right = subs.iter().any(|m| {
            m.source() == LambekConcept::RightAdjoint && m.target() == LambekConcept::LambekType
        });
        if left && right {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LambekHasBothAdjoints",
        "LambekType has both LeftAdjoint and RightAdjoint as sub-kinds (Lambek 1999: pregroup = non-commutative adjoint calculus)",
        "Lambek (1958) The Mathematics of Sentence Structure, American Mathematical Monthly 65(3); Lambek (1999) Type Grammars Revisited"
    );
}
pr4xis::register_axiom!(
    LambekHasBothAdjoints,
    "Lambek (1958) The Mathematics of Sentence Structure, American Mathematical Monthly 65(3); Lambek (1999) Type Grammars Revisited"
);

impl Ontology for LambekOntology {
    type Cat = LambekCategory;
    type Qual = LambekRole;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(LambekHasBothAdjoints));
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
        assert_category_laws::<LambekCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        LambekOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lambek_has_both_adjoints_holds() {
        match LambekHasBothAdjoints.verify() {
            Ok(_) => {}
            Err(c) => panic!(
                "LambekHasBothAdjoints failed: {}",
                c.meta().description.as_str()
            ),
        }
    }
}
