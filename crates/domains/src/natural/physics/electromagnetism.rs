#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Electromagnetism as an ontology:
/// - Situation: Circuit (V, I, R)
/// - Axiom: Ohm's law V=IR enforced on every change
/// - Actions: set voltage, set resistance (current derived)
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

pub const K_E: f64 = 8.988e9;

#[derive(Debug, Clone, PartialEq)]
pub struct Circuit {
    pub voltage: f64,
    pub current: f64,
    pub resistance: f64,
}

impl Circuit {
    pub fn from_vr(voltage: f64, resistance: f64) -> Result<Self, &'static str> {
        if resistance <= 0.0 {
            return Err("resistance must be positive");
        }
        Ok(Self {
            voltage,
            current: voltage / resistance,
            resistance,
        })
    }

    pub fn ohms_law_holds(&self) -> bool {
        let expected = self.current * self.resistance;
        let scale = self.voltage.abs().max(expected.abs()).max(1e-10);
        (self.voltage - expected).abs() / scale < 1e-6
    }

    pub fn power(&self) -> f64 {
        self.voltage * self.current
    }
}

impl Situation for Circuit {
    fn describe(&self) -> String {
        format!(
            "V={:.4} I={:.4} R={:.4} P={:.4}",
            self.voltage,
            self.current,
            self.resistance,
            self.power()
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitAction {
    SetVoltage(f64),
    SetResistance(f64),
}

impl Action for CircuitAction {
    type Sit = Circuit;
    fn describe(&self) -> String {
        format!("{:?}", self)
    }
}

struct OhmsLaw;
impl Precondition<CircuitAction> for OhmsLaw {
    fn check(&self, c: &Circuit, a: &CircuitAction) -> PreconditionResult {
        let next = apply_circuit(c, a).unwrap_or_else(|_| c.clone());
        if next.ohms_law_holds() {
            PreconditionResult::satisfied("ohms_law", "V=IR holds")
        } else {
            PreconditionResult::violated("ohms_law", "V≠IR", &c.describe(), &a.describe())
        }
    }
    fn describe(&self) -> &str {
        "V = IR"
    }
}

struct PositiveR;
impl Precondition<CircuitAction> for PositiveR {
    fn check(&self, c: &Circuit, a: &CircuitAction) -> PreconditionResult {
        if let CircuitAction::SetResistance(r) = a
            && *r <= 0.0
        {
            return PreconditionResult::violated(
                "positive_r",
                "R must be > 0",
                &c.describe(),
                &a.describe(),
            );
        }
        PreconditionResult::satisfied("positive_r", "R>0")
    }
    fn describe(&self) -> &str {
        "R > 0"
    }
}

fn apply_circuit(c: &Circuit, a: &CircuitAction) -> Result<Circuit, String> {
    Ok(match a {
        CircuitAction::SetVoltage(v) => Circuit {
            voltage: *v,
            current: v / c.resistance,
            resistance: c.resistance,
        },
        CircuitAction::SetResistance(r) => {
            if *r > 0.0 {
                Circuit {
                    voltage: c.voltage,
                    current: c.voltage / r,
                    resistance: *r,
                }
            } else {
                c.clone()
            }
        }
    })
}

pub fn new_circuit(voltage: f64, resistance: f64) -> Result<Engine<CircuitAction>, &'static str> {
    let c = Circuit::from_vr(voltage, resistance)?;
    Ok(Engine::new(
        c,
        vec![Box::new(PositiveR), Box::new(OhmsLaw)],
        apply_circuit,
    ))
}

pub fn coulomb_force(q1: f64, q2: f64, r: f64) -> f64 {
    K_E * q1 * q2 / (r * r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_ohms_law() {
        let e = new_circuit(12.0, 4.0).unwrap();
        assert!((e.situation().current - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_change_voltage() {
        let e = new_circuit(12.0, 4.0)
            .unwrap()
            .next(CircuitAction::SetVoltage(24.0))
            .unwrap();
        assert!((e.situation().current - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_zero_r_blocked() {
        assert!(
            new_circuit(12.0, 4.0)
                .unwrap()
                .next(CircuitAction::SetResistance(0.0))
                .is_err()
        );
    }

    proptest! {
        #[test]
        fn prop_ohms_always(v in -100.0..100.0f64, r in 0.01..1000.0f64) {
            prop_assert!(new_circuit(v, r).unwrap().situation().ohms_law_holds());
        }

        #[test]
        fn prop_ohms_after_change(v1 in -100.0..100.0f64, r in 0.01..1000.0f64, v2 in -100.0..100.0f64) {
            let e = new_circuit(v1, r).unwrap().next(CircuitAction::SetVoltage(v2)).unwrap();
            prop_assert!(e.situation().ohms_law_holds());
        }

        #[test]
        fn prop_coulomb_symmetric(q1 in 1e-10..1e-6f64, q2 in 1e-10..1e-6f64, r in 0.01..1.0f64) {
            let scale = coulomb_force(q1, q2, r).abs().max(1e-30);
            prop_assert!((coulomb_force(q1, q2, r) - coulomb_force(q2, q1, r)).abs() / scale < 1e-10);
        }
    }
}
