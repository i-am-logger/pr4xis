use super::precondition::PreconditionResult;

/// A single entry in the trace log.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEntry {
    pub step: usize,
    pub situation_before: String,
    pub action: String,
    pub precondition_results: Vec<PreconditionResult>,
    pub situation_after: Option<String>,
    pub success: bool,
}

/// A trace of actions applied to situations — full history for debugging.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub entries: Vec<TraceEntry>,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, entry: TraceEntry) {
        self.entries.push(entry);
    }

    /// Number of successful steps.
    pub fn successful_steps(&self) -> usize {
        self.entries.iter().filter(|e| e.success).count()
    }

    /// Number of failed steps (violations).
    pub fn violations(&self) -> usize {
        self.entries.iter().filter(|e| !e.success).count()
    }

    /// All violation entries.
    pub fn violation_entries(&self) -> Vec<&TraceEntry> {
        self.entries.iter().filter(|e| !e.success).collect()
    }

    /// Last entry.
    pub fn last(&self) -> Option<&TraceEntry> {
        self.entries.last()
    }

    /// Human-readable trace dump.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            let status = if entry.success { "OK" } else { "VIOLATION" };
            out.push_str(&format!(
                "[{}] {} | {} → {}\n",
                status,
                entry.action,
                entry.situation_before,
                entry.situation_after.as_deref().unwrap_or("(blocked)"),
            ));
            for result in &entry.precondition_results {
                match result {
                    PreconditionResult::Satisfied { rule, reason } => {
                        out.push_str(&format!("  + {}: {}\n", rule, reason));
                    }
                    PreconditionResult::Violated { rule, reason, .. } => {
                        out.push_str(&format!("  x {}: {}\n", rule, reason));
                    }
                }
            }
        }
        out
    }
}
