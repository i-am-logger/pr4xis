//! Physics ontology: laws of physics as entities with relationships.
//!
//! Source: Newton (1687); Maxwell (1865).

use pr4xis::category::Entity;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Physics",
    source: "Newton (1687); Maxwell (1865)",
    being: AbstractObject,

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

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(MaxwellDerivesC), Box::new(AllBranchesRepresented)]
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
    fn description(&self) -> &str {
        "Maxwell's 4 equations together derive c = 1/sqrt(mu0 eps0)"
    }
    fn holds(&self) -> bool {
        let c = super::maxwell::speed_of_light();
        (c - 2.998e8).abs() < 1e6
    }
}
pr4xis::register_axiom!(MaxwellDerivesC);

/// Axiom: all branches are represented (no empty branch).
pub struct AllBranchesRepresented;

impl Axiom for AllBranchesRepresented {
    fn description(&self) -> &str {
        "every branch of physics has at least one law"
    }
    fn holds(&self) -> bool {
        let branch = LawBranch;
        let branches: std::collections::HashSet<Branch> = PhysicsConcept::variants()
            .iter()
            .map(|l| branch.get(l).unwrap())
            .collect();
        branches.len() == 5
    }
}
pr4xis::register_axiom!(AllBranchesRepresented);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_14_laws() {
        assert_eq!(PhysicsConcept::variants().len(), 14);
    }

    #[test]
    fn test_category_laws() {
        pr4xis::category::validate::check_category_laws::<PhysicsCategory>().unwrap();
    }

    #[test]
    fn test_all_branches() {
        assert!(AllBranchesRepresented.holds());
    }

    #[test]
    fn test_maxwell_derives_c() {
        assert!(MaxwellDerivesC.holds());
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
        PhysicsOntology::validate().unwrap();
    }
}
