#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use hashbrown::HashMap;

use super::authority::Authority;
use super::lifecycle::PhaseTag;
use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

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
    Concept,
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
// praxis trait implementations: Entity, Category, Quality, Axiom
// =============================================================================

/// Case phases as entities.
impl Concept for PhaseTag {
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

/// Relation kind for the judicial case lifecycle category.
///
/// Per OBO-RO (Smith et al. 2005), every arrow carries a relation-kind
/// tag. The case lifecycle has one relation: phase transition under
/// procedural rules (Hart 1961 secondary rules; Sartor 2005 ch. 7
/// procedural norms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudicialRelationKind {
    PhaseTransition,
}

/// Phase transition arrow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseTransitionRel {
    pub from: PhaseTag,
    pub to: PhaseTag,
}

impl Arrow for PhaseTransitionRel {
    type Object = PhaseTag;
    type Kind = JudicialRelationKind;
    fn source(&self) -> PhaseTag {
        self.from
    }
    fn target(&self) -> PhaseTag {
        self.to
    }
    fn kind(&self) -> JudicialRelationKind {
        JudicialRelationKind::PhaseTransition
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("PhaseTransitionRel"),
            description: Label::new_static(
                "case lifecycle phase transition under procedural rules",
            ),
            citation: Citation::parse_static("Hart (1961); Sartor (2005)"),
            module_path: ModulePath::new_static(module_path!()),
        }
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
        let candidate = PhaseTransitionRel {
            from: f.from,
            to: g.to,
        };
        // Partial category (#166): only emit composites that are themselves
        // declared morphisms. `morphisms()` builds the full reachability
        // closure (Warshall 1962), so any composable pair lands inside it.
        if Self::morphisms().contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<PhaseTransitionRel> {
        use hashbrown::HashSet;
        let phases = PhaseTag::variants();
        // Direct edges = identities + declared valid_transitions
        let mut direct: HashSet<(PhaseTag, PhaseTag)> = HashSet::new();
        for &p in &phases {
            direct.insert((p, p));
            for &t in &p.valid_transitions() {
                direct.insert((p, t));
            }
        }
        // Warshall transitive closure — required for associativity:
        // every (f∘g)∘h and f∘(g∘h) must produce a member of morphisms().
        let mut closure = direct.clone();
        loop {
            let mut added = false;
            let snap: Vec<_> = closure.iter().cloned().collect();
            for &(a, b) in &snap {
                for &(b2, c) in &snap {
                    if b == b2 && !closure.contains(&(a, c)) {
                        closure.insert((a, c));
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        closure
            .into_iter()
            .map(|(a, b)| PhaseTransitionRel { from: a, to: b })
            .collect()
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

/// Axiom: only `Closed` is a terminal phase.
///
/// Sartor (2005) ch. 7: procedural norms partition phases into transitional
/// (those with at least one outgoing procedural move) and terminal
/// (those from which no further procedural move is defined). For the
/// judicial case lifecycle modelled here, `Closed` is the unique
/// terminal phase.
pub struct OnlyClosedIsTerminal;

impl Axiom for OnlyClosedIsTerminal {
    fn verify(&self) -> Verdict {
        if PhaseTag::variants()
            .iter()
            .all(|p| p.is_terminal() == (*p == PhaseTag::Closed))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "OnlyClosedIsTerminal",
        "only Closed is a terminal phase",
        "Hart (1961); Sartor (2005) ch. 7"
    );
}
pr4xis::register_axiom!(OnlyClosedIsTerminal, "Hart (1961); Sartor (2005) ch. 7");

/// Axiom: every non-terminal phase has at least one valid transition.
///
/// Hart (1961) "rules of change": a legal process without an
/// onward procedural move from a non-terminal state would be
/// deadlocked. Sartor (2005) ch. 7 frames this as a well-formedness
/// constraint on procedural norm systems.
pub struct NoDeadPhases;

impl Axiom for NoDeadPhases {
    fn verify(&self) -> Verdict {
        if PhaseTag::variants()
            .iter()
            .all(|p| p.is_terminal() || !p.valid_transitions().is_empty())
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "NoDeadPhases",
        "every non-terminal phase has at least one valid transition",
        "Hart (1961); Sartor (2005) ch. 7"
    );
}
pr4xis::register_axiom!(NoDeadPhases, "Hart (1961); Sartor (2005) ch. 7");

/// The judicial case lifecycle ontology.
pub struct CaseLifecycleOntology;

impl Ontology for CaseLifecycleOntology {
    type Cat = CaseLifecycleCategory;
    type Qual = IsTerminalPhase;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(OnlyClosedIsTerminal), Box::new(NoDeadPhases)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<CaseLifecycleCategory>();
    }

    #[test]
    fn ontology_validates() {
        CaseLifecycleOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
