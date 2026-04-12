/// Special relativity as an ontology:
/// - Situation: Body (rest mass, velocity)
/// - Axiom: v < c enforced, rest mass invariant
/// - Actions: accelerate, set velocity
/// - Derived: Lorentz factor, time dilation, length contraction, E=mc²
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

pub const C: f64 = 299_792_458.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub rest_mass: f64,
    pub velocity: f64,
}

impl Body {
    pub fn new(rest_mass: f64) -> Result<Self, &'static str> {
        if rest_mass <= 0.0 {
            return Err("rest mass must be positive");
        }
        Ok(Self {
            rest_mass,
            velocity: 0.0,
        })
    }

    pub fn lorentz_factor(&self) -> f64 {
        let beta = self.velocity / C;
        1.0 / (1.0 - beta * beta).sqrt()
    }

    pub fn rest_energy(&self) -> f64 {
        self.rest_mass * C * C
    }
    pub fn total_energy(&self) -> f64 {
        self.lorentz_factor() * self.rest_mass * C * C
    }
    pub fn kinetic_energy(&self) -> f64 {
        (self.lorentz_factor() - 1.0) * self.rest_mass * C * C
    }
    pub fn momentum(&self) -> f64 {
        self.lorentz_factor() * self.rest_mass * self.velocity
    }
    pub fn time_dilation(&self, proper_time: f64) -> f64 {
        proper_time * self.lorentz_factor()
    }
    pub fn length_contraction(&self, proper_length: f64) -> f64 {
        proper_length / self.lorentz_factor()
    }
}

impl Situation for Body {
    fn describe(&self) -> String {
        format!(
            "m₀={:.4} v={:.4} γ={:.6}",
            self.rest_mass,
            self.velocity,
            self.lorentz_factor()
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelativityAction {
    Accelerate { delta_v: f64 },
    SetVelocity { v: f64 },
}

impl Action for RelativityAction {
    type Sit = Body;
    fn describe(&self) -> String {
        format!("{:?}", self)
    }
}

struct SpeedLimit;
impl Precondition<RelativityAction> for SpeedLimit {
    fn check(&self, body: &Body, a: &RelativityAction) -> PreconditionResult {
        let next = apply_rel(body, a).unwrap_or_else(|_| body.clone());
        if next.velocity.abs() < C {
            PreconditionResult::satisfied(
                "speed_limit",
                &format!("|v|={:.4} < c", next.velocity.abs()),
            )
        } else {
            PreconditionResult::violated(
                "speed_limit",
                "cannot reach speed of light",
                &body.describe(),
                &a.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "v < c"
    }
}

fn apply_rel(body: &Body, a: &RelativityAction) -> Result<Body, String> {
    let mut next = body.clone();
    match a {
        RelativityAction::Accelerate { delta_v } => next.velocity += delta_v,
        RelativityAction::SetVelocity { v } => next.velocity = *v,
    }
    Ok(next)
}

pub fn new_body(rest_mass: f64) -> Result<Engine<RelativityAction>, &'static str> {
    let body = Body::new(rest_mass)?;
    Ok(Engine::new(body, vec![Box::new(SpeedLimit)], apply_rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_emc2() {
        assert!((Body::new(1.0).unwrap().rest_energy() - C * C).abs() < 1.0);
    }

    #[test]
    fn test_speed_limit() {
        assert!(
            new_body(1.0)
                .unwrap()
                .next(RelativityAction::SetVelocity { v: C })
                .is_err()
        );
    }

    #[test]
    fn test_time_dilation() {
        let e = new_body(1.0)
            .unwrap()
            .next(RelativityAction::SetVelocity { v: C * 0.5 })
            .unwrap();
        assert!(e.situation().time_dilation(1.0) > 1.0);
    }

    proptest! {
        #[test]
        fn prop_lorentz_gte_1(v in 0.0..(C * 0.99)) {
            let e = new_body(1.0).unwrap().next(RelativityAction::SetVelocity { v }).unwrap();
            prop_assert!(e.situation().lorentz_factor() >= 1.0);
        }

        #[test]
        fn prop_speed_limit(v in C..(C * 2.0)) {
            let result = new_body(1.0).unwrap().next(RelativityAction::SetVelocity { v });
            prop_assert!(result.is_err());
        }

        #[test]
        fn prop_ke_nonneg(v in 0.0..(C * 0.99)) {
            let e = new_body(1.0).unwrap().next(RelativityAction::SetVelocity { v }).unwrap();
            prop_assert!(e.situation().kinetic_energy() >= -1e-10);
        }

        #[test]
        fn prop_time_dilation_gte(v in 1.0..(C * 0.99)) {
            let e = new_body(1.0).unwrap().next(RelativityAction::SetVelocity { v }).unwrap();
            prop_assert!(e.situation().time_dilation(1.0) >= 1.0);
        }

        #[test]
        fn prop_length_contraction_lte(v in 1.0..(C * 0.99)) {
            let e = new_body(1.0).unwrap().next(RelativityAction::SetVelocity { v }).unwrap();
            prop_assert!(e.situation().length_contraction(1.0) <= 1.0);
        }

        #[test]
        fn prop_emc2_roundtrip(m in 0.001..1000.0f64) {
            let b = Body::new(m).unwrap();
            let e = b.rest_energy();
            let m_back = e / (C * C);
            prop_assert!((m_back - m).abs() < 1e-6);
        }
    }
}
