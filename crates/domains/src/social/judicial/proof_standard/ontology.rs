//! Proof standard ontology — concepts, is_a, stringency ordering, axioms.
//!
//! See `mod.rs` for the literature inventory.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ProofStandard",
    source: "McCauliff (1982) Burdens of Proof: Degrees of Belief, Quanta of Evidence, or Constitutional Guarantees?, U. Pittsburgh L. Rev. 35:1293; Brilmayer (1990) Second-Order Evidence and Bayesian Logic, Boston U. L. Rev. 66:673; Lawson v. FMR LLC, 571 U.S. 429 (2014); Murray v. UBS Securities, LLC, 601 U.S. 23 (2024); 18 U.S.C. § 1514A(b)(2)(C); AIR21 — 49 U.S.C. § 42121(b)(2)(B)",

    concepts: [
        // Root
        ProofStandard,

        // Leaves, in ascending stringency
        ContributingFactor,    // AIR21 / SOX 1514A plaintiff's prima facie
        Preponderance,         // civil default (> 50%)
        ClearAndConvincing,    // heightened civil (~ 75%) — also SOX defendant's rebuttal
        BeyondReasonableDoubt, // criminal (~ 95%)
    ],

    labels: {
        ProofStandard: ("en", "Proof standard",
            "McCauliff (1982): the fraction-of-evidence tier required to carry a burden on a given issue."),
        ContributingFactor: ("en", "Contributing factor",
            "AIR21 § 42121(b)(2)(B), incorporated by 18 U.S.C. § 1514A(b)(2)(C): plaintiff's prima facie burden in SOX whistleblower actions — protected activity need only be a contributing factor in the adverse action."),
        Preponderance: ("en", "Preponderance of the evidence",
            "Federal civil default (Federal Rules of Evidence; McCauliff 1982): more likely than not (> 50% probability)."),
        ClearAndConvincing: ("en", "Clear and convincing",
            "Heightened civil standard (~ 75% probability). Also the defendant's rebuttal standard in SOX whistleblower actions under AIR21 § 42121(b)(2)(B)(iv)."),
        BeyondReasonableDoubt: ("en", "Beyond reasonable doubt",
            "Criminal-prosecution standard (~ 95% probability). In re Winship, 397 U.S. 358 (1970)."),
    },

    is_a: [
        (ContributingFactor, ProofStandard),
        (Preponderance, ProofStandard),
        (ClearAndConvincing, ProofStandard),
        (BeyondReasonableDoubt, ProofStandard),
    ],
}

// ---------------------------------------------------------------------------
// Quality: StringencyOf — total ordering on the four leaves
// ---------------------------------------------------------------------------

/// Quality: integer stringency tier for ordering proof standards. Lower
/// numbers = easier to carry the burden. Calibrated against McCauliff
/// (1982)'s probability estimates: ContributingFactor < Preponderance
/// < ClearAndConvincing < BeyondReasonableDoubt.
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
            P::ContributingFactor => Some(1),
            P::Preponderance => Some(2),
            P::ClearAndConvincing => Some(3),
            P::BeyondReasonableDoubt => Some(4),
            P::ProofStandard => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn leaves() -> [ProofStandardConcept; 4] {
    [
        ProofStandardConcept::ContributingFactor,
        ProofStandardConcept::Preponderance,
        ProofStandardConcept::ClearAndConvincing,
        ProofStandardConcept::BeyondReasonableDoubt,
    ]
}

pub fn is_leaf(c: ProofStandardConcept) -> bool {
    matches!(
        c,
        ProofStandardConcept::ContributingFactor
            | ProofStandardConcept::Preponderance
            | ProofStandardConcept::ClearAndConvincing
            | ProofStandardConcept::BeyondReasonableDoubt
    )
}

/// True iff `a` is at least as stringent as `b` (i.e., harder or equal to
/// carry). Total on leaves; both must be leaves or the answer is `None`.
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
        axioms.push(Box::new(ContributingFactorIsLeastStringent));
        axioms.push(Box::new(BeyondReasonableDoubtIsMostStringent));
        axioms.push(Box::new(SoxAsymmetry));
        axioms
    }
}

/// Axiom: the four proof-standard leaves exhaust the partition for U.S.
/// civil + SOX whistleblower + criminal litigation. (Other jurisdictions
/// may have additional standards — e.g. "intermediate scrutiny" — but
/// those are addressed by extending the partition, not by adding
/// untyped tiers.)
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let count = ProofStandardConcept::variants()
            .into_iter()
            .filter(|c| is_leaf(*c))
            .count();
        if count == 4 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PartitionCompleteness",
        "the proof-standard partition has exactly four leaves",
        "McCauliff (1982) Burdens of Proof, U. Pittsburgh L. Rev. 35:1293"
    );
}

pr4xis::register_axiom!(
    PartitionCompleteness,
    "McCauliff (1982) Burdens of Proof, U. Pittsburgh L. Rev. 35:1293"
);

/// Axiom: the `StringencyOf` quality is a total ordering on the four
/// leaves — every leaf has a distinct stringency tier, and the tiers
/// admit a strict linear order.
pub struct StringencyIsTotalOnLeaves;

impl Axiom for StringencyIsTotalOnLeaves {
    fn verify(&self) -> Verdict {
        let s = StringencyOf;
        let mut tiers: Vec<u8> = leaves().iter().map(|c| s.get(c).unwrap_or(0)).collect();
        tiers.sort();
        tiers.dedup();
        if tiers.len() == 4 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StringencyIsTotalOnLeaves",
        "StringencyOf assigns four distinct tiers to the four leaves",
        "McCauliff (1982); Brilmayer (1990) Bayesian Logic, BU L. Rev. 66:673"
    );
}

pr4xis::register_axiom!(
    StringencyIsTotalOnLeaves,
    "McCauliff (1982); Brilmayer (1990) Bayesian Logic, BU L. Rev. 66:673"
);

/// Axiom: ContributingFactor is the *least* stringent standard. The
/// AIR21 / SOX 1514A framework deliberately tilts the prima-facie
/// burden in favor of the whistleblower-plaintiff.
pub struct ContributingFactorIsLeastStringent;

impl Axiom for ContributingFactorIsLeastStringent {
    fn verify(&self) -> Verdict {
        let cf_tier = StringencyOf.get(&ProofStandardConcept::ContributingFactor);
        let others = [
            ProofStandardConcept::Preponderance,
            ProofStandardConcept::ClearAndConvincing,
            ProofStandardConcept::BeyondReasonableDoubt,
        ];
        if let Some(cf) = cf_tier {
            for other in others {
                if let Some(o) = StringencyOf.get(&other)
                    && cf >= o
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
        "ContributingFactorIsLeastStringent",
        "ContributingFactor has the lowest stringency tier of all four standards",
        "AIR21 § 42121(b)(2)(B); Lawson v. FMR LLC, 571 U.S. 429 (2014); Murray v. UBS, 601 U.S. 23 (2024)"
    );
}

pr4xis::register_axiom!(
    ContributingFactorIsLeastStringent,
    "AIR21 § 42121(b)(2)(B); Lawson v. FMR LLC, 571 U.S. 429 (2014)"
);

/// Axiom: BeyondReasonableDoubt is the *most* stringent standard. In re
/// Winship, 397 U.S. 358 (1970) establishes it as constitutionally
/// required for every element of a criminal offense.
pub struct BeyondReasonableDoubtIsMostStringent;

impl Axiom for BeyondReasonableDoubtIsMostStringent {
    fn verify(&self) -> Verdict {
        let brd_tier = StringencyOf.get(&ProofStandardConcept::BeyondReasonableDoubt);
        let others = [
            ProofStandardConcept::ContributingFactor,
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
        "BeyondReasonableDoubt has the highest stringency tier of all four standards",
        "In re Winship, 397 U.S. 358 (1970)"
    );
}

pr4xis::register_axiom!(
    BeyondReasonableDoubtIsMostStringent,
    "In re Winship, 397 U.S. 358 (1970)"
);

/// Axiom: SOX 1514A's burden-shifting framework imposes asymmetric
/// standards on plaintiff and defendant. Plaintiff carries
/// `ContributingFactor`; defendant rebuts at `ClearAndConvincing`.
/// Stated as a stringency invariant: the defendant's rebuttal tier
/// strictly exceeds the plaintiff's prima-facie tier.
///
/// AIR21 § 42121(b)(2)(B)(iv): "Relief may not be ordered ... if the
/// employer demonstrates by clear and convincing evidence that the
/// employer would have taken the same unfavorable personnel action in
/// the absence of [protected] behavior."
pub struct SoxAsymmetry;

impl Axiom for SoxAsymmetry {
    fn verify(&self) -> Verdict {
        let plaintiff = StringencyOf.get(&ProofStandardConcept::ContributingFactor);
        let defendant = StringencyOf.get(&ProofStandardConcept::ClearAndConvincing);
        match (plaintiff, defendant) {
            (Some(p), Some(d)) if d > p => Ok(Box::new(SimpleProof::new(self.meta()))),
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "SoxAsymmetry",
        "SOX 1514A defendant's clear-and-convincing rebuttal exceeds plaintiff's contributing-factor prima facie",
        "AIR21 § 42121(b)(2)(B)(iv); 18 U.S.C. § 1514A(b)(2)(C); Murray v. UBS, 601 U.S. 23 (2024)"
    );
}

pr4xis::register_axiom!(
    SoxAsymmetry,
    "AIR21 § 42121(b)(2)(B)(iv); 18 U.S.C. § 1514A(b)(2)(C)"
);
