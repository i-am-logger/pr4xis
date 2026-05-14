pub mod ontology;
pub mod systems_functor;

// `chess_functor.rs` is disabled — it was hand-rolled on the pre-#152
// `Relationship` trait (Chen 1976 ER vocabulary), which has been replaced
// by `Arrow` (Mac Lane / Awodey). Rewriting the hand-rolled
// `ChessConcurrentCategory` onto the new substrate is out of scope for the
// formal-ontology migration; the chess→concurrency functor will be
// re-introduced as part of #155/#157 once chess itself is fully on the
// proc-macro `ontology!` shape.
// pub mod chess_functor;

pub use ontology::*;

#[cfg(test)]
mod tests;
