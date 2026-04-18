use super::situation::Situation;
#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

/// An action transforms one situation into another.
///
/// Actions are the morphisms in the praxis category.
/// They carry full context of WHAT is being attempted.
pub trait Action: Clone + Debug {
    /// The situation type this action operates on.
    type Sit: Situation;

    /// Human-readable description of this action.
    fn describe(&self) -> String;
}
