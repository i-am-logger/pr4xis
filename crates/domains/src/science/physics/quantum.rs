/// Quantum mechanics as an ontology:
/// - Situation: a quantum particle (position uncertainty, momentum uncertainty)
/// - Axiom: Heisenberg ΔxΔp ≥ ℏ/2 enforced
/// - Actions: measure position (reduces Δx, increases Δp), measure momentum
use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

pub const H: f64 = 6.626e-34;
pub const HBAR: f64 = 1.055e-34;

#[derive(Debug, Clone, PartialEq)]
pub struct QuantumParticle {
    pub delta_x: f64, // position uncertainty
    pub delta_p: f64, // momentum uncertainty
}

impl QuantumParticle {
    pub fn new(delta_x: f64, delta_p: f64) -> Result<Self, &'static str> {
        if delta_x <= 0.0 || delta_p <= 0.0 {
            return Err("uncertainties must be positive");
        }
        if delta_x * delta_p < HBAR / 2.0 - 1e-40 {
            return Err("violates Heisenberg uncertainty principle");
        }
        Ok(Self { delta_x, delta_p })
    }

    /// Minimum uncertainty state: ΔxΔp = ℏ/2.
    pub fn minimum_uncertainty() -> Self {
        let dx = (HBAR / 2.0).sqrt();
        Self {
            delta_x: dx,
            delta_p: dx,
        }
    }

    pub fn uncertainty_product(&self) -> f64 {
        self.delta_x * self.delta_p
    }
    pub fn heisenberg_holds(&self) -> bool {
        self.uncertainty_product() >= HBAR / 2.0 - 1e-40
    }
}

impl Situation for QuantumParticle {
    fn describe(&self) -> String {
        format!(
            "Δx={:.4e} Δp={:.4e} ΔxΔp={:.4e} ≥ ℏ/2={:.4e}",
            self.delta_x,
            self.delta_p,
            self.uncertainty_product(),
            HBAR / 2.0
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantumAction {
    /// Measure position: reduce Δx to new_dx (Δp increases to maintain ΔxΔp ≥ ℏ/2).
    MeasurePosition { new_delta_x: f64 },
    /// Measure momentum: reduce Δp to new_dp (Δx increases).
    MeasureMomentum { new_delta_p: f64 },
}

impl Action for QuantumAction {
    type Sit = QuantumParticle;
    fn describe(&self) -> String {
        match self {
            QuantumAction::MeasurePosition { new_delta_x } => {
                format!("measure position (Δx→{:.4e})", new_delta_x)
            }
            QuantumAction::MeasureMomentum { new_delta_p } => {
                format!("measure momentum (Δp→{:.4e})", new_delta_p)
            }
        }
    }
}

/// Heisenberg uncertainty principle: ΔxΔp ≥ ℏ/2.
struct HeisenbergUncertainty;
impl Precondition<QuantumAction> for HeisenbergUncertainty {
    fn check(&self, p: &QuantumParticle, a: &QuantumAction) -> PreconditionResult {
        let next = apply_quantum(p, a).unwrap_or_else(|_| p.clone());
        if next.heisenberg_holds() {
            PreconditionResult::satisfied(
                "heisenberg",
                &format!(
                    "ΔxΔp={:.4e} ≥ ℏ/2={:.4e}",
                    next.uncertainty_product(),
                    HBAR / 2.0
                ),
            )
        } else {
            PreconditionResult::violated(
                "heisenberg",
                &format!(
                    "ΔxΔp={:.4e} < ℏ/2={:.4e}: uncertainty principle violated",
                    next.uncertainty_product(),
                    HBAR / 2.0
                ),
                &p.describe(),
                &a.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "ΔxΔp ≥ ℏ/2"
    }
}

struct PositiveUncertainty;
impl Precondition<QuantumAction> for PositiveUncertainty {
    fn check(&self, p: &QuantumParticle, a: &QuantumAction) -> PreconditionResult {
        let valid = match a {
            QuantumAction::MeasurePosition { new_delta_x } => *new_delta_x > 0.0,
            QuantumAction::MeasureMomentum { new_delta_p } => *new_delta_p > 0.0,
        };
        if valid {
            PreconditionResult::satisfied("positive_uncertainty", "uncertainty > 0")
        } else {
            PreconditionResult::violated(
                "positive_uncertainty",
                "uncertainty must be positive",
                &p.describe(),
                &a.describe(),
            )
        }
    }
    fn describe(&self) -> &str {
        "uncertainties must be positive"
    }
}

fn apply_quantum(p: &QuantumParticle, a: &QuantumAction) -> Result<QuantumParticle, String> {
    Ok(match a {
        QuantumAction::MeasurePosition { new_delta_x } => {
            // Reducing Δx increases Δp to maintain ΔxΔp ≥ ℏ/2
            let min_dp = HBAR / (2.0 * new_delta_x);
            QuantumParticle {
                delta_x: *new_delta_x,
                delta_p: min_dp.max(p.delta_p),
            }
        }
        QuantumAction::MeasureMomentum { new_delta_p } => {
            let min_dx = HBAR / (2.0 * new_delta_p);
            QuantumParticle {
                delta_x: min_dx.max(p.delta_x),
                delta_p: *new_delta_p,
            }
        }
    })
}

pub fn new_particle(delta_x: f64, delta_p: f64) -> Result<Engine<QuantumAction>, &'static str> {
    let p = QuantumParticle::new(delta_x, delta_p)?;
    Ok(Engine::new(
        p,
        vec![
            Box::new(PositiveUncertainty),
            Box::new(HeisenbergUncertainty),
        ],
        apply_quantum,
    ))
}

pub fn new_minimum_uncertainty() -> Engine<QuantumAction> {
    Engine::new(
        QuantumParticle::minimum_uncertainty(),
        vec![
            Box::new(PositiveUncertainty),
            Box::new(HeisenbergUncertainty),
        ],
        apply_quantum,
    )
}

/// Photon energy: E = hf.
pub fn photon_energy(frequency: f64) -> f64 {
    H * frequency
}

/// De Broglie wavelength: λ = h/p.
pub fn de_broglie_wavelength(momentum: f64) -> f64 {
    H / momentum
}

/// Hydrogen energy levels: E_n = -13.6/n² eV.
pub fn hydrogen_energy_level(n: u32) -> f64 {
    -13.6 / (n as f64 * n as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_minimum_uncertainty() {
        let p = QuantumParticle::minimum_uncertainty();
        assert!(p.heisenberg_holds());
        assert!((p.uncertainty_product() - HBAR / 2.0).abs() < 1e-40);
    }

    #[test]
    fn test_measure_position_increases_momentum_uncertainty() {
        let e = new_minimum_uncertainty()
            .next(QuantumAction::MeasurePosition { new_delta_x: 1e-20 })
            .unwrap();
        // Smaller Δx → larger Δp
        assert!(e.situation().delta_p > QuantumParticle::minimum_uncertainty().delta_p);
        assert!(e.situation().heisenberg_holds());
    }

    #[test]
    fn test_zero_uncertainty_blocked() {
        assert!(
            new_minimum_uncertainty()
                .next(QuantumAction::MeasurePosition { new_delta_x: 0.0 })
                .is_err()
        );
    }

    #[test]
    fn test_hydrogen_levels() {
        assert!((hydrogen_energy_level(1) - (-13.6)).abs() < 1e-10);
        assert!(hydrogen_energy_level(2) > hydrogen_energy_level(1)); // less negative
    }

    proptest! {
        /// Heisenberg always holds after any measurement
        #[test]
        fn prop_heisenberg_holds(dx in 1e-35..1e-20f64) {
            let e = new_minimum_uncertainty()
                .next(QuantumAction::MeasurePosition { new_delta_x: dx }).unwrap();
            prop_assert!(e.situation().heisenberg_holds());
        }

        /// More precise position → less precise momentum
        #[test]
        fn prop_position_momentum_tradeoff(dx in 1e-35..1e-25f64) {
            let e = new_minimum_uncertainty()
                .next(QuantumAction::MeasurePosition { new_delta_x: dx }).unwrap();
            let min_dp = HBAR / (2.0 * dx);
            prop_assert!(e.situation().delta_p >= min_dp - 1e-50);
        }

        /// E = hf: proportional
        #[test]
        fn prop_photon_proportional(f in 1e9..1e15f64) {
            let e1 = photon_energy(f);
            let e2 = photon_energy(2.0 * f);
            prop_assert!((e2 - 2.0 * e1).abs() < 1e-30);
        }

        /// Hydrogen levels always negative
        #[test]
        fn prop_hydrogen_negative(n in 1..100u32) {
            prop_assert!(hydrogen_energy_level(n) < 0.0);
        }

        /// Higher n → less negative
        #[test]
        fn prop_hydrogen_monotonic(n in 1..99u32) {
            prop_assert!(hydrogen_energy_level(n + 1) > hydrogen_energy_level(n));
        }
    }
}
