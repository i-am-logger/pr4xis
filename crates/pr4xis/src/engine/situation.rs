#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::fmt::Debug;

/// A situation — a snapshot of the world at a point in time.
///
/// # Literature
///
/// Situation calculus:
/// - McCarthy (1963) "Situations, Actions, and Causal Laws" (Stanford AI Memo 2)
/// - McCarthy & Hayes (1969) "Some Philosophical Problems from the Standpoint
///   of Artificial Intelligence", in *Machine Intelligence* 4
///
/// A situation `s` in the calculus is a complete state of the world at a
/// single moment. Actions are functions `do(a, s)` producing the
/// successor situation. Praxis engine models the same pattern: every
/// action takes a Situation and produces a new Situation.
///
/// # No `is_terminal() -> bool` (#161)
///
/// Previous versions of this trait had `is_terminal(&self) -> bool` — a
/// primitive-leak. Situation calculus does not define "terminal" as a
/// state flag; in planning (Fikes & Nilsson 1971 STRIPS, McCarthy &
/// Hayes 1969), a situation is terminal iff **no action is `Poss`ible**
/// in it. That's a logical derivation from the declared preconditions,
/// not a method on the situation itself.
///
/// If an engine needs a terminal check, it's configured via a typed
/// precondition/axiom at Engine construction (see
/// [`Precondition`](super::Precondition)), returning a [`Verdict`]. No
/// booleans in the public API (see `feedback_core_no_bool_api`).
///
/// # No `describe() -> String`
///
/// Display is `Debug` (required as a supertrait). Domain-facing labels
/// live in each domain's own Situation ontology (via Lemon), not on the
/// runtime trait.
pub trait Situation: Clone + Debug + PartialEq {}
