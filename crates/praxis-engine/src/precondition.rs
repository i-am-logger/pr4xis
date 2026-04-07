use crate::action::Action;

/// Result of checking a precondition — both states carry context.
#[derive(Debug, Clone, PartialEq)]
pub enum PreconditionResult {
    /// Precondition is satisfied — carries which rule passed and why.
    Satisfied { rule: String, reason: String },
    /// Precondition violated — carries full diagnostic.
    Violated {
        rule: String,
        reason: String,
        situation: String,
        attempted_action: String,
    },
}

impl PreconditionResult {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, PreconditionResult::Satisfied { .. })
    }

    pub fn satisfied(rule: &str, reason: &str) -> Self {
        PreconditionResult::Satisfied {
            rule: rule.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn violated(rule: &str, reason: &str, situation: &str, action: &str) -> Self {
        PreconditionResult::Violated {
            rule: rule.to_string(),
            reason: reason.to_string(),
            situation: situation.to_string(),
            attempted_action: action.to_string(),
        }
    }

    /// The rule name, regardless of outcome.
    pub fn rule(&self) -> &str {
        match self {
            PreconditionResult::Satisfied { rule, .. } => rule,
            PreconditionResult::Violated { rule, .. } => rule,
        }
    }

    /// The reason, regardless of outcome.
    pub fn reason(&self) -> &str {
        match self {
            PreconditionResult::Satisfied { reason, .. } => reason,
            PreconditionResult::Violated { reason, .. } => reason,
        }
    }
}

/// A precondition that must hold before an action can be applied.
///
/// This is where enforcement happens — the ontology's rules are
/// checked against the current situation and the proposed action.
pub trait Precondition<A: Action> {
    /// Check if this action is valid in the given situation.
    fn check(&self, situation: &A::Sit, action: &A) -> PreconditionResult;

    /// Human-readable description of what this precondition enforces.
    fn describe(&self) -> &str;
}
