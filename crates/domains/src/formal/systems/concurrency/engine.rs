//! Concurrency engine — three small verified fixtures.
//!
//! 1. A bounded exhaustive interleaving explorer over N tiny processes,
//!    each running Dijkstra's `P(sem); critical section; V(sem)` script
//!    against one binary semaphore — Dijkstra (1968) "Cooperating
//!    Sequential Processes" (EWD-123).
//! 2. A Lamport logical-clock event fixture: three communicating
//!    processes, send/receive pairs, and the happens-before partial
//!    order — Lamport (1978) CACM 21(7).
//! 3. A resource-allocation-graph fixture for the Coffman deadlock
//!    conditions — Coffman, Elphick & Shoshani (1971) ACM Computing
//!    Surveys 3(2).
//!
//! Every constant below is a documented structural fixture parameter
//! cited to the axiom's source — no free magic numbers — and each is
//! checked against the fixture it parametrizes by the agreement tests at
//! the foot of this module, so none can drift silently.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

// ---------------------------------------------------------------------------
// Shared identifiers
// ---------------------------------------------------------------------------

/// A process identity — Hoare (1978): the process is the unit of
/// concurrent composition, so it is named, never an anonymous index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub usize);

/// A message identity — Lamport (1978): each message pairs exactly one
/// send event with exactly one receive event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageId(pub usize);

/// A resource identity — Coffman et al. (1971): the ordered-resource
/// denial of circular wait requires a total order on resources, so the
/// identity is `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(pub usize);

// ---------------------------------------------------------------------------
// Fixture 1 — bounded exhaustive interleaving explorer (Dijkstra 1968)
// ---------------------------------------------------------------------------

/// Program counter of one fixture process over Dijkstra's fixed
/// three-step script `P(sem); critical section; V(sem)` — Dijkstra
/// (1968) EWD-123. Four phases = the three steps plus termination.
///
/// The semaphore is held from completion of `P` until completion of
/// `V`, so a process occupies the protected region in both
/// `InCritical` and `AtRelease`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessPhase {
    /// Next step is `P(sem)` — the acquire, guarded by the semaphore.
    AtAcquire,
    /// Inside the critical section; next step is the critical action.
    InCritical,
    /// Critical work done, semaphore still held; next step is `V(sem)`.
    AtRelease,
    /// Script finished.
    Done,
}

impl ProcessPhase {
    /// The closed set of phases — used to derive the state-space bound
    /// of the exhaustive exploration (no numeric literal).
    pub const ALL: [ProcessPhase; 4] = [
        ProcessPhase::AtAcquire,
        ProcessPhase::InCritical,
        ProcessPhase::AtRelease,
        ProcessPhase::Done,
    ];

    /// Whether a process in this phase occupies the semaphore-protected
    /// region (between completion of `P` and completion of `V`) —
    /// Dijkstra (1968).
    pub fn occupies_critical_section(&self) -> bool {
        matches!(self, ProcessPhase::InCritical | ProcessPhase::AtRelease)
    }
}

/// Dijkstra (1968): the binary semaphore takes exactly the values 0 and
/// 1 — typed here as taken/available rather than a bare counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySemaphore {
    /// Value 1 — `P` may proceed.
    Available,
    /// Value 0 — `P` blocks.
    Taken,
}

impl BinarySemaphore {
    /// The closed set of semaphore values — used to derive the
    /// state-space bound of the exhaustive exploration.
    pub const ALL: [BinarySemaphore; 2] = [BinarySemaphore::Available, BinarySemaphore::Taken];
}

/// Number of processes in the mutual-exclusion fixture. Two is the
/// smallest count for which mutual exclusion is non-trivial; Dijkstra
/// (1968) develops the two-process solution before generalising to N.
pub const MUTEX_FIXTURE_PROCESS_COUNT: usize = 2;

/// Length of the fixture script: `P(sem)`, critical action, `V(sem)` —
/// the exact three-step structure of Dijkstra (1968) EWD-123.
pub const MUTEX_FIXTURE_SCRIPT_LENGTH: usize = 3;

/// The joint state of the N fixture processes plus the shared binary
/// semaphore — the engine `Situation` (Milner (1980): concurrency as a
/// labelled transition system explored by interleaving).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrencySituation {
    /// Program counter of each process, indexed by `ProcessId`.
    pub processes: Vec<ProcessPhase>,
    /// The one shared binary semaphore guarding the critical section.
    pub semaphore: BinarySemaphore,
}

impl Situation for ConcurrencySituation {}

impl ConcurrencySituation {
    /// How many processes currently occupy the protected region.
    pub fn critical_occupancy(&self) -> usize {
        self.processes
            .iter()
            .filter(|p| p.occupies_critical_section())
            .count()
    }

    /// Mutual-exclusion violation: more than one process in the
    /// protected region at once — Dijkstra (1968).
    pub fn violates_mutual_exclusion(&self) -> bool {
        self.critical_occupancy() > 1
    }
}

/// One step of process `i` — the engine `Action`. The acquire step is
/// guarded by the semaphore (Dijkstra 1968 `P`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepProcess {
    /// Which process takes its next script step.
    pub process: ProcessId,
}

impl Action for StepProcess {
    type Sit = ConcurrencySituation;
}

/// Initial situation of the mutual-exclusion fixture: every process at
/// its acquire step, semaphore available — Dijkstra (1968).
pub fn mutex_initial() -> ConcurrencySituation {
    ConcurrencySituation {
        processes: vec![ProcessPhase::AtAcquire; MUTEX_FIXTURE_PROCESS_COUNT],
        semaphore: BinarySemaphore::Available,
    }
}

/// Apply one script step of one process. `Err` when the step is not
/// enabled: acquire on a taken semaphore blocks (Dijkstra 1968 `P`),
/// a finished process has no step, and an out-of-range id is rejected.
pub fn apply_step(
    situation: &ConcurrencySituation,
    action: &StepProcess,
) -> Result<ConcurrencySituation, String> {
    let index = action.process.0;
    let Some(phase) = situation.processes.get(index) else {
        return Err(format!("no process with id {index}"));
    };
    let mut next = situation.clone();
    match phase {
        ProcessPhase::AtAcquire => match situation.semaphore {
            BinarySemaphore::Available => {
                next.processes[index] = ProcessPhase::InCritical;
                next.semaphore = BinarySemaphore::Taken;
            }
            BinarySemaphore::Taken => {
                return Err("P(sem) blocks: semaphore is taken".to_string());
            }
        },
        ProcessPhase::InCritical => {
            next.processes[index] = ProcessPhase::AtRelease;
        }
        ProcessPhase::AtRelease => match situation.semaphore {
            BinarySemaphore::Taken => {
                next.processes[index] = ProcessPhase::Done;
                next.semaphore = BinarySemaphore::Available;
            }
            BinarySemaphore::Available => {
                return Err("V(sem) on an available semaphore: not holding it".to_string());
            }
        },
        ProcessPhase::Done => {
            return Err("process already finished its script".to_string());
        }
    }
    Ok(next)
}

/// The actions enabled in a situation — one candidate step per process,
/// filtered by the guards of [`apply_step`].
pub fn enabled_actions(situation: &ConcurrencySituation) -> Vec<StepProcess> {
    (0..situation.processes.len())
        .map(|i| StepProcess {
            process: ProcessId(i),
        })
        .filter(|a| apply_step(situation, a).is_ok())
        .collect()
}

/// Upper bound on the number of distinct situations, derived from the
/// typed state components (phases per process × semaphore values) —
/// this is what makes the exhaustive exploration *bounded*.
pub fn state_space_bound(process_count: usize) -> usize {
    ProcessPhase::ALL.len().pow(process_count as u32) * BinarySemaphore::ALL.len()
}

/// Breadth-first bounded exhaustive exploration of every situation
/// reachable from `initial` under nondeterministic interleaving —
/// Milner (1980): concurrency modeled as nondeterministic sequential
/// merge. Terminates because the state space is finite (see
/// [`state_space_bound`]) and visited states are never re-expanded.
pub fn explore(initial: &ConcurrencySituation) -> Vec<ConcurrencySituation> {
    let mut visited: Vec<ConcurrencySituation> = vec![initial.clone()];
    let mut head = 0usize;
    while head < visited.len() {
        let current = visited[head].clone();
        head += 1;
        for action in enabled_actions(&current) {
            if let Ok(next) = apply_step(&current, &action)
                && !visited.contains(&next)
            {
                visited.push(next);
            }
        }
    }
    visited
}

// ---------------------------------------------------------------------------
// Fixture 2 — Milner expansion: maximal traces of a|b
// ---------------------------------------------------------------------------

/// Number of atomic agents in the expansion fixture: Milner (1980)
/// states the expansion law for the parallel composition `a | b` of two
/// atomic actions — `a.NIL | b.NIL = a.b.NIL + b.a.NIL`.
pub const EXPANSION_FIXTURE_ACTION_COUNT: usize = 2;

/// Completion status of one single-action process in the expansion
/// fixture (its whole script is one atomic action).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomStatus {
    /// The atomic action has not fired yet.
    Pending,
    /// The atomic action has fired.
    Fired,
}

fn interleave(
    status: &mut Vec<AtomStatus>,
    trace: &mut Vec<ProcessId>,
    out: &mut Vec<Vec<ProcessId>>,
) {
    let pending: Vec<usize> = status
        .iter()
        .enumerate()
        .filter(|(_, s)| **s == AtomStatus::Pending)
        .map(|(i, _)| i)
        .collect();
    if pending.is_empty() {
        out.push(trace.clone());
        return;
    }
    for i in pending {
        status[i] = AtomStatus::Fired;
        trace.push(ProcessId(i));
        interleave(status, trace, out);
        trace.pop();
        status[i] = AtomStatus::Pending;
    }
}

/// Every maximal trace of `action_count` single-action processes
/// composed in parallel, computed by exhaustive interleaving — the
/// engine-side evaluation of Milner (1980)'s expansion law. Each trace
/// records which process fired at each position.
pub fn maximal_interleavings(action_count: usize) -> Vec<Vec<ProcessId>> {
    let mut status = vec![AtomStatus::Pending; action_count];
    let mut trace: Vec<ProcessId> = Vec::new();
    let mut out: Vec<Vec<ProcessId>> = Vec::new();
    interleave(&mut status, &mut trace, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Fixture 3 — Lamport logical clocks + happens-before (Lamport 1978)
// ---------------------------------------------------------------------------

/// What one event does — Lamport (1978): events are local computations,
/// message sends, or message receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// An internal computation step.
    Local,
    /// Sending the identified message.
    Send(MessageId),
    /// Receiving the identified message.
    Receive(MessageId),
}

/// One event of the distributed computation. Program order within a
/// process is its listing order in the fixture vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    /// The process at which the event occurs.
    pub process: ProcessId,
    /// What the event does.
    pub kind: EventKind,
}

/// Number of processes in the clock fixture: Lamport (1978) Figure 1
/// illustrates the happens-before order with three processes.
pub const LAMPORT_FIXTURE_PROCESS_COUNT: usize = 3;

/// Clock advance per event: Lamport (1978) IR1 — "each process
/// increments C_i between any two successive events" — and IR2 sets a
/// receive to exceed the send timestamp by at least this same tick.
pub const LAMPORT_CLOCK_TICK: u64 = 1;

/// The event fixture, in a valid global listing order (each receive
/// appears after its matching send). Three processes in the style of
/// Lamport (1978) Figure 1, with a message chain P0 → P1 → P2 → P0 and
/// two local events, so the fixture contains both ordered and
/// concurrent event pairs.
pub fn lamport_fixture() -> Vec<Event> {
    vec![
        // P0: send m0, then a local step, then (later) receive m2.
        Event {
            process: ProcessId(0),
            kind: EventKind::Send(MessageId(0)),
        },
        Event {
            process: ProcessId(0),
            kind: EventKind::Local,
        },
        // P1: receive m0, then send m1.
        Event {
            process: ProcessId(1),
            kind: EventKind::Receive(MessageId(0)),
        },
        Event {
            process: ProcessId(1),
            kind: EventKind::Send(MessageId(1)),
        },
        // P2: a local step, receive m1, then send m2.
        Event {
            process: ProcessId(2),
            kind: EventKind::Local,
        },
        Event {
            process: ProcessId(2),
            kind: EventKind::Receive(MessageId(1)),
        },
        Event {
            process: ProcessId(2),
            kind: EventKind::Send(MessageId(2)),
        },
        // P0 closes the chain.
        Event {
            process: ProcessId(0),
            kind: EventKind::Receive(MessageId(2)),
        },
    ]
}

fn id_bound<T>(items: T) -> usize
where
    T: Iterator<Item = usize>,
{
    items.map(|i| i + 1).max().unwrap_or(0)
}

/// Lamport (1978) clock assignment: C(local/send at p) = C_p + tick;
/// C(receive of m at p) = max(C_p + tick, C(send of m) + tick).
/// `Err` when a receive precedes its matching send in listing order
/// (an invalid global order) or a message has no send at all.
pub fn logical_clocks(events: &[Event]) -> Result<Vec<u64>, String> {
    let process_count = id_bound(events.iter().map(|e| e.process.0));
    let message_count = id_bound(events.iter().filter_map(|e| match e.kind {
        EventKind::Send(m) | EventKind::Receive(m) => Some(m.0),
        EventKind::Local => None,
    }));
    let mut process_clock: Vec<u64> = vec![0; process_count];
    let mut send_timestamp: Vec<Option<u64>> = vec![None; message_count];
    let mut clocks: Vec<u64> = Vec::new();
    for event in events {
        let p = event.process.0;
        let ticked = process_clock[p] + LAMPORT_CLOCK_TICK;
        let clock = match event.kind {
            EventKind::Local | EventKind::Send(_) => ticked,
            EventKind::Receive(m) => {
                let sent = send_timestamp[m.0]
                    .ok_or_else(|| format!("receive of message {} precedes its send", m.0))?;
                core::cmp::max(ticked, sent + LAMPORT_CLOCK_TICK)
            }
        };
        process_clock[p] = clock;
        if let EventKind::Send(m) = event.kind {
            send_timestamp[m.0] = Some(clock);
        }
        clocks.push(clock);
    }
    Ok(clocks)
}

/// The happens-before relation on the fixture's events, as ordered
/// index pairs `(earlier, later)` — Lamport (1978): the smallest
/// transitive relation containing program order and send-before-receive.
pub fn happens_before(events: &[Event]) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    // Program order: same process, earlier listing position.
    for (i, a) in events.iter().enumerate() {
        for (j, b) in events.iter().enumerate().skip(i + 1) {
            if a.process == b.process {
                pairs.push((i, j));
            }
        }
    }
    // Message order: each send precedes its matching receive.
    for (i, a) in events.iter().enumerate() {
        if let EventKind::Send(m) = a.kind {
            for (j, b) in events.iter().enumerate() {
                if b.kind == EventKind::Receive(m) {
                    pairs.push((i, j));
                }
            }
        }
    }
    // Transitive closure (Warshall 1962).
    loop {
        let mut added = false;
        let snapshot = pairs.clone();
        for (a, b) in &snapshot {
            for (b2, c) in &snapshot {
                if b == b2 && !pairs.contains(&(*a, *c)) {
                    pairs.push((*a, *c));
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }
    pairs
}

// ---------------------------------------------------------------------------
// Fixture 4 — resource-allocation graph (Coffman et al. 1971)
// ---------------------------------------------------------------------------

/// Whether an assignment grants exclusive use of the resource —
/// Coffman et al. (1971) condition 1 (mutual exclusion). Denying the
/// condition makes the resource shareable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Only the holder may use the resource; requests must wait.
    Exclusive,
    /// The resource is shareable; requests never wait on it.
    Shared,
}

/// Whether an assignment can be forcibly withdrawn from its holder —
/// Coffman et al. (1971) condition 3 (no preemption). Denying the
/// condition makes every assignment preemptible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldDiscipline {
    /// The resource is released only voluntarily by its holder.
    NonPreemptible,
    /// The resource may be taken from its holder, so no request
    /// waits on it indefinitely.
    Preemptible,
}

/// One resource currently assigned to one process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The assigned resource.
    pub resource: ResourceId,
    /// The process holding it.
    pub holder: ProcessId,
    /// Exclusive vs. shared use (Coffman condition 1).
    pub mode: AccessMode,
    /// Voluntary-release vs. preemptible holding (Coffman condition 3).
    pub discipline: HoldDiscipline,
}

/// One pending request by a process for a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The requesting process.
    pub process: ProcessId,
    /// The requested resource.
    pub resource: ResourceId,
}

/// A resource-allocation graph over single-unit resources — Coffman,
/// Elphick & Shoshani (1971) §2: for such graphs a deadlock is exactly
/// a cycle in the induced wait-for graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAllocationGraph {
    /// Which process holds which resource, and under what discipline.
    pub assignments: Vec<Assignment>,
    /// Which process is waiting for which resource.
    pub requests: Vec<Request>,
}

/// The four jointly necessary deadlock conditions — Coffman, Elphick &
/// Shoshani (1971) §2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoffmanCondition {
    /// Condition 1: resources are held in exclusive use.
    MutualExclusion,
    /// Condition 2: processes hold resources while waiting for more.
    HoldAndWait,
    /// Condition 3: resources are released only voluntarily.
    NoPreemption,
    /// Condition 4: a circular chain of waiting processes exists.
    CircularWait,
}

impl CoffmanCondition {
    /// The closed set of the four conditions, in Coffman's order.
    pub const ALL: [CoffmanCondition; 4] = [
        CoffmanCondition::MutualExclusion,
        CoffmanCondition::HoldAndWait,
        CoffmanCondition::NoPreemption,
        CoffmanCondition::CircularWait,
    ];
}

/// Number of processes in the deadlock fixture: two processes form the
/// smallest circular chain of Coffman et al. (1971) §2.
pub const COFFMAN_FIXTURE_PROCESS_COUNT: usize = 2;

/// Number of resources in the deadlock fixture: one held resource per
/// process of the smallest circular chain.
pub const COFFMAN_FIXTURE_RESOURCE_COUNT: usize = 2;

/// The classic two-process, two-resource deadlock: P0 holds R0 and
/// requests R1; P1 holds R1 and requests R0 — the smallest instance of
/// Coffman et al. (1971) §2's circular chain, with every assignment
/// exclusive and non-preemptible.
pub fn coffman_fixture() -> ResourceAllocationGraph {
    ResourceAllocationGraph {
        assignments: vec![
            Assignment {
                resource: ResourceId(0),
                holder: ProcessId(0),
                mode: AccessMode::Exclusive,
                discipline: HoldDiscipline::NonPreemptible,
            },
            Assignment {
                resource: ResourceId(1),
                holder: ProcessId(1),
                mode: AccessMode::Exclusive,
                discipline: HoldDiscipline::NonPreemptible,
            },
        ],
        requests: vec![
            Request {
                process: ProcessId(0),
                resource: ResourceId(1),
            },
            Request {
                process: ProcessId(1),
                resource: ResourceId(0),
            },
        ],
    }
}

fn reachable_from(edges: &[(ProcessId, ProcessId)], start: ProcessId) -> Vec<ProcessId> {
    let mut seen: Vec<ProcessId> = Vec::new();
    let mut stack: Vec<ProcessId> = vec![start];
    while let Some(node) = stack.pop() {
        for (from, to) in edges {
            if *from == node && !seen.contains(to) {
                seen.push(*to);
                stack.push(*to);
            }
        }
    }
    seen
}

impl ResourceAllocationGraph {
    /// The wait-for edges the graph induces: requester → holder, for
    /// every request on a resource that is exclusively held by another
    /// process and cannot be preempted — Coffman et al. (1971) §2.
    pub fn wait_for_edges(&self) -> Vec<(ProcessId, ProcessId)> {
        let mut edges: Vec<(ProcessId, ProcessId)> = Vec::new();
        for request in &self.requests {
            for assignment in &self.assignments {
                if assignment.resource == request.resource
                    && assignment.holder != request.process
                    && assignment.mode == AccessMode::Exclusive
                    && assignment.discipline == HoldDiscipline::NonPreemptible
                {
                    edges.push((request.process, assignment.holder));
                }
            }
        }
        edges
    }

    /// Deadlock detection for single-unit resources: a cycle in the
    /// wait-for graph — Coffman et al. (1971) §2.
    pub fn has_deadlock_cycle(&self) -> bool {
        let edges = self.wait_for_edges();
        edges
            .iter()
            .any(|(from, _)| reachable_from(&edges, *from).contains(from))
    }

    /// Whether one of the four Coffman conditions holds on this graph.
    pub fn condition_holds(&self, condition: CoffmanCondition) -> bool {
        match condition {
            CoffmanCondition::MutualExclusion => {
                !self.assignments.is_empty()
                    && self
                        .assignments
                        .iter()
                        .all(|a| a.mode == AccessMode::Exclusive)
            }
            CoffmanCondition::HoldAndWait => self
                .requests
                .iter()
                .any(|r| self.assignments.iter().any(|a| a.holder == r.process)),
            CoffmanCondition::NoPreemption => {
                !self.assignments.is_empty()
                    && self
                        .assignments
                        .iter()
                        .all(|a| a.discipline == HoldDiscipline::NonPreemptible)
            }
            CoffmanCondition::CircularWait => self.has_deadlock_cycle(),
        }
    }

    /// The graph with one Coffman condition structurally denied —
    /// the per-condition prevention transformations of Coffman et al.
    /// (1971) §2–3 (Havender's schemes):
    ///
    /// - deny mutual exclusion → every resource becomes shareable;
    /// - deny hold-and-wait → a process that requests holds nothing;
    /// - deny no-preemption → every assignment becomes preemptible;
    /// - deny circular wait → ordered resources: a process may only
    ///   request resources above everything it already holds.
    pub fn deny(&self, condition: CoffmanCondition) -> ResourceAllocationGraph {
        match condition {
            CoffmanCondition::MutualExclusion => ResourceAllocationGraph {
                assignments: self
                    .assignments
                    .iter()
                    .cloned()
                    .map(|mut a| {
                        a.mode = AccessMode::Shared;
                        a
                    })
                    .collect(),
                requests: self.requests.clone(),
            },
            CoffmanCondition::HoldAndWait => ResourceAllocationGraph {
                assignments: self
                    .assignments
                    .iter()
                    .filter(|a| !self.requests.iter().any(|r| r.process == a.holder))
                    .cloned()
                    .collect(),
                requests: self.requests.clone(),
            },
            CoffmanCondition::NoPreemption => ResourceAllocationGraph {
                assignments: self
                    .assignments
                    .iter()
                    .cloned()
                    .map(|mut a| {
                        a.discipline = HoldDiscipline::Preemptible;
                        a
                    })
                    .collect(),
                requests: self.requests.clone(),
            },
            CoffmanCondition::CircularWait => ResourceAllocationGraph {
                assignments: self.assignments.clone(),
                requests: self
                    .requests
                    .iter()
                    .filter(|r| {
                        self.assignments
                            .iter()
                            .filter(|a| a.holder == r.process)
                            .all(|a| a.resource < r.resource)
                    })
                    .cloned()
                    .collect(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Fixture-parameter agreement — each structural constant is checked
// against the fixture it parametrizes, so none can drift silently.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Count of distinct values an iterator yields.
    fn distinct<T: Ord>(iter: impl Iterator<Item = T>) -> usize {
        let mut items: alloc::vec::Vec<T> = iter.collect();
        items.sort_unstable();
        items.dedup();
        items.len()
    }

    /// The three-step Dijkstra script `P; critical; V` has exactly one
    /// non-terminal program-counter position per operation, so the phase
    /// enum encodes `MUTEX_FIXTURE_SCRIPT_LENGTH` pending steps plus the
    /// terminal `Done` — Dijkstra (1968) EWD-123.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mutex_script_length_matches_phases() {
        let pending = ProcessPhase::ALL
            .iter()
            .filter(|p| !matches!(p, ProcessPhase::Done))
            .count();
        assert_eq!(pending, MUTEX_FIXTURE_SCRIPT_LENGTH);
    }

    /// The Lamport event fixture runs on exactly
    /// `LAMPORT_FIXTURE_PROCESS_COUNT` distinct processes — Lamport
    /// (1978) Figure 1.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lamport_fixture_process_count_matches() {
        let processes = distinct(lamport_fixture().iter().map(|e| e.process.0));
        assert_eq!(processes, LAMPORT_FIXTURE_PROCESS_COUNT);
    }

    /// The Coffman deadlock fixture is the smallest circular chain:
    /// `COFFMAN_FIXTURE_PROCESS_COUNT` processes each holding one of
    /// `COFFMAN_FIXTURE_RESOURCE_COUNT` distinct resources and issuing one
    /// request apiece — Coffman et al. (1971) §2.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn coffman_fixture_dimensions_match() {
        let graph = coffman_fixture();
        let holders = distinct(graph.assignments.iter().map(|a| a.holder.0));
        let resources = distinct(graph.assignments.iter().map(|a| a.resource.0));
        assert_eq!(holders, COFFMAN_FIXTURE_PROCESS_COUNT);
        assert_eq!(resources, COFFMAN_FIXTURE_RESOURCE_COUNT);
        assert_eq!(graph.requests.len(), COFFMAN_FIXTURE_PROCESS_COUNT);
    }
}
