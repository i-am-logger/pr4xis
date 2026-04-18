#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::source::Source;
use chrono::NaiveDate;

/// A legal actor — any person or organization involved in a case.
#[derive(Debug, Clone, PartialEq)]
pub enum Concept {
    Person(Person),
    Corporation(Corporation),
    LawFirm(LawFirm),
    Agency(Agency),
    Court(Court),
}

impl Concept {
    pub fn name(&self) -> &str {
        match self {
            Concept::Person(p) => &p.name,
            Concept::Corporation(c) => &c.name,
            Concept::LawFirm(f) => &f.name,
            Concept::Agency(a) => &a.name,
            Concept::Court(c) => &c.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub name: String,
    pub title: Option<String>,
    pub organization: Option<Box<Concept>>,
    pub bar_admissions: Vec<String>,
    pub source: Option<Source>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Corporation {
    pub name: String,
    pub structure: CorporateStructure,
    pub jurisdiction: String,
    pub source: Option<Source>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CorporateStructure {
    PublicCompany,
    Subsidiary {
        parent: Box<Concept>,
    },
    SpinOff {
        parent: Box<Concept>,
        date: NaiveDate,
    },
    JointVenture {
        partners: Vec<Concept>,
    },
    Partnership,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LawFirm {
    pub name: String,
    pub source: Option<Source>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Agency {
    pub name: String,
    pub jurisdiction: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Court {
    pub name: String,
    pub district: Option<String>,
    pub circuit: Option<String>,
}

/// Tenure — NOT a bool `is_current`.
#[derive(Debug, Clone, PartialEq)]
pub enum Tenure {
    Current { start: NaiveDate },
    Former { start: NaiveDate, end: NaiveDate },
}

/// Representation status — NOT a bool.
#[derive(Debug, Clone, PartialEq)]
pub enum RepresentationStatus {
    Current { since: Option<NaiveDate> },
    Former { ended: Option<NaiveDate> },
}

/// Relationships between entities.
#[derive(Debug, Clone, PartialEq)]
pub enum Relationship {
    Corporate {
        parent: Concept,
        child: Concept,
        source: Source,
    },
    Employment {
        person: Concept,
        organization: Concept,
        role: String,
        tenure: Tenure,
        source: Source,
    },
    Legal {
        counsel: Concept,
        client: Concept,
        matter: Option<String>,
        status: RepresentationStatus,
        source: Source,
    },
    SupplyChain {
        supplier: Concept,
        customer: Concept,
        revenue_pct: Option<f64>,
        source: Source,
    },
}
