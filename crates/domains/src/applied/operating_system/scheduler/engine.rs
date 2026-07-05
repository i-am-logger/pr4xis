//! Scheduler engine — a ready-queue processor-scheduling simulator over
//! an integer slot grid, discharged against two cited fixtures:
//!
//! 1. **Liu & Layland (1973)** *Scheduling Algorithms for Multiprogramming
//!    in a Hard-Real-Time Environment*, JACM 20(1) — the §3 two-task
//!    example set, simulated under both the rate-monotonic and the
//!    deadline-driven (earliest-deadline-first) dispatch orders.
//! 2. **Sha, Rajkumar & Lehoczky (1990)** *Priority Inheritance
//!    Protocols: An Approach to Real-Time Synchronization*, IEEE
//!    Transactions on Computers 39(9) — the classic three-job
//!    priority-inversion scenario, simulated with and without the basic
//!    priority-inheritance protocol.
//!
//! # Discretization
//!
//! Liu & Layland develop the periodic task model in continuous time. The
//! engine discretizes to an integer grid of unit **slots**, each one
//! [`time_quantum`] (1 s) long. Task parameters (period, worst-case
//! execution time, relative deadline) are typed [`Quantity`] values in
//! `unit::SECOND` and must be whole multiples of the quantum —
//! [`slot_count`] refuses fractional parameters — so the discretization
//! is *exact* for the integer-valued cited fixtures, not an
//! approximation.
//!
//! Every constant below is a named, documented fixture parameter cited
//! to its source — no free magic numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

// ---------------------------------------------------------------------------
// The time grid (Liu & Layland 1973 §2, discretized)
// ---------------------------------------------------------------------------

/// One scheduling slot — the engine's unit time quantum (1 s). Liu &
/// Layland (1973) §2 state the model in continuous time; the engine's
/// integer grid samples it at this quantum, exactly, for whole-second
/// task parameters.
pub fn time_quantum() -> Quantity {
    Quantity::from_unit(1.0, &unit::SECOND)
}

/// Convert a typed timing parameter to a whole number of slots.
///
/// The quotient `q / time_quantum()` must be dimensionless (both are
/// times) and a whole non-negative number — the documented exactness
/// condition of the discretization (see the module docs). Refuses
/// (panics on) anything else rather than silently rounding.
pub fn slot_count(q: &Quantity) -> usize {
    let quanta = q.div(&time_quantum());
    assert!(
        quanta.is_dimensionless(),
        "a scheduler timing parameter must be a time quantity"
    );
    let v = quanta.value;
    let whole = v as u64;
    assert!(
        v >= 0.0 && whole as f64 == v,
        "a scheduler timing parameter must be a whole number of time quanta"
    );
    whole as usize
}

// ---------------------------------------------------------------------------
// The periodic task model (Liu & Layland 1973 §2)
// ---------------------------------------------------------------------------

/// A task identity — tasks are named, never anonymous indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub usize);

/// A periodic task — Liu & Layland (1973) §2: a recurring computation
/// with fixed request period `T`, worst-case execution time `C`, and a
/// completion deadline. All three are typed time [`Quantity`] values in
/// `unit::SECOND` over the integer slot grid.
#[derive(Debug, Clone, PartialEq)]
pub struct PeriodicTask {
    /// The task's identity.
    pub id: TaskId,
    /// Inter-release (request) interval `T` — Liu & Layland (1973) §2.
    pub period: Quantity,
    /// Worst-case execution time `C` — Liu & Layland (1973) §2.
    pub wcet: Quantity,
    /// Relative deadline: each job must complete this long after its
    /// release. In the base Liu & Layland model the deadline equals the
    /// period (each request must complete before the next).
    pub deadline: Quantity,
}

impl PeriodicTask {
    /// The period in whole slots.
    pub fn period_slots(&self) -> usize {
        slot_count(&self.period)
    }

    /// The worst-case execution time in whole slots.
    pub fn wcet_slots(&self) -> usize {
        slot_count(&self.wcet)
    }

    /// The relative deadline in whole slots.
    pub fn deadline_slots(&self) -> usize {
        slot_count(&self.deadline)
    }
}

/// Build a Liu & Layland base-model task: deadline = period ("each
/// request must be completed before the next request for the same task
/// occurs" — Liu & Layland 1973 §2, assumption A2).
pub fn base_model_task(id: TaskId, wcet_seconds: f64, period_seconds: f64) -> PeriodicTask {
    PeriodicTask {
        id,
        period: Quantity::from_unit(period_seconds, &unit::SECOND),
        wcet: Quantity::from_unit(wcet_seconds, &unit::SECOND),
        deadline: Quantity::from_unit(period_seconds, &unit::SECOND),
    }
}

// ---------------------------------------------------------------------------
// Utilization and the schedulability bounds (Liu & Layland 1973 §5, §7)
// ---------------------------------------------------------------------------

/// The processor utilization factor `U = Σ Ci/Ti` — Liu & Layland
/// (1973) §4, a dimensionless [`Quantity`] (`unit::UNITLESS`). Each
/// summand is the *typed* dimensionless quotient of two time quantities,
/// and the running sum stays a typed `UNITLESS` quantity rather than
/// dropping to a bare float.
pub fn utilization(tasks: &[PeriodicTask]) -> Quantity {
    tasks
        .iter()
        .fold(Quantity::from_unit(0.0, &unit::UNITLESS), |acc, t| {
            let share = t.wcet.div(&t.period);
            assert!(
                share.is_dimensionless(),
                "Ci/Ti must be dimensionless (time over time)"
            );
            acc.add(&share)
                .expect("dimensionless summands share the UNITLESS dimension")
        })
}

/// Liu & Layland (1973) Theorem 5: the least upper utilization bound
/// for rate-monotonic (fixed-priority, rate-ordered) scheduling of `n`
/// tasks is `U_n = n(2^(1/n) − 1)` — a dimensionless [`Quantity`]
/// (`unit::UNITLESS`). The closed form is evaluated in the numeric
/// kernel (raw `f64`) and wrapped as a typed quantity at the boundary.
pub fn rm_utilization_bound(n: usize) -> Quantity {
    let n_f = n as f64;
    let bound = n_f * ((2.0_f64).powf(1.0 / n_f) - 1.0);
    Quantity::from_unit(bound, &unit::UNITLESS)
}

/// Liu & Layland (1973) §7, Theorem 7: the deadline-driven (earliest-
/// deadline-first) algorithm is feasible if and only if `U ≤ 1` — full
/// processor utilization; the bound is exactly one, a dimensionless
/// [`Quantity`] (`unit::UNITLESS`).
pub fn edf_utilization_bound() -> Quantity {
    Quantity::from_unit(1.0, &unit::UNITLESS)
}

/// The rate-monotonic admission test: `U ≤ n(2^(1/n) − 1)` — Liu &
/// Layland (1973) Theorem 5. Sufficient, not necessary (the paper's own
/// §3 example with `C2 = 2` exceeds the bound yet is schedulable). The
/// comparison runs through the typed `Quantity` ordering (both sides are
/// dimensionless), never a bare float.
pub fn rm_admits(tasks: &[PeriodicTask]) -> bool {
    utilization(tasks) <= rm_utilization_bound(tasks.len())
}

// ---------------------------------------------------------------------------
// The ready-queue simulator (Liu & Layland 1973 §3, §7)
// ---------------------------------------------------------------------------

/// The pluggable dispatch orders — Liu & Layland (1973).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyOrder {
    /// Rate-monotonic: static priority by rate — the shorter the
    /// period, the higher the priority (Liu & Layland 1973 §3).
    RateMonotonic,
    /// Deadline-driven (earliest-deadline-first): dynamic priority by
    /// nearest absolute deadline (Liu & Layland 1973 §7).
    EarliestDeadlineFirst,
}

/// One released, not-yet-complete job of a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveJob {
    /// Execution slots still owed.
    pub remaining_slots: usize,
    /// The slot by which the job must have completed.
    pub absolute_deadline_slot: usize,
}

/// The scheduler's state — the engine `Situation`: the typed task set,
/// the elapsed slot count, the running job's task, the per-task active
/// job, and each task's next release slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerSituation {
    /// The periodic task set under schedule.
    pub tasks: Vec<PeriodicTask>,
    /// Slots elapsed since the synchronous start.
    pub elapsed_slots: usize,
    /// The task whose job currently holds the processor.
    pub running: Option<TaskId>,
    /// The active (released, unfinished) job per task, indexed by task.
    pub jobs: Vec<Option<ActiveJob>>,
    /// The slot of each task's next periodic release, indexed by task.
    pub next_release_slot: Vec<usize>,
}

impl Situation for SchedulerSituation {}

/// One scheduling event — the engine `Action` vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerAction {
    /// A new job of the task becomes ready — the periodic request of
    /// Liu & Layland (1973) §2.
    Release(TaskId),
    /// The policy grants the processor to the task's job.
    Dispatch(TaskId),
    /// The running job is suspended in favour of a higher-priority one
    /// — the preemptive model Liu & Layland (1973) §2 assume.
    Preempt(TaskId),
    /// The running job finishes its execution demand.
    Complete(TaskId),
}

impl Action for SchedulerAction {
    type Sit = SchedulerSituation;
}

/// A finished simulation: the typed action trace plus every recorded
/// deadline miss `(task, slot)`.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationTrace {
    /// The actions in occurrence order.
    pub actions: Vec<SchedulerAction>,
    /// Deadline misses, each as the missing task and the missed slot.
    pub deadline_misses: Vec<(TaskId, usize)>,
}

impl SimulationTrace {
    /// Did every job meet its deadline?
    pub fn met_all_deadlines(&self) -> bool {
        self.deadline_misses.is_empty()
    }

    /// The number of `Preempt` actions in the trace.
    pub fn preemption_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|a| matches!(a, SchedulerAction::Preempt(_)))
            .count()
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// The hyperperiod — the least common multiple of the task periods.
/// With synchronous release, a periodic schedule repeats with this
/// period, so simulating one hyperperiod decides feasibility (Liu &
/// Layland 1973 §2's periodicity assumption).
pub fn hyperperiod_slots(tasks: &[PeriodicTask]) -> usize {
    tasks
        .iter()
        .map(|t| t.period_slots())
        .fold(1, |acc, p| acc / gcd(acc, p) * p)
}

/// Simulate the task set under the given dispatch order for one
/// hyperperiod from a synchronous start (all first releases at slot 0 —
/// Liu & Layland 1973 Theorem 1's critical instant, when every request
/// arrives together with all higher-priority requests).
pub fn simulate_periodic(tasks: &[PeriodicTask], policy: PolicyOrder) -> SimulationTrace {
    let horizon = hyperperiod_slots(tasks);
    let mut sit = SchedulerSituation {
        tasks: tasks.to_vec(),
        elapsed_slots: 0,
        running: None,
        jobs: vec![None; tasks.len()],
        next_release_slot: vec![0; tasks.len()],
    };
    let mut actions: Vec<SchedulerAction> = Vec::new();
    let mut deadline_misses: Vec<(TaskId, usize)> = Vec::new();

    for t in 0..horizon {
        sit.elapsed_slots = t;

        // 1. Record deadline misses BEFORE releases, so an unfinished
        //    job is charged its miss before the next release of the
        //    same task replaces it (deadline = period in the base model).
        for (i, task) in tasks.iter().enumerate() {
            if let Some(job) = &sit.jobs[i]
                && job.absolute_deadline_slot == t
                && job.remaining_slots > 0
            {
                deadline_misses.push((task.id, t));
            }
        }

        // 2. Periodic releases (Liu & Layland 1973 §2: one request per
        //    period).
        for (i, task) in tasks.iter().enumerate() {
            if sit.next_release_slot[i] == t {
                sit.jobs[i] = Some(ActiveJob {
                    remaining_slots: task.wcet_slots(),
                    absolute_deadline_slot: t + task.deadline_slots(),
                });
                sit.next_release_slot[i] += task.period_slots();
                actions.push(SchedulerAction::Release(task.id));
            }
        }

        // 3. The policy chooses among ready jobs. Ties break by task
        //    index — a deterministic total order; the Liu & Layland
        //    bounds hold for any fixed tie-break.
        let chosen: Option<TaskId> = tasks
            .iter()
            .enumerate()
            .filter(|(i, _)| sit.jobs[*i].is_some())
            .min_by_key(|(i, task)| match policy {
                PolicyOrder::RateMonotonic => (task.period_slots(), *i),
                PolicyOrder::EarliestDeadlineFirst => (
                    sit.jobs[*i]
                        .as_ref()
                        .map(|j| j.absolute_deadline_slot)
                        .unwrap_or(usize::MAX),
                    *i,
                ),
            })
            .map(|(_, task)| task.id);

        // 4. Dispatch / preempt bookkeeping.
        if chosen != sit.running {
            if let (Some(prev), Some(_)) = (sit.running, chosen)
                && sit.jobs[prev.0].is_some()
            {
                actions.push(SchedulerAction::Preempt(prev));
            }
            if let Some(c) = chosen {
                actions.push(SchedulerAction::Dispatch(c));
            }
        }
        sit.running = chosen;

        // 5. Run one slot.
        if let Some(c) = chosen
            && let Some(job) = sit.jobs[c.0].as_mut()
        {
            job.remaining_slots -= 1;
            if job.remaining_slots == 0 {
                actions.push(SchedulerAction::Complete(c));
                sit.jobs[c.0] = None;
                sit.running = None;
            }
        }
    }

    SimulationTrace {
        actions,
        deadline_misses,
    }
}

// ---------------------------------------------------------------------------
// The Liu & Layland §3 example task set
// ---------------------------------------------------------------------------

/// τ1's execution time `C1 = 1 s` — Liu & Layland (1973) §3 example.
pub const LL_TASK1_WCET_SECONDS: f64 = 1.0;

/// τ1's period `T1 = 2 s` — Liu & Layland (1973) §3 example.
pub const LL_TASK1_PERIOD_SECONDS: f64 = 2.0;

/// τ2's execution time `C2 = 1 s` — Liu & Layland (1973) §3 example.
pub const LL_TASK2_WCET_SECONDS: f64 = 1.0;

/// τ2's period `T2 = 5 s` — Liu & Layland (1973) §3 example.
pub const LL_TASK2_PERIOD_SECONDS: f64 = 5.0;

/// The §3 continuation: `C2` increased to 2 s — Liu & Layland use it to
/// show a task set above the Theorem 5 bound (U = 0.9) that is still
/// rate-monotonic-schedulable: the bound is sufficient, not necessary.
pub const LL_TASK2_INCREASED_WCET_SECONDS: f64 = 2.0;

/// The Liu & Layland (1973) §3 two-task example: τ1 = (C 1 s, T 2 s),
/// τ2 = (C 1 s, T 5 s); `U = 0.7`, below the two-task bound
/// `U_2 = 2(√2 − 1) ≈ 0.828`.
pub fn ll_example_task_set() -> Vec<PeriodicTask> {
    vec![
        base_model_task(TaskId(0), LL_TASK1_WCET_SECONDS, LL_TASK1_PERIOD_SECONDS),
        base_model_task(TaskId(1), LL_TASK2_WCET_SECONDS, LL_TASK2_PERIOD_SECONDS),
    ]
}

/// The §3 continuation with `C2 = 2 s`: `U = 0.9` exceeds `U_2 ≈ 0.828`
/// yet the set remains rate-monotonic-schedulable — and its trace
/// exercises `Preempt` (τ1's slot-2 release preempts τ2).
pub fn ll_increased_task_set() -> Vec<PeriodicTask> {
    vec![
        base_model_task(TaskId(0), LL_TASK1_WCET_SECONDS, LL_TASK1_PERIOD_SECONDS),
        base_model_task(
            TaskId(1),
            LL_TASK2_INCREASED_WCET_SECONDS,
            LL_TASK2_PERIOD_SECONDS,
        ),
    ]
}

// ---------------------------------------------------------------------------
// Priority inversion and inheritance (Sha, Rajkumar & Lehoczky 1990)
// ---------------------------------------------------------------------------

/// An ordinal dispatch rank — Sha, Rajkumar & Lehoczky (1990): the rank
/// by which the processor is granted; higher rank preempts lower. An
/// ordinal, not a quantity — only its order is meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

/// One phase of a job's execution script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionSegment {
    /// Slots of computation touching no shared resource.
    Normal(usize),
    /// Slots inside the critical section guarding the one shared
    /// resource — Sha et al. (1990)'s semaphore-protected section.
    Critical(usize),
}

impl ExecutionSegment {
    fn slots(&self) -> usize {
        match self {
            ExecutionSegment::Normal(s) | ExecutionSegment::Critical(s) => *s,
        }
    }
}

/// A one-shot job with an arrival time, a base priority, and an
/// execution script — the job model of Sha et al. (1990) §II.
#[derive(Debug, Clone, PartialEq)]
pub struct SporadicJob {
    /// The job's identity.
    pub id: TaskId,
    /// The job's assigned (base) priority.
    pub priority: Priority,
    /// The slot at which the job becomes ready.
    pub arrival_slot: usize,
    /// The job's execution script, run in order.
    pub segments: Vec<ExecutionSegment>,
}

impl SporadicJob {
    fn total_slots(&self) -> usize {
        self.segments.iter().map(ExecutionSegment::slots).sum()
    }
}

/// The synchronization protocol in force — Sha et al. (1990).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockingProtocol {
    /// Plain priority scheduling with an unmanaged semaphore — the §II
    /// scenario in which priority inversion is unbounded (prolonged
    /// arbitrarily by medium-priority preemption of the blocker).
    NoInheritance,
    /// The basic priority-inheritance protocol (§IV): a job holding the
    /// resource executes at the highest priority among the jobs it
    /// blocks, so blocking is bounded by critical-section length.
    BasicPriorityInheritance,
}

/// A job's progress through its script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobProgress {
    /// Index of the segment currently being executed.
    pub segment_index: usize,
    /// Slots left in the current segment.
    pub slots_left: usize,
    /// Whether the job currently holds the shared resource.
    pub holds_resource: bool,
    /// The slot after the job's last executed slot, once finished.
    pub completion_slot: Option<usize>,
}

/// The inversion simulator's state — a `Situation` over one shared
/// resource (Sha et al. 1990 use a single semaphore in the classic
/// scenario).
#[derive(Debug, Clone, PartialEq)]
pub struct InversionSituation {
    /// Slots elapsed.
    pub elapsed_slots: usize,
    /// The job currently holding the processor.
    pub running: Option<TaskId>,
    /// The job currently holding the shared resource.
    pub holder: Option<TaskId>,
    /// Per-job progress, indexed by job position.
    pub progress: Vec<JobProgress>,
}

impl Situation for InversionSituation {}

/// A finished inversion simulation: the typed action trace plus each
/// job's completion slot (the slot after its last executed slot).
#[derive(Debug, Clone, PartialEq)]
pub struct InversionTrace {
    /// The actions in occurrence order.
    pub actions: Vec<SchedulerAction>,
    /// Completion slot per job, indexed by job position.
    pub completion_slot: Vec<Option<usize>>,
}

fn is_finished(job: &SporadicJob, p: &JobProgress) -> bool {
    p.segment_index >= job.segments.len()
}

fn wants_resource(job: &SporadicJob, p: &JobProgress) -> bool {
    !is_finished(job, p)
        && matches!(job.segments[p.segment_index], ExecutionSegment::Critical(_))
        && !p.holds_resource
}

/// Simulate one-shot jobs sharing a single resource under preemptive
/// priority scheduling, with or without basic priority inheritance —
/// Sha et al. (1990) §II (the inversion scenario) and §IV (the
/// protocol). The horizon is structural: the sum of all execution
/// demand plus the latest arrival (every job finishes within it when no
/// job starves, and a starved job is reported as unfinished).
pub fn simulate_with_shared_resource(
    jobs: &[SporadicJob],
    protocol: LockingProtocol,
) -> InversionTrace {
    let horizon: usize = jobs.iter().map(SporadicJob::total_slots).sum::<usize>()
        + jobs.iter().map(|j| j.arrival_slot).max().unwrap_or(0);
    let mut sit = InversionSituation {
        elapsed_slots: 0,
        running: None,
        holder: None,
        progress: jobs
            .iter()
            .map(|j| JobProgress {
                segment_index: 0,
                slots_left: j.segments.first().map(ExecutionSegment::slots).unwrap_or(0),
                holds_resource: false,
                completion_slot: None,
            })
            .collect(),
    };
    let mut actions: Vec<SchedulerAction> = Vec::new();

    for t in 0..horizon {
        sit.elapsed_slots = t;

        // Arrivals.
        for job in jobs {
            if job.arrival_slot == t {
                actions.push(SchedulerAction::Release(job.id));
            }
        }

        // Effective priorities — under basic priority inheritance the
        // resource holder executes at the highest priority among the
        // jobs blocked on the resource it holds (Sha et al. 1990 §IV).
        let blocked_max: Option<Priority> = jobs
            .iter()
            .enumerate()
            .filter(|(i, job)| {
                job.arrival_slot <= t
                    && !is_finished(job, &sit.progress[*i])
                    && wants_resource(job, &sit.progress[*i])
                    && sit.holder.is_some()
                    && sit.holder != Some(job.id)
            })
            .map(|(_, job)| job.priority)
            .max();
        let effective = |i: usize, job: &SporadicJob| -> Priority {
            if protocol == LockingProtocol::BasicPriorityInheritance
                && sit.progress[i].holds_resource
                && let Some(b) = blocked_max
            {
                return job.priority.max(b);
            }
            job.priority
        };

        // Runnable = arrived, unfinished, and not blocked on the
        // resource. Ties cannot arise in the fixture (Sha et al. assume
        // distinct priorities); the index tie-break is a deterministic
        // total order for completeness.
        let chosen: Option<TaskId> = jobs
            .iter()
            .enumerate()
            .filter(|(i, job)| {
                job.arrival_slot <= t
                    && !is_finished(job, &sit.progress[*i])
                    && !(wants_resource(job, &sit.progress[*i])
                        && sit.holder.is_some()
                        && sit.holder != Some(job.id))
            })
            .max_by(|(i, a), (j, b)| {
                effective(*i, a).cmp(&effective(*j, b)).then(j.cmp(i)) // lower index wins on (never-exercised) ties
            })
            .map(|(_, job)| job.id);

        // Dispatch / preempt bookkeeping.
        if chosen != sit.running {
            if let (Some(prev), Some(_)) = (sit.running, chosen) {
                let prev_index = jobs.iter().position(|j| j.id == prev);
                if let Some(pi) = prev_index
                    && !is_finished(&jobs[pi], &sit.progress[pi])
                {
                    actions.push(SchedulerAction::Preempt(prev));
                }
            }
            if let Some(c) = chosen {
                actions.push(SchedulerAction::Dispatch(c));
            }
        }
        sit.running = chosen;

        // Execute one slot.
        if let Some(c) = chosen {
            let i = jobs
                .iter()
                .position(|j| j.id == c)
                .expect("chosen job must exist");
            let job = &jobs[i];
            // Entering a critical section acquires the free resource.
            if wants_resource(job, &sit.progress[i]) {
                sit.holder = Some(c);
                sit.progress[i].holds_resource = true;
            }
            sit.progress[i].slots_left -= 1;
            if sit.progress[i].slots_left == 0 {
                // Leaving a critical section releases the resource.
                if matches!(
                    job.segments[sit.progress[i].segment_index],
                    ExecutionSegment::Critical(_)
                ) {
                    sit.holder = None;
                    sit.progress[i].holds_resource = false;
                }
                sit.progress[i].segment_index += 1;
                if let Some(next) = job.segments.get(sit.progress[i].segment_index) {
                    sit.progress[i].slots_left = next.slots();
                } else {
                    sit.progress[i].completion_slot = Some(t + 1);
                    actions.push(SchedulerAction::Complete(c));
                    sit.running = None;
                }
            }
        }
    }

    InversionTrace {
        actions,
        completion_slot: sit.progress.iter().map(|p| p.completion_slot).collect(),
    }
}

// ---------------------------------------------------------------------------
// The Sha, Rajkumar & Lehoczky (1990) three-job fixture
// ---------------------------------------------------------------------------
//
// The scenario of Sha et al. (1990) §II: a low-priority job locks the
// semaphore; the high-priority job preempts it and blocks on the
// semaphore; a medium-priority job then preempts the low-priority
// blocker, prolonging the high-priority job's blocking arbitrarily.
// The slot durations below are the engine's minimal structural
// instantiation of that cited scenario — any instantiation with the
// medium job's burst longer than one critical section shows the same
// unbounded-inversion signature.

/// The low job's ordinal rank (Sha et al. 1990 §II job J3).
pub const SHA_LOW_PRIORITY: Priority = Priority(1);
/// The medium job's ordinal rank (Sha et al. 1990 §II job J2).
pub const SHA_MEDIUM_PRIORITY: Priority = Priority(2);
/// The high job's ordinal rank (Sha et al. 1990 §II job J1).
pub const SHA_HIGH_PRIORITY: Priority = Priority(3);

/// The low job arrives first and reaches its critical section alone.
pub const SHA_LOW_ARRIVAL_SLOT: usize = 0;
/// The high job arrives while the low job is inside its critical
/// section — the precondition of the §II scenario.
pub const SHA_HIGH_ARRIVAL_SLOT: usize = 2;
/// The medium job arrives once the high job has blocked on the
/// semaphore — the preemption that makes the inversion unbounded.
pub const SHA_MEDIUM_ARRIVAL_SLOT: usize = 3;

/// The low job's non-critical prelude before it locks the semaphore.
pub const SHA_LOW_PRELUDE_SLOTS: usize = 1;
/// The low job's (outermost) critical-section length — the quantity
/// Sha et al. (1990)'s inheritance bound is stated in.
pub const SHA_LOW_CRITICAL_SECTION_SLOTS: usize = 2;
/// The high job's non-critical prelude before it needs the semaphore.
pub const SHA_HIGH_PRELUDE_SLOTS: usize = 1;
/// The high job's critical-section length.
pub const SHA_HIGH_CRITICAL_SECTION_SLOTS: usize = 1;
/// The high job's non-critical postlude after the critical section.
pub const SHA_HIGH_POSTLUDE_SLOTS: usize = 1;
/// The medium job's compute-bound burst — the §II "indefinitely"
/// surrogate that prolongs the inversion past the inheritance bound.
pub const SHA_MEDIUM_EXECUTION_SLOTS: usize = 10;

/// Fixture position of the low job in [`sha_inversion_jobs`].
pub const SHA_LOW_JOB_INDEX: usize = 0;
/// Fixture position of the medium job in [`sha_inversion_jobs`].
pub const SHA_MEDIUM_JOB_INDEX: usize = 1;
/// Fixture position of the high job in [`sha_inversion_jobs`].
pub const SHA_HIGH_JOB_INDEX: usize = 2;

/// The Sha et al. (1990) §II three-job scenario as typed jobs.
pub fn sha_inversion_jobs() -> Vec<SporadicJob> {
    vec![
        SporadicJob {
            id: TaskId(SHA_LOW_JOB_INDEX),
            priority: SHA_LOW_PRIORITY,
            arrival_slot: SHA_LOW_ARRIVAL_SLOT,
            segments: vec![
                ExecutionSegment::Normal(SHA_LOW_PRELUDE_SLOTS),
                ExecutionSegment::Critical(SHA_LOW_CRITICAL_SECTION_SLOTS),
            ],
        },
        SporadicJob {
            id: TaskId(SHA_MEDIUM_JOB_INDEX),
            priority: SHA_MEDIUM_PRIORITY,
            arrival_slot: SHA_MEDIUM_ARRIVAL_SLOT,
            segments: vec![ExecutionSegment::Normal(SHA_MEDIUM_EXECUTION_SLOTS)],
        },
        SporadicJob {
            id: TaskId(SHA_HIGH_JOB_INDEX),
            priority: SHA_HIGH_PRIORITY,
            arrival_slot: SHA_HIGH_ARRIVAL_SLOT,
            segments: vec![
                ExecutionSegment::Normal(SHA_HIGH_PRELUDE_SLOTS),
                ExecutionSegment::Critical(SHA_HIGH_CRITICAL_SECTION_SLOTS),
                ExecutionSegment::Normal(SHA_HIGH_POSTLUDE_SLOTS),
            ],
        },
    ]
}

/// The priority-inheritance completion bound for the fixture's high
/// job — Sha et al. (1990), Priority Inheritance Protocols: under the
/// basic protocol a job is blocked for at most the duration of one
/// (outermost) critical section per lower-priority job per semaphore;
/// with one semaphore and one critical-section-holding lower job, the
/// high job must complete by
/// `arrival + own demand + one low critical section`.
pub fn sha_high_completion_bound_slot() -> usize {
    SHA_HIGH_ARRIVAL_SLOT
        + SHA_HIGH_PRELUDE_SLOTS
        + SHA_HIGH_CRITICAL_SECTION_SLOTS
        + SHA_HIGH_POSTLUDE_SLOTS
        + SHA_LOW_CRITICAL_SECTION_SLOTS
}
