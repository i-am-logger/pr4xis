//! Concurrency — the abstract process-composition theory grounding the
//! operating-system family.
//!
//! Dijkstra (1968) critical sections and semaphores; Hoare (1974)
//! monitors and (1978) communicating sequential processes; Milner
//! (1980) parallel composition and interleaving; Lamport (1977)
//! safety/liveness and (1978) happens-before with logical clocks;
//! Coffman, Elphick & Shoshani (1971) deadlock conditions.
//!
//! - [`ontology`] — the `Concurrency` ontology and its five domain
//!   axioms, each discharged against an engine fixture.
//! - [`engine`] — bounded exhaustive interleaving explorer, Lamport
//!   clock fixture, and resource-allocation graph.
//! - [`system_functor`] — the verified `Concurrency → System` functor
//!   (every concurrent program is a system, von Bertalanffy 1968).

pub mod engine;
pub mod ontology;
pub mod parallelism_functor;
pub mod system_functor;

#[cfg(test)]
mod tests;
