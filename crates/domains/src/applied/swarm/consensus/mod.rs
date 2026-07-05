//! Consensus — distributed agreement over a peer graph, with trust
//! wiring for protocol-detectable misbehaviour.
//!
//! Average consensus and its disagreement Lyapunov function
//! (Olfati-Saber, Fax & Murray 2007; Olfati-Saber & Murray 2004),
//! push-sum gossip and its mass conservation (Kempe, Dobra & Gehrke
//! 2003; Xiao & Boyd 2004), the algebraic-connectivity view of
//! convergence (Fiedler 1973), and equivocation detection with
//! exclusion-before-aggregation (Lamport, Shostak & Pease 1982; Li,
//! Krohn, Mazieres & Shasha 2004 SUNDR). Established mathematics; this
//! module encodes it and claims no novelty for it.
//!
//! - [`ontology`] — the `Consensus` ontology and its five domain
//!   axioms, each discharged against an engine fixture.
//! - [`engine`] — average-consensus / push-sum simulator over `P3` and
//!   a disconnected `2+1` graph, with equivocation flagging.
//! - [`mape_k_functor`] — the peer's local loop as `Consensus → MapeK`.
//! - [`dependability_functor`] — the Avizienis reading:
//!   `Equivocation → ByzantineFault`, exclusion as fault handling.
//! - [`constitutive_functor`] — the trust bridge into the constitutive
//!   protocol: `PeerIdentity → Identity`, `Equivocation → Equivocation`,
//!   `DistrustedPeer → Slashing`.

pub mod constitutive_functor;
pub mod dependability_functor;
pub mod engine;
pub mod mape_k_functor;
pub mod ontology;

#[cfg(test)]
mod tests;
