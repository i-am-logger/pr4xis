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
    being: AbstractObject,

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

/// Whether a concept is a type-forming construct or a reduction move.
#[derive(Debug, Clone)]
pub struct LambekRole;

impl Quality for LambekRole {
    type Individual = LambekConcept;
    type Value = &'static str;

    fn get(&self, c: &LambekConcept) -> Option<&'static str> {
        use LambekConcept as L;
        Some(match c {
            L::LambekType | L::LeftAdjoint | L::RightAdjoint => "type-forming",
            L::Contraction | L::Expansion | L::Reduction => "reduction-move",
        })
    }
}

/// Axiom: Lambek types include both left and right adjoints — the defining
/// property of a *pregroup* (non-commutative residuated monoid).
pub struct LambekHasBothAdjoints;

impl Axiom for LambekHasBothAdjoints {
    fn description(&self) -> &str {
        "LambekType has both LeftAdjoint and RightAdjoint as sub-kinds (Lambek 1999: pregroup = non-commutative adjoint calculus)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
        let rels = LambekTaxonomy::relations();
        rels.iter()
            .any(|(c, p)| *c == LambekConcept::LeftAdjoint && *p == LambekConcept::LambekType)
            && rels
                .iter()
                .any(|(c, p)| *c == LambekConcept::RightAdjoint && *p == LambekConcept::LambekType)
    }
}
pr4xis::register_axiom!(
    LambekHasBothAdjoints,
    "- Lambek, J. (1958). *The Mathematics of Sentence Structure*. American"
);

impl Ontology for LambekOntology {
    type Cat = LambekCategory;
    type Qual = LambekRole;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        LambekOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(LambekHasBothAdjoints)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<LambekCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        LambekOntology::validate().unwrap();
    }

    #[test]
    fn lambek_has_both_adjoints_holds() {
        assert!(
            LambekHasBothAdjoints.holds(),
            "{}",
            LambekHasBothAdjoints.description()
        );
    }
}
