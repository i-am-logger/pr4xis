/// Physics ontology: laws of physics as entities with relationships.
use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Quality};

/// The fundamental laws as entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum PhysicsLaw {
    // Mechanics
    NewtonFirst,  // inertia
    NewtonSecond, // F = ma
    NewtonThird,  // action-reaction
    // Conservation
    EnergyConservation,
    MomentumConservation,
    ChargeConservation,
    // Electromagnetism
    GaussElectric, // div E = rho/eps0
    GaussMagnetic, // div B = 0
    FaradayLaw,    // curl E = -dB/dt
    AmpereMaxwell, // curl B = mu0 J + mu0 eps0 dE/dt
    // Relativity
    SpeedOfLight, // c is constant
    MassEnergy,   // E = mc^2
    // Quantum
    Heisenberg, // dx dp >= hbar/2
    Planck,     // E = hf
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over PhysicsLaw entities.
    pub PhysicsCategory {
        entity: PhysicsLaw,
        relation: Derives,
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
    type Individual = PhysicsLaw;
    type Value = Branch;

    fn get(&self, law: &PhysicsLaw) -> Option<Branch> {
        Some(match law {
            PhysicsLaw::NewtonFirst | PhysicsLaw::NewtonSecond | PhysicsLaw::NewtonThird => {
                Branch::Mechanics
            }
            PhysicsLaw::EnergyConservation
            | PhysicsLaw::MomentumConservation
            | PhysicsLaw::ChargeConservation => Branch::Conservation,
            PhysicsLaw::GaussElectric
            | PhysicsLaw::GaussMagnetic
            | PhysicsLaw::FaradayLaw
            | PhysicsLaw::AmpereMaxwell => Branch::Electromagnetism,
            PhysicsLaw::SpeedOfLight | PhysicsLaw::MassEnergy => Branch::Relativity,
            PhysicsLaw::Heisenberg | PhysicsLaw::Planck => Branch::Quantum,
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

/// Axiom: all branches are represented (no empty branch).
pub struct AllBranchesRepresented;

impl Axiom for AllBranchesRepresented {
    fn description(&self) -> &str {
        "every branch of physics has at least one law"
    }
    fn holds(&self) -> bool {
        let branch = LawBranch;
        let branches: std::collections::HashSet<Branch> = PhysicsLaw::variants()
            .iter()
            .map(|l| branch.get(l).unwrap())
            .collect();
        branches.len() == 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;

    #[test]
    fn test_14_laws() {
        assert_eq!(PhysicsLaw::variants().len(), 14);
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
        let em_laws: Vec<_> = PhysicsLaw::variants()
            .into_iter()
            .filter(|l| branch.get(l) == Some(Branch::Electromagnetism))
            .collect();
        assert_eq!(em_laws.len(), 4);
    }

    #[test]
    fn test_3_newton_laws() {
        let branch = LawBranch;
        let mech: Vec<_> = PhysicsLaw::variants()
            .into_iter()
            .filter(|l| branch.get(l) == Some(Branch::Mechanics))
            .collect();
        assert_eq!(mech.len(), 3);
    }
}
