//! Resilience ontology — how systems recover from faults.
//!
//! Companion to [Dependability](crate::applied::dependability): where
//! Dependability defines what errors ARE, Resilience defines how we RESPOND.
//!
//! Grounded in four literatures:
//!
//! 1. **Stability patterns** — Nygard, M. (2007). *Release It!: Design and
//!    Deploy Production-Ready Software*. Pragmatic Bookshelf.
//!    Core concepts: `CircuitBreaker` (Closed/Open/HalfOpen), `Bulkhead`,
//!    `Timeout`, `SteadyState`, `FailFast`, `HandshakingProtocol`, `TestHarness`.
//!
//! 2. **Backoff** — Brooker, M. (2015). "Exponential Backoff and Jitter".
//!    AWS Architecture Blog.
//!    Metcalfe, R. & Boggs, D. (1976). "Ethernet: Distributed packet
//!    switching for local computer networks". CACM 19(7). — original
//!    exponential backoff.
//!    Concepts: `Retry`, `ExponentialBackoff`, `FullJitter`, `EqualJitter`,
//!    `DecorrelatedJitter`, `MaxAttempts`, `RetryBudget`.
//!
//! 3. **Supervision** — Armstrong, J. (2003). "Making reliable distributed
//!    systems in the presence of software errors". PhD thesis. — Erlang/OTP
//!    "let it crash" with supervision trees.
//!    Hewitt, C. (1973). "A universal modular ACTOR formalism for AI". IJCAI.
//!    Concepts: `Supervisor`, `SupervisedChild`, `OneForOne`, `OneForAll`,
//!    `RestForOne`, `RestartIntensity`, `LetItCrash`.
//!
//! 4. **Recovery-Oriented Computing** — Patterson, D. et al. (2002).
//!    "Recovery-Oriented Computing (ROC)". UC Berkeley + Stanford.
//!    Concepts: `UndoOperation`, `Microreboot`, `Quarantine`.

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Resilience",
    source: "Nygard (2007); Brooker (2015); Metcalfe & Boggs (1976); Armstrong (2003); Patterson et al. (2002)",
    being: Process,

    concepts: [
        // === Pattern families (parents for classification) ===
        StabilityPattern,
        BackoffStrategy,
        JitterStrategy,
        SupervisionStrategy,
        RecoveryPattern,

        // === Nygard (2007) stability patterns ===
        CircuitBreaker,
        CircuitBreakerClosed,
        CircuitBreakerOpen,
        CircuitBreakerHalfOpen,
        Bulkhead,
        Timeout,
        FailFast,
        SteadyState,
        HandshakingProtocol,
        TestHarness,

        // === Retry + backoff (Brooker 2015, Metcalfe-Boggs 1976) ===
        Retry,
        ExponentialBackoff,
        LinearBackoff,
        ConstantBackoff,
        FullJitter,
        EqualJitter,
        DecorrelatedJitter,
        MaxAttempts,
        RetryBudget,

        // === Supervision (Armstrong 2003 Erlang/OTP) ===
        Supervisor,
        SupervisedChild,
        OneForOne,
        OneForAll,
        RestForOne,
        RestartIntensity,
        RestartPeriod,
        LetItCrash,

        // === Recovery-Oriented Computing (Patterson 2002) ===
        UndoOperation,
        Microreboot,
        Quarantine,

        // === Targets (what resilience protects / acts on) ===
        Service,
        Resource,
        Request,
    ],

    labels: {
        StabilityPattern: ("en", "Stability pattern", "A structural design for preventing cascading failures under stress (Nygard 2007)."),
        BackoffStrategy: ("en", "Backoff strategy", "A function from attempt count to wait duration."),
        JitterStrategy: ("en", "Jitter strategy", "Randomisation added to backoff to avoid thundering-herd synchronisation (Brooker 2015)."),
        SupervisionStrategy: ("en", "Supervision strategy", "A policy for restarting supervised processes after failure (Armstrong 2003)."),
        RecoveryPattern: ("en", "Recovery pattern", "A technique for restoring correct service after a failure (Patterson et al. 2002)."),

        CircuitBreaker: ("en", "Circuit breaker", "Nygard (2007): stops calls to a failing downstream dependency after an error threshold, resuming after a timeout."),
        CircuitBreakerClosed: ("en", "Circuit breaker: closed", "Normal operation — requests flow through."),
        CircuitBreakerOpen: ("en", "Circuit breaker: open", "All requests fail fast without touching the dependency."),
        CircuitBreakerHalfOpen: ("en", "Circuit breaker: half-open", "A single probe request is permitted; success returns to Closed, failure returns to Open."),
        Bulkhead: ("en", "Bulkhead", "Nygard (2007): resource partitioning so failure in one area cannot drain resources from another."),
        Timeout: ("en", "Timeout", "Bounded wait time on a request — converts indefinite hangs into observable timing failures."),
        FailFast: ("en", "Fail fast", "Reject a request as early as possible when preconditions are known to be unmet."),
        SteadyState: ("en", "Steady state", "System behaviour for arbitrary long-running operation without external intervention — log rotation, cache eviction, cleanup."),
        HandshakingProtocol: ("en", "Handshaking protocol", "Client and server negotiate capacity before work begins."),
        TestHarness: ("en", "Test harness", "A controllable environment that simulates adversarial conditions for stability testing."),

        Retry: ("en", "Retry", "Re-invocation of a failed operation."),
        ExponentialBackoff: ("en", "Exponential backoff", "Wait = base × 2^attempt (Metcalfe & Boggs 1976; Brooker 2015)."),
        LinearBackoff: ("en", "Linear backoff", "Wait = base × attempt."),
        ConstantBackoff: ("en", "Constant backoff", "Wait is constant across all attempts."),
        FullJitter: ("en", "Full jitter", "Brooker (2015): wait = random(0, base × 2^attempt). Maximum decorrelation."),
        EqualJitter: ("en", "Equal jitter", "Brooker (2015): wait = half + random(0, half) where half = base × 2^attempt / 2."),
        DecorrelatedJitter: ("en", "Decorrelated jitter", "Brooker (2015): wait = random(base, prev × 3), capped. Smoothest distribution."),
        MaxAttempts: ("en", "Max attempts", "An upper bound on retry count."),
        RetryBudget: ("en", "Retry budget", "A rate-limit on retries over a window — prevents retry-induced overload."),

        Supervisor: ("en", "Supervisor", "Armstrong (2003): a process responsible for restarting its children after failure."),
        SupervisedChild: ("en", "Supervised child", "A process whose lifecycle is managed by a Supervisor."),
        OneForOne: ("en", "One-for-one", "Armstrong (2003): only the failed child is restarted."),
        OneForAll: ("en", "One-for-all", "Armstrong (2003): every child is restarted when any one fails."),
        RestForOne: ("en", "Rest-for-one", "Armstrong (2003): the failed child plus every child started after it are restarted."),
        RestartIntensity: ("en", "Restart intensity", "Maximum restarts per period before the supervisor itself fails."),
        RestartPeriod: ("en", "Restart period", "The time window for counting RestartIntensity."),
        LetItCrash: ("en", "Let it crash", "Armstrong (2003): treat crash as the normal failure path; recovery is the Supervisor's job, not the worker's."),

        UndoOperation: ("en", "Undo operation", "Patterson et al. (2002): reverse an operation to restore prior state."),
        Microreboot: ("en", "Microreboot", "Patterson et al. (2002): restart a small component instead of the whole system."),
        Quarantine: ("en", "Quarantine", "Patterson et al. (2002): isolate a suspect component to prevent further damage while preserving evidence."),

        Service: ("en", "Service", "The unit of behaviour that resilience protects (matches Dependability::Service)."),
        Resource: ("en", "Resource", "Finite capacity consumed by requests — threads, connections, memory."),
        Request: ("en", "Request", "A single unit of work flowing through the system."),
    },

    is_a: [
        // === Stability pattern family (Nygard) ===
        (CircuitBreaker, StabilityPattern),
        (Bulkhead, StabilityPattern),
        (Timeout, StabilityPattern),
        (FailFast, StabilityPattern),
        (SteadyState, StabilityPattern),
        (HandshakingProtocol, StabilityPattern),
        (TestHarness, StabilityPattern),

        // === Circuit breaker states ===
        (CircuitBreakerClosed, CircuitBreaker),
        (CircuitBreakerOpen, CircuitBreaker),
        (CircuitBreakerHalfOpen, CircuitBreaker),

        // === Backoff family ===
        (ExponentialBackoff, BackoffStrategy),
        (LinearBackoff, BackoffStrategy),
        (ConstantBackoff, BackoffStrategy),

        // === Jitter family (Brooker 2015) ===
        (FullJitter, JitterStrategy),
        (EqualJitter, JitterStrategy),
        (DecorrelatedJitter, JitterStrategy),

        // === Supervision family (Armstrong 2003) ===
        (OneForOne, SupervisionStrategy),
        (OneForAll, SupervisionStrategy),
        (RestForOne, SupervisionStrategy),

        // === Recovery-Oriented Computing (Patterson 2002) ===
        (UndoOperation, RecoveryPattern),
        (Microreboot, RecoveryPattern),
        (Quarantine, RecoveryPattern),
    ],

    edges: [
        // === Protection relationships ===
        (CircuitBreaker, Service, Protects),
        (Bulkhead, Resource, Isolates),
        (Timeout, Request, Bounds),

        // === Retry dynamics ===
        (Retry, Request, Reissues),
        (BackoffStrategy, Retry, Schedules),
        (JitterStrategy, BackoffStrategy, Randomises),
        (MaxAttempts, Retry, Limits),
        (RetryBudget, Retry, RateLimits),

        // === Supervision (Armstrong) ===
        (Supervisor, SupervisedChild, Restarts),
        (SupervisionStrategy, Supervisor, Governs),
        (RestartIntensity, Supervisor, Bounds),
        (RestartPeriod, RestartIntensity, Windows),
        (LetItCrash, Supervisor, Delegates),

        // === Recovery (Patterson) ===
        (UndoOperation, Service, Restores),
        (Microreboot, Service, Restarts),
        (Quarantine, Resource, Isolates),

        // === Circuit breaker state transitions (closed→open→half-open→…) ===
        (CircuitBreakerClosed, CircuitBreakerOpen, TripsTo),
        (CircuitBreakerOpen, CircuitBreakerHalfOpen, CoolsTo),
        (CircuitBreakerHalfOpen, CircuitBreakerClosed, ResolvesTo),
        (CircuitBreakerHalfOpen, CircuitBreakerOpen, RelapsesTo),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: the resilience category each concept belongs to.
#[derive(Debug, Clone)]
pub struct ResilienceCategoryOf;

impl Quality for ResilienceCategoryOf {
    type Individual = ResilienceConcept;
    type Value = &'static str;

    fn get(&self, c: &ResilienceConcept) -> Option<&'static str> {
        use ResilienceConcept as R;
        Some(match c {
            R::StabilityPattern
            | R::CircuitBreaker
            | R::CircuitBreakerClosed
            | R::CircuitBreakerOpen
            | R::CircuitBreakerHalfOpen
            | R::Bulkhead
            | R::Timeout
            | R::FailFast
            | R::SteadyState
            | R::HandshakingProtocol
            | R::TestHarness => "stability-pattern",
            R::BackoffStrategy | R::ExponentialBackoff | R::LinearBackoff | R::ConstantBackoff => {
                "backoff"
            }
            R::JitterStrategy | R::FullJitter | R::EqualJitter | R::DecorrelatedJitter => "jitter",
            R::Retry | R::MaxAttempts | R::RetryBudget => "retry",
            R::SupervisionStrategy
            | R::Supervisor
            | R::SupervisedChild
            | R::OneForOne
            | R::OneForAll
            | R::RestForOne
            | R::RestartIntensity
            | R::RestartPeriod
            | R::LetItCrash => "supervision",
            R::RecoveryPattern | R::UndoOperation | R::Microreboot | R::Quarantine => "recovery",
            R::Service | R::Resource | R::Request => "target",
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect direct (non-transitive) children of a concept under the resilience
/// taxonomy.
fn direct_children_of(parent: ResilienceConcept) -> Vec<ResilienceConcept> {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    ResilienceTaxonomy::relations()
        .into_iter()
        .filter_map(|(child, p)| if p == parent { Some(child) } else { None })
        .collect()
}

// ---------------------------------------------------------------------------
// Axioms — invariants of the resilience literature
// ---------------------------------------------------------------------------

/// Axiom: the direct children of `CircuitBreaker` are exactly the three states
/// {Closed, Open, HalfOpen} (Nygard 2007). A fourth state would invalidate.
pub struct CircuitBreakerThreeStates;

impl Axiom for CircuitBreakerThreeStates {
    fn description(&self) -> &str {
        "the direct children of CircuitBreaker are exactly {Closed, Open, HalfOpen} (Nygard 2007)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(ResilienceConcept::CircuitBreaker);
        let expected = [
            ResilienceConcept::CircuitBreakerClosed,
            ResilienceConcept::CircuitBreakerOpen,
            ResilienceConcept::CircuitBreakerHalfOpen,
        ];
        actual.len() == expected.len() && expected.iter().all(|s| actual.contains(s))
    }
}

/// Axiom: the three jitter strategies from Brooker (2015) are all classified
/// as JitterStrategy. {Full, Equal, Decorrelated} are the canonical set;
/// other variants may exist but at minimum these three must be present.
pub struct BrookerJitterStrategiesExist;

impl Axiom for BrookerJitterStrategiesExist {
    fn description(&self) -> &str {
        "the three Brooker (2015) jitter strategies {Full, Equal, Decorrelated} are all classified as JitterStrategy"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(ResilienceConcept::JitterStrategy);
        let expected = [
            ResilienceConcept::FullJitter,
            ResilienceConcept::EqualJitter,
            ResilienceConcept::DecorrelatedJitter,
        ];
        expected.iter().all(|j| actual.contains(j))
    }
}

/// Axiom: Armstrong (2003) supervision strategies are exactly
/// {OneForOne, OneForAll, RestForOne} — the three OTP strategies.
pub struct OtpSupervisionStrategies;

impl Axiom for OtpSupervisionStrategies {
    fn description(&self) -> &str {
        "the direct children of SupervisionStrategy are exactly {OneForOne, OneForAll, RestForOne} (Armstrong 2003 OTP)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(ResilienceConcept::SupervisionStrategy);
        let expected = [
            ResilienceConcept::OneForOne,
            ResilienceConcept::OneForAll,
            ResilienceConcept::RestForOne,
        ];
        actual.len() == expected.len() && expected.iter().all(|s| actual.contains(s))
    }
}

/// Axiom: the circuit breaker state machine is a cycle closed ↔ open ↔ half-open.
/// The four transitions must exist as morphisms.
pub struct CircuitBreakerTransitionsExist;

impl Axiom for CircuitBreakerTransitionsExist {
    fn description(&self) -> &str {
        "circuit breaker transitions {Closed→Open, Open→HalfOpen, HalfOpen→Closed, HalfOpen→Open} all exist (Nygard 2007)"
    }
    fn holds(&self) -> bool {
        use ResilienceConcept as R;
        use ResilienceRelationKind as K;
        let m = ResilienceCategory::morphisms();
        // Kind-strict match: the Composed morphism kind (transitive closure)
        // would otherwise let this axiom pass without the direct state-machine
        // edges being declared.
        let has = |from: R, to: R, kind: K| {
            m.iter()
                .any(|r| r.from == from && r.to == to && r.kind == kind)
        };
        has(R::CircuitBreakerClosed, R::CircuitBreakerOpen, K::TripsTo)
            && has(R::CircuitBreakerOpen, R::CircuitBreakerHalfOpen, K::CoolsTo)
            && has(
                R::CircuitBreakerHalfOpen,
                R::CircuitBreakerClosed,
                K::ResolvesTo,
            )
            && has(
                R::CircuitBreakerHalfOpen,
                R::CircuitBreakerOpen,
                K::RelapsesTo,
            )
    }
}

/// Axiom: Patterson (2002) ROC recovery patterns are all classified under
/// RecoveryPattern.
pub struct RocPatternsClassified;

impl Axiom for RocPatternsClassified {
    fn description(&self) -> &str {
        "Patterson et al. (2002) ROC patterns {Undo, Microreboot, Quarantine} are all classified as RecoveryPattern"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(ResilienceConcept::RecoveryPattern);
        let expected = [
            ResilienceConcept::UndoOperation,
            ResilienceConcept::Microreboot,
            ResilienceConcept::Quarantine,
        ];
        expected.iter().all(|p| actual.contains(p))
    }
}

impl Ontology for ResilienceOntology {
    type Cat = ResilienceCategory;
    type Qual = ResilienceCategoryOf;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        ResilienceOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(CircuitBreakerThreeStates),
            Box::new(BrookerJitterStrategiesExist),
            Box::new(OtpSupervisionStrategies),
            Box::new(CircuitBreakerTransitionsExist),
            Box::new(RocPatternsClassified),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ResilienceCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ResilienceOntology::validate().unwrap();
    }

    #[test]
    fn circuit_breaker_three_states_axiom_holds() {
        assert!(
            CircuitBreakerThreeStates.holds(),
            "{}",
            CircuitBreakerThreeStates.description()
        );
    }

    #[test]
    fn brooker_jitter_axiom_holds() {
        assert!(
            BrookerJitterStrategiesExist.holds(),
            "{}",
            BrookerJitterStrategiesExist.description()
        );
    }

    #[test]
    fn otp_supervision_axiom_holds() {
        assert!(
            OtpSupervisionStrategies.holds(),
            "{}",
            OtpSupervisionStrategies.description()
        );
    }

    #[test]
    fn circuit_breaker_transitions_axiom_holds() {
        assert!(
            CircuitBreakerTransitionsExist.holds(),
            "{}",
            CircuitBreakerTransitionsExist.description()
        );
    }

    #[test]
    fn roc_patterns_axiom_holds() {
        assert!(
            RocPatternsClassified.holds(),
            "{}",
            RocPatternsClassified.description()
        );
    }
}
