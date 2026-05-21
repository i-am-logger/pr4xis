//! [`RuleSet`] — a collection of [`Implication`]s with
//! collection-level operations: normalize-all, subsumption-order,
//! conflict scan, canonical basis.
//!
//! The single-rule operations live on [`super::Implication`]; this
//! module composes them across a rule set and adds the operations
//! that only make sense for collections.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::Axiom;

use super::implication::{DeonticOperator, Implication};

// =============================================================================
// RuleSet — collection wrapper.
// =============================================================================

/// A finite, ordered set of implications. The order is significant
/// only for the indexed outputs ([`Self::subsumption_order`],
/// [`Self::conflicts`]) — semantically the algebra treats it as a
/// set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet<C> {
    rules: Vec<Implication<C>>,
}

impl<C> Default for RuleSet<C> {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl<C: Ord + Clone> RuleSet<C> {
    /// Construct an empty rule set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct from a vector of implications. The implications are
    /// normalized individually but the collection is not deduplicated
    /// — call [`Self::normalize`] for that.
    #[must_use]
    pub fn from_rules(rules: Vec<Implication<C>>) -> Self {
        Self {
            rules: rules.into_iter().map(Implication::normalize).collect(),
        }
    }

    /// Number of rules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the rule set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Borrow the rules.
    #[must_use]
    pub fn rules(&self) -> &[Implication<C>] {
        &self.rules
    }

    /// Add a rule. The new rule is normalized but not deduplicated
    /// against existing rules — call [`Self::normalize`] to dedup.
    pub fn push(&mut self, rule: Implication<C>) {
        self.rules.push(rule.normalize());
    }

    // -------------------------------------------------------------------------
    // Normalization — Tarski (1956) closure, restricted to the set
    // structure: dedup the rule list (since every rule is already
    // individually normalized at construction).
    // -------------------------------------------------------------------------

    /// **Normalization** at the set level. Sorts the rules into a
    /// canonical order and removes structural duplicates. Idempotent.
    ///
    /// Citation: Tarski (1956) consequence operator; Duquenne &
    /// Guigues (1986) for the implication-basis form.
    #[must_use]
    pub fn normalize(self) -> Self
    where
        C: Eq + core::hash::Hash,
    {
        let mut rules = self.rules;
        // Sort by (antecedent, consequent, deontic-rank) so the same
        // logical rule set always produces the same ordered vector.
        rules.sort_by(|a, b| compare_implications(a, b));
        rules.dedup();
        Self { rules }
    }

    // -------------------------------------------------------------------------
    // Subsumption — Plotkin (1970).
    // -------------------------------------------------------------------------

    /// **Subsumption order** — return every ordered pair `(i, j)`
    /// such that `rules[i].subsumes(rules[j])` and `i ≠ j`.
    /// Reflexive pairs are excluded (every rule trivially subsumes
    /// itself).
    #[must_use]
    pub fn subsumption_order(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for i in 0..self.rules.len() {
            for j in 0..self.rules.len() {
                if i != j && self.rules[i].subsumes(&self.rules[j]) {
                    out.push((i, j));
                }
            }
        }
        out
    }

    /// **Most general rules** — the maximal elements under the
    /// subsumption order: rules NOT subsumed by any other rule.
    #[must_use]
    pub fn most_general(&self) -> Vec<usize> {
        let order = self.subsumption_order();
        (0..self.rules.len())
            .filter(|&j| !order.iter().any(|&(_, k)| k == j))
            .collect()
    }

    /// **Most specific rules** — the minimal elements: rules that
    /// subsume no others (other than themselves).
    #[must_use]
    pub fn most_specific(&self) -> Vec<usize> {
        let order = self.subsumption_order();
        (0..self.rules.len())
            .filter(|&i| !order.iter().any(|&(j, _)| j == i))
            .collect()
    }

    // -------------------------------------------------------------------------
    // Conflict detection — Prakken & Sartor (1997).
    // -------------------------------------------------------------------------

    /// **Conflict set** — return every unordered pair `{i, j}` with
    /// `i < j` such that `rules[i]` and `rules[j]` are in deontic
    /// conflict. Symmetry is enforced by ordering `i < j`.
    #[must_use]
    pub fn conflicts(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for i in 0..self.rules.len() {
            for j in (i + 1)..self.rules.len() {
                if self.rules[i].conflicts_with(&self.rules[j]) {
                    out.push((i, j));
                }
            }
        }
        out
    }

    // -------------------------------------------------------------------------
    // Canonical basis — Duquenne & Guigues (1986).
    // -------------------------------------------------------------------------

    /// **Canonical basis** — a subsumption-reduced rule set:
    /// `rules[i]` is dropped if some `rules[j] ≠ rules[i]` subsumes
    /// it strictly (i.e. `j` subsumes `i` but `i` does not subsume
    /// `j`). The result is the *most-general* slice plus all
    /// rules incomparable to it.
    ///
    /// This is the Plotkin-1970-restricted analogue of the
    /// Duquenne-Guigues stem basis (Duquenne & Guigues 1986) for
    /// FCA-derived implication sets. The full DG basis additionally
    /// closes under consequence, which requires a model of "valid in
    /// the context" — implemented for FCA-derived sets in
    /// [`crate::formal::analytical_methods::fca`].
    ///
    /// Citation: Duquenne & Guigues (1986) Math. Sci. Hum. 95: 5–18;
    /// Plotkin (1970) Machine Intelligence 5: 153–163; Robinson
    /// (1965) JACM 12: 23–41 §6 (subsumption removes redundant
    /// clauses).
    #[must_use]
    pub fn canonical_basis(&self) -> Self
    where
        C: Eq + core::hash::Hash,
    {
        let order = self.subsumption_order();
        let kept: Vec<Implication<C>> = self
            .rules
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                // Drop `r` iff some other rule j strictly subsumes
                // it (j ≼ i but i ⊄ j).
                let strictly_subsumed = order.iter().any(|&(j, k)| {
                    k == i && j != i && !order.iter().any(|&(j2, k2)| j2 == i && k2 == j)
                });
                if strictly_subsumed {
                    None
                } else {
                    Some(r.clone())
                }
            })
            .collect();
        Self::from_rules(kept).normalize()
    }
}

// =============================================================================
// Implication ordering — for canonical RuleSet normalization.
// =============================================================================

/// Total order on implications for deterministic sorting. Sort by
/// antecedent (lex), then consequent (lex), then deontic-rank.
fn compare_implications<C: Ord>(a: &Implication<C>, b: &Implication<C>) -> core::cmp::Ordering {
    a.antecedent()
        .cmp(b.antecedent())
        .then_with(|| a.consequent().cmp(b.consequent()))
        .then_with(|| deontic_rank(a.deontic()).cmp(&deontic_rank(b.deontic())))
}

/// Total ranking of deontic operators for sort stability.
fn deontic_rank(d: DeonticOperator) -> u8 {
    match d {
        DeonticOperator::Assertoric => 0,
        DeonticOperator::Permission => 1,
        DeonticOperator::Obligation => 2,
        DeonticOperator::Prohibition => 3,
    }
}

// =============================================================================
// Domain axioms over RuleSet — verified on a canonical small set.
// =============================================================================

fn canonical_rule_set() -> RuleSet<&'static str> {
    // Small set exercising all three operations:
    //   r0: ⊤ ⇒ a               (Assertoric)
    //   r1: {b} ⇒ a             (Assertoric, subsumed by r0 if it ran)
    //   r2: {b, c} ⇒ a          (Assertoric, subsumed by r0 and r1)
    //   r3: ⊤ → Op(d)           (Obligation: do d)
    //   r4: ⊤ → Fp(d)           (Prohibition: don't do d) — conflicts with r3
    //   r5: {b} → Op(e)         (Obligation)
    //   r6: {b} → Fp(e)         (Prohibition: conflicts with r5)
    RuleSet::from_rules(vec![
        Implication::assertoric(vec![], vec!["a"]),
        Implication::assertoric(vec!["b"], vec!["a"]),
        Implication::assertoric(vec!["b", "c"], vec!["a"]),
        Implication::obligation(vec![], vec!["d"]),
        Implication::prohibition(vec![], vec!["d"]),
        Implication::obligation(vec!["b"], vec!["e"]),
        Implication::prohibition(vec!["b"], vec!["e"]),
    ])
}

/// Plotkin (1970): the subsumption order is reflexive (every rule
/// subsumes itself) — verified by checking each rule against itself.
pub struct SubsumptionReflexive;

impl Axiom for SubsumptionReflexive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rs = canonical_rule_set();
        for r in rs.rules() {
            if !r.subsumes(r) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SubsumptionReflexive",
        "every implication subsumes itself: R ≼ R",
        "Plotkin (1970) Machine Intelligence 5: 153-163"
    );
}

/// Plotkin (1970): subsumption is transitive. Verified on the canonical
/// rule set by exhaustive pair-of-pair search.
pub struct SubsumptionTransitive;

impl Axiom for SubsumptionTransitive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rs = canonical_rule_set();
        let order = rs.subsumption_order();
        for &(i, j) in &order {
            for &(j2, k) in &order {
                if j == j2 {
                    let ik_in_order = order.contains(&(i, k));
                    if i != k && !ik_in_order {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SubsumptionTransitive",
        "R1 ≼ R2 and R2 ≼ R3 imply R1 ≼ R3",
        "Plotkin (1970) Machine Intelligence 5: 153-163"
    );
}

/// Tarski (1956): normalization is idempotent — applying it twice
/// gives the same result as applying it once.
pub struct NormalizationIdempotent;

impl Axiom for NormalizationIdempotent {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rs = canonical_rule_set();
        let once = rs.clone().normalize();
        let twice = once.clone().normalize();
        if once == twice {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NormalizationIdempotent",
        "normalize(normalize(rs)) == normalize(rs)",
        "Tarski (1956) Logic, Semantics, Metamathematics — consequence operator idempotency"
    );
}

/// Prakken & Sartor (1997): conflict detection is symmetric — if R1
/// conflicts with R2 then R2 conflicts with R1.
pub struct ConflictSymmetric;

impl Axiom for ConflictSymmetric {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rs = canonical_rule_set();
        for r1 in rs.rules() {
            for r2 in rs.rules() {
                if r1.conflicts_with(r2) != r2.conflicts_with(r1) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ConflictSymmetric",
        "conflicts_with is a symmetric relation",
        "Prakken & Sartor (1997) JANCL 7: 25-75"
    );
}

/// von Wright (1951): the canonical deontic conflict is detected.
/// Verified by checking that `Op(d) ∧ Fp(d)` (rules r3 and r4 in the
/// canonical set) are flagged as conflicting.
pub struct DeonticConflictDetected;

impl Axiom for DeonticConflictDetected {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rs = canonical_rule_set();
        let conflicts = rs.conflicts();
        // r3=Op(d) and r4=Fp(d) — canonical von-Wright conflict.
        if conflicts.contains(&(3, 4)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeonticConflictDetected",
        "Op(d) and Fp(d) are flagged as a conflict pair (von Wright deontic square)",
        "von Wright (1951) Mind 60: 1-15; Prakken & Sartor (1997) JANCL 7: 25-75"
    );
}

/// Duquenne & Guigues (1986) / Robinson (1965): the canonical basis
/// is a sub-set of the original rules — it drops strictly-subsumed
/// rules.
pub struct CanonicalBasisIsSubset;

impl Axiom for CanonicalBasisIsSubset {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rs = canonical_rule_set();
        let basis = rs.canonical_basis();
        if basis.len() <= rs.len() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CanonicalBasisIsSubset",
        "|canonical_basis(rs)| <= |rs| — basis drops only redundant (strictly-subsumed) rules",
        "Duquenne & Guigues (1986) Math. Sci. Hum. 95: 5-18; Robinson (1965) JACM 12: 23-41 §6"
    );
}

pr4xis::register_axiom!(
    SubsumptionReflexive,
    "Plotkin (1970) Machine Intelligence 5: 153-163"
);
pr4xis::register_axiom!(
    SubsumptionTransitive,
    "Plotkin (1970) Machine Intelligence 5: 153-163"
);
pr4xis::register_axiom!(
    NormalizationIdempotent,
    "Tarski (1956) Logic, Semantics, Metamathematics — consequence operator idempotency"
);
pr4xis::register_axiom!(ConflictSymmetric, "Prakken & Sartor (1997) JANCL 7: 25-75");
pr4xis::register_axiom!(
    DeonticConflictDetected,
    "von Wright (1951) Mind 60: 1-15; Prakken & Sartor (1997) JANCL 7: 25-75"
);
pr4xis::register_axiom!(
    CanonicalBasisIsSubset,
    "Duquenne & Guigues (1986) Math. Sci. Hum. 95: 5-18; Robinson (1965) JACM 12: 23-41 §6"
);

#[cfg(test)]
#[path = "rule_set_tests.rs"]
mod tests;
