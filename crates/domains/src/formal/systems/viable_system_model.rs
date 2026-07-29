//! Beer's Viable System Model (VSM) — the five subsystems every viable
//! (self-regulating, capable of independent existence) organization
//! needs, and how they connect.
//!
//! # Literature
//!
//! - **Beer, S. (1972)** *Brain of the Firm*, John Wiley & Sons — the
//!   original statement: S1 (Operations), S2 (Coordination), S3
//!   (Control), S3* (sporadic Audit, a direct bypass channel into S1),
//!   S4 (Intelligence), S5 (Policy).
//! - **Beer, S. (1979)** *The Heart of the Enterprise*, John Wiley &
//!   Sons — the fuller diagrammatic treatment, including the S3-S4
//!   homeostat S5 balances.
//! - **Beer, S. (1985)** *Diagnosing the System for Organizations*,
//!   John Wiley & Sons — the practitioner's checklist form this
//!   ontology's completeness axiom mirrors.
//! - **Espejo, R. & Reyes, A. (2011)** *Organizational Systems:
//!   Managing Complexity with the Viable System Model*, Springer — a
//!   modern restatement, including the recursion theorem.
//!
//! # Scope, honestly bounded
//!
//! Beer's central theorem is RECURSIVE: any viable system contains, and
//! is itself contained in, other viable systems (a firm's S1 units are
//! themselves viable systems with their own S1-S5). This module models
//! ONE level of the hierarchy — the five subsystems and their
//! connections — not the recursive embedding itself, which is a
//! separate, larger structural feature left for a tracked follow-up.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ViableSystemModel",
    source: "Beer (1972) Brain of the Firm; Beer (1979) The Heart of the Enterprise; Beer (1985) Diagnosing the System for Organizations; Espejo & Reyes (2011) Organizational Systems: Managing Complexity with the Viable System Model",

    concepts: [
        S1Operations,
        S2Coordination,
        S3Control,
        S3StarAudit,
        S4Intelligence,
        S5Policy,
        Environment,
    ],

    labels: {
        S1Operations: ("en", "S1 — Operations",
            "Beer (1972) Ch. 6: the primary activities that do the organization's actual work — the autonomous operational units, each capable of viability in its own right."),
        S2Coordination: ("en", "S2 — Coordination",
            "Beer (1972) Ch. 6: the anti-oscillatory channel that damps conflict and resource contention BETWEEN S1 units, without commanding them."),
        S3Control: ("en", "S3 — Control",
            "Beer (1972) Ch. 6: the 'here and now' management of the S1 units in aggregate — resource bargaining, synergy, and overall operational cohesion."),
        S3StarAudit: ("en", "S3* — Sporadic Audit",
            "Beer (1972) Ch. 6/9: a direct, occasional channel from S3 into S1 that bypasses the normal command structure — a spot-check, not a standing report."),
        S4Intelligence: ("en", "S4 — Intelligence",
            "Beer (1972) Ch. 7: looks OUTWARD to the environment and FORWARD in time — market/technology/regulatory scanning, planning, adaptation."),
        S5Policy: ("en", "S5 — Policy",
            "Beer (1972) Ch. 7: ultimate identity and authority; balances S3's inward/present focus against S4's outward/future focus and sets the direction both must serve."),
        Environment: ("en", "Environment",
            "Beer (1972) Ch. 7: everything outside the organization's boundary that S4 scans — the source of both opportunity and disturbance."),
    },

    edges: [
        // S2 dampens oscillation/conflict BETWEEN S1 units — a
        // coordination channel, not a command channel (Beer 1972 Ch. 6).
        (S2Coordination, S1Operations, Dampens),
        // S3 manages S1 in aggregate: resource bargaining, synergy.
        (S3Control, S1Operations, Manages),
        // S3 uses S2 as its coordination instrument.
        (S3Control, S2Coordination, Carries),
        // S3* is S3's direct audit channel into S1, bypassing the
        // normal S3->S2->S1 reporting path (Beer 1972 Ch. 9).
        (S3StarAudit, S1Operations, Audits),
        (S3Control, S3StarAudit, Carries),
        // S4 scans the environment (Beer 1972 Ch. 7).
        (S4Intelligence, Environment, Observes),
        // S5 balances S3 (inward, present) against S4 (outward,
        // future) — the central VSM homeostat (Beer 1979).
        (S5Policy, S3Control, Balances),
        (S5Policy, S4Intelligence, Balances),
    ],

    composed: [
        (S5Policy, S1Operations),
        (S3Control, Environment),
    ],
}

/// Quality: each level's "reach" — Beer (1972) frames S1-S3 as the
/// INSIDE-AND-NOW subsystems (operating and managing the organization
/// as it currently is) and S4-S5 as OUTSIDE-AND-THEN (scanning the
/// environment and setting future-facing identity/policy). S3* is
/// classified with S3 (it is S3's own instrument).
#[derive(Debug, Clone)]
pub struct InsideAndNow;

impl Quality for InsideAndNow {
    type Individual = ViableSystemModelConcept;
    type Value = bool;

    fn get(&self, c: &ViableSystemModelConcept) -> Option<bool> {
        use ViableSystemModelConcept as V;
        Some(matches!(
            c,
            V::S1Operations | V::S2Coordination | V::S3Control | V::S3StarAudit
        ))
    }
}

impl Ontology for ViableSystemModelOntology {
    type Cat = ViableSystemModelCategory;
    type Qual = InsideAndNow;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(VsmCompleteness));
        axioms
    }
}

/// Whether a declared system is VSM-complete — Beer (1985)
/// *Diagnosing the System for Organizations*'s own diagnostic
/// checklist: a viable system needs ALL FIVE subsystems present and
/// connected. Missing S5 means no identity/policy driving S3 and S4;
/// missing S4 means no adaptation to a changing environment (Beer
/// 1972 Ch. 7's "without S4 the system cannot survive change");
/// missing S2 means S1 units fight each other for resources
/// (oscillation, Beer 1972 Ch. 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsmDiagnosis {
    /// All five subsystems are present in the declared level set.
    Viable,
    /// At least one subsystem is missing — the declared system cannot
    /// sustain independent existence per Beer's own diagnostic.
    NotViable {
        /// The first missing subsystem found, for a concrete
        /// counterexample rather than a bare "something is missing."
        missing: ViableSystemModelConcept,
    },
}

/// Diagnose whether `present` (the levels a caller has actually
/// implemented/declared for some real system — see this module's
/// `poly_architecture_mapping_is_vsm_complete` test for a worked
/// example) forms a VSM-complete system. `Environment` and
/// `S3StarAudit` are excluded: the
/// former is not itself a subsystem of the organization (it's what S4
/// scans), and Beer treats S3* as optional ("sporadic," Ch. 9) rather
/// than a REQUIRED subsystem the way S1-S5 are.
pub fn diagnose_vsm_completeness(present: &[ViableSystemModelConcept]) -> VsmDiagnosis {
    use ViableSystemModelConcept as V;
    for required in [
        V::S1Operations,
        V::S2Coordination,
        V::S3Control,
        V::S4Intelligence,
        V::S5Policy,
    ] {
        if !present.contains(&required) {
            return VsmDiagnosis::NotViable { missing: required };
        }
    }
    VsmDiagnosis::Viable
}

pub struct VsmCompleteness;

impl Axiom for VsmCompleteness {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use ViableSystemModelConcept as V;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // Positive control: the full five-level set is diagnosed Viable.
        let complete = [
            V::S1Operations,
            V::S2Coordination,
            V::S3Control,
            V::S4Intelligence,
            V::S5Policy,
        ];
        if !matches!(diagnose_vsm_completeness(&complete), VsmDiagnosis::Viable) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }

        // Negative controls: removing each REQUIRED level in turn must
        // be diagnosed NotViable, naming exactly that level missing
        // (Beer 1985's own per-subsystem diagnostic).
        for missing_level in [
            V::S1Operations,
            V::S2Coordination,
            V::S3Control,
            V::S4Intelligence,
            V::S5Policy,
        ] {
            let partial: Vec<_> = complete
                .iter()
                .copied()
                .filter(|c| *c != missing_level)
                .collect();
            match diagnose_vsm_completeness(&partial) {
                VsmDiagnosis::NotViable { missing } if missing == missing_level => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }

        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "VsmCompleteness",
        "a system is diagnosed viable iff all five VSM subsystems (S1-S5) are present; removing any one is diagnosed not-viable, naming that subsystem",
        "Beer (1985) Diagnosing the System for Organizations"
    );
}

pr4xis::register_axiom!(
    VsmCompleteness,
    "Beer (1985) Diagnosing the System for Organizations"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ViableSystemModelCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ViableSystemModelOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn seven_concepts() {
        assert_eq!(ViableSystemModelConcept::variants().len(), 7);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn s5_balances_s3_and_s4() {
        let morphisms = ViableSystemModelCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ViableSystemModelConcept::S5Policy
                    && m.to == ViableSystemModelConcept::S3Control
                    && m.kind == ViableSystemModelRelationKind::Balances)
        );
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ViableSystemModelConcept::S5Policy
                    && m.to == ViableSystemModelConcept::S4Intelligence
                    && m.kind == ViableSystemModelRelationKind::Balances)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn s3_star_audits_s1_directly() {
        let morphisms = ViableSystemModelCategory::morphisms();
        assert!(morphisms.iter().any(|m| {
            m.from == ViableSystemModelConcept::S3StarAudit
                && m.to == ViableSystemModelConcept::S1Operations
                && m.kind == ViableSystemModelRelationKind::Audits
        }));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vsm_completeness_holds() {
        assert!(VsmCompleteness.verify().is_ok());
    }

    /// poly (the cybernetic trading bot this ontology arc was built to
    /// migrate — task #22) mapped onto the five VSM levels, per its
    /// own architecture: poly-strategy's registered strategies are S1
    /// (the operational units doing the actual trading); poly-meta's
    /// orchestrator coordinates them (S2, damping resource conflict
    /// between concurrently-running strategies); poly-meta's
    /// controller.rs is S3 (aggregate management via the PID/bandit
    /// control loop); poly-research + poly-meta's evolver are S4
    /// (scanning the literature/market environment, proposing new
    /// strategies); poly-meta's promotion gates (gates.rs) are S5 (the
    /// operator-defined policy a strategy must satisfy to go live).
    /// This is a DECLARED mapping (poly's own architecture, described
    /// from the outside — task #35 has not yet wired poly to this
    /// ontology), not a claim that poly's code imports this module.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn poly_architecture_mapping_is_vsm_complete() {
        use ViableSystemModelConcept as V;
        let poly_levels = [
            V::S1Operations,   // poly-strategy's registered strategies
            V::S2Coordination, // poly-meta::orchestrator
            V::S3Control,      // poly-meta::controller (PID/bandit)
            V::S4Intelligence, // poly-research + poly-meta::evolver
            V::S5Policy,       // poly-meta::gates (promotion gates)
        ];
        assert!(matches!(
            diagnose_vsm_completeness(&poly_levels),
            VsmDiagnosis::Viable
        ));
    }

    fn arb_concept() -> impl Strategy<Value = ViableSystemModelConcept> {
        proptest::sample::select(ViableSystemModelConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_inside_and_now_total(c in arb_concept()) {
            prop_assert!(InsideAndNow.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::Arrow;
            for m in ViableSystemModelCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ViableSystemModelOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_inside_and_now_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
