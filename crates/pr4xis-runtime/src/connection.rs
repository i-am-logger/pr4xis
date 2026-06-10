//! The connection — the uniform serialized morphism.
//!
//! One node subsumes functor / adjunction / lens / natural transformation: they
//! differ only in `kind` (a name resolved against the meta-ontology), the
//! [`GeneratorAction`] variant, and the `laws` they must satisfy — never in
//! code. By the finite-presentation theorem (Lawvere functorial semantics), a
//! structure-preserving map out of a finitely-presented category is fully
//! determined by — and recoverable from — its finite action on generators; on
//! load the action is replayed by folding generator-images through the target's
//! composition (a `FreeExtension`), so no executable code is carried.
//!
//! The address is definition-bearing and cycle-safe: `source`/`target` are
//! names bound by the ontology's Merkle root; the recursive form (where this
//! address depends on the targets' own addresses) is the Merkle-DAG layer.

use serde::{Deserialize, Serialize};

use crate::address::ContentAddress;
use crate::codec::{self, CodecError};

/// The finite action-on-generators that *is* a connection. The variant names
/// the categorical family; every table is `(source-generator name → target
/// expression)`, which is why one representation serves all of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratorAction {
    /// A functor: where each source generating object and relation-kind goes.
    Functor {
        /// `(source object name, target expression)`, one row per generator.
        map_object: Vec<(String, String)>,
        /// `(source relation-kind name, target kind expression)`.
        map_morphism: Vec<(String, String)>,
    },
    /// A natural transformation `η: F ⇒ G`: one component per object,
    /// `η_x : F(x) → G(x)`.
    NaturalTransformation {
        /// `(object name, component-morphism expression)`.
        components: Vec<(String, String)>,
    },
    /// A bidirectional lens (Foster et al. 2007): a `view` focus with `get`/
    /// `put` morphisms. The `.prx` decompile round-trip is itself a lens.
    Lens {
        view: String,
        get: String,
        put: String,
    },
    /// An adjunction `F ⊣ G`: the two functors' object maps plus the unit `η`
    /// and counit `ε` component families.
    Adjunction {
        left_map_object: Vec<(String, String)>,
        right_map_object: Vec<(String, String)>,
        unit: Vec<(String, String)>,
        counit: Vec<(String, String)>,
    },
}

impl GeneratorAction {
    /// Sort every table so the encoding is order-independent.
    fn canonicalize(&mut self) {
        fn sort(v: &mut Vec<(String, String)>) {
            v.sort();
            v.dedup();
        }
        match self {
            GeneratorAction::Functor {
                map_object,
                map_morphism,
            } => {
                sort(map_object);
                sort(map_morphism);
            }
            GeneratorAction::NaturalTransformation { components } => sort(components),
            GeneratorAction::Lens { .. } => {}
            GeneratorAction::Adjunction {
                left_map_object,
                right_map_object,
                unit,
                counit,
            } => {
                sort(left_map_object);
                sort(right_map_object);
                sort(unit);
                sort(counit);
            }
        }
    }
}

/// The uniform serialized morphism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    /// The specific kind, resolved against the meta-ontology (e.g.
    /// `"FullyFaithful"`, `"FreeForgetful"`). The action variant names the
    /// family; `kind` refines it.
    pub kind: String,
    /// The source category/ontology name.
    pub source: String,
    /// The target category/ontology name.
    pub target: String,
    /// The finite action-on-generators that IS this connection.
    pub action: GeneratorAction,
    /// Names of the laws this connection must satisfy (functor laws, the
    /// triangle identities, the lens laws, …) — carried uniformly as data.
    pub laws: Vec<String>,
}

impl Connection {
    /// The content address of this connection — its definition-bearing identity,
    /// over the canonical (sorted) action + laws.
    pub fn address(&self) -> Result<ContentAddress, CodecError> {
        let mut canon = self.clone();
        canon.action.canonicalize();
        canon.laws.sort();
        canon.laws.dedup();
        codec::address_of(&canon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn functor() -> Connection {
        Connection {
            kind: "FullyFaithful".into(),
            source: "OntologyArchive".into(),
            target: "PraxisKnowledgeGraph".into(),
            action: GeneratorAction::Functor {
                map_object: vec![("ContentAddressableNode".into(), "ConceptNode".into())],
                map_morphism: vec![("Subsumption".into(), "RelationEdge".into())],
            },
            laws: vec!["PreservesComposition".into(), "PreservesIdentity".into()],
        }
    }

    #[test]
    fn identical_connections_share_an_address() {
        assert_eq!(functor().address().unwrap(), functor().address().unwrap());
    }

    #[test]
    fn changing_the_action_changes_the_address() {
        let mut b = functor();
        if let GeneratorAction::Functor { map_object, .. } = &mut b.action {
            map_object.push(("MerkleEdge".into(), "RelationEdge".into()));
        }
        assert_ne!(functor().address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn action_table_order_does_not_affect_address() {
        let mut a = functor();
        let mut b = functor();
        if let GeneratorAction::Functor { map_object, .. } = &mut a.action {
            map_object.push(("A".into(), "X".into()));
            map_object.push(("B".into(), "Y".into()));
        }
        if let GeneratorAction::Functor { map_object, .. } = &mut b.action {
            map_object.push(("B".into(), "Y".into())); // reversed
            map_object.push(("A".into(), "X".into()));
        }
        assert_eq!(a.address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn distinct_families_get_distinct_addresses() {
        let mut l = functor();
        l.action = GeneratorAction::Lens {
            view: "Source".into(),
            get: "parse".into(),
            put: "generate".into(),
        };
        assert_ne!(functor().address().unwrap(), l.address().unwrap());
    }

    #[test]
    fn changing_the_kind_changes_the_address() {
        let mut b = functor();
        b.kind = "Faithful".into();
        assert_ne!(functor().address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn every_family_addresses() {
        for action in [
            GeneratorAction::Functor {
                map_object: vec![("a".into(), "b".into())],
                map_morphism: vec![],
            },
            GeneratorAction::NaturalTransformation {
                components: vec![("x".into(), "eta_x".into())],
            },
            GeneratorAction::Lens {
                view: "v".into(),
                get: "g".into(),
                put: "p".into(),
            },
            GeneratorAction::Adjunction {
                left_map_object: vec![],
                right_map_object: vec![],
                unit: vec![],
                counit: vec![],
            },
        ] {
            let mut c = functor();
            c.action = action;
            assert!(c.address().is_ok());
        }
    }
}
