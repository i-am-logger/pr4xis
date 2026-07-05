//! Scheduler — processor scheduling: policies, the periodic task model,
//! schedulability bounds, and priority-inversion control.
//!
//! Source traditions:
//!
//! - **Liu & Layland (1973)** *Scheduling Algorithms for Multiprogramming
//!   in a Hard-Real-Time Environment*, JACM 20(1):46–61 — the periodic
//!   task model, rate-monotonic and deadline-driven scheduling, the
//!   utilization bounds (Theorems 5 and 7).
//! - **Sha, Rajkumar & Lehoczky (1990)** *Priority Inheritance
//!   Protocols: An Approach to Real-Time Synchronization*, IEEE
//!   Transactions on Computers 39(9):1175–1185 — priority inversion and
//!   its inheritance-based bounding.
//! - **Corbató, Merwin-Daggett & Daley (1962)** *An Experimental
//!   Time-Sharing System*, AFIPS SJCC 21 — time-sliced round-robin
//!   dispatch and the multilevel feedback queue (CTSS).
//! - **Leung & Whitehead (1982)** *On the complexity of fixed-priority
//!   scheduling of periodic, real-time tasks*, Performance Evaluation
//!   2(4):237–250 — deadline-monotonic priority assignment.
//!
//! The four domain axioms are discharged against the ready-queue
//! simulator and the cited fixtures in [`super::engine`].

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::unit::{self, Unit};

use super::engine::{
    EDF_UTILIZATION_BOUND, LockingProtocol, PolicyOrder, SHA_HIGH_JOB_INDEX, SHA_MEDIUM_JOB_INDEX,
    ll_example_task_set, rm_admits, rm_utilization_bound, sha_high_completion_bound_slot,
    sha_inversion_jobs, simulate_periodic, simulate_with_shared_resource,
};

pr4xis::ontology! {
    name: "Scheduler",
    source: "Liu & Layland (1973) JACM 20(1); Sha, Rajkumar & Lehoczky (1990) IEEE TC 39(9); Corbato, Merwin-Daggett & Daley (1962) AFIPS SJCC 21; Leung & Whitehead (1982) Performance Evaluation 2(4)",

    concepts: [
        // === Policies ===
        SchedulingPolicy,
        FixedPriority,
        RateMonotonic,
        DeadlineMonotonic,
        EarliestDeadlineFirst,
        RoundRobin,
        MultilevelFeedbackQueue,
        FairShare,
        CompletelyFairScheduler,

        // === The task model (Liu & Layland 1973 §2) ===
        Task,
        Job,
        Priority,
        Period,
        Deadline,
        Wcet,
        Utilization,
        UtilizationBound,

        // === Blocking and its control (Sha et al. 1990) ===
        PriorityInversion,
        PriorityInheritance,
        Preemption,
    ],

    labels: {
        SchedulingPolicy: ("en", "Scheduling policy", "Liu & Layland (1973) 'Scheduling Algorithms for Multiprogramming in a Hard-Real-Time Environment' JACM 20(1): a rule ordering task execution - the scheduling algorithm."),
        FixedPriority: ("en", "Fixed priority", "Liu & Layland (1973): a policy whose priorities are assigned to tasks once, ahead of execution (static assignment)."),
        RateMonotonic: ("en", "Rate monotonic", "Liu & Layland (1973) sec 3: static priority by rate - the shorter the period, the higher the priority; optimal among fixed-priority assignments."),
        DeadlineMonotonic: ("en", "Deadline monotonic", "Leung & Whitehead (1982) 'On the complexity of fixed-priority scheduling of periodic, real-time tasks' Performance Evaluation 2(4):237-250: fixed priority by relative deadline; generalizes rate-monotonic to deadlines no longer than periods."),
        EarliestDeadlineFirst: ("en", "Earliest deadline first", "Liu & Layland (1973) sec 7 (the deadline driven algorithm): dynamic priority by nearest absolute deadline; feasible up to full utilization U = 1."),
        RoundRobin: ("en", "Round robin", "Corbato, Merwin-Daggett & Daley (1962) 'An Experimental Time-Sharing System' AFIPS SJCC 21 (CTSS): time-sliced cyclic dispatch of the ready queue."),
        MultilevelFeedbackQueue: ("en", "Multilevel feedback queue", "Corbato, Merwin-Daggett & Daley (1962): priority queues with feedback demotion - longer-running programs sink to lower-priority levels with longer quanta."),
        FairShare: ("en", "Fair share", "Proportional CPU share by weight - the abstract fair-share policy family whose Linux realization is the Completely Fair Scheduler (Molnar 2007, Linux kernel documentation, non-peer-reviewed; see citings.md honest-tier note)."),
        CompletelyFairScheduler: ("en", "Completely fair scheduler", "Molnar (2007), Linux kernel documentation, Documentation/scheduler/sched-design-CFS - non-peer-reviewed kernel documentation: the Linux CFS, weight-proportional fair sharing of CPU time via virtual runtime."),
        Task: ("en", "Task", "Liu & Layland (1973) sec 2: a recurring computation with a fixed request period."),
        Job: ("en", "Job", "Liu & Layland (1973) sec 2: a single release (request instance) of a task."),
        Priority: ("en", "Priority", "Sha, Rajkumar & Lehoczky (1990) IEEE TC 39(9): the ordinal dispatch rank by which the processor is granted."),
        Period: ("en", "Period", "Liu & Layland (1973): the fixed inter-release (request) interval T of a periodic task."),
        Deadline: ("en", "Deadline", "Liu & Layland (1973): the bound on a job's completion time - in the base model each request must complete before the next request of the same task."),
        Wcet: ("en", "Worst-case execution time", "Liu & Layland (1973): the worst-case execution time C - the maximum processor demand of one job."),
        Utilization: ("en", "Utilization", "Liu & Layland (1973): the processor utilization factor U = sum of Ci/Ti over the task set."),
        UtilizationBound: ("en", "Utilization bound", "Liu & Layland (1973) Theorem 5: the least upper bound n(2^(1/n)-1) on the utilization schedulable under rate-monotonic priorities."),
        PriorityInversion: ("en", "Priority inversion", "Sha, Rajkumar & Lehoczky (1990) 'Priority Inheritance Protocols' IEEE TC 39(9): a lower-priority task blocks a higher-priority one - unbounded when medium-priority tasks preempt the blocker."),
        PriorityInheritance: ("en", "Priority inheritance", "Sha, Rajkumar & Lehoczky (1990): the blocker executes at the highest priority among the jobs it blocks, bounding inversion by critical-section length."),
        Preemption: ("en", "Preemption", "Liu & Layland (1973): suspension of the running task in favour of a higher-priority one - the preemptive model both of the paper's algorithms assume."),
    },

    is_a: [
        // Liu & Layland (1973) sec 3: rate-monotonic is the rate-ordered
        // fixed-priority assignment.
        (RateMonotonic, FixedPriority),
        // Leung & Whitehead (1982): deadline-monotonic is the
        // deadline-ordered fixed-priority assignment.
        (DeadlineMonotonic, FixedPriority),
        // The policy taxonomy.
        (FixedPriority, SchedulingPolicy),
        (EarliestDeadlineFirst, SchedulingPolicy),
        (RoundRobin, SchedulingPolicy),
        (MultilevelFeedbackQueue, SchedulingPolicy),
        (FairShare, SchedulingPolicy),
        // Molnar (2007): CFS is a fair-share policy.
        (CompletelyFairScheduler, FairShare),
    ],

    has_a: [
        // Liu & Layland (1973) sec 2: a task's releases are its jobs.
        (Task, Job),
    ],

    edges: [
        // Liu & Layland (1973): a policy orders task execution.
        (SchedulingPolicy, Task, Schedules),
        // Liu & Layland (1973) Theorem 5: the bound governs
        // rate-monotonic admission.
        (UtilizationBound, RateMonotonic, Bounds),
        // Liu & Layland (1973) sec 7: the deadline-driven algorithm
        // schedules any task set rate-monotonic can, up to U = 1.
        (EarliestDeadlineFirst, RateMonotonic, Dominates),
        // Sha et al. (1990): inheritance bounds inversion.
        (PriorityInheritance, PriorityInversion, Mitigates),
        // Liu & Layland (1973) preemptive model: both algorithm
        // families rely on preemption.
        (FixedPriority, Preemption, Employs),
        (EarliestDeadlineFirst, Preemption, Employs),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// How a policy assigns priorities — the Liu & Layland (1973)
/// distinction between the fixed (static, §3) and the deadline-driven
/// (dynamic, §7) assignment, plus the time-sliced / fair-share family
/// that dispatches without per-task priority ranks (Corbató et al.
/// 1962; Molnar 2007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityKind {
    /// Priorities assigned to tasks once, ahead of execution — Liu &
    /// Layland (1973) §3.
    Static,
    /// Priorities recomputed per job from the deadlines at hand — Liu &
    /// Layland (1973) §7.
    Dynamic,
    /// Dispatch by time slice or proportional share, not by a per-task
    /// priority rank — Corbató et al. (1962); Molnar (2007).
    NotPriorityBased,
}

/// Which [`PriorityKind`] a scheduling policy uses. `Some` for exactly
/// the concrete policies; `None` for the abstract `SchedulingPolicy`
/// parent and every non-policy concept.
#[derive(Debug, Clone)]
pub struct PolicyPriorityAssignment;

impl Quality for PolicyPriorityAssignment {
    type Individual = SchedulerConcept;
    type Value = PriorityKind;

    fn get(&self, c: &SchedulerConcept) -> Option<PriorityKind> {
        use SchedulerConcept as S;
        match c {
            S::RateMonotonic | S::DeadlineMonotonic | S::FixedPriority => {
                Some(PriorityKind::Static)
            }
            S::EarliestDeadlineFirst => Some(PriorityKind::Dynamic),
            S::RoundRobin
            | S::MultilevelFeedbackQueue
            | S::FairShare
            | S::CompletelyFairScheduler => Some(PriorityKind::NotPriorityBased),
            _ => None,
        }
    }
}

/// The typed measurement unit of each timing attribute of the task
/// model — the value is the typed [`Unit`] from the `quantity`
/// ontology, not a prose symbol string (the `applied/navigation/imu`
/// precedent). Period, deadline, and worst-case execution time are
/// times (Liu & Layland 1973 §2); utilization and its bound are
/// dimensionless ratios (Liu & Layland 1973 §4–5). `None` for concepts
/// that are not timing attributes.
#[derive(Debug, Clone)]
pub struct TimingAttribute;

impl Quality for TimingAttribute {
    type Individual = SchedulerConcept;
    type Value = Unit;

    fn get(&self, c: &SchedulerConcept) -> Option<Unit> {
        use SchedulerConcept as S;
        match c {
            S::Period | S::Deadline | S::Wcet => Some(unit::SECOND),
            S::Utilization | S::UtilizationBound => Some(unit::UNITLESS),
            _ => None,
        }
    }
}

/// Whether a policy suspends the running task for a higher-priority
/// arrival — the preemptive model Liu & Layland (1973) assume for both
/// their algorithms, shared by deadline-monotonic (Leung & Whitehead
/// 1982) and by CTSS's quantum-expiry switching (Corbató et al. 1962).
/// `None` for abstract or non-policy concepts.
#[derive(Debug, Clone)]
pub struct IsPreemptive;

impl Quality for IsPreemptive {
    type Individual = SchedulerConcept;
    type Value = bool;

    fn get(&self, c: &SchedulerConcept) -> Option<bool> {
        use SchedulerConcept as S;
        match c {
            S::RateMonotonic
            | S::DeadlineMonotonic
            | S::EarliestDeadlineFirst
            | S::FixedPriority
            | S::RoundRobin
            | S::MultilevelFeedbackQueue => Some(true),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Numeric parameters (all named + cited — no magic numbers)
// ---------------------------------------------------------------------------

/// The task-count grid `n ∈ 1..=8` over which the Liu & Layland
/// Theorem 5 bound is probed — the bound holds for all `n`; the grid
/// spans the small task counts the paper itself discusses (`U_2 ≈
/// 0.828` in §5) into the asymptotic regime approaching `ln 2`.
pub const RM_BOUND_TASK_COUNT_GRID_MAX: usize = 8;

/// Float comparison tolerance for the algebraic identities below — the
/// round-off slack; same value and provenance as the
/// `formal::systems::parallelism` precedent (Goldberg 1991, *What Every
/// Computer Scientist Should Know About Floating-Point Arithmetic*,
/// ACM Computing Surveys 23(1):5-48).
pub const NUMERIC_TOLERANCE: f64 = 1e-9;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn kinded_edge_exists(
    from: SchedulerConcept,
    to: SchedulerConcept,
    kind: SchedulerRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    SchedulerCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// `base^n` by repeated multiplication — a core-only cross-check of the
/// bound values, independent of `powf`.
fn nth_power(base: f64, n: usize) -> f64 {
    let mut acc = 1.0;
    for _ in 0..n {
        acc *= base;
    }
    acc
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Liu & Layland (1973) Theorem 5: the rate-monotonic least upper
/// utilization bound is `U_n = n(2^(1/n) − 1)`. Verified over the
/// structural grid `n ∈ 1..=RM_BOUND_TASK_COUNT_GRID_MAX`:
///
/// - each `U_n` satisfies the defining identity `(1 + U_n/n)^n = 2`
///   within [`NUMERIC_TOLERANCE`], checked by repeated multiplication
///   (a core-only route independent of `powf`);
/// - `U_1 = 1` exactly;
/// - `U_n` is strictly decreasing in `n`;
/// - every `U_n` exceeds the `ln 2` limit;
/// - the engine's rate-monotonic admission test admits the Liu &
///   Layland §3 fixture set (`U = 0.7 ≤ U_2`), and the admitted set
///   indeed meets every deadline in simulation;
/// - the category carries the matching `Bounds` edge from
///   `UtilizationBound` to `RateMonotonic`.
pub struct LiuLaylandBound;

impl Axiom for LiuLaylandBound {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let mut ok = true;
        let mut prev = f64::INFINITY;
        for n in 1..=RM_BOUND_TASK_COUNT_GRID_MAX {
            let u_n = rm_utilization_bound(n);
            // Defining identity: 1 + U_n/n = 2^(1/n), so (1 + U_n/n)^n = 2.
            let identity = (nth_power(1.0 + u_n / n as f64, n) - 2.0).abs() <= NUMERIC_TOLERANCE;
            // U_1 = 1(2^1 − 1) = 1 exactly.
            let base_case = n != 1 || u_n == 1.0;
            // Strictly decreasing towards, and always above, ln 2.
            let decreasing = u_n < prev;
            let above_limit = u_n > core::f64::consts::LN_2;
            ok = ok && identity && base_case && decreasing && above_limit;
            prev = u_n;
        }
        // Engine: the cited §3 fixture is admitted and schedulable.
        let fixture = ll_example_task_set();
        let admitted = rm_admits(&fixture);
        let schedulable =
            simulate_periodic(&fixture, PolicyOrder::RateMonotonic).met_all_deadlines();
        let edge = kinded_edge_exists(
            SchedulerConcept::UtilizationBound,
            SchedulerConcept::RateMonotonic,
            SchedulerRelationKind::Bounds,
        );
        if ok && admitted && schedulable && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "LiuLaylandBound",
        "U_n = n(2^(1/n)-1) satisfies (1+U_n/n)^n = 2, equals 1 at n = 1, decreases strictly towards ln 2 over the task-count grid, and the admitted sec-3 fixture meets every deadline under rate-monotonic simulation",
        "Liu & Layland (1973) JACM 20(1), Theorem 5"
    );
}
pr4xis::register_axiom!(
    LiuLaylandBound,
    "Liu & Layland (1973) JACM 20(1), Theorem 5"
);

/// Liu & Layland (1973) §7: the deadline-driven algorithm dominates
/// rate-monotonic — it schedules any task set rate-monotonic can, and
/// its feasibility bound is exactly full utilization (`U = 1`, Theorem
/// 7), while the rate-monotonic bound is strictly below 1 for every
/// `n ≥ 2` (verified numerically from the Theorem 5 values). The
/// category carries the matching `Dominates` edge.
pub struct EdfDominatesRm;

impl Axiom for EdfDominatesRm {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let edf_bound_is_full_utilization = EDF_UTILIZATION_BOUND == 1.0;
        let mut rm_below_one = true;
        for n in 2..=RM_BOUND_TASK_COUNT_GRID_MAX {
            if rm_utilization_bound(n) >= EDF_UTILIZATION_BOUND {
                rm_below_one = false;
            }
        }
        let edge = kinded_edge_exists(
            SchedulerConcept::EarliestDeadlineFirst,
            SchedulerConcept::RateMonotonic,
            SchedulerRelationKind::Dominates,
        );
        if edf_bound_is_full_utilization && rm_below_one && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EdfDominatesRm",
        "the deadline-driven feasibility bound is exactly 1.0 while the rate-monotonic bound is strictly below 1.0 for every n >= 2, and the Dominates edge is present",
        "Liu & Layland (1973) JACM 20(1) sec 7"
    );
}
pr4xis::register_axiom!(EdfDominatesRm, "Liu & Layland (1973) JACM 20(1) sec 7");

/// Liu & Layland (1973) §3: rate-monotonic is a fixed-priority
/// assignment, which is a scheduling policy — the transitive
/// Subsumption path `RateMonotonic → FixedPriority → SchedulingPolicy`
/// exists in the category, both as direct edges and as their
/// composition.
pub struct RmIsFixedPriority;

impl Axiom for RmIsFixedPriority {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::Category;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let leg1 = kinded_edge_exists(
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::FixedPriority,
            SchedulerRelationKind::Subsumption,
        );
        let leg2 = kinded_edge_exists(
            SchedulerConcept::FixedPriority,
            SchedulerConcept::SchedulingPolicy,
            SchedulerRelationKind::Subsumption,
        );
        let closure = kinded_edge_exists(
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::SchedulingPolicy,
            SchedulerRelationKind::Subsumption,
        );
        // The path also composes categorically to the closure edge.
        let f = SchedulerRelation {
            from: SchedulerConcept::RateMonotonic,
            to: SchedulerConcept::FixedPriority,
            kind: SchedulerRelationKind::Subsumption,
        };
        let g = SchedulerRelation {
            from: SchedulerConcept::FixedPriority,
            to: SchedulerConcept::SchedulingPolicy,
            kind: SchedulerRelationKind::Subsumption,
        };
        let composes = SchedulerCategory::compose(&f, &g)
            == Some(SchedulerRelation {
                from: SchedulerConcept::RateMonotonic,
                to: SchedulerConcept::SchedulingPolicy,
                kind: SchedulerRelationKind::Subsumption,
            });
        if leg1 && leg2 && closure && composes {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RmIsFixedPriority",
        "the transitive Subsumption path RateMonotonic -> FixedPriority -> SchedulingPolicy exists in the category and composes to the closure edge",
        "Liu & Layland (1973) JACM 20(1) sec 3"
    );
}
pr4xis::register_axiom!(RmIsFixedPriority, "Liu & Layland (1973) JACM 20(1) sec 3");

/// Sha, Rajkumar & Lehoczky (1990): the basic priority-inheritance
/// protocol bounds priority inversion. On the engine's three-job §II
/// fixture (low job holds the resource, high job blocks on it, medium
/// job preempts the blocker):
///
/// - **without** inheritance the high job's completion is delayed past
///   the one-critical-section bound by the medium job's interference —
///   the medium job even finishes first (the inversion signature);
/// - **with** inheritance the blocker runs at the high job's priority,
///   so the high job completes within the bound
///   (`arrival + own demand + one critical section`).
///
/// The category carries the matching `Mitigates` edge.
pub struct PriorityInheritanceBoundsInversion;

impl Axiom for PriorityInheritanceBoundsInversion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let jobs = sha_inversion_jobs();
        let bound = sha_high_completion_bound_slot();

        let without = simulate_with_shared_resource(&jobs, LockingProtocol::NoInheritance);
        let with = simulate_with_shared_resource(&jobs, LockingProtocol::BasicPriorityInheritance);

        let (Some(h_without), Some(m_without), Some(h_with)) = (
            without.completion_slot[SHA_HIGH_JOB_INDEX],
            without.completion_slot[SHA_MEDIUM_JOB_INDEX],
            with.completion_slot[SHA_HIGH_JOB_INDEX],
        ) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };

        // Without inheritance: unbounded inversion — the high job blows
        // the bound, and the medium job finishes before it.
        let inversion_shown = h_without > bound && m_without < h_without;
        // With inheritance: blocking is bounded by one critical section.
        let inversion_bounded = h_with <= bound;
        let edge = kinded_edge_exists(
            SchedulerConcept::PriorityInheritance,
            SchedulerConcept::PriorityInversion,
            SchedulerRelationKind::Mitigates,
        );
        if inversion_shown && inversion_bounded && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PriorityInheritanceBoundsInversion",
        "on the three-job fixture the high job blows the one-critical-section completion bound without inheritance (the medium job finishing first) and meets it with basic priority inheritance",
        "Sha, Rajkumar & Lehoczky (1990) IEEE Transactions on Computers 39(9), Priority Inheritance Protocols"
    );
}
pr4xis::register_axiom!(
    PriorityInheritanceBoundsInversion,
    "Sha, Rajkumar & Lehoczky (1990) IEEE Transactions on Computers 39(9), Priority Inheritance Protocols"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for SchedulerOntology {
    type Cat = SchedulerCategory;
    type Qual = TimingAttribute;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(LiuLaylandBound));
        axioms.push(Box::new(EdfDominatesRm));
        axioms.push(Box::new(RmIsFixedPriority));
        axioms.push(Box::new(PriorityInheritanceBoundsInversion));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SchedulerCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SchedulerOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn liu_layland_bound_holds() {
        assert!(LiuLaylandBound.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn edf_dominates_rm_holds() {
        assert!(EdfDominatesRm.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rm_is_fixed_priority_holds() {
        assert!(RmIsFixedPriority.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn priority_inheritance_bounds_inversion_holds() {
        assert!(PriorityInheritanceBoundsInversion.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn timing_attributes_carry_typed_units() {
        for c in [
            SchedulerConcept::Period,
            SchedulerConcept::Deadline,
            SchedulerConcept::Wcet,
        ] {
            assert_eq!(
                TimingAttribute.get(&c),
                Some(unit::SECOND),
                "{c:?} is a time (Liu & Layland 1973 sec 2)"
            );
        }
        for c in [
            SchedulerConcept::Utilization,
            SchedulerConcept::UtilizationBound,
        ] {
            assert_eq!(
                TimingAttribute.get(&c),
                Some(unit::UNITLESS),
                "{c:?} is a dimensionless ratio (Liu & Layland 1973 sec 4-5)"
            );
        }
        assert_eq!(
            TimingAttribute.get(&SchedulerConcept::SchedulingPolicy),
            None,
            "a policy is not a timing attribute"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn policy_priority_assignment_classification() {
        let q = PolicyPriorityAssignment;
        for c in [
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::DeadlineMonotonic,
            SchedulerConcept::FixedPriority,
        ] {
            assert_eq!(q.get(&c), Some(PriorityKind::Static), "{c:?}");
        }
        assert_eq!(
            q.get(&SchedulerConcept::EarliestDeadlineFirst),
            Some(PriorityKind::Dynamic)
        );
        for c in [
            SchedulerConcept::RoundRobin,
            SchedulerConcept::MultilevelFeedbackQueue,
            SchedulerConcept::FairShare,
            SchedulerConcept::CompletelyFairScheduler,
        ] {
            assert_eq!(q.get(&c), Some(PriorityKind::NotPriorityBased), "{c:?}");
        }
        assert_eq!(
            q.get(&SchedulerConcept::SchedulingPolicy),
            None,
            "the abstract parent has no assignment discipline of its own"
        );
        assert_eq!(q.get(&SchedulerConcept::Task), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn preemptive_classification() {
        for c in [
            SchedulerConcept::RateMonotonic,
            SchedulerConcept::DeadlineMonotonic,
            SchedulerConcept::EarliestDeadlineFirst,
            SchedulerConcept::FixedPriority,
            SchedulerConcept::RoundRobin,
            SchedulerConcept::MultilevelFeedbackQueue,
        ] {
            assert_eq!(IsPreemptive.get(&c), Some(true), "{c:?}");
        }
        assert_eq!(
            IsPreemptive.get(&SchedulerConcept::SchedulingPolicy),
            None,
            "the abstract parent carries no preemption discipline"
        );
        assert_eq!(IsPreemptive.get(&SchedulerConcept::PriorityInversion), None);
    }
}
