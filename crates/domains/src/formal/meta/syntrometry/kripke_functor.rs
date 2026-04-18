//! Cross-functor: Syntrometry → Kripke.
//!
//! Heim's Aspektrelativität — the property of an Aspektivsystem that
//! different Aspekts see different facets of the same underlying
//! distinction-system — is structurally Kripke semantics applied to
//! syntrometric aspects. Each Aspekt is a frame; the relation between
//! Aspekts is an accessibility relation; truth is forced at each frame.
//!
//! # The mapping
//!
//! | Syntrometry | Kripke | Why |
//! |---|---|---|
//! | `Aspekt`        | `KripkeFrame`     | Each observer-aspect IS a Kripke frame |
//! | `Aspektivsystem` | `AccessibilityRelation` | Multi-aspect structure = accessibility |
//! | `Predicate`     | `PossibleWorld`   | Atomic distinction = point at which truth is evaluated |
//! | `Predikatrix`   | `Valuation`       | Structured predicate-system = truth-value assignment |
//! | `Dialektik`     | `FrameCondition`  | Binary opposition constrains accessibility |
//! | `Koordination`  | `ForcingRelation` | Ordering between predicates = the ⊩ relation |
//! | `Synkolator`    | `Necessity`       | Endofunctor over all accessible frames |
//! | `Korporator`    | `Possibility`     | Cross-syntrix functor = ∃ accessible frame |
//! | `Reflexivity`   | `Reflexive`       | Self-observation ↔ reflexive frame condition |
//! | `Maxime` / `Telecenter` / `Metroplex` / `Transzendenzstufe` | `KripkeFrame` | Collapse — frame-level concepts |
//! | `Syntrix` / `SyntrixLevel` / `Part` / `SequencePermutation` / `OrientationPermutation` | `PossibleWorld` | Collapse — point-level concepts |

use pr4xis::category::{Category, Functor};

use super::ontology::{
    SyntrometryCategory, SyntrometryConcept, SyntrometryRelation, SyntrometryRelationKind,
};
use crate::formal::logic::kripke::ontology::{
    KripkeCategory, KripkeConcept, KripkeRelation, KripkeRelationKind,
};

fn map_concept(c: &SyntrometryConcept) -> KripkeConcept {
    use KripkeConcept as K;
    use SyntrometryConcept as S;
    match c {
        S::Aspect | S::Maxim | S::Telecenter | S::Metroplex | S::TranscendenceLevel => {
            K::KripkeFrame
        }
        S::AspectivalSystem => K::AccessibilityRelation,
        S::Predicate => K::PossibleWorld,
        S::PredicateMatrix => K::Valuation,
        S::Dialectic => K::FrameCondition,
        S::Coordination => K::ForcingRelation,
        S::Syncolator => K::Necessity,
        S::Composer => K::Possibility,
        S::Reflexivity => K::Reflexive,
        S::Syntrix
        | S::SyntrixLevel
        | S::Part
        | S::SequencePermutation
        | S::OrientationPermutation => K::PossibleWorld,
    }
}

/// Cross-functor: Syntrometry → Kripke.
pub struct SyntrometryToKripke;

impl Functor for SyntrometryToKripke {
    type Source = SyntrometryCategory;
    type Target = KripkeCategory;

    fn map_object(obj: &SyntrometryConcept) -> KripkeConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> KripkeRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            SyntrometryRelationKind::Identity => KripkeCategory::identity(&from),
            SyntrometryRelationKind::Composed => KripkeRelation {
                from,
                to,
                kind: KripkeRelationKind::Composed,
            },
            // Either self-loop (Composed self-loop exists on every Kripke
            // concept) or cross-concept (construct Composed at the target
            // pair); both branches produce the same shape so share a body.
            _ => KripkeRelation {
                from,
                to,
                kind: KripkeRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(SyntrometryToKripke);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn syntrometry_to_kripke_laws_pass() {
        check_functor_laws::<SyntrometryToKripke>().unwrap();
    }
}
