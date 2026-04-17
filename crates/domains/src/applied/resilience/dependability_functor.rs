//! Terminal functor Resilience → Dependability.
//!
//! Every resilience pattern (CircuitBreaker, Retry, Supervisor, Microreboot, …)
//! factors through `Dependability::FaultTolerance` — they are all instances of
//! fault-tolerance means. The natural structure-preserving map sends every
//! resilience concept to `FaultTolerance` and every resilience morphism to the
//! identity morphism on `FaultTolerance`.
//!
//! Expressed via the reusable [`pr4xis::category::TerminalFunctor`] helper
//! (landed in #131): the `FaultToleranceTarget` marker declares which target
//! object; `ResilienceToFaultTolerance` is the type alias wiring it to
//! `ResilienceCategory`.
//!
//! # Laws
//!
//! Trivial: every morphism maps to `id_FaultTolerance`, every composite
//! maps to `id_FaultTolerance ∘ id_FaultTolerance = id_FaultTolerance`.
//!
//! # Ontological content
//!
//! "Every resilience pattern is a fault-tolerance means" (Avizienis et al.
//! 2004 §5.2). Non-trivial sub-structure (e.g., `CircuitBreaker →
//! StabilityMeans ⊂ FaultTolerance`) would require enriching Dependability's
//! means hierarchy — a content decision separate from the functor laws. See
//! `docs/research/kinded-functor-failures.md` for context.

use pr4xis::category::{TerminalFunctor, TerminalTarget};

use crate::applied::dependability::ontology::{DependabilityCategory, DependabilityConcept};
use crate::applied::resilience::ontology::ResilienceCategory;

/// Marker selecting `FaultTolerance` as the single target of the terminal
/// functor.
pub struct FaultToleranceTarget;

impl TerminalTarget for FaultToleranceTarget {
    type Category = DependabilityCategory;
    fn target() -> DependabilityConcept {
        DependabilityConcept::FaultTolerance
    }
}

/// Terminal functor: every Resilience concept ↦ Dependability::FaultTolerance.
pub type ResilienceToFaultTolerance = TerminalFunctor<ResilienceCategory, FaultToleranceTarget>;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn terminal_functor_satisfies_laws() {
        check_functor_laws::<ResilienceToFaultTolerance>().unwrap();
    }
}
