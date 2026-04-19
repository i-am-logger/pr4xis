use crate::logic::proof::Verdict;

use super::action::Action;

/// A single entry in the trace log — typed over the Action (#161).
///
/// Fields carry the typed situation and action directly (previously all
/// `String`); preconditions' check results are typed `Verdict`s
/// (previously `PreconditionResult` with String fields). No primitive
/// leaks.
#[derive(Debug)]
pub struct TraceEntry<A: Action> {
    pub step: usize,
    pub situation_before: A::Sit,
    pub action: A,
    pub precondition_verdicts: Vec<Verdict>,
    pub situation_after: Option<A::Sit>,
}

impl<A: Action> TraceEntry<A> {
    /// Did every precondition pass?
    pub fn preconditions_all_hold(&self) -> bool {
        self.precondition_verdicts.iter().all(|v| v.is_ok())
    }

    /// Did the action actually apply? (Preconditions passed AND apply
    /// produced a new situation.)
    pub fn applied(&self) -> bool {
        self.situation_after.is_some() && self.preconditions_all_hold()
    }
}

/// A trace of actions applied to situations — full history for debugging.
///
/// Typed over the Action (#161) — carries typed situations and actions,
/// not Strings.
#[derive(Debug)]
pub struct Trace<A: Action> {
    entries: Vec<TraceEntry<A>>,
}

impl<A: Action> Default for Trace<A> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<A: Action> Trace<A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// All trace entries as a slice.
    pub fn entries(&self) -> &[TraceEntry<A>] {
        &self.entries
    }

    pub fn record(&mut self, entry: TraceEntry<A>) {
        self.entries.push(entry);
    }

    /// Number of successful steps.
    pub fn successful_steps(&self) -> usize {
        self.entries.iter().filter(|e| e.applied()).count()
    }

    /// Number of failed steps (violations).
    pub fn violations(&self) -> usize {
        self.entries.iter().filter(|e| !e.applied()).count()
    }

    /// All violation entries.
    pub fn violation_entries(&self) -> Vec<&TraceEntry<A>> {
        self.entries.iter().filter(|e| !e.applied()).collect()
    }

    /// Last entry.
    pub fn last(&self) -> Option<&TraceEntry<A>> {
        self.entries.last()
    }
}
