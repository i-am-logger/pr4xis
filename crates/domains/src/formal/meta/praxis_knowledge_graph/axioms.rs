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
//! - [`LensLawPreservation`] (delegates to the lens harness over every
//!   registered lens), [`SelectionClosedUnderEdgeKinds`] (BFS reachability
//!   over the macro-emitted `morphisms()`), and [`PairOntologyRoundTrip`]
//!   (the archive-subset `(data, binding)` pair round-trip).
//!
//! ## Deferred, deliberately NOT declared here (no-stub doctrine)
//!
//! `AdjunctionTrianglePreservation` waits on an adjunction in the graph's
//! binding set (the `AdjunctionTriangleLaw` machinery already exists in
//! `pr4xis::category::laws`); `GraphSnapshotReproducible` + the whole-graph
//! leg of `PairOntologyRoundTrip` wait on the #271 snapshot machinery;
//! `AttestationChainVerifiable` + `IntegrityClaimVerifiable` wait on
//! TUF/in-toto/SLSA. A passing `verify()` for any of these today would be a
//! stub.

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use pr4xis::category::Category;
use pr4xis::category::laws::{fully_faithful_law_axioms, functor_law_axioms};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, axiom_by_name, axiom_constructors};

use super::functor::ArchiveIntoGraph;
use super::ontology::{
    PraxisKnowledgeGraphCategory, PraxisKnowledgeGraphConcept, PraxisKnowledgeGraphRelationKind,
};
use crate::formal::meta::ontology_archive::axioms as archive;
use crate::formal::meta::well_behaved_lens::harness::RoundTripHarnessAllVerified;

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

/// Every persisted `LensNode`'s get/put round-trip law holds on
/// rehydration — delegated to the lens harness, which dispatches each
/// registered lens on its `RoundTripFidelity` and checks its signature
/// against `praxis.lock`. Foster et al. (2007) §2.2.
pub struct LensLawPreservation;

impl Axiom for LensLawPreservation {
    fn verify(&self) -> Verdict {
        if RoundTripHarnessAllVerified.verify().is_err() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LensLawPreservation",
        "every persisted LensNode's get/put round-trip law holds on rehydration (the lens harness verifies all registered lenses)",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(LensLawPreservation, constructor);

/// A slice computed from `(roots, edge-kind filter)` contains every node
/// reachable from the roots through the chosen kinds, and no filtered edge
/// leaves it (a leaving reference would be an `UnboundReference`). Checked
/// by BFS over the macro-emitted `morphisms()` (a graph-reachability
/// invariant, in-memory, no #271 emit machinery).
pub struct SelectionClosedUnderEdgeKinds;

impl Axiom for SelectionClosedUnderEdgeKinds {
    fn verify(&self) -> Verdict {
        use PraxisKnowledgeGraphConcept as C;
        use PraxisKnowledgeGraphRelationKind as K;
        let ms = PraxisKnowledgeGraphCategory::morphisms();
        // Representative selection: root = MerkleRoot, filter = Subsumption.
        let root = C::MerkleRoot;
        let filter = K::Subsumption;
        let mut reachable = alloc::vec![root];
        let mut frontier = alloc::vec![root];
        while let Some(n) = frontier.pop() {
            for m in ms.iter().filter(|m| m.kind == filter && m.from == n) {
                if !reachable.contains(&m.to) {
                    reachable.push(m.to);
                    frontier.push(m.to);
                }
            }
        }
        // Closure — every filtered edge from an in-set node stays in-set.
        for m in ms.iter().filter(|m| m.kind == filter) {
            if reachable.contains(&m.from) && !reachable.contains(&m.to) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        // Non-vacuous — MerkleRoot reaches ContentAddressableNode through
        // Subsumption, but the slice is NOT the whole graph (SourcePin is
        // unreachable from MerkleRoot via is_a, so it is excluded).
        if !reachable.contains(&C::ContentAddressableNode) || reachable.contains(&C::SourcePin) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SelectionClosedUnderEdgeKinds",
        "a slice from (roots, edge-kind filter) contains every node reachable through those kinds, and no filtered edge leaves it",
        "graph-reachability invariant; Mac Lane (1971) Categories for the Working Mathematician Ch. I §1"
    );
}

pr4xis::register_axiom!(SelectionClosedUnderEdgeKinds, constructor);

/// The pair-ontology `(structural data, binding)` round-trips together for
/// the archive subset: the structural data survives the emit/load lens
/// ([`archive::EmitLoadWellBehaved`], Foster 2007 §2.2) and a behavioural
/// node's binding re-binds by name on the receiving side ([`axiom_by_name`])
/// — the composition the wire protocol's slice → emit → receive → re-bind
/// reduces to (the whole-graph leg awaits the #271 snapshot machinery).
pub struct PairOntologyRoundTrip;

impl Axiom for PairOntologyRoundTrip {
    fn verify(&self) -> Verdict {
        // Structural leg — the archived data round-trips through rkyv bytes.
        if archive::EmitLoadWellBehaved.verify().is_err() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Binding leg — a persisted axiom binding re-binds by its name.
        if axiom_by_name(AxiomBindingComplete.name().as_str()).is_none() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PairOntologyRoundTrip",
        "the (structural data, binding) pair round-trips together for the archive subset: data via the emit/load lens, binding via re-bind by name",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2; graph-rebind invariant"
    );
}

pr4xis::register_axiom!(PairOntologyRoundTrip, constructor);

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
        Box::new(LensLawPreservation),
        Box::new(SelectionClosedUnderEdgeKinds),
        Box::new(PairOntologyRoundTrip),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every domain axiom holds against the real machinery (the `.prx`
    /// realisation, the ArchiveIntoGraph functor, and the registries).
    #[test]
    fn all_domain_axioms_hold() {
        // Guard against a future vacuous AxiomBindingComplete: the re-bind
        // handler table must actually carry registrations to round-trip
        // (the three #272 axioms register via the constructor arm).
        assert!(
            !axiom_constructors().is_empty(),
            "AXIOM_CONSTRUCTORS must be populated for the re-bind axioms to have teeth"
        );
        for ax in domain_axioms() {
            ax.verify()
                .unwrap_or_else(|c| panic!("axiom failed: {}", c.meta().name.as_str()));
        }
    }
}
