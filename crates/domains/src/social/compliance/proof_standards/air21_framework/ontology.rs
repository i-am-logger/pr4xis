//! AIR21 § 42121(b)(2)(B) contributing-factor causation standard —
//! application-layer ontology adjoining into the reference proof-
//! standard tier. Statute-only sourcing: case-law confirmations live
//! at per-case homes (see `mod.rs`).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::social::judicial::proof_standard::ontology::{
    ProofStandardConcept, StringencyOf as ReferenceStringency, StringencyTier,
};

/// Extends the reference layer's [`StringencyTier`] ordering one step
/// below its minimum (`Preponderance`), so AIR21's below-preponderance
/// "contributing factor" causation standard is a *typed* ordinal — not
/// a raw integer compared against the reference layer's raw integer on
/// a shared, undocumented numeric scale.
///
/// `BelowPreponderance` is declared as the first (unit) variant, ahead
/// of the `Reference(StringencyTier)` wrapping variant. Rust's derived
/// `Ord` for an enum compares the outer variant index before recursing
/// into payload fields, so `BelowPreponderance` sorts strictly below
/// every `Reference(_)` value regardless of which `StringencyTier` is
/// wrapped — exactly the "below the entire reference partition"
/// semantics AIR21 needs.
///
/// Source: 49 U.S.C. § 42121(b)(2)(B)(i) — statutory text expressly
/// frames the showing as "a contributing factor" rather than "more
/// likely than not," establishing the lower bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Air21StringencyTier {
    BelowPreponderance,
    Reference(StringencyTier),
}

/// The minimum stringency tier in the reference layer, wrapped into
/// [`Air21StringencyTier`]. Captured as a constant here so the
/// cross-layer asymmetry axiom is verifiable without crossing ontology
/// boundaries at axiom time.
///
/// Source: `social::judicial::proof_standard::ontology::StringencyOf`
/// assigns `StringencyTier::Preponderance` as the lowest reference-layer
/// tier.
const REFERENCE_MIN_TIER: Air21StringencyTier =
    Air21StringencyTier::Reference(StringencyTier::Preponderance);

pr4xis::ontology! {
    name: "Air21ProofStandard",
    source: "49 U.S.C. § 42121(b)(2)(B)(i)-(iv); 18 U.S.C. § 1514A(b)(2)(C); Stuckenschmidt, Parent, Spaccapietra (2009) Modular Ontologies, Springer LNCS 5445; Guarino & Welty (2002) Evaluating Ontological Decisions with OntoClean, CACM 45(2):61-65",

    concepts: [
        // Root
        Air21ProofStandard,

        // The single application-layer leaf — strictly below the
        // reference layer's minimum tier (Preponderance).
        ContributingFactor,
    ],

    labels: {
        Air21ProofStandard: ("en", "AIR21 proof standard",
            "49 U.S.C. § 42121(b)(2)(B): the four-clause burden-shifting framework for federal whistleblower retaliation, incorporated by reference into SOX § 1514A, FRSA § 20109, CFPA § 1057, NDAA § 4712, and other federal whistleblower regimes. Application-layer extension of the reference proof-standard partition."),
        ContributingFactor: ("en", "Contributing factor",
            "49 U.S.C. § 42121(b)(2)(B)(i): the complainant's causation burden in a federal whistleblower-retaliation claim. Protected activity need only have been *a* contributing factor in the adverse action; the bar is statutorily lower than preponderance."),
    },

    is_a: [
        (ContributingFactor, Air21ProofStandard),
    ],
}

// ---------------------------------------------------------------------------
// Quality: Air21StringencyOf — extends the reference stringency ordering
// ---------------------------------------------------------------------------

/// Quality: typed stringency tier for AIR21 contributing-factor
/// causation. ContributingFactor's tier is [`Air21StringencyTier::BelowPreponderance`]
/// — strictly below the reference layer's minimum (`Preponderance`).
/// Returns `None` for the abstract root.
///
/// Source: 49 U.S.C. § 42121(b)(2)(B)(i) — statutory text expressly
/// frames the showing as "a contributing factor" rather than "more
/// likely than not," establishing the lower bar.
#[derive(Debug, Clone)]
pub struct Air21StringencyOf;

impl Quality for Air21StringencyOf {
    type Individual = Air21ProofStandardConcept;
    type Value = Air21StringencyTier;

    fn get(&self, c: &Air21ProofStandardConcept) -> Option<Air21StringencyTier> {
        match c {
            Air21ProofStandardConcept::ContributingFactor => {
                Some(Air21StringencyTier::BelowPreponderance)
            }
            Air21ProofStandardConcept::Air21ProofStandard => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The single application-layer leaf.
pub fn leaves() -> [Air21ProofStandardConcept; 1] {
    [Air21ProofStandardConcept::ContributingFactor]
}

/// True iff `c` is a leaf of this application-layer ontology.
pub fn is_leaf(c: Air21ProofStandardConcept) -> bool {
    matches!(c, Air21ProofStandardConcept::ContributingFactor)
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for Air21ProofStandardOntology {
    type Cat = Air21ProofStandardCategory;
    type Qual = Air21StringencyOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(ContributingFactorBelowReferencePartition));
        axioms.push(Box::new(ReferenceMinTierCoherence));
        axioms
    }
}

/// Axiom: the AIR21-specific partition has exactly one leaf
/// (ContributingFactor). Additional AIR21-specific tiers would require
/// fresh primary-source attestation.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let count = Air21ProofStandardConcept::variants()
            .into_iter()
            .filter(|c| is_leaf(*c))
            .count();
        if count == 1 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PartitionCompleteness",
        "the AIR21 application-layer partition has exactly one leaf (ContributingFactor)",
        "49 U.S.C. § 42121(b)(2)(B); 18 U.S.C. § 1514A(b)(2)(C)"
    );
}

pr4xis::register_axiom!(
    PartitionCompleteness,
    "49 U.S.C. § 42121(b)(2)(B); 18 U.S.C. § 1514A(b)(2)(C)"
);

/// Axiom: ContributingFactor sits strictly *below* the reference
/// layer's minimum stringency tier — the AIR21 causation burden is
/// statutorily lower than the classical civil-default Preponderance
/// standard. § 42121(b)(2)(B)(i) frames the requirement as "a
/// contributing factor," a phrasing the Federal Circuit and ARB have
/// consistently read to fall below preponderance.
///
/// Verifies by comparing `Air21StringencyOf(ContributingFactor)` against
/// the internalized `REFERENCE_MIN_TIER` constant.
pub struct ContributingFactorBelowReferencePartition;

impl Axiom for ContributingFactorBelowReferencePartition {
    fn verify(&self) -> Verdict {
        let cf_tier = Air21StringencyOf.get(&Air21ProofStandardConcept::ContributingFactor);
        match cf_tier {
            Some(t) if t < REFERENCE_MIN_TIER => Ok(Box::new(SimpleProof::new(self.meta()))),
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "ContributingFactorBelowReferencePartition",
        "ContributingFactor's stringency tier is strictly below the reference layer's minimum (Preponderance)",
        "49 U.S.C. § 42121(b)(2)(B)(i)"
    );
}

pr4xis::register_axiom!(
    ContributingFactorBelowReferencePartition,
    "49 U.S.C. § 42121(b)(2)(B)(i)"
);

/// Axiom: the constant we internalize for the reference layer's
/// minimum tier matches what the reference-layer `StringencyOf`
/// quality assigns to `Preponderance`. Guards against silent drift if
/// the reference layer renumbers its tiers without updating
/// `REFERENCE_MIN_TIER` here.
///
/// This is *not* a cross-ontology axiom in the categorical sense
/// (which would require an Adjunction); it's a coherence check that
/// the two ontologies' value spaces agree at the seam point.
pub struct ReferenceMinTierCoherence;

impl Axiom for ReferenceMinTierCoherence {
    fn verify(&self) -> Verdict {
        let reference_preponderance = ReferenceStringency.get(&ProofStandardConcept::Preponderance);
        match reference_preponderance {
            Some(t) if Air21StringencyTier::Reference(t) == REFERENCE_MIN_TIER => {
                Ok(Box::new(SimpleProof::new(self.meta())))
            }
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "ReferenceMinTierCoherence",
        "REFERENCE_MIN_TIER matches the reference layer's StringencyOf(Preponderance) — the seam between AIR21 application layer and the general proof-standard partition holds",
        "social::judicial::proof_standard::ontology::StringencyOf"
    );
}

pr4xis::register_axiom!(
    ReferenceMinTierCoherence,
    "Coherence with reference proof_standard::StringencyOf"
);
