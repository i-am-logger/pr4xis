//! Swarm — networked multi-agent estimation over peer-to-peer graphs.
//!
//! Two ontologies grounded in the distributed-estimation literature (the
//! mathematics is established — Olfati-Saber, Fax & Murray 2007; Xiao &
//! Boyd 2004; Kempe, Dobra & Gehrke 2003; Fiedler 1973; Julier & Uhlmann
//! 1997; Bar-Shalom 1981; Mutambara 1998 — this family claims no novelty
//! for it):
//!
//! - [`consensus`] — the `Consensus` ontology: peers, neighbourhoods,
//!   gossip, average consensus, the disagreement Lyapunov function, the
//!   Fiedler spectral gap, and the equivocation-detection trust wiring
//!   into the constitutive protocol.
//! - [`fusion`] — the `DistributedFusion` ontology: distributed Kalman
//!   filtering by consensus on information contributions, covariance
//!   intersection over a network, and the data-incest failure mode of
//!   naive additive re-fusion on cyclic topologies.
//! - [`smart_element`] — the `SmartElement` ontology: the synthesis of the
//!   two above with the autonomic loop — an autonomic edge element that is
//!   simultaneously a MAPE-K manager (Kephart & Chess 2003) and a
//!   signed-estimate fusion peer, carrying a queryable local ontology.

pub mod consensus;
pub mod fusion;
pub mod smart_element;
