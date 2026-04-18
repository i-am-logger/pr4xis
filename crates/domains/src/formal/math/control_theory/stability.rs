#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Stability analysis for control systems.
///
/// Lyapunov, A.M. (1892). "The General Problem of the Stability of Motion."
/// Ogata (2010). *Modern Control Engineering* (5th ed.).
///
/// - BIBO stability: bounded input produces bounded output
/// - Lyapunov stability: equilibrium point stability classification
/// - A system is stable if all poles have negative real parts
///
/// Stability classification of an equilibrium point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StabilityClass {
    /// Asymptotically stable: trajectories converge to equilibrium.
    AsymptoticallyStable,
    /// Marginally stable: trajectories stay bounded but don't converge.
    MarginallyStable,
    /// Unstable: trajectories diverge from equilibrium.
    Unstable,
}

/// Check BIBO stability from system poles (real parts).
///
/// A continuous-time LTI system is BIBO stable iff all poles
/// have strictly negative real parts.
pub fn is_bibo_stable(pole_real_parts: &[f64]) -> bool {
    pole_real_parts.iter().all(|&re| re < 0.0)
}

/// Classify stability from pole locations (real parts).
///
/// - All poles have Re < 0 → asymptotically stable
/// - Some poles have Re = 0, none Re > 0 → marginally stable
/// - Any pole has Re > 0 → unstable
pub fn classify_stability(pole_real_parts: &[f64]) -> StabilityClass {
    let has_positive = pole_real_parts.iter().any(|&re| re > 1e-10);
    let has_zero = pole_real_parts.iter().any(|&re| re.abs() <= 1e-10);

    if has_positive {
        StabilityClass::Unstable
    } else if has_zero {
        StabilityClass::MarginallyStable
    } else {
        StabilityClass::AsymptoticallyStable
    }
}

/// Check if a first-order discrete system y[n] = a*y[n-1] + b*x[n] is stable.
///
/// Stable iff |a| < 1.
pub fn is_discrete_first_order_stable(a: f64) -> bool {
    a.abs() < 1.0
}

/// Evaluate a simple Lyapunov candidate V(x) = x^2 and check V_dot < 0.
///
/// For the system dx/dt = -a*x with a > 0:
/// V(x) = x^2, V_dot = 2*x*(-a*x) = -2*a*x^2 < 0 for x != 0.
///
/// Returns true if the system is Lyapunov stable with V(x) = x^2.
pub fn is_lyapunov_stable_linear(a: f64) -> bool {
    // For dx/dt = -a*x, stable iff a > 0
    a > 0.0
}
