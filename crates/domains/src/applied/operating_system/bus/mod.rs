//! Bus — the event/message bus: a medium decoupling senders from
//! receivers, covering both the kernel IPC bus and the publish/subscribe
//! event bus.
//!
//! Publish/subscribe and its three decoupling dimensions (Eugster,
//! Felber, Guerraoui & Kermarrec 2003); content-based routing and
//! subscriptions as predicates (Carzaniga, Rosenblum & Wolf 2001,
//! SIENA); actors, messages, and mailboxes (Hewitt, Bishop & Steiger
//! 1973); delivery guarantees and virtual synchrony (Birman & Joseph
//! 1987, ISIS).
//!
//! - [`ontology`] — the `Bus` ontology and its five domain axioms, the
//!   behavioural one discharged against the broker simulator.
//! - [`engine`] — the broker simulator: routing table, per-subscriber
//!   queues, dedup sets, and the loss/retransmission fixture that
//!   separates the three delivery semantics.
//! - [`system_functor`] — the `Bus → System` functor (the broker is the
//!   controller, the bus a boundary, guarantees are constraints).

pub mod engine;
pub mod ontology;
pub mod system_functor;

#[cfg(test)]
mod tests;
