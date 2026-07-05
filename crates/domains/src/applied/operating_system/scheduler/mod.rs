//! Scheduler — processor scheduling for the operating-system family.
//!
//! Liu & Layland (1973) periodic tasks, rate-monotonic and
//! deadline-driven scheduling with their utilization bounds; Sha,
//! Rajkumar & Lehoczky (1990) priority inversion and inheritance;
//! Corbató, Merwin-Daggett & Daley (1962) time-sliced and multilevel
//! feedback dispatch (CTSS); Leung & Whitehead (1982)
//! deadline-monotonic assignment.
//!
//! - [`ontology`] — the `Scheduler` ontology, three typed qualities,
//!   and its four domain axioms, discharged against the engine.
//! - [`engine`] — a ready-queue simulator over an integer slot grid
//!   (typed `Quantity` task parameters), plus the Sha et al. three-job
//!   priority-inversion fixture with/without inheritance.
//! - [`mape_k_functor`] — the `Scheduler → MapeK` functor (the
//!   scheduler as an autonomic control loop).
//! - [`system_functor`] — the `Scheduler → System` functor (the
//!   scheduler as Ashby's regulator).
//! - [`parallelism_functor`] — the forgetful `Scheduler → Parallelism`
//!   functor (Graham list scheduling IS priority scheduling).

pub mod engine;
pub mod mape_k_functor;
pub mod ontology;
pub mod parallelism_functor;
pub mod system_functor;

#[cfg(test)]
mod tests;
