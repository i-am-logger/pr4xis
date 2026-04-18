#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Newtonian mechanics as an ontology:
/// - Situation: a particle (mass, position, velocity)
/// - Axioms: F=ma, mass conservation
/// - Actions: apply force, free fall
/// - Enforcement: Newton's laws are preconditions
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

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

impl Situation for Particle {
    fn describe(&self) -> String {
        format!(
            "m={:.4} pos={:.4} v={:.4} p={:.4} KE={:.4}",
            self.mass,
            self.position,
            self.velocity,
            self.momentum(),
            self.kinetic_energy()
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MechanicsAction {
    ApplyForce { force: f64, duration: f64 },
    FreeFall { duration: f64 },
}

impl Action for MechanicsAction {
    type Sit = Particle;
    fn describe(&self) -> String {
        match self {
            MechanicsAction::ApplyForce { force, duration } => {
                format!("F={:.4}N for {:.4}s", force, duration)
            }
            MechanicsAction::FreeFall { duration } => format!("free fall {:.4}s", duration),
        }
    }
}

struct MassConservation;
impl Precondition<MechanicsAction> for MassConservation {
    fn check(&self, p: &Particle, action: &MechanicsAction) -> PreconditionResult {
        let next = apply(p, action).unwrap_or_else(|_| p.clone());
        if (next.mass - p.mass).abs() < 1e-10 {
            PreconditionResult::satisfied("mass_conservation", "mass preserved")
        } else {
            PreconditionResult::violated(
                "mass_conservation",
                "mass changed",
                &p.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "mass must be conserved"
    }
}

struct PositiveDuration;
impl Precondition<MechanicsAction> for PositiveDuration {
    fn check(&self, p: &Particle, action: &MechanicsAction) -> PreconditionResult {
        let dt = match action {
            MechanicsAction::ApplyForce { duration, .. } => *duration,
            MechanicsAction::FreeFall { duration } => *duration,
        };
        if dt >= 0.0 {
            PreconditionResult::satisfied("positive_duration", "time moves forward")
        } else {
            PreconditionResult::violated(
                "positive_duration",
                "duration must be non-negative",
                &p.describe(),
                &action.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "time must move forward"
    }
}

fn apply(p: &Particle, action: &MechanicsAction) -> Result<Particle, String> {
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
    Ok(next)
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

    #[test]
    fn test_free_fall() {
        let e = new_particle(1.0)
            .unwrap()
            .next(MechanicsAction::FreeFall { duration: 2.0 })
            .unwrap();
        assert!((e.situation().velocity - 19.62).abs() < 0.01);
    }

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
}
