use super::situation::Situation;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

/// An action transforms one situation into another.
///
/// Actions are the morphisms in the praxis category.
/// They carry full context of WHAT is being attempted.
///
/// # No `describe() -> String` (#161)
///
/// Previously the trait included `describe(&self) -> String` — a
/// primitive-leak into the domain interface. Display / diagnostic
/// rendering is Rust's `Debug` trait, required as a supertrait here.
/// Domain-facing descriptive labels (if needed) live in the action's
/// own ontology, not on the runtime trait.
pub trait Action: Clone + Debug {
    /// The situation type this action operates on.
    type Sit: Situation;
}
