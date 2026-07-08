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
/// The lens is FALLIBLE: it returns a typed [`LinkError`] when a node DECLARES a
/// grounding whose target atom cannot be realized (a declared-but-unresolvable
/// target — an authoring/version fault), so a declared grounding is never
/// silently dropped. A node that declares nothing returns `Ok(vec![])` (a
/// legitimate no-op), distinct from a declared target that fails to resolve.
pub fn ground(
    archive: &Archive,
    lens: impl Fn(&Definition) -> Result<Vec<(String, EdgeTarget)>, LinkError>,
) -> Result<Archive, LinkError> {
    let mut nodes = Vec::with_capacity(archive.nodes.len());
    for node in &archive.nodes {
        let mut grounded = node.clone();
        grounded.edges.extend(lens(node)?);
        nodes.push(grounded);
    }
    Ok(Archive {
        nodes,
        connections: archive.connections.clone(),
    })
}

/// The TYPE grounding LENS — the general, source-agnostic producer of typed
/// cross-ontology `instantiates` edges, over ANY target ontology.
///
/// It is the PRODUCE side of an instance-grounding FUNCTOR carried as `.prx`
/// DATA (a [`Connection`](crate::connection::Connection) whose kind reaches
/// `InstanceFunctor` in the meta-ontology). `type_map` is the functor's
/// `map_object` (a source node KIND → the target concept NAME it instantiates —
/// `Section ↦ Statute`, `Pet ↦ Dog`), `relation` is the functor's `map_morphism`
/// image (the reachability kind the instantiation edge asserts, e.g.
/// `Subsumption`), `target_ontology` is the connected ontology the edges address
/// (`LegalSources`, `taxonomy`, …), and `peer` is that ontology's LOADED archive
/// — the atoms are its nodes' content addresses, so the generic
/// [`AtomResolver`] binds them by agreement.
///
/// For each node whose kind is a key in `type_map`, it emits one
/// `(relation, `[`EdgeTarget::Grounded`]`)` edge into the target concept's
/// `Definition` atom BY CONTENT ADDRESS. A node whose kind is NOT in `type_map` is
/// left ungrounded (`Ok(vec![])`) — a legitimate no-op. A node whose kind IS a
/// `type_map` key but whose mapped concept NAME is absent from the `peer` is a
/// DECLARED-but-unrealizable grounding — an authoring/version fault — and FAILS
/// CLOSED with [`LinkError::GroundTargetAbsent`], never a silent empty edge. There
/// is no `match node.kind` hardcode and no target-ontology name baked in: both are
/// the loaded functor's DATA and the loaded peer's archive.
///
/// Spivak (2012) *Functorial Data Migration* — an instance is a functor into the
/// schema; a typed node's grounding IS that functor, carried as data and produced
/// here.
pub fn type_lens<'a>(
    type_map: &'a [(String, String)],
    relation: &'a str,
    target_ontology: &'a str,
    peer: &'a Archive,
) -> impl Fn(&Definition) -> Result<Vec<(String, EdgeTarget)>, LinkError> + 'a {
    move |node| {
        // A node whose kind is NOT declared by the functor is a legitimate no-op.
        let Some((_, concept)) = type_map
            .iter()
            .find(|(source_kind, _)| source_kind.as_str() == node.kind.as_str())
        else {
            return Ok(Vec::new());
        };
        // The kind IS declared: the mapped target concept MUST resolve to a peer
        // atom. A miss is a declared-but-unrealizable grounding (a stale map entry,
        // a version skew, a typo) — fail closed, never a silently dropped edge.
        let target = peer
            .nodes
            .iter()
            .find(|n| n.name.as_str() == concept.as_str())
            .ok_or_else(|| LinkError::GroundTargetAbsent {
                kind: node.kind.clone(),
                target: concept.clone(),
                peer: target_ontology.to_string(),
            })?;
        let atom = target.address().map_err(LinkError::Codec)?;
        Ok(vec![(
            relation.to_string(),
            EdgeTarget::Grounded {
                ontology: target_ontology.to_string(),
                atom,
            },
        )])
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
    /// The edge grounds into an ontology the manifest does not declare (a
    /// resolve-time fault).
    UnknownOntology { ontology: String },
    /// A manifest-DECLARED ontology has no supplied peer archive, so the resolver
    /// cannot be built (a build-time fault — distinct from [`UnknownOntology`](Self::UnknownOntology),
    /// which is an edge into an ontology the manifest never declared).
    MissingPeerArchive { ontology: String },
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
    /// A node's kind IS declared by a grounding functor's `map_object`, but the
    /// concept NAME it maps to is absent from the (present) target peer — a
    /// declared-but-unrealizable grounding (stale map entry / version skew /
    /// typo). Fail-closed: the declared grounding is never silently dropped.
    /// Distinct from [`MissingPeerArchive`](Self::MissingPeerArchive): the peer IS
    /// loaded; it just does not hold the named concept.
    GroundTargetAbsent {
        /// The source node kind the functor declared a grounding for.
        kind: String,
        /// The target concept name the functor mapped that kind to.
        target: String,
        /// The target ontology (present) that lacks the named concept.
        peer: String,
    },
    /// A loaded ontology used AS a grounding TARGET is itself a grounding SOURCE —
    /// the unsupported multi-level chain. Grounding freezes the peer set before any
    /// slot is grounded, so re-materializing a target (to add ITS grounding edges)
    /// shifts the very node addresses a source grounded against. Fail-closed: the
    /// documented one-level contract is enforced at runtime, not left to dangle.
    GroundingTargetIsSource {
        /// The ontology that is both a grounding target and a grounding source.
        target: String,
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
            LinkError::MissingPeerArchive { ontology } => write!(
                f,
                "grounding: declared ontology {ontology:?} has no supplied peer archive"
            ),
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
            LinkError::GroundTargetAbsent { kind, target, peer } => write!(
                f,
                "grounding: declared target {target:?} for kind {kind:?} is absent from \
                 loaded peer {peer:?} (declared-but-unrealizable grounding)"
            ),
            LinkError::GroundingTargetIsSource { target } => write!(
                f,
                "grounding: {target:?} is used as a grounding target but is itself a \
                 grounding source — the unsupported multi-level chain (one-level contract)"
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
    /// [`MissingPeerArchive`](LinkError::MissingPeerArchive); a supplied archive
    /// whose root disagrees with the pinned root is [`RootMismatch`](LinkError::RootMismatch).
    /// Only after every pin agrees is any atom index built.
    pub fn new(
        manifest: &ConnectedOntologies,
        peers: &'a BTreeMap<String, Archive>,
    ) -> Result<Self, LinkError> {
        let mut atoms: BTreeMap<String, BTreeMap<ContentAddress, &'a Definition>> = BTreeMap::new();
        for decl in &manifest.0 {
            let archive = peers
                .get(&decl.name)
                .ok_or_else(|| LinkError::MissingPeerArchive {
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
            Ok(vec![(
                "denotes".to_string(),
                EdgeTarget::Grounded {
                    ontology: "english_wordnet".to_string(),
                    atom,
                },
            )])
        })
        .expect("the infallible lens grounds");
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
            Err(LinkError::MissingPeerArchive {
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

    #[test]
    fn type_lens_kind_not_in_map_is_a_silent_noop() {
        // A node whose kind is NOT a functor key grounds nothing — Ok(empty), the
        // legitimate no-op (distinct from a declared-but-absent target).
        let (peers, _, _) = fixture();
        let english = peers.get("english_wordnet").unwrap();
        let type_map = vec![("Canine".to_string(), "s-dog".to_string())];
        let lens = type_lens(&type_map, "Subsumption", "english_wordnet", english);
        let node = Definition {
            kind: "Mineral".into(), // NOT a key in the map
            name: "salmon".into(),
            edges: vec![],
            axioms: vec![],
            lexical: None,
        };
        assert_eq!(
            lens(&node),
            Ok(vec![]),
            "an undeclared kind grounds nothing"
        );
    }

    #[test]
    fn type_lens_declared_but_absent_target_fails_closed() {
        // A node whose kind IS a functor key but whose mapped concept is absent
        // from the present peer is a declared-but-unrealizable grounding — it must
        // FAIL CLOSED (typed GroundTargetAbsent), NOT silently drop the edge.
        let (peers, _, _) = fixture();
        let english = peers.get("english_wordnet").unwrap();
        let type_map = vec![("Canine".to_string(), "s-DOESNOTEXIST".to_string())];
        let lens = type_lens(&type_map, "Subsumption", "english_wordnet", english);
        let node = Definition {
            kind: "Canine".into(), // declared, but the target concept is absent
            name: "rex".into(),
            edges: vec![],
            axioms: vec![],
            lexical: None,
        };
        assert_eq!(
            lens(&node),
            Err(LinkError::GroundTargetAbsent {
                kind: "Canine".to_string(),
                target: "s-DOESNOTEXIST".to_string(),
                peer: "english_wordnet".to_string(),
            }),
            "a declared but unresolvable target fails closed, never a silent empty edge"
        );
    }
}
