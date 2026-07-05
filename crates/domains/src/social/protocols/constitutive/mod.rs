//! ConstitutiveProtocol — the prx p2p trust protocol's conceptual layer.
//!
//! Re-manifests the prose ontology of the prx protocol as pr4xis code: six
//! top-level categories (Identity, Authority, Constitution, Praxis,
//! Membership, Slashing) plus the transport-address and signature-scheme
//! concepts its two foundational axioms (`Ipv6Only`, `PostQuantumOnly`)
//! quantify over. The illocutionary force of the nine praxis event types is
//! valued in the EXISTING `SearleCategory` from
//! `cognitive::linguistics::pragmatics::speech_act` — the protocol maps onto
//! the speech-act taxonomy (Searle 1969; Austin 1962) rather than redefining
//! it.
//!
//! - `ontology.rs` — the `ontology!` block, five typed Qualities, and seven
//!   domain axioms.
//! - `engine.rs` — the receiver-side admittance state machine
//!   (`ChannelSituation` / `ChannelAction` / `apply_channel`): gate 0
//!   (slashed devices dropped) and the least-privilege role-grant gate.
//!
//! No cross-functor yet: the consensus → constitutive functor is planned for
//! a later wave, once the consensus ontology it maps from exists.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
