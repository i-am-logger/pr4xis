use crate::authority::Authority;
use crate::lifecycle::PhaseTag;
use praxis_category::{Category, Entity, Relationship};
use praxis_ontology::{Axiom, Quality};
use std::collections::HashMap;

/// Valence of a legal term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Valence {
    Supportive, // pro-claimant
    Defensive,  // pro-respondent
    Procedural, // scope, jurisdiction
}

/// Proof standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofStandard {
    Preponderance,
    ClearAndConvincing,
    BeyondReasonableDoubt,
}

/// Obligation language.
#[derive(Debug, Clone, PartialEq)]
pub enum ObligationLanguage {
    Mandatory { word: String },     // "shall", "must"
    Discretionary { word: String }, // "may", "can"
    Prohibitive { word: String },   // "shall not"
}

/// A deadline triggered by an event.
#[derive(Debug, Clone, PartialEq)]
pub struct Deadline {
    pub duration: DeadlineDuration,
    pub trigger: String,
    pub consequence: Option<String>,
    pub source_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeadlineDuration {
    Days(u32),
    Months(u32),
    Immediate,
}

/// Burden of proof.
#[derive(Debug, Clone, PartialEq)]
pub struct BurdenOfProof {
    pub standard: ProofStandard,
    pub borne_by: String,
    pub source_text: String,
}

/// A remedy available under a legal term.
#[derive(Debug, Clone, PartialEq)]
pub struct Remedy {
    pub name: String,
    pub description: String,
    pub source_text: String,
}

/// An obligation imposed by a legal term.
#[derive(Debug, Clone, PartialEq)]
pub struct Obligation {
    pub actor: String,
    pub action: String,
    pub language: ObligationLanguage,
    pub source_text: String,
}

/// An exception to a rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Exception {
    pub to_rule: String,
    pub exception: String,
    pub source_text: String,
}

/// Evidence requirement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequirementLevel {
    Required,
    Recommended,
    Optional,
}

/// Evidence type expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceType {
    Date,
    Entity,
    Document,
    Currency,
    Duration,
    Narrative,
    Count,
    Text,
}

/// An evidence requirement for a legal term.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRequirement {
    pub field: String,
    pub field_type: EvidenceType,
    pub required: RequirementLevel,
    pub description: Option<String>,
}

/// A legal term — an object in the legal category.
#[derive(Debug, Clone, PartialEq)]
pub struct LegalTerm {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub source_text: Option<String>,
    pub valence: Valence,
    pub subsection: Option<String>,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub obligations: Vec<Obligation>,
    pub deadlines: Vec<Deadline>,
    pub rights: Vec<String>,
    pub remedies: Vec<Remedy>,
    pub burdens: Vec<BurdenOfProof>,
    pub exceptions: Vec<Exception>,
}

/// Relation types between legal terms — morphisms in the category.
#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    Requires,
    Precedes { max_days: Option<i64> },
    Implies { consequence: String },
    Contradicts,
    Composes { into: String },
    SubtypeOf,
    Triggers { obligation: String },
    Negates,
    AlternativeTo,
    Rebuts { burden: String },
    AffirmativeDefenseTo,
    SafeHarborFor,
    ExhaustionRequiredFor,
}

/// A relation between two legal terms.
#[derive(Debug, Clone, PartialEq)]
pub struct LegalRelation {
    pub from: String,
    pub to: String,
    pub relation: RelationType,
}

/// A legal category: a body of law with terms and their relations.
#[derive(Debug, Clone, PartialEq)]
pub struct LegalCategory {
    pub name: String,
    pub description: String,
    pub authority: Authority,
    pub terms: Vec<LegalTerm>,
    pub relations: Vec<LegalRelation>,
}

/// Validation result for a typed fact against a term.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationCompleteness {
    Complete,
    Sufficient,
    Insufficient { missing_required: Vec<String> },
}

/// Registry of legal categories.
#[derive(Debug, Clone)]
pub struct OntologyRegistry {
    pub categories: HashMap<String, LegalCategory>,
}

impl OntologyRegistry {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
        }
    }

    pub fn register(&mut self, category: LegalCategory) {
        self.categories.insert(category.name.clone(), category);
    }

    pub fn get_category(&self, name: &str) -> Option<&LegalCategory> {
        self.categories.get(name)
    }

    pub fn get_term(&self, term_id: &str) -> Option<&LegalTerm> {
        for cat in self.categories.values() {
            if let Some(term) = cat.terms.iter().find(|t| t.id == term_id) {
                return Some(term);
            }
        }
        None
    }
}

impl Default for OntologyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// rust-ontology trait implementations: Entity, Category, Quality, Axiom
// =============================================================================

/// Case phases as entities.
impl Entity for PhaseTag {
    fn variants() -> Vec<Self> {
        vec![
            PhaseTag::PreFiling,
            PhaseTag::Filed,
            PhaseTag::Discovery,
            PhaseTag::Motions,
            PhaseTag::PreTrial,
            PhaseTag::Trial,
            PhaseTag::PostTrial,
            PhaseTag::Appeal,
            PhaseTag::Closed,
        ]
    }
}

/// Phase transition relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseTransitionRel {
    pub from: PhaseTag,
    pub to: PhaseTag,
}

impl Relationship for PhaseTransitionRel {
    type Object = PhaseTag;
    fn source(&self) -> PhaseTag {
        self.from
    }
    fn target(&self) -> PhaseTag {
        self.to
    }
}

/// The case lifecycle as a category.
pub struct CaseLifecycleCategory;

impl Category for CaseLifecycleCategory {
    type Object = PhaseTag;
    type Morphism = PhaseTransitionRel;

    fn identity(obj: &PhaseTag) -> PhaseTransitionRel {
        PhaseTransitionRel {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &PhaseTransitionRel, g: &PhaseTransitionRel) -> Option<PhaseTransitionRel> {
        if f.to != g.from {
            return None;
        }
        Some(PhaseTransitionRel {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<PhaseTransitionRel> {
        let phases = PhaseTag::variants();
        let mut m = Vec::new();
        for &p in &phases {
            m.push(PhaseTransitionRel { from: p, to: p });
            for &t in &p.valid_transitions() {
                m.push(PhaseTransitionRel { from: p, to: t });
            }
        }
        // Composites for closure
        let direct = m.clone();
        for f in &direct {
            for g in &direct {
                if f.to == g.from {
                    let composed = PhaseTransitionRel {
                        from: f.from,
                        to: g.to,
                    };
                    if !m.contains(&composed) {
                        m.push(composed);
                    }
                }
            }
        }
        m
    }
}

/// Quality: is this phase terminal?
#[derive(Debug, Clone)]
pub struct IsTerminalPhase;

impl Quality for IsTerminalPhase {
    type Individual = PhaseTag;
    type Value = ();
    fn get(&self, phase: &PhaseTag) -> Option<()> {
        if phase.is_terminal() { Some(()) } else { None }
    }
}

/// Axiom: only Closed is terminal.
pub struct OnlyClosedIsTerminal;

impl Axiom<CaseLifecycleCategory> for OnlyClosedIsTerminal {
    fn description(&self) -> &str {
        "only Closed is a terminal phase"
    }
    fn holds(&self) -> bool {
        PhaseTag::variants()
            .iter()
            .all(|p| p.is_terminal() == (*p == PhaseTag::Closed))
    }
}

/// Axiom: every non-terminal phase has at least one transition.
pub struct NoDeadPhases;

impl Axiom<CaseLifecycleCategory> for NoDeadPhases {
    fn description(&self) -> &str {
        "every non-terminal phase has transitions"
    }
    fn holds(&self) -> bool {
        PhaseTag::variants()
            .iter()
            .all(|p| p.is_terminal() || !p.valid_transitions().is_empty())
    }
}
