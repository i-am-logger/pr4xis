//! Storage — repository, store, and the materialise/realise pair.
//!
//! A Repository is the abstract interface to stored ontologies. A Store
//! is the pluggable physical backend. `Materialize` converts a live
//! ontology into stored form; `Realize` loads stored form back into a
//! live ontology. Their roundtrip is identity (Gupta & Mumick 1995).
//!
//! # Literature
//!
//! - **RDF4J (Eclipse Foundation)** — Repository / Sail (Storage and
//!   Inference Layer) architecture. The Repository concept enters
//!   pr4xis from this lineage.
//! - **Spivak (2012)** "Functorial Data Migration", *Information and
//!   Computation* 217:31-51 — different stores produce naturally
//!   isomorphic instance functors.
//! - **W3C (2013)** *SPARQL 1.1 Graph Store HTTP Protocol* — endpoint
//!   stores.
//! - **OMG (2014)** *MDA Guide v2.0* — PIM → PSM model
//!   transformation (the materialize/realize pair).
//! - **Gupta & Mumick (1995)** "Maintenance of Materialized Views:
//!   Problems, Techniques, and Applications", *IEEE Data Engineering
//!   Bulletin* 18(2):3-18 — materialisation theory.
//! - **Haerder & Reuter (1983)** "Principles of Transaction-Oriented
//!   Database Recovery", *Computing Surveys* 15(4):287-317 — ACID for
//!   DatabaseStore.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Storage",
    source: "RDF4J Repository/Sail architecture (Eclipse Foundation); Spivak (2012) Functorial Data Migration, Information and Computation 217:31-51; W3C (2013) SPARQL 1.1 Graph Store HTTP Protocol; OMG (2014) MDA Guide v2.0; Gupta & Mumick (1995) Maintenance of Materialized Views, IEEE Data Engineering Bulletin 18(2):3-18; Haerder & Reuter (1983) Principles of Transaction-Oriented Database Recovery, Computing Surveys 15(4):287-317",

    concepts: [
        Repository,
        Store,
        StoredOntology,
        Materialize,
        Realize,
        Equivalence,
        StaticStore,
        MappedStore,
        HeapStore,
        DatabaseStore,
        EndpointStore,
    ],

    labels: {
        Repository: ("en", "Repository",
            "RDF4J: the abstract interface to stored ontologies - the main access point."),
        Store: ("en", "Store",
            "RDF4J Sail / Jena TDB Store: a pluggable physical backend for storage."),
        StoredOntology: ("en", "Stored ontology",
            "W3C Named Graph; Spivak (2012): a specific schema + instance held in a store."),
        Materialize: ("en", "Materialize",
            "Gupta & Mumick (1995): the act of converting a live ontology into stored form. MDA: PIM → PSM."),
        Realize: ("en", "Realize",
            "OMG MDA (2014): the act of loading stored form back into a live ontology. CQL: Presentation → Algebra evaluation."),
        Equivalence: ("en", "Equivalence",
            "Spivak (2012): a natural isomorphism between two stored ontologies - proof that two stores hold the same content."),
        StaticStore: ("en", "Static store",
            "AOT-compiled into the binary. Load: 0s. Mutable: no. Hot reload: no."),
        MappedStore: ("en", "Mapped store",
            "Memory-mapped file. SNIA NVM Programming Model (2017) NVM.PM.FILE DAX. Load ~2ms. Mutable via msync."),
        HeapStore: ("en", "Heap store",
            "Heap-allocated in-memory store. Spivak (2012): I: C → Set landing in heap. Mutable; hot reload."),
        DatabaseStore: ("en", "Database store",
            "Persistent transactional store with ACID guarantees (Haerder & Reuter 1983)."),
        EndpointStore: ("en", "Endpoint store",
            "W3C SPARQL endpoint or REST API. Network latency; depends-on remote mutability."),
    },

    is_a: [
        // Store backend specialisations.
        (StaticStore, Store),
        (MappedStore, Store),
        (HeapStore, Store),
        (DatabaseStore, Store),
        (EndpointStore, Store),
    ],

    edges: [
        // RDF4J: Repository contains Stores.
        (Repository, Store, Contains),
        // Store holds StoredOntology.
        (Store, StoredOntology, Holds),
        // Materialise and Realise are the key operations.
        (Materialize, StoredOntology, Materializes),
        (Realize, StoredOntology, Realizes),
        // Spivak (2012): Equivalence proves isomorphism.
        (Equivalence, StoredOntology, Proves),
        // Gupta & Mumick (1995): Materialize ∘ Realize = identity.
        (Materialize, Realize, Roundtrip),
    ],
}

/// Quality: whether a store backend supports hot reload (live update of
/// the stored ontology without restart). StaticStore is the only
/// no-hot-reload backend; all others permit live updates.
#[derive(Debug, Clone)]
pub struct SupportsHotReload;

impl Quality for SupportsHotReload {
    type Individual = StorageConcept;
    type Value = bool;

    fn get(&self, c: &StorageConcept) -> Option<bool> {
        use StorageConcept as S;
        match c {
            S::StaticStore => Some(false),
            S::MappedStore | S::HeapStore | S::DatabaseStore | S::EndpointStore => Some(true),
            _ => None,
        }
    }
}

impl Ontology for StorageOntology {
    type Cat = StorageCategory;
    type Qual = SupportsHotReload;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

// Re-export legacy names.
pub type RepositoryConcept = StorageConcept;
pub type RepositoryCategory = StorageCategory;
pub type RepositoryRelation = StorageRelation;
pub type RepositoryRelationKind = StorageRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<StorageCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        StorageOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eleven_concepts() {
        assert_eq!(StorageConcept::variants().len(), 11);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn repository_contains_stores() {
        let m = StorageCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == StorageConcept::Repository
            && r.target() == StorageConcept::Store
            && r.kind() == StorageRelationKind::Contains));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_store_backends_specialize_store() {
        let m = StorageCategory::morphisms();
        let backends = [
            StorageConcept::StaticStore,
            StorageConcept::MappedStore,
            StorageConcept::HeapStore,
            StorageConcept::DatabaseStore,
            StorageConcept::EndpointStore,
        ];
        for backend in backends {
            assert!(
                m.iter().any(|r| r.source() == backend
                    && r.target() == StorageConcept::Store
                    && r.kind() == StorageRelationKind::Subsumption),
                "{backend:?} should subsume Store"
            );
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn materialize_realize_roundtrip() {
        // Gupta & Mumick (1995).
        let m = StorageCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == StorageConcept::Materialize
            && r.target() == StorageConcept::Realize
            && r.kind() == StorageRelationKind::Roundtrip));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn static_no_hot_reload() {
        assert_eq!(
            SupportsHotReload.get(&StorageConcept::StaticStore),
            Some(false)
        );
        assert_eq!(
            SupportsHotReload.get(&StorageConcept::HeapStore),
            Some(true)
        );
    }

    fn arb_concept() -> impl Strategy<Value = StorageConcept> {
        proptest::sample::select(StorageConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in StorageCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in StorageOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_hot_reload_partial(c in arb_concept()) {
            use StorageConcept as S;
            let v = SupportsHotReload.get(&c);
            let is_backend = matches!(c,
                S::StaticStore | S::MappedStore | S::HeapStore
                | S::DatabaseStore | S::EndpointStore);
            prop_assert_eq!(v.is_some(), is_backend);
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_hot_reload_partial, Honest, Verifiable);
}
