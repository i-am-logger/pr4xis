use crate::entity::Entity;
use crate::fact::{Fact, Severity};
use crate::source::Source;
use chrono::{DateTime, NaiveDate, Utc};

/// A finding: what facts mean together.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub title: String,
    pub facts: Vec<Fact>,
    pub analysis: String,
    pub severity: Severity,
    pub subject: Option<Entity>,
    pub analyzed_at: Option<DateTime<Utc>>,
}

/// A contradiction between claimed and actual facts.
#[derive(Debug, Clone, PartialEq)]
pub struct Contradiction {
    pub claimed: Fact,
    pub actual: Fact,
    pub claimed_by: Entity,
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
    pub court: Entity,
    pub judge: Option<Entity>,
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
    pub court: Entity,
    pub judge: Option<Entity>,
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
    Transferred { to: Entity },
    Stayed { duration: Option<String> },
    Granted { detail: String },
    Denied { detail: String },
}
