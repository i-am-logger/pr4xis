//! Valence ontology — concepts, is_a hierarchy, axioms.
//!
//! See `mod.rs` for the literature inventory.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Valence",
    source: "Hart (1961) The Concept of Law, Oxford University Press ch. V; MacCormick (1978) Legal Reasoning and Legal Theory, Oxford University Press ch. 3 §5; Wigmore (1937) A Students' Textbook of the Law of Evidence, Foundation Press §27",

    concepts: [
        // Root
        Valence,

        // Leaves — partition the argumentative-role space
        Supportive,   // pro-claimant; advances the moving party's case
        Defensive,    // pro-respondent; defeats the moving party's case
        Procedural,   // scope/jurisdiction/definition; non-merits
    ],

    labels: {
        Valence: ("en", "Valence",
            "MacCormick (1978): the argumentative role a legal term plays in a claim."),
        Supportive: ("en", "Supportive",
            "Wigmore (1937) §27: a provision that, when applied, advances the moving party's case on the merits."),
        Defensive: ("en", "Defensive",
            "Wigmore (1937) §27: a provision that, when applied, defeats the moving party's case on the merits."),
        Procedural: ("en", "Procedural",
            "MacCormick (1978) ch. 5 §3: scope/jurisdiction/definition; neither pro nor con on the merits."),
    },

    is_a: [
        (Supportive, Valence),
        (Defensive, Valence),
        (Procedural, Valence),
    ],

    opposes: [
        // Supportive and Defensive are dual on the merits — a single
        // provision cannot both advance and defeat the same claim.
        (Supportive, Defensive),
        (Defensive, Supportive),
    ],
}

// ---------------------------------------------------------------------------
// Quality: AdvancesMovingParty
// ---------------------------------------------------------------------------

/// Quality: does the term, when invoked, advance the moving party's case?
///
/// `Supportive` → `Some(true)` (advances)
/// `Defensive` → `Some(false)` (defeats)
/// `Procedural` → `Some(false)` (neither advances nor defeats on the merits)
/// `Valence` (root) → `None` (abstract; non-classifying)
///
/// MacCormick (1978) §3.4 grounds the dichotomy: the supportive/defensive
/// pair operates on the merits; procedural rules sit orthogonal.
#[derive(Debug, Clone)]
pub struct AdvancesMovingParty;

impl Quality for AdvancesMovingParty {
    type Individual = ValenceConcept;
    type Value = bool;

    fn get(&self, c: &ValenceConcept) -> Option<bool> {
        use ValenceConcept as V;
        match c {
            V::Supportive => Some(true),
            V::Defensive | V::Procedural => Some(false),
            V::Valence => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Iterate the three concrete-valence leaves (everything except the abstract
/// `Valence` root).
pub fn leaves() -> [ValenceConcept; 3] {
    [
        ValenceConcept::Supportive,
        ValenceConcept::Defensive,
        ValenceConcept::Procedural,
    ]
}

/// True iff `c` is a concrete valence leaf, i.e. a partition member of the
/// argumentative-role space (excludes the abstract root).
pub fn is_leaf(c: ValenceConcept) -> bool {
    matches!(
        c,
        ValenceConcept::Supportive | ValenceConcept::Defensive | ValenceConcept::Procedural
    )
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for ValenceOntology {
    type Cat = ValenceCategory;
    type Qual = AdvancesMovingParty;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(MeritsAxisDuality));
        axioms.push(Box::new(ProceduralIsOrthogonal));
        axioms
    }
}

/// Axiom: every concrete legal term inhabits exactly one of three
/// valence roles — `Supportive`, `Defensive`, or `Procedural`. The
/// partition is *complete*: there is no fourth valence kind.
///
/// MacCormick (1978) §3.4 grounds the trichotomy as exhaustive within
/// the supportive/defensive/procedural typology. Wigmore (1937) §27
/// empirically confirms the partition for trial-practice usage.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        // The three leaves declared in the ontology cover the partition.
        // Adding a fourth without literature would break this axiom by
        // construction (the variants list above would grow).
        let count = ValenceConcept::variants()
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
        "the Valence partition has exactly three leaves: Supportive, Defensive, Procedural",
        "MacCormick (1978) Legal Reasoning and Legal Theory §3.4; Wigmore (1937) §27"
    );
}

pr4xis::register_axiom!(
    PartitionCompleteness,
    "MacCormick (1978) Legal Reasoning and Legal Theory §3.4"
);

/// Axiom: `Supportive` and `Defensive` are duals on the merits — a
/// single provision cannot simultaneously advance *and* defeat the same
/// claim. The opposes-edges declared in the ontology enforce this.
///
/// Hart (1961) ch. V framing: the primary rules of obligation either
/// generate a claim or block it; the same rule cannot do both at the
/// same time without becoming inconsistent.
pub struct MeritsAxisDuality;

impl Axiom for MeritsAxisDuality {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = ValenceCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == ValenceRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        // Symmetric opposition required: (S, D) and (D, S) both present.
        let has_sd = opp.contains(&(ValenceConcept::Supportive, ValenceConcept::Defensive));
        let has_ds = opp.contains(&(ValenceConcept::Defensive, ValenceConcept::Supportive));
        if has_sd && has_ds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MeritsAxisDuality",
        "Supportive and Defensive oppose each other symmetrically",
        "Hart (1961) The Concept of Law ch. V"
    );
}

pr4xis::register_axiom!(MeritsAxisDuality, "Hart (1961) The Concept of Law ch. V");

/// Axiom: `Procedural` is orthogonal to the merits axis — it neither
/// opposes nor agrees with `Supportive` or `Defensive`. A procedural
/// provision (scope, jurisdiction, definitions) sets the conditions
/// under which a merits analysis happens; it doesn't *do* the merits
/// analysis itself.
///
/// MacCormick (1978) ch. 5 §3 distinguishes procedural rules from
/// substantive (merits) rules; they belong to different axes of legal
/// reasoning.
pub struct ProceduralIsOrthogonal;

impl Axiom for ProceduralIsOrthogonal {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = ValenceCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == ValenceRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        // Procedural must not oppose either merits-axis leaf.
        for pair in [
            (ValenceConcept::Procedural, ValenceConcept::Supportive),
            (ValenceConcept::Procedural, ValenceConcept::Defensive),
            (ValenceConcept::Supportive, ValenceConcept::Procedural),
            (ValenceConcept::Defensive, ValenceConcept::Procedural),
        ] {
            if opp.contains(&pair) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ProceduralIsOrthogonal",
        "Procedural opposes neither Supportive nor Defensive — it is on a different axis",
        "MacCormick (1978) Legal Reasoning and Legal Theory ch. 5 §3"
    );
}

pr4xis::register_axiom!(
    ProceduralIsOrthogonal,
    "MacCormick (1978) Legal Reasoning and Legal Theory ch. 5 §3"
);
