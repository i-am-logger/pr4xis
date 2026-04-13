use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Repository / Store ontology — where and how ontologies are persisted.
//
// A Repository is the abstract interface to stored ontologies.
// A Store is the pluggable physical backend.
// Materialize = ontology → stored form. Realize = stored form → live ontology.
//
// The same ontology can live in different stores (codegen binary, mmap file,
// heap memory, database, HTTP endpoint). All stores produce naturally
// isomorphic instances — the store is transparent to the ontology.
//
// References:
// - RDF4J: Repository / Sail architecture (Eclipse Foundation)
// - Jena TDB: Dataset / Store model (Apache Foundation)
// - W3C: Graph Store HTTP Protocol, SPARQL 1.1 (2013)
// - Spivak, "Functorial Data Migration" (2012) — instance functors to different targets
// - OMG MDA v2.0 (2014) — PIM → PSM model transformation
// - Database theory: materialized views (Gupta & Mumick, 1995)

/// Concepts in the Repository ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum RepositoryConcept {
    /// The abstract interface to stored ontologies.
    /// RDF4J (2004): "A Repository is the main access point."
    Repository,

    /// A pluggable physical backend for storage.
    /// Jena: TDB Store. RDF4J: Sail (Storage and Inference Layer).
    Store,

    /// A specific schema+instance stored in a repository.
    /// W3C: Named Graph. Spivak: a specific instance functor.
    StoredOntology,

    /// The act of converting live ontology → stored form.
    /// DB theory: materialization (Gupta & Mumick, 1995).
    /// MDA: model transformation PIM → PSM.
    Materialize,

    /// The act of loading stored form → live ontology.
    /// MDA: realization. CQL: Presentation → Algebra evaluation.
    Realize,

    /// The proof that two stores contain the same ontology.
    /// Category theory: natural isomorphism between instance functors.
    /// MDA: semantic preservation.
    Equivalence,

    /// Static materialization — compiled into binary at build time.
    /// Analogy: ahead-of-time (AOT) compilation.
    /// Load: 0s. Mutable: no. Hot reload: no.
    StaticStore,

    /// Memory-mapped file store — OS manages paging.
    /// SNIA: NVM.PM.FILE (DAX mode).
    /// Load: ~2ms. Mutable: via msync. Hot reload: yes.
    MappedStore,

    /// Heap-allocated in-memory store — full runtime flexibility.
    /// Spivak: instance functor I: C → Set (landing in Rust heap).
    /// Load: varies. Mutable: yes. Hot reload: yes.
    HeapStore,

    /// Persistent database store — queryable, transactional.
    /// ACID guarantees (Haerder & Reuter, 1983).
    /// Load: query time. Mutable: yes. Hot reload: yes.
    DatabaseStore,

    /// Remote endpoint store — served over network.
    /// W3C: SPARQL Endpoint. REST API.
    /// Load: network latency. Mutable: depends. Hot reload: yes.
    EndpointStore,
}

define_ontology! {
    pub StorageOntology for RepositoryCategory {
        concepts: RepositoryConcept,
        relation: RepositoryRelation,
        kind: RepositoryRelationKind,
        kinds: [
            /// Repository contains Stores.
            Contains,
            /// Store holds StoredOntology.
            Holds,
            /// Materialize: live ontology → StoredOntology in a Store.
            Materializes,
            /// Realize: StoredOntology → live ontology from a Store.
            Realizes,
            /// Equivalence proves two StoredOntologies are isomorphic.
            Proves,
            /// Store specializes to specific backend (is-a).
            SpecializesTo,
            /// Materialize∘Realize = identity (roundtrip).
            Roundtrip,
        ],
        edges: [
            // Repository contains Stores
            (Repository, Store, Contains),
            // Store holds StoredOntology
            (Store, StoredOntology, Holds),
            // Materialize and Realize are the key operations
            (Materialize, StoredOntology, Materializes),
            (Realize, StoredOntology, Realizes),
            // Equivalence proves isomorphism between stored ontologies
            (Equivalence, StoredOntology, Proves),
            // Store backend specializations (taxonomy)
            (StaticStore, Store, SpecializesTo),
            (MappedStore, Store, SpecializesTo),
            (HeapStore, Store, SpecializesTo),
            (DatabaseStore, Store, SpecializesTo),
            (EndpointStore, Store, SpecializesTo),
            // Materialize∘Realize = identity (the roundtrip axiom)
            (Materialize, Realize, Roundtrip),
        ],
        composed: [
            // Repository → StoredOntology (through Store)
            (Repository, StoredOntology),
        ],
    }
}

/// Whether a store backend supports hot reload.
#[derive(Debug, Clone)]
pub struct SupportsHotReload;

impl Quality for SupportsHotReload {
    type Individual = RepositoryConcept;
    type Value = bool;

    fn get(&self, individual: &RepositoryConcept) -> Option<bool> {
        match individual {
            RepositoryConcept::StaticStore => Some(false),
            RepositoryConcept::MappedStore => Some(true),
            RepositoryConcept::HeapStore => Some(true),
            RepositoryConcept::DatabaseStore => Some(true),
            RepositoryConcept::EndpointStore => Some(true),
            _ => None,
        }
    }
}

impl Ontology for StorageOntology {
    type Cat = RepositoryCategory;
    type Qual = SupportsHotReload;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<RepositoryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        StorageOntology::validate().unwrap();
    }

    #[test]
    fn has_eleven_concepts() {
        assert_eq!(RepositoryConcept::variants().len(), 11);
    }

    // --- RDF4J: Repository contains Stores ---

    #[test]
    fn repository_contains_stores() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Repository
            && r.to == RepositoryConcept::Store
            && r.kind == RepositoryRelationKind::Contains));
    }

    // --- Store holds StoredOntology ---

    #[test]
    fn store_holds_stored_ontology() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Store
            && r.to == RepositoryConcept::StoredOntology
            && r.kind == RepositoryRelationKind::Holds));
    }

    // --- Five store backends, all specialize Store ---

    #[test]
    fn five_store_backends() {
        let m = RepositoryCategory::morphisms();
        let backends = [
            RepositoryConcept::StaticStore,
            RepositoryConcept::MappedStore,
            RepositoryConcept::HeapStore,
            RepositoryConcept::DatabaseStore,
            RepositoryConcept::EndpointStore,
        ];
        for backend in backends {
            assert!(
                m.iter().any(|r| r.from == backend
                    && r.to == RepositoryConcept::Store
                    && r.kind == RepositoryRelationKind::SpecializesTo),
                "{backend:?} should specialize Store"
            );
        }
    }

    // --- Gupta & Mumick (1995): Materialize produces StoredOntology ---

    #[test]
    fn materialize_produces_stored_ontology() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Materialize
            && r.to == RepositoryConcept::StoredOntology
            && r.kind == RepositoryRelationKind::Materializes));
    }

    // --- MDA: Realize loads StoredOntology ---

    #[test]
    fn realize_loads_stored_ontology() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Realize
            && r.to == RepositoryConcept::StoredOntology
            && r.kind == RepositoryRelationKind::Realizes));
    }

    // --- Spivak: natural isomorphism between instance functors ---

    #[test]
    fn equivalence_proves_stored_ontology_isomorphism() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Equivalence
            && r.to == RepositoryConcept::StoredOntology
            && r.kind == RepositoryRelationKind::Proves));
    }

    // --- Roundtrip: Materialize∘Realize = identity ---

    #[test]
    fn materialize_realize_roundtrip() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Materialize
            && r.to == RepositoryConcept::Realize
            && r.kind == RepositoryRelationKind::Roundtrip));
    }

    // --- Composition: Repository reaches StoredOntology ---

    #[test]
    fn repository_reaches_stored_ontology() {
        let m = RepositoryCategory::morphisms();
        assert!(m.iter().any(|r| r.from == RepositoryConcept::Repository
            && r.to == RepositoryConcept::StoredOntology
            && r.kind == RepositoryRelationKind::Composed));
    }
}
