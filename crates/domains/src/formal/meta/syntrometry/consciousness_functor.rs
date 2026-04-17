//! Cross-functor: Syntrometry → C1 (Dehaene Global Workspace).
//!
//! The `project_heim_transport.md` memory identifies Heim's `Maxime` as
//! architecturally equivalent to C1 `Attention` and Heim's `Metroplex`
//! as the `GlobalWorkspace`. This functor encodes that identification
//! and verifies it via `check_functor_laws`.
//!
//! Only two of the 14 Syntrometry concepts land at non-collapse targets:
//!
//! | Syntrometry | C1 | Why |
//! |---|---|---|
//! | `Maxime`        | `Attention`        | Extremal selection = GWT spotlight (Dehaene 2014) |
//! | `Metroplex`     | `GlobalWorkspace`  | Hierarchical container = workspace |
//! | everything else | `ConsciousAccess`  | Honest collapse |
//!
//! The collapse is an artifact of C1's deliberately narrow vocabulary —
//! GWT distinguishes only the broadcast mechanism, not the layered
//! distinction-system Heim describes. What matters for the lineage claim
//! is the two explicit mappings: Heim *anticipated* the attention/workspace
//! split that Dehaene's GWT formalises 34 years later.
//!
//! Edge `(Maxime, Aspekt, Selects)` lands on C1's declared
//! `(Attention, ConsciousAccess, Selects)` — both Heim's "extremal of
//! expedient ideas selects among candidate Aspekts" and GWT's "attention
//! selects which coalition accesses consciousness" are structurally the
//! same morphism. Edge `(Maxime, Telecenter, ConvergesToward)` likewise
//! collapses to `Selects` on `(Attention, ConsciousAccess)` since
//! `Telecenter` falls under the ConsciousAccess bucket in this projection.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    SyntrometryCategory, SyntrometryConcept, SyntrometryRelation, SyntrometryRelationKind,
};
use crate::cognitive::cognition::consciousness::ontology::{
    C1Category, C1Concept, C1Relation, C1RelationKind,
};

fn map_concept(c: &SyntrometryConcept) -> C1Concept {
    use C1Concept as C;
    use SyntrometryConcept as S;
    match c {
        S::Maxime => C::Attention,
        S::Metroplex => C::GlobalWorkspace,
        _ => C::ConsciousAccess,
    }
}

/// Cross-functor: Syntrometry → C1 (Global Workspace).
pub struct SyntrometryToC1;

impl Functor for SyntrometryToC1 {
    type Source = SyntrometryCategory;
    type Target = C1Category;

    fn map_object(obj: &SyntrometryConcept) -> C1Concept {
        map_concept(obj)
    }

    fn map_morphism(m: &SyntrometryRelation) -> C1Relation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            // Identity preservation.
            SyntrometryRelationKind::Identity => C1Category::identity(&from),
            // Composed source must map to Composed target so the composition
            // law F(g∘f) = F(g)∘F(f) holds (target compose always produces
            // Composed for non-Identity inputs).
            SyntrometryRelationKind::Composed => C1Relation {
                from,
                to,
                kind: C1RelationKind::Composed,
            },
            _ => {
                if from == to {
                    // Self-loop — every C1 concept has a Composed self-loop.
                    C1Relation {
                        from,
                        to,
                        kind: C1RelationKind::Composed,
                    }
                } else if from == C1Concept::Attention && to == C1Concept::ConsciousAccess {
                    // The Maxime → {Aspekt, Telecenter} edges both land
                    // here; C1 declares (Attention, ConsciousAccess, Selects).
                    C1Relation {
                        from,
                        to,
                        kind: C1RelationKind::Selects,
                    }
                } else {
                    // Fallback — shouldn't fire under the current concept
                    // mapping; if it does, check_functor_laws will report.
                    C1Relation {
                        from,
                        to,
                        kind: C1RelationKind::Composed,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    /// Heim's `Maxime → Attention` and `Metroplex → GlobalWorkspace`
    /// identifications land in pr4xis's C1 (Global Workspace Theory)
    /// ontology as a strict functor. The lineage from Heim's
    /// Maximentelezentrik to Dehaene's GWT is structurally verified.
    #[test]
    fn syntrometry_to_c1_laws_pass() {
        check_functor_laws::<SyntrometryToC1>().unwrap();
    }
}
