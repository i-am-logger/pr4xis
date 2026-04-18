#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::authority::Authority;
use super::element::Answer;
use super::fact::{Fact, Severity};
use super::finding::Finding;

/// A legal argument: reasoning connecting findings to a conclusion.
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub ground: String,
    pub title: String,
    pub findings: Vec<Finding>,
    pub standards: Vec<Authority>,
    pub explanation: String,
    pub reasoning: String,
    pub counterarguments: Vec<Counterargument>,
    pub checklist: Vec<CheckItem>,
    pub severity: Severity,
}

/// An anticipated counterargument with rebuttal.
#[derive(Debug, Clone, PartialEq)]
pub struct Counterargument {
    pub anticipated: String,
    pub rebuttal: String,
    pub strength: CounterStrength,
}

/// How strong a counterargument is.
#[derive(Debug, Clone, PartialEq)]
pub enum CounterStrength {
    Weak { reason: String },
    Moderate { reason: String },
    Strong { reason: String },
}

/// A checklist item: question + answer + evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckItem {
    pub question: String,
    pub answer: Answer,
    pub evidence: Vec<Fact>,
}
