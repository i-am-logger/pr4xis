#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::entity::Concept;
use chrono::NaiveDate;

/// A statute or regulation.
#[derive(Debug, Clone, PartialEq)]
pub struct Statute {
    pub citation: Citation,
    pub title: Option<String>,
    pub text: Option<String>,
    pub effective_date: Option<NaiveDate>,
}

/// A case law precedent.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseLaw {
    pub name: String,
    pub citation: Citation,
    pub year: u16,
    pub court: String,
    pub holding: String,
    pub quote: Option<String>,
}

/// Precedent tracking — how a doctrine evolves.
#[derive(Debug, Clone, PartialEq)]
pub struct Precedent {
    pub doctrine: String,
    pub established_by: CaseLaw,
    pub followed_by: Vec<CaseLaw>,
    pub distinguished_by: Vec<CaseLaw>,
    pub overruled: Option<CaseLaw>,
}

/// Legal authority hierarchy — what the law IS.
#[derive(Debug, Clone, PartialEq)]
pub enum Authority {
    Constitution {
        provision: String,
    },
    Legislature {
        statute: Statute,
    },
    SupremeCourt {
        case: CaseLaw,
        interprets: Box<Authority>,
    },
    AppellateCourt {
        jurisdiction: String,
        case: CaseLaw,
        interprets: Box<Authority>,
        precedent: Option<Precedent>,
    },
    TrialCourt {
        court: Concept,
        case: CaseLaw,
    },
    Regulation {
        agency: Concept,
        regulation: Statute,
        implements: Box<Authority>,
    },
    AgencyAction {
        agency: Concept,
        action_type: String,
        under: Box<Authority>,
    },
    ProfessionalBody {
        body: Concept,
        rule: Statute,
    },
}

impl Authority {
    /// Weight of authority (constitutional = highest).
    pub fn weight(&self) -> u8 {
        match self {
            Authority::Constitution { .. } => 10,
            Authority::SupremeCourt { .. } => 9,
            Authority::Legislature { .. } => 8,
            Authority::AppellateCourt { .. } => 7,
            Authority::Regulation { .. } => 6,
            Authority::AgencyAction { .. } => 5,
            Authority::TrialCourt { .. } => 4,
            Authority::ProfessionalBody { .. } => 3,
        }
    }
}

/// Whether authority is binding or persuasive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingStatus {
    Binding,
    Persuasive,
}

/// Citation formats.
#[derive(Debug, Clone, PartialEq)]
pub enum Citation {
    Statute {
        code: String,
        section: String,
        subsection: Option<String>,
        short: Option<String>,
    },
    Regulation {
        code: String,
        part: String,
        section: String,
    },
    CaseLaw {
        name: String,
        reporter: String,
        volume: u32,
        page: u32,
        court: String,
        year: u16,
    },
    SupremeCourt {
        name: String,
        reporter_volume: u32,
        page: u32,
        year: u16,
    },
    ProfessionalRule {
        body: String,
        rule: String,
    },
    AdministrativeRuling {
        body: String,
        date: NaiveDate,
        description: String,
    },
}

/// Jurisdiction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jurisdiction {
    Federal,
    Supreme,
    Appellate(String),
    State(String),
    International,
}
