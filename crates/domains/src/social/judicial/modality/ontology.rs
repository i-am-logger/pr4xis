//! Obligation modality ontology — concepts, is_a hierarchy, axioms.
//!
//! See `mod.rs` for the literature inventory.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ObligationModality",
    source: "von Wright (1951) Deontic Logic, Mind 60(237):1-15; Halliday (1985) An Introduction to Functional Grammar, Edward Arnold ch. 10; Sergot, Sadri, Kowalski, Kriwaczek, Hammond, Cory (1986) The British Nationality Act as a logic program, CACM 29(5):370-386; Hohfeld (1913) Some Fundamental Legal Conceptions, Yale L.J. 23(1):16-59",

    concepts: [
        // Root
        ObligationModality,

        // Leaves — partition the deontic-primitive space
        Mandatory,      // O p   ("shall", "must")  — Hohfeld's duty
        Prohibitive,    // F p   ("shall not")      — Hohfeld's disability
        Discretionary,  // P p   ("may")            — Hohfeld's privilege
    ],

    labels: {
        ObligationModality: ("en", "Obligation modality",
            "von Wright (1951): the deontic mode (O / P / F) in which a legal rule binds the actor."),
        Mandatory: ("en", "Mandatory",
            "von Wright (1951) O operator: an obligation to perform; surface markers \"shall\" / \"must\" (Halliday 1985 ch. 10)."),
        Prohibitive: ("en", "Prohibitive",
            "von Wright (1951) F operator (F p ≡ O ¬p): a duty to refrain; surface markers \"shall not\" / \"must not\" / \"may not\"."),
        Discretionary: ("en", "Discretionary",
            "von Wright (1951) P operator (P p ≡ ¬O ¬p): a permission without compulsion; surface marker \"may\". Hohfeld (1913): privilege."),
    },

    is_a: [
        (Mandatory, ObligationModality),
        (Prohibitive, ObligationModality),
        (Discretionary, ObligationModality),
    ],

    opposes: [
        // von Wright (1951): O p and F p are *contraries* under classical
        // deontic logic — the same act cannot be simultaneously required
        // and forbidden under the same conditions. Stated symmetrically.
        (Mandatory, Prohibitive),
        (Prohibitive, Mandatory),

        // P p and F p are *contradictories* — exactly one holds at a
        // time. Same surface treatment in this ontology (oppose).
        (Discretionary, Prohibitive),
        (Prohibitive, Discretionary),
    ],
}

// ---------------------------------------------------------------------------
// Quality: PermitsAction — whether the modality leaves the actor free to
// perform p (Mandatory: must, so permitted; Discretionary: permitted but
// not required; Prohibitive: not permitted).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PermitsAction;

impl Quality for PermitsAction {
    type Individual = ObligationModalityConcept;
    type Value = bool;

    fn get(&self, c: &ObligationModalityConcept) -> Option<bool> {
        use ObligationModalityConcept as M;
        match c {
            M::Mandatory | M::Discretionary => Some(true),
            M::Prohibitive => Some(false),
            M::ObligationModality => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Quality: CompelsAction — whether the modality requires the actor to
// perform p (Mandatory: yes; Discretionary, Prohibitive: no — though for
// different reasons).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CompelsAction;

impl Quality for CompelsAction {
    type Individual = ObligationModalityConcept;
    type Value = bool;

    fn get(&self, c: &ObligationModalityConcept) -> Option<bool> {
        use ObligationModalityConcept as M;
        match c {
            M::Mandatory => Some(true),
            M::Discretionary | M::Prohibitive => Some(false),
            M::ObligationModality => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn leaves() -> [ObligationModalityConcept; 3] {
    [
        ObligationModalityConcept::Mandatory,
        ObligationModalityConcept::Prohibitive,
        ObligationModalityConcept::Discretionary,
    ]
}

pub fn is_leaf(c: ObligationModalityConcept) -> bool {
    matches!(
        c,
        ObligationModalityConcept::Mandatory
            | ObligationModalityConcept::Prohibitive
            | ObligationModalityConcept::Discretionary
    )
}

/// Recognize the modality from a surface modal marker. Halliday (1985)
/// §10 grounds the small closed set of English deontic markers.
/// Returns `None` for tokens that aren't deontic modals in legal usage.
///
/// The matcher is case-insensitive and matches whole words. It does not
/// attempt to parse compound forms like "shall not" — callers pre-split
/// such forms or pass each token.
pub fn classify_modal(word: &str) -> Option<ObligationModalityConcept> {
    use ObligationModalityConcept as M;
    let lower = word.to_lowercase();
    match lower.as_str() {
        "shall" | "must" | "required" | "requires" => Some(M::Mandatory),
        // Prohibitive surface forms are typically two-word: "shall not",
        // "must not", "may not". `classify_modal_pair` handles those.
        // A bare "prohibited" / "forbidden" surfaces as Prohibitive too.
        "prohibited" | "forbidden" => Some(M::Prohibitive),
        "may" | "permitted" | "entitled" | "discretionary" => Some(M::Discretionary),
        _ => None,
    }
}

/// Recognize a compound modal-pair ("shall not", "must not", "may not")
/// as Prohibitive. Returns `None` for non-prohibitive pairs.
pub fn classify_modal_pair(first: &str, second: &str) -> Option<ObligationModalityConcept> {
    let f = first.to_lowercase();
    let s = second.to_lowercase();
    match (f.as_str(), s.as_str()) {
        ("shall", "not") | ("must", "not") | ("may", "not") => {
            Some(ObligationModalityConcept::Prohibitive)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for ObligationModalityOntology {
    type Cat = ObligationModalityCategory;
    type Qual = PermitsAction;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(MandatoryAndProhibitiveAreContraries));
        axioms.push(Box::new(DiscretionaryAndProhibitiveAreContradictories));
        axioms.push(Box::new(MandatoryImpliesPermitted));
        axioms
    }
}

/// Axiom: the three deontic primitives (Mandatory/Prohibitive/Discretionary)
/// exhaust the modality partition — there is no fourth deontic kind.
///
/// von Wright (1951) introduced O, F, P as a complete primitive set for
/// classical deontic logic; later refinements (Ohad-Pang, Carmo & Jones,
/// etc.) add nuance but do not enlarge the primitive partition.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let count = ObligationModalityConcept::variants()
            .into_iter()
            .filter(|c| is_leaf(*c))
            .count();
        if count == 3 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PartitionCompleteness",
        "the deontic-modality partition has exactly three leaves: Mandatory, Prohibitive, Discretionary",
        "von Wright (1951) Deontic Logic, Mind 60(237):1-15"
    );
}

pr4xis::register_axiom!(
    PartitionCompleteness,
    "von Wright (1951) Deontic Logic, Mind 60(237):1-15"
);

/// Axiom: Mandatory and Prohibitive are *contraries* — the same provision
/// cannot simultaneously compel and forbid the same act. Stated as a
/// symmetric opposition edge in the ontology.
///
/// von Wright (1951) §3: O p and O ¬p (= F p) cannot both hold; that
/// would make the deontic system inconsistent.
pub struct MandatoryAndProhibitiveAreContraries;

impl Axiom for MandatoryAndProhibitiveAreContraries {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = ObligationModalityCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == ObligationModalityRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        let has_m_p = opp.contains(&(
            ObligationModalityConcept::Mandatory,
            ObligationModalityConcept::Prohibitive,
        ));
        let has_p_m = opp.contains(&(
            ObligationModalityConcept::Prohibitive,
            ObligationModalityConcept::Mandatory,
        ));
        if has_m_p && has_p_m {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MandatoryAndProhibitiveAreContraries",
        "Mandatory and Prohibitive oppose each other symmetrically (von Wright 1951 §3)",
        "von Wright (1951) Deontic Logic, Mind 60(237):1-15 §3"
    );
}

pr4xis::register_axiom!(
    MandatoryAndProhibitiveAreContraries,
    "von Wright (1951) Deontic Logic, Mind 60(237):1-15 §3"
);

/// Axiom: Discretionary and Prohibitive are *contradictories* — exactly
/// one holds for any given act-under-conditions. Permission and
/// prohibition exhaust the deontic space when Mandatory is absent.
///
/// von Wright (1951) §2: P p ↔ ¬F p; the law cannot simultaneously
/// permit and forbid the same act.
pub struct DiscretionaryAndProhibitiveAreContradictories;

impl Axiom for DiscretionaryAndProhibitiveAreContradictories {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = ObligationModalityCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == ObligationModalityRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        let has_d_p = opp.contains(&(
            ObligationModalityConcept::Discretionary,
            ObligationModalityConcept::Prohibitive,
        ));
        let has_p_d = opp.contains(&(
            ObligationModalityConcept::Prohibitive,
            ObligationModalityConcept::Discretionary,
        ));
        if has_d_p && has_p_d {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DiscretionaryAndProhibitiveAreContradictories",
        "Discretionary and Prohibitive oppose each other symmetrically (von Wright 1951 §2)",
        "von Wright (1951) Deontic Logic, Mind 60(237):1-15 §2"
    );
}

pr4xis::register_axiom!(
    DiscretionaryAndProhibitiveAreContradictories,
    "von Wright (1951) Deontic Logic, Mind 60(237):1-15 §2"
);

/// Axiom: Mandatory entails permitted — if an act is required, it is
/// (a fortiori) permitted. This is the deontic-logic schema O p → P p,
/// sometimes called the "ought implies can" entailment in its weak
/// version.
///
/// Stated as a Quality cross-check: every concept where `CompelsAction`
/// returns `Some(true)` must also have `PermitsAction` returning
/// `Some(true)`.
///
/// von Wright (1951) §4 establishes the entailment from O to P as
/// fundamental to consistent deontic systems.
pub struct MandatoryImpliesPermitted;

impl Axiom for MandatoryImpliesPermitted {
    fn verify(&self) -> Verdict {
        let permits = PermitsAction;
        let compels = CompelsAction;
        for c in ObligationModalityConcept::variants() {
            if let Some(true) = compels.get(&c)
                && permits.get(&c) != Some(true)
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MandatoryImpliesPermitted",
        "if a modality compels, it also permits — O p ⊢ P p",
        "von Wright (1951) Deontic Logic, Mind 60(237):1-15 §4"
    );
}

pr4xis::register_axiom!(
    MandatoryImpliesPermitted,
    "von Wright (1951) Deontic Logic, Mind 60(237):1-15 §4"
);
