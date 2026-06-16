//! Recursive content-addressing — a concept's address that transitively fixes
//! its definition (referents by ADDRESS, not name), cycle-safe.
//!
//! [`Definition::address`](crate::definition::Definition::address) is the LOCAL
//! floor: a node's own fields, with edges referencing targets BY NAME. So a deep
//! change to a referenced concept does not propagate to a referencing node's
//! local address (`connection.rs` / `archive.rs` name this gap verbatim). The
//! teach-a-peer North Star needs PER-CONCEPT transitive identity: send one
//! concept (+ its functor) and a peer content-addresses it INCLUDING its
//! reachable definition, then agrees on identity.
//!
//! The construction (design `~/.claude/plans/a2-recursive-content-address-design.md`):
//! 1. **Tarjan SCC-condensation** over the LOCAL reference digraph (nodes by
//!    name; edges = `(kind, Local(name))`; a `Grounded` edge is already a
//!    `ContentAddress` — a LEAF). The condensation is acyclic by construction —
//!    that IS the cycle-safety theorem (Tarjan 1972).
//! 2. **Reverse-topological Merkle fold** over the condensation (children
//!    addressed first — exactly Tarjan's emission order): a node's recursive
//!    address hashes its fields PLUS its referents' recursive addresses.
//! 3. **Inside a cycle (SCC)**, members are put in a canonical order by their
//!    LOCAL address (a total order, since names are unique within an ontology,
//!    and `name` is part of identity); intra-cycle back-edges encode as the
//!    target's within-SCC INDEX (never a recursive address — that is what makes
//!    a cycle terminate). A TIE (two members with the same local address = a
//!    genuine automorphism with no distinguishing label) is REFUSED, not
//!    coin-flipped (the user's fail-closed policy, 2026-06-16): an unlabeled
//!    automorphic cycle is a modeling error.
//!
//! ADDITIVE: the local floor and `Archive::root` are untouched, so every
//! committed `.prx` pin re-verifies byte-for-byte (#186 preserved). The
//! recursive address is a SEPARATE claim ([`Archive::recursive_root`]).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::address::ContentAddress;
use crate::archive::Archive;
use crate::codec::{self, CodecError};
use crate::definition::{Definition, EdgeTarget};

/// Why a recursive address could not be computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecursiveAddressError {
    /// The canonical encoding failed.
    Codec(CodecError),
    /// A local edge names a target absent from the archive (referential closure
    /// is violated — the same condition `materialize` rejects as `DanglingEdge`).
    DanglingLocalEdge {
        /// The node carrying the dangling edge.
        from: String,
        /// The local target name that resolves to no node.
        to: String,
    },
    /// Two nodes in one cycle share a LOCAL identity (same definition incl. name)
    /// — a genuine automorphism with no canonical order. Fail-closed: a cycle
    /// whose members carry no distinguishing label is a modeling error to fix at
    /// the source, not an identity to invent (the user's 2026-06-16 decision).
    AmbiguousCyclicIdentity {
        /// The colliding member names.
        names: Vec<String>,
    },
    /// The concept to extract is not in the archive.
    ConceptNotFound {
        /// The requested concept name.
        name: String,
    },
}

impl From<CodecError> for RecursiveAddressError {
    fn from(e: CodecError) -> Self {
        RecursiveAddressError::Codec(e)
    }
}

/// A target resolved for the recursive encoding — never a bare name.
#[derive(Serialize)]
enum ResolvedTarget {
    /// A true Merkle child link: the target's OWN recursive address.
    Local([u8; 32]),
    /// An intra-cycle back-edge: the target's canonical within-SCC index. This is
    /// the only non-address encoding, and it is what makes a cycle terminate.
    SelfIndex(u64),
    /// A cross-ontology atom — already content-addressed.
    Grounded { ontology: String, atom: [u8; 32] },
}

/// The canonical recursive form of one node (referents resolved to addresses).
#[derive(Serialize)]
struct NodeCanon<'a> {
    kind: &'a str,
    name: &'a str,
    lexical: Option<&'a str>,
    axioms: Vec<&'a str>,
    edges: Vec<(&'a str, ResolvedTarget)>,
}

impl Archive {
    /// The recursive (transitive) content address of every node, keyed by name —
    /// each address fixes the node's reachable definition, cycle-safe. See the
    /// module doc. `Err` on a dangling local edge or an unlabeled automorphic
    /// cycle (fail-closed).
    pub fn recursive_addresses(
        &self,
    ) -> Result<BTreeMap<String, ContentAddress>, RecursiveAddressError> {
        recursive_addresses(self)
    }

    /// The recursive Merkle root — the content address over the sorted set of
    /// every node's RECURSIVE address. The transitive-identity analogue of
    /// [`root`](Archive::root); additive, leaves `root` untouched.
    pub fn recursive_root(&self) -> Result<ContentAddress, RecursiveAddressError> {
        let by_name = self.recursive_addresses()?;
        let mut addrs: Vec<[u8; 32]> = by_name.values().map(|a| *a.as_bytes()).collect();
        addrs.sort_unstable();
        addrs.dedup();
        Ok(codec::address_of(&addrs)?)
    }

    /// The minimal sub-archive carrying `name` and its transitive LOCAL closure
    /// (every node reachable by `Local` edges) — the teach-a-peer payload. A peer
    /// that loads it recomputes `name`'s recursive address to the SAME value as in
    /// this archive, because a recursive address depends only on this closure plus
    /// the `Grounded` leaves (whose foreign roots the peer must already hold). A
    /// `Grounded` edge is kept verbatim (it points at a foreign atom BY address).
    ///
    /// Connections (the rideable functor that lets the peer INTERPRET the concept)
    /// are not carried here — node identity is the closure; connection transport is
    /// a separate step. `Err` if `name` is absent or a local edge dangles.
    pub fn extract_concept(&self, name: &str) -> Result<Archive, RecursiveAddressError> {
        let index: BTreeMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, d)| (d.name.as_str(), i))
            .collect();
        let &start = index
            .get(name)
            .ok_or_else(|| RecursiveAddressError::ConceptNotFound {
                name: name.to_string(),
            })?;
        // DFS the local closure from `name`.
        let mut keep: BTreeSet<usize> = BTreeSet::new();
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            if !keep.insert(i) {
                continue;
            }
            for (_kind, target) in &self.nodes[i].edges {
                if let EdgeTarget::Local(n) = target {
                    match index.get(n.as_str()) {
                        Some(&j) => stack.push(j),
                        None => {
                            return Err(RecursiveAddressError::DanglingLocalEdge {
                                from: self.nodes[i].name.clone(),
                                to: n.clone(),
                            });
                        }
                    }
                }
            }
        }
        let nodes: Vec<Definition> = keep.iter().map(|&i| self.nodes[i].clone()).collect();
        Ok(Archive {
            nodes,
            connections: Vec::new(),
        })
    }
}

fn recursive_addresses(
    archive: &Archive,
) -> Result<BTreeMap<String, ContentAddress>, RecursiveAddressError> {
    let n = archive.nodes.len();
    // name -> node index. A duplicate name shadows (last wins); referential
    // resolution + the dedup'd root make duplicate names a no-op for identity.
    let mut index: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, node) in archive.nodes.iter().enumerate() {
        index.insert(node.name.as_str(), i);
    }

    // Local adjacency: i -> j for every (kind, Local(name)) edge whose name is in
    // the archive. A Local edge to an absent name is a dangling reference.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, node) in archive.nodes.iter().enumerate() {
        for (_kind, target) in &node.edges {
            if let EdgeTarget::Local(name) = target {
                match index.get(name.as_str()) {
                    Some(&j) => adj[i].push(j),
                    None => {
                        return Err(RecursiveAddressError::DanglingLocalEdge {
                            from: node.name.clone(),
                            to: name.clone(),
                        });
                    }
                }
            }
        }
    }

    // Tarjan SCC (iterative) → `scc_of[i]` + `order` = SCC ids in the order Tarjan
    // emits them, which is REVERSE topological: when an SCC is emitted, every SCC
    // reachable from it is already emitted (so its recursive addresses exist).
    let (scc_of, members, order) = tarjan_scc(n, &adj);

    let mut rec: BTreeMap<String, ContentAddress> = BTreeMap::new();
    // index -> ContentAddress, filled as SCCs are processed (children first).
    let mut addr_of: Vec<Option<ContentAddress>> = vec![None; n];

    for &scc_id in &order {
        let scc = &members[scc_id];
        // A cycle iff the SCC has >1 member, or a single member with a self-loop.
        let is_cycle = scc.len() > 1 || adj[scc[0]].contains(&scc[0]);

        if !is_cycle {
            // Acyclic singleton: a direct Merkle fold. All referents are in
            // already-processed SCCs, so their addresses exist.
            let i = scc[0];
            let canon = node_canon(archive, i, &index, &scc_of, &addr_of, None)?;
            let a = codec::address_of(&canon)?;
            addr_of[i] = Some(a);
            rec.insert(archive.nodes[i].name.clone(), a);
            continue;
        }

        // A cycle. Canonical order = sort members by their LOCAL address (total,
        // since names are unique + part of identity). A tie is an unlabeled
        // automorphism → refuse.
        let mut ordered: Vec<(usize, ContentAddress)> = Vec::with_capacity(scc.len());
        for &i in scc {
            ordered.push((i, archive.nodes[i].address()?));
        }
        ordered.sort_by(|a, b| a.1.as_bytes().cmp(b.1.as_bytes()));
        for w in ordered.windows(2) {
            if w[0].1 == w[1].1 {
                return Err(RecursiveAddressError::AmbiguousCyclicIdentity {
                    names: scc.iter().map(|&i| archive.nodes[i].name.clone()).collect(),
                });
            }
        }
        // within-SCC canonical index per node index.
        let mut within: BTreeMap<usize, u64> = BTreeMap::new();
        for (pos, (i, _)) in ordered.iter().enumerate() {
            within.insert(*i, pos as u64);
        }

        // Component digest x: over every member in canonical order, intra-cycle
        // edges as SelfIndex (terminates), the rest already addressed.
        let mut member_canons: Vec<NodeCanon> = Vec::with_capacity(ordered.len());
        for (i, _) in &ordered {
            member_canons.push(node_canon(
                archive,
                *i,
                &index,
                &scc_of,
                &addr_of,
                Some((scc_id, &within)),
            )?);
        }
        let x = codec::address_of(&member_canons)?;

        // Each member's address = ("#cycle", x, its index) — one shared component
        // digest, distinct per-member addresses.
        for (i, _) in &ordered {
            let idx = within[i];
            let a = codec::address_of(&("#cycle", x.as_bytes(), idx))?;
            addr_of[*i] = Some(a);
            rec.insert(archive.nodes[*i].name.clone(), a);
        }
    }

    Ok(rec)
}

/// Build the canonical recursive form of node `i`. When `cycle` is `Some((scc,
/// within))`, an edge whose target is in the SAME scc encodes as `SelfIndex`;
/// otherwise targets resolve to their (already-computed) recursive address.
fn node_canon<'a>(
    archive: &'a Archive,
    i: usize,
    index: &BTreeMap<&str, usize>,
    scc_of: &[usize],
    addr_of: &[Option<ContentAddress>],
    cycle: Option<(usize, &BTreeMap<usize, u64>)>,
) -> Result<NodeCanon<'a>, RecursiveAddressError> {
    let node = &archive.nodes[i];
    let mut edges: Vec<(&str, ResolvedTarget)> = Vec::with_capacity(node.edges.len());
    for (kind, target) in &node.edges {
        let resolved = match target {
            EdgeTarget::Grounded { ontology, atom } => ResolvedTarget::Grounded {
                ontology: ontology.clone(),
                atom: *atom.as_bytes(),
            },
            EdgeTarget::Local(name) => {
                let j = index[name.as_str()];
                match cycle {
                    Some((scc, within)) if scc_of[j] == scc => {
                        ResolvedTarget::SelfIndex(within[&j])
                    }
                    _ => ResolvedTarget::Local(
                        *addr_of[j].expect("child addressed first").as_bytes(),
                    ),
                }
            }
        };
        edges.push((kind.as_str(), resolved));
    }
    // Canonical: sort + dedup edges and axioms (assembly-order-independent, like
    // Definition::address). Edges sort by their serialized form via the derived
    // Ord on the tuple — but ResolvedTarget isn't Ord, so sort by a key.
    edges.sort_by(|a, b| (a.0, target_key(&a.1)).cmp(&(b.0, target_key(&b.1))));
    let mut axioms: Vec<&str> = node.axioms.iter().map(|s| s.as_str()).collect();
    axioms.sort_unstable();
    axioms.dedup();
    Ok(NodeCanon {
        kind: node.kind.as_str(),
        name: node.name.as_str(),
        lexical: node.lexical.as_deref(),
        axioms,
        edges,
    })
}

/// A total sort key over a resolved target (canonical edge ordering).
fn target_key(t: &ResolvedTarget) -> (u8, Vec<u8>) {
    match t {
        ResolvedTarget::Local(a) => (0, a.to_vec()),
        ResolvedTarget::SelfIndex(n) => (1, n.to_le_bytes().to_vec()),
        ResolvedTarget::Grounded { ontology, atom } => {
            let mut k = ontology.as_bytes().to_vec();
            k.extend_from_slice(atom);
            (2, k)
        }
    }
}

/// Iterative Tarjan SCC. Returns `(scc_of, members, order)` where `order` lists
/// SCC ids in Tarjan's emission order = reverse topological (a reachable SCC is
/// emitted first).
fn tarjan_scc(n: usize, adj: &[Vec<usize>]) -> (Vec<usize>, Vec<Vec<usize>>, Vec<usize>) {
    const UNVISITED: usize = usize::MAX;
    let mut idx = vec![UNVISITED; n];
    let mut low = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut scc_of = vec![UNVISITED; n];
    let mut members: Vec<Vec<usize>> = Vec::new();
    let mut order: Vec<usize> = Vec::new();
    let mut next_idx = 0usize;

    // Explicit DFS stack of (node, next-child-cursor).
    for s in 0..n {
        if idx[s] != UNVISITED {
            continue;
        }
        let mut work: Vec<(usize, usize)> = Vec::new();
        work.push((s, 0));
        idx[s] = next_idx;
        low[s] = next_idx;
        next_idx += 1;
        stack.push(s);
        on_stack[s] = true;

        while let Some(&(v, ci)) = work.last() {
            if ci < adj[v].len() {
                work.last_mut().unwrap().1 += 1;
                let w = adj[v][ci];
                if idx[w] == UNVISITED {
                    idx[w] = next_idx;
                    low[w] = next_idx;
                    next_idx += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    work.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(idx[w]);
                }
            } else {
                // Done with v: propagate low to parent, maybe close an SCC.
                if low[v] == idx[v] {
                    let scc_id = members.len();
                    let mut comp = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        scc_of[w] = scc_id;
                        comp.push(w);
                        if w == v {
                            break;
                        }
                    }
                    members.push(comp);
                    order.push(scc_id);
                }
                work.pop();
                if let Some(&(p, _)) = work.last() {
                    low[p] = low[p].min(low[v]);
                }
            }
        }
    }
    (scc_of, members, order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Definition;

    fn node(name: &str, lexical: Option<&str>, edges: &[(&str, &str)]) -> Definition {
        Definition {
            kind: "Concept".into(),
            name: name.into(),
            edges: edges
                .iter()
                .map(|(k, t)| ((*k).to_string(), EdgeTarget::Local((*t).to_string())))
                .collect(),
            axioms: vec![],
            lexical: lexical.map(|s| s.to_string()),
        }
    }

    fn rec(a: &Archive, name: &str) -> ContentAddress {
        a.recursive_addresses().unwrap()[name]
    }

    /// Test 1 — a concept's recursive address transitively fixes a DEEP referent
    /// (the property the local floor lacked).
    #[test]
    fn recursive_address_fixes_a_deep_referent() {
        let mk = |c_lex: &str| Archive {
            nodes: vec![
                node("A", Some("a"), &[("Subsumption", "B")]),
                node("B", Some("b"), &[("Subsumption", "C")]),
                node("C", Some(c_lex), &[]),
            ],
            connections: vec![],
        };
        let a1 = mk("c");
        let a2 = mk("c-changed"); // only C's lexical differs
        assert_ne!(rec(&a1, "A"), rec(&a2, "A"), "deep change propagates to A");
        assert_eq!(
            a1.nodes[0].address().unwrap(),
            a2.nodes[0].address().unwrap(),
            "A's LOCAL address is unchanged — only the recursive one moves"
        );
    }

    /// Test 2 — assembly-order independence, deep-difference sensitivity.
    #[test]
    fn recursive_root_is_order_independent_but_deep_sensitive() {
        let a = Archive {
            nodes: vec![
                node("A", None, &[("Subsumption", "B")]),
                node("B", Some("b"), &[]),
            ],
            connections: vec![],
        };
        let shuffled = Archive {
            nodes: vec![a.nodes[1].clone(), a.nodes[0].clone()],
            connections: vec![],
        };
        assert_eq!(
            a.recursive_root().unwrap(),
            shuffled.recursive_root().unwrap(),
            "assembly order must not change the recursive root"
        );
        let deep = Archive {
            nodes: vec![
                node("A", None, &[("Subsumption", "B")]),
                node("B", Some("b-DIFFERENT"), &[]),
            ],
            connections: vec![],
        };
        assert_ne!(a.recursive_root().unwrap(), deep.recursive_root().unwrap());
        assert_ne!(rec(&a, "A"), rec(&deep, "A"));
    }

    /// Test 3 — a labeled symmetric cycle terminates, addresses each member
    /// distinctly, and is deterministic under node reordering.
    #[test]
    fn a_labeled_symmetric_cycle_addresses_deterministically() {
        let hot = node("Hot", Some("hot"), &[("Equivalence", "Cold")]);
        let cold = node("Cold", Some("cold"), &[("Equivalence", "Hot")]);
        let a = Archive {
            nodes: vec![hot.clone(), cold.clone()],
            connections: vec![],
        };
        let r = a.recursive_addresses().unwrap(); // terminates (no stack overflow)
        assert_ne!(
            r["Hot"], r["Cold"],
            "distinct members get distinct addresses"
        );
        let reordered = Archive {
            nodes: vec![cold, hot],
            connections: vec![],
        };
        let r2 = reordered.recursive_addresses().unwrap();
        assert_eq!(
            r["Hot"], r2["Hot"],
            "cycle addressing is node-order independent"
        );
        assert_eq!(r["Cold"], r2["Cold"]);
    }

    /// A dangling local edge fails closed (referential closure).
    #[test]
    fn a_dangling_local_edge_is_refused() {
        let a = Archive {
            nodes: vec![node("A", None, &[("Subsumption", "Ghost")])],
            connections: vec![],
        };
        assert!(matches!(
            a.recursive_addresses(),
            Err(RecursiveAddressError::DanglingLocalEdge { .. })
        ));
    }

    /// Test 5 (the headline) — TEACH-A-PEER round trip. A sender extracts a
    /// concept's minimal payload; a receiver recomputes its recursive address and
    /// AGREES, including the transitive (and grounded) dependencies — from a
    /// payload that excludes everything unrelated.
    #[test]
    fn teach_a_peer_round_trip() {
        // G grounds into a foreign ontology by atom address (a leaf).
        let mut g = node("G", Some("g"), &[]);
        g.edges.push((
            "denotes".to_string(),
            EdgeTarget::Grounded {
                ontology: "english".to_string(),
                atom: ContentAddress::of(b"the-foreign-atom"),
            },
        ));
        let full = Archive {
            nodes: vec![
                node("A", Some("a"), &[("Subsumption", "B")]),
                node("B", Some("b"), &[("Subsumption", "C"), ("Denotes", "G")]),
                node("C", Some("c"), &[]),
                node("Unrelated", Some("u"), &[]), // not in A's closure
                g,
            ],
            connections: vec![],
        };
        let sender_addr = rec(&full, "A");

        // Sender extracts the minimal payload for A.
        let payload = full.extract_concept("A").unwrap();
        let have = |n: &str| payload.nodes.iter().any(|d| d.name == n);
        assert!(
            have("A") && have("B") && have("C") && have("G"),
            "the closure travels"
        );
        assert!(
            !have("Unrelated"),
            "unrelated nodes are excluded — minimal payload"
        );

        // Receiver recomputes A's recursive address from the payload alone.
        let receiver_addr = rec(&payload, "A");
        assert_eq!(
            sender_addr, receiver_addr,
            "the peer agrees on A's identity, transitive + grounded deps included, from the minimal payload"
        );
    }
}
