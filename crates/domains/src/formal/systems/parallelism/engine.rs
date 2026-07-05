//! Parallelism engine — the work-span (multithreaded computation) DAG
//! and a greedy scheduler, discharged against one canonical fixture:
//! the P-FIB(4) computation of Cormen, Leiserson, Rivest & Stein (2009)
//! *Introduction to Algorithms* 3e, Ch. 27 (Multithreaded Algorithms).
//!
//! A computation is a directed acyclic graph of unit-time **strands**
//! (maximal instruction sequences containing no spawn/sync). Two typed
//! measures summarise it:
//!
//! - **Work** `T1` — the number of strands (serial running time).
//! - **Span** `T∞` — the number of strands on a longest path (the
//!   critical-path length; the running time on unboundedly many
//!   processing elements).
//!
//! A **greedy scheduler** never idles a processing element while a ready
//! strand exists (Graham 1966; 1969; Brent 1974 Lemma 2). On unit-time
//! strands, a step is *complete* when at least `p` strands are ready
//! (execute any `p`) and *incomplete* otherwise (execute all ready). The
//! number of steps is `T_p`.
//!
//! Every constant below is a documented structural fixture parameter
//! cited to CLRS Ch. 27 — no free magic numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use crate::formal::math::quantity::value::Quantity;

// ---------------------------------------------------------------------------
// Fixture parameters (Cormen, Leiserson, Rivest & Stein 2009, Ch. 27)
// ---------------------------------------------------------------------------

/// The Fibonacci index of the canonical fixture: CLRS Ch. 27 develops
/// the multithreaded model on `P-FIB(4)` (Figures 27.2 and 27.4).
pub const FIB_INDEX: u64 = 4;

/// Work `T1` of the `P-FIB(4)` computation DAG: 17 strands — Cormen,
/// Leiserson, Rivest & Stein (2009) *Introduction to Algorithms* 3e,
/// Ch. 27 (the strand count of Figure 27.2/27.4). Re-derived structurally
/// by [`fib_dag`] and cross-checked against this cited constant.
pub const WORK_FIB4: usize = 17;

/// Span `T∞` of the `P-FIB(4)` computation DAG: 8 strands on the longest
/// path — Cormen, Leiserson, Rivest & Stein (2009) *Introduction to
/// Algorithms* 3e, Ch. 27. Re-derived structurally by [`span`].
pub const SPAN_FIB4: usize = 8;

/// One unit-time step — the CLRS Ch. 27 convention that every strand
/// executes in unit time, so Work and Span are *dimensionless* counts of
/// this quantum rather than a physical duration.
pub fn unit_time_step() -> Quantity {
    Quantity::dimensionless(1.0)
}

// ---------------------------------------------------------------------------
// Computation DAG (CLRS Ch. 27 §1)
// ---------------------------------------------------------------------------

/// A strand identity — CLRS Ch. 27: a strand is the unit of the
/// computation DAG, so it is named, never an anonymous index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub usize);

/// The role a strand plays in the `P-FIB` decomposition — CLRS Ch. 27
/// §1: an internal call `P-FIB(n≥2)` decomposes into three strands
/// (initial / continuation / final), and a base call `P-FIB(n≤1)` is a
/// single strand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrandRole {
    /// A base case `P-FIB(n≤1)`: a single strand returning `n`.
    BaseCase,
    /// The initial strand of `P-FIB(n≥2)`: the test, then the spawn of
    /// `P-FIB(n−1)` and the continuation.
    Init,
    /// The continuation strand of `P-FIB(n≥2)`: the call to `P-FIB(n−2)`.
    Continue,
    /// The final strand of `P-FIB(n≥2)`: the sync, then `return x + y`.
    Sync,
}

/// One strand of the computation DAG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strand {
    /// This strand's identity (equal to its index in [`ComputationDag`]).
    pub id: TaskId,
    /// Its role in the `P-FIB` decomposition.
    pub role: StrandRole,
    /// For a `BaseCase` strand, the value `P-FIB` returns (0 or 1);
    /// `None` for control strands, whose result is not a Fibonacci value.
    pub base_value: Option<u64>,
    /// Strands that must complete before this one may run (the DAG's
    /// precedence edges, stored on the target).
    pub preds: Vec<TaskId>,
    /// For a `Sync` strand, the two child-exit strands whose returned
    /// values it sums (`x + y`); empty otherwise.
    pub summands: Vec<TaskId>,
}

/// A multithreaded computation DAG — CLRS Ch. 27 §1. Strands are stored
/// in a construction order that is also a topological order (every
/// predecessor has a strictly smaller index), which lets [`span`] and
/// [`evaluate`] walk the vector directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputationDag {
    /// The strands, indexed by `TaskId`.
    pub strands: Vec<Strand>,
    /// The exit strand of the whole computation — its returned value is
    /// the overall result.
    pub root_exit: TaskId,
}

impl ComputationDag {
    /// Work `T1` — the number of strands (CLRS Ch. 27: serial time).
    pub fn work(&self) -> usize {
        self.strands.len()
    }
}

/// Build the `P-FIB(n)` computation DAG structurally: an internal call
/// (`n ≥ 2`) becomes three strands wired as
/// `init → spawn P-FIB(n−1)`, `init → continue → P-FIB(n−2)`, and both
/// child exits `→ sync`; a base call (`n ≤ 1`) becomes one strand. The
/// 17-strand / span-8 structure of [`WORK_FIB4`]/[`SPAN_FIB4`] emerges
/// from this recursion, it is not hand-placed.
pub fn fib_dag(n: u64) -> ComputationDag {
    let mut strands: Vec<Strand> = Vec::new();
    let root = build_fib_node(n, &mut strands);
    ComputationDag {
        strands,
        root_exit: root.exit,
    }
}

/// The entry and exit strand of one built `P-FIB` subgraph.
struct NodeHandle {
    entry: TaskId,
    exit: TaskId,
}

fn push_strand(
    strands: &mut Vec<Strand>,
    role: StrandRole,
    base_value: Option<u64>,
    preds: Vec<TaskId>,
    summands: Vec<TaskId>,
) -> TaskId {
    let id = TaskId(strands.len());
    strands.push(Strand {
        id,
        role,
        base_value,
        preds,
        summands,
    });
    id
}

fn build_fib_node(n: u64, strands: &mut Vec<Strand>) -> NodeHandle {
    // CLRS Ch. 27: P-FIB(n) for n ≤ 1 returns n directly — one strand.
    if n <= 1 {
        let id = push_strand(
            strands,
            StrandRole::BaseCase,
            Some(n),
            Vec::new(),
            Vec::new(),
        );
        return NodeHandle {
            entry: id,
            exit: id,
        };
    }
    // Internal call: `init` tests and spawns P-FIB(n−1).
    let init = push_strand(strands, StrandRole::Init, None, Vec::new(), Vec::new());
    let child1 = build_fib_node(n - 1, strands);
    // Spawn edge: init → entry of P-FIB(n−1).
    strands[child1.entry.0].preds.push(init);
    // Continuation: init → continue, which calls P-FIB(n−2).
    let cont = push_strand(strands, StrandRole::Continue, None, vec![init], Vec::new());
    let child2 = build_fib_node(n - 2, strands);
    strands[child2.entry.0].preds.push(cont);
    // Sync: both child exits must finish before `return x + y`.
    let sync = push_strand(
        strands,
        StrandRole::Sync,
        None,
        vec![child1.exit, child2.exit],
        vec![child1.exit, child2.exit],
    );
    NodeHandle {
        entry: init,
        exit: sync,
    }
}

/// Span `T∞` — the number of strands on a longest path (CLRS Ch. 27:
/// the critical-path length). Computed by a single forward pass, valid
/// because the strand vector is in topological order.
pub fn span(dag: &ComputationDag) -> usize {
    let mut depth = vec![0usize; dag.strands.len()];
    let mut best = 0usize;
    for (i, strand) in dag.strands.iter().enumerate() {
        let pred_depth = strand.preds.iter().map(|p| depth[p.0]).max().unwrap_or(0);
        depth[i] = pred_depth + 1;
        best = best.max(depth[i]);
    }
    best
}

// ---------------------------------------------------------------------------
// Evaluation (the DAG's dataflow value)
// ---------------------------------------------------------------------------

/// The sequential specification `P-FIB` computes — the ordinary
/// Fibonacci recurrence `F(0)=0, F(1)=1, F(n)=F(n−1)+F(n−2)` (CLRS
/// Ch. 27; OEIS A000045). The DAG's result must equal this.
pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 2..=n {
        let next = a + b;
        a = b;
        b = next;
    }
    b
}

/// Evaluate the computation along a given strand order — the order must
/// be topological (every predecessor precedes its successor), which each
/// greedy [`Schedule`] flattening is. Base strands return their value,
/// sync strands sum their two summands, control strands carry no value.
/// The returned `u64` is the overall computation result (`P-FIB(n)`).
pub fn evaluate_along(dag: &ComputationDag, order: &[TaskId]) -> u64 {
    let mut result: Vec<Option<u64>> = vec![None; dag.strands.len()];
    for id in order {
        let strand = &dag.strands[id.0];
        let value = match strand.role {
            StrandRole::BaseCase => strand.base_value.unwrap_or(0),
            StrandRole::Sync => strand
                .summands
                .iter()
                .map(|s| result[s.0].unwrap_or(0))
                .sum(),
            StrandRole::Init | StrandRole::Continue => 0,
        };
        result[id.0] = Some(value);
    }
    result[dag.root_exit.0].unwrap_or(0)
}

/// Evaluate the computation in its intrinsic topological order (the
/// strand-vector order) — the schedule-independent result.
pub fn evaluate(dag: &ComputationDag) -> u64 {
    let order: Vec<TaskId> = (0..dag.strands.len()).map(TaskId).collect();
    evaluate_along(dag, &order)
}

// ---------------------------------------------------------------------------
// Greedy scheduler (Graham 1966; 1969; Brent 1974; CLRS Ch. 27 §3)
// ---------------------------------------------------------------------------

/// The state of a greedy scheduling run — the engine `Situation`: which
/// strands have completed, and how many processing elements are
/// available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerState {
    /// Completion flag per strand, indexed by `TaskId`.
    pub completed: Vec<bool>,
    /// The processing-element count `p` (≥ 1).
    pub processor_count: usize,
}

impl Situation for SchedulerState {}

/// One greedy dispatch step — the engine `Action`: the strands executed
/// simultaneously in one unit-time step (at most `p` of them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchStep {
    /// The strands dispatched this step.
    pub tasks: Vec<TaskId>,
}

impl Action for DispatchStep {
    type Sit = SchedulerState;
}

/// A complete greedy schedule — the sequence of unit-time dispatch steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    /// `steps[t]` is the set of strands executed at time step `t`.
    pub steps: Vec<DispatchStep>,
}

impl Schedule {
    /// The makespan `T_p` — the number of unit-time steps.
    pub fn makespan(&self) -> usize {
        self.steps.len()
    }

    /// The greatest number of strands executed simultaneously in any
    /// single step — the realised degree of parallelism.
    pub fn max_parallelism(&self) -> usize {
        self.steps.iter().map(|s| s.tasks.len()).max().unwrap_or(0)
    }

    /// The strands in dispatch order — a topological order of the DAG,
    /// suitable for [`evaluate_along`].
    pub fn flatten(&self) -> Vec<TaskId> {
        self.steps
            .iter()
            .flat_map(|s| s.tasks.iter().copied())
            .collect()
    }
}

/// The strands ready to run in a state: not yet completed, with every
/// predecessor completed — CLRS Ch. 27 §3.
pub fn ready_tasks(dag: &ComputationDag, state: &SchedulerState) -> Vec<TaskId> {
    dag.strands
        .iter()
        .filter(|s| !state.completed[s.id.0] && s.preds.iter().all(|p| state.completed[p.0]))
        .map(|s| s.id)
        .collect()
}

/// Schedule `dag` greedily onto `p` processing elements — Graham (1966;
/// 1969); Brent (1974) Lemma 2; CLRS Ch. 27 §3. At each step the greedy
/// rule executes up to `p` ready strands, never idling a processing
/// element while a ready strand remains. `p` is clamped to at least one
/// (a schedule needs a processing element).
pub fn greedy_schedule(dag: &ComputationDag, p: usize) -> Schedule {
    let p = p.max(1);
    let mut state = SchedulerState {
        completed: vec![false; dag.strands.len()],
        processor_count: p,
    };
    let mut steps: Vec<DispatchStep> = Vec::new();
    let mut remaining = dag.strands.len();
    while remaining > 0 {
        let ready = ready_tasks(dag, &state);
        // A DAG always has a ready strand while any remain; guard anyway
        // so the loop can never spin.
        if ready.is_empty() {
            break;
        }
        let dispatched: Vec<TaskId> = ready.into_iter().take(p).collect();
        for id in &dispatched {
            state.completed[id.0] = true;
            remaining -= 1;
        }
        steps.push(DispatchStep { tasks: dispatched });
    }
    Schedule { steps }
}

/// The processing-element counts to probe the greedy bound over:
/// successive powers of two up to and past the span, so the tested grid
/// spans the whole regime from serial (`p = 1`) to critically-limited
/// (`p > T∞`). Structurally derived from `span`, not hand-listed.
pub fn greedy_processor_counts(span: usize) -> Vec<usize> {
    let mut counts = Vec::new();
    let mut p = 1usize;
    loop {
        counts.push(p);
        if p > span {
            break;
        }
        p *= 2;
    }
    counts
}
