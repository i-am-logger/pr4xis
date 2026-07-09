//! `ArchiveLens` — the `rkyv` local-cache/query lens over a `.prx`
//! [`Archive`].
//!
//! This is the runtime-graph sibling of the OWL leaf's `.prx.gz` rkyv envelope
//! (`domains/.../social/software/markup/xml/owl/prx.rs`): where that module
//! archives the `CodegenData` interchange through a hand-authored owned mirror
//! ([`OwnedCodegenData`]), this one archives the *runtime graph* — the
//! [`Archive`] of [`Definition`] nodes and [`Connection`] morphisms — through
//! hand-authored `rkyv` mirror types.
//!
//! ## Two serialized forms — this is NOT the address form
//!
//! A `.prx` [`Archive`] serializes two different ways, under two different lens
//! laws:
//!
//! - [`crate::load::emit`] / [`crate::load::load`] — the **content-address**
//!   form: a canonical, deterministic, toolchain-independent DAG-CBOR encoding
//!   whose BLAKE3 digest ([`crate::address::ContentAddress`]) IS the archive's
//!   identity. That is the wire / peer-agreement form.
//! - [`ArchiveLens`] (this module) — the **local cache / query** form: a
//!   `rkyv` zero-copy layout for fast local materialization. `rkyv`'s byte
//!   layout is determined by the `rkyv` version and the target, so it is
//!   *deliberately* NOT a content address — it is a private cache, never
//!   committed to the tree, never pinned in `praxis.lock`, exactly as the OWL
//!   `.prx.gz` rkyv blob is a cross-toolchain liability that stays uncommitted.
//!
//! Emitting a stable content address is the DAG-CBOR lens's job; this lens
//! trades that stability for zero-copy access speed on the local query path.
//!
//! ## Hand-authored mirror, not a derive over the live types
//!
//! The live runtime types ([`Archive`], [`Definition`], [`Connection`],
//! [`EdgeTarget`], [`GeneratorAction`]) carry `serde` derives for the canonical
//! DAG-CBOR codec and a bespoke [`EdgeTarget`] (de)serializer that keeps a
//! `Local` target byte-identical to the bare string it replaced. We do NOT slap
//! `#[derive(rkyv::Archive)]` on them — that would couple the address-bearing
//! wire types to `rkyv`'s layout and its blanket derives. Instead this module
//! authors a *purpose-built* mirror ([`ArchivedArchive`] and friends) and the
//! [`from`](ArchivedArchive::from_live)/[`into`](ArchivedArchive::into_live)
//! conversions, exactly as `owl/prx.rs` authors [`OwnedCodegenData`] as the
//! serializable mirror of `CodegenData`. The mirror is a distinct
//! representation: `ContentAddress` (opaque 32-byte digest, no `rkyv`-friendly
//! constructor) is carried in the mirror as its 64-char lowercase hex, which
//! round-trips losslessly through [`ContentAddress::from_hex`].
//!
//! ## The lens, and what `get` returns
//!
//! [`ArchiveLens::put`] `rkyv`-serializes the mirror (the lens PUT); it is the
//! local cache form, EXPLICITLY not the DAG-CBOR address form.
//! [`ArchiveLens::get`] copies into a 16-aligned buffer (a fetched/mmapped
//! `&[u8]` carries no alignment guarantee) and `rkyv::from_bytes`-validates it
//! with `bytecheck` before materializing, so a corrupted or truncated blob
//! fails closed — the same fail-closed shape as
//! [`envelope_from_bytes`](../../../../pr4xis_domains/social/software/markup/xml/owl/prx/fn.envelope_from_bytes.html)
//! in the OWL leaf.
//!
//! Two GETs live here. [`ArchiveLens::get`] uses **validated owning**
//! deserialization — it returns an owned [`Archive`], for callers that need one
//! (e.g. re-deriving the content root, which needs [`Definition`] addressing).
//! [`ArchiveLens::access`] is the **zero-copy** GET (Step 1c): it
//! `bytecheck`-validates the buffer once and returns a borrowed
//! [`ArchivedArchiveView`] materialized IN PLACE (no owned rebuild), and
//! [`ArchiveLens::access_unchecked`] serves the hot query path over that
//! already-validated, immutable buffer. That is what [`RuntimeOntology`] reasons
//! over.
//!
//! [`RuntimeOntology`]: crate::ontology::RuntimeOntology
//!
//! ## Lens laws
//!
//! `put`/`get` form a well-behaved lens (Foster, Greenwald, Moore, Pierce &
//! Schmitt 2007, "Combinators for Bidirectional Tree Transformations", *ACM
//! TOPLAS* 29(3) §3, Definition 3.2) between the runtime [`Archive`] and its `rkyv` bytes —
//! the same law family the OWL `.prx.gz` `emit`/`load` pair realizes. The two
//! runnable axioms [`ArchiveLensGetPut`] and [`ArchiveLensPutGet`] witness the
//! GetPut and PutGet legs (behind the `emit` feature, which supplies the
//! `pr4xis` axiom machinery).
//!
//! ## Citations
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §3, Definition 3.2 — the lens
//!   laws (GetPut / PutGet).
//! - **Koloski, D.** *rkyv: zero-copy deserialization framework for Rust*, v0.8,
//!   <https://github.com/rkyv/rkyv>.
//!
//! [`OwnedCodegenData`]: https://docs.rs/pr4xis-domains
//! [`Archive`]: crate::archive::Archive
//! [`Definition`]: crate::definition::Definition
//! [`Connection`]: crate::connection::Connection
//! [`EdgeTarget`]: crate::definition::EdgeTarget
//! [`GeneratorAction`]: crate::connection::GeneratorAction

use crate::address::ContentAddress;
use crate::archive::Archive;
use crate::connection::{Connection, GeneratorAction};
use crate::definition::{Definition, EdgeTarget};
use crate::lens::rkyv_lens::{RkyvLens, RkyvLensError, RkyvMirror, RkyvOwned};

// =============================================================================
// Hand-authored rkyv mirror types — the serializable shadow of the live graph.
// =============================================================================

/// `rkyv` mirror of [`Archive`] — the serializable shadow of the runtime graph.
///
/// Field-for-field identical to [`Archive`] except every nested type is its own
/// `rkyv` mirror. Authored (not `#[derive]`d on the live [`Archive`]) so the
/// address-bearing wire type stays free of `rkyv`'s layout coupling. `rkyv`'s
/// derive generates this type's OWN archived form internally; that generated
/// name is never referenced by hand.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ArchivedArchive {
    /// Mirrors [`Archive::nodes`].
    pub nodes: Vec<ArchivedDefinition>,
    /// Mirrors [`Archive::connections`].
    pub connections: Vec<ArchivedConnection>,
}

/// `rkyv` mirror of [`Definition`].
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ArchivedDefinition {
    /// Mirrors [`Definition::kind`].
    pub kind: String,
    /// Mirrors [`Definition::name`].
    pub name: String,
    /// Mirrors [`Definition::edges`]: `(relation-kind name, target)`.
    pub edges: Vec<(String, ArchivedEdgeTarget)>,
    /// Mirrors [`Definition::axioms`].
    pub axioms: Vec<String>,
    /// Mirrors [`Definition::lexical`].
    pub lexical: Option<String>,
}

/// `rkyv` mirror of [`EdgeTarget`].
///
/// The `Grounded` variant's [`ContentAddress`] has no `rkyv`-friendly byte
/// constructor, so it is carried here as its 64-char lowercase hex
/// ([`ContentAddress::to_hex`]) and rebuilt via [`ContentAddress::from_hex`]
/// on the way back (fail-closed if the hex is malformed).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum ArchivedEdgeTarget {
    /// Mirrors [`EdgeTarget::Local`].
    Local(String),
    /// Mirrors [`EdgeTarget::Grounded`]; `atom_hex` is the foreign atom's
    /// content address as 64-char lowercase hex.
    Grounded {
        /// Mirrors [`EdgeTarget::Grounded::ontology`].
        ontology: String,
        /// The foreign atom's [`ContentAddress`], as 64-char lowercase hex.
        atom_hex: String,
    },
}

/// `rkyv` mirror of [`Connection`].
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ArchivedConnection {
    /// Mirrors [`Connection::kind`].
    pub kind: String,
    /// Mirrors [`Connection::source`].
    pub source: String,
    /// Mirrors [`Connection::target`].
    pub target: String,
    /// Mirrors [`Connection::action`].
    pub action: ArchivedGeneratorAction,
    /// Mirrors [`Connection::laws`].
    pub laws: Vec<String>,
}

/// `rkyv` mirror of [`GeneratorAction`] — one arm per categorical family.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum ArchivedGeneratorAction {
    /// Mirrors [`GeneratorAction::Functor`].
    Functor {
        /// `(source object name, target expression)`.
        map_object: Vec<(String, String)>,
        /// `(source relation-kind name, target kind expression)`.
        map_morphism: Vec<(String, String)>,
    },
    /// Mirrors [`GeneratorAction::NaturalTransformation`].
    NaturalTransformation {
        /// `(object name, component-morphism expression)`.
        components: Vec<(String, String)>,
    },
    /// Mirrors [`GeneratorAction::Lens`].
    Lens {
        /// The focused view.
        view: String,
        /// The `get` morphism name.
        get: String,
        /// The `put` morphism name.
        put: String,
    },
    /// Mirrors [`GeneratorAction::Adjunction`].
    Adjunction {
        /// Left functor object map.
        left_map_object: Vec<(String, String)>,
        /// Right functor object map.
        right_map_object: Vec<(String, String)>,
        /// The unit `η` components.
        unit: Vec<(String, String)>,
        /// The counit `ε` components.
        counit: Vec<(String, String)>,
    },
}

// =============================================================================
// Errors.
// =============================================================================

/// Why [`ArchiveLens::get`] refused a blob. Fail-closed: `get` returns an
/// [`Archive`] only when the bytes both `bytecheck`-validate AND every carried
/// grounded-atom address is well-formed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveLensError {
    /// `rkyv` deserialization or `bytecheck` validation failed — a corrupted,
    /// truncated, or misaligned blob.
    Rkyv(String),
    /// A grounded edge's carried atom hex is not a valid 64-char lowercase-hex
    /// [`ContentAddress`]. Fail-closed rather than fabricating an address.
    BadAtomAddress(String),
}

impl core::fmt::Display for ArchiveLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ArchiveLensError::Rkyv(m) => write!(f, "rkyv archive-lens error: {m}"),
            ArchiveLensError::BadAtomAddress(h) => {
                write!(f, "grounded edge carries a malformed atom address: {h:?}")
            }
        }
    }
}

impl std::error::Error for ArchiveLensError {}

// =============================================================================
// Conversions — live graph ⇄ rkyv mirror.
// =============================================================================

impl ArchivedEdgeTarget {
    fn from_live(target: &EdgeTarget) -> Self {
        match target {
            EdgeTarget::Local(name) => ArchivedEdgeTarget::Local(name.clone()),
            EdgeTarget::Grounded { ontology, atom } => ArchivedEdgeTarget::Grounded {
                ontology: ontology.clone(),
                atom_hex: atom.to_hex(),
            },
        }
    }

    fn into_live(self) -> Result<EdgeTarget, ArchiveLensError> {
        match self {
            ArchivedEdgeTarget::Local(name) => Ok(EdgeTarget::Local(name)),
            ArchivedEdgeTarget::Grounded { ontology, atom_hex } => {
                let atom = ContentAddress::from_hex(&atom_hex)
                    .ok_or(ArchiveLensError::BadAtomAddress(atom_hex))?;
                Ok(EdgeTarget::Grounded { ontology, atom })
            }
        }
    }
}

impl ArchivedGeneratorAction {
    fn from_live(action: &GeneratorAction) -> Self {
        match action {
            GeneratorAction::Functor {
                map_object,
                map_morphism,
            } => ArchivedGeneratorAction::Functor {
                map_object: map_object.clone(),
                map_morphism: map_morphism.clone(),
            },
            GeneratorAction::NaturalTransformation { components } => {
                ArchivedGeneratorAction::NaturalTransformation {
                    components: components.clone(),
                }
            }
            GeneratorAction::Lens { view, get, put } => ArchivedGeneratorAction::Lens {
                view: view.clone(),
                get: get.clone(),
                put: put.clone(),
            },
            GeneratorAction::Adjunction {
                left_map_object,
                right_map_object,
                unit,
                counit,
            } => ArchivedGeneratorAction::Adjunction {
                left_map_object: left_map_object.clone(),
                right_map_object: right_map_object.clone(),
                unit: unit.clone(),
                counit: counit.clone(),
            },
        }
    }

    fn into_live(self) -> GeneratorAction {
        match self {
            ArchivedGeneratorAction::Functor {
                map_object,
                map_morphism,
            } => GeneratorAction::Functor {
                map_object,
                map_morphism,
            },
            ArchivedGeneratorAction::NaturalTransformation { components } => {
                GeneratorAction::NaturalTransformation { components }
            }
            ArchivedGeneratorAction::Lens { view, get, put } => {
                GeneratorAction::Lens { view, get, put }
            }
            ArchivedGeneratorAction::Adjunction {
                left_map_object,
                right_map_object,
                unit,
                counit,
            } => GeneratorAction::Adjunction {
                left_map_object,
                right_map_object,
                unit,
                counit,
            },
        }
    }
}

impl ArchivedDefinition {
    fn from_live(def: &Definition) -> Self {
        ArchivedDefinition {
            kind: def.kind.clone(),
            name: def.name.clone(),
            edges: def
                .edges
                .iter()
                .map(|(rel, target)| (rel.clone(), ArchivedEdgeTarget::from_live(target)))
                .collect(),
            axioms: def.axioms.clone(),
            lexical: def.lexical.clone(),
        }
    }

    fn into_live(self) -> Result<Definition, ArchiveLensError> {
        let mut edges = Vec::with_capacity(self.edges.len());
        for (rel, target) in self.edges {
            edges.push((rel, target.into_live()?));
        }
        Ok(Definition {
            kind: self.kind,
            name: self.name,
            edges,
            axioms: self.axioms,
            lexical: self.lexical,
        })
    }
}

impl ArchivedConnection {
    fn from_live(conn: &Connection) -> Self {
        ArchivedConnection {
            kind: conn.kind.clone(),
            source: conn.source.clone(),
            target: conn.target.clone(),
            action: ArchivedGeneratorAction::from_live(&conn.action),
            laws: conn.laws.clone(),
        }
    }

    fn into_live(self) -> Connection {
        Connection {
            kind: self.kind,
            source: self.source,
            target: self.target,
            action: self.action.into_live(),
            laws: self.laws,
        }
    }
}

impl ArchivedArchive {
    /// Project a live [`Archive`] into its `rkyv` mirror (the forgetful
    /// direction: the address-bearing wire representation is dropped, grounded
    /// atoms become hex).
    pub fn from_live(archive: &Archive) -> Self {
        ArchivedArchive {
            nodes: archive
                .nodes
                .iter()
                .map(ArchivedDefinition::from_live)
                .collect(),
            connections: archive
                .connections
                .iter()
                .map(ArchivedConnection::from_live)
                .collect(),
        }
    }

    /// Rebuild a live [`Archive`] from this mirror. Fallible: a malformed
    /// grounded-atom hex fails closed ([`ArchiveLensError::BadAtomAddress`]).
    pub fn into_live(self) -> Result<Archive, ArchiveLensError> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for node in self.nodes {
            nodes.push(node.into_live()?);
        }
        Ok(Archive {
            nodes,
            connections: self
                .connections
                .into_iter()
                .map(ArchivedConnection::into_live)
                .collect(),
        })
    }
}

// =============================================================================
// The leaf lens — Archive ⇄ ArchivedArchive as an `RkyvLens` instance.
// =============================================================================

/// PUT leg of the runtime-`Archive` leaf lens: project the live graph into its
/// `rkyv` mirror (the forgetful direction — the address-bearing wire
/// representation is dropped, grounded atoms become hex).
impl RkyvMirror<Archive> for ArchivedArchive {
    fn from_owned(archive: &Archive) -> Self {
        ArchivedArchive::from_live(archive)
    }
}

/// GET leg of the runtime-`Archive` leaf lens: rebuild the live graph from the
/// mirror. Fallible — a malformed grounded-atom hex fails closed
/// ([`ArchiveLensError::BadAtomAddress`]).
impl RkyvOwned<ArchivedArchive> for Archive {
    type Error = ArchiveLensError;
    fn from_mirror(mirror: ArchivedArchive) -> Result<Self, ArchiveLensError> {
        mirror.into_live()
    }
}

// =============================================================================
// ArchiveLens — the runtime `Archive` instance of the generic `RkyvLens`.
// =============================================================================

/// The zero-copy archived VIEW of the runtime graph — `rkyv`'s in-buffer form of
/// the [`ArchivedArchive`] mirror. A `&ArchivedArchiveView` borrows the validated
/// bytes directly (no owned rebuild); its fields mirror [`ArchivedArchive`] but
/// every leaf is its `rkyv` archived form (`ArchivedString`, `ArchivedVec`,
/// `ArchivedOption`, …). This is what [`RuntimeOntology`] reasons over in place.
///
/// [`RuntimeOntology`]: crate::ontology::RuntimeOntology
pub type ArchivedArchiveView = rkyv::Archived<ArchivedArchive>;

/// The zero-copy archived form of one [`ArchivedDefinition`] node — the element
/// type of [`ArchivedArchiveView`]'s `nodes`.
pub type ArchivedDefinitionView = rkyv::Archived<ArchivedDefinition>;

/// The zero-copy archived form of an [`ArchivedEdgeTarget`] — the target half of
/// an [`ArchivedDefinitionView`]'s `edges`. Read its local name with
/// [`archived_local_name`].
pub type ArchivedEdgeTargetView = rkyv::Archived<ArchivedEdgeTarget>;

/// The local target name of an archived edge target, or `None` for a grounded
/// (cross-ontology) target — the zero-copy analogue of
/// [`EdgeTarget::local_name`](crate::definition::EdgeTarget::local_name), for
/// traversers reading the graph straight out of the `rkyv` buffer. It forces the
/// foreign case to be handled explicitly, never silently read as a local name.
pub fn archived_local_name(target: &ArchivedEdgeTargetView) -> Option<&str> {
    match target {
        ArchivedArchivedEdgeTarget::Local(name) => Some(name.as_str()),
        ArchivedArchivedEdgeTarget::Grounded { .. } => None,
    }
}

/// The `(ontology, atom)` a GROUNDED (cross-ontology) archived edge target names,
/// or `None` for a local target — the zero-copy dual of [`archived_local_name`],
/// for a traverser reading a node's foreign-atom edges straight out of the `rkyv`
/// buffer. This is the query surface [`morphisms_from`](crate::ontology::RuntimeOntology::morphisms_from)
/// deliberately DROPS (a grounded target is not a local generator): the resolve
/// side reads it here, then resolves the atom against a connected ontology via the
/// generic [`AtomResolver`](crate::grounding::AtomResolver). A grounded target
/// whose `atom_hex` is not a valid content address is `None` (the same fail-closed
/// stance the owning `into_live` decode takes).
pub fn archived_grounded(target: &ArchivedEdgeTargetView) -> Option<(&str, ContentAddress)> {
    match target {
        ArchivedArchivedEdgeTarget::Local(_) => None,
        ArchivedArchivedEdgeTarget::Grounded { ontology, atom_hex } => {
            ContentAddress::from_hex(atom_hex.as_str()).map(|atom| (ontology.as_str(), atom))
        }
    }
}

/// The `rkyv` local-cache/query lens between a runtime [`Archive`] and its
/// zero-copy bytes — the runtime instance of the generic
/// [`RkyvLens`]`<`[`Archive`]`, `[`ArchivedArchive`]`>`.
/// Its methods are thin, type-fixing forwarders to that lens (the serialize /
/// validate-once / zero-copy / owning-decode boilerplate lives there, once). See
/// the [module docs](self) for why this is NOT the content-address form.
///
/// `get` flattens the generic [`RkyvLensError`]`<`[`ArchiveLensError`]`>` back
/// into a plain [`ArchiveLensError`] so the public GET surface the
/// [`RuntimeOntology`](crate::ontology::RuntimeOntology) reads is unchanged.
pub struct ArchiveLens;

/// The concrete lens for the runtime `Archive` instance.
type ArchiveRkyvLens = RkyvLens<Archive, ArchivedArchive>;

impl ArchiveLens {
    /// The lens PUT, keeping `rkyv`'s own 16-aligned buffer — the form
    /// [`RuntimeOntology`](crate::ontology::RuntimeOntology) stores and the
    /// alignment [`access`](Self::access) / [`access_unchecked`](Self::access_unchecked)
    /// require. **Not** the DAG-CBOR content-address form.
    pub fn put_aligned(archive: &Archive) -> rkyv::util::AlignedVec<16> {
        ArchiveRkyvLens::put_aligned(archive)
    }

    /// The lens PUT as a plain `Vec<u8>` — [`put_aligned`](Self::put_aligned)
    /// with the alignment guarantee dropped, for callers that only round-trip the
    /// bytes through [`get`](Self::get) or compare them (the lens-law axioms).
    pub fn put(archive: &Archive) -> Vec<u8> {
        ArchiveRkyvLens::put(archive)
    }

    /// The ZERO-COPY GET: `bytecheck`-validate `bytes` and return a borrowed
    /// [`ArchivedArchiveView`] over them — NO owned rebuild (contrast
    /// [`get`](Self::get)). `bytes` must be 16-aligned (an `AlignedVec<16>` as
    /// [`put_aligned`](Self::put_aligned) produces). Fail-closed on a corrupted /
    /// truncated / misaligned blob.
    ///
    /// This is Step 1c: the runtime validates ONCE here at materialize, then
    /// serves every hot query through [`access_unchecked`](Self::access_unchecked).
    pub fn access(bytes: &[u8]) -> Result<&ArchivedArchiveView, ArchiveLensError> {
        ArchiveRkyvLens::access(bytes).map_err(flatten_err)
    }

    /// The ZERO-COPY GET without re-validation — the hot query path.
    ///
    /// # Safety
    ///
    /// `bytes` must be a 16-aligned buffer previously accepted by
    /// [`access`](Self::access) (bytecheck-validated) and kept immutable since —
    /// the deliberate `access_unchecked` the runtime uses to pay bytecheck
    /// exactly once.
    pub unsafe fn access_unchecked(bytes: &[u8]) -> &ArchivedArchiveView {
        // SAFETY: forwarded to the caller's contract above — validated once,
        // immutable since.
        unsafe { ArchiveRkyvLens::access_unchecked(bytes) }
    }

    /// The lens GET: `bytecheck`-validate the bytes and materialize an owned
    /// [`Archive`], failing closed on a corrupted blob OR a malformed
    /// grounded-atom hex. The OWNING GET — kept for callers that genuinely need
    /// an owned [`Archive`] (the content root is derived over [`Definition`]
    /// addressing, absent from the archived view).
    pub fn get(bytes: &[u8]) -> Result<Archive, ArchiveLensError> {
        ArchiveRkyvLens::get(bytes).map_err(flatten_err)
    }
}

/// Flatten the generic [`RkyvLensError`]`<`[`ArchiveLensError`]`>` into a plain
/// [`ArchiveLensError`]: the lens `Rkyv` leg becomes [`ArchiveLensError::Rkyv`],
/// the leaf `Conversion` leg is already an [`ArchiveLensError`]
/// ([`ArchiveLensError::BadAtomAddress`]). Keeps the public GET surface byte-
/// identical to the pre-generalization one.
fn flatten_err(e: RkyvLensError<ArchiveLensError>) -> ArchiveLensError {
    match e {
        RkyvLensError::Rkyv(m) => ArchiveLensError::Rkyv(m),
        RkyvLensError::Conversion(inner) => inner,
    }
}

// =============================================================================
// Lens-law axioms — behind `emit` (the pr4xis axiom machinery lives there).
// =============================================================================

#[cfg(feature = "emit")]
pub use axioms::{ArchiveLensDeterminism, ArchiveLensGetPut, ArchiveLensPutGet};

#[cfg(feature = "emit")]
mod axioms {
    use super::*;
    use crate::lens::rkyv_lens::{determinism_holds, getput_holds, putget_holds};

    use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
    use pr4xis::ontology::Axiom;

    /// Witness archives exercising every mirror branch — an empty archive, a
    /// node-only archive, a rich archive with `Local` + `Grounded` edges,
    /// non-empty `axioms`, `Some`/`None` lexicals, and one connection of each
    /// [`GeneratorAction`] family. Shared by the axioms and the unit tests.
    pub(super) fn witness_archives() -> Vec<Archive> {
        let grounded_atom = ContentAddress::of(b"a foreign ontology's atom definition");

        let rich = Archive {
            nodes: vec![
                Definition {
                    kind: "Concept".into(),
                    name: "Employer".into(),
                    edges: vec![
                        ("Subsumption".into(), EdgeTarget::Local("Agent".into())),
                        (
                            "denotes".into(),
                            EdgeTarget::Grounded {
                                ontology: "english_wordnet".into(),
                                atom: grounded_atom,
                            },
                        ),
                    ],
                    axioms: vec!["EmployerIsAgent".into(), "EmployerHiresEmployee".into()],
                    lexical: Some("One who employs.".into()),
                },
                Definition {
                    kind: "Concept".into(),
                    name: "Agent".into(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: None,
                },
            ],
            connections: vec![
                Connection {
                    kind: "FullyFaithful".into(),
                    source: "Org".into(),
                    target: "Workforce".into(),
                    action: GeneratorAction::Functor {
                        map_object: vec![("Employer".into(), "Employer".into())],
                        map_morphism: vec![("Subsumption".into(), "Subsumption".into())],
                    },
                    laws: vec!["FunctorIdentityLaw".into(), "FunctorCompositionLaw".into()],
                },
                Connection {
                    kind: "NatTrans".into(),
                    source: "F".into(),
                    target: "G".into(),
                    action: GeneratorAction::NaturalTransformation {
                        components: vec![("Employer".into(), "eta_Employer".into())],
                    },
                    laws: vec!["Naturality".into()],
                },
                Connection {
                    kind: "Decompile".into(),
                    source: "Source".into(),
                    target: "Archive".into(),
                    action: GeneratorAction::Lens {
                        view: "Source".into(),
                        get: "parse".into(),
                        put: "generate".into(),
                    },
                    laws: vec!["GetPut".into(), "PutGet".into()],
                },
                Connection {
                    kind: "FreeForgetful".into(),
                    source: "Grph".into(),
                    target: "Cat".into(),
                    action: GeneratorAction::Adjunction {
                        left_map_object: vec![("g".into(), "Free(g)".into())],
                        right_map_object: vec![("c".into(), "U(c)".into())],
                        unit: vec![("g".into(), "eta_g".into())],
                        counit: vec![("c".into(), "eps_c".into())],
                    },
                    laws: vec!["TriangleLeft".into(), "TriangleRight".into()],
                },
            ],
        };

        vec![
            Archive::new(),
            Archive {
                nodes: vec![Definition {
                    kind: "Form".into(),
                    name: "employer".into(),
                    edges: vec![],
                    axioms: vec![],
                    lexical: Some("employer".into()),
                }],
                connections: vec![],
            },
            rich,
        ]
    }

    /// GetPut leg of the `ArchiveLens` well-behaved lens: for bytes `b`
    /// canonically produced by [`ArchiveLens::put`], `put(get(b)) == b` — the
    /// serialized form re-emitted from what it decodes to is byte-identical, so
    /// the cache blob is stable under a decode/re-encode round-trip. Foster,
    /// Greenwald, Moore, Pierce & Schmitt (2007) §3, Definition 3.2.
    pub struct ArchiveLensGetPut;

    impl Axiom for ArchiveLensGetPut {
        fn verify(&self) -> Verdict {
            if getput_holds::<Archive, ArchivedArchive>(&witness_archives()) {
                Ok(Box::new(SimpleProof::new(self.meta())))
            } else {
                Err(Box::new(SimpleCounterexample::new(self.meta())))
            }
        }

        pr4xis::axiom_meta!(
            "ArchiveLensGetPut",
            "put(get(b)) == b for canonically-produced rkyv archive-lens bytes",
            "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
        );
    }

    pr4xis::register_axiom!(
        ArchiveLensGetPut,
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §3, Definition 3.2"
    );

    /// PutGet leg of the `ArchiveLens` well-behaved lens: `get(put(a))` recovers
    /// `a` — an archive round-trips through the `rkyv` cache form with its full
    /// query image (nodes, edges incl. grounded atoms, axioms, lexicals,
    /// connections) intact. Foster, Greenwald, Moore, Pierce & Schmitt (2007)
    /// §3, Definition 3.2.
    pub struct ArchiveLensPutGet;

    impl Axiom for ArchiveLensPutGet {
        fn verify(&self) -> Verdict {
            if putget_holds::<Archive, ArchivedArchive>(&witness_archives()) {
                Ok(Box::new(SimpleProof::new(self.meta())))
            } else {
                Err(Box::new(SimpleCounterexample::new(self.meta())))
            }
        }

        pr4xis::axiom_meta!(
            "ArchiveLensPutGet",
            "get(put(a)) == a: an Archive round-trips through the rkyv cache form with its full query image intact",
            "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
        );
    }

    pr4xis::register_axiom!(
        ArchiveLensPutGet,
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §3, Definition 3.2"
    );

    /// Determinism leg of the `ArchiveLens` well-behaved lens: `put(a) == put(a)`
    /// — the `rkyv` cache bytes are a deterministic function of the [`Archive`]
    /// alone (no build-order or address nondeterminism), the property that
    /// underwrites [`ArchiveLensGetPut`]. Foster, Greenwald, Moore, Pierce &
    /// Schmitt (2007) §3, Definition 3.2.
    pub struct ArchiveLensDeterminism;

    impl Axiom for ArchiveLensDeterminism {
        fn verify(&self) -> Verdict {
            if determinism_holds::<Archive, ArchivedArchive>(&witness_archives()) {
                Ok(Box::new(SimpleProof::new(self.meta())))
            } else {
                Err(Box::new(SimpleCounterexample::new(self.meta())))
            }
        }

        pr4xis::axiom_meta!(
            "ArchiveLensDeterminism",
            "put(a) == put(a): the rkyv cache bytes are a deterministic function of the Archive alone",
            "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
        );
    }

    pr4xis::register_axiom!(
        ArchiveLensDeterminism,
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

// =============================================================================
// Tests — behind `emit` so the pr4xis praxis_value machinery is available.
// =============================================================================

#[cfg(all(test, feature = "emit"))]
mod tests {
    use super::axioms::{
        ArchiveLensDeterminism, ArchiveLensGetPut, ArchiveLensPutGet, witness_archives,
    };
    use super::*;

    use pr4xis::ontology::Axiom;
    use proptest::prelude::*;

    // A small REAL ontology — the emitted-category corpus fixture. Generated by
    // the same `ontology!` macro every domain ontology is.
    pr4xis::ontology! {
        name: "Org",
        source: "pr4xis-runtime archive-lens test fixture",
        concepts: [Employer, Employee, Person, Agent],
        labels: {
            Employer: ("en", "Employer", "One who employs."),
            Employee: ("en", "Employee", "One who is employed."),
            Person: ("en", "Person", "A human being."),
            Agent: ("en", "Agent", "One who acts."),
        },
        is_a: [
            (Employer, Person),
            (Employee, Person),
            (Person, Agent),
        ],
    }

    /// Unit round-trip on a small hand-built archive: `get(put(a)) == a`.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn small_archive_round_trips_through_the_lens() {
        let archive = Archive {
            nodes: vec![Definition {
                kind: "Concept".into(),
                name: "Employer".into(),
                edges: vec![("Subsumption".into(), EdgeTarget::Local("Agent".into()))],
                axioms: vec!["EmployerIsAgent".into()],
                lexical: Some("One who employs.".into()),
            }],
            connections: vec![Connection {
                kind: "FullyFaithful".into(),
                source: "Org".into(),
                target: "Workforce".into(),
                action: GeneratorAction::Functor {
                    map_object: vec![("Employer".into(), "Employer".into())],
                    map_morphism: vec![],
                },
                laws: vec!["FunctorIdentityLaw".into()],
            }],
        };
        let bytes = ArchiveLens::put(&archive);
        let got = ArchiveLens::get(&bytes).expect("canonical rkyv bytes must decode");
        assert_eq!(got, archive, "get(put(a)) must recover a");
    }

    /// A grounded (cross-ontology) edge round-trips: the foreign atom's content
    /// address survives the hex mirror exactly.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn grounded_edge_round_trips_through_the_lens() {
        let atom = ContentAddress::of(b"some english form atom");
        let archive = Archive {
            nodes: vec![Definition {
                kind: "Concept".into(),
                name: "Employer".into(),
                edges: vec![(
                    "denotes".into(),
                    EdgeTarget::Grounded {
                        ontology: "english_wordnet".into(),
                        atom,
                    },
                )],
                axioms: vec![],
                lexical: None,
            }],
            connections: vec![],
        };
        let got = ArchiveLens::get(&ArchiveLens::put(&archive)).expect("decodes");
        assert_eq!(got, archive);
        // The grounded atom address survived intact (not just some address).
        match &got.nodes[0].edges[0].1 {
            EdgeTarget::Grounded { atom: back, .. } => assert_eq!(*back, atom),
            other => panic!("expected a grounded edge, got {other:?}"),
        }
    }

    /// The two lens-law axioms hold — the runnable GetPut/PutGet witnesses.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archive_lens_laws_hold() {
        assert!(
            ArchiveLensGetPut.verify().is_ok(),
            "put(get(b)) == b must hold over the witness archives"
        );
        assert!(
            ArchiveLensPutGet.verify().is_ok(),
            "get(put(a)) == a must hold over the witness archives"
        );
        assert!(
            ArchiveLensDeterminism.verify().is_ok(),
            "put(a) == put(a) must hold over the witness archives"
        );
    }

    /// Fail-closed teeth: `get` rejects a truncated/corrupted blob rather than
    /// materializing unsound references.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn get_rejects_a_corrupted_blob() {
        let archive = witness_archives().pop().expect("a witness archive");
        let mut bytes = ArchiveLens::put(&archive);
        // Truncate to half — the archived layout no longer validates.
        bytes.truncate(bytes.len() / 2);
        assert!(
            ArchiveLens::get(&bytes).is_err(),
            "a truncated rkyv blob must fail closed"
        );
    }

    /// A real emitted category round-trips through the lens, and the cache form
    /// is stable (`put(get(b)) == b`) — the emitted-category corpus check.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn emitted_ontology_round_trips_through_the_lens() {
        let archive = crate::emit::emit::<OrgCategory>();
        let bytes = ArchiveLens::put(&archive);
        let got = ArchiveLens::get(&bytes).expect("emitted-ontology cache bytes must decode");
        assert_eq!(
            got, archive,
            "an emitted ontology round-trips through the lens"
        );
        assert_eq!(
            ArchiveLens::put(&got),
            bytes,
            "the cache form is stable under decode/re-encode"
        );
    }

    /// The ZERO-COPY GET: `access` over `put_aligned` bytes returns a borrowed
    /// view whose node/edge/lexical image equals the live archive's — no owned
    /// rebuild. This is the Step 1c query surface.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn access_reads_the_archive_image_zero_copy() {
        let archive = witness_archives().pop().expect("the rich witness archive");
        let buf = ArchiveLens::put_aligned(&archive);
        let view = ArchiveLens::access(buf.as_slice()).expect("canonical bytes validate");

        // Same node set, in order, with names/kinds/lexicals/edge local-names
        // read straight from the buffer.
        assert_eq!(view.nodes.len(), archive.nodes.len());
        for (node, live) in view.nodes.iter().zip(&archive.nodes) {
            assert_eq!(node.name.as_str(), live.name, "node name");
            assert_eq!(node.kind.as_str(), live.kind, "node kind");
            assert_eq!(
                node.lexical.as_deref(),
                live.lexical.as_deref(),
                "node lexical"
            );
            assert_eq!(node.edges.len(), live.edges.len(), "edge count");
            for (edge, live_edge) in node.edges.iter().zip(&live.edges) {
                // Archived edges are `ArchivedTuple2(rel, target)`.
                assert_eq!(edge.0.as_str(), live_edge.0, "edge relation");
                assert_eq!(
                    archived_local_name(&edge.1),
                    live_edge.1.local_name(),
                    "edge local name (grounded ⇒ None)"
                );
            }
        }
        assert_eq!(view.connections.len(), archive.connections.len());
    }

    /// `access` fails closed on a truncated blob — the zero-copy view is never
    /// handed out over unsound bytes (the `access_unchecked` precondition).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn access_rejects_a_corrupted_blob() {
        let archive = witness_archives().pop().expect("a witness archive");
        let buf = ArchiveLens::put_aligned(&archive);
        let mut truncated = rkyv::util::AlignedVec::<16>::new();
        truncated.extend_from_slice(&buf.as_slice()[..buf.len() / 2]);
        assert!(
            ArchiveLens::access(truncated.as_slice()).is_err(),
            "a truncated rkyv blob must fail closed at access"
        );
    }

    // --- Corpus-wide round-trip proptest --------------------------------------

    fn edge_target_strategy() -> impl Strategy<Value = EdgeTarget> {
        prop_oneof![
            "[A-Za-z_]{1,10}".prop_map(EdgeTarget::Local),
            ("[a-z_]{1,10}", any::<[u8; 32]>()).prop_map(|(ontology, seed)| {
                EdgeTarget::Grounded {
                    ontology,
                    // `of` yields a valid 64-hex address, so the hex mirror
                    // always round-trips (the space we're fuzzing over).
                    atom: ContentAddress::of(&seed),
                }
            }),
        ]
    }

    fn definition_strategy() -> impl Strategy<Value = Definition> {
        (
            "[A-Za-z]{1,8}",
            "[A-Za-z_]{1,10}",
            prop::collection::vec(("[A-Za-z]{1,8}", edge_target_strategy()), 0..4),
            prop::collection::vec("[A-Za-z]{1,10}", 0..3),
            prop::option::of("[A-Za-z .]{0,20}"),
        )
            .prop_map(|(kind, name, edges, axioms, lexical)| Definition {
                kind,
                name,
                edges,
                axioms,
                lexical,
            })
    }

    fn table() -> impl Strategy<Value = Vec<(String, String)>> {
        prop::collection::vec(("[A-Za-z]{1,8}", "[A-Za-z]{1,8}"), 0..3)
    }

    fn generator_action_strategy() -> impl Strategy<Value = GeneratorAction> {
        prop_oneof![
            (table(), table()).prop_map(|(map_object, map_morphism)| GeneratorAction::Functor {
                map_object,
                map_morphism
            }),
            table().prop_map(|components| GeneratorAction::NaturalTransformation { components }),
            ("[A-Za-z]{1,8}", "[A-Za-z]{1,8}", "[A-Za-z]{1,8}")
                .prop_map(|(view, get, put)| GeneratorAction::Lens { view, get, put }),
            (table(), table(), table(), table()).prop_map(
                |(left_map_object, right_map_object, unit, counit)| GeneratorAction::Adjunction {
                    left_map_object,
                    right_map_object,
                    unit,
                    counit,
                }
            ),
        ]
    }

    fn connection_strategy() -> impl Strategy<Value = Connection> {
        (
            "[A-Za-z]{1,10}",
            "[A-Za-z]{1,8}",
            "[A-Za-z]{1,8}",
            generator_action_strategy(),
            prop::collection::vec("[A-Za-z]{1,12}", 0..3),
        )
            .prop_map(|(kind, source, target, action, laws)| Connection {
                kind,
                source,
                target,
                action,
                laws,
            })
    }

    fn archive_strategy() -> impl Strategy<Value = Archive> {
        (
            prop::collection::vec(definition_strategy(), 0..5),
            prop::collection::vec(connection_strategy(), 0..3),
        )
            .prop_map(|(nodes, connections)| Archive { nodes, connections })
    }

    proptest! {
        /// Corpus-wide: over the full space of archives (nodes with local +
        /// grounded edges, every `GeneratorAction` family, connections),
        /// `get(put(a)) == a` — the query-relevant content round-trips exactly.
        #[test]
        fn prop_archive_lens_round_trips(archive in archive_strategy()) {
            let bytes = ArchiveLens::put(&archive);
            let got = ArchiveLens::get(&bytes).expect("canonical rkyv bytes must decode");
            prop_assert_eq!(got, archive);
        }
    }

    pr4xis::register_praxis_value!(prop_archive_lens_round_trips, Deterministic);
}
