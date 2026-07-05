//! Operating systems — the applied family covering kernel structure,
//! scheduling, inter-process communication, and device management,
//! grounded in the `formal::systems` foundations (concurrency,
//! parallelism, general systems, MAPE-K).
//! DOLCE: Process (applied engineering).
//!
//! - [`bus`] — the event/message bus: publish/subscribe (Eugster et al.
//!   2003), content-based routing (Carzaniga et al. 2001), actors and
//!   mailboxes (Hewitt et al. 1973), and delivery guarantees (Birman &
//!   Joseph 1987).
//! - [`driver`] — device drivers: the software/hardware bridge, fault
//!   isolation (Swift et al. 2003 Nooks), and driver synthesis.
//! - [`microkernel`] — kernel construction: what runs privileged, what
//!   runs in user space, and why (Brinch Hansen 1970, Levin et al.
//!   1975, Liedtke 1995, Klein et al. 2009).
//! - [`scheduler`] — processor scheduling: rate-monotonic and EDF
//!   theory (Liu & Layland 1973), priority inversion and inheritance
//!   (Sha et al. 1990).

pub mod bus;
pub mod driver;
pub mod microkernel;
pub mod scheduler;
