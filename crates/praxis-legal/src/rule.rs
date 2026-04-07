use crate::authority::Authority;
use crate::fact::{Fact, Severity};

/// A legal rule: conditions → consequence.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub authority: Vec<Authority>,
    pub conditions: Vec<Condition>,
    pub consequence: Consequence,
}

/// Conditions for a rule — compositional (AllOf, AnyOf, Not).
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    FactExists(FactMatcher),
    TemporalProximityWithin {
        from: FactMatcher,
        to: FactMatcher,
        max_days: i64,
    },
    SeverityAtLeast(Severity),
    AllOf(Vec<Condition>),
    AnyOf(Vec<Condition>),
    Not(Box<Condition>),
    TermSatisfied(String), // ontology term ID
}

/// Fact matching criteria.
#[derive(Debug, Clone, PartialEq)]
pub enum FactMatcher {
    ClaimContains(String),
    MinSeverity(Severity),
    DateRange {
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
    },
    InvolvesEntity(String),
    Any,
}

/// What happens when a rule fires.
#[derive(Debug, Clone, PartialEq)]
pub struct Consequence {
    pub finding_type: String,
    pub severity: Severity,
    pub recommendation: Recommendation,
    pub explanation: String,
}

/// Possible recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    Investigate,
    Disclose,
    Monitor,
    Compel,
    Sanction,
    NoAction,
}

/// Result of evaluating a rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleEvaluation {
    pub rule_name: String,
    pub condition_results: Vec<ConditionResult>,
    pub triggered: Triggered,
}

/// Whether a rule was triggered.
#[derive(Debug, Clone, PartialEq)]
pub enum Triggered {
    Yes {
        recommendation: Recommendation,
    },
    No {
        unmet: Vec<String>,
    },
    Partial {
        met: Vec<String>,
        unmet: Vec<String>,
    },
}

/// Result of checking a single condition.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionResult {
    pub description: String,
    pub status: ConditionStatus,
    pub evidence: Vec<Fact>,
}

/// Status of a condition.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionStatus {
    Met,
    NotMet {
        reason: String,
    },
    Partial {
        what_met: String,
        what_remains: String,
    },
    Unknown {
        needs: String,
    },
}
