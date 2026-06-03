//! Physics ontology: laws of physics as entities with relationships.
//!
//! Source: Newton (1687); Maxwell (1865).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Physics",
    source: "Newton (1687); Maxwell (1865)",

    concepts: [
        NewtonFirst,
        NewtonSecond,
        NewtonThird,
        EnergyConservation,
        MomentumConservation,
        ChargeConservation,
        GaussElectric,
        GaussMagnetic,
        FaradayLaw,
        AmpereMaxwell,
        SpeedOfLight,
        MassEnergy,
        Heisenberg,
        Planck,
    ],

    labels: {
        NewtonFirst: ("en", "Newton's First Law", "Law of inertia."),
        NewtonSecond: ("en", "Newton's Second Law", "F = ma."),
        NewtonThird: ("en", "Newton's Third Law", "Action-reaction."),
        EnergyConservation: ("en", "Energy conservation", "Total energy is conserved."),
        MomentumConservation: ("en", "Momentum conservation", "Total momentum is conserved."),
        ChargeConservation: ("en", "Charge conservation", "Total electric charge is conserved."),
        GaussElectric: ("en", "Gauss's law (electric)", "div E = rho/eps0."),
        GaussMagnetic: ("en", "Gauss's law (magnetic)", "div B = 0."),
        FaradayLaw: ("en", "Faraday's law", "curl E = -dB/dt."),
        AmpereMaxwell: ("en", "Ampère-Maxwell law", "curl B = mu0 J + mu0 eps0 dE/dt."),
        SpeedOfLight: ("en", "Speed of light", "c is constant in all inertial frames."),
        MassEnergy: ("en", "Mass-energy equivalence", "E = mc^2."),
        Heisenberg: ("en", "Heisenberg uncertainty", "dx dp >= hbar/2."),
        Planck: ("en", "Planck relation", "E = hf."),
    },
}

impl Ontology for PhysicsOntology {
    type Cat = PhysicsCategory;
    type Qual = LawBranch;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MaxwellDerivesC));
        axioms.push(Box::new(AllBranchesRepresented));
        axioms
    }
}

/// Which branch of physics does this law belong to?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Branch {
    Mechanics,
    Conservation,
    Electromagnetism,
    Relativity,
    Quantum,
}

#[derive(Debug, Clone)]
pub struct LawBranch;

impl Quality for LawBranch {
    type Individual = PhysicsConcept;
    type Value = Branch;

    fn get(&self, law: &PhysicsConcept) -> Option<Branch> {
        Some(match law {
            PhysicsConcept::NewtonFirst
            | PhysicsConcept::NewtonSecond
            | PhysicsConcept::NewtonThird => Branch::Mechanics,
            PhysicsConcept::EnergyConservation
            | PhysicsConcept::MomentumConservation
            | PhysicsConcept::ChargeConservation => Branch::Conservation,
            PhysicsConcept::GaussElectric
            | PhysicsConcept::GaussMagnetic
            | PhysicsConcept::FaradayLaw
            | PhysicsConcept::AmpereMaxwell => Branch::Electromagnetism,
            PhysicsConcept::SpeedOfLight | PhysicsConcept::MassEnergy => Branch::Relativity,
            PhysicsConcept::Heisenberg | PhysicsConcept::Planck => Branch::Quantum,
        })
    }
}

/// Axiom: Maxwell's equations (all 4) derive the speed of light.
pub struct MaxwellDerivesC;

impl Axiom for MaxwellDerivesC {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let c = super::maxwell::speed_of_light();
        if (c - 2.998e8).abs() < 1e6 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "MaxwellDerivesC",
        "Maxwell's 4 equations together derive c = 1/sqrt(mu0 eps0)",
        "Maxwell (1865) A Dynamical Theory of the Electromagnetic Field"
    );
}
pr4xis::register_axiom!(
    MaxwellDerivesC,
    "Maxwell (1865) A Dynamical Theory of the Electromagnetic Field"
);

/// Axiom: all branches are represented (no empty branch).
pub struct AllBranchesRepresented;

impl Axiom for AllBranchesRepresented {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let branch = LawBranch;
        let branches: hashbrown::HashSet<Branch> = PhysicsConcept::variants()
            .iter()
            .map(|l| branch.get(l).unwrap())
            .collect();
        if branches.len() == 5 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "AllBranchesRepresented",
        "every branch of physics has at least one law",
        "Newton (1687) Principia; Maxwell (1865) A Dynamical Theory of the Electromagnetic Field"
    );
}
pr4xis::register_axiom!(
    AllBranchesRepresented,
    "Newton (1687) Principia; Maxwell (1865) A Dynamical Theory of the Electromagnetic Field"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_14_laws() {
        assert_eq!(PhysicsConcept::variants().len(), 14);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::laws::assert_category_laws::<PhysicsCategory>();
    }

    #[test]
    fn test_all_branches() {
        assert!(AllBranchesRepresented.verify().is_ok());
    }

    #[test]
    fn test_maxwell_derives_c() {
        assert!(MaxwellDerivesC.verify().is_ok());
    }

    #[test]
    fn test_4_maxwell_equations() {
        let branch = LawBranch;
        let em_laws: Vec<_> = PhysicsConcept::variants()
            .into_iter()
            .filter(|l| branch.get(l) == Some(Branch::Electromagnetism))
            .collect();
        assert_eq!(em_laws.len(), 4);
    }

    #[test]
    fn test_3_newton_laws() {
        let branch = LawBranch;
        let mech: Vec<_> = PhysicsConcept::variants()
            .into_iter()
            .filter(|l| branch.get(l) == Some(Branch::Mechanics))
            .collect();
        assert_eq!(mech.len(), 3);
    }

    #[test]
    fn ontology_validates() {
        PhysicsOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
