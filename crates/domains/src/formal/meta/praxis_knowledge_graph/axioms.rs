//! Runnable axioms of the [`PraxisKnowledgeGraph`](super::ontology)
//! ontology — each `verify()` is a predicate that RUNS against real
//! machinery (the `.prx` realisation, the `ArchiveIntoGraph` functor, and
//! the registries), never a doc-comment claim
//! (`feedback_praxis_as_compiler_self_describing`).
//!
//! Gated on `feature = "prx"`, where the realisation and the storage
//! substratum exist.
//!
//! ## What ships in #272
//!
//! - The **seven storage axioms** of [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive),
//!   re-exported as graph axioms. The [`ArchiveIntoGraph`](super::functor)
//!   functor is the formal statement that they ARE graph axioms; its
//!   full-and-faithful law (`FunctorLawPreservation`) proves the carry-over
//!   is structure-preserving.
//! - [`FunctorLawPreservation`] — the embedding is functorial AND fully
//!   faithful (the machine proof behind `KIND = FullyFaithful`).
//! - [`AxiomBindingComplete`] + [`UnboundReferenceFailsClosed`] — the
//!   load-time re-bind handler table (`pr4xis::ontology::axiom_by_name`):
//!   every persisted `AxiomNode` re-binds to a live predicate, and an
//!   unresolvable binding fails closed.
//!
//! ## Deferred, deliberately NOT declared here (no-stub doctrine)
//!
//! `LensLawPreservation`, `SelectionClosedUnderEdgeKinds`, and the
//! archive-leg of `PairOntologyRoundTrip` are buildable against existing
//! machinery and land next; `AdjunctionTrianglePreservation` waits on an
//! adjunction in the graph's binding set (the `AdjunctionTriangleLaw`
//! machinery already exists in `pr4xis::category::laws`);
//! `GraphSnapshotReproducible` + the whole-graph leg wait on the #271
//! snapshot machinery; `AttestationChainVerifiable` + `IntegrityClaimVerifiable`
//! wait on TUF/in-toto/SLSA. A passing `verify()` for any of these today
//! would be a stub.

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use pr4xis::category::laws::{fully_faithful_law_axioms, functor_law_axioms};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, axiom_by_name, axiom_constructors};

use super::functor::ArchiveIntoGraph;
use crate::formal::meta::ontology_archive::axioms as archive;

/// The `ArchiveIntoGraph` embedding preserves functor structure AND is
/// fully faithful — identity + composition + faithful (injective on each
/// hom-set) + full onto its image. This is the machine proof that the
/// archive's persisted nodes and edges round-trip into the graph without
/// loss or collision, the carry-over witness for the seven re-exported
/// storage axioms. Mac Lane (1971) CWM Ch. I §3-4.
pub struct FunctorLawPreservation;

impl Axiom for FunctorLawPreservation {
    fn verify(&self) -> Verdict {
        let mut laws = functor_law_axioms::<ArchiveIntoGraph>();
        laws.extend(fully_faithful_law_axioms::<ArchiveIntoGraph>());
        for law in laws {
            if law.verify().is_err() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FunctorLawPreservation",
        "the ArchiveIntoGraph embedding preserves id/composition and is fully faithful (faithful + full onto image)",
        "Mac Lane (1971) Categories for the Working Mathematician Ch. I §3-4"
    );
}

pr4xis::register_axiom!(FunctorLawPreservation, constructor);

/// Every registered axiom constructor re-binds by its stable name:
/// reconstruct each axiom, resolve it through the handler table
/// ([`axiom_by_name`]), and confirm the resolved axiom carries the same
/// name. A persisted `AxiomNode` therefore re-binds to a live predicate on
/// load — no dangling unbound binding survives.
pub struct AxiomBindingComplete;

impl Axiom for AxiomBindingComplete {
    fn verify(&self) -> Verdict {
        for a in axiom_constructors() {
            let name = a.name();
            match axiom_by_name(name.as_str()) {
                Some(b) if b.name().as_str() == name.as_str() => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "AxiomBindingComplete",
        "every persisted AxiomNode re-binds to a registered axiom constructor by its stable name",
        "graph-rebind invariant; Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
    );
}

pr4xis::register_axiom!(AxiomBindingComplete, constructor);

/// A binding the receiver cannot resolve fails closed: resolving an
/// unregistered axiom name returns `None`, so the load gate refuses rather
/// than silently producing a partial graph.
pub struct UnboundReferenceFailsClosed;

impl Axiom for UnboundReferenceFailsClosed {
    fn verify(&self) -> Verdict {
        let unbound = "__praxis_unregistered_axiom_binding__";
        if axiom_by_name(unbound).is_some() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "UnboundReferenceFailsClosed",
        "an unresolvable binding identity resolves to None (fail-closed), never a partial graph",
        "load-gate discipline; Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
    );
}

pr4xis::register_axiom!(UnboundReferenceFailsClosed, constructor);

/// The runnable domain axioms of the knowledge-graph ontology — spliced
/// into [`PraxisKnowledgeGraphOntology::axioms`](super::ontology) under
/// `feature = "prx"`. The seven archive axioms are re-exported (the functor
/// is the formal carry-over); the three whole-graph axioms exercise the
/// embedding and the re-bind handler table.
pub fn domain_axioms() -> Vec<Box<dyn Axiom>> {
    vec![
        // The storage substratum (OntologyArchiveStorage), carried over by
        // the fully-faithful ArchiveIntoGraph functor.
        Box::new(archive::MerkleHashDeterministic),
        Box::new(archive::MerkleDedupCorrect),
        Box::new(archive::CompressionRoundTrip),
        Box::new(archive::RkyvDeterminism),
        Box::new(archive::EmitLoadWellBehaved),
        Box::new(archive::SourceHashFaithfulness),
        Box::new(archive::LoadGateFailsClosed),
        // Whole-graph axioms.
        Box::new(FunctorLawPreservation),
        Box::new(AxiomBindingComplete),
        Box::new(UnboundReferenceFailsClosed),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every domain axiom holds against the real machinery (the `.prx`
    /// realisation, the ArchiveIntoGraph functor, and the registries).
    #[test]
    fn all_domain_axioms_hold() {
        for ax in domain_axioms() {
            ax.verify()
                .unwrap_or_else(|c| panic!("axiom failed: {}", c.meta().name.as_str()));
        }
    }
}
