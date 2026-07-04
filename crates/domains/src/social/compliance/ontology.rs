use std::sync::OnceLock;

use pr4xis::category::{Arrow, Category, FinitelyGenerated, NamedCategory};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::social::compliance::escalation::EscalationLevel;
use crate::social::compliance::law;

// ---------------------------------------------------------------------------
// Arrow kind: compliance has a single relation — escalation transitions.
// ---------------------------------------------------------------------------

/// Relation kind for the compliance category.
///
/// Per OBO-RO (Smith et al. 2005), every arrow carries a relation-kind
/// tag. The compliance category's only relation is the rules-of-engagement
/// state transition between escalation levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceRelationKind {
    EscalationTransition,
}

// ---------------------------------------------------------------------------
// Arrow: permitted transitions between escalation levels
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EscalationTransition {
    pub from: EscalationLevel,
    pub to: EscalationLevel,
}

impl Arrow for EscalationTransition {
    type Object = EscalationLevel;
    type Kind = ComplianceRelationKind;

    fn source(&self) -> EscalationLevel {
        self.from
    }

    fn target(&self) -> EscalationLevel {
        self.to
    }

    fn kind(&self) -> ComplianceRelationKind {
        ComplianceRelationKind::EscalationTransition
    }

    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("EscalationTransition"),
            description: Label::new_static(
                "rules-of-engagement transition between escalation levels",
            ),
            citation: Citation::parse_static("ISO 37301 (2021); NATO MC 362/1 Rules of Engagement"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

// ---------------------------------------------------------------------------
// Category: escalation ladder
// ---------------------------------------------------------------------------

/// The compliance category.
///
/// Objects: escalation levels.
/// Morphisms: permitted transitions.
///
/// The category structure enforces that only valid escalation
/// paths exist. There is no morphism from Observe to Engage
/// that doesn't pass through every intermediate level.
pub struct ComplianceCategory;

impl Category for ComplianceCategory {
    type Object = EscalationLevel;
    type Morphism = EscalationTransition;

    fn identity(obj: &EscalationLevel) -> EscalationTransition {
        EscalationTransition {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &EscalationTransition, g: &EscalationTransition) -> Option<EscalationTransition> {
        if f.to != g.from {
            return None;
        }
        let candidate = EscalationTransition {
            from: f.from,
            to: g.to,
        };
        // Total category over the closed transition graph: every composite
        // of declared transitions must itself be a declared transition
        // (ClosureLaw, Mac Lane CWM Ch. I §1). The morphism builder
        // computes the full reachability closure, so any composable pair
        // lands inside it.
        if morphism_set().contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<EscalationTransition> {
        morphism_set().iter().cloned().collect()
    }
}

/// The declared ontology identity of the compliance category.
///
/// `ComplianceCategory` is hand-rolled (it predates the `ontology!` macro), so
/// it declares its [`NamedCategory`] name by hand — the one-line impl the trait
/// doc prescribes for a hand-written category that participates as a functor
/// endpoint. This is what lets [`SituationToCompliance`](crate::social::military::situation::compliance_functor::SituationToCompliance)
/// serialize its target by content-addressable ontology name rather than a
/// toolchain-bound `type_name`.
impl NamedCategory for ComplianceCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("Compliance")
    }
}

/// The full morphism set, cached. Building it is O(|levels|³) (Warshall
/// 1962 transitive closure); `assert_category_laws` performs O(|m|³)
/// associativity checks each of which calls `compose` which queries the
/// morphism set, so rebuilding inside `compose` is O(|m|⁶). Caching with
/// `OnceLock` (idiomatic since Rust 1.70) drops the overall cost back to
/// O(|m|³). Returning `&HashSet` lets `compose` skip a linear `Vec::contains`
/// in favour of an O(1) hash lookup.
fn morphism_set() -> &'static std::collections::HashSet<EscalationTransition> {
    static CACHE: OnceLock<std::collections::HashSet<EscalationTransition>> = OnceLock::new();
    CACHE.get_or_init(build_morphism_set)
}

fn build_morphism_set() -> std::collections::HashSet<EscalationTransition> {
    use EscalationLevel::*;
    use std::collections::HashSet;

    let ladder = [
        Observe,
        Identify,
        Classify,
        Alert,
        Warn,
        ShowForce,
        NonLethal,
        WarningAction,
        Engage,
    ];

    // Direct edges (the LOAC rules-of-engagement primitive transitions).
    let mut direct: HashSet<(EscalationLevel, EscalationLevel)> = HashSet::new();
    // Sequential escalation (forward one step)
    for w in ladder.windows(2) {
        direct.insert((w[0], w[1]));
    }
    // De-escalation and abort are always available from ladder rungs
    for &level in &ladder {
        direct.insert((level, Deescalate));
        direct.insert((level, Abort));
    }
    // De-escalate and abort return to Observe
    direct.insert((Deescalate, Observe));
    direct.insert((Abort, Observe));

    // Transitive closure (Warshall 1962). Required by ClosureLaw +
    // AssociativityLaw — every composable pair (f.to == g.from) must
    // produce a composite that is itself a declared morphism, so the
    // morphism set must be closed under reachability.
    let mut closure = direct.clone();
    loop {
        let mut added = false;
        let snapshot: Vec<_> = closure.iter().cloned().collect();
        for &(a, b) in &snapshot {
            for &(b2, c) in &snapshot {
                if b == b2 && !closure.contains(&(a, c)) {
                    closure.insert((a, c));
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }

    let mut m: HashSet<EscalationTransition> = HashSet::new();
    for level in EscalationLevel::variants() {
        m.insert(EscalationTransition {
            from: level,
            to: level,
        });
    }
    for (a, b) in closure {
        if a != b {
            m.insert(EscalationTransition { from: a, to: b });
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

/// Quality: what authorization level does each escalation level require?
#[derive(Debug, Clone)]
pub struct RequiredAuthorization;

impl Quality for RequiredAuthorization {
    type Individual = EscalationLevel;
    type Value = crate::social::compliance::escalation::Authorization;

    fn get(
        &self,
        level: &EscalationLevel,
    ) -> Option<crate::social::compliance::escalation::Authorization> {
        Some(crate::social::compliance::escalation::required_authorization(*level))
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The compliance ontology.
///
/// Provable compliance with:
///   - Geneva Conventions I-IV (1949)
///   - Additional Protocols I & II (1977)
///   - US DoD Directive 3000.09 (2023)
///   - NATO MC 362/1 Rules of Engagement
///   - Hague Convention (1954) Cultural Property
///
/// If all axioms hold, the system is LOAC-compliant.
pub struct ComplianceOntology;

impl Ontology for ComplianceOntology {
    type Cat = ComplianceCategory;
    type Qual = RequiredAuthorization;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(law::DistinctionPrinciple),
            Box::new(law::CivilianPresumption),
            Box::new(law::HumanInTheLoop),
            Box::new(law::SequentialEscalation),
            Box::new(law::AdvanceWarning),
            Box::new(law::AbortAlwaysAvailable),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ComplianceCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ComplianceOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
