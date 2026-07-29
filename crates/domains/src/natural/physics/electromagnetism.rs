#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Electromagnetism as an ontology:
/// - Situation: Circuit (V, I, R)
/// - Axiom: Ohm's law V=IR enforced on every change
/// - Actions: set voltage, set resistance (current derived)
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::value::Quantity;

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

    /// Electrical power P = VI. Ohm (1827).
    pub fn power(&self) -> Quantity {
        Quantity::new(self.voltage * self.current, Dimension::POWER)
    }
}

impl Situation for Circuit {}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitAction {
    SetVoltage(f64),
    SetResistance(f64),
}

impl Action for CircuitAction {
    type Sit = Circuit;
}

fn em_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Ohm (1827) Die galvanische Kette, mathematisch bearbeitet; Coulomb (1785) Premier mémoire sur l'électricité et le magnétisme",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

struct OhmsLaw;
impl Precondition<CircuitAction> for OhmsLaw {
    fn check(&self, c: &Circuit, action: &CircuitAction) -> Verdict {
        let meta = em_meta("OhmsLaw", "V = IR");
        let next = apply_circuit_inner(c, action).unwrap_or_else(|_| c.clone());
        if next.ohms_law_holds() {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

struct PositiveR;
impl Precondition<CircuitAction> for PositiveR {
    fn check(&self, _c: &Circuit, action: &CircuitAction) -> Verdict {
        let meta = em_meta("PositiveR", "R > 0");
        if let CircuitAction::SetResistance(r) = action
            && *r <= 0.0
        {
            return Err(Box::new(SimpleCounterexample::new(meta)));
        }
        Ok(Box::new(SimpleProof::new(meta)))
    }
}

fn apply_circuit_inner(c: &Circuit, a: &CircuitAction) -> Result<Circuit, &'static str> {
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

fn apply_circuit(c: &Circuit, a: &CircuitAction) -> Result<Circuit, Box<dyn Counterexample>> {
    apply_circuit_inner(c, a).map_err(|_| {
        let meta = em_meta("ApplyFailed", "circuit transformation failed");
        Box::new(SimpleCounterexample::new(meta)) as Box<dyn Counterexample>
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

/// Coulomb's law: F = k·q1·q2/r². Coulomb (1785).
pub fn coulomb_force(q1: f64, q2: f64, r: f64) -> Quantity {
    Quantity::new(K_E * q1 * q2 / (r * r), Dimension::FORCE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_ohms_law() {
        let e = new_circuit(12.0, 4.0).unwrap();
        assert!((e.situation().current - 3.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_change_voltage() {
        let e = new_circuit(12.0, 4.0)
            .unwrap()
            .next(CircuitAction::SetVoltage(24.0))
            .unwrap();
        assert!((e.situation().current - 6.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Honest)]
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
            let scale = coulomb_force(q1, q2, r).value.abs().max(1e-30);
            prop_assert!((coulomb_force(q1, q2, r).value - coulomb_force(q2, q1, r).value).abs() / scale < 1e-10);
        }
    }

    pr4xis::register_praxis_value!(prop_ohms_always, Verifiable);
    pr4xis::register_praxis_value!(prop_ohms_after_change, Verifiable);
    pr4xis::register_praxis_value!(prop_coulomb_symmetric, Verifiable);
}
