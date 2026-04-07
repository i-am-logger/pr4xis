/// Physics ontology: laws of physics as entities with relationships.
use praxis_category::{Category, Entity, Relationship};
use praxis_ontology::{Axiom, Quality};

/// The fundamental laws as entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    GaussElectric, // ∇⋅E = ρ/ε₀
    GaussMagnetic, // ∇⋅B = 0
    FaradayLaw,    // ∇×E = -∂B/∂t
    AmpereMaxwell, // ∇×B = μ₀J + μ₀ε₀∂E/∂t
    // Relativity
    SpeedOfLight, // c is constant
    MassEnergy,   // E = mc²
    // Quantum
    Heisenberg, // ΔxΔp ≥ ℏ/2
    Planck,     // E = hf
}

impl Entity for PhysicsLaw {
    fn variants() -> Vec<Self> {
        vec![
            PhysicsLaw::NewtonFirst,
            PhysicsLaw::NewtonSecond,
            PhysicsLaw::NewtonThird,
            PhysicsLaw::EnergyConservation,
            PhysicsLaw::MomentumConservation,
            PhysicsLaw::ChargeConservation,
            PhysicsLaw::GaussElectric,
            PhysicsLaw::GaussMagnetic,
            PhysicsLaw::FaradayLaw,
            PhysicsLaw::AmpereMaxwell,
            PhysicsLaw::SpeedOfLight,
            PhysicsLaw::MassEnergy,
            PhysicsLaw::Heisenberg,
            PhysicsLaw::Planck,
        ]
    }
}

/// Relationship: one law derives from or depends on another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derives {
    pub from: PhysicsLaw,
    pub to: PhysicsLaw,
}

impl Relationship for Derives {
    type Object = PhysicsLaw;
    fn source(&self) -> PhysicsLaw {
        self.from
    }
    fn target(&self) -> PhysicsLaw {
        self.to
    }
}

/// The physics domain as category.
pub struct PhysicsCategory;

impl Category for PhysicsCategory {
    type Object = PhysicsLaw;
    type Morphism = Derives;

    fn identity(obj: &PhysicsLaw) -> Derives {
        Derives {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &Derives, g: &Derives) -> Option<Derives> {
        if f.to != g.from {
            return None;
        }
        Some(Derives {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<Derives> {
        let laws = PhysicsLaw::variants();
        laws.iter()
            .flat_map(|&a| laws.iter().map(move |&b| Derives { from: a, to: b }))
            .collect()
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

impl Axiom<PhysicsCategory> for MaxwellDerivesC {
    fn description(&self) -> &str {
        "Maxwell's 4 equations together derive c = 1/√(μ₀ε₀)"
    }
    fn holds(&self) -> bool {
        let c = super::maxwell::speed_of_light();
        (c - 2.998e8).abs() < 1e6
    }
}

/// Axiom: all branches are represented (no empty branch).
pub struct AllBranchesRepresented;

impl Axiom<PhysicsCategory> for AllBranchesRepresented {
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

    #[test]
    fn test_14_laws() {
        assert_eq!(PhysicsLaw::variants().len(), 14);
    }

    #[test]
    fn test_category_laws() {
        praxis_category::validate::check_category_laws::<PhysicsCategory>().unwrap();
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
