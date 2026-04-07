use praxis_category::Category;

use crate::axiom::Axiom;
use crate::quality::Quality;

/// An ontology: what exists, how things relate, and what rules govern them.
///
/// An `Ontology` ties together:
/// - A category (individuals + relations with composition)
/// - Qualities on individuals (attributes, capabilities)
/// - Axioms that the ontology must satisfy (beyond basic category laws)
///
/// The ontology validates itself — if it compiles and passes validation,
/// the domain model is mathematically sound.
pub trait Ontology {
    /// The underlying category (individuals + relations).
    type Cat: Category;

    /// Qualities that individuals can have.
    type Qual: Quality<Individual = <Self::Cat as Category>::Object>;

    /// Axioms that must hold for this ontology to be valid.
    fn axioms() -> Vec<Box<dyn Axiom<Self::Cat>>>;

    /// Validate the entire ontology: category laws + all axioms.
    fn validate() -> Result<(), Vec<String>>
    where
        <Self::Cat as Category>::Morphism: PartialEq,
    {
        let mut errors = Vec::new();

        // Category laws — collect all errors instead of short-circuiting
        if let Err(e) = praxis_category::validate::check_identity_law::<Self::Cat>() {
            errors.push(e);
        }
        if let Err(e) = praxis_category::validate::check_associativity::<Self::Cat>() {
            errors.push(e);
        }
        if let Err(e) = praxis_category::validate::check_closure::<Self::Cat>() {
            errors.push(e);
        }

        // Axioms
        for axiom in Self::axioms() {
            if !axiom.holds() {
                errors.push(format!("axiom violated: {}", axiom.description()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
