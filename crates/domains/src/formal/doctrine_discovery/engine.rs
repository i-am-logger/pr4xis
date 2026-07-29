//! Runtime engine for [`DoctrineDiscovery`]:
//! `discover` takes a `FormalContext<O, A>` and returns the canonical
//! discovery output — concept-lattice fibration + canonical
//! implication basis + an abductive-schema reading per Peirce (1903).
//!
//! See [`super::ontology`] for the type-level concept inventory and
//! full literature.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use core::marker::PhantomData;

use pr4xis::ontology::Axiom;

use crate::formal::analytical_methods::fca::BitSet;
use crate::formal::analytical_methods::{ConceptLatticeFibration, FormalContext};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::rule_algebra::{Implication, RuleSet};

// =============================================================================
// AttributeExtractor — the pluggable feature-extraction interface.
// =============================================================================

/// Map each object in a corpus to its attribute set. Implementors
/// supply the "what are the attributes of this section?" answer the
/// discovery engine relies on.
///
/// Per Maedche-Staab (2001) §3, attribute extraction is the pluggable
/// boundary between corpus-specific feature engineering and the
/// corpus-agnostic FCA + rule-algebra machinery downstream.
pub trait AttributeExtractor<O, A> {
    /// Return the attribute set for `object`. Implementations are
    /// expected to be **total**: no panics, finite output. Repeated
    /// attributes are deduplicated by the engine.
    fn attributes_of(&self, object: &O) -> Vec<A>;
}

impl<O, A, F> AttributeExtractor<O, A> for F
where
    F: Fn(&O) -> Vec<A>,
{
    fn attributes_of(&self, object: &O) -> Vec<A> {
        self(object)
    }
}

// =============================================================================
// DoctrineDiscovery — the engine's output bundle.
// =============================================================================

/// Result of running [`discover`] over a corpus. Holds the concept-
/// lattice fibration over the Classification ontology, the canonical
/// implication basis derived from attribute closures, and the
/// subsumption order on those implications.
///
/// All three views agree on the source corpus and attribute alphabet
/// — the lattice's concepts and the implications' antecedents/
/// consequents share `A` indices.
pub struct DoctrineDiscovery<O, A> {
    /// The concept-lattice fibration over the Classification
    /// ontology. Each concept is a doctrine cluster; the lattice
    /// order encodes the hierarchy returned by `hierarchy()`.
    pub fibration: ConceptLatticeFibration<O, A>,
    /// The canonical implication basis: a [`RuleSet`] of
    /// `Implication::assertoric({m}, closure({m}))` for every
    /// attribute `m` whose closure is non-trivial, normalized and
    /// subsumption-reduced via `RuleSet::canonical_basis`.
    pub basis: RuleSet<A>,
    /// The Plotkin (1970) θ-subsumption order over `basis.rules()`.
    /// `subsumption_order[k] = (i, j)` means `basis.rules()[i]`
    /// strictly subsumes `basis.rules()[j]`.
    pub subsumption_order: Vec<(usize, usize)>,
    _phantom: PhantomData<fn() -> (O, A)>,
}

impl<O, A: Clone + Ord> DoctrineDiscovery<O, A> {
    /// Number of formal concepts discovered (= doctrine clusters). A
    /// dimensionless count.
    #[must_use]
    pub fn cluster_count(&self) -> Quantity {
        Quantity::from_unit(self.fibration.lattice.len() as f64, &unit::UNITLESS)
    }

    /// Number of implications in the canonical basis.
    #[must_use]
    pub fn implication_count(&self) -> usize {
        self.basis.len()
    }
}

// =============================================================================
// `discover` — the engine entry point.
// =============================================================================

/// Run the full doctrine-discovery pipeline on the supplied formal
/// context. Stages 1–4 produce the concept lattice (Wille 1982;
/// Ganter 1984), stage 5 lifts it to the Classification fibration
/// (Grothendieck 1971), stages 6–8 extract and normalise
/// implications (Duquenne-Guigues 1986; Plotkin 1970; Tarski 1956),
/// stage 9 leaves an abductive reading available via the
/// `causation::CausationToDerivation` functor (Peirce 1903), and
/// stage 10 packages the result as [`DoctrineDiscovery`].
///
/// The function is *deterministic*: same context → same output, byte-
/// identical, per the praxis-correctness discipline.
pub fn discover<O, A>(ctx: &FormalContext<O, A>) -> DoctrineDiscovery<O, A>
where
    O: Clone,
    A: Clone + Ord + core::hash::Hash,
{
    // Stages 4–5: lattice build + fibration lift.
    let fibration = ConceptLatticeFibration::from_context(ctx);

    // Stage 6: closure extraction. For each attribute m, compute
    // `{m}'' = (singleton(m))''` — the set of attributes implied by
    // m's presence. If the closure adds anything beyond {m}, emit an
    // `Implication::assertoric({m}, closure({m}))`.
    let m_count = ctx.attributes().len();
    let mut raw_implications: Vec<Implication<A>> = Vec::new();
    for mi in 0..m_count {
        let mut x = BitSet::empty(m_count);
        x.set(mi);
        let close = ctx.intent_closure(&x);
        if close.count().value <= 1.0 {
            // Trivial closure (singleton remains a singleton — no
            // co-occurring attributes). Skip.
            continue;
        }
        let antecedent: Vec<A> = vec![ctx.attributes()[mi].clone()];
        let consequent: Vec<A> = close.iter().map(|i| ctx.attributes()[i].clone()).collect();
        raw_implications.push(Implication::assertoric(antecedent, consequent));
    }

    // Stages 7–8: normalise + canonical basis + subsumption order.
    let raw = RuleSet::from_rules(raw_implications);
    let basis = raw.canonical_basis();
    let subsumption_order = basis.subsumption_order();

    // Stages 9–10: abductive lift is available via callers using
    // `causation::CausationToDerivation` against the lattice; we
    // expose the data here without forcing the lift.
    DoctrineDiscovery {
        fibration,
        basis,
        subsumption_order,
        _phantom: PhantomData,
    }
}

// =============================================================================
// Domain axioms — invariants the engine output must satisfy.
// =============================================================================

fn ganter_wille_context() -> FormalContext<&'static str, &'static str> {
    // Canonical small context from Ganter & Wille (1999) Fig. 1.1.
    // Same one used by FCA + fibration axioms — gives us a
    // hand-verifiable doctrine-discovery output.
    FormalContext::from_matrix(
        vec!["fish", "dog", "reed", "bean"],
        vec!["needs_water", "has_limbs", "can_move", "has_skeleton"],
        vec![
            vec![true, false, true, true],
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    )
}

/// Ganter & Wille (1999) Theorem 3: the discovered cluster count
/// equals the underlying concept-lattice size — the engine must not
/// drop or hallucinate clusters.
pub struct DiscoveredClustersMatchLattice;

impl Axiom for DiscoveredClustersMatchLattice {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = ganter_wille_context();
        let lat_size = ctx.build_lattice().len();
        let disc = discover(&ctx);
        if disc.cluster_count().value as usize == lat_size {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DiscoveredClustersMatchLattice",
        "the engine's cluster count equals the underlying concept-lattice size",
        "Ganter & Wille (1999) Formal Concept Analysis Theorem 3"
    );
}

/// Ganter & Wille (1999) §2.3: every implication the engine emits is
/// *valid in the context* — i.e., its consequent is contained in the
/// closure of its antecedent under the context's double-derivation.
pub struct EveryImplicationIsContextValid;

impl Axiom for EveryImplicationIsContextValid {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = ganter_wille_context();
        let disc = discover(&ctx);
        let m_count = ctx.attributes().len();
        let attr_index = |name: &&str| {
            ctx.attributes()
                .iter()
                .position(|a| a == name)
                .expect("attribute present")
        };
        for imp in disc.basis.rules() {
            let mut x = BitSet::empty(m_count);
            for a in imp.antecedent() {
                x.set(attr_index(a));
            }
            let close = ctx.intent_closure(&x);
            for a in imp.consequent() {
                if !close.contains(attr_index(a)) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryImplicationIsContextValid",
        "every implication emitted by the engine is valid in the source context (consequent ⊆ closure(antecedent))",
        "Ganter & Wille (1999) Formal Concept Analysis §2.3"
    );
}

/// Plotkin (1970) / Robinson (1965): the canonical basis is
/// subsumption-minimal — no implication in `basis.rules()` is
/// strictly subsumed by another.
pub struct CanonicalBasisIsSubsumptionMinimal;

impl Axiom for CanonicalBasisIsSubsumptionMinimal {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = ganter_wille_context();
        let disc = discover(&ctx);
        for &(i, j) in &disc.subsumption_order {
            // No rule should be strictly subsumed: if i ≼ j AND
            // j ⊀ i, the basis lost a redundant rule.
            let inverse = disc.subsumption_order.contains(&(j, i));
            if !inverse {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CanonicalBasisIsSubsumptionMinimal",
        "no implication in the canonical basis is strictly subsumed by another",
        "Plotkin (1970) Machine Intelligence 5: 153-163; Robinson (1965) JACM 12: 23-41 §6"
    );
}

/// Deterministic discovery — calling [`discover`] twice on the same
/// context produces equal outputs (cluster counts, implication
/// counts, subsumption-order edges). A regression guard for any
/// future cache or randomised heuristic.
pub struct DiscoveryIsDeterministic;

impl Axiom for DiscoveryIsDeterministic {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ctx = ganter_wille_context();
        let a = discover(&ctx);
        let b = discover(&ctx);
        if a.cluster_count() == b.cluster_count()
            && a.implication_count() == b.implication_count()
            && a.subsumption_order == b.subsumption_order
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DiscoveryIsDeterministic",
        "the engine is deterministic: same context → same output (clusters, basis, subsumption order)",
        "Cimiano-Hotho-Staab (2005) JAIR 24:305-339 (reproducibility of FCA-based ontology learning)"
    );
}

pr4xis::register_axiom!(
    DiscoveredClustersMatchLattice,
    "Ganter & Wille (1999) Formal Concept Analysis Theorem 3"
);
pr4xis::register_axiom!(
    EveryImplicationIsContextValid,
    "Ganter & Wille (1999) Formal Concept Analysis §2.3"
);
pr4xis::register_axiom!(
    CanonicalBasisIsSubsumptionMinimal,
    "Plotkin (1970) Machine Intelligence 5: 153-163; Robinson (1965) JACM 12: 23-41 §6"
);
pr4xis::register_axiom!(
    DiscoveryIsDeterministic,
    "Cimiano-Hotho-Staab (2005) JAIR 24:305-339"
);

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
