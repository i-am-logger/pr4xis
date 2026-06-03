//! Proof standard ontology — concepts, is_a, stringency ordering, axioms.
//!
//! See `mod.rs` for the literature inventory and the modular-layer
//! rationale (statute-specific tiers live in application ontologies,
//! not here).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ProofStandard",
    source: "McCauliff (1982) Burdens of Proof: Degrees of Belief, Quanta of Evidence, or Constitutional Guarantees?, U. Pittsburgh L. Rev. 35:1293; Brilmayer (1990) Second-Order Evidence and Bayesian Logic, Boston U. L. Rev. 66:673; In re Winship, 397 U.S. 358 (1970); McCormick on Evidence (Strong et al., 8th ed. 2022) §339-343; Federal Rules of Evidence (2024); Guarino & Welty (2002) Evaluating Ontological Decisions with OntoClean, CACM 45(2):61-65; Stuckenschmidt, Parent, Spaccapietra (2009) Modular Ontologies, Springer LNCS 5445",

    concepts: [
        // Root
        ProofStandard,

        // The three classical leaves, in ascending stringency.
        Preponderance,         // civil default (> 50%)
        ClearAndConvincing,    // heightened civil (~ 75%)
        BeyondReasonableDoubt, // criminal (~ 95%)
    ],

    labels: {
        ProofStandard: ("en", "Proof standard",
            "McCauliff (1982): the fraction-of-evidence tier required to carry a burden on a given issue."),
        Preponderance: ("en", "Preponderance of the evidence",
            "Federal civil default (FRE; McCauliff 1982): more likely than not (> 50% probability)."),
        ClearAndConvincing: ("en", "Clear and convincing",
            "Heightened civil standard (~ 75% probability) — used in fraud, fitness for parenthood, and similar issues with elevated reliability requirements."),
        BeyondReasonableDoubt: ("en", "Beyond reasonable doubt",
            "Criminal-prosecution standard (~ 95% probability). In re Winship, 397 U.S. 358 (1970) establishes it as constitutionally required for every element of a criminal offense."),
    },

    is_a: [
        (Preponderance, ProofStandard),
        (ClearAndConvincing, ProofStandard),
        (BeyondReasonableDoubt, ProofStandard),
    ],
}

// ---------------------------------------------------------------------------
// Quality: StringencyOf — total ordering on the three leaves
// ---------------------------------------------------------------------------

/// Quality: integer stringency tier for ordering proof standards. Lower
/// numbers = easier to carry the burden. Calibrated against McCauliff
/// (1982)'s probability estimates: Preponderance < ClearAndConvincing
/// < BeyondReasonableDoubt.
///
/// Returns `None` for the abstract root `ProofStandard`.
#[derive(Debug, Clone)]
pub struct StringencyOf;

impl Quality for StringencyOf {
    type Individual = ProofStandardConcept;
    type Value = u8;

    fn get(&self, c: &ProofStandardConcept) -> Option<u8> {
        use ProofStandardConcept as P;
        match c {
            P::Preponderance => Some(1),
            P::ClearAndConvincing => Some(2),
            P::BeyondReasonableDoubt => Some(3),
            P::ProofStandard => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn leaves() -> [ProofStandardConcept; 3] {
    [
        ProofStandardConcept::Preponderance,
        ProofStandardConcept::ClearAndConvincing,
        ProofStandardConcept::BeyondReasonableDoubt,
    ]
}

pub fn is_leaf(c: ProofStandardConcept) -> bool {
    matches!(
        c,
        ProofStandardConcept::Preponderance
            | ProofStandardConcept::ClearAndConvincing
            | ProofStandardConcept::BeyondReasonableDoubt
    )
}

/// True iff `a` is at least as stringent as `b` (i.e., harder or equal
/// to carry). Total on leaves; both must be leaves or the answer is
/// `None`.
pub fn at_least_as_stringent(a: ProofStandardConcept, b: ProofStandardConcept) -> Option<bool> {
    let s = StringencyOf;
    match (s.get(&a), s.get(&b)) {
        (Some(x), Some(y)) => Some(x >= y),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for ProofStandardOntology {
    type Cat = ProofStandardCategory;
    type Qual = StringencyOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(StringencyIsTotalOnLeaves));
        axioms.push(Box::new(BeyondReasonableDoubtIsMostStringent));
        axioms
    }
}

/// Axiom: the three classical proof-standard leaves exhaust the
/// reference-layer partition. Statute-specific tiers (e.g.,
/// "contributing factor") are application-layer extensions adjoining
/// into this reference layer via the SourceTaxonomy's
/// `Statute ⊣ LegalLexicon` edge — they don't appear as leaves here.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let count = ProofStandardConcept::variants()
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
        "the reference-layer proof-standard partition has exactly three leaves",
        "McCauliff (1982) Burdens of Proof, U. Pittsburgh L. Rev. 35:1293; In re Winship, 397 U.S. 358 (1970)"
    );
}

pr4xis::register_axiom!(
    PartitionCompleteness,
    "McCauliff (1982); In re Winship (1970)"
);

/// Axiom: the `StringencyOf` quality is a total ordering on the three
/// leaves — every leaf has a distinct stringency tier, and the tiers
/// admit a strict linear order.
pub struct StringencyIsTotalOnLeaves;

impl Axiom for StringencyIsTotalOnLeaves {
    fn verify(&self) -> Verdict {
        let s = StringencyOf;
        let mut tiers: Vec<u8> = leaves().iter().map(|c| s.get(c).unwrap_or(0)).collect();
        tiers.sort();
        tiers.dedup();
        if tiers.len() == 3 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StringencyIsTotalOnLeaves",
        "StringencyOf assigns three distinct tiers to the three leaves",
        "McCauliff (1982); Brilmayer (1990) Bayesian Logic, BU L. Rev. 66:673"
    );
}

pr4xis::register_axiom!(
    StringencyIsTotalOnLeaves,
    "McCauliff (1982); Brilmayer (1990) Bayesian Logic, BU L. Rev. 66:673"
);

/// Axiom: BeyondReasonableDoubt is the *most* stringent standard. In re
/// Winship, 397 U.S. 358 (1970) establishes it as constitutionally
/// required for every element of a criminal offense.
pub struct BeyondReasonableDoubtIsMostStringent;

impl Axiom for BeyondReasonableDoubtIsMostStringent {
    fn verify(&self) -> Verdict {
        let brd_tier = StringencyOf.get(&ProofStandardConcept::BeyondReasonableDoubt);
        let others = [
            ProofStandardConcept::Preponderance,
            ProofStandardConcept::ClearAndConvincing,
        ];
        if let Some(brd) = brd_tier {
            for other in others {
                if let Some(o) = StringencyOf.get(&other)
                    && brd <= o
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BeyondReasonableDoubtIsMostStringent",
        "BeyondReasonableDoubt has the highest stringency tier of the three reference standards",
        "In re Winship, 397 U.S. 358 (1970)"
    );
}

pr4xis::register_axiom!(
    BeyondReasonableDoubtIsMostStringent,
    "In re Winship, 397 U.S. 358 (1970)"
);
