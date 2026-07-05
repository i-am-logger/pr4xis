//! Microkernel engine — a minimal kernel-state fixture.
//!
//! The situation is a small kernel state in the sense of Liedtke (1995)
//! SOSP "On µ-Kernel Construction": threads bound to address spaces
//! (§2.1–2.2), endpoint message queues (Klein et al. 2009 SOSP; Brinch
//! Hansen 1970 CACM 13(4) message buffering), and one current thread
//! (the scheduler's choice — Liedtke 1996 CACM 39(9)). The transition
//! function enforces the two kernel guarantees the ontology's axioms
//! talk about:
//!
//! 1. **Address-space isolation** — a thread only touches memory in its
//!    own address space: a `Send` whose buffer lies in a foreign
//!    address space is rejected (Liedtke 1995 §2.1: the address space
//!    is the unit of isolation).
//! 2. **Kernel mediation** — every message passes through an endpoint:
//!    the only representable delivery path is `Send` → endpoint queue →
//!    `Receive`, and each delivered message records the endpoint that
//!    mediated it (Brinch Hansen 1970: all communication passes through
//!    the nucleus's message buffers).
//!
//! Every constant below is a documented structural fixture parameter
//! cited to its source — no free magic numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

// ---------------------------------------------------------------------------
// Identifiers — typed, never bare indices in APIs
// ---------------------------------------------------------------------------

/// A thread identity — Liedtke (1995) §2.2: the thread is the unit of
/// activity, so it is named, never an anonymous index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub usize);

/// An address-space identity — Liedtke (1995) §2.1: the address space
/// is the unit of isolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressSpaceId(pub usize);

/// An endpoint identity — Klein et al. (2009) SOSP: the kernel object
/// messages are sent to and received from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointId(pub usize);

/// One transferred word — Brinch Hansen (1970): the message content the
/// nucleus copies between processes. Typed so a payload is never a bare
/// integer in an API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadWord(pub u64);

// ---------------------------------------------------------------------------
// Kernel objects
// ---------------------------------------------------------------------------

/// A thread control block, reduced to the isolation-relevant binding:
/// Liedtke (1995) §2.2 characterises a thread by its register set
/// *and the address space it executes in*; here the address-space
/// binding plus the mailbox of kernel-delivered messages (which the
/// kernel writes into the thread's own space on `Receive`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadState {
    /// The address space this thread executes in (Liedtke 1995 §2.2).
    pub space: AddressSpaceId,
    /// Messages the kernel has delivered to this thread, with full
    /// mediation provenance.
    pub delivered: Vec<DeliveredMessage>,
}

/// A message as queued at an endpoint, between `Send` and `Receive` —
/// Brinch Hansen (1970): the nucleus buffers messages between sender
/// and receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedMessage {
    /// The thread that sent the message.
    pub sender: ThreadId,
    /// The transferred word.
    pub payload: PayloadWord,
}

/// A message as delivered to a receiver, carrying its kernel-mediation
/// provenance: *which endpoint* the message passed through. Direct
/// thread-to-thread delivery is unrepresentable — there is no
/// constructor path that skips the endpoint (Brinch Hansen 1970;
/// Liedtke 1995 §2.2: IPC is the only communication primitive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveredMessage {
    /// The thread that sent the message.
    pub sender: ThreadId,
    /// The endpoint that mediated the transfer.
    pub via: EndpointId,
    /// The transferred word.
    pub payload: PayloadWord,
}

/// The message a `Send` names: the payload plus the address space the
/// send buffer lies in. The kernel checks the buffer space against the
/// sender's own space — the isolation guard of Liedtke (1995) §2.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelMessage {
    /// The address space holding the send buffer.
    pub buffer_space: AddressSpaceId,
    /// The transferred word.
    pub payload: PayloadWord,
}

/// One endpoint's FIFO message queue — Brinch Hansen (1970): messages
/// are queued in order of arrival ("first come, first served").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointState {
    /// Messages sent but not yet received, in arrival order.
    pub queue: Vec<QueuedMessage>,
}

// ---------------------------------------------------------------------------
// Situation + actions
// ---------------------------------------------------------------------------

/// The kernel state: threads with their address-space bindings,
/// endpoint message queues, and the current (running) thread — the
/// engine `Situation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelSituation {
    /// Thread control blocks, indexed by `ThreadId`.
    pub threads: Vec<ThreadState>,
    /// Endpoint queues, indexed by `EndpointId`.
    pub endpoints: Vec<EndpointState>,
    /// The thread currently chosen to run (Liedtke 1996: scheduling is
    /// a privileged mechanism).
    pub current: ThreadId,
}

impl Situation for KernelSituation {}

impl KernelSituation {
    /// Every delivered message names an endpoint that exists — the
    /// kernel-mediation invariant made checkable: no delivery without a
    /// mediating endpoint (Brinch Hansen 1970).
    pub fn every_delivery_is_endpoint_mediated(&self) -> bool {
        self.threads
            .iter()
            .all(|t| t.delivered.iter().all(|d| d.via.0 < self.endpoints.len()))
    }

    /// How many delivered messages crossed an address-space boundary
    /// (sender's space differs from receiver's) — used for non-vacuity:
    /// the fixture round trip must actually exercise isolation.
    pub fn cross_space_delivery_count(&self) -> usize {
        self.threads
            .iter()
            .map(|receiver| {
                receiver
                    .delivered
                    .iter()
                    .filter(|d| {
                        self.threads
                            .get(d.sender.0)
                            .is_some_and(|sender| sender.space != receiver.space)
                    })
                    .count()
            })
            .sum()
    }
}

/// One kernel entry — the engine `Action`. The three system calls of
/// the minimal kernel: IPC send, IPC receive (Liedtke 1995 §2.2), and
/// thread switch (the scheduling mechanism, Liedtke 1996).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelAction {
    /// Send `msg` to `endpoint` on behalf of thread `from`.
    Send {
        /// The sending thread.
        from: ThreadId,
        /// The endpoint to enqueue at.
        endpoint: EndpointId,
        /// The message, with its buffer's address space.
        msg: KernelMessage,
    },
    /// Receive the next queued message from `endpoint` into `thread`'s
    /// mailbox.
    Receive {
        /// The receiving thread.
        thread: ThreadId,
        /// The endpoint to dequeue from.
        endpoint: EndpointId,
    },
    /// Switch the current thread to `to` — the scheduler acting.
    Switch {
        /// The thread to run next.
        to: ThreadId,
    },
}

impl Action for KernelAction {
    type Sit = KernelSituation;
}

// ---------------------------------------------------------------------------
// Fixture constants (named + cited)
// ---------------------------------------------------------------------------

/// Number of threads in the fixture: a client and a server. The
/// canonical microkernel interaction is one user-level client invoking
/// one user-level server by IPC (Liedtke 1995 §3; Haertig et al. 1997
/// SOSP); two threads is the smallest such configuration.
pub const FIXTURE_THREAD_COUNT: usize = 2;

/// Number of address spaces in the fixture: one per thread, so the
/// fixture IPC actually crosses an isolation boundary — the address
/// space is the unit of isolation (Liedtke 1995 §2.1) and same-space
/// messaging would leave the isolation guard unexercised.
pub const FIXTURE_ADDRESS_SPACE_COUNT: usize = FIXTURE_THREAD_COUNT;

/// Number of endpoints in the fixture: a single endpoint connects
/// client and server — the rendezvous object of IPC (Klein et al. 2009
/// SOSP).
pub const FIXTURE_ENDPOINT_COUNT: usize = 1;

/// The fixture payload is *derived* from the sender's identity rather
/// than being a free constant, so the end-to-end round trip through the
/// endpoint queue is checkable without introducing a magic number.
pub fn fixture_payload(sender: ThreadId) -> PayloadWord {
    PayloadWord(sender.0 as u64)
}

/// Initial fixture situation: thread `i` bound to address space `i`
/// (per-server address spaces — Liedtke 1995 §3), all endpoint queues
/// empty, thread 0 (the client) running.
pub fn kernel_initial() -> KernelSituation {
    KernelSituation {
        threads: (0..FIXTURE_THREAD_COUNT)
            .map(|i| ThreadState {
                // One space per thread; in-range because the space
                // count equals the thread count (see the constants).
                space: AddressSpaceId(i % FIXTURE_ADDRESS_SPACE_COUNT),
                delivered: Vec::new(),
            })
            .collect(),
        endpoints: (0..FIXTURE_ENDPOINT_COUNT)
            .map(|_| EndpointState { queue: Vec::new() })
            .collect(),
        current: ThreadId(0),
    }
}

// ---------------------------------------------------------------------------
// Transition function
// ---------------------------------------------------------------------------

/// Apply one kernel entry. `Err` when the entry violates a kernel
/// guarantee or names a nonexistent object:
///
/// - `Send` from a thread that is not running is rejected (only the
///   running thread can trap into the kernel);
/// - `Send` whose buffer lies outside the sender's own address space is
///   rejected — the isolation guard (Liedtke 1995 §2.1);
/// - `Receive` on an empty queue is rejected — nothing can be delivered
///   that did not first pass through the endpoint (Brinch Hansen 1970:
///   `wait message` delays the receiver until a message is queued);
/// - out-of-range thread/endpoint ids are rejected.
pub fn apply(
    situation: &KernelSituation,
    action: &KernelAction,
) -> Result<KernelSituation, String> {
    let mut next = situation.clone();
    match action {
        KernelAction::Send {
            from,
            endpoint,
            msg,
        } => {
            let Some(sender) = situation.threads.get(from.0) else {
                return Err(format!("no thread with id {}", from.0));
            };
            if *from != situation.current {
                return Err(format!(
                    "thread {} cannot send: it is not the running thread",
                    from.0
                ));
            }
            if msg.buffer_space != sender.space {
                return Err(format!(
                    "isolation violation: thread {} in space {} named a buffer in space {}",
                    from.0, sender.space.0, msg.buffer_space.0
                ));
            }
            let Some(queue) = next.endpoints.get_mut(endpoint.0) else {
                return Err(format!("no endpoint with id {}", endpoint.0));
            };
            queue.queue.push(QueuedMessage {
                sender: *from,
                payload: msg.payload,
            });
        }
        KernelAction::Receive { thread, endpoint } => {
            if situation.threads.get(thread.0).is_none() {
                return Err(format!("no thread with id {}", thread.0));
            }
            if *thread != situation.current {
                return Err(format!(
                    "thread {} cannot receive: it is not the running thread",
                    thread.0
                ));
            }
            let Some(queue) = next.endpoints.get_mut(endpoint.0) else {
                return Err(format!("no endpoint with id {}", endpoint.0));
            };
            if queue.queue.is_empty() {
                return Err(format!(
                    "receive blocks: endpoint {} has no queued message",
                    endpoint.0
                ));
            }
            // FIFO — Brinch Hansen (1970): first come, first served.
            let queued = queue.queue.remove(0);
            next.threads[thread.0].delivered.push(DeliveredMessage {
                sender: queued.sender,
                via: *endpoint,
                payload: queued.payload,
            });
        }
        KernelAction::Switch { to } => {
            if situation.threads.get(to.0).is_none() {
                return Err(format!("no thread with id {}", to.0));
            }
            next.current = *to;
        }
    }
    Ok(next)
}

/// The kernel entries enabled in a situation, filtered by the guards of
/// [`apply`]: a switch to each thread, plus — for the running thread —
/// a send of its own-space fixture payload to each endpoint and a
/// receive from each non-empty endpoint.
pub fn enabled_actions(situation: &KernelSituation) -> Vec<KernelAction> {
    let mut actions: Vec<KernelAction> = Vec::new();
    for i in 0..situation.threads.len() {
        actions.push(KernelAction::Switch { to: ThreadId(i) });
    }
    let current = situation.current;
    if let Some(running) = situation.threads.get(current.0) {
        for e in 0..situation.endpoints.len() {
            actions.push(KernelAction::Send {
                from: current,
                endpoint: EndpointId(e),
                msg: KernelMessage {
                    buffer_space: running.space,
                    payload: fixture_payload(current),
                },
            });
            if !situation.endpoints[e].queue.is_empty() {
                actions.push(KernelAction::Receive {
                    thread: current,
                    endpoint: EndpointId(e),
                });
            }
        }
    }
    actions
        .into_iter()
        .filter(|a| apply(situation, a).is_ok())
        .collect()
}
