#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::entity::Concept;
use super::fact::{Fact, Severity};
use super::source::Source;
use chrono::{DateTime, NaiveDate, Utc};

/// A finding: what facts mean together.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub title: String,
    pub facts: Vec<Fact>,
    pub analysis: String,
    pub severity: Severity,
    pub subject: Option<Concept>,
    pub analyzed_at: Option<DateTime<Utc>>,
}

/// A contradiction between claimed and actual facts.
#[derive(Debug, Clone, PartialEq)]
pub struct Contradiction {
    pub claimed: Fact,
    pub actual: Fact,
    pub claimed_by: Concept,
    pub refuted_by: Source,
    pub significance: String,
}

/// A ruling on a motion.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Ruling {
    Merits(MeritsRuling),
    Procedural(ProceduralRuling),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeritsRuling {
    pub date: NaiveDate,
    pub court: Concept,
    pub judge: Option<Concept>,
    pub motion: String,
    pub outcome: MeritsOutcome,
    pub significance: String,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MeritsOutcome {
    Denied { detail: Option<String> },
    Granted { detail: Option<String> },
    GrantedInPart { granted: String, denied: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProceduralRuling {
    pub date: NaiveDate,
    pub court: Concept,
    pub judge: Option<Concept>,
    pub motion: String,
    pub outcome: ProceduralOutcome,
    pub significance: String,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ProceduralOutcome {
    DismissedWithPrejudice { reason: String },
    DismissedWithoutPrejudice { reason: String },
    Transferred { to: Concept },
    Stayed { duration: Option<String> },
    Granted { detail: String },
    Denied { detail: String },
}
