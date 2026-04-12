/// Energy conservation as an ontology:
/// - Situation: a system with mass, velocity, height
/// - Axiom: total mechanical energy (KE + PE) is conserved
/// - Actions: change velocity or height (energy transforms, total constant)
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

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

impl Situation for System {
    fn describe(&self) -> String {
        format!(
            "m={:.2} v={:.2} h={:.2} KE={:.2} PE={:.2} E={:.2}",
            self.mass,
            self.velocity,
            self.height,
            self.kinetic_energy(),
            self.potential_energy(),
            self.total_energy()
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnergyAction {
    /// Drop: convert PE to KE by falling Δh.
    Drop { delta_h: f64 },
    /// Rise: convert KE to PE by rising Δh.
    Rise { delta_h: f64 },
}

impl Action for EnergyAction {
    type Sit = System;
    fn describe(&self) -> String {
        match self {
            EnergyAction::Drop { delta_h } => format!("drop {:.2}m", delta_h),
            EnergyAction::Rise { delta_h } => format!("rise {:.2}m", delta_h),
        }
    }
}

/// Axiom: total energy must be conserved.
struct EnergyConservation;
impl Precondition<EnergyAction> for EnergyConservation {
    fn check(&self, sys: &System, action: &EnergyAction) -> PreconditionResult {
        let next = apply_energy(sys, action).unwrap_or_else(|_| sys.clone());
        let e_before = sys.total_energy();
        let e_after = next.total_energy();
        let scale = e_before.abs().max(1.0);
        if (e_before - e_after).abs() / scale < 1e-6 {
            PreconditionResult::satisfied(
                "energy_conservation",
                &format!("E={:.4} conserved", e_before),
            )
        } else {
            PreconditionResult::violated(
                "energy_conservation",
                &format!("E changed: {:.4} → {:.4}", e_before, e_after),
                &sys.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "KE + PE must remain constant"
    }
}

/// Can't rise higher than KE allows, can't drop below ground.
struct PhysicalConstraints;
impl Precondition<EnergyAction> for PhysicalConstraints {
    fn check(&self, sys: &System, action: &EnergyAction) -> PreconditionResult {
        match action {
            EnergyAction::Drop { delta_h } => {
                if *delta_h <= 0.0 {
                    return PreconditionResult::violated(
                        "physical",
                        "drop must be positive",
                        &sys.describe(),
                        &action.describe(),
                    );
                }
                if *delta_h > sys.height {
                    return PreconditionResult::violated(
                        "physical",
                        "can't drop below ground",
                        &sys.describe(),
                        &action.describe(),
                    );
                }
            }
            EnergyAction::Rise { delta_h } => {
                if *delta_h <= 0.0 {
                    return PreconditionResult::violated(
                        "physical",
                        "rise must be positive",
                        &sys.describe(),
                        &action.describe(),
                    );
                }
                // Check if enough KE to rise
                let pe_needed = sys.mass * G * delta_h;
                if pe_needed > sys.kinetic_energy() + 1e-6 {
                    return PreconditionResult::violated(
                        "physical",
                        &format!(
                            "need {:.2}J PE but only {:.2}J KE available",
                            pe_needed,
                            sys.kinetic_energy()
                        ),
                        &sys.describe(),
                        &action.describe(),
                    );
                }
            }
        }
        PreconditionResult::satisfied("physical", "physically valid")
    }
    fn describe(&self) -> &str {
        "must have enough energy and stay above ground"
    }
}

fn apply_energy(sys: &System, action: &EnergyAction) -> Result<System, String> {
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
