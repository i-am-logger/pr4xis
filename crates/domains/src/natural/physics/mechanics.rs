#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Newtonian mechanics as an ontology:
/// - Situation: a particle (mass, position, velocity)
/// - Axioms: F=ma, mass conservation
/// - Actions: apply force, free fall
/// - Enforcement: Newton's laws are preconditions
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

pub const G: f64 = 6.674e-11;
pub const EARTH_G: f64 = 9.81;

#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub mass: f64,
    pub position: f64,
    pub velocity: f64,
}

impl Particle {
    pub fn new(mass: f64) -> Result<Self, &'static str> {
        if mass <= 0.0 {
            return Err("mass must be positive");
        }
        Ok(Self {
            mass,
            position: 0.0,
            velocity: 0.0,
        })
    }

    pub fn with_velocity(mass: f64, velocity: f64) -> Result<Self, &'static str> {
        if mass <= 0.0 {
            return Err("mass must be positive");
        }
        Ok(Self {
            mass,
            position: 0.0,
            velocity,
        })
    }

    pub fn momentum(&self) -> f64 {
        self.mass * self.velocity
    }
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity * self.velocity
    }
}

impl Situation for Particle {}

fn mech_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        citation: Citation::parse_static(
            "Newton (1687) Philosophiae Naturalis Principia Mathematica",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MechanicsAction {
    ApplyForce { force: f64, duration: f64 },
    FreeFall { duration: f64 },
}

impl Action for MechanicsAction {
    type Sit = Particle;
}

struct MassConservation;
impl Precondition<MechanicsAction> for MassConservation {
    fn check(&self, p: &Particle, action: &MechanicsAction) -> Verdict {
        let meta = mech_meta("MassConservation", "mass must be conserved");
        let next = apply_inner(p, action);
        if (next.mass - p.mass).abs() < 1e-10 {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

struct PositiveDuration;
impl Precondition<MechanicsAction> for PositiveDuration {
    fn check(&self, _p: &Particle, action: &MechanicsAction) -> Verdict {
        let meta = mech_meta("PositiveDuration", "time must move forward");
        let dt = match action {
            MechanicsAction::ApplyForce { duration, .. } => *duration,
            MechanicsAction::FreeFall { duration } => *duration,
        };
        if dt >= 0.0 {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

fn apply_inner(p: &Particle, action: &MechanicsAction) -> Particle {
    let mut next = p.clone();
    match action {
        MechanicsAction::ApplyForce { force, duration } => {
            let a = force / p.mass;
            next.position += p.velocity * duration + 0.5 * a * duration * duration;
            next.velocity += a * duration;
        }
        MechanicsAction::FreeFall { duration } => {
            next.position += p.velocity * duration + 0.5 * EARTH_G * duration * duration;
            next.velocity += EARTH_G * duration;
        }
    }
    next
}

fn apply(p: &Particle, action: &MechanicsAction) -> Result<Particle, Box<dyn Counterexample>> {
    Ok(apply_inner(p, action))
}

pub fn new_particle(mass: f64) -> Result<Engine<MechanicsAction>, &'static str> {
    let p = Particle::new(mass)?;
    Ok(Engine::new(
        p,
        vec![Box::new(MassConservation), Box::new(PositiveDuration)],
        apply,
    ))
}

pub fn new_particle_with_velocity(
    mass: f64,
    velocity: f64,
) -> Result<Engine<MechanicsAction>, &'static str> {
    let p = Particle::with_velocity(mass, velocity)?;
    Ok(Engine::new(
        p,
        vec![Box::new(MassConservation), Box::new(PositiveDuration)],
        apply,
    ))
}

pub fn gravity(m1: f64, m2: f64, r: f64) -> f64 {
    G * m1 * m2 / (r * r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_f_equals_ma() {
        let e = new_particle(10.0)
            .unwrap()
            .next(MechanicsAction::ApplyForce {
                force: 100.0,
                duration: 1.0,
            })
            .unwrap();
        assert!((e.situation().velocity - 10.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn test_free_fall() {
        let e = new_particle(1.0)
            .unwrap()
            .next(MechanicsAction::FreeFall { duration: 2.0 })
            .unwrap();
        assert!((e.situation().velocity - 19.62).abs() < 0.01);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn test_negative_duration_blocked() {
        let e = new_particle(1.0).unwrap();
        assert!(
            e.next(MechanicsAction::ApplyForce {
                force: 10.0,
                duration: -1.0
            })
            .is_err()
        );
    }

    proptest! {
        #[test]
        fn prop_fma(mass in 0.1..100.0f64, force in -1000.0..1000.0f64, dt in 0.01..10.0f64) {
            let e = new_particle(mass).unwrap()
                .next(MechanicsAction::ApplyForce { force, duration: dt }).unwrap();
            prop_assert!((e.situation().velocity - (force / mass) * dt).abs() < 1e-6);
        }

        #[test]
        fn prop_mass_conserved(mass in 0.1..100.0f64, force in -1000.0..1000.0f64, dt in 0.01..10.0f64) {
            let e = new_particle(mass).unwrap()
                .next(MechanicsAction::ApplyForce { force, duration: dt }).unwrap();
            prop_assert_eq!(e.situation().mass, mass);
        }

        #[test]
        fn prop_ke_nonneg(mass in 0.1..100.0f64, force in -1000.0..1000.0f64, dt in 0.01..10.0f64) {
            let e = new_particle(mass).unwrap()
                .next(MechanicsAction::ApplyForce { force, duration: dt }).unwrap();
            prop_assert!(e.situation().kinetic_energy() >= 0.0);
        }

        #[test]
        fn prop_zero_force_preserves_v(mass in 0.1..100.0f64, v in -100.0..100.0f64, dt in 0.01..10.0f64) {
            let e = new_particle_with_velocity(mass, v).unwrap()
                .next(MechanicsAction::ApplyForce { force: 0.0, duration: dt }).unwrap();
            prop_assert!((e.situation().velocity - v).abs() < 1e-10);
        }

        #[test]
        fn prop_gravity_symmetric(m1 in 1.0..1e10f64, m2 in 1.0..1e10f64, r in 1.0..1e6f64) {
            let f12 = gravity(m1, m2, r);
            let f21 = gravity(m2, m1, r);
            let scale = f12.abs().max(1e-30);
            prop_assert!((f12 - f21).abs() / scale < 1e-10);
        }
    }

    pr4xis::register_praxis_value!(prop_fma, Verifiable);
    pr4xis::register_praxis_value!(prop_mass_conserved, Verifiable);
    pr4xis::register_praxis_value!(prop_ke_nonneg, Verifiable);
    pr4xis::register_praxis_value!(prop_zero_force_preserves_v, Verifiable);
    pr4xis::register_praxis_value!(prop_gravity_symmetric, Verifiable);
}
