//! Definition-bearing addressing — a node's identity is the content address of
//! its DEFINITION, not its name.
//!
//! This is the head of the build spine: it closes the **G5 wire gap**. Under
//! name-only addressing (`hash(kind, name)`) two nodes that share a name but
//! differ in structure collide to the same address, so peers could "agree" on a
//! node while meaning different things. Addressing the *definition* — kind +
//! name + outgoing edges + governing axioms + lexical grounding — makes
//! agreement mean agreement on what the node actually IS.
//!
//! References to other nodes (edge targets, axioms) are by NAME at this layer,
//! resolved within the node's own ontology and bound by the ontology's Merkle
//! root. The stronger *recursive* form — where this address depends on the
//! targets' own addresses — is the Merkle-DAG layer, which must additionally
//! handle cyclic graphs (e.g. symmetric `opposition`) via strongly-connected-
//! component hashing. This module is the cycle-safe floor that layer builds on.

use serde::{Deserialize, Serialize};

use crate::address::ContentAddress;
use crate::codec::{self, CodecError};

/// The canonical definition of a node — the structure its content-address is
/// taken over. Field order is the serialized order; the `edges`/`axioms` rows
/// are sorted + deduplicated by [`Definition::address`] before encoding, so the
/// address does not depend on assembly order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    /// The node's kind — a name resolved against the meta-ontology
    /// (e.g. `"Concept"`, `"Functor"`, `"Lens"`).
    pub kind: String,
    /// The node's name within its ontology.
    pub name: String,
    /// Outgoing typed edges, each `(relation-kind name, target name)`.
    pub edges: Vec<(String, String)>,
    /// Names of the axioms that constrain this node.
    pub axioms: Vec<String>,
    /// The node's lexical grounding (canonical English form / Lemon entry), if
    /// any — part of the definition because in praxis "everything is Lemon".
    pub lexical: Option<String>,
}

impl Definition {
    /// The content address of this node — its definition-bearing identity.
    ///
    /// Canonical: the `edges` and `axioms` rows are sorted + deduplicated before
    /// the DAG-CBOR encoding, so two definitions with the same content assembled
    /// in different orders share the same address.
    pub fn address(&self) -> Result<ContentAddress, CodecError> {
        let mut canon = self.clone();
        canon.edges.sort();
        canon.edges.dedup();
        canon.axioms.sort();
        canon.axioms.dedup();
        codec::address_of(&canon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Definition {
        Definition {
            kind: "Concept".into(),
            name: "Employer".into(),
            edges: vec![("Subsumption".into(), "Agent".into())],
            axioms: vec!["EmployerIsAgent".into()],
            lexical: Some("employer".into()),
        }
    }

    #[test]
    fn identical_definitions_share_an_address() {
        assert_eq!(base().address().unwrap(), base().address().unwrap());
    }

    #[test]
    fn changing_an_edge_changes_the_address() {
        let mut b = base();
        b.edges = vec![("Subsumption".into(), "Person".into())]; // different target
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn changing_an_axiom_changes_the_address() {
        let mut b = base();
        b.axioms = vec!["EmployerHiresEmployee".into()];
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn changing_the_lexical_changes_the_address() {
        let mut b = base();
        b.lexical = Some("boss".into());
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn same_name_different_definition_does_not_collide() {
        // The G5 fix: same name, different structure → different address.
        let mut b = base();
        b.edges.push(("Opposition".into(), "Employee".into()));
        assert_ne!(base().address().unwrap(), b.address().unwrap());
    }

    #[test]
    fn address_is_order_independent() {
        let mut a = base();
        a.edges = vec![
            ("Subsumption".into(), "Agent".into()),
            ("Opposition".into(), "Employee".into()),
        ];
        a.axioms = vec!["B".into(), "A".into()];
        let mut b = base();
        b.edges = vec![
            ("Opposition".into(), "Employee".into()),
            ("Subsumption".into(), "Agent".into()),
        ];
        b.axioms = vec!["A".into(), "B".into()];
        assert_eq!(a.address().unwrap(), b.address().unwrap());
    }
}
