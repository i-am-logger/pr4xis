//! MAPE-K — Kephart & Chess (2003) autonomic-computing control loop.
//!
//! The ontology formalises the four-phase Monitor / Analyze / Plan /
//! Execute cycle over a shared Knowledge base, and provides a verified
//! cross-functor from the existing `PipelineStep` enum in
//! `formal::information::diagnostics::trace_functors` — proving the pr4xis
//! chat pipeline structurally IS a MAPE-K loop.
//!
//! Companion research note:
//! `docs/research/pipeline-architecture-survey.md`.

pub mod ontology;
pub mod pipeline_step_functor;
#[cfg(test)]
mod proptests;
