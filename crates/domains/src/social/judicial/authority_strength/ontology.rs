//! AuthorityStrength ontology — concepts, is_a, binding-force ordering,
//! jurisdiction-scope quality, axioms.
//!
//! See `mod.rs` for the literature inventory and the dimension
//! rationale (vertical strength vs. horizontal jurisdiction).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::meta::identifier_format::Identifier;

pr4xis::ontology! {
    name: "AuthorityStrength",
    source: "Hart (1961) The Concept of Law, Oxford; Schauer (2009) Thinking Like a Lawyer, Harvard; Garner et al. (2016) The Law of Judicial Precedent, Thomson Reuters; Sartor (2005) Legal Reasoning, Springer (Treatise vol. 5); Eskridge, Frickey, Garrett, Brudney (latest ed.) Legislation and Statutory Interpretation; Marbury v. Madison, 5 U.S. 137 (1803); U.S. Const. Art. VI, cl. 2; Erie R.R. v. Tompkins, 304 U.S. 64 (1938); Chevron U.S.A. v. NRDC, 467 U.S. 837 (1984); Skidmore v. Swift & Co., 323 U.S. 134 (1944); Guarino & Welty (2002) Evaluating Ontological Decisions with OntoClean, CACM 45(2):61-65; Stuckenschmidt, Parent, Spaccapietra (2009) Modular Ontologies, Springer LNCS 5445",

    concepts: [
        // Root
        AuthorityStrength,

        // Binding/persuasive branches (Schauer 2009 Ch. 5; Garner 2016
        // §1.2). These are intermediate — they have no tier but
        // partition the leaves.
        BindingAuthority,
        PersuasiveAuthority,

        // Binding leaves, descending tier (tiers 9..5)
        ConstitutionalText,
        FederalStatute,
        SupremeCourtPrecedent,
        FederalRegulation,
        ControllingCircuitPrecedent,

        // Persuasive leaves, descending tier (tiers 4..1)
        AdministrativeReviewBoardDecision,
        SisterCircuitPrecedent,
        DistrictCourtPrecedent,
        SecondarySource,
    ],

    labels: {
        AuthorityStrength: ("en", "Authority strength",
            "Hart (1961); Schauer (2009): the meta-level property of a legal source describing how much weight a court must give it."),
        BindingAuthority: ("en", "Binding authority",
            "Schauer (2009) Ch. 5: an authority a court is required to follow when applicable and within jurisdiction."),
        PersuasiveAuthority: ("en", "Persuasive authority",
            "Schauer (2009) Ch. 5: an authority a court may consult and may follow but is not required to apply."),

        ConstitutionalText: ("en", "Constitutional text",
            "U.S. Const. Art. VI, cl. 2 (Supremacy Clause); Marbury v. Madison, 5 U.S. 137 (1803): the supreme law of the land, controlling over all conflicting federal or state law."),
        FederalStatute: ("en", "Federal statute",
            "U.S. Const. Art. I; Schauer (2009) Ch. 5: legislation enacted by Congress within Article I powers; binds federal courts subject to constitutional conformity."),
        SupremeCourtPrecedent: ("en", "Supreme Court precedent",
            "Garner (2016) §1.2; Schauer (2009) Ch. 3: holdings of the Supreme Court of the United States bind every lower federal court on the points decided."),
        FederalRegulation: ("en", "Federal regulation",
            "Chevron U.S.A. v. NRDC, 467 U.S. 837 (1984): agency regulations adopted under valid delegation bind courts when the statute is ambiguous and the regulation is reasonable."),
        ControllingCircuitPrecedent: ("en", "Controlling circuit precedent",
            "Garner (2016) §11.1: published precedent of a U.S. Court of Appeals binds all district courts within that circuit on the points decided."),

        AdministrativeReviewBoardDecision: ("en", "Administrative Review Board decision",
            "Skidmore v. Swift & Co., 323 U.S. 134 (1944): agency adjudications outside Chevron's domain receive persuasive weight calibrated to thoroughness, validity of reasoning, and consistency."),
        SisterCircuitPrecedent: ("en", "Sister-circuit precedent",
            "Garner (2016) §11.2: a published precedent from one U.S. Court of Appeals is persuasive — not binding — in the courts of every other circuit."),
        DistrictCourtPrecedent: ("en", "District-court precedent",
            "Garner (2016) §11.3: published opinions of U.S. district courts do not bind other district courts and bind only the issuing court horizontally; they retain persuasive weight."),
        SecondarySource: ("en", "Secondary source",
            "Garner (2016) §13.1; Bluebook (21st ed.) §15-19: treatises, law-review articles, Restatements, and similar non-primary sources. Persuasive only; weight calibrated to author authority, citation frequency, and reasoning."),
    },

    is_a: [
        // Binding branch
        (ConstitutionalText, BindingAuthority),
        (FederalStatute, BindingAuthority),
        (SupremeCourtPrecedent, BindingAuthority),
        (FederalRegulation, BindingAuthority),
        (ControllingCircuitPrecedent, BindingAuthority),

        // Persuasive branch
        (AdministrativeReviewBoardDecision, PersuasiveAuthority),
        (SisterCircuitPrecedent, PersuasiveAuthority),
        (DistrictCourtPrecedent, PersuasiveAuthority),
        (SecondarySource, PersuasiveAuthority),

        // Branches inherit from root
        (BindingAuthority, AuthorityStrength),
        (PersuasiveAuthority, AuthorityStrength),
    ],
}

// ---------------------------------------------------------------------------
// Quality: BindingForceOf — strict total ordering on leaves
// ---------------------------------------------------------------------------

/// Typed ordinal ranking of abstract binding force. Declared in
/// **ascending** binding-force order — `SecondarySource` (weakest,
/// formerly tier 1) first, `ConstitutionalText` (strongest, formerly
/// tier 9) last — so that Rust's derived `Ord` for a fieldless enum
/// (which orders variants by declaration order: earlier declared
/// compares as *lesser*) directly mirrors the original 1..9 numeric
/// tier ordering without inverting any comparison operator at call
/// sites.
///
/// Calibration sources, ascending:
/// - SecondarySource (was 1): Garner (2016) §13.1
/// - DistrictCourtPrecedent (was 2): Garner (2016) §11.3
/// - SisterCircuitPrecedent (was 3): Garner (2016) §11.2
/// - AdministrativeReviewBoardDecision (was 4): Skidmore v. Swift (1944)
/// - ControllingCircuitPrecedent (was 5): Garner (2016) §11.1
/// - FederalRegulation (was 6): Chevron U.S.A. v. NRDC (1984)
/// - SupremeCourtPrecedent (was 7): Garner (2016) §1.2
/// - FederalStatute (was 8): Schauer (2009) Ch. 5
/// - ConstitutionalText (was 9): Marbury v. Madison (1803); U.S. Const. Art. VI cl. 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingForceTier {
    SecondarySource,
    DistrictCourtPrecedent,
    SisterCircuitPrecedent,
    AdministrativeReviewBoardDecision,
    ControllingCircuitPrecedent,
    FederalRegulation,
    SupremeCourtPrecedent,
    FederalStatute,
    ConstitutionalText,
}

/// Quality: typed binding-force tier. Higher [`BindingForceTier`]
/// (per its ascending `Ord`) = greater abstract binding force.
/// Returns `None` on the abstract root and the two branch concepts
/// (which partition the leaves but have no tier of their own).
#[derive(Debug, Clone)]
pub struct BindingForceOf;

impl Quality for BindingForceOf {
    type Individual = AuthorityStrengthConcept;
    type Value = BindingForceTier;

    fn get(&self, c: &AuthorityStrengthConcept) -> Option<BindingForceTier> {
        use AuthorityStrengthConcept as A;
        use BindingForceTier as T;
        match c {
            A::ConstitutionalText => Some(T::ConstitutionalText),
            A::FederalStatute => Some(T::FederalStatute),
            A::SupremeCourtPrecedent => Some(T::SupremeCourtPrecedent),
            A::FederalRegulation => Some(T::FederalRegulation),
            A::ControllingCircuitPrecedent => Some(T::ControllingCircuitPrecedent),
            A::AdministrativeReviewBoardDecision => Some(T::AdministrativeReviewBoardDecision),
            A::SisterCircuitPrecedent => Some(T::SisterCircuitPrecedent),
            A::DistrictCourtPrecedent => Some(T::DistrictCourtPrecedent),
            A::SecondarySource => Some(T::SecondarySource),
            // Root and branches have no tier of their own.
            A::AuthorityStrength | A::BindingAuthority | A::PersuasiveAuthority => None,
        }
    }
}

/// Lowest binding-tier (inclusive). Tier values strictly less than
/// this are persuasive-only.
///
/// Calibration: ControllingCircuitPrecedent is the lowest binding
/// tier; ARB decisions are the highest persuasive tier (Skidmore
/// 1944) — they hover at the boundary but the binding cutoff sits
/// above them per Chevron/Skidmore distinction.
pub const BINDING_TIER_FLOOR: BindingForceTier = BindingForceTier::ControllingCircuitPrecedent;

// ---------------------------------------------------------------------------
// Quality: JurisdictionScopeOf — horizontal scope as typed Identifier
// ---------------------------------------------------------------------------

/// Quality: the jurisdiction within which an authority is *binding*.
/// Returns a typed [`Identifier`] (CURIE into a future jurisdiction
/// ontology, e.g. `jurisdiction:us_federal`, `jurisdiction:circuit_10`,
/// `jurisdiction:district_co`). For concepts whose binding scope is
/// universal in the federal system (`SupremeCourtPrecedent`,
/// `ConstitutionalText`, `FederalStatute`), the value is
/// `jurisdiction:us_federal`. For controlling-circuit precedent, the
/// circuit identifier is the concept's binding scope but varies by
/// instance — the *type*-level quality returns the generic
/// `jurisdiction:single_circuit` placeholder; each instance carries
/// its own concrete CURIE in the broader source registry.
///
/// Returns `None` for persuasive concepts (their binding scope is
/// trivially empty — they bind no court) and for the abstract root.
///
/// Source: Garner (2016) §11.1 (circuit-internal binding scope);
/// Erie R.R. v. Tompkins, 304 U.S. 64 (1938) (federal-state hierarchy
/// in diversity); Guarino & Welty (2002) (type/role distinction —
/// scope is a *type*-level property when generic, attaches to
/// instances when concrete).
#[derive(Debug, Clone)]
pub struct JurisdictionScopeOf;

impl Quality for JurisdictionScopeOf {
    type Individual = AuthorityStrengthConcept;
    type Value = Identifier;

    fn get(&self, c: &AuthorityStrengthConcept) -> Option<Identifier> {
        use AuthorityStrengthConcept as A;
        match c {
            // Universal federal scope at type level.
            A::ConstitutionalText | A::FederalStatute | A::SupremeCourtPrecedent => {
                Identifier::curie("jurisdiction:us_federal").ok()
            }
            // Universal federal scope; binds when delegation valid.
            A::FederalRegulation => Identifier::curie("jurisdiction:us_federal").ok(),
            // Circuit-internal — placeholder at type level; instances
            // carry concrete circuit CURIE.
            A::ControllingCircuitPrecedent => Identifier::curie("jurisdiction:single_circuit").ok(),
            // Persuasive concepts have empty binding scope.
            A::AdministrativeReviewBoardDecision
            | A::SisterCircuitPrecedent
            | A::DistrictCourtPrecedent
            | A::SecondarySource => None,
            // Abstract concepts.
            A::AuthorityStrength | A::BindingAuthority | A::PersuasiveAuthority => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The nine leaves of the partition, in descending binding-force order.
pub fn leaves() -> [AuthorityStrengthConcept; 9] {
    [
        AuthorityStrengthConcept::ConstitutionalText,
        AuthorityStrengthConcept::FederalStatute,
        AuthorityStrengthConcept::SupremeCourtPrecedent,
        AuthorityStrengthConcept::FederalRegulation,
        AuthorityStrengthConcept::ControllingCircuitPrecedent,
        AuthorityStrengthConcept::AdministrativeReviewBoardDecision,
        AuthorityStrengthConcept::SisterCircuitPrecedent,
        AuthorityStrengthConcept::DistrictCourtPrecedent,
        AuthorityStrengthConcept::SecondarySource,
    ]
}

/// The five binding-authority leaves.
pub fn binding_leaves() -> [AuthorityStrengthConcept; 5] {
    [
        AuthorityStrengthConcept::ConstitutionalText,
        AuthorityStrengthConcept::FederalStatute,
        AuthorityStrengthConcept::SupremeCourtPrecedent,
        AuthorityStrengthConcept::FederalRegulation,
        AuthorityStrengthConcept::ControllingCircuitPrecedent,
    ]
}

/// The four persuasive-authority leaves.
pub fn persuasive_leaves() -> [AuthorityStrengthConcept; 4] {
    [
        AuthorityStrengthConcept::AdministrativeReviewBoardDecision,
        AuthorityStrengthConcept::SisterCircuitPrecedent,
        AuthorityStrengthConcept::DistrictCourtPrecedent,
        AuthorityStrengthConcept::SecondarySource,
    ]
}

/// True iff `c` is one of the nine concrete leaves.
pub fn is_leaf(c: AuthorityStrengthConcept) -> bool {
    leaves().contains(&c)
}

/// True iff `c` is a binding-authority leaf (tier ≥ [`BINDING_TIER_FLOOR`]).
pub fn is_binding(c: AuthorityStrengthConcept) -> bool {
    binding_leaves().contains(&c)
}

/// True iff `c` is a persuasive-authority leaf (tier < [`BINDING_TIER_FLOOR`]).
pub fn is_persuasive(c: AuthorityStrengthConcept) -> bool {
    persuasive_leaves().contains(&c)
}

/// True iff `a` is at least as binding as `b` — i.e., `a`'s tier is
/// greater than or equal to `b`'s tier. Total on leaves; `None` if
/// either argument is non-leaf.
pub fn at_least_as_binding(
    a: AuthorityStrengthConcept,
    b: AuthorityStrengthConcept,
) -> Option<bool> {
    let q = BindingForceOf;
    match (q.get(&a), q.get(&b)) {
        (Some(x), Some(y)) => Some(x >= y),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

impl Ontology for AuthorityStrengthOntology {
    type Cat = AuthorityStrengthCategory;
    type Qual = BindingForceOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartitionCompleteness));
        axioms.push(Box::new(BindingExceedsAllPersuasive));
        axioms.push(Box::new(ConstitutionalSupremacy));
        axioms.push(Box::new(StatuteExceedsRegulation));
        axioms.push(Box::new(SupremeCourtAtopPrecedentHierarchy));
        axioms.push(Box::new(ForceTiersAreDistinct));
        axioms
    }
}

/// Axiom: the partition has exactly nine concrete leaves — five
/// binding plus four persuasive — and the two branch concepts plus
/// the root partition them. Any additional leaf would require fresh
/// primary-source attestation.
pub struct PartitionCompleteness;

impl Axiom for PartitionCompleteness {
    fn verify(&self) -> Verdict {
        let n = AuthorityStrengthConcept::variants()
            .into_iter()
            .filter(|c| is_leaf(*c))
            .count();
        let bn = binding_leaves().iter().filter(|c| is_binding(**c)).count();
        let pn = persuasive_leaves()
            .iter()
            .filter(|c| is_persuasive(**c))
            .count();
        if n == 9 && bn == 5 && pn == 4 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PartitionCompleteness",
        "the AuthorityStrength partition has exactly nine leaves (5 binding + 4 persuasive)",
        "Schauer (2009) Ch. 5; Garner (2016) §1.2"
    );
}

pr4xis::register_axiom!(PartitionCompleteness, "Schauer (2009); Garner (2016)");

/// Axiom: every binding-authority leaf has a strictly higher
/// binding-force tier than every persuasive-authority leaf. The
/// boundary is the [`BINDING_TIER_FLOOR`] constant.
pub struct BindingExceedsAllPersuasive;

impl Axiom for BindingExceedsAllPersuasive {
    fn verify(&self) -> Verdict {
        let q = BindingForceOf;
        for b in binding_leaves() {
            for p in persuasive_leaves() {
                match (q.get(&b), q.get(&p)) {
                    (Some(bx), Some(px)) if bx > px => {}
                    _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "BindingExceedsAllPersuasive",
        "every binding-authority leaf's tier strictly exceeds every persuasive-authority leaf's tier",
        "Schauer (2009) Ch. 5; Garner (2016) §1.2"
    );
}

pr4xis::register_axiom!(BindingExceedsAllPersuasive, "Schauer (2009); Garner (2016)");

/// Axiom: ConstitutionalText is the *unique* maximum tier — no other
/// authority concept has a tier as high. Captures Marbury (1803) and
/// the Supremacy Clause (Art. VI cl. 2).
pub struct ConstitutionalSupremacy;

impl Axiom for ConstitutionalSupremacy {
    fn verify(&self) -> Verdict {
        let q = BindingForceOf;
        let ct = q.get(&AuthorityStrengthConcept::ConstitutionalText);
        for c in AuthorityStrengthConcept::variants() {
            if c == AuthorityStrengthConcept::ConstitutionalText || !is_leaf(c) {
                continue;
            }
            match (ct, q.get(&c)) {
                (Some(top), Some(other)) if top > other => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ConstitutionalSupremacy",
        "ConstitutionalText is the unique maximum tier — all other authority concepts have strictly lower binding force",
        "U.S. Const. Art. VI cl. 2; Marbury v. Madison, 5 U.S. 137 (1803)"
    );
}

pr4xis::register_axiom!(
    ConstitutionalSupremacy,
    "U.S. Const. Art. VI cl. 2; Marbury v. Madison (1803)"
);

/// Axiom: federal statutes exceed federal regulations in binding
/// force. Captured doctrinally by Article I (legislative supremacy
/// over delegated rulemaking) and the Chevron framework, which
/// conditions regulatory binding force on statutory authorization.
pub struct StatuteExceedsRegulation;

impl Axiom for StatuteExceedsRegulation {
    fn verify(&self) -> Verdict {
        let q = BindingForceOf;
        match (
            q.get(&AuthorityStrengthConcept::FederalStatute),
            q.get(&AuthorityStrengthConcept::FederalRegulation),
        ) {
            (Some(s), Some(r)) if s > r => Ok(Box::new(SimpleProof::new(self.meta()))),
            _ => Err(Box::new(SimpleCounterexample::new(self.meta()))),
        }
    }

    pr4xis::axiom_meta!(
        "StatuteExceedsRegulation",
        "FederalStatute tier strictly exceeds FederalRegulation tier",
        "U.S. Const. Art. I; Chevron U.S.A. v. NRDC, 467 U.S. 837 (1984)"
    );
}

pr4xis::register_axiom!(
    StatuteExceedsRegulation,
    "U.S. Const. Art. I; Chevron (1984)"
);

/// Axiom: SupremeCourtPrecedent exceeds every other precedent concept
/// (Controlling/Sister/District). Captures the vertical precedent
/// hierarchy from Garner (2016) §1.2 and Schauer (2009) Ch. 3.
pub struct SupremeCourtAtopPrecedentHierarchy;

impl Axiom for SupremeCourtAtopPrecedentHierarchy {
    fn verify(&self) -> Verdict {
        let q = BindingForceOf;
        let sc = q.get(&AuthorityStrengthConcept::SupremeCourtPrecedent);
        for c in [
            AuthorityStrengthConcept::ControllingCircuitPrecedent,
            AuthorityStrengthConcept::SisterCircuitPrecedent,
            AuthorityStrengthConcept::DistrictCourtPrecedent,
        ] {
            match (sc, q.get(&c)) {
                (Some(top), Some(other)) if top > other => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SupremeCourtAtopPrecedentHierarchy",
        "SupremeCourtPrecedent tier strictly exceeds Controlling, Sister, and District precedent tiers",
        "Garner (2016) §1.2; Schauer (2009) Ch. 3"
    );
}

pr4xis::register_axiom!(
    SupremeCourtAtopPrecedentHierarchy,
    "Garner (2016) §1.2; Schauer (2009) Ch. 3"
);

/// Axiom: every leaf has a distinct binding-force tier — the ordering
/// is strict, not merely partial. Required so that conflict-resolution
/// rules over composed authorities have a unique winner whenever the
/// authorities are at different tiers.
pub struct ForceTiersAreDistinct;

impl Axiom for ForceTiersAreDistinct {
    fn verify(&self) -> Verdict {
        let q = BindingForceOf;
        let mut tiers: Vec<BindingForceTier> = leaves().iter().filter_map(|c| q.get(c)).collect();
        tiers.sort();
        let original = tiers.len();
        tiers.dedup();
        if tiers.len() == original {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ForceTiersAreDistinct",
        "the nine leaves each receive a distinct binding-force tier — the ordering is strict",
        "Sartor (2005) Ch. 21 (conflict resolution requires strict ordering)"
    );
}

pr4xis::register_axiom!(ForceTiersAreDistinct, "Sartor (2005) Ch. 21");
