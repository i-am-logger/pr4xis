#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use hashbrown::HashMap;

use super::authority::Authority;
use super::citation::PinpointCite;
use super::lifecycle::PhaseTag;
use super::source_text::SourceTextRef;
use crate::formal::meta::identifier_format::Identifier;
use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// Faithful praxis-typed concepts (hand-coded prototypes of what codegen
// will produce when the PDF loader + NLP extraction infrastructure are
// ready — each one is verbatim with a single primary source so the
// future-loaded version is semantically identical):

// `ProofStandard` ← McCauliff (1982) "Burdens of Proof", U. Pittsburgh
// L. Rev. 35:1293 + In re Winship, 397 U.S. 358 (1970): exactly three
// classical tiers (Preponderance / ClearAndConvincing / BeyondReasonableDoubt).
pub use super::proof_standard::ontology::ProofStandardConcept as ProofStandard;

// `RequirementLevel` ← RFC 2119 / BCP 14 (Bradner 1997): exactly three
// requirement levels (Required / Recommended / Optional).
pub use super::evidence_requirement::ontology::RequirementLevelConcept as RequirementLevel;

// `ObligationLanguage` ← von Wright (1951) "Deontic Logic", Mind 60:
// exactly three deontic primitives (Mandatory / Discretionary /
// Prohibitive — O / P / F operators). The surface modal word is a
// separate typed `SourceTextRef` field on `Obligation`.
pub use super::modality::ontology::ObligationModalityConcept as ObligationLanguage;

// Removed:
//
// - `Valence` (Supportive/Defensive/Procedural): synthesized trichotomy
//   with no single primary source attesting the partition. The
//   `valence` field on `LegalTerm` is removed entirely; a primary
//   source attesting the trichotomy (if found later) would restore it.
//
// - `LegalActor` (typed enum with Plaintiff/Defendant/Court/etc. in a
//   four-family hierarchy): the four-family Party/Adjudicator/Witness/
//   Counsel grouping is synthesis across multiple primary sources
//   (FRCP Rule 17 + FRCP 38 + FRCP 72 + FRE 702 + ABA Rule 3.7 + …).
//   Container fields previously typed as `LegalActor` are now typed
//   as `Identifier` (CURIE references) — they resolve into the union
//   of loaded per-source actor concepts via the `SourceTaxonomy`
//   `Adjoins` graph.
//
// - `DeadlineDuration` typed enum (Day / BusinessDay / Week / Month /
//   Year / Immediate unified into one ontology): synthesis across
//   ISO 8601 + TimeML + FRCP Rule 6(a)(6). Replaced by the `Duration`
//   value type below, which holds an `Identifier`-typed unit (CURIE
//   reference into a per-source granularity ontology) and a numeric
//   count.

/// A typed duration value. `unit` is an [`Identifier`] CURIE pointing
/// at a granularity concept in a primary-source ontology (e.g.,
/// `iso8601_calendar:day`, `frcp_rule_6:business_day`, or
/// `timeml:immediate`). `count` is the numeric count of `unit`s;
/// ignored when the unit is an instantaneous concept (TimeML PT0S).
///
/// This is the praxis-bottom-up replacement for the previously-
/// synthesized `TemporalConstraint`-typed `Duration`: rather than
/// unifying ISO 8601 + TimeML + FRCP concepts into one fabricated
/// ontology, we type the unit as a CURIE and let the resolver walk the
/// `SourceTaxonomy` `Adjoins` graph to the right primary-source
/// ontology.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Duration {
    pub unit: Identifier,
    pub count: u32,
}

/// A deadline triggered by an event.
///
/// Every field is a typed praxis concept — no bare `String`. The
/// `duration` is a typed [`Duration`] (unit-as-CURIE + count); the
/// trigger phrase, optional consequence, and verbatim source citation
/// are typed [`SourceTextRef`] values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Deadline {
    pub duration: Duration,
    pub trigger: SourceTextRef,
    pub consequence: Option<SourceTextRef>,
    pub source_text: SourceTextRef,
}

/// Burden of proof — typed standard + actor (CURIE) + verbatim citation.
///
/// `standard` is the typed [`ProofStandard`] concept (McCauliff 1982 /
/// Winship 1970 grounded). `borne_by` is an [`Identifier`] CURIE
/// pointing at the actor concept (e.g., `frcp_rule_17:plaintiff`) in
/// the per-source actor union resolved via the SourceTaxonomy `Adjoins`
/// graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BurdenOfProof {
    pub standard: ProofStandard,
    pub borne_by: Identifier,
    pub source_text: SourceTextRef,
}

/// A remedy available under a legal term. Name / description / citing
/// are typed [`SourceTextRef`] values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Remedy {
    pub name: SourceTextRef,
    pub description: SourceTextRef,
    pub source_text: SourceTextRef,
}

/// An obligation imposed by a legal term.
///
/// `actor` is an [`Identifier`] CURIE pointing at the actor concept
/// (e.g., `frcp_rule_17:plaintiff`, `sox_1514a:a` for SOX 1514A's
/// "Covered Employer", etc.) — resolves into the union of loaded
/// per-source actor concepts via the SourceTaxonomy `Adjoins` graph.
/// `modality` is the typed deontic mode (Mandatory / Prohibitive /
/// Discretionary, von Wright 1951); the surface modal word is a
/// separate `modal_word` field carrying the verbatim "shall" / "may
/// not" / etc. as a [`SourceTextRef`] for citation purposes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Obligation {
    pub actor: Identifier,
    pub action: SourceTextRef,
    pub modality: ObligationLanguage,
    pub modal_word: SourceTextRef,
    pub source_text: SourceTextRef,
}

/// An exception to a rule. `to_rule` is a typed [`Identifier`] (CURIE
/// reference to the rule being excepted); the exception text and
/// verbatim citation are typed [`SourceTextRef`] values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Exception {
    pub to_rule: Identifier,
    pub exception: SourceTextRef,
    pub source_text: SourceTextRef,
}

/// Evidence type expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// An evidence requirement for a legal term. Field name / description
/// are typed [`SourceTextRef`]; the requirement level is the typed
/// RFC 2119 concept.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EvidenceRequirement {
    pub field: SourceTextRef,
    pub field_type: EvidenceType,
    pub required: RequirementLevel,
    pub description: Option<SourceTextRef>,
}

/// A legal term — the typed value form of a statutory/regulatory
/// provision. Every field is a praxis concept or a typed value
/// (composed of praxis concepts) — no bare `String` anywhere.
///
/// The Lumen-shaped `valence` field is intentionally absent: the
/// Supportive/Defensive/Procedural trichotomy is a synthesis with no
/// single primary-source attestation. When a primary source attesting
/// the partition surfaces, the field gets restored typed against that
/// source's ontology.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegalTerm {
    pub id: Identifier,
    pub name: SourceTextRef,
    pub definition: SourceTextRef,
    pub source_text: Option<SourceTextRef>,
    pub subsection: Option<PinpointCite>,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub obligations: Vec<Obligation>,
    pub deadlines: Vec<Deadline>,
    pub rights: Vec<SourceTextRef>,
    pub remedies: Vec<Remedy>,
    pub burdens: Vec<BurdenOfProof>,
    pub exceptions: Vec<Exception>,
}

/// Relation types between legal terms — morphisms in the legal
/// category. Parametric variants carry typed praxis values:
/// `Precedes.max_days` is an `Option<Duration>` (TimeML-typed);
/// `Implies.consequence` is a `SourceTextRef`; `Composes.into` /
/// `Triggers.obligation` are typed `Identifier`s pointing at other
/// terms; `Rebuts.burden` is a `SourceTextRef` describing the burden.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    Requires,
    Precedes {
        max_days: Option<Duration>,
    },
    Implies {
        consequence: SourceTextRef,
    },
    Contradicts,
    Composes {
        into: Identifier,
    },
    SubtypeOf,
    Triggers {
        obligation: Identifier,
    },
    Negates,
    AlternativeTo,
    Rebuts {
        burden: SourceTextRef,
    },
    AffirmativeDefenseTo,
    SafeHarborFor,
    ExhaustionRequiredFor,
    /// A definitional provision establishes the meaning of a term within its
    /// scope — source is the defining provision, target is the defined term.
    /// The statutory-definition morphism; resolution among competing
    /// definitions is lex specialis (see
    /// `statute_structure::definition_scope`). Scalia & Garner (2012) §28;
    /// 1 U.S.C. §1.
    Defines,
    /// The inverse of [`RelationType::Defines`]: a use of a term is governed by
    /// the definition that establishes its meaning in the use's scope.
    DefinedIn,
}

/// A relation between two legal terms. `from` and `to` are typed
/// [`Identifier`]s (CURIE-validated).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegalRelation {
    pub from: Identifier,
    pub to: Identifier,
    pub relation: RelationType,
}

/// A legal category: a body of law with typed terms and relations.
/// Not `Eq`/`Hash` because `Authority` doesn't implement them yet.
#[derive(Debug, Clone, PartialEq)]
pub struct LegalCategory {
    pub name: SourceTextRef,
    pub description: SourceTextRef,
    pub authority: Authority,
    pub terms: Vec<LegalTerm>,
    pub relations: Vec<LegalRelation>,
}

/// Validation result for a typed fact against a term. `missing_required`
/// is a list of typed [`SourceTextRef`] field names that the fact failed
/// to satisfy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationCompleteness {
    Complete,
    Sufficient,
    Insufficient {
        missing_required: Vec<SourceTextRef>,
    },
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
        self.categories.insert(category.name.text.clone(), category);
    }

    pub fn get_category(&self, name: &str) -> Option<&LegalCategory> {
        self.categories.get(name)
    }

    pub fn get_term(&self, term_id: &str) -> Option<&LegalTerm> {
        for cat in self.categories.values() {
            if let Some(term) = cat.terms.iter().find(|t| t.id.value() == term_id) {
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
impl Concept for PhaseTag {}
impl FinitelyGenerated for PhaseTag {
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

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CaseLifecycleCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        CaseLifecycleOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
