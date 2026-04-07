use crate::action::Action;
use crate::precondition::{Precondition, PreconditionResult};
use crate::situation::Situation;
use crate::trace::{Trace, TraceEntry};

/// The enforcement engine — applies actions to situations with precondition checking.
///
/// Implements the `.next()` pattern with back/forward history:
/// ```ignore
/// let engine = Engine::new(initial_situation, preconditions, apply_fn);
/// let engine = engine.next(action1)?;   // validates + applies
/// let engine = engine.next(action2)?;   // validates + applies
/// let engine = engine.back()?;          // undo
/// let engine = engine.forward()?;       // redo
/// engine.trace().dump()                 // full history
/// ```
#[allow(clippy::type_complexity)]
pub struct Engine<A: Action> {
    situation: A::Sit,
    past: Vec<A::Sit>,
    future: Vec<A::Sit>,
    preconditions: Vec<Box<dyn Precondition<A>>>,
    apply_fn: Box<dyn Fn(&A::Sit, &A) -> A::Sit>,
    trace: Trace,
}

impl<A: Action> std::fmt::Debug for Engine<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("situation", &self.situation)
            .field("step", &self.step())
            .field("back_depth", &self.back_depth())
            .field("forward_depth", &self.forward_depth())
            .field("trace_entries", &self.trace.entries.len())
            .finish()
    }
}

impl<A: Action> Engine<A> {
    /// Create a new engine with an initial situation, preconditions, and apply function.
    pub fn new(
        situation: A::Sit,
        preconditions: Vec<Box<dyn Precondition<A>>>,
        apply_fn: impl Fn(&A::Sit, &A) -> A::Sit + 'static,
    ) -> Self {
        Self {
            situation,
            past: Vec::new(),
            future: Vec::new(),
            preconditions,
            apply_fn: Box::new(apply_fn),
            trace: Trace::new(),
        }
    }

    /// Current step number (derived from history depth).
    pub fn step(&self) -> usize {
        self.past.len()
    }

    /// The current situation.
    pub fn situation(&self) -> &A::Sit {
        &self.situation
    }

    /// The full trace of all actions.
    pub fn trace(&self) -> &Trace {
        &self.trace
    }

    /// Is the current situation terminal?
    pub fn is_terminal(&self) -> bool {
        self.situation.is_terminal()
    }

    /// Apply an action — the `.next()` method.
    ///
    /// Checks all preconditions. If any fail, returns Err with the violations.
    /// If all pass, applies the action and records the trace.
    #[allow(clippy::result_large_err)]
    pub fn next(mut self, action: A) -> Result<Self, (Self, Vec<PreconditionResult>)> {
        let situation_before = self.situation.describe();
        let action_desc = action.describe();
        let step = self.step();

        // Check all preconditions
        let results: Vec<PreconditionResult> = self
            .preconditions
            .iter()
            .map(|p| p.check(&self.situation, &action))
            .collect();

        let violations: Vec<PreconditionResult> = results
            .iter()
            .filter(|r| !r.is_satisfied())
            .cloned()
            .collect();

        if !violations.is_empty() {
            self.trace.record(TraceEntry {
                step,
                situation_before,
                action: action_desc,
                precondition_results: results,
                situation_after: None,
                success: false,
            });
            return Err((self, violations));
        }

        // Apply the action — save current for undo, clear redo stack
        let new_situation = (self.apply_fn)(&self.situation, &action);
        let situation_after = new_situation.describe();

        self.trace.record(TraceEntry {
            step,
            situation_before,
            action: action_desc,
            precondition_results: results,
            situation_after: Some(situation_after),
            success: true,
        });

        self.past.push(self.situation.clone());
        self.future.clear();
        self.situation = new_situation;
        Ok(self)
    }

    /// Go back one step. The current situation moves to the redo stack.
    pub fn back(mut self) -> Result<Self, Self> {
        match self.past.pop() {
            Some(previous) => {
                self.future.push(self.situation.clone());
                self.situation = previous;
                Ok(self)
            }
            None => Err(self),
        }
    }

    /// Go forward one step (redo). Only available after back().
    pub fn forward(mut self) -> Result<Self, Self> {
        match self.future.pop() {
            Some(next) => {
                self.past.push(self.situation.clone());
                self.situation = next;
                Ok(self)
            }
            None => Err(self),
        }
    }

    /// How many steps back are available.
    pub fn back_depth(&self) -> usize {
        self.past.len()
    }

    /// How many steps forward are available (after back).
    pub fn forward_depth(&self) -> usize {
        self.future.len()
    }

    /// Try to apply an action, returning the new engine or the violations as strings.
    pub fn try_next(self, action: A) -> Result<Self, Vec<String>> {
        self.next(action).map_err(|(_, violations)| {
            violations
                .into_iter()
                .map(|v| match v {
                    PreconditionResult::Violated { rule, reason, .. } => {
                        format!("{}: {}", rule, reason)
                    }
                    PreconditionResult::Satisfied { .. } => unreachable!(),
                })
                .collect()
        })
    }
}
