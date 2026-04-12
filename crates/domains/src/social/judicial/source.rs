use chrono::{DateTime, NaiveDate, Utc};
use std::path::PathBuf;

/// Reliability tier for evidence sources (1 = highest, 4 = lowest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceTier(pub u8);

impl SourceTier {
    pub fn tier1() -> Self {
        SourceTier(1)
    }
    pub fn tier2() -> Self {
        SourceTier(2)
    }
    pub fn tier3() -> Self {
        SourceTier(3)
    }
    pub fn tier4() -> Self {
        SourceTier(4)
    }
}

/// Lightweight reference to a source (avoids recursive nesting).
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRef {
    pub url: Option<String>,
    pub description: String,
    pub tier: SourceTier,
}

/// Verification state of evidence — NOT a bool.
#[derive(Debug, Clone, PartialEq)]
pub enum Verification {
    Verified {
        confidence: f64,
        verified_at: DateTime<Utc>,
        method: String,
        corroborated_by: Vec<SourceRef>,
    },
    Partial {
        confidence: f64,
        verified_at: DateTime<Utc>,
        what_verified: String,
        what_remains: String,
    },
    Unverified {
        reason: Option<String>,
    },
    Stale {
        was_verified_at: DateTime<Utc>,
        stale_since: DateTime<Utc>,
    },
}

/// Type of document providing evidence.
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    CourtDocket {
        court: String,
        case_number: String,
        docket_id: Option<String>,
    },
    PressArticle {
        publication: String,
        date: NaiveDate,
        author: Option<String>,
    },
    LegalOpinion {
        court: String,
        case_name: String,
    },
    Exhibit {
        label: String,
        filed_with: String,
        proves: Vec<String>,
    },
    Declaration {
        declarant: String,
        date: NaiveDate,
    },
    PolicyReport {
        organization: String,
        date: NaiveDate,
    },
    RegulatoryFiling {
        agency: String,
        form_type: String,
        filed: NaiveDate,
    },
    Spreadsheet {
        filename: String,
        sheet: String,
    },
    Recording {
        participants: Vec<String>,
        date: NaiveDate,
        transcribed: bool,
    },
    Website {
        organization: String,
        page_type: String,
    },
    Other {
        description: String,
    },
}

/// Archived copy of evidence for integrity.
#[derive(Debug, Clone, PartialEq)]
pub struct Archive {
    pub text_path: Option<PathBuf>,
    pub image_path: Option<PathBuf>,
    pub sha256: Option<String>,
    pub wayback_url: Option<String>,
}

/// A source of evidence with full provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub url: Option<String>,
    pub document: DocumentType,
    pub tier: SourceTier,
    pub retrieved_at: DateTime<Utc>,
    pub archive: Option<Archive>,
    pub verification: Verification,
}
