//! Rule Algebra — the pure-science ontology of operations over
//! conditional rules: subsumption, normalization, conflict detection.
//!
//! This is a PURE-SCIENCE ontology. The runtime counterparts (typed
//! [`super::Implication`], [`super::RuleSet`], with `normalize` /
//! `subsumes` / `conflicts_with` implementations) live in
//! [`super::implication`] and [`super::rule_set`]. The concept
//! inventory here names the operations and their outputs.
//!
//! # Position in praxis
//!
//! Praxis already carries:
//!
//! - [`crate::social::judicial::rule::Rule`] — the operational, fact-
//!   driven legal rule (used by the evaluation engine).
//! - [`crate::social::judicial::ontology::RelationType`] — 13 relation
//!   kinds (Requires, Implies, Contradicts, Triggers, …).
//! - [`crate::formal::derivation::ontology`] — Peirce/Gentzen
//!   reasoning vocabulary.
//! - [`crate::formal::causation::ontology`] — Lewis/Pearl/Reichenbach
//!   causation.
//! - [`crate::formal::analytical_methods::fca`] — FCA concept lattice
//!   whose Duquenne-Guigues *implication basis* (Duquenne & Guigues
//!   1986) is the canonical-form output of the normalization
//!   operation declared here.
//! - Deontic operators in [`crate::social::judicial::ontology`] (via
//!   `ObligationModalityOntology` — Obligation, Permission,
//!   Prohibition per von Wright 1951).
//!
//! This module COMPOSES those into the pure algebraic vocabulary of
//! rule subsumption/normalization/conflict — no new substrate, only
//! the operations that arise when we treat each rule as a logical
//! object rather than an evaluator.
//!
//! # Literature
//!
//! - **Robinson (1965)** "A Machine-Oriented Logic Based on the
//!   Resolution Principle", *JACM* 12: 23–41 — clause subsumption
//!   under resolution.
//! - **Plotkin (1970)** "A Note on Inductive Generalization",
//!   *Machine Intelligence* 5: 153–163 — θ-subsumption ordering on
//!   clauses; the canonical "more general than" partial order.
//! - **Duquenne & Guigues (1986)** "Familles minimales d'implications
//!   informatives résultant d'un tableau de données binaires",
//!   *Mathématiques et Sciences Humaines* 95: 5–18 — the canonical
//!   *Stem Basis* (Duquenne-Guigues basis) of implications of a
//!   formal context: a minimal generating set under closure.
//! - **Tarski (1956)** "On Some Fundamental Concepts of
//!   Metamathematics", in *Logic, Semantics, Metamathematics*,
//!   Oxford UP — consequence operators, closure under entailment.
//! - **Reiter (1980)** "A Logic for Default Reasoning", *Artificial
//!   Intelligence* 13: 81–132 — default rules, defeasibility.
//! - **Pollock (1987)** "Defeasible Reasoning", *Cognitive Science*
//!   11: 481–518 — undercutting and rebutting defeaters.
//! - **Prakken & Sartor (1997)** "Argument-Based Extended Logic
//!   Programming with Defeasible Priorities", *Journal of Applied
//!   Non-Classical Logics* 7: 25–75 — legal rule conflicts;
//!   priority-based resolution.
//! - **Modgil & Prakken (2014)** "The ASPIC+ framework for structured
//!   argumentation: a tutorial", *Argument & Computation* 5: 31–62.
//! - **von Wright (1951)** "Deontic Logic", *Mind* 60: 1–15 —
//!   Obligation / Permission / Prohibition operators; the deontic
//!   square of opposition (Obligation ↔ ¬Prohibition; conflict
//!   = Obligation(p) ∧ Prohibition(p)).
//! - **McCarthy (1980)** "Circumscription — A Form of Non-Monotonic
//!   Reasoning", *Artificial Intelligence* 13: 27–39 — closure under
//!   minimal models; relevant to normalization.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "RuleAlgebra",
    source: "Robinson (1965) JACM 12: 23-41; Plotkin (1970) Machine Intelligence 5: 153-163; Duquenne & Guigues (1986) Math. Sci. Hum. 95: 5-18; Tarski (1956) Logic, Semantics, Metamathematics; Reiter (1980) AI 13: 81-132; Pollock (1987) Cog. Sci. 11: 481-518; Prakken & Sartor (1997) JANCL 7: 25-75; Modgil & Prakken (2014) Argument & Computation 5: 31-62; von Wright (1951) Mind 60: 1-15",

    concepts: [
        // === Rule shape (Robinson 1965, Plotkin 1970) ===
        Implication,           // antecedent ⇒ consequent
        Antecedent,            // the conditions, a finite set of concepts
        Consequent,            // the conclusion, a finite set of concepts
        StrictRule,            // no exceptions admitted (Reiter 1980 "monotonic")
        DefeasibleRule,        // overridable by defeaters (Pollock 1987)
        Defeater,              // a fact/rule that undercuts another (Pollock 1987)

        // === Deontic flavours (von Wright 1951) ===
        Obligation,            // Op  — "p is required"
        Permission,            // Pp  — "p is allowed"
        Prohibition,           // Fp  — "p is forbidden"
        Assertoric,            // plain "p holds" — no deontic flavour

        // === Operations on rules ===
        Subsumption,           // R1 ≼ R2 (Plotkin 1970 θ-subsumption)
        Normalization,         // R ↦ canonical(R) (Tarski 1956 closure)
        ConflictDetection,     // (R1, R2) ↦ Conflict | None

        // === Outputs ===
        SubsumptionOrder,      // the partial order on a rule set
        CanonicalBasis,        // Duquenne-Guigues 1986 minimal generator
        ConflictPair,          // (R1, R2) in deontic conflict
        ConflictSet,           // the set of all conflict pairs

        // === Abstract categories ===
        RuleShape,             // ⊇ Implication, StrictRule, DefeasibleRule
        DeonticFlavour,        // ⊇ Obligation, Permission, Prohibition, Assertoric
        RuleOperation,         // ⊇ Subsumption, Normalization, ConflictDetection
        RuleOutput,            // ⊇ SubsumptionOrder, CanonicalBasis, ConflictPair, ConflictSet

        // === Pipeline (typical rule-algebra workflow) ===
        Parsing,
        NormalizationStep,
        SubsumptionTest,
        ConflictTest,
        ResolutionStep,
        OutputAssembly,
    ],

    labels: {
        Implication: ("en", "Implication",
            "Robinson (1965): a propositional implication antecedent ⇒ consequent. The atom of rule algebra: a finite conjunction of concept conditions implies a finite conjunction of concept conclusions, optionally tagged with a deontic flavour (von Wright 1951)."),
        Antecedent: ("en", "Antecedent",
            "Robinson (1965): the conjunction of literals on the left of an implication. Plotkin (1970) θ-subsumption compares two implications by antecedent inclusion."),
        Consequent: ("en", "Consequent",
            "Robinson (1965): the conjunction of literals on the right of an implication. Plotkin (1970) θ-subsumption compares by consequent containment in the other direction (more-general → fewer commitments)."),
        StrictRule: ("en", "Strict rule",
            "Reiter (1980): a rule with no exceptions — every model in which the antecedent holds is one in which the consequent holds. Contrast DefeasibleRule."),
        DefeasibleRule: ("en", "Defeasible rule",
            "Pollock (1987): a rule that licenses its conclusion *unless* a defeater is present. Foundation for Prakken-Sartor (1997) legal-rule reasoning."),
        Defeater: ("en", "Defeater",
            "Pollock (1987): a fact or rule that defeats another. Two species: *rebutting* (assert the negation of the conclusion) and *undercutting* (attack the connection between antecedent and consequent)."),

        Obligation: ("en", "Obligation",
            "von Wright (1951): the deontic operator Op meaning 'p is required'. Op ∧ Fp is the canonical deontic conflict."),
        Permission: ("en", "Permission",
            "von Wright (1951): Pp meaning 'p is allowed'. Defined by Pp ≡ ¬F p (the dual of prohibition)."),
        Prohibition: ("en", "Prohibition",
            "von Wright (1951): Fp meaning 'p is forbidden'. Equivalent to O¬p."),
        Assertoric: ("en", "Assertoric",
            "A rule with no deontic flavour — plain 'if antecedent then consequent'. Permits classical entailment without modal interpretation."),

        Subsumption: ("en", "Subsumption",
            "Plotkin (1970) θ-subsumption: R1 subsumes R2 iff antecedent(R1) ⊆ antecedent(R2) and consequent(R1) ⊇ consequent(R2) — R1 fires on fewer conditions and concludes at least as much. Robinson (1965) §6 originally for resolution."),
        Normalization: ("en", "Normalization",
            "Tarski (1956): bring a rule into a canonical form so equivalent rules become equal. For FCA-derived implications the canonical basis is the Duquenne-Guigues stem basis (Duquenne & Guigues 1986)."),
        ConflictDetection: ("en", "Conflict detection",
            "Prakken & Sartor (1997): identify pairs of rules whose firing antecedents are jointly satisfiable but whose deontic-tagged consequents contradict (Op vs Fp on the same target)."),

        SubsumptionOrder: ("en", "Subsumption order",
            "Plotkin (1970): the partial order ≼ on a rule set induced by θ-subsumption. Reflexive (every rule subsumes itself) and transitive."),
        CanonicalBasis: ("en", "Canonical basis",
            "Duquenne & Guigues (1986): the minimal set of implications generating all implications valid in a context, called the *stem basis* or *Duquenne-Guigues basis*."),
        ConflictPair: ("en", "Conflict pair",
            "Prakken & Sartor (1997): an ordered pair (R1, R2) where R1 and R2 are in deontic conflict — R1 obligates p, R2 forbids p, with compatible antecedents."),
        ConflictSet: ("en", "Conflict set",
            "All ConflictPair instances in a rule set. The conflict-resolution problem (Reiter 1987) finds a *minimal hitting set* over the conflicts."),

        RuleShape: ("en", "Rule shape",
            "Abstract category — Implication, StrictRule, DefeasibleRule fall under it."),
        DeonticFlavour: ("en", "Deontic flavour",
            "Abstract category — Obligation, Permission, Prohibition, Assertoric fall under it (von Wright 1951 deontic operators)."),
        RuleOperation: ("en", "Rule operation",
            "Abstract category — Subsumption, Normalization, ConflictDetection fall under it."),
        RuleOutput: ("en", "Rule output",
            "Abstract category — SubsumptionOrder, CanonicalBasis, ConflictPair, ConflictSet fall under it."),

        Parsing: ("en", "Parsing",
            "Pipeline stage 1: extract Implication structures from input (legal text, statute terms, FCA implication basis)."),
        NormalizationStep: ("en", "Normalization step",
            "Pipeline stage 2: apply Normalization to every rule — sort antecedents and consequents, deduplicate."),
        SubsumptionTest: ("en", "Subsumption test",
            "Pipeline stage 3: compute the SubsumptionOrder over the normalized rule set."),
        ConflictTest: ("en", "Conflict test",
            "Pipeline stage 4: scan the rule set for ConflictPair instances under deontic flavour interaction."),
        ResolutionStep: ("en", "Resolution step",
            "Pipeline stage 5: resolve conflicts by priority (Prakken & Sartor 1997) or by computing a minimal hitting set (Reiter 1987)."),
        OutputAssembly: ("en", "Output assembly",
            "Pipeline stage 6: package the canonical rule set, subsumption order, and resolved conflicts for downstream consumers."),
    },

    is_a: [
        // Rule shapes.
        (Implication, RuleShape),
        (StrictRule, RuleShape),
        (DefeasibleRule, RuleShape),
        // Deontic flavours.
        (Obligation, DeonticFlavour),
        (Permission, DeonticFlavour),
        (Prohibition, DeonticFlavour),
        (Assertoric, DeonticFlavour),
        // Operations.
        (Subsumption, RuleOperation),
        (Normalization, RuleOperation),
        (ConflictDetection, RuleOperation),
        // Outputs.
        (SubsumptionOrder, RuleOutput),
        (CanonicalBasis, RuleOutput),
        (ConflictPair, RuleOutput),
        (ConflictSet, RuleOutput),
        // A StrictRule is an Implication with no defeaters.
        (StrictRule, Implication),
        // A DefeasibleRule is an Implication with potential defeaters.
        (DefeasibleRule, Implication),
    ],

    causes: [
        // Canonical pipeline.
        (Parsing, NormalizationStep),
        (NormalizationStep, SubsumptionTest),
        (SubsumptionTest, ConflictTest),
        (ConflictTest, ResolutionStep),
        (ResolutionStep, OutputAssembly),
    ],

    opposes: [
        // The classical deontic conflict (von Wright 1951): Obligation
        // and Prohibition are contradictory operators on the same target.
        (Obligation, Prohibition),
        (Prohibition, Obligation),
        // Strict and Defeasible are the two complementary rule kinds.
        (StrictRule, DefeasibleRule),
        (DefeasibleRule, StrictRule),
    ],
}

// =============================================================================
// Domain axioms — invariants the operations must satisfy.
// =============================================================================

fn subsumption_pair_exists(child: RuleAlgebraConcept, parent: RuleAlgebraConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    RuleAlgebraCategory::morphisms().iter().any(|m| {
        m.source() == child
            && m.target() == parent
            && m.kind() == RuleAlgebraRelationKind::Subsumption
    })
}

fn opposition_pair_exists(a: RuleAlgebraConcept, b: RuleAlgebraConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    RuleAlgebraCategory::morphisms().iter().any(|m| {
        m.source() == a && m.target() == b && m.kind() == RuleAlgebraRelationKind::Opposition
    })
}

/// Robinson (1965): an Implication is the substrate of the algebra —
/// every other rule kind (Strict / Defeasible) specialises it.
pub struct ImplicationIsRuleShape;

impl Axiom for ImplicationIsRuleShape {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = subsumption_pair_exists(
            RuleAlgebraConcept::Implication,
            RuleAlgebraConcept::RuleShape,
        ) && subsumption_pair_exists(
            RuleAlgebraConcept::StrictRule,
            RuleAlgebraConcept::Implication,
        ) && subsumption_pair_exists(
            RuleAlgebraConcept::DefeasibleRule,
            RuleAlgebraConcept::Implication,
        );
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ImplicationIsRuleShape",
        "Implication is a RuleShape and both StrictRule and DefeasibleRule specialise it",
        "Robinson (1965) JACM 12: 23-41; Reiter (1980) AI 13: 81-132"
    );
}

/// von Wright (1951): the deontic square of opposition. Obligation
/// and Prohibition are the canonical opposed operators.
pub struct DeonticSquareOpposes;

impl Axiom for DeonticSquareOpposes {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if opposition_pair_exists(
            RuleAlgebraConcept::Obligation,
            RuleAlgebraConcept::Prohibition,
        ) && opposition_pair_exists(
            RuleAlgebraConcept::Prohibition,
            RuleAlgebraConcept::Obligation,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeonticSquareOpposes",
        "Obligation and Prohibition are opposed via the deontic square of opposition",
        "von Wright (1951) Deontic Logic, Mind 60: 1-15"
    );
}

/// Reiter / Pollock: Strict and Defeasible rules are complementary.
pub struct StrictDefeasibleOppose;

impl Axiom for StrictDefeasibleOppose {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if opposition_pair_exists(
            RuleAlgebraConcept::StrictRule,
            RuleAlgebraConcept::DefeasibleRule,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StrictDefeasibleOppose",
        "StrictRule and DefeasibleRule are opposed kinds (monotonic vs non-monotonic licence of conclusion)",
        "Reiter (1980) AI 13: 81-132; Pollock (1987) Cog. Sci. 11: 481-518"
    );
}

pr4xis::register_axiom!(
    ImplicationIsRuleShape,
    "Robinson (1965) JACM 12: 23-41; Reiter (1980) AI 13: 81-132"
);
pr4xis::register_axiom!(
    DeonticSquareOpposes,
    "von Wright (1951) Deontic Logic, Mind 60: 1-15"
);
pr4xis::register_axiom!(
    StrictDefeasibleOppose,
    "Reiter (1980) AI 13: 81-132; Pollock (1987) Cog. Sci. 11: 481-518"
);

/// The scholarly lineage (author-year tradition) that introduces each
/// rule-algebra concept.
///
/// A closed set of the literatures composed by this ontology (see the
/// module-level `source:` in the `ontology!` block): each variant names
/// the tradition a concept descends from.
///
/// - [`RobinsonPlotkin`](Self::RobinsonPlotkin): Robinson (1965) JACM 12:
///   23-41; Plotkin (1970) Machine Intelligence 5: 153-163 — the
///   implication / clause shape (Implication, Antecedent, Consequent).
/// - [`ReiterPollock`](Self::ReiterPollock): Reiter (1980) AI 13: 81-132;
///   Pollock (1987) Cog. Sci. 11: 481-518 — strict vs defeasible rules
///   and defeaters.
/// - [`VonWright`](Self::VonWright): von Wright (1951) Mind 60: 1-15 — the
///   deontic operators (Obligation / Permission / Prohibition / Assertoric).
/// - [`Plotkin`](Self::Plotkin): Plotkin (1970) Machine Intelligence 5:
///   153-163 — θ-subsumption and the subsumption order.
/// - [`DuquenneGuiguesTarski`](Self::DuquenneGuiguesTarski): Duquenne &
///   Guigues (1986) Math. Sci. Hum. 95: 5-18; Tarski (1956) Logic,
///   Semantics, Metamathematics — normalization and the canonical basis.
/// - [`PrakkenSartor`](Self::PrakkenSartor): Prakken & Sartor (1997) JANCL
///   7: 25-75 — conflict detection and conflict pairs/sets.
/// - [`Structural`](Self::Structural): abstract-category and pipeline
///   concepts that carry no single literature lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineageTradition {
    /// Robinson (1965) / Plotkin (1970) — implication / clause shape.
    RobinsonPlotkin,
    /// Reiter (1980) / Pollock (1987) — strict vs defeasible rules, defeaters.
    ReiterPollock,
    /// von Wright (1951) — deontic operators.
    VonWright,
    /// Plotkin (1970) — θ-subsumption and the subsumption order.
    Plotkin,
    /// Duquenne & Guigues (1986) / Tarski (1956) — normalization, canonical basis.
    DuquenneGuiguesTarski,
    /// Prakken & Sartor (1997) — conflict detection, conflict pairs/sets.
    PrakkenSartor,
    /// Structural / abstract-category and pipeline concepts (no single lineage).
    Structural,
}

/// Quality: which literature lineage introduces each concept?
#[derive(Debug, Clone)]
pub struct RuleAlgebraLineage;

impl Quality for RuleAlgebraLineage {
    type Individual = RuleAlgebraConcept;
    type Value = LineageTradition;

    fn get(&self, c: &RuleAlgebraConcept) -> Option<LineageTradition> {
        use LineageTradition as L;
        use RuleAlgebraConcept as C;
        Some(match c {
            C::Implication | C::Antecedent | C::Consequent => L::RobinsonPlotkin,
            C::StrictRule | C::DefeasibleRule | C::Defeater => L::ReiterPollock,
            C::Obligation | C::Permission | C::Prohibition | C::Assertoric => L::VonWright,
            C::Subsumption => L::Plotkin,
            C::Normalization | C::CanonicalBasis => L::DuquenneGuiguesTarski,
            C::ConflictDetection | C::ConflictPair | C::ConflictSet => L::PrakkenSartor,
            C::SubsumptionOrder => L::Plotkin,
            _ => L::Structural,
        })
    }
}

impl Ontology for RuleAlgebraOntology {
    type Cat = RuleAlgebraCategory;
    type Qual = RuleAlgebraLineage;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ImplicationIsRuleShape));
        axioms.push(Box::new(DeonticSquareOpposes));
        axioms.push(Box::new(StrictDefeasibleOppose));
        axioms
    }
}

#[cfg(test)]
#[path = "ontology_tests.rs"]
mod tests;
