//! Lambek → Montague functor — the syntax–semantics homomorphism.
//!
//! Montague's central methodological claim is that syntax and semantics are
//! *parallel algebras* joined by a homomorphism: every syntactic rule has a
//! corresponding semantic operation, so meaning is composed along the same
//! structure the parse builds. Montague (1970) states it as the requirement
//! that the interpretation map be a homomorphism from the syntactic algebra
//! into the algebra of meanings; Montague (1973) works the fragment out. A
//! homomorphism between two categories is exactly a functor, which is why this
//! belongs here rather than in prose.
//!
//! The parse step already computes the map on *terms*
//! ([`super::montague::interpret`] sends a reduced Lambek type to its semantic
//! domain, and reduction to function application). What was missing was the map
//! on the *ontologies*: the pipeline trace claimed a `Lambek→Montague`
//! connection that resolved to no functor at all, so a reader following the
//! trace was reading a name backed by nothing. This supplies the arrow the
//! trace names, and `assert_functor_laws` holds it to identity- and
//! composition-preservation.
//!
//! ## The object map
//!
//! Lambek's pregroup separates the *algebra of types* from the *proof system*
//! of reduction moves (Lambek 1999). Montague's semantics separates *domains*
//! from the *combinators* over their inhabitants. The functor carries that
//! separation across, which is what makes it structure-preserving rather than a
//! table of coincidences:
//!
//! | Lambek (syntax) | Montague (semantics) | why |
//! |---|---|---|
//! | `LambekType` | `SemanticDomain` | a grammatical category denotes a type of meaning |
//! | `LeftAdjoint` / `RightAdjoint` | `FunctionDomain` | an adjoint is the type of something awaiting an argument — a function space |
//! | `Contraction` | `FunctionApplication` | cancelling `a · aᴿ → 1` IS applying a function to its argument |
//! | `Expansion` | `LambdaAbstraction` | the reverse move introduces a function, which is abstraction |
//! | `Reduction` | `Denotation` | a completed reduction to `s` is what carries a sentence's meaning |
//!
//! The `Contraction ↦ FunctionApplication` row is the homomorphism proper —
//! Montague (1973)'s "syntactic combination = semantic application", which the
//! [`MontagueConcept::FunctionApplication`] label states in those words.
//!
//! References:
//! - Lambek, J. (1958). *The Mathematics of Sentence Structure*. American
//!   Mathematical Monthly 65(3).
//! - Lambek, J. (1999). *Type Grammars Revisited*. LNCS 1582.
//! - Montague, R. (1970). *Universal Grammar*. Theoria 36 — the homomorphism
//!   requirement.
//! - Montague, R. (1973). *The Proper Treatment of Quantification in Ordinary
//!   English*. In Hintikka et al. (eds), Approaches to Natural Language.
//! - Coecke, B., Sadrzadeh, M., Clark, S. (2010). *Mathematical foundations for
//!   a compositional distributional model of meaning*. Linguistic Analysis 36 —
//!   the pregroup-to-meaning functor made explicit.

use pr4xis::category::Functor;

use super::ontology::{LambekCategory, LambekConcept, LambekRelation, LambekRelationKind};
use crate::cognitive::linguistics::semantics::ontology::{
    MontagueCategory, MontagueConcept, MontagueRelation, MontagueRelationKind,
};

/// The syntax–semantics homomorphism: Lambek pregroup syntax → Montague
/// semantics. See the module docs for the object map and its justification.
pub struct LambekToMontague;

impl Functor for LambekToMontague {
    type Source = LambekCategory;
    type Target = MontagueCategory;

    fn map_object(obj: &LambekConcept) -> MontagueConcept {
        match obj {
            // A grammatical category denotes a type of meaning.
            LambekConcept::LambekType => MontagueConcept::SemanticDomain,
            // Both adjoints are the type of an expression awaiting an argument
            // on one side — a function space either way. The functor does not
            // distinguish them because Montague's type theory does not: `ᴸa`
            // and `aᴿ` differ in which side they consume, and directionality is
            // syntactic information that the semantic algebra discards.
            LambekConcept::LeftAdjoint | LambekConcept::RightAdjoint => {
                MontagueConcept::FunctionDomain
            }
            // THE homomorphism: syntactic cancellation is semantic application.
            LambekConcept::Contraction => MontagueConcept::FunctionApplication,
            // The reverse move introduces rather than consumes a function.
            LambekConcept::Expansion => MontagueConcept::LambdaAbstraction,
            // A completed reduction is what carries the sentence's meaning.
            LambekConcept::Reduction => MontagueConcept::Denotation,
        }
    }

    fn map_morphism(m: &LambekRelation) -> MontagueRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            LambekRelationKind::Identity => MontagueRelationKind::Identity,
            // `Contraction --Cancels--> LambekType` becomes
            // `FunctionApplication --Combines--> SemanticDomain`: what syntax
            // cancels, semantics combines. Same arrow, read on the other side.
            LambekRelationKind::Cancels => MontagueRelationKind::Combines,
            LambekRelationKind::Produces => MontagueRelationKind::Produces,
            // A reduction is composed of contractions; a denotation is combined
            // from applications.
            LambekRelationKind::ComposesOf => MontagueRelationKind::Combines,
            LambekRelationKind::Subsumption => MontagueRelationKind::Subsumption,
            // Canonical Relations-ontology kinds (Smith 2005 OBO-RO) emitted by
            // the ontology! macro — the Lambek source declares no edges of
            // these kinds, so these arms are unreachable in practice and map to
            // their Montague counterparts for totality.
            LambekRelationKind::Parthood => MontagueRelationKind::Parthood,
            LambekRelationKind::Causation => MontagueRelationKind::Causation,
            LambekRelationKind::Opposition => MontagueRelationKind::Opposition,
        };
        MontagueRelation { from, to, kind }
    }
}
pr4xis::register_functor!(LambekToMontague);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<LambekToMontague>();
    }

    /// The homomorphism's load-bearing row, asserted on its own.
    ///
    /// Montague (1973)'s claim is specifically that syntactic combination
    /// corresponds to semantic *application* — not merely that some map exists.
    /// A functor that satisfied the laws while sending `Contraction` somewhere
    /// else would still be a functor, and would no longer be Montague's.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn contraction_is_function_application() {
        assert_eq!(
            LambekToMontague::map_object(&LambekConcept::Contraction),
            MontagueConcept::FunctionApplication,
            "the syntax-semantics homomorphism sends pregroup contraction to \
             function application (Montague 1973); anything else is a \
             different functor wearing the same name"
        );
    }

    /// Type-forming syntax lands in domains, reduction moves land in
    /// combinators or values — the separation Lambek (1999) and Montague (1970)
    /// each draw, carried across intact.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn the_type_reduction_split_survives_the_functor() {
        use MontagueConcept as M;
        for (syntax, semantic) in [
            (LambekConcept::LambekType, M::SemanticDomain),
            (LambekConcept::LeftAdjoint, M::FunctionDomain),
            (LambekConcept::RightAdjoint, M::FunctionDomain),
        ] {
            assert_eq!(
                LambekToMontague::map_object(&syntax),
                semantic,
                "a type-forming Lambek concept must land in a semantic domain"
            );
        }
        for syntax in [
            LambekConcept::Contraction,
            LambekConcept::Expansion,
            LambekConcept::Reduction,
        ] {
            assert!(
                !matches!(
                    LambekToMontague::map_object(&syntax),
                    M::SemanticDomain
                        | M::EntityDomain
                        | M::PropositionDomain
                        | M::PredicateDomain
                        | M::FunctionDomain
                ),
                "a reduction move must not land in a semantic DOMAIN — it is an \
                 operation over inhabitants, not a type"
            );
        }
    }
}
