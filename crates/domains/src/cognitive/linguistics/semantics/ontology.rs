//! Montague semantics as a pr4xis ontology.
//!
//! Richard Montague's type-driven compositional semantics — the foundation
//! of pr4xis's Interpret step. Each syntactic type corresponds to a
//! semantic domain; composition in syntax corresponds to function
//! application in semantics. This is what makes `PipelineStep::Interpret`
//! resolvable to `MontagueOntology::meta().name` rather than a hardcoded
//! string.
//!
//! References:
//! - Montague, R. (1973). *The Proper Treatment of Quantification in
//!   Ordinary English*. In Hintikka et al. (eds), Approaches to Natural
//!   Language (Reidel).
//! - Montague, R. (1970). *Universal Grammar*. Theoria 36.
//! - Partee, B. (1975). *Montague Grammar and Transformational Grammar*.
//!   Linguistic Inquiry 6.
//! - Coecke, B., Sadrzadeh, M., Clark, S. (2010). *Mathematical
//!   foundations for a compositional distributional model of meaning*.
//!   Linguistic Analysis 36 — DisCoCat combines Montague's compositional
//!   structure with distributional word vectors.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Montague",
    source: "Montague (1973) in Hintikka et al.; Montague (1970) Theoria 36",

    concepts: [
        SemanticDomain,
        EntityDomain,
        PropositionDomain,
        PredicateDomain,
        FunctionDomain,
        Denotation,
        FunctionApplication,
        LambdaAbstraction,
    ],

    labels: {
        SemanticDomain: ("en", "Semantic domain", "A type of meaning — what a syntactic category denotes. Montague (1973)."),
        EntityDomain: ("en", "Concept domain (e)", "The domain of individuals. NPs denote entities. Montague (1973)."),
        PropositionDomain: ("en", "Proposition domain (t)", "The domain of truth values. Sentences denote propositions. Montague (1973)."),
        PredicateDomain: ("en", "Predicate domain (e→t)", "Functions from entities to truth values. Common nouns denote predicates."),
        FunctionDomain: ("en", "Function domain", "Higher-order function spaces built from the atomic domains e and t. Montague (1970)."),
        Denotation: ("en", "Denotation", "The meaning assigned to an expression by the interpretation function ⟦·⟧. Montague (1970)."),
        FunctionApplication: ("en", "Function application", "The compositional rule: ⟦f(x)⟧ = ⟦f⟧(⟦x⟧). Syntactic combination = semantic application. Montague (1973)."),
        LambdaAbstraction: ("en", "Lambda abstraction", "The binder λx.φ that produces a function from a context with a free variable. The calculus underlying Montague semantics."),
    },

    is_a: [
        (EntityDomain, SemanticDomain),
        (PropositionDomain, SemanticDomain),
        (PredicateDomain, SemanticDomain),
        (FunctionDomain, SemanticDomain),
    ],

    edges: [
        (Denotation, SemanticDomain, Inhabits),
        (FunctionApplication, Denotation, Combines),
        (LambdaAbstraction, FunctionDomain, Produces),
    ],
}

/// Whether a concept is a semantic domain, a denotation, or a combinator.
#[derive(Debug, Clone)]
pub struct MontagueRole;

impl Quality for MontagueRole {
    type Individual = MontagueConcept;
    type Value = &'static str;

    fn get(&self, c: &MontagueConcept) -> Option<&'static str> {
        use MontagueConcept as M;
        Some(match c {
            M::SemanticDomain
            | M::EntityDomain
            | M::PropositionDomain
            | M::PredicateDomain
            | M::FunctionDomain => "domain",
            M::Denotation => "value",
            M::FunctionApplication | M::LambdaAbstraction => "combinator",
        })
    }
}

/// Axiom: the two atomic semantic domains are entity (e) and truth value (t);
/// every other domain is built from them by function formation. This is the
/// defining property of Montague's type theory.
pub struct MontagueHasAtomicDomains;

impl Axiom for MontagueHasAtomicDomains {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let subs: Vec<_> = MontagueCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == MontagueRelationKind::Subsumption)
            .collect();
        if subs.iter().any(|m| {
            m.source() == MontagueConcept::EntityDomain
                && m.target() == MontagueConcept::SemanticDomain
        }) && subs.iter().any(|m| {
            m.source() == MontagueConcept::PropositionDomain
                && m.target() == MontagueConcept::SemanticDomain
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MontagueHasAtomicDomains",
        "Montague semantics has two atomic domains: EntityDomain (e) and PropositionDomain (t) — all other domains are functions built from them (Montague 1970)",
        "Montague (1970) Universal Grammar, Theoria 36; Montague (1973) The Proper Treatment of Quantification in Ordinary English"
    );
}
pr4xis::register_axiom!(
    MontagueHasAtomicDomains,
    "Montague (1970) Universal Grammar, Theoria 36; Montague (1973) The Proper Treatment of Quantification in Ordinary English"
);

impl Ontology for MontagueOntology {
    type Cat = MontagueCategory;
    type Qual = MontagueRole;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MontagueHasAtomicDomains));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<MontagueCategory>();
    }

    #[test]
    fn ontology_validates() {
        MontagueOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn montague_has_atomic_domains_holds() {
        match MontagueHasAtomicDomains.verify() {
            Ok(_) => {}
            Err(c) => panic!(
                "MontagueHasAtomicDomains failed: {}",
                c.meta().description.as_str()
            ),
        }
    }
}
