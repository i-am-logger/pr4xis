//! Rebind — the closing half of the free ⊣ forgetful load/store cycle.
//!
//! A loaded [`Archive`] is the OPEN, free-category form: nodes by name + data.
//! Rebinding interprets it into the CLOSED world a running system actually
//! knows. The rule is the keystone: a node binds to the running system's concept
//! ONLY when the system knows its name AND its address AGREES with the node's
//! definition-bearing address. Agreement is content-addressed, so a name
//! collision with a *different* definition does NOT bind (the G5 fix at rebind
//! time). A node the system doesn't know — or knows at a different address —
//! stays FREE, carried as live data. That partial, faithful rebind is what lets
//! one `.prx` span open and closed world at once.
//!
//! This module rebinds NODES by address. Rebinding a CONNECTION — replaying its
//! `generator_action` as a `FreeExtension` into the closed-world `Category` — is
//! the deeper form that depends on `pr4xis`; this address-agreement layer is its
//! foundation (a connection can only be applied between nodes that bound).

use crate::address::ContentAddress;
use crate::archive::Archive;
use crate::codec::CodecError;
use crate::definition::Definition;

/// The closed world a `.prx` rebinds INTO — whatever the running system knows,
/// keyed by name, each at the content address the system holds for it.
pub trait RebindTarget {
    /// The content address the running system has for the concept named `name`,
    /// or `None` if it does not know that concept.
    fn address_of(&self, name: &str) -> Option<ContentAddress>;
}

/// The outcome of rebinding one node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rebound {
    /// The target knows this concept at the SAME address — bound to the closed
    /// world.
    Bound(String),
    /// The target does not know it, or knows it at a DIFFERENT address — left in
    /// the free category as live data (graceful open-world).
    Free(String),
}

impl Rebound {
    /// The node's name, bound or free.
    pub fn name(&self) -> &str {
        match self {
            Rebound::Bound(n) | Rebound::Free(n) => n,
        }
    }

    /// Whether the node bound to the closed world.
    pub fn is_bound(&self) -> bool {
        matches!(self, Rebound::Bound(_))
    }
}

/// Rebind one node against a target, by content-address agreement.
pub fn rebind_node(node: &Definition, target: &impl RebindTarget) -> Result<Rebound, CodecError> {
    let node_addr = node.address()?;
    Ok(match target.address_of(&node.name) {
        Some(target_addr) if target_addr == node_addr => Rebound::Bound(node.name.clone()),
        _ => Rebound::Free(node.name.clone()),
    })
}

/// Rebind every node of an archive against a target. The bound nodes are the
/// archive's closed-world surface; the free nodes are its open-world remainder.
pub fn rebind_nodes(
    archive: &Archive,
    target: &impl RebindTarget,
) -> Result<Vec<Rebound>, CodecError> {
    archive
        .nodes
        .iter()
        .map(|n| rebind_node(n, target))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A toy closed world: a name → address table.
    struct Known(HashMap<String, ContentAddress>);

    impl RebindTarget for Known {
        fn address_of(&self, name: &str) -> Option<ContentAddress> {
            self.0.get(name).copied()
        }
    }

    fn node(name: &str, edge_target: &str) -> Definition {
        Definition {
            kind: "Concept".into(),
            name: name.into(),
            edges: vec![("Subsumption".into(), edge_target.into())],
            axioms: vec![],
            lexical: None,
        }
    }

    #[test]
    fn binds_when_name_and_address_agree() {
        let n = node("Employer", "Agent");
        let mut known = HashMap::new();
        known.insert("Employer".to_string(), n.address().unwrap());
        assert_eq!(
            rebind_node(&n, &Known(known)).unwrap(),
            Rebound::Bound("Employer".into())
        );
    }

    #[test]
    fn stays_free_when_unknown() {
        let n = node("Employer", "Agent");
        assert!(!rebind_node(&n, &Known(HashMap::new())).unwrap().is_bound());
    }

    #[test]
    fn stays_free_on_address_disagreement_even_with_same_name() {
        // The G5 fix at rebind time: same NAME, different DEFINITION (hence a
        // different address) does NOT bind — that would be a wrong rebind.
        let loaded = node("Employer", "Agent");
        let different = node("Employer", "Person"); // different edge → different address
        let mut known = HashMap::new();
        known.insert("Employer".to_string(), different.address().unwrap());
        assert!(!rebind_node(&loaded, &Known(known)).unwrap().is_bound());
    }

    #[test]
    fn partial_rebind_keeps_unknowns_free() {
        let a = node("Employer", "Agent");
        let b = node("Stranger", "Thing");
        let mut known = HashMap::new();
        known.insert("Employer".to_string(), a.address().unwrap());
        let archive = Archive {
            nodes: vec![a, b],
            connections: vec![],
        };
        let rebound = rebind_nodes(&archive, &Known(known)).unwrap();
        let bound: Vec<&str> = rebound
            .iter()
            .filter(|r| r.is_bound())
            .map(|r| r.name())
            .collect();
        let free: Vec<&str> = rebound
            .iter()
            .filter(|r| !r.is_bound())
            .map(|r| r.name())
            .collect();
        assert_eq!(bound, vec!["Employer"]);
        assert_eq!(free, vec!["Stranger"]);
    }
}
