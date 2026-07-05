//! Parallelism — the theory of executing computation simultaneously to
//! reduce completion time, and the coarser sibling of `concurrency`.
//!
//! Flynn's machine taxonomy (Flynn 1966, 1972); Amdahl's serial-fraction
//! bound (Amdahl 1967); Gustafson's scaled speedup (Gustafson 1988);
//! work/span and greedy scheduling (Brent 1974; Graham 1966, 1969;
//! Blelloch 1996); cost models (Valiant 1990; Fortune & Wyllie 1978;
//! Culler et al. 1993); determinism by default (Bocchino et al. 2009).
//!
//! - [`ontology`] — the `Parallelism` ontology and its eight domain
//!   axioms, discharged against numeric grids and the CLRS work-span
//!   fixture.
//! - [`engine`] — the `P-FIB(4)` computation DAG (work 17, span 8) and a
//!   greedy scheduler.
//! - [`concurrency_functor`] — the `Parallelism → Concurrency` functor
//!   and the `Concurrency ⊣ Parallelism` adjunction whose theorem is the
//!   interleaving-collapse gap.
//! - [`system_functor`] — the `Parallelism → System` functor
//!   (`ProcessingElement`'s faithful home).

pub mod concurrency_functor;
pub mod engine;
pub mod ontology;
pub mod system_functor;

#[cfg(test)]
mod tests;
