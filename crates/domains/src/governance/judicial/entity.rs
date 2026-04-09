use super::source::Source;
use chrono::NaiveDate;

/// A legal actor — any person or organization involved in a case.
#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Person(Person),
    Corporation(Corporation),
    LawFirm(LawFirm),
    Agency(Agency),
    Court(Court),
}

impl Entity {
    pub fn name(&self) -> &str {
        match self {
            Entity::Person(p) => &p.name,
            Entity::Corporation(c) => &c.name,
            Entity::LawFirm(f) => &f.name,
            Entity::Agency(a) => &a.name,
            Entity::Court(c) => &c.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub name: String,
    pub title: Option<String>,
    pub organization: Option<Box<Entity>>,
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
        parent: Box<Entity>,
    },
    SpinOff {
        parent: Box<Entity>,
        date: NaiveDate,
    },
    JointVenture {
        partners: Vec<Entity>,
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
        parent: Entity,
        child: Entity,
        source: Source,
    },
    Employment {
        person: Entity,
        organization: Entity,
        role: String,
        tenure: Tenure,
        source: Source,
    },
    Legal {
        counsel: Entity,
        client: Entity,
        matter: Option<String>,
        status: RepresentationStatus,
        source: Source,
    },
    SupplyChain {
        supplier: Entity,
        customer: Entity,
        revenue_pct: Option<f64>,
        source: Source,
    },
}
