use crate::entity::Entity;
use crate::source::Source;
use chrono::NaiveDate;

/// Severity levels — ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Date precision for approximate dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePrecision {
    Exact,
    Month,
    Year,
    Approximate,
}

/// Typed fact values.
#[derive(Debug, Clone, PartialEq)]
pub enum FactValue {
    Currency {
        amount: f64,
        currency: String,
    },
    Percentage {
        value: f64,
        of_what: String,
    },
    Count {
        n: u64,
        of_what: String,
    },
    Duration {
        days: i64,
        from: NaiveDate,
        to: NaiveDate,
    },
    Rating {
        score: f64,
        scale: (f64, f64),
    },
    Text(String),
}

/// A narrative step — HOW we know something.
#[derive(Debug, Clone, PartialEq)]
pub struct NarrativeStep {
    pub actor: Entity,
    pub action: String,
    pub detail: Option<String>,
    pub quote: Option<String>,
    pub source: Source,
    pub date: Option<NaiveDate>,
}

/// A narrative chain showing provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct Narrative {
    pub chain: Vec<NarrativeStep>,
    pub significance: String,
}

/// An atom of evidence — a sourced observation.
#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub claim: String,
    pub value: Option<FactValue>,
    pub date: Option<NaiveDate>,
    pub date_precision: DatePrecision,
    pub source: Source,
    pub actors: Vec<Entity>,
    pub narrative: Option<Narrative>,
    pub severity: Option<Severity>,
}

/// Temporal proximity between two facts.
#[derive(Debug, Clone, PartialEq)]
pub struct TemporalProximity {
    pub from: Fact,
    pub to: Fact,
    pub days: i64,
    pub significance: String,
}

/// Document integrity issues.
#[derive(Debug, Clone, PartialEq)]
pub enum IntegrityIssue {
    Backdated {
        document_date: NaiveDate,
        actual_timestamp: NaiveDate,
        gap_days: i64,
    },
    CreatedAfterLitigation {
        document_date: NaiveDate,
        litigation_filed: NaiveDate,
    },
    Unsigned,
    Incomplete {
        missing: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentIntegrity {
    pub issues: Vec<IntegrityIssue>,
}
