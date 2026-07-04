//! The five guarantees, as a first-class ontology.
//!
//! These are not promises offered to a user; they are the properties pr4xis
//! holds about its own reasoning — the conditions under which a computation
//! counts as pr4xis reasoning at all. Each test in the workspace declares,
//! via `#[pr4xis::praxis_value(..)]`, which guarantee it witnesses, and the
//! `constitution_coverage` meta-test partitions the suite across them.
//!
//! # Structure: five answer-guarantees + one composition guarantee
//!
//! Five of the values are **answer-guarantees** — properties of a single answer:
//! `Verifiable`, `Deterministic`, `Explainable`, `Honest`, `Consistent`.
//! `Extensible` is **second-order** — the property that those five are preserved
//! under composition. It is a meta-property *over* the others, modeled with
//! `Preserves` edges pointing at each, not a peer.
//!
//! `Honest` is the keystone of the answer-guarantees: Verifiable, Deterministic,
//! Explainable and Consistent are credible only because the system can refuse —
//! leave a claim ungrounded rather than confabulate. Hence each carries a
//! `Grounds` edge *from* `Honest`.
//!
//! # Literature
//!
//! - **von Foerster (1981)** *Observing Systems* — the eigenform (grounds `Explainable`).
//! - **Knuth (1997)** *TAOCP Vol. 1* §1.1 — definiteness (grounds `Deterministic`).
//! - **Grice (1975)** *Logic and Conversation* — maxim of Quality (grounds `Honest`).
//! - **Lakatos (1976)** *Proofs and Refutations* — refutation is constitutive of
//!   knowledge (grounds the keystone role of `Honest`).
//! - **Gentzen (1936)** *Die Widerspruchsfreiheit der reinen Zahlentheorie* —
//!   a theory is consistent iff it proves no `⊥` (grounds `Consistent`).
//! - **Peroni & Shotton (2012)** *FaBiO and CiTO* — typed provenance (grounds `Verifiable`).
//! - **Spivak (2014)** *Category Theory for the Sciences* — functorial
//!   composition; proven parts assemble into a proven whole (grounds `Extensible`).

use crate as pr4xis;

pr4xis::ontology! {
    name: "Constitution",
    source: "von Foerster (1981) Observing Systems; Knuth (1997) The Art of Computer Programming, Vol. 1, §1.1; Grice (1975) Logic and Conversation, in Syntax and Semantics 3; Lakatos (1976) Proofs and Refutations, Cambridge University Press; Spivak (2014) Category Theory for the Sciences, MIT Press; Peroni & Shotton (2012) FaBiO and CiTO, J. Web Semantics 17; Gentzen (1936) Die Widerspruchsfreiheit der reinen Zahlentheorie, Math. Annalen 112; Gruber (1995) Toward Principles for the Design of Ontologies, IJHCS 43:907",

    concepts: [
        Verifiable,
        Deterministic,
        Explainable,
        Honest,
        Consistent,
        Extensible,
    ],

    labels: {
        Verifiable: ("en", "Verifiable", "Every claim carries its source and can be checked by an external observer; provenance is mandatory, not optional (Peroni & Shotton 2012, CiTO)."),
        Deterministic: ("en", "Deterministic", "The same input yields the same output on every run — definiteness: each step is precisely defined, with no randomness or state-dependence (Knuth 1997, TAOCP I §1.1)."),
        Explainable: ("en", "Explainable", "The system describes its own structure; the reasoning path is the answer — a fixed point of self-observation (von Foerster 1981, eigenform)."),
        Honest: ("en", "Honest", "What it cannot ground it leaves ungrounded — it refuses rather than confabulate (Grice 1975, maxim of Quality)."),
        Consistent: ("en", "Consistent", "The axiom base derives no contradiction — every registered axiom holds against the corpus, so the engine cannot prove a thing and its negation (Gentzen 1936, cut-elimination; Gruber 1995, Coherence)."),
        Extensible: ("en", "Extensible", "Second-order: the five answer-guarantees are PRESERVED under composition — new ontologies attach by law-checked functor without degrading the rest. A meta-property over the others, not a peer answer-guarantee (Spivak 2014, functorial composition; Gruber 1995, Extendibility)."),
    },

    edges: [
        // Honest is the keystone: it grounds every other *answer*-guarantee.
        (Honest, Verifiable, Grounds),
        (Honest, Deterministic, Grounds),
        (Honest, Explainable, Grounds),
        (Honest, Consistent, Grounds),
        // Determinism is what makes verification and explanation possible.
        (Deterministic, Verifiable, Enables),
        (Deterministic, Explainable, Enables),
        // A consistent base is what makes verification meaningful at all.
        (Consistent, Verifiable, Enables),
        // Extensible is second-order: it is the property that composition
        // PRESERVES each answer-guarantee. It points AT the other five — it is
        // a meta-property over them, not a sibling of them.
        (Extensible, Verifiable, Preserves),
        (Extensible, Deterministic, Preserves),
        (Extensible, Explainable, Preserves),
        (Extensible, Honest, Preserves),
        (Extensible, Consistent, Preserves),
    ],
}

/// The five constitutional guarantees, as a concept.
///
/// Alias of the macro-generated `ConstitutionConcept`, named for the role it
/// plays in [`super::GuaranteeTag`]: every test declares the `Guarantee` it
/// witnesses.
pub type Guarantee = ConstitutionConcept;

use crate::category::{Arrow, Category};
use crate::ontology::Axiom;

/// The keystone, made checkable: `Honest` grounds every other *answer*-guarantee.
///
/// Verifiable, Deterministic, Explainable and Consistent are credible only
/// because the system can refuse — leave a claim ungrounded rather than
/// confabulate. The ontology encodes that as a `Grounds` edge from `Honest`
/// to each of the four; this axiom verifies every such edge is present, so
/// the keystone is a structural fact about the Constitution, not prose.
/// (`Extensible` is excluded by design — it is second-order, a property over
/// these answer-guarantees rather than one of them; see
/// [`ExtensiblePreservesEveryGuarantee`].)
///
/// Grounded in Lakatos (1976): refutation is constitutive of knowledge, so a
/// system that cannot refuse cannot hold its other guarantees as invariants.
pub struct HonestGroundsEveryGuarantee;

/// The five answer-guarantees: the properties of a single answer that `Honest`
/// grounds and that `Extensible` preserves under composition.
const ANSWER_GUARANTEES: [ConstitutionConcept; 5] = [
    ConstitutionConcept::Verifiable,
    ConstitutionConcept::Deterministic,
    ConstitutionConcept::Explainable,
    ConstitutionConcept::Honest,
    ConstitutionConcept::Consistent,
];

impl Axiom for HonestGroundsEveryGuarantee {
    fn verify(&self) -> crate::logic::proof::Verdict {
        use crate::logic::proof::{SimpleCounterexample, SimpleProof};
        // Honest grounds the OTHER four answer-guarantees (not itself).
        let all_grounded = ANSWER_GUARANTEES
            .iter()
            .filter(|g| **g != ConstitutionConcept::Honest)
            .all(|target| {
                ConstitutionCategory::morphisms()
                    .iter()
                    .any(|m| m.source() == ConstitutionConcept::Honest && m.target() == *target)
            });
        if all_grounded {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "HonestGroundsEveryGuarantee",
        "Honest grounds Verifiable, Deterministic, Explainable and Consistent — without the capacity to refuse, the other answer-guarantees are preferences, not invariants.",
        "Lakatos (1976) Proofs and Refutations, Cambridge University Press"
    );
}
pr4xis::register_axiom!(HonestGroundsEveryGuarantee, constructor);

/// `Extensible`, modeled correctly: it is second-order — the property that
/// composition PRESERVES every answer-guarantee.
///
/// Extensible is not a peer of the five; it is a meta-property *over* them. The
/// ontology encodes this as a `Preserves` edge from `Extensible` to each
/// answer-guarantee (Extensible points AT the others, where the others point at
/// each other). This axiom verifies that meta-structure is present: every
/// answer-guarantee is something composition is claimed to preserve. The claim
/// is *discharged* operationally by the workspace's functor-law checks
/// (`check_functor_laws`, the law-side of ruling #9) — a law-checked functor is
/// exactly a composition that does not degrade the guarantees.
///
/// Grounded in Spivak (2014): functorial composition; and the
/// verified-component-composition result that proven parts assemble into a
/// proven whole.
pub struct ExtensiblePreservesEveryGuarantee;

impl Axiom for ExtensiblePreservesEveryGuarantee {
    fn verify(&self) -> crate::logic::proof::Verdict {
        use crate::logic::proof::{SimpleCounterexample, SimpleProof};
        let all_preserved = ANSWER_GUARANTEES.iter().all(|target| {
            ConstitutionCategory::morphisms()
                .iter()
                .any(|m| m.source() == ConstitutionConcept::Extensible && m.target() == *target)
        });
        if all_preserved {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ExtensiblePreservesEveryGuarantee",
        "Extensible is second-order: it preserves Verifiable, Deterministic, Explainable, Honest and Consistent under composition — a meta-property over the answer-guarantees, not a peer.",
        "Spivak (2014) Category Theory for the Sciences, MIT Press — functorial composition"
    );
}
pr4xis::register_axiom!(ExtensiblePreservesEveryGuarantee, constructor);

/// The `Consistent` guarantee, made checkable: the whole registered axiom base
/// derives no contradiction.
///
/// This is the structural backing for the `Consistent` value, not a slogan:
/// it folds the entire `AXIOM_CONSTRUCTORS` registry — every axiom every
/// ontology registered, including the catalog's structural axioms (taxonomy
/// acyclicity, opposition irreflexivity) — and verifies each one holds. If the
/// corpus ever proved a thing and its negation, a registered axiom would fail
/// and this returns a counterexample. It is a *universal* check over the base,
/// not a sample, so `Consistent` is enforced the moment it is named.
///
/// Self-excluding (it skips its own entry) so the fold does not recurse.
/// Grounded in Gentzen (1936): a theory is consistent iff it proves no `⊥`.
pub struct OntologyBaseIsConsistent;

impl Axiom for OntologyBaseIsConsistent {
    fn verify(&self) -> crate::logic::proof::Verdict {
        use crate::logic::proof::SimpleProof;
        // The registry is native-only (linkme is unsupported on wasm32, where it
        // is empty); there the check is vacuous.
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::logic::proof::SimpleCounterexample;
            let me = self.name();
            for axiom in crate::ontology::axiom_constructors() {
                if axiom.name() == me {
                    continue; // skip self: the fold must not run itself
                }
                if axiom.verify().is_err() {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "OntologyBaseIsConsistent",
        "Every registered axiom across the whole ontology base holds — the corpus derives no contradiction.",
        "Gentzen (1936) Die Widerspruchsfreiheit der reinen Zahlentheorie, Math. Annalen 112"
    );
}
pr4xis::register_axiom!(OntologyBaseIsConsistent, constructor);

/// The `Explainable` guarantee, made checkable: every verdict carries its
/// explanation.
///
/// In pr4xis every reasoning step is an [`Axiom`] whose `verify()` returns a
/// [`Verdict`](crate::logic::proof::Verdict) — a `Proof` or `Counterexample`
/// that carries a [`Provenance`](crate::ontology::meta::Provenance): the
/// axiom's name (*what* was checked), description (*why* it holds) and citation
/// (*the source*). The proof object **is** the machine-readable explanation, so
/// "the reasoning path is the answer" is structural, not aspirational.
///
/// This axiom folds the (cheap, metadata-only) axiom registry and verifies that
/// *every* registered axiom's explanation is complete — none is named without
/// saying what it proves and where the claim comes from. Grounded in
/// Martin-Löf (1984): a proof term is its own explanation.
pub struct EveryAxiomCarriesItsExplanation;

impl Axiom for EveryAxiomCarriesItsExplanation {
    fn verify(&self) -> crate::logic::proof::Verdict {
        use crate::logic::proof::SimpleProof;
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::logic::proof::SimpleCounterexample;
            for p in crate::ontology::describe_axioms() {
                if p.name.as_str().is_empty()
                    || p.description.as_str().is_empty()
                    || p.citation.as_str().is_empty()
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "EveryAxiomCarriesItsExplanation",
        "Every registered axiom's verdict carries a complete explanation — name, what it proves, and citation — so the reasoning path is itself the answer.",
        "Martin-Löf (1984) Intuitionistic Type Theory, Bibliopolis — a proof term is its own explanation"
    );
}
pr4xis::register_axiom!(EveryAxiomCarriesItsExplanation, constructor);

/// Determinism enables verification and explanation.
///
/// A result that is reproducible can be checked (Verifiable) and its
/// derivation re-walked (Explainable). The ontology encodes this as `Enables`
/// edges from `Deterministic`; this axiom verifies both are present.
pub struct DeterminismEnablesVerificationAndExplanation;

impl Axiom for DeterminismEnablesVerificationAndExplanation {
    fn verify(&self) -> crate::logic::proof::Verdict {
        use crate::logic::proof::{SimpleCounterexample, SimpleProof};
        let enabled = [
            ConstitutionConcept::Verifiable,
            ConstitutionConcept::Explainable,
        ];
        let all_enabled = enabled.iter().all(|target| {
            ConstitutionCategory::morphisms()
                .iter()
                .any(|m| m.source() == ConstitutionConcept::Deterministic && m.target() == *target)
        });
        if all_enabled {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "DeterminismEnablesVerificationAndExplanation",
        "Deterministic enables Verifiable and Explainable: a reproducible result can be checked and its derivation re-walked.",
        "Knuth (1997) The Art of Computer Programming, Vol. 1, §1.1 (definiteness)"
    );
}
pr4xis::register_axiom!(DeterminismEnablesVerificationAndExplanation, constructor);

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn honest_grounds_every_guarantee_holds() {
        assert!(
            HonestGroundsEveryGuarantee.verify().is_ok(),
            "the keystone edge (Honest Grounds each other guarantee) is missing",
        );
    }

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn determinism_enables_verification_and_explanation_holds() {
        assert!(
            DeterminismEnablesVerificationAndExplanation
                .verify()
                .is_ok(),
            "the Deterministic Enables {{Verifiable, Explainable}} edges are missing",
        );
    }
}
