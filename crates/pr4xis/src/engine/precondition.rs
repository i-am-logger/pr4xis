use super::action::Action;
use crate::logic::proof::Verdict;

/// A precondition that must hold before an action can be applied.
///
/// This is where enforcement happens — the ontology's rules are
/// checked against the current situation and the proposed action.
///
/// # Returns a typed Verdict (#161)
///
/// `check` returns [`Verdict`] — `Ok(Box<dyn Proof>)` witnessing the
/// precondition holds, or `Err(Box<dyn Counterexample>)` refuting it.
/// The `Proof` / `Counterexample` carry their own `meta()` (name,
/// description, citation, module path) — no separate `rule` or `reason`
/// string fields are needed.
///
/// Preconditions are thus themselves ontological axioms at the engine
/// boundary: each check is a verification, each result is a typed
/// witness.
pub trait Precondition<A: Action> {
    /// Check if this action is valid in the given situation.
    fn check(&self, situation: &A::Sit, action: &A) -> Verdict;
}
