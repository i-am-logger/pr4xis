//! Grounding resolution — turning a [`Grounded`](crate::definition::EdgeTarget::Grounded)
//! edge's foreign atom back into the [`Definition`] it names, by content-address
//! agreement across the connected ontologies an archive declares.
//!
//! This is the address-keyed DUAL of the name-keyed [`rebind`](crate::rebind):
//! rebind asks "does the running system know a concept by this NAME at an
//! agreeing address?"; resolution asks "does this connected ontology hold an
//! atom at this ADDRESS?". A foreign atom has no name in our archive — only a
//! content address — so name-keyed rebind cannot reach it. [`AtomResolver`] is
//! the one new primitive that can (inward gap #3).
//!
//! # Fail-closed, the lock decides
//!
//! An archive declares its external connections in a [`ConnectedOntologies`]
//! manifest: for each connected ontology, the `root` its lock pins. The resolver
//! is built ONCE and gated — every declared ontology's supplied archive must
//! match its pinned root before a single atom resolves, so a version/content
//! skew is refused up front (the G5 fail-closed spirit, now across archives).
//! A grounded edge into an undeclared ontology, or naming an atom the connected
//! ontology does not hold, returns a typed [`LinkError`] — never a silent miss
//! and never a wrong bind.

use std::collections::BTreeMap;

use crate::address::ContentAddress;
use crate::archive::Archive;
use crate::codec::CodecError;
use crate::definition::{Definition, EdgeTarget};

/// Add typed cross-ontology grounding edges to an [`Archive`]'s nodes — the
/// PRODUCE side of grounding, GENERAL over the lens (the [`resolve`](AtomResolver::resolve)
/// counterpart).
///
/// `lens(node)` maps a node to the `(kind, `[`EdgeTarget::Grounded`]`)` edges its
/// content points along — a node's lexical prose grounding into a connected
/// ontology's atoms, say. The lexical `denotes` floor is ONE lens; `cites` /
/// `defines` are others, the same shape. The lens is the only place a specific
/// ontology (English, a cited title, …) enters; `ground` itself is
/// source-agnostic, so any content archive (USC, English, …) grounds the same way
/// — the returned archive's grounded edges resolve through [`AtomResolver`].
pub fn ground(
    archive: &Archive,
    lens: impl Fn(&Definition) -> Vec<(String, EdgeTarget)>,
) -> Archive {
    let nodes = archive
        .nodes
        .iter()
        .map(|node| {
            let mut grounded = node.clone();
            grounded.edges.extend(lens(node));
            grounded
        })
        .collect();
    Archive {
        nodes,
        connections: archive.connections.clone(),
    }
}

/// One declared connection: a connected ontology, the `root` its lock pins, and
/// the `role` the grounding edges into it carry (the kind — `denotes` for the
/// lexical floor; carried here so the floor spends no per-edge kind tag).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedOntology {
    /// The connected ontology's name (how a grounded edge addresses it).
    pub name: String,
    /// The content address the lock pins for that ontology — resolution refuses
    /// a supplied archive whose root disagrees.
    pub root: ContentAddress,
    /// The grounding kind edges into this ontology assert (e.g. `denotes`).
    pub role: String,
}

/// The `[connected_ontologies]` manifest — which ontologies this archive's
/// grounded edges point into, each pinned to a root the lock must satisfy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectedOntologies(pub Vec<ConnectedOntology>);

impl ConnectedOntologies {
    /// The declaration for `name`, if this manifest names it.
    pub fn get(&self, name: &str) -> Option<&ConnectedOntology> {
        self.0.iter().find(|c| c.name == name)
    }
}

/// Why a grounded edge could not be resolved — fail-closed, never a silent bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkError {
    /// The edge grounds into an ontology the manifest does not declare.
    UnknownOntology { ontology: String },
    /// A declared ontology was supplied, but its archive's actual root disagrees
    /// with the pinned root — a version/content skew. Refused.
    RootMismatch {
        ontology: String,
        pinned: ContentAddress,
        actual: ContentAddress,
    },
    /// The connected ontology holds no atom at the named address.
    AtomAbsent {
        ontology: String,
        atom: ContentAddress,
    },
    /// The target is a [`Local`](EdgeTarget::Local) edge — not a grounded edge
    /// to resolve. (Callers traverse local edges by name, not through here.)
    NotGrounded,
    /// A node or archive address could not be derived (codec failure).
    Codec(CodecError),
}

impl core::fmt::Display for LinkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LinkError::UnknownOntology { ontology } => {
                write!(f, "grounding: edge into undeclared ontology {ontology:?}")
            }
            LinkError::RootMismatch {
                ontology,
                pinned,
                actual,
            } => write!(
                f,
                "grounding: {ontology:?} root skew — pinned {}, supplied {}",
                pinned.to_hex(),
                actual.to_hex()
            ),
            LinkError::AtomAbsent { ontology, atom } => write!(
                f,
                "grounding: {ontology:?} holds no atom at {}",
                atom.to_hex()
            ),
            LinkError::NotGrounded => write!(f, "grounding: target is local, not a grounded edge"),
            LinkError::Codec(e) => write!(f, "grounding: {e}"),
        }
    }
}

impl std::error::Error for LinkError {}

/// Resolves a [`Grounded`](EdgeTarget::Grounded) edge target to the foreign atom
/// it names, by content-address agreement across the loaded connected archives.
///
/// Built once from a manifest + the supplied peer archives, gated so every
/// declared ontology's archive matches its pinned root. Each connected ontology
/// is indexed by its nodes' definition-bearing addresses, so resolution is an
/// O(log n) lookup, never a scan.
#[derive(Debug)]
pub struct AtomResolver<'a> {
    /// ontology name → (atom address → its node).
    atoms: BTreeMap<String, BTreeMap<ContentAddress, &'a Definition>>,
}

impl<'a> AtomResolver<'a> {
    /// Build the resolver from the `manifest` and the loaded `peers` (by name).
    ///
    /// Fail-closed: a declared ontology with no supplied archive is
    /// [`UnknownOntology`](LinkError::UnknownOntology); a supplied archive whose
    /// root disagrees with the pinned root is [`RootMismatch`](LinkError::RootMismatch).
    /// Only after every pin agrees is any atom index built.
    pub fn new(
        manifest: &ConnectedOntologies,
        peers: &'a BTreeMap<String, Archive>,
    ) -> Result<Self, LinkError> {
        let mut atoms: BTreeMap<String, BTreeMap<ContentAddress, &'a Definition>> = BTreeMap::new();
        for decl in &manifest.0 {
            let archive = peers
                .get(&decl.name)
                .ok_or_else(|| LinkError::UnknownOntology {
                    ontology: decl.name.clone(),
                })?;
            let actual = archive.root().map_err(LinkError::Codec)?;
            if actual != decl.root {
                return Err(LinkError::RootMismatch {
                    ontology: decl.name.clone(),
                    pinned: decl.root,
                    actual,
                });
            }
            let mut index: BTreeMap<ContentAddress, &'a Definition> = BTreeMap::new();
            for node in &archive.nodes {
                index.insert(node.address().map_err(LinkError::Codec)?, node);
            }
            atoms.insert(decl.name.clone(), index);
        }
        Ok(Self { atoms })
    }

    /// Resolve a grounded edge `target` to its foreign atom. Fail-closed: an
    /// undeclared ontology or an absent atom returns a typed [`LinkError`], never
    /// a silent miss. A [`Local`](EdgeTarget::Local) target is
    /// [`NotGrounded`](LinkError::NotGrounded) — there is nothing foreign to
    /// resolve.
    pub fn resolve(&self, target: &EdgeTarget) -> Result<&'a Definition, LinkError> {
        let (ontology, atom) = match target {
            EdgeTarget::Grounded { ontology, atom } => (ontology, atom),
            EdgeTarget::Local(_) => return Err(LinkError::NotGrounded),
        };
        let index = self
            .atoms
            .get(ontology)
            .ok_or_else(|| LinkError::UnknownOntology {
                ontology: ontology.clone(),
            })?;
        index
            .get(atom)
            .copied()
            .ok_or_else(|| LinkError::AtomAbsent {
                ontology: ontology.clone(),
                atom: *atom,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synset(name: &str, gloss: &str) -> Definition {
        Definition {
            kind: "Concept".into(),
            name: name.into(),
            edges: vec![],
            axioms: vec![],
            lexical: Some(gloss.into()),
        }
    }

    /// A miniature connected ontology + a manifest pinning its real root, and a
    /// grounded edge into one of its atoms.
    fn fixture() -> (
        BTreeMap<String, Archive>,
        ConnectedOntologies,
        ContentAddress,
    ) {
        let dog = synset("s-dog", "a domesticated canine");
        let atom = dog.address().unwrap();
        let english = Archive {
            nodes: vec![dog, synset("s-animal", "a living organism")],
            connections: vec![],
        };
        let root = english.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert("english_wordnet".to_string(), english);
        let manifest = ConnectedOntologies(vec![ConnectedOntology {
            name: "english_wordnet".to_string(),
            root,
            role: "denotes".to_string(),
        }]);
        (peers, manifest, atom)
    }

    #[test]
    fn resolves_a_grounded_atom_by_content_address() {
        let (peers, manifest, atom) = fixture();
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let node = resolver
            .resolve(&EdgeTarget::Grounded {
                ontology: "english_wordnet".to_string(),
                atom,
            })
            .expect("the atom resolves");
        assert_eq!(node.name, "s-dog");
        assert_eq!(node.lexical.as_deref(), Some("a domesticated canine"));
    }

    #[test]
    fn ground_adds_lens_edges_that_then_resolve() {
        // The produce side: a content archive grounds via a lens (here, a node
        // named "provision" grounds into the english atom), and the added typed
        // Grounded edge resolves through the resolver — produce ∘ resolve, all
        // source-agnostic.
        let (peers, manifest, atom) = fixture();
        let content = Archive {
            nodes: vec![Definition {
                kind: "Provision".into(),
                name: "title-1-§1".into(),
                edges: vec![],
                axioms: vec![],
                lexical: Some("a domesticated canine occurs here".into()),
            }],
            connections: vec![],
        };
        // A lens that grounds any node into the fixture's atom (a stand-in for a
        // real denotes producer).
        let grounded = ground(&content, |_node| {
            vec![(
                "denotes".to_string(),
                EdgeTarget::Grounded {
                    ontology: "english_wordnet".to_string(),
                    atom,
                },
            )]
        });
        let edge = &grounded.nodes[0].edges[0];
        assert_eq!(edge.0, "denotes");
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        let resolved = resolver
            .resolve(&edge.1)
            .expect("the grounded edge resolves");
        assert_eq!(resolved.name, "s-dog");
    }

    #[test]
    fn an_absent_atom_fails_closed() {
        let (peers, manifest, _) = fixture();
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        // An address of an atom the connected ontology does not hold.
        let ghost = ContentAddress::of(b"a synset that was never declared");
        assert_eq!(
            resolver.resolve(&EdgeTarget::Grounded {
                ontology: "english_wordnet".to_string(),
                atom: ghost,
            }),
            Err(LinkError::AtomAbsent {
                ontology: "english_wordnet".to_string(),
                atom: ghost,
            })
        );
    }

    #[test]
    fn an_undeclared_ontology_fails_closed() {
        let (peers, manifest, atom) = fixture();
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        assert_eq!(
            resolver.resolve(&EdgeTarget::Grounded {
                ontology: "klingon".to_string(),
                atom,
            }),
            Err(LinkError::UnknownOntology {
                ontology: "klingon".to_string(),
            })
        );
    }

    #[test]
    fn a_root_skew_refuses_to_build() {
        // The manifest pins a root that does NOT match the supplied archive — a
        // version/content skew. The resolver refuses to build at all.
        let (peers, _, _) = fixture();
        let wrong = ConnectedOntologies(vec![ConnectedOntology {
            name: "english_wordnet".to_string(),
            root: ContentAddress::of(b"some other english version"),
            role: "denotes".to_string(),
        }]);
        match AtomResolver::new(&wrong, &peers) {
            Err(LinkError::RootMismatch { ontology, .. }) => {
                assert_eq!(ontology, "english_wordnet");
            }
            other => panic!("expected a RootMismatch skew refusal; got {other:?}"),
        }
    }

    #[test]
    fn a_missing_peer_archive_fails_closed() {
        let (_, manifest, _) = fixture();
        let empty: BTreeMap<String, Archive> = BTreeMap::new();
        assert_eq!(
            AtomResolver::new(&manifest, &empty).map(|_| ()),
            Err(LinkError::UnknownOntology {
                ontology: "english_wordnet".to_string(),
            })
        );
    }

    #[test]
    fn a_local_target_is_not_a_grounded_edge() {
        let (peers, manifest, _) = fixture();
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();
        assert_eq!(
            resolver.resolve(&EdgeTarget::Local("s-dog".to_string())),
            Err(LinkError::NotGrounded)
        );
    }
}
