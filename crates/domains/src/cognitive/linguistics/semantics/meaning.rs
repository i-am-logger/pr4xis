#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// Semantic role — the function an entity plays in a proposition.
/// "The dog chased the cat" → dog=Agent, cat=Patient, chased=Predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticRole {
    Agent,
    Patient,
    Theme,
    Experiencer,
    Instrument,
    Location,
    Goal,
    Source,
}

impl Concept for SemanticRole {
    fn variants() -> Vec<Self> {
        vec![
            Self::Agent,
            Self::Patient,
            Self::Theme,
            Self::Experiencer,
            Self::Instrument,
            Self::Location,
            Self::Goal,
            Self::Source,
        ]
    }
}

/// A predicate — the core meaning of a verb or relational expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Predicate {
    pub lemma: String,
    pub arity: usize,
}

impl Predicate {
    pub fn unary(lemma: &str) -> Self {
        Self {
            lemma: lemma.into(),
            arity: 1,
        }
    }

    pub fn binary(lemma: &str) -> Self {
        Self {
            lemma: lemma.into(),
            arity: 2,
        }
    }

    pub fn ternary(lemma: &str) -> Self {
        Self {
            lemma: lemma.into(),
            arity: 3,
        }
    }
}

/// An entity reference in semantic representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityRef {
    pub name: String,
    pub role: SemanticRole,
}

/// A proposition — a predicate applied to entities with roles.
/// "The dog runs" → Proposition { predicate: run, arguments: [dog:Agent] }
/// "The dog sees the cat" → Proposition { predicate: see, arguments: [dog:Agent, cat:Patient] }
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticProposition {
    pub predicate: Predicate,
    pub arguments: Vec<EntityRef>,
    pub negated: bool,
}

impl SemanticProposition {
    pub fn new(predicate: Predicate, arguments: Vec<EntityRef>) -> Self {
        Self {
            predicate,
            arguments,
            negated: false,
        }
    }

    pub fn negated(mut self) -> Self {
        self.negated = !self.negated;
        self
    }

    pub fn agent(&self) -> Option<&EntityRef> {
        self.arguments
            .iter()
            .find(|a| a.role == SemanticRole::Agent)
    }

    pub fn patient(&self) -> Option<&EntityRef> {
        self.arguments
            .iter()
            .find(|a| a.role == SemanticRole::Patient)
    }

    pub fn describe(&self) -> String {
        let neg = if self.negated { "NOT " } else { "" };
        let args: Vec<String> = self
            .arguments
            .iter()
            .map(|a| format!("{}:{:?}", a.name, a.role))
            .collect();
        format!("{}{}({})", neg, self.predicate.lemma, args.join(", "))
    }

    /// Check arity: does the proposition have the right number of arguments?
    pub fn is_well_formed(&self) -> bool {
        self.arguments.len() == self.predicate.arity
    }
}

/// A meaning representation — one or more propositions composed with logical connectives.
#[derive(Debug, Clone, PartialEq)]
pub enum MeaningRep {
    Atomic(SemanticProposition),
    And(Box<MeaningRep>, Box<MeaningRep>),
    Or(Box<MeaningRep>, Box<MeaningRep>),
    Not(Box<MeaningRep>),
    Implies(Box<MeaningRep>, Box<MeaningRep>),
}

impl MeaningRep {
    pub fn and(self, other: MeaningRep) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: MeaningRep) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }

    pub fn negate(self) -> Self {
        Self::Not(Box::new(self))
    }

    pub fn implies(self, consequent: MeaningRep) -> Self {
        Self::Implies(Box::new(self), Box::new(consequent))
    }

    /// Extract all atomic propositions.
    pub fn propositions(&self) -> Vec<&SemanticProposition> {
        match self {
            Self::Atomic(p) => vec![p],
            Self::And(a, b) | Self::Or(a, b) | Self::Implies(a, b) => {
                let mut props = a.propositions();
                props.extend(b.propositions());
                props
            }
            Self::Not(inner) => inner.propositions(),
        }
    }

    /// Check if all propositions are well-formed (correct arity).
    pub fn is_well_formed(&self) -> bool {
        self.propositions().iter().all(|p| p.is_well_formed())
    }

    pub fn describe(&self) -> String {
        match self {
            Self::Atomic(p) => p.describe(),
            Self::And(a, b) => format!("({} AND {})", a.describe(), b.describe()),
            Self::Or(a, b) => format!("({} OR {})", a.describe(), b.describe()),
            Self::Not(inner) => format!("NOT({})", inner.describe()),
            Self::Implies(a, b) => format!("({} → {})", a.describe(), b.describe()),
        }
    }
}
