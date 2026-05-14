pub mod concurrent_functor;
pub mod ontology;
pub mod systems_functor;

// `chess_functor.rs` is disabled — it was hand-rolled on the pre-#152
// `Relationship` trait, which has been replaced by `Arrow`. Rewriting
// the hand-rolled `ChessEventCategory` onto the proc-macro shape is
// out of scope for the formal-ontology migration.
// pub mod chess_functor;

pub use ontology::*;

#[cfg(test)]
mod tests;
