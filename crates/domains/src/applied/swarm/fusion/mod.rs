//! DistributedFusion — fusing state estimates across a network of
//! peers.
//!
//! Distributed Kalman filtering by consensus on information
//! contributions (Olfati-Saber 2005, 2007), covariance intersection
//! over a network (Julier & Uhlmann 1997), and the data-incest failure
//! of naive additive re-fusion on cyclic topologies (Bar-Shalom 1981;
//! Mutambara 1998). Established mathematics; this module encodes it and
//! claims no novelty for it.
//!
//! - [`ontology`] — the `DistributedFusion` ontology and its four
//!   domain axioms (one an honest negative: the demonstrated
//!   over-confidence of naive re-fusion under cycles).
//! - [`engine`] — fixtures wrapping the existing sensor-fusion
//!   `InformationEstimate` (its `fuse()` is every additive step) and
//!   reusing the sibling `swarm::consensus` engine for the
//!   consensus-on-information run.
//! - [`architecture_functor`] — onto the existing `FusionArchitecture`
//!   enum via a discrete-category wrapper.
//! - [`composition_functor`] — onto the existing `CompositionStrategy`
//!   enum via an indiscrete-category wrapper.
//! - [`consensus_functor`] — onto the sibling `Consensus` ontology:
//!   distributed fusion IS consensus on information contributions.

pub mod architecture_functor;
pub mod composition_functor;
pub mod consensus_functor;
pub mod engine;
pub mod ontology;

#[cfg(test)]
mod tests;
