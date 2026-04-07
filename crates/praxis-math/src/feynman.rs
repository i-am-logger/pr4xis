/// Feynman path integral as an ontology:
/// - Situation: a quantum system with paths from A to B
/// - Axiom: probability amplitude = sum of e^(iS/ℏ) over all paths
/// - Axiom: total probability is normalized (|amplitude|² sums to 1)
/// - Actions: add path, compute amplitude
///
/// Simplified: discrete paths on a grid, each path has a phase.
use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

/// A path from point A to point B through intermediate points.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub points: Vec<f64>, // positions at each time step
    pub action: f64,      // classical action S (integral of Lagrangian)
}

impl Path {
    /// Compute classical action for a free particle: S = ½m∫v²dt.
    pub fn free_particle_action(positions: &[f64], mass: f64, dt: f64) -> f64 {
        let mut s = 0.0;
        for w in positions.windows(2) {
            let v = (w[1] - w[0]) / dt;
            s += 0.5 * mass * v * v * dt;
        }
        s
    }
}

/// A quantum amplitude: complex number as (real, imag).
#[derive(Debug, Clone, PartialEq)]
pub struct Amplitude {
    pub real: f64,
    pub imag: f64,
}

impl Amplitude {
    pub fn zero() -> Self {
        Self {
            real: 0.0,
            imag: 0.0,
        }
    }

    pub fn from_phase(phase: f64) -> Self {
        Self {
            real: phase.cos(),
            imag: phase.sin(),
        }
    }

    pub fn add(&self, other: &Amplitude) -> Amplitude {
        Amplitude {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }

    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }
}

/// The path integral state: accumulates paths and their amplitudes.
#[derive(Debug, Clone, PartialEq)]
pub struct PathIntegral {
    pub paths: Vec<Path>,
    pub hbar: f64, // reduced Planck constant
    pub total_amplitude: Amplitude,
}

impl PathIntegral {
    pub fn new(hbar: f64) -> Self {
        Self {
            paths: vec![],
            hbar,
            total_amplitude: Amplitude::zero(),
        }
    }

    /// Probability = |amplitude|².
    pub fn probability(&self) -> f64 {
        self.total_amplitude.magnitude_squared()
    }
}

impl Situation for PathIntegral {
    fn describe(&self) -> String {
        format!(
            "{} paths, amplitude=({:.6},{:.6}), P={:.6}",
            self.paths.len(),
            self.total_amplitude.real,
            self.total_amplitude.imag,
            self.probability()
        )
    }
    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FeynmanAction {
    /// Add a path with given action S. Phase = S/ℏ.
    AddPath { path: Path },
    /// Reset: clear all paths.
    Reset,
}

impl Action for FeynmanAction {
    type Sit = PathIntegral;
    fn describe(&self) -> String {
        match self {
            FeynmanAction::AddPath { path } => format!(
                "add path (S={:.6}, {} points)",
                path.action,
                path.points.len()
            ),
            FeynmanAction::Reset => "reset".into(),
        }
    }
}

/// Axiom: each path contributes e^(iS/ℏ) to the amplitude.
struct PhaseConsistency;
impl Precondition<FeynmanAction> for PhaseConsistency {
    fn check(&self, pi: &PathIntegral, action: &FeynmanAction) -> PreconditionResult {
        if let FeynmanAction::AddPath { path } = action {
            if pi.hbar <= 0.0 {
                return PreconditionResult::violated(
                    "phase_consistency",
                    "ℏ must be positive",
                    &pi.describe(),
                    &action.describe(),
                );
            }
            let phase = path.action / pi.hbar;
            if phase.is_nan() || phase.is_infinite() {
                return PreconditionResult::violated(
                    "phase_consistency",
                    "phase is NaN/infinite",
                    &pi.describe(),
                    &action.describe(),
                );
            }
            PreconditionResult::satisfied("phase_consistency", &format!("phase S/ℏ={:.6}", phase))
        } else {
            PreconditionResult::satisfied("phase_consistency", "no path added")
        }
    }
    fn describe(&self) -> &str {
        "e^(iS/ℏ) must be well-defined"
    }
}

fn apply_feynman(pi: &PathIntegral, action: &FeynmanAction) -> PathIntegral {
    let mut next = pi.clone();
    match action {
        FeynmanAction::AddPath { path } => {
            let phase = path.action / pi.hbar;
            let amp = Amplitude::from_phase(phase);
            next.total_amplitude = next.total_amplitude.add(&amp);
            next.paths.push(path.clone());
        }
        FeynmanAction::Reset => {
            next.paths.clear();
            next.total_amplitude = Amplitude::zero();
        }
    }
    next
}

pub fn new_path_integral(hbar: f64) -> Engine<FeynmanAction> {
    Engine::new(
        PathIntegral::new(hbar),
        vec![Box::new(PhaseConsistency)],
        apply_feynman,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_single_path() {
        let path = Path {
            points: vec![0.0, 1.0],
            action: 1.0,
        };
        let e = new_path_integral(1.0)
            .next(FeynmanAction::AddPath { path })
            .unwrap();
        assert!(e.situation().probability() > 0.0);
        assert_eq!(e.situation().paths.len(), 1);
    }

    #[test]
    fn test_two_paths_interfere() {
        // Two paths with action difference = π: destructive interference
        let p1 = Path {
            points: vec![0.0, 1.0],
            action: 0.0,
        };
        let p2 = Path {
            points: vec![0.0, 1.0],
            action: std::f64::consts::PI,
        };
        let e = new_path_integral(1.0)
            .next(FeynmanAction::AddPath { path: p1 })
            .unwrap()
            .next(FeynmanAction::AddPath { path: p2 })
            .unwrap();
        // cos(0) + cos(π) = 1 + (-1) = 0
        assert!(e.situation().total_amplitude.real.abs() < 1e-10);
    }

    #[test]
    fn test_constructive_interference() {
        // Two paths with same action: constructive interference
        let p1 = Path {
            points: vec![0.0, 1.0],
            action: 0.0,
        };
        let p2 = Path {
            points: vec![0.0, 1.0],
            action: 0.0,
        };
        let e = new_path_integral(1.0)
            .next(FeynmanAction::AddPath { path: p1 })
            .unwrap()
            .next(FeynmanAction::AddPath { path: p2 })
            .unwrap();
        // cos(0) + cos(0) = 2, |amp|² = 4
        assert!((e.situation().probability() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_free_particle_action() {
        let positions = vec![0.0, 1.0, 2.0];
        let s = Path::free_particle_action(&positions, 1.0, 1.0);
        // v=1 at each step, S = ½(1)(1²)(1) + ½(1)(1²)(1) = 1.0
        assert!((s - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let p = Path {
            points: vec![0.0],
            action: 1.0,
        };
        let e = new_path_integral(1.0)
            .next(FeynmanAction::AddPath { path: p })
            .unwrap()
            .next(FeynmanAction::Reset)
            .unwrap();
        assert_eq!(e.situation().paths.len(), 0);
        assert!((e.situation().probability() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_amplitude_from_phase() {
        let a = Amplitude::from_phase(0.0);
        assert!((a.real - 1.0).abs() < 1e-10);
        assert!(a.imag.abs() < 1e-10);

        let a = Amplitude::from_phase(std::f64::consts::PI / 2.0);
        assert!(a.real.abs() < 1e-10);
        assert!((a.imag - 1.0).abs() < 1e-10);
    }

    proptest! {
        /// |e^(iθ)|² = 1 for any phase
        #[test]
        fn prop_unit_amplitude(phase in -100.0..100.0f64) {
            let a = Amplitude::from_phase(phase);
            prop_assert!((a.magnitude_squared() - 1.0).abs() < 1e-10);
        }

        /// e^(iθ) + e^(-iθ) = 2cos(θ) (Euler's formula)
        #[test]
        fn prop_euler_formula(theta in -10.0..10.0f64) {
            let a1 = Amplitude::from_phase(theta);
            let a2 = Amplitude::from_phase(-theta);
            let sum = a1.add(&a2);
            prop_assert!((sum.real - 2.0 * theta.cos()).abs() < 1e-10);
            prop_assert!(sum.imag.abs() < 1e-10); // imaginary parts cancel
        }

        /// Probability is always non-negative
        #[test]
        fn prop_probability_nonneg(actions in proptest::collection::vec(-10.0..10.0f64, 1..10)) {
            let mut e = new_path_integral(1.0);
            for s in actions {
                e = e.next(FeynmanAction::AddPath {
                    path: Path { points: vec![0.0], action: s }
                }).unwrap();
            }
            prop_assert!(e.situation().probability() >= 0.0);
        }

        /// Classical action for stationary particle is 0
        #[test]
        fn prop_stationary_zero_action(mass in 0.1..100.0f64, n in 2..10usize) {
            let positions = vec![0.0; n];
            let s = Path::free_particle_action(&positions, mass, 1.0);
            prop_assert!(s.abs() < 1e-10);
        }

        /// Free particle action scales with mass
        #[test]
        fn prop_action_scales_with_mass(m1 in 0.1..10.0f64, m2 in 0.1..10.0f64) {
            let positions = vec![0.0, 1.0, 2.0];
            let s1 = Path::free_particle_action(&positions, m1, 1.0);
            let s2 = Path::free_particle_action(&positions, m2, 1.0);
            let ratio = s1 / s2;
            let expected = m1 / m2;
            prop_assert!((ratio - expected).abs() < 1e-6);
        }
    }
}
