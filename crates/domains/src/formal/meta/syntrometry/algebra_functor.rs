//! Cross-functor: Syntrometry → Algebra (ontology composition operations).
//!
//! Heim's Korporator and Aspekt align directly with Goguen/Zimmermann's
//! categorical ontology-algebra primitives. The mapping provides a
//! structural realisation of Heim's composition operators as the
//! algebraic constructions pr4xis uses for real composition via the
//! `compose` API (#103, merged).
//!
//! # The mapping
//!
//! | Syntrometry | AlgebraConcept | Why |
//! |---|---|---|
//! | `Predicate`     | `Ontology`      | Minimal ontology = single predicate |
//! | `Predikatrix`   | `Ontology`      | Structured predicates = ontology |
//! | `Dialektik`     | `Coproduct`     | Binary opposition = coproduct of two |
//! | `Koordination`  | `Mapping`       | Ordering = inter-ontology mapping |
//! | `Aspekt`        | `Product`       | [D × K × P] = product of three |
//! | `Syntrix`       | `Diagram`       | Leveled category = categorical diagram |
//! | `SyntrixLevel`  | `Ontology`      | Single level = single ontology |
//! | `Synkolator`    | `Mapping`       | Endofunctor = self-mapping |
//! | `Korporator`    | `Mapping`       | Structure-mapping = Mapping |
//! | `Part`          | `Pullback`      | Shared sub-structure (CEM Part) |
//! | `Telecenter`    | `Pushout`       | Goal-attractor = colimit/pushout |
//! | `Maxime`        | `Pullback`      | Selection = pullback of candidates |
//! | `Transzendenzstufe` | `Diagram`   | Grade within a hierarchy |
//! | `Metroplex`     | `Diagram`       | Hierarchical composition = diagram |

use pr4xis::category::{Category, Functor};

use super::ontology::{SyntrometryCategory, SyntrometryConcept, SyntrometryRelation};
use crate::formal::meta::algebra::ontology::{AlgebraCategory, AlgebraConcept, AlgebraRelation};

fn map_concept(c: &SyntrometryConcept) -> AlgebraConcept {
    use AlgebraConcept as A;
    use SyntrometryConcept as S;
    match c {
        S::Predicate | S::PredicateMatrix | S::SyntrixLevel => A::Ontology,
        S::Dialectic => A::Coproduct,
        S::Coordination | S::Syncolator | S::Composer => A::Mapping,
        S::Aspect => A::Product,
        S::Syntrix | S::TranscendenceLevel | S::Metroplex => A::Diagram,
        S::Part | S::Maxim => A::Pullback,
        S::Telecenter => A::Pushout,
        // Permutation operators are mappings on the category.
        S::SequencePermutation | S::OrientationPermutation => A::Mapping,
        // AspectivalSystem is a diagram (structured collection of objects).
        S::AspectivalSystem => A::Diagram,
        // Reflexivity is a natural transformation = a mapping-between-mappings.
        S::Reflexivity => A::Mapping,
    }
}

/// Cross-functor: Syntrometry → Algebra.
pub struct SyntrometryToAlgebra;

impl Functor for SyntrometryToAlgebra {
    type Source = SyntrometryCategory;
    type Target = AlgebraCategory;

    fn map_object(obj: &SyntrometryConcept) -> AlgebraConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> AlgebraRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        if from == to {
            AlgebraCategory::identity(&from)
        } else {
            AlgebraRelation { from, to }
        }
    }
}
pr4xis::register_functor!(SyntrometryToAlgebra);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn algebra_functor_laws_pass() {
        check_functor_laws::<SyntrometryToAlgebra>().unwrap();
    }
}
