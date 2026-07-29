//! The `.prx` archive — the serialized ontology as a content-addressed set of
//! definition-bearing nodes and the connections (morphisms) over them.
//!
//! The archive's **Merkle root** is the `.prx`'s identity: the content address
//! over the sorted set of every node's and connection's definition-bearing
//! address. Two archives are the same ontology iff their roots match — iff every
//! node and connection agrees by address. Insertion order and duplicates do not
//! affect it.
//!
//! This is the cycle-safe envelope: it roots over the already-computed *local*
//! addresses, so the cyclic graphs (`opposition`, …) that complicate per-node
//! *recursive* addressing never reach this layer. Promoting per-node addresses
//! to the recursive Merkle-DAG form (where a node's address depends on its
//! children's) is a separate step inside [`Definition`](crate::definition) /
//! [`Connection`](crate::connection); the envelope is unchanged by it.

use serde::{Deserialize, Serialize};

use crate::address::ContentAddress;
use crate::codec::{self, CodecError};
use crate::connection::Connection;
use crate::definition::Definition;

/// A `.prx` archive: the nodes and connections of one serialized ontology.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Archive {
    /// The node definitions (concepts, axioms, lenses, …).
    pub nodes: Vec<Definition>,
    /// The connections (morphisms) — functors / adjunctions / lenses / nat-trans.
    pub connections: Vec<Connection>,
}

impl Archive {
    /// An empty archive.
    pub fn new() -> Self {
        Self::default()
    }

    /// The Merkle root — the content address over the sorted, deduplicated set
    /// of every node's and connection's definition-bearing address.
    pub fn root(&self) -> Result<ContentAddress, CodecError> {
        let mut addrs: Vec<[u8; 32]> =
            Vec::with_capacity(self.nodes.len() + self.connections.len());
        for n in &self.nodes {
            addrs.push(*n.address()?.as_bytes());
        }
        for c in &self.connections {
            addrs.push(*c.address()?.as_bytes());
        }
        addrs.sort_unstable();
        addrs.dedup();
        codec::address_of(&addrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::GeneratorAction;

    fn node(name: &str) -> Definition {
        Definition {
            kind: "Concept".into(),
            name: name.into(),
            edges: vec![],
            axioms: vec![],
            lexical: None,
        }
    }

    fn conn() -> Connection {
        Connection {
            kind: "Faithful".into(),
            source: "A".into(),
            target: "B".into(),
            action: GeneratorAction::Functor {
                map_object: vec![("A".into(), "B".into())],
                map_morphism: vec![],
            },
            laws: vec![],
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn empty_archive_has_a_stable_root() {
        assert_eq!(
            Archive::new().root().unwrap(),
            Archive::new().root().unwrap()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn adding_a_node_changes_the_root() {
        let mut a = Archive::new();
        a.nodes.push(node("X"));
        assert_ne!(Archive::new().root().unwrap(), a.root().unwrap());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn root_is_node_order_independent() {
        let a = Archive {
            nodes: vec![node("X"), node("Y")],
            connections: vec![],
        };
        let b = Archive {
            nodes: vec![node("Y"), node("X")],
            connections: vec![],
        };
        assert_eq!(a.root().unwrap(), b.root().unwrap());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn duplicate_node_does_not_change_the_root() {
        let a = Archive {
            nodes: vec![node("X")],
            connections: vec![],
        };
        let b = Archive {
            nodes: vec![node("X"), node("X")],
            connections: vec![],
        };
        assert_eq!(a.root().unwrap(), b.root().unwrap());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_connection_contributes_to_the_root() {
        let a = Archive {
            nodes: vec![node("A"), node("B")],
            connections: vec![],
        };
        let mut b = a.clone();
        b.connections.push(conn());
        assert_ne!(a.root().unwrap(), b.root().unwrap());
    }
}
