#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Energy conservation as an ontology:
/// - Situation: a system with mass, velocity, height
/// - Axiom: total mechanical energy (KE + PE) is conserved
/// - Actions: change velocity or height (energy transforms, total constant)
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

pub const G: f64 = 9.81;

#[derive(Debug, Clone, PartialEq)]
pub struct System {
    pub mass: f64,
    pub velocity: f64,
    pub height: f64,
}

impl System {
    pub fn new(mass: f64, velocity: f64, height: f64) -> Result<Self, &'static str> {
        if mass <= 0.0 {
            return Err("mass must be positive");
        }
        if height < 0.0 {
            return Err("height must be non-negative");
        }
        Ok(Self {
            mass,
            velocity,
            height,
        })
    }

    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity * self.velocity
    }
    pub fn potential_energy(&self) -> f64 {
        self.mass * G * self.height
    }
    pub fn total_energy(&self) -> f64 {
        self.kinetic_energy() + self.potential_energy()
    }
}

impl Situation for System {}

#[derive(Debug, Clone, PartialEq)]
pub enum EnergyAction {
    /// Drop: convert PE to KE by falling Δh.
    Drop { delta_h: f64 },
    /// Rise: convert KE to PE by rising Δh.
    Rise { delta_h: f64 },
}

impl Action for EnergyAction {
    type Sit = System;
}

fn energy_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Helmholtz (1847) Über die Erhaltung der Kraft; Joule (1843) On the calorific effects of magneto-electricity",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

/// Axiom: total energy must be conserved.
struct EnergyConservation;
impl Precondition<EnergyAction> for EnergyConservation {
    fn check(&self, sys: &System, action: &EnergyAction) -> Verdict {
        let meta = energy_meta("EnergyConservation", "KE + PE must remain constant");
        let next = apply_energy_inner(sys, action).unwrap_or_else(|_| sys.clone());
        let e_before = sys.total_energy();
        let e_after = next.total_energy();
        let scale = e_before.abs().max(1.0);
        if (e_before - e_after).abs() / scale < 1e-6 {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

/// Can't rise higher than KE allows, can't drop below ground.
struct PhysicalConstraints;
impl Precondition<EnergyAction> for PhysicalConstraints {
    fn check(&self, sys: &System, action: &EnergyAction) -> Verdict {
        let meta = energy_meta(
            "PhysicalConstraints",
            "must have enough energy and stay above ground",
        );
        match action {
            EnergyAction::Drop { delta_h } => {
                if *delta_h <= 0.0 || *delta_h > sys.height {
                    return Err(Box::new(SimpleCounterexample::new(meta)));
                }
            }
            EnergyAction::Rise { delta_h } => {
                if *delta_h <= 0.0 {
                    return Err(Box::new(SimpleCounterexample::new(meta)));
                }
                let pe_needed = sys.mass * G * delta_h;
                if pe_needed > sys.kinetic_energy() + 1e-6 {
                    return Err(Box::new(SimpleCounterexample::new(meta)));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_energy_inner(sys: &System, action: &EnergyAction) -> Result<System, &'static str> {
    let mut next = sys.clone();
    match action {
        EnergyAction::Drop { delta_h } => {
            next.height -= delta_h;
            // v² = v₀² + 2gΔh
            let v_sq = sys.velocity * sys.velocity + 2.0 * G * delta_h;
            next.velocity = v_sq.sqrt();
        }
        EnergyAction::Rise { delta_h } => {
            next.height += delta_h;
            // v² = v₀² - 2gΔh
            let v_sq = sys.velocity * sys.velocity - 2.0 * G * delta_h;
            next.velocity = if v_sq > 0.0 { v_sq.sqrt() } else { 0.0 };
        }
    }
    Ok(next)
}

fn apply_energy(sys: &System, action: &EnergyAction) -> Result<System, Box<dyn Counterexample>> {
    apply_energy_inner(sys, action).map_err(|_| {
        let meta = energy_meta("ApplyFailed", "energy transformation failed");
        Box::new(SimpleCounterexample::new(meta)) as Box<dyn Counterexample>
    })
}

pub fn new_system(
    mass: f64,
    velocity: f64,
    height: f64,
) -> Result<Engine<EnergyAction>, &'static str> {
    let sys = System::new(mass, velocity, height)?;
    Ok(Engine::new(
        sys,
        vec![Box::new(PhysicalConstraints), Box::new(EnergyConservation)],
        apply_energy,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_drop_converts_pe_to_ke() {
        let e = new_system(1.0, 0.0, 10.0)
            .unwrap()
            .next(EnergyAction::Drop { delta_h: 5.0 })
            .unwrap();
        assert!(e.situation().kinetic_energy() > 0.0);
        assert!((e.situation().height - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_energy_conserved_on_drop() {
        let e0 = new_system(1.0, 0.0, 10.0).unwrap();
        let e_before = e0.situation().total_energy();
        let e1 = e0.next(EnergyAction::Drop { delta_h: 10.0 }).unwrap();
        let e_after = e1.situation().total_energy();
        assert!((e_before - e_after).abs() < 0.01);
    }

    #[test]
    fn test_cant_drop_below_ground() {
        let e = new_system(1.0, 0.0, 5.0).unwrap();
        assert!(e.next(EnergyAction::Drop { delta_h: 10.0 }).is_err());
    }

    #[test]
    fn test_cant_rise_without_ke() {
        let e = new_system(1.0, 0.0, 5.0).unwrap(); // no velocity = no KE
        assert!(e.next(EnergyAction::Rise { delta_h: 1.0 }).is_err());
    }

    #[test]
    fn test_rise_then_drop_roundtrip() {
        let e = new_system(1.0, 10.0, 0.0)
            .unwrap()
            .next(EnergyAction::Rise { delta_h: 3.0 })
            .unwrap()
            .next(EnergyAction::Drop { delta_h: 3.0 })
            .unwrap();
        assert!((e.situation().velocity - 10.0).abs() < 0.01);
    }

    proptest! {
        #[test]
        fn prop_energy_conserved(mass in 0.1..100.0f64, v in 0.0..50.0f64, h in 1.0..100.0f64) {
            let e0 = new_system(mass, v, h).unwrap();
            let e_before = e0.situation().total_energy();
            let drop_h = h / 2.0;
            let e1 = e0.next(EnergyAction::Drop { delta_h: drop_h }).unwrap();
            let e_after = e1.situation().total_energy();
            let scale = e_before.abs().max(1.0);
            prop_assert!((e_before - e_after).abs() / scale < 1e-6);
        }

        #[test]
        fn prop_ke_nonneg(mass in 0.1..100.0f64, v in 0.0..50.0f64, h in 1.0..100.0f64) {
            let e = new_system(mass, v, h).unwrap()
                .next(EnergyAction::Drop { delta_h: h / 2.0 }).unwrap();
            prop_assert!(e.situation().kinetic_energy() >= 0.0);
        }

        #[test]
        fn prop_height_nonneg(mass in 0.1..100.0f64, v in 0.0..50.0f64, h in 1.0..100.0f64) {
            let e = new_system(mass, v, h).unwrap()
                .next(EnergyAction::Drop { delta_h: h / 2.0 }).unwrap();
            prop_assert!(e.situation().height >= 0.0);
        }
    }
}
