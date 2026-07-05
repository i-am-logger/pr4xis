//! Microkernel — the minimality principle of kernel construction.
//!
//! Liedtke (1995) SOSP "On µ-Kernel Construction": a kernel provides
//! ONLY address spaces, threads, and IPC; everything else — drivers,
//! file systems, pagers, policy — runs as user-space servers (Liedtke
//! 1996 CACM 39(9); Haertig et al. 1997 SOSP). The lineage runs from
//! Dijkstra (1968) CACM 11(5) layered privilege through Brinch Hansen
//! (1970) CACM 13(4) nucleus and Levin et al. (1975) SOSP
//! policy/mechanism separation to Klein et al. (2009) SOSP seL4, where
//! the minimised trusted computing base is formally verified.
//!
//! - [`ontology`] — the `Microkernel` ontology, two qualities, and four
//!   domain axioms (minimal privileged set, policy/mechanism
//!   separation, unprivileged servers, TCB-minimising contrast).
//! - [`engine`] — a minimal kernel-state fixture: threads bound to
//!   address spaces, endpoint message queues, kernel-mediated IPC.
//! - [`system_functor`] — the `Microkernel → System` functor (every
//!   kernel is a system; von Bertalanffy 1968).
//! - [`concurrency_functor`] — the `Microkernel → Concurrency` functor
//!   (threads + IPC ARE communicating sequential processes; Hoare 1978).
//! - [`bus_functor`] — the `Microkernel → Bus` functor (kernel IPC IS a
//!   message bus; Liedtke 1995; Eugster et al. 2003).

pub mod bus_functor;
pub mod concurrency_functor;
pub mod engine;
pub mod ontology;
pub mod system_functor;

#[cfg(test)]
mod tests;
