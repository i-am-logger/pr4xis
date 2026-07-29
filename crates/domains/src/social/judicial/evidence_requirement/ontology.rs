//! Evidence requirement ontology — concepts, is_a, axioms.
//!
//! See `mod.rs` for the literature inventory.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "RequirementLevel",
    source: "RFC 2119 (Bradner 1997) Key words for use in RFCs to Indicate Requirement Levels, IETF; BCP 14 (Leiba 2017) Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words; McCormick on Evidence (Strong et al., 8th ed. 2022) §337; Federal Rules of Evidence, Rule 104",

    concepts: [
        // Root
        RequirementLevel,

        // Leaves: RFC 2119 BCP 14 partition
        Required,     // MUST / SHALL
        Recommended,  // SHOULD
        Optional,     // MAY
    ],

    labels: {
        RequirementLevel: ("en", "Requirement level",
            "RFC 2119: the strict-need tier attached to a piece of evidence or specification requirement."),
        Required: ("en", "Required",
            "RFC 2119 MUST/SHALL: absolute requirement — failure to provide this evidence defeats the pleading on this issue."),
        Recommended: ("en", "Recommended",
            "RFC 2119 SHOULD: there may exist valid reasons in particular circumstances to ignore, but the full implications must be understood."),
        Optional: ("en", "Optional",
            "RFC 2119 MAY: the requirement is truly optional; absence has no consequence."),
    },

    is_a: [
        (Required, RequirementLevel),
        (Recommended, RequirementLevel),
        (Optional, RequirementLevel),
    ],

    opposes: [
        // Required and Optional are duals on the strict-need axis.
        (Required, Optional),
        (Optional, Required),
    ],
}

// ---------------------------------------------------------------------------
// Quality: Strictness — total ordering (Optional < Recommended < Required)
// ---------------------------------------------------------------------------

/// Typed ordinal ranking of RFC 2119 requirement strictness. Declared
/// in **ascending** strictness order — `Optional` (least strict,
/// formerly tier 1) first, `Required` (most strict, formerly tier 3)
/// last — so Rust's derived `Ord` for a fieldless enum
/// (earlier-declared variant compares as *lesser*) directly mirrors the
/// original 1..3 numeric tier ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrictnessTier {
    Optional,
    Recommended,
    Required,
}

/// Quality: typed strictness tier for ordering requirement levels.
///
/// Returns `None` for the abstract root.
#[derive(Debug, Clone)]
pub struct Strictness;

impl Quality for Strictness {
    type Individual = RequirementLevelConcept;
    type Value = StrictnessTier;

    fn get(&self, c: &RequirementLevelConcept) -> Option<StrictnessTier> {
        use RequirementLevelConcept as R;
        match c {
            R::Optional => Some(StrictnessTier::Optional),
            R::Recommended => Some(StrictnessTier::Recommended),
            R::Required => Some(StrictnessTier::Required),
            R::RequirementLevel => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn leaves() -> [RequirementLevelConcept; 3] {
    [
        RequirementLevelConcept::Required,
        RequirementLevelConcept::Recommended,
        RequirementLevelConcept::Optional,
    ]
}

pub fn is_leaf(c: RequirementLevelConcept) -> bool {
    matches!(
        c,
        RequirementLevelConcept::Required
            | RequirementLevelConcept::Recommended
            | RequirementLevelConcept::Optional
    )
}

/// Parse an RFC 2119 keyword (case-insensitive) into a typed concept.
///
/// Accepts: "MUST" / "SHALL" / "REQUIRED" → Required;
/// "SHOULD" / "RECOMMENDED" → Recommended;
/// "MAY" / "OPTIONAL" → Optional.
/// Negative forms ("MUST NOT", "SHOULD NOT", "SHALL NOT") are not
/// classified as RequirementLevel — they belong to the
/// `ObligationModality::Prohibitive` concept (see
/// `social::judicial::modality`).
pub fn parse_rfc2119(keyword: &str) -> Option<RequirementLevelConcept> {
    use RequirementLevelConcept as R;
    let upper = keyword.to_uppercase();
    match upper.as_str() {
        "MUST" | "SHALL" | "REQUIRED" => Some(R::Required),
        "SHOULD" | "RECOMMENDED" => Some(R::Recommended),
        "MAY" | "OPTIONAL" => Some(R::Optional),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for RequirementLevelOntology {
    type Cat = RequirementLevelCategory;
    type Qual = Strictness;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(StrictnessIsTotalOrder));
        axioms.push(Box::new(RequiredAndOptionalAreDuals));
        axioms
    }
}

/// Axiom: RFC 2119's partition has exactly three levels (Required /
/// Recommended / Optional). Negative forms (MUST NOT, etc.) belong to
/// the deontic-modality ontology, not here.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let count = RequirementLevelConcept::variants()
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
        "RFC 2119 partition has exactly three levels: Required, Recommended, Optional",
        "RFC 2119 (Bradner 1997)"
    );
}

pr4xis::register_axiom!(PartitionCompleteness, "RFC 2119 (Bradner 1997)");

/// Axiom: `Strictness` is a strict total order on the three leaves
/// (Optional < Recommended < Required).
pub struct StrictnessIsTotalOrder;

impl Axiom for StrictnessIsTotalOrder {
    fn verify(&self) -> Verdict {
        let s = Strictness;
        let opt = s.get(&RequirementLevelConcept::Optional);
        let rec = s.get(&RequirementLevelConcept::Recommended);
        let req = s.get(&RequirementLevelConcept::Required);
        match (opt, rec, req) {
            (Some(o), Some(r), Some(q)) if o < r && r < q => {
                Ok(Box::new(SimpleProof::new(self.meta())))
            }
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "StrictnessIsTotalOrder",
        "Strictness gives a strict total order: Optional < Recommended < Required",
        "RFC 2119 (Bradner 1997); McCormick on Evidence §337"
    );
}

pr4xis::register_axiom!(
    StrictnessIsTotalOrder,
    "RFC 2119 (Bradner 1997); McCormick on Evidence §337"
);

/// Axiom: Required and Optional are opposites — they sit at the two
/// poles of the strict-need axis. The opposition edges declared in
/// the ontology enforce this symmetrically.
pub struct RequiredAndOptionalAreDuals;

impl Axiom for RequiredAndOptionalAreDuals {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = RequirementLevelCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == RequirementLevelRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        let has_r_o = opp.contains(&(
            RequirementLevelConcept::Required,
            RequirementLevelConcept::Optional,
        ));
        let has_o_r = opp.contains(&(
            RequirementLevelConcept::Optional,
            RequirementLevelConcept::Required,
        ));
        if has_r_o && has_o_r {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RequiredAndOptionalAreDuals",
        "Required and Optional oppose each other symmetrically (strict-need axis duality)",
        "RFC 2119 (Bradner 1997)"
    );
}

pr4xis::register_axiom!(RequiredAndOptionalAreDuals, "RFC 2119 (Bradner 1997)");
