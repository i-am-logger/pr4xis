// Pipeline → Algebra functor.
//
// Proof that the Parse ⊣ Generate pipeline IS algebraic composition.
// Parse = Delta migration (pull meaning back from surface).
// Generate = Sigma migration (push meaning forward to surface).
// The lexicon homomorphism IS a functor between ontologies.
// A proof term IS a span. A stream IS a colimit.
//
// This connects the language pipeline to the ontology algebra:
// every query through the pipeline IS a migration operation.
// "Is a dog a mammal?" = Delta migration through taxonomy.
//
// Source: de Groote (2001) × Spivak (2012)

use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::meta::algebra::ontology::*;

pub struct PipelineToAlgebra;

impl Functor for PipelineToAlgebra {
    type Source = PipelineCategory;
    type Target = AlgebraCategory;

    fn map_object(obj: &PipelineConcept) -> AlgebraConcept {
        match obj {
            // Parse = Delta migration (pull back along functor)
            PipelineConcept::Parse => AlgebraConcept::DeltaMigration,
            // Generate = Sigma migration (push forward)
            PipelineConcept::Generate => AlgebraConcept::SigmaMigration,
            // Unit = pullback (shared structure surviving round-trip)
            PipelineConcept::Unit => AlgebraConcept::Pullback,
            // Counit = pushout (merge from round-trip)
            PipelineConcept::Counit => AlgebraConcept::Pushout,
            // Surface and Semantic are both ontologies (object/abstract vocab)
            PipelineConcept::SurfaceForm => AlgebraConcept::Ontology,
            PipelineConcept::SemanticRepresentation => AlgebraConcept::Ontology,
            // Syntax is a diagram (composition structure)
            PipelineConcept::SyntacticStructure => AlgebraConcept::Diagram,
            // Lexicon homomorphism and meaning functor are mappings
            PipelineConcept::LexiconHomomorphism => AlgebraConcept::Mapping,
            PipelineConcept::MeaningFunctor => AlgebraConcept::Mapping,
            // Proof term is a span (connects syntax to semantics)
            PipelineConcept::ProofTerm => AlgebraConcept::Span,
            // Partial result is pullback (shared structure so far)
            PipelineConcept::PartialResult => AlgebraConcept::Pullback,
            // Stream is colimit (incremental composition)
            PipelineConcept::Stream => AlgebraConcept::Colimit,
        }
    }

    fn map_morphism(m: &PipelineRelation) -> AlgebraRelation {
        AlgebraRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
        }
    }
}
pr4xis::register_functor!(PipelineToAlgebra);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<PipelineToAlgebra>().unwrap();
    }
}
