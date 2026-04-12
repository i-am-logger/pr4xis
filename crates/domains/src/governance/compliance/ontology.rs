use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::governance::compliance::escalation::EscalationLevel;
use crate::governance::compliance::law;

// ---------------------------------------------------------------------------
// Entity: escalation levels are the objects in the compliance category
// ---------------------------------------------------------------------------

impl Entity for EscalationLevel {
    fn variants() -> Vec<Self> {
        vec![
            Self::Observe,
            Self::Identify,
            Self::Classify,
            Self::Alert,
            Self::Warn,
            Self::ShowForce,
            Self::NonLethal,
            Self::WarningAction,
            Self::Engage,
            Self::Deescalate,
            Self::Abort,
        ]
    }
}

// ---------------------------------------------------------------------------
// Relationship: permitted transitions between escalation levels
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EscalationTransition {
    pub from: EscalationLevel,
    pub to: EscalationLevel,
}

impl Relationship for EscalationTransition {
    type Object = EscalationLevel;

    fn source(&self) -> EscalationLevel {
        self.from
    }

    fn target(&self) -> EscalationLevel {
        self.to
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
        Some(EscalationTransition {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<EscalationTransition> {
        use EscalationLevel::*;
        let mut m = Vec::new();

        // Identity for all
        for level in EscalationLevel::variants() {
            m.push(Self::identity(&level));
        }

        // Sequential escalation ladder (forward one step)
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
        for w in ladder.windows(2) {
            m.push(EscalationTransition {
                from: w[0],
                to: w[1],
            });
        }

        // De-escalation: any level can go to Deescalate
        for &level in &ladder {
            m.push(EscalationTransition {
                from: level,
                to: Deescalate,
            });
        }

        // Abort: any level can go to Abort
        for &level in &ladder {
            m.push(EscalationTransition {
                from: level,
                to: Abort,
            });
        }

        // Deescalate and Abort return to Observe
        m.push(EscalationTransition {
            from: Deescalate,
            to: Observe,
        });
        m.push(EscalationTransition {
            from: Abort,
            to: Observe,
        });

        // Transitive closure for sequential escalation (2-step, 3-step, etc.)
        // So that compose(Observe→Identify, Identify→Classify) = Observe→Classify exists
        for i in 0..ladder.len() {
            for j in (i + 2)..ladder.len() {
                m.push(EscalationTransition {
                    from: ladder[i],
                    to: ladder[j],
                });
            }
        }

        m
    }
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

/// Quality: what authorization level does each escalation level require?
#[derive(Debug, Clone)]
pub struct RequiredAuthorization;

impl Quality for RequiredAuthorization {
    type Individual = EscalationLevel;
    type Value = crate::governance::compliance::escalation::Authorization;

    fn get(
        &self,
        level: &EscalationLevel,
    ) -> Option<crate::governance::compliance::escalation::Authorization> {
        Some(crate::governance::compliance::escalation::required_authorization(*level))
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
