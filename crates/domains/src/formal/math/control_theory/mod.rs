//! The control theory ontology — feedback systems, stability (Lyapunov), PID, transfer functions
pub mod adwin;
pub mod feedback;
pub mod ontology;
pub mod pid;
pub mod stability;
pub mod systems_functor;
pub mod transfer_function;

#[cfg(test)]
mod tests;
