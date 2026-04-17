//! Cross-functor: Syntrometry → Staging (Futamura projections).
//!
//! Heim's Transzendenzstufen (transcendence levels within a Metroplex) map
//! directly to Futamura's (1971) staging hierarchy of programs. The
//! mapping formalises what `project_heim_transport.md` calls the
//! "Transzendenzstufen = Staging grades" claim.
//!
//! # The mapping
//!
//! | Syntrometry | StageConcept | Why |
//! |---|---|---|
//! | `Predicate`     | `SourceProgram`   | Atomic unit of a program |
//! | `Predikatrix`   | `Program`         | A structured set of predicates |
//! | `Dialektik`     | `Program`         | Opposition-structured program |
//! | `Koordination`  | `Program`         | Ordered program |
//! | `Aspekt`        | `Program`         | Product program |
//! | `Syntrix`       | `Program`         | Leveled program |
//! | `SyntrixLevel`  | `StaticInput`     | A fixed level = static input |
//! | `Synkolator`    | `Specializer`     | Endofunctor ≈ partial evaluator |
//! | `Korporator`    | `Compiler`        | General transformer |
//! | `Part`          | `StaticInput`     | A known part = static value |
//! | `Telecenter`    | `ObjectProgram`   | Goal-attractor = final compiled artifact |
//! | `Maxime`        | `Specializer`     | Extremal selection = specialization |
//! | **`Transzendenzstufe`** | **`Interpreter`** | **Transcendence-level = Futamura grade; Interpreter is the canonical grade-1 primitive** |
//! | `Metroplex`     | `CompilerGenerator` | Hierarchical generator (3rd Futamura projection) |
//!
//! Many-to-one collapse is intentional: Heim's grain is finer than
//! Futamura's. This functor demonstrates the *direction* of the lineage —
//! Heim's layered architecture is Futamura's staging seen from an
//! Austrian-physics vantage point — without forcing false precision.

use pr4xis::category::{Category, Functor};

use super::ontology::{SyntrometryCategory, SyntrometryConcept, SyntrometryRelation};
use crate::formal::meta::staging::ontology::{StageConcept, StagingCategory, StagingRelation};

fn map_concept(c: &SyntrometryConcept) -> StageConcept {
    use StageConcept as Stg;
    use SyntrometryConcept as S;
    match c {
        S::Predicate => Stg::SourceProgram,
        S::Predikatrix | S::Dialektik | S::Koordination | S::Aspekt | S::Syntrix => Stg::Program,
        S::SyntrixLevel | S::Part => Stg::StaticInput,
        S::Synkolator | S::Maxime => Stg::Specializer,
        S::Korporator => Stg::Compiler,
        S::Telecenter => Stg::ObjectProgram,
        S::Transzendenzstufe => Stg::Interpreter,
        S::Metroplex => Stg::CompilerGenerator,
        // Permutations act program-on-program — specializer-like.
        S::SequencePermutation | S::OrientationPermutation => Stg::Specializer,
        // Aspektivsystem is a structured system of programs = Program.
        S::Aspektivsystem => Stg::Program,
        // Reflexivity is the self-interpreter pattern.
        S::Reflexivity => Stg::Interpreter,
    }
}

/// Cross-functor: Syntrometry → Staging.
pub struct SyntrometryToStaging;

impl Functor for SyntrometryToStaging {
    type Source = SyntrometryCategory;
    type Target = StagingCategory;

    fn map_object(obj: &SyntrometryConcept) -> StageConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> StagingRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        if from == to {
            StagingCategory::identity(&from)
        } else {
            StagingRelation { from, to }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn staging_functor_laws_pass() {
        check_functor_laws::<SyntrometryToStaging>().unwrap();
    }
}
