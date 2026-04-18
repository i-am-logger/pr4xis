//! Cross-functor: Syntrometry → Dialectics.
//!
//! Heim's `Dialektik` — "the binary-opposition structure on a Predikatrix"
//! — is exactly what Hegelian dialectical reasoning calls a
//! `DialecticalMoment` positioned against another. The functor carries
//! Heim's single primitive into the richer dialectical vocabulary with
//! full literature grounding (Aristotle's Square, Hegel's triad,
//! Adorno's non-identity, Priest's dialetheism).
//!
//! # The mapping
//!
//! | Syntrometry | Dialectics | Why |
//! |---|---|---|
//! | `Predicate`     | `Thesis`          | Atomic posited content |
//! | `Predikatrix`   | `DialecticalArgument` | A structured predicate-system is an argument-form |
//! | `Dialektik`     | `DialecticalMoment` | The binary-opposition primitive itself |
//! | `Koordination`  | `SquareOfOpposition` | Ordering-between-predicates = structured opposition |
//! | `Aspekt`        | `Synthesis`       | [D × K × P] is a higher unity |
//! | `Syntrix`       | `DialecticalArgument` | Collapse (Heim's category ≅ argument structure) |
//! | `SyntrixLevel`  | `Thesis`          | Collapse (level as a posited stance) |
//! | `Synkolator`    | `Sublation`       | Endofunctor on a Syntrix = Aufhebung move |
//! | `Korporator`    | `Sublation`       | Collapse |
//! | `Part`          | `DeterminateNegation` | Mereological split = specific negation |
//! | `Telecenter`    | `Synthesis`       | Goal-attractor = the synthesising move |
//! | `Maxime`        | `Sublation`       | Extremal selection = the sublating action |
//! | `Transzendenzstufe` | `Sublation`   | Transcendence-level = an elevation-move |
//! | `Metroplex`     | `DialecticalArgument` | Hierarchical composition of arguments |

use pr4xis::category::{Category, Functor};

use super::ontology::{
    SyntrometryCategory, SyntrometryConcept, SyntrometryRelation, SyntrometryRelationKind,
};
use crate::formal::logic::dialectics::ontology::{
    DialecticsCategory, DialecticsConcept, DialecticsRelation, DialecticsRelationKind,
};

fn map_concept(c: &SyntrometryConcept) -> DialecticsConcept {
    use DialecticsConcept as D;
    use SyntrometryConcept as S;
    match c {
        S::Predicate | S::SyntrixLevel => D::Thesis,
        S::PredicateMatrix | S::Syntrix | S::Metroplex => D::DialecticalArgument,
        S::Dialectic => D::DialecticalMoment,
        S::Coordination => D::SquareOfOpposition,
        S::Aspect | S::Telecenter => D::Synthesis,
        S::Syncolator | S::Composer | S::Maxim | S::TranscendenceLevel => D::Sublation,
        S::Part => D::DeterminateNegation,
        // Permutations are forms of determinate negation (they specifically
        // rearrange / negate existing order).
        S::SequencePermutation | S::OrientationPermutation => D::DeterminateNegation,
        // AspectivalSystem is a dialectical argument (structured opposition).
        S::AspectivalSystem => D::DialecticalArgument,
        // Reflexivity = self-sublation, the move that turns contradiction
        // into synthesis by self-application.
        S::Reflexivity => D::Sublation,
    }
}

/// Cross-functor: Syntrometry → Dialectics.
pub struct SyntrometryToDialectics;

impl Functor for SyntrometryToDialectics {
    type Source = SyntrometryCategory;
    type Target = DialecticsCategory;

    fn map_object(obj: &SyntrometryConcept) -> DialecticsConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> DialecticsRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            // Identity preservation: F(id_A) == id_{F(A)}.
            SyntrometryRelationKind::Identity => DialecticsCategory::identity(&from),
            // Every other kind maps to Composed in the target — matching how
            // the target's compose produces Composed morphisms for non-Identity
            // inputs (so F(g∘f) == F(g)∘F(f) holds even when F collapses
            // distinct source objects to the same target object).
            _ => DialecticsRelation {
                from,
                to,
                kind: DialecticsRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(SyntrometryToDialectics);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn syntrometry_to_dialectics_laws_pass() {
        check_functor_laws::<SyntrometryToDialectics>().unwrap();
    }
}
