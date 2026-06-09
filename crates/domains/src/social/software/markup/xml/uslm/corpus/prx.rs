//! `.prx.gz` — the self-describing, load-validated distribution envelope
//! for the loaded U.S. Code corpus.
//!
//! This is the **USC second consumer** of the
//! [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive)
//! ontology — the legislative twin of the OWL leaf
//! ([`crate::social::software::markup::xml::owl::prx`]). It realises the
//! SAME archive ontology (same seven runnable axioms, same content-addressed
//! load gate, same OMV/PROV-O-grounded metadata schema) over the U.S. Code
//! rather than over an OWL vocabulary — a parallel *realisation*, never a
//! parallel envelope module. Every shared primitive
//! ([`source_content_hash`],
//! [`gzip`] /
//! [`gunzip`],
//! `raw_hash::verify`, [`RoundTripFidelity`], `PrxMetadata`,
//! [`RawSource`]) is reused as-is; only the two private monomorphic gate
//! *legs* are mirrored, and exactly ONE structural field is added
//! ([`UsCodePrxEnvelope::aux`]) because the U.S. Code carries subdivision
//! DEPTH the OWL flat edge tables do not.
//!
//! ## The one delta: subdivision depth
//!
//! OWL archives a flat [`OwnedCodegenData`] — entities plus `(u64, u64)`
//! edge tables. The U.S. Code section leaf is the same flat shape, but each
//! `<section>` roots a typed subdivision tree (`<subsection>` /
//! `<paragraph>` / `<subparagraph>` / `<clause>` / `<subclause>` / `<item>`
//! / `<subitem>`), each node with its own USLM URN and a parent↔child
//! `Composes` edge. That tree is the corpus
//! [`UscSectionAux`] side-channel
//! [`UsCode::from_codegen_with_aux`](super::UsCode::from_codegen_with_aux)
//! attaches. The archive mirrors it with an owned, rkyv-serializable
//! [`OwnedUscSectionAux`] / [`OwnedUscSubdivision`] tree.
//!
//! ## Corpus-faithful, not parse-faithful (the round-trip target)
//!
//! [`OwnedUscSubdivision`] mirrors the **corpus**
//! [`UscSubdivision`] (the 7-field
//! node the corpus API exposes), NOT the richer parse
//! [`UsCodeSubdivision`] (whose
//! `heading_runs` / `chapeau_runs` / `content_runs` / `refs` / `def_blocks`
//! / `markers` / `amendments` are inline-run detail). Those parse-only
//! fields are deliberately NOT archived: the corpus projection
//! ([`UsCode::from_uslm_titles_owned`](super::UsCode::from_uslm_titles_owned)
//! → `subdivisions_to_static`) already discards them, so the round-trip
//! equivalence target is the **`UsCode` corpus value** (`section_count`,
//! `subdivision_count`, `section_by_urn`, every `UscComposesEdge`,
//! `to_statute`) — not the `UsCodeTitle` parse.
//!
//! Byte-exact source fidelity (`hash(out) == hash(in)`, the #186 invariant) is
//! GRAPH-FAITHFUL for any title bound to a
//! [`UslmGraphFaithfulLens`](super::super::lens::writer): that title's `.prx`
//! regenerates the exact source bytes from the typed [`UsCodeTitle`] ontology
//! plus a content-addressed concrete-syntax complement ([`UscGraphFaithful`] —
//! the `<?xml-stylesheet?>` prolog PI, the `<!DOCTYPE>`, the root `xmlns`
//! declarations, the §2.4 inter-element white-space, the §3.1 intra-tag layout,
//! the §4.6 entity-reference form, the §2.11 end-of-line form, the source
//! attribute sequences), with NO stored raw blob. The capture/reconstruct pair
//! ([`capture_uslm_complement`] / [`reconstruct_uslm_source`], parser
//! `source_syntax` residue + the title-AGNOSTIC USLM structural writer
//! [`write_uslm`](super::super::lens::writer::write_uslm)) is a byte-exact
//! inverse, proven over the real on-disk titles by the writer's
//! `flipped_titles_reconstruct_byte_exact` gate and the all-sources round-trip
//! integration test. Such a title emits
//! [`RoundTripFidelity::ByteExactGraphFaithful`].
//!
//! A title with NO graph-faithful registration HONESTLY DEGRADES to the universal
//! floor [`RoundTripFidelity::RawBytesComplementFloor`] — byte-exact via the
//! content-addressed [`RawSource::blob`] (the whole unzipped USLM XML), exactly
//! as the OWL leaf does. (Either an uncovered USLM family `write_uslm` cannot yet
//! reconstruct — a `<continuation>` flush-text run or `<def>` / `<marker>` /
//! `<ins>` amendment markup — or, for a title the title-agnostic writer DOES
//! handle, a deliberate floor held off the always-run byte-exact gate for the CI
//! per-test budget.) `data` + `aux` are the runtime reasoning view, carried
//! unchanged in BOTH tiers; the source is regenerated from the graph in the
//! graph-faithful tier and from `raw.blob` in the floor tier. The degrade is
//! never a silent lie: the floor tier is explicit in `mode`, and the completeness
//! meter only declares a title graph-faithful when a `UslmGraphFaithfulLens` is
//! registered for it. A MALFORMED source stays a hard error in both tiers.
//!
//! ## Citations
//!
//! - LRC, *USLM XML User Guide* §V (USC URN hierarchy / subdivision depth).
//! - Foster, Greenwald, Moore, Pierce & Schmitt (2007) "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2 (the strict
//!   byte-exact PutGet law the graph-faithful tier satisfies).
//! - NIST (2015) FIPS 180-4 §6.2 (SHA-256); Dolstra (2006)
//!   content-addressing.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::codegen_data::CodegenData;

use super::{
    SubdivisionKind, UsCode, UsCodeSubdivision, UsCodeTitle, UscComposesEdge, UscSectionAux,
    UscSubdivision,
};
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::raw_hash;
use crate::formal::meta::identifier_format::Identifier;
use crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept;
use crate::formal::meta::well_behaved_lens::RoundTripFidelity;
use crate::social::software::markup::xml::uslm::lens::read_uslm_title;
// The graph-faithful capture/reconstruct pair (slices U1–U5) + the
// concrete-syntax complement they thread — the USC analogue of the WN-LMF
// `capture_wn_complement` / `reconstruct_wn_lmf_source` / `WnSyntaxComplement`.
use crate::social::software::markup::xml::uslm::lens::writer::{
    UslmReconstructError, UslmSyntaxComplement, capture_uslm_complement, reconstruct_uslm_source,
};
// Shared archive primitives — reused VERBATIM from the OWL leaf (the genuine
// second-consumer boundary): the content-hash, the gzip/rkyv codecs, the
// owned codegen mirror, the raw-source complement, and the error type. Only
// the two PRIVATE monomorphic gate legs below (`usc_verify_content_address`
// / `usc_admit_validated`) are mirrored, because OWL's are `&PrxEnvelope`-
// typed and return `LoadedOwlVocabulary`.
use crate::social::software::markup::xml::owl::prx::{
    EmittedArtifact, OwnedCodegenData, PrxError, RawSource, gunzip, gzip, prx_archive_address,
    source_content_hash,
};
use crate::social::software::markup::xml::succinct::{
    get_blob, get_cv, get_dict_fc, get_opt, get_varint, put_blob, put_cv, put_dict_fc, put_opt,
    put_varint,
};

// =============================================================================
// Owned aux mirror — the rkyv-serializable twin of the corpus subdivision tree.
// =============================================================================

/// Owned, rkyv-serializable mirror of the corpus
/// [`UscSubdivision`].
///
/// Field-for-field identical except that every `&'static str` becomes an
/// owned [`String`], the typed [`SubdivisionKind`] becomes its canonical
/// USLM tag name ([`SubdivisionKind::tag`], lowered losslessly and re-typed
/// on load via the total, XSD-free [`SubdivisionKind::parse`]), the typed
/// [`Identifier`] URN becomes its raw URN string (re-tagged `UslmUrn` on
/// load), and `children` recurse. This is the subdivision-depth payload the
/// archive carries that the OWL flat [`OwnedCodegenData`] cannot.
// `OwnedUscSubdivision` is recursive (`children: Vec<Self>`), so the rkyv
// derive needs `#[rkyv(omit_bounds)]` on the recursive field to break the
// `Self: Archive` bound cycle, plus the manual non-recursive container
// bounds the omitted derive would otherwise have supplied — the canonical
// rkyv 0.8 recursive-type pattern (rkyv `examples/json_like_schema.rs`).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
))]
#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(
    __C: rkyv::validation::ArchiveContext,
    __C::Error: rkyv::rancor::Source,
)))]
pub struct OwnedUscSubdivision {
    /// Full typed USLM URN string — e.g. `/us/usc/t18/s1514A/a/1/A`.
    /// Re-tagged [`IdentifierFormatConcept::UslmUrn`] on load.
    pub urn: String,
    /// The [`SubdivisionKind`] variant's canonical USLM tag name (e.g.
    /// `"subsection"`, `"paragraph"`). Lowered via [`SubdivisionKind::tag`];
    /// re-typed on load via the total, XSD-free [`SubdivisionKind::parse`]
    /// (NOT `from_xsd_element`, which would make the load path depend on the
    /// loaded XSD ontology).
    pub kind: String,
    /// `<num>` value verbatim — e.g. `"a"`, `"1"`, `"A"`.
    pub num: String,
    /// `<heading>` plain text, when present.
    pub heading: Option<String>,
    /// `<chapeau>` introductory text, when present.
    pub chapeau: Option<String>,
    /// `<content>` leaf body text, when present.
    pub content: Option<String>,
    /// Nested subdivisions, in USLM document order.
    #[rkyv(omit_bounds)]
    pub children: Vec<OwnedUscSubdivision>,
}

/// Owned, rkyv-serializable mirror of the corpus
/// [`UscSectionAux`]: one section's
/// subdivision tree plus its `Composes`-edge list.
///
/// `relations` are stored as `(child_urn, parent_urn)` pairs — redundant
/// with the tree (re-derivable by the same document-order DFS
/// `subdivisions_to_static` runs) but carried explicitly so the load path
/// reconstructs exactly the archived aux, and emitted in that DFS order so
/// the `MerkleRoot` over the rkyv bytes is reproducible.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct OwnedUscSectionAux {
    /// USLM URN of the section this aux record describes.
    pub urn: String,
    /// Subdivision tree rooted at the section, document order.
    pub subdivisions: Vec<OwnedUscSubdivision>,
    /// `Composes` edges `(child_urn, parent_urn)` across the whole tree,
    /// in the document-order DFS `subdivisions_to_static` emits.
    pub relations: Vec<(String, String)>,
}

impl OwnedUscSubdivision {
    /// Promote one owned subdivision node into the `&'static`
    /// [`UscSubdivision`] the corpus API requires, recursing into children.
    ///
    /// [`Box::leak`] gives the process-lifetime `&'static` lifetimes (the
    /// same trade `subdivisions_to_static` and the `OnceLock`-cached corpus
    /// singleton make). The `kind` string is re-typed through the total,
    /// XSD-free [`SubdivisionKind::parse`]; an unknown tag is a corrupt
    /// archive, but the load gate re-derives and verifies the `MerkleRoot`
    /// over these very bytes BEFORE materialization, so a non-canonical tag
    /// cannot reach this path under a trusted pin.
    fn to_static(&self) -> UscSubdivision {
        let urn_str: &'static str = Box::leak(self.urn.clone().into_boxed_str());
        let kind = SubdivisionKind::parse(&self.kind).expect(
            "subdivision kind must be a canonical USLM tag — the load gate \
             content-verifies these bytes before materialization",
        );
        let num: &'static str = Box::leak(self.num.clone().into_boxed_str());
        let heading: Option<&'static str> = self
            .heading
            .as_ref()
            .map(|h| -> &'static str { Box::leak(h.clone().into_boxed_str()) });
        let chapeau: Option<&'static str> = self
            .chapeau
            .as_ref()
            .map(|c| -> &'static str { Box::leak(c.clone().into_boxed_str()) });
        let content: Option<&'static str> = self
            .content
            .as_ref()
            .map(|c| -> &'static str { Box::leak(c.clone().into_boxed_str()) });
        let children_vec: Vec<UscSubdivision> = self.children.iter().map(Self::to_static).collect();
        let children: &'static [UscSubdivision] = Box::leak(children_vec.into_boxed_slice());
        let urn = Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, urn_str);
        UscSubdivision {
            urn,
            kind,
            num,
            heading,
            chapeau,
            content,
            children,
        }
    }
}

/// Promote a slice of owned section-aux records into the `&'static`
/// [`UscSectionAux`] slice [`UsCode::from_codegen_with_aux`] consumes.
///
/// Each section's subdivision tree is leaked via [`OwnedUscSubdivision::to_static`]
/// and its stored `(child, parent)` relations are leaked into
/// [`UscComposesEdge`]s — re-tagging URNs as `UslmUrn` so the section's
/// `section_by_urn` typed-format guard holds. Mirrors `subdivisions_to_static`'s
/// leak set so an archive-materialized corpus is identity-identical to the
/// parse-materialized one.
///
/// [`UsCode::from_codegen_with_aux`]: super::UsCode::from_codegen_with_aux
fn to_aux_leaked(aux: &[OwnedUscSectionAux]) -> &'static [UscSectionAux] {
    let mut out: Vec<UscSectionAux> = Vec::with_capacity(aux.len());
    for entry in aux {
        let urn: &'static str = Box::leak(entry.urn.clone().into_boxed_str());
        let subs_vec: Vec<UscSubdivision> = entry
            .subdivisions
            .iter()
            .map(OwnedUscSubdivision::to_static)
            .collect();
        let subdivisions: &'static [UscSubdivision] = Box::leak(subs_vec.into_boxed_slice());
        let rels_vec: Vec<UscComposesEdge> = entry
            .relations
            .iter()
            .map(|(from, to)| UscComposesEdge {
                from_urn: Box::leak(from.clone().into_boxed_str()),
                to_urn: Box::leak(to.clone().into_boxed_str()),
            })
            .collect();
        let relations: &'static [UscComposesEdge] = Box::leak(rels_vec.into_boxed_slice());
        out.push(UscSectionAux {
            urn,
            subdivisions,
            relations,
        });
    }
    Box::leak(out.into_boxed_slice())
}

// =============================================================================
// UscGraphFaithful — the typed ontology + concrete-syntax complement, the
// graph-faithful reconstruction payload (no stored raw blob).
// =============================================================================

/// The graph-faithful reconstruction payload: the typed USLM [`UsCodeTitle`]
/// ontology PLUS the concrete-syntax [`UslmSyntaxComplement`] the byte-exact
/// `put` ([`reconstruct_uslm_source`]) re-applies. Present in a
/// [`UsCodePrxEnvelope`] iff `mode == ByteExactGraphFaithful`.
///
/// The USC realisation of #186's graph-faithful tier (the exact analogue of the
/// WN-LMF
/// [`WnGraphFaithful`](crate::social::software::markup::xml::lmf::prx::WnGraphFaithful)):
/// the source bytes are regenerated from the ONTOLOGY GRAPH (`title`) plus a
/// content-addressed SYNTAX residue (`complement`) — the `<?xml-stylesheet?>`
/// prolog PI, the `<!DOCTYPE>`, the root `xmlns` declarations, the §2.4
/// inter-element white-space, the §3.1 intra-tag layout, the §4.6
/// entity-reference form, the §2.11 end-of-line form, and the source attribute
/// sequences — and NO stored raw blob (the `RawBytesComplementFloor`
/// constant-complement). The complement is concrete-syntax, NOT ontology: the
/// same `title` serialized two ways keeps one content address; only the
/// per-source `complement` differs. The capture/reconstruct pair
/// ([`capture_uslm_complement`] / [`reconstruct_uslm_source`]) is proven a
/// byte-exact inverse over the LITERAL on-disk `usc_title_1-pl-119-90.xml`
/// (slices U1–U5).
///
/// rkyv-serializable through the `prx`-gated derives on [`UsCodeTitle`] and
/// [`UslmSyntaxComplement`] (and the runtime/residue types they reference) —
/// additive `#[cfg_attr(feature = "prx", derive(rkyv::…))]` derives, so the
/// default + wasm builds are unaffected.
///
/// `Eq` is intentionally NOT derived (unlike the WN-LMF `WnGraphFaithful`): the
/// USC runtime aggregate [`UsCodeTitle`] is `PartialEq`-only by design, so this
/// payload and the [`UsCodePrxEnvelope`] that carries it are `PartialEq`-only.
/// rkyv (`Archive`/`Serialize`/`Deserialize`) does not require `Eq`, and the
/// envelope is never used as a `HashSet`/`BTreeSet` key.
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct UscGraphFaithful {
    /// The typed USLM title ontology — the GRAPH the source is regenerated from.
    /// Captured by [`capture_uslm_complement`] (the same `read_uslm_title` model
    /// `data` + `aux` project from), serialized here directly.
    pub title: UsCodeTitle,
    /// The concrete-syntax COMPLEMENT — the byte-affecting residue the typed
    /// ontology does not carry (prolog PI, DOCTYPE, namespaces, white-space
    /// layout, entity-reference form, end-of-line form, source attribute
    /// sequences). Re-applied by [`reconstruct_uslm_source`] to reproduce the
    /// source bytes exactly.
    pub complement: UslmSyntaxComplement,
}

// =============================================================================
// UscPrxMetadata — the OMV/PROV-O-grounded metadata block, USC structural metrics.
// =============================================================================

/// Self-describing metadata carried by a [`UsCodePrxEnvelope`].
///
/// Reuses the OWL leaf's OMV (Hartmann, Palma & Sure 2005) / PROV-O (Lebo,
/// Sahoo & McGuinness 2013) grounding for identity — `name` (`omv:name`),
/// `version` (`omv:version`), `corpus_uri` (`omv:URI`), `source_url`
/// (`prov:atLocation`), `source_sha256` (`prov:wasDerivedFrom` content
/// address) — but swaps OWL's `omv:numberOfClasses`/`omv:numberOfProperties`
/// structural metrics (meaningless for a statute corpus) for the
/// legislative-appropriate `number_of_sections` / `number_of_subdivisions`,
/// cited to the LRC USLM XML User Guide §V hierarchy. The OWL
/// [`PrxMetadata`](crate::social::software::markup::xml::owl::prx::PrxMetadata)
/// is left untouched (its rkyv layout — hence the OWL `[archive_signatures]`
/// pins — must not change for a USC feature).
///
/// The two structural metrics are **self-description**, exactly like OWL's
/// `omv:numberOf*` fields: the load gate does NOT check them (it binds the
/// whole envelope through the `MerkleRoot` content address, which already
/// covers `data` + `aux`). They self-describe the archive without
/// materializing it; `usc_metadata_metrics_match_corpus` proves they are
/// computed faithfully from the projection.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct UscPrxMetadata {
    /// `omv:name` (Hartmann 2005) — the registry source name, e.g.
    /// `"usc_title_18"`. With [`Self::version`] this is the
    /// `"{name}@{version}"` key the load gate looks pins up under.
    pub name: String,
    /// `omv:version` (Hartmann 2005) — the source version, e.g.
    /// `"pl-119-90"` (the public-law point release).
    pub version: String,
    /// `omv:URI` (Hartmann 2005) — the canonical IRI of the markup the
    /// archived corpus realises (the USLM namespace). The legislative
    /// analogue of OWL's ontology IRI.
    pub corpus_uri: String,
    /// `prov:atLocation` (Lebo 2013 §3.2) — the URL the source USLM XML is
    /// published at (the registry `url`).
    pub source_url: String,
    /// `prov:wasDerivedFrom` / `prov:Entity` (Lebo 2013) content address —
    /// the SHA-256 (NIST FIPS 180-4 §6.2; Dolstra 2006) of the exact source
    /// USLM bytes. The load gate validates this against the `praxis.lock`
    /// `[hashes]` pin; a mismatch fails closed.
    pub source_sha256: String,
    /// Count of `<section>` leaves in the archived title — a structural
    /// metric per LRC USLM XML User Guide §V, the legislative analogue of
    /// `omv:numberOfClasses`.
    pub number_of_sections: u64,
    /// Count of subdivision nodes (subsection / paragraph / … / subitem)
    /// across the archived title — a structural metric per LRC USLM XML
    /// User Guide §V, the legislative analogue of `omv:numberOfProperties`.
    pub number_of_subdivisions: u64,
}

/// The rkyv-serializable `.prx` envelope for the U.S. Code: the archived
/// section corpus, its subdivision-depth aux side-channel, the source
/// reconstruction payload (graph-faithful OR raw floor), and the
/// OMV/PROV-O-grounded metadata. Serialized to rkyv bytes and gzip-wrapped
/// to form the `.prx.gz` artifact.
///
/// Structurally the OWL
/// [`PrxEnvelope`](crate::social::software::markup::xml::owl::prx::PrxEnvelope)
/// plus the [`Self::aux`] field (because the U.S. Code carries subdivision DEPTH
/// the OWL flat edge tables do not) and the [`Self::graph`] field (the
/// graph-faithful reconstruction payload, exactly as the WN-LMF
/// [`WordNetPrxEnvelope`](crate::social::software::markup::xml::lmf::prx::WordNetPrxEnvelope)
/// carries one).
///
/// # The two reconstruction tiers — one envelope, exactly one payload
///
/// `mode` selects which source-reconstruction payload the envelope carries, and
/// the two are mutually exclusive:
///
/// - [`RoundTripFidelity::ByteExactGraphFaithful`] — `graph` is `Some`, `raw`
///   is `None`: the source regenerates from the typed [`UsCodeTitle`] ontology
///   plus the concrete-syntax [`UslmSyntaxComplement`] ([`UscGraphFaithful`]),
///   NO stored raw blob. This is the tier of all registered USC `.prx` titles.
/// - [`RoundTripFidelity::RawBytesComplementFloor`] — `raw` is `Some`, `graph`
///   is `None`: the source bytes are stored as a content-addressed constant
///   complement (the universal floor, for any source exercising USLM families
///   `write_uslm` does not yet cover).
///
/// In both tiers [`Self::data`] + [`Self::aux`] (the reasoning view) are carried
/// unchanged — the runtime materializes [`UsCode`] from them identically
/// regardless of the reconstruction tier.
///
/// The envelope is `PartialEq`-only: the [`Self::graph`] payload transitively
/// carries the `PartialEq`-only [`UsCodeTitle`]. rkyv does not need `Eq` and
/// nothing keys the envelope in a set.
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct UsCodePrxEnvelope {
    /// OMV/PROV-O-grounded self-description, incl. the source content hash
    /// the load gate validates.
    pub metadata: UscPrxMetadata,
    /// The archived section corpus — the owned mirror of the
    /// `CodegenData<UsCode>` interchange (flat section leaves). The runtime
    /// reasoning view, carried unchanged in both reconstruction tiers.
    pub data: OwnedCodegenData,
    /// The subdivision-depth side-channel — the owned mirror of the
    /// [`UscSectionAux`] table [`UsCode::from_codegen_with_aux`] attaches.
    /// The structural delta from the OWL envelope; carried unchanged in both
    /// reconstruction tiers.
    pub aux: Vec<OwnedUscSectionAux>,
    /// The source lens's [`RoundTripFidelity`] — `ByteExactGraphFaithful` for
    /// all registered USC titles (the typed ontology + concrete-syntax
    /// complement regenerate the source from the graph alone),
    /// `RawBytesComplementFloor` for any source whose families `write_uslm` does
    /// not yet cover.
    pub mode: RoundTripFidelity,
    /// The graph-faithful reconstruction payload (typed ontology + concrete-
    /// syntax complement) — `Some` iff `mode == ByteExactGraphFaithful`, `None`
    /// otherwise (the floor stores `raw` instead). No raw blob is kept in this
    /// tier; the source is regenerated from the graph.
    pub graph: Option<UscGraphFaithful>,
    /// The content-addressed source bytes (the constant-complement) — `Some`
    /// iff `mode == RawBytesComplementFloor`. `None` in the graph-faithful tier.
    pub raw: Option<RawSource>,
}

// =============================================================================
// rkyv layer — USC envelope ⇄ bytes (bytecheck-validated).
// =============================================================================

/// Serialize a USC envelope to rkyv bytes (the lens *put*). Deterministic —
/// equal envelopes yield equal bytes, so the blob's SHA-256 is a stable
/// `MerkleRoot` content address.
pub fn usc_envelope_to_bytes(envelope: &UsCodePrxEnvelope) -> Result<Vec<u8>, PrxError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(envelope)
        .map(|v| v.to_vec())
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

/// Materialize a USC envelope from rkyv bytes (the lens *get*). Copies into
/// an aligned buffer, then `bytecheck`-validates before materializing, so a
/// corrupted blob fails closed.
pub fn usc_envelope_from_bytes(bytes: &[u8]) -> Result<UsCodePrxEnvelope, PrxError> {
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<UsCodePrxEnvelope, rkyv::rancor::Error>(&aligned)
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

// =============================================================================
// The load gate — mirrors the OWL legs over the USC envelope, SHARED primitive.
// =============================================================================

/// Discharge a content-hash `IntegrityClaim` over `bytes` against a trusted
/// pin — byte-for-byte the OWL `verify_content_address` leg, re-implemented
/// here only because the OWL one is private and `&PrxEnvelope`-typed. The
/// integrity primitive [`raw_hash::verify`] is the SAME one the fetch path
/// and the OWL gate use (Dolstra 2006 content-addressing; W3C SRI 2016):
/// `raw_hash::verify` re-hashes `bytes`, so the pin is checked against bytes
/// actually present, never a self-asserted label.
fn usc_verify_content_address(bytes: &[u8], trusted_pin: &str, key: &str) -> Result<(), PrxError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(trusted_pin.to_string()),
    };
    match raw_hash::verify(&claim, bytes) {
        VerificationResult::Verified(_) => Ok(()),
        VerificationResult::Mismatch { expected, actual } => Err(PrxError::HashMismatch {
            key: key.to_string(),
            expected,
            found: actual,
        }),
        VerificationResult::Unverifiable { reason } => Err(PrxError::IntegrityUnverifiable {
            key: key.to_string(),
            reason,
        }),
    }
}

/// Reconstruct the exact source USLM bytes from a USC envelope — the
/// `.prx → xml` leg of the #186 byte-hash invariant. Mirrors the WN-LMF
/// `wn_reconstruct_source` over the USC envelope.
///
/// For [`RoundTripFidelity::RawBytesComplementFloor`] (the USC titles whose
/// families `write_uslm` does not yet cover): return the stored `raw.blob` after
/// enforcing the in-envelope honesty doctrine
/// (`sha256(blob) == raw.content_address == metadata.source_sha256`). A tampered
/// blob is rejected.
///
/// For [`RoundTripFidelity::ByteExactGraphFaithful`] (all registered USC
/// titles): regenerate the source from the typed [`UsCodeTitle`] ontology PLUS the
/// concrete-syntax [`UslmSyntaxComplement`] carried in `graph` via
/// [`reconstruct_uslm_source`] (the graph-faithful `put`, NO stored raw blob),
/// then enforce the SAME sha256 honesty gate the floor arm uses — the
/// regenerated bytes MUST hash to `metadata.source_sha256` (the `praxis.lock`
/// `[byte_exact_signatures]` / `[hashes]` pin). A regeneration that does not
/// reproduce the pinned source is rejected ([`PrxError::HashMismatch`]),
/// fail-closed, never fabricating or returning unverified bytes. No
/// `unwrap`/`expect` on the path.
pub fn usc_reconstruct_source(envelope: &UsCodePrxEnvelope) -> Result<Vec<u8>, PrxError> {
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    match envelope.mode {
        RoundTripFidelity::RawBytesComplementFloor => {
            let raw = envelope
                .raw
                .as_ref()
                .ok_or_else(|| PrxError::SourceNotReconstructible {
                    reason: "RawBytesComplementFloor envelope is missing its raw source leaf"
                        .to_string(),
                })?;
            let computed = source_content_hash(&raw.blob);
            if computed != raw.content_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw content address)"),
                    expected: raw.content_address.clone(),
                    found: computed,
                });
            }
            if computed != envelope.metadata.source_sha256 {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw vs metadata)"),
                    expected: envelope.metadata.source_sha256.clone(),
                    found: computed,
                });
            }
            Ok(raw.blob.clone())
        }
        RoundTripFidelity::ByteExactGraphFaithful => {
            // The graph-faithful payload (typed ontology + concrete-syntax
            // complement) must be present — its absence is a malformed envelope,
            // not a fabrication opportunity.
            let graph =
                envelope
                    .graph
                    .as_ref()
                    .ok_or_else(|| PrxError::SourceNotReconstructible {
                        reason: "ByteExactGraphFaithful envelope is missing its graph payload \
                                 (typed ontology + concrete-syntax complement)"
                            .to_string(),
                    })?;
            // Regenerate from the GRAPH alone (no stored raw blob): the typed
            // USLM title ontology + the captured concrete-syntax complement. This
            // is the byte-exact `put` proven inverse over the literal Title 1
            // source (slices U1–U5).
            let bytes = reconstruct_uslm_source(&graph.title, &graph.complement).map_err(|e| {
                PrxError::SourceNotReconstructible {
                    reason: format!("graph-faithful USLM reconstruction failed: {e}"),
                }
            })?;
            // The SAME honesty gate the floor arm enforces: the regenerated bytes
            // must hash to the pinned source content address. A regeneration that
            // drifts from the pinned source fails closed.
            let computed = source_content_hash(&bytes);
            if computed != envelope.metadata.source_sha256 {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (graph-faithful reconstruction vs metadata)"),
                    expected: envelope.metadata.source_sha256.clone(),
                    found: computed,
                });
            }
            Ok(bytes)
        }
    }
}

/// Verify the source-identity leg: reconstruct the source and bind it to the
/// trusted `SourcePin` (`praxis.lock` `[hashes]`). Mirrors the WN-LMF
/// `wn_verify_source_leg`.
///
/// Both tiers reconstruct the source and bind it to the trusted source pin — the
/// floor from its stored raw complement, the graph-faithful tier from the
/// ontology + concrete-syntax complement. [`usc_reconstruct_source`] already
/// enforces the in-envelope honesty gate (regenerated == metadata hash); binding
/// to `source_pin` additionally anchors it to the EXTERNAL `praxis.lock` pin
/// (`[hashes]` == `[byte_exact_signatures]` for a graph-faithful title, since
/// `put(get(b)) == b` makes the round-trip hash the raw-source hash).
fn usc_verify_source_leg(
    envelope: &UsCodePrxEnvelope,
    source_pin: &str,
    key: &str,
) -> Result<(), PrxError> {
    let source_bytes = usc_reconstruct_source(envelope)?;
    usc_verify_content_address(&source_bytes, source_pin, key)
}

/// Admit a decoded USC envelope only after BOTH gate legs verify, then
/// materialize the corpus. The fail-closed realisation of the `LoadGate`
/// concept, identical in structure to OWL `admit_validated`:
///
/// 1. **Installed-node integrity** — the `BinaryEnvelope`'s `MerkleRoot`,
///    re-derived from `rkyv_bytes`, must equal `archive_pin`. This binds the
///    whole envelope INCLUDING the recursive `aux` tree, so a poisoned
///    subdivision under a genuine source label is rejected (Merkle 1987;
///    Benet 2014; W3C SRI 2016).
/// 2. **Source identity** — the carried source re-hashes to `source_pin`.
/// 3. Only on both `Verified` rebuild the `CodegenData` view + leak the aux
///    and materialize via `from_codegen_with_aux` (NEVER `from_codegen`,
///    which would drop subdivision depth).
fn usc_admit_validated(
    rkyv_bytes: &[u8],
    envelope: UsCodePrxEnvelope,
    archive_pin: &str,
    source_pin: &str,
) -> Result<UsCode, PrxError> {
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    usc_verify_content_address(rkyv_bytes, archive_pin, &key)?;
    usc_verify_source_leg(&envelope, source_pin, &key)?;
    let data: CodegenData<UsCode> = envelope.data.to_codegen_data_leaked();
    let aux = to_aux_leaked(&envelope.aux);
    Ok(UsCode::from_codegen_with_aux(&data, aux))
}

/// Load a USC `.prx.gz` blob into a materialized [`UsCode`], gated on two
/// externally trusted pins (`praxis.lock` `[archive_signatures]` +
/// `[hashes]`). Mirrors OWL `load_prx_gz`.
pub fn load_usc_prx_gz(
    prx_gz: &[u8],
    archive_pin: &str,
    source_pin: &str,
) -> Result<UsCode, PrxError> {
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = usc_envelope_from_bytes(&rkyv_bytes)?;
    usc_admit_validated(&rkyv_bytes, envelope, archive_pin, source_pin)
}

/// Load a USC `.prx.gz` blob, reaching both pins through the live registry:
/// the `MerkleRoot` from `[archive_signatures]`, the `SourcePin` from
/// `[hashes]`, keyed by `"{name}@{version}"`. Mirrors OWL
/// `load_prx_gz_from_lock`. Fail-closed if either pin is unregistered.
pub fn load_usc_prx_gz_from_lock(prx_gz: &[u8]) -> Result<UsCode, PrxError> {
    use crate::applied::data_provisioning::registry::{lock_archive_signature, lock_hashes};
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = usc_envelope_from_bytes(&rkyv_bytes)?;
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    let archive_pin = lock_archive_signature(&envelope.metadata.name, &envelope.metadata.version)
        .ok_or_else(|| PrxError::NoArchivePin { key: key.clone() })?
        .to_string();
    let source_pin = lock_hashes()
        .get(&key)
        .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?
        .clone();
    usc_admit_validated(&rkyv_bytes, envelope, &archive_pin, &source_pin)
}

// =============================================================================
// Emit — read_uslm_title → title_to_owned → envelope → rkyv → gzip.
// =============================================================================
//
// Unlike `owl::prx`'s emit (gated on `codegen` because `owl_to_builder` needs
// `pr4xis::codegen`), USC emit is reachable under `prx` ALONE: it parses via
// `read_uslm_title` (quick-xml, non-optional — the exact path `loaded()`
// uses), so no `xsd-parser` substrate leaks into the WASM-facing build. This
// is the one legitimate divergence from the OWL template.

/// Project a parsed [`UsCodeTitle`] into the owned archival shape: flat
/// section columns ([`OwnedCodegenData`]) plus the subdivision-depth aux
/// ([`OwnedUscSectionAux`]).
///
/// ONE parse, no third walker. `entity_defs` reuses the EXACT
/// `section_body_text` the runtime corpus loader uses, and the aux mirrors
/// `subdivisions_to_static`'s document-order DFS — so an archive-materialized
/// corpus matches the parse-materialized one.
fn title_to_owned(title: &UsCodeTitle) -> (OwnedCodegenData, Vec<OwnedUscSectionAux>) {
    let mut entity_ids = Vec::with_capacity(title.sections.len());
    let mut entity_kind = Vec::with_capacity(title.sections.len());
    let mut entity_labels = Vec::with_capacity(title.sections.len());
    let mut entity_defs = Vec::with_capacity(title.sections.len());
    let mut aux = Vec::with_capacity(title.sections.len());
    for section in &title.sections {
        entity_ids.push(section.identifier.clone());
        entity_kind.push("section".to_string());
        // PROSE heading — footnote annotation stripped via the typed tree, so
        // the archived `entity_labels` heading column MATCHES the XML path's
        // `from_uslm_titles_owned` (`prose_text()`). The byte-exact source
        // reconstruction regenerates from the typed `heading_mixed`, not this
        // flat column, so U6 byte-exactness is unaffected.
        entity_labels.push(section.heading_mixed.prose_text());
        // Reuse the EXACT corpus body-text projection (mod.rs) so the archive
        // matches `from_uslm_titles_owned`.
        entity_defs.push(super::section_body_text(section));
        let (subdivisions, relations) = owned_subdivisions(&section.children, &section.identifier);
        aux.push(OwnedUscSectionAux {
            urn: section.identifier.clone(),
            subdivisions,
            relations,
        });
    }
    let data = OwnedCodegenData {
        entity_count: entity_ids.len() as u64,
        entity_ids,
        entity_kind,
        entity_labels,
        entity_defs,
        word_index: Vec::new(),
        taxonomy: Vec::new(),
        mereology: Vec::new(),
        opposition: Vec::new(),
        equivalence: Vec::new(),
        causation: Vec::new(),
        references: Vec::new(),
    };
    (data, aux)
}

/// Owned twin of `subdivisions_to_static`: project a runtime
/// [`UsCodeSubdivision`] tree into the owned mirror, emitting one
/// `(child_urn, parent_urn)` `Composes` edge per child in the SAME
/// document-order DFS (edge-before-recursion) — so the relation order is
/// pinned and the `MerkleRoot` over the rkyv bytes is reproducible.
fn owned_subdivisions(
    subs: &[UsCodeSubdivision],
    parent_urn: &str,
) -> (Vec<OwnedUscSubdivision>, Vec<(String, String)>) {
    let mut result = Vec::with_capacity(subs.len());
    let mut edges = Vec::new();
    for sub in subs {
        edges.push((sub.identifier.clone(), parent_urn.to_string()));
        let (child_subs, child_edges) = owned_subdivisions(&sub.children, &sub.identifier);
        edges.extend(child_edges);
        result.push(OwnedUscSubdivision {
            urn: sub.identifier.clone(),
            kind: sub.kind.tag().to_string(),
            num: sub.num.clone(),
            heading: sub.heading.clone(),
            chapeau: sub.chapeau.clone(),
            content: sub.content.clone(),
            children: child_subs,
        });
    }
    (result, edges)
}

/// Count subdivision nodes in an owned tree (pre-order).
fn count_owned(subs: &[OwnedUscSubdivision]) -> usize {
    subs.iter().map(|s| 1 + count_owned(&s.children)).sum()
}

/// Build a [`UsCodePrxEnvelope`] from USLM source bytes plus its registry
/// `(name, version, url)`, preferring the GRAPH-FAITHFUL tier for any title that
/// carries a registered graph-faithful lens, and gracefully degrading to the
/// universal floor otherwise (no graph-faithful registration, or a title whose
/// USLM families the structural writer cannot yet reproduce). Title-AGNOSTIC: it
/// branches on the lens registry, never on a title name. Mirrors the WN-LMF
/// `build_wordnet_envelope`.
///
/// 1. Parse the typed [`UsCodeTitle`] ontology once ([`read_uslm_title`]) and
///    project the [`OwnedCodegenData`] + [`OwnedUscSectionAux`] reasoning view.
///    This is carried unchanged in BOTH tiers.
/// 2. ATTEMPT the graph-faithful `get`: [`capture_uslm_complement`] re-parses the
///    source and captures the concrete-syntax [`UslmSyntaxComplement`] (prolog
///    PI, DOCTYPE, namespaces, white-space layout, entity-reference form,
///    end-of-line form, source attribute sequences) the byte-exact `put`
///    re-applies.
///    - **Captured** (and a graph-faithful lens is registered) → emit
///      `mode = ByteExactGraphFaithful`, the `graph` payload (ontology +
///      complement), `raw = None` (NO stored raw blob; the source regenerates
///      from the graph).
///    - **Uncovered family** ([`UslmReconstructError::Write`]) or **backbone
///      divergence** ([`UslmReconstructError::Complement`]) → the structural
///      writer cannot yet regenerate this title's markup (an uncovered USLM
///      family such as a `<continuation>` flush-text run or `<def>` / `<marker>`
///      / `<ins>` markup, surfacing as a
///      [`UslmWriteError::UncoveredFamily`](super::super::lens::writer::UslmWriteError);
///      or a residue that is not a pure white-space/decl complement). Degrade
///      HONESTLY to the universal floor: emit `mode = RawBytesComplementFloor`,
///      `graph = None`, `raw =` the content-addressed source blob — the same
///      constant-complement OWL rides. NEVER a silent lie: the floor tier is
///      explicit in `mode`, and the completeness meter only declares a title
///      graph-faithful when a lens is registered for it.
///    - **Malformed source** ([`UslmReconstructError::Parse`] /
///      [`UslmReconstructError::Read`]) → a hard error; a non-well-formed or
///      unrecognised USLM document is a defect, not a floor candidate.
///
/// The OMV/PROV-O metadata's `source_sha256` is the content address of the exact
/// source bytes (the `[hashes]` / `[byte_exact_signatures]` pin), against which
/// [`usc_reconstruct_source`] gates the regenerated bytes fail-closed in BOTH
/// tiers.
pub fn build_usc_envelope(
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Result<UsCodePrxEnvelope, PrxError> {
    let text = core::str::from_utf8(source)
        .map_err(|e| PrxError::Read(format!("source is not UTF-8: {e}")))?;
    // Parse the typed ontology once — the reasoning view (`data` + `aux`) is
    // projected from it in both tiers, so the materialized `UsCode` is identical
    // either way.
    let title = read_uslm_title(text).map_err(|e| PrxError::Read(format!("{e}")))?;
    let (data, aux) = title_to_owned(&title);
    let number_of_sections = aux.len() as u64;
    let number_of_subdivisions = aux
        .iter()
        .map(|a| count_owned(&a.subdivisions))
        .sum::<usize>() as u64;
    let source_sha256 = source_content_hash(source);
    let metadata = UscPrxMetadata {
        name: name.to_string(),
        version: version.to_string(),
        corpus_uri: super::USLM_NAMESPACE_URI.to_string(),
        source_url: url.to_string(),
        source_sha256: source_sha256.clone(),
        number_of_sections,
        number_of_subdivisions,
    };

    // The emitted tier MUST agree with the completeness meter's DECLARED tier —
    // they are not allowed to be two disagreeing sources of truth. The meter
    // declares a title graph-faithful ONLY when a graph-faithful lens is
    // REGISTERED for `(name, version)` (it reads
    // `lens_registrations().find(name@version).fidelity`; see
    // `well_behaved_lens::completeness::completeness_meter`). So we consult THAT
    // SAME registry here — a successful `capture_uslm_complement` is necessary but
    // NOT sufficient to claim the graph-faithful tier; the title must ALSO have a
    // registered `RoundTripFidelity::ByteExactGraphFaithful` lens. A title bound
    // to a graph-faithful lens qualifies; a title with only the floor
    // `UslmXmlLens` (or no registration at all) does not and rides the floor —
    // matching the meter, and branching on the REGISTRY, never on a title name.
    // The lens registry is native-only (`linkme`); emit is a native
    // `fetch`/`codegen` path, so the lookup is sound here.
    let registered_graph_faithful =
        crate::formal::meta::well_behaved_lens::lens_by_name(&format!("{name}@{version}"))
            .is_some_and(|r| r.fidelity == RoundTripFidelity::ByteExactGraphFaithful);

    // Attempt the graph-faithful capture. Prefer it ONLY for a registered
    // graph-faithful title; otherwise degrade to the floor even on a successful
    // capture. We still also degrade on an uncovered-family / backbone divergence.
    match capture_uslm_complement(text) {
        Ok((title_captured, complement)) if registered_graph_faithful => Ok(UsCodePrxEnvelope {
            metadata,
            data,
            aux,
            mode: RoundTripFidelity::ByteExactGraphFaithful,
            graph: Some(UscGraphFaithful {
                title: title_captured,
                complement,
            }),
            raw: None,
        }),
        // Capture SUCCEEDED but no graph-faithful lens is registered for this
        // title — the meter declares it FLOOR, so emit the floor (the raw
        // content-addressed blob), keeping emit-tier == meter-tier. The captured
        // graph is discarded; the source regenerates from the stored blob.
        Ok(_) => Ok(UsCodePrxEnvelope {
            metadata,
            data,
            aux,
            mode: RoundTripFidelity::RawBytesComplementFloor,
            graph: None,
            raw: Some(RawSource {
                content_address: source_sha256,
                blob: source.to_vec(),
            }),
        }),
        // The structural writer cannot reproduce this title's backbone (an
        // uncovered USLM family or a non-pure-white-space residue) — ride the
        // universal floor (the content-addressed raw blob), honestly tiered.
        Err(UslmReconstructError::Write(_) | UslmReconstructError::Complement(_)) => {
            Ok(UsCodePrxEnvelope {
                metadata,
                data,
                aux,
                mode: RoundTripFidelity::RawBytesComplementFloor,
                graph: None,
                raw: Some(RawSource {
                    content_address: source_sha256,
                    blob: source.to_vec(),
                }),
            })
        }
        // A malformed / unrecognised USLM source is a defect, not a floor candidate.
        Err(e @ (UslmReconstructError::Parse(_) | UslmReconstructError::Read(_))) => {
            Err(PrxError::Read(format!("graph-faithful capture: {e}")))
        }
    }
}

/// Emit a USC `.prx.gz` artifact from USLM source bytes:
/// `build_usc_envelope → usc_envelope_to_bytes (rkyv) → gzip`.
pub fn emit_usc_prx_gz(
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Result<Vec<u8>, PrxError> {
    let envelope = build_usc_envelope(source, name, version, url)?;
    let rkyv_bytes = usc_envelope_to_bytes(&envelope)?;
    gzip(&rkyv_bytes)
}

// =============================================================================
// Compact runtime `.prx.gz` — the small (data + aux) reasoning view, bit-packed,
// with NO source-reconstruction payload (the USC sibling of the OWL
// `emit_compact_prx_gz`). The byte-exact / `praxis.lock` integrity path stays the
// separate envelope above; this is what the runtime embeds or downloads.
// =============================================================================

/// Pre-order columnar view of a subdivision forest: one row per node, the tree
/// shape carried by the `child_count` column (pre-order + child-count uniquely
/// reconstructs an ordered tree), the six per-node text fields as dictionary
/// indices (`heading`/`chapeau`/`content` optional).
#[derive(Default)]
struct AuxColumns {
    child_count: Vec<usize>,
    urn: Vec<usize>,
    kind: Vec<usize>,
    num: Vec<usize>,
    heading: Vec<Option<u32>>,
    chapeau: Vec<Option<u32>>,
    content: Vec<Option<u32>>,
}

impl AuxColumns {
    /// Flatten one node and its descendants pre-order into the columns.
    fn push_tree(&mut self, node: &OwnedUscSubdivision, idx: &hashbrown::HashMap<&str, usize>) {
        self.child_count.push(node.children.len());
        self.urn.push(idx[node.urn.as_str()]);
        self.kind.push(idx[node.kind.as_str()]);
        self.num.push(idx[node.num.as_str()]);
        self.heading
            .push(node.heading.as_deref().map(|s| idx[s] as u32));
        self.chapeau
            .push(node.chapeau.as_deref().map(|s| idx[s] as u32));
        self.content
            .push(node.content.as_deref().map(|s| idx[s] as u32));
        for c in &node.children {
            self.push_tree(c, idx);
        }
    }
}

/// Rebuild the owned subdivision forest from the decoded columns via a single
/// global cursor (pre-order + `child_count`).
struct AuxDecoder<'a> {
    dict: &'a [String],
    child_count: &'a [usize],
    urn: &'a [usize],
    kind: &'a [usize],
    num: &'a [usize],
    heading: &'a [Option<u32>],
    chapeau: &'a [Option<u32>],
    content: &'a [Option<u32>],
    cur: usize,
}

impl AuxDecoder<'_> {
    fn rebuild(&mut self) -> OwnedUscSubdivision {
        let i = self.cur;
        self.cur += 1;
        let cc = self.child_count[i];
        let urn = self.dict[self.urn[i]].clone();
        let kind = self.dict[self.kind[i]].clone();
        let num = self.dict[self.num[i]].clone();
        let heading = self.heading[i].map(|v| self.dict[v as usize].clone());
        let chapeau = self.chapeau[i].map(|v| self.dict[v as usize].clone());
        let content = self.content[i].map(|v| self.dict[v as usize].clone());
        let mut children = Vec::with_capacity(cc);
        for _ in 0..cc {
            children.push(self.rebuild());
        }
        OwnedUscSubdivision {
            urn,
            kind,
            num,
            heading,
            chapeau,
            content,
            children,
        }
    }
}

/// Re-derive a section's `Composes` edges from its decoded subdivision tree —
/// the identical document-order, edge-before-recursion DFS [`owned_subdivisions`]
/// emits. The edges are pure derived data (one child→parent edge per node), so
/// the compact codec stores NONE of them and regenerates them here losslessly.
fn derive_compose_edges(subs: &[OwnedUscSubdivision], parent_urn: &str) -> Vec<(String, String)> {
    let mut edges = Vec::new();
    for sub in subs {
        edges.push((sub.urn.clone(), parent_urn.to_string()));
        edges.extend(derive_compose_edges(&sub.children, &sub.urn));
    }
    edges
}

/// Append the aux subdivision forest to `out`: one shared front-coded dictionary
/// over every aux string (section URNs + node urn/kind/num + present
/// heading/chapeau/content), then per-section the section-URN index + root count,
/// then the global pre-order node columns. The `Composes` edges are NOT stored
/// (re-derived on load).
fn aux_to_succinct(out: &mut Vec<u8>, aux: &[OwnedUscSectionAux]) {
    use hashbrown::HashMap;

    fn collect<'a>(subs: &'a [OwnedUscSubdivision], all: &mut Vec<&'a str>) {
        for s in subs {
            all.push(s.urn.as_str());
            all.push(s.kind.as_str());
            all.push(s.num.as_str());
            if let Some(h) = &s.heading {
                all.push(h);
            }
            if let Some(c) = &s.chapeau {
                all.push(c);
            }
            if let Some(c) = &s.content {
                all.push(c);
            }
            collect(&s.children, all);
        }
    }

    let mut all: Vec<&str> = Vec::new();
    for sec in aux {
        all.push(sec.urn.as_str());
        collect(&sec.subdivisions, &mut all);
    }
    all.sort_unstable();
    all.dedup();
    let idx: HashMap<&str, usize> = all.iter().enumerate().map(|(i, &s)| (s, i)).collect();
    let dict: Vec<String> = all.iter().map(|s| String::from(*s)).collect();
    put_dict_fc(out, &dict);

    put_varint(out, aux.len() as u64);
    let mut sec_urn = Vec::with_capacity(aux.len());
    let mut root_count = Vec::with_capacity(aux.len());
    let mut cols = AuxColumns::default();
    for sec in aux {
        sec_urn.push(idx[sec.urn.as_str()]);
        root_count.push(sec.subdivisions.len());
        for node in &sec.subdivisions {
            cols.push_tree(node, &idx);
        }
    }
    put_cv(out, &sec_urn);
    put_cv(out, &root_count);
    put_cv(out, &cols.child_count);
    put_cv(out, &cols.urn);
    put_cv(out, &cols.kind);
    put_cv(out, &cols.num);
    put_opt(out, &cols.heading);
    put_opt(out, &cols.chapeau);
    put_opt(out, &cols.content);
}

/// Decode the aux subdivision forest from `buf` at `pos` (inverse of
/// [`aux_to_succinct`]), re-deriving each section's `Composes` edges from the
/// rebuilt tree.
fn aux_from_succinct(buf: &[u8], pos: &mut usize) -> Vec<OwnedUscSectionAux> {
    let dict = get_dict_fc(buf, pos);
    let n_sec = get_varint(buf, pos) as usize;
    let sec_urn = get_cv(buf, pos);
    let root_count = get_cv(buf, pos);
    let child_count = get_cv(buf, pos);
    let urn = get_cv(buf, pos);
    let kind = get_cv(buf, pos);
    let num = get_cv(buf, pos);
    let heading = get_opt(buf, pos);
    let chapeau = get_opt(buf, pos);
    let content = get_opt(buf, pos);

    let mut dec = AuxDecoder {
        dict: &dict,
        child_count: &child_count,
        urn: &urn,
        kind: &kind,
        num: &num,
        heading: &heading,
        chapeau: &chapeau,
        content: &content,
        cur: 0,
    };
    let mut aux = Vec::with_capacity(n_sec);
    for s in 0..n_sec {
        let rc = root_count[s];
        let mut subdivisions = Vec::with_capacity(rc);
        for _ in 0..rc {
            subdivisions.push(dec.rebuild());
        }
        let urn_s = dec.dict[sec_urn[s]].clone();
        let relations = derive_compose_edges(&subdivisions, &urn_s);
        aux.push(OwnedUscSectionAux {
            urn: urn_s,
            subdivisions,
            relations,
        });
    }
    aux
}

/// Serialize the compact reasoning view `(data, aux)` to `.prx` bytes: the
/// length-prefixed flat-section codec ([`OwnedCodegenData::to_succinct`], the
/// SAME source-agnostic codec OWL uses) followed by the aux subdivision-tree
/// columns. `compact_usc_from_succinct(&compact_usc_to_succinct(d, a)) == (d, a)`.
fn compact_usc_to_succinct(data: &OwnedCodegenData, aux: &[OwnedUscSectionAux]) -> Vec<u8> {
    let mut out = Vec::new();
    put_blob(&mut out, &data.to_succinct());
    aux_to_succinct(&mut out, aux);
    out
}

/// Decode the compact reasoning view (inverse of [`compact_usc_to_succinct`]).
fn compact_usc_from_succinct(buf: &[u8]) -> (OwnedCodegenData, Vec<OwnedUscSectionAux>) {
    let mut pos = 0usize;
    let data = OwnedCodegenData::from_succinct(get_blob(buf, &mut pos));
    let aux = aux_from_succinct(buf, &mut pos);
    (data, aux)
}

/// Emit the COMPACT runtime `.prx.gz` from USLM source — the small artifact the
/// runtime loads (`read_uslm_title → title_to_owned → compact codec → gzip`),
/// the data-only reasoning view with no stored source bytes and no graph-faithful
/// complement. The OWL sibling of `emit_compact_prx_gz`; reachable under `prx`
/// alone (no codegen), so the WASM runtime can produce and load it.
pub fn emit_compact_usc_prx_gz(source: &[u8]) -> Result<Vec<u8>, PrxError> {
    let text = core::str::from_utf8(source)
        .map_err(|e| PrxError::Read(format!("USLM source is not UTF-8: {e}")))?;
    let title = read_uslm_title(text).map_err(|e| PrxError::Read(format!("{e}")))?;
    let (data, aux) = title_to_owned(&title);
    gzip(&compact_usc_to_succinct(&data, &aux))
}

/// Materialize a [`UsCode`] from the uncompressed compact succinct bytes:
/// [`compact_usc_from_succinct`] → [`to_aux_leaked`] +
/// [`UsCode::from_codegen_with_aux`]. No XML re-parse, no rkyv envelope.
fn materialize_compact(raw_succinct: &[u8]) -> UsCode {
    let (data, aux) = compact_usc_from_succinct(raw_succinct);
    let leaked = to_aux_leaked(&aux);
    let codegen: CodegenData<UsCode> = data.to_codegen_data_leaked();
    UsCode::from_codegen_with_aux(&codegen, leaked)
}

/// Load a compact runtime `.prx.gz` (produced by [`emit_compact_usc_prx_gz`])
/// into a materialized [`UsCode`] WITHOUT an integrity gate — gunzip →
/// materialize. For trusted bytes (tests, the build emitter's round-trip
/// check); the runtime fast path uses the gated [`load_compact_usc_prx_gz_gated`].
pub fn load_compact_usc_prx_gz(prx_gz: &[u8]) -> Result<UsCode, PrxError> {
    Ok(materialize_compact(&gunzip(prx_gz)?))
}

/// The content address of a compact `.prx.gz` archive — the SHA-256 of its
/// uncompressed succinct bytes (gzip-level-independent), as 64-char lowercase
/// hex. The value pinned in `praxis.lock` `[compact_archive_signatures]` and the
/// one [`load_compact_usc_prx_gz_gated`] re-derives and verifies. Portable: the
/// compact codec is dependency-free bit-packing, so this is stable across
/// toolchains and targets (unlike the rkyv [`prx_archive_address`]).
pub fn compact_prx_archive_address(cprx_gz: &[u8]) -> Result<String, PrxError> {
    Ok(source_content_hash(&gunzip(cprx_gz)?))
}

/// Load a compact runtime `.prx.gz` into a [`UsCode`] through the fail-closed
/// content-address gate — gunzip → discharge a content-hash `IntegrityClaim`
/// (the succinct bytes must SHA-256 to `archive_pin`, the
/// `[compact_archive_signatures]` pin) → only then materialize. A compact
/// archive whose bytes do not match the pin is rejected before any data is
/// installed (Dolstra 2006 content-addressing; W3C SRI 2016). The portable,
/// no-source-reconstruction sibling of [`load_usc_prx_gz`]: the gate hashes the
/// installed bytes directly, so the giant-title source-regeneration cost the
/// envelope gate pays on load is gone.
pub fn load_compact_usc_prx_gz_gated(
    cprx_gz: &[u8],
    archive_pin: &str,
    key: &str,
) -> Result<UsCode, PrxError> {
    let raw = gunzip(cprx_gz)?;
    usc_verify_content_address(&raw, archive_pin, key)?;
    Ok(materialize_compact(&raw))
}

/// Workspace root — the grandparent of `CARGO_MANIFEST_DIR` (`crates/domains/`),
/// against which registry `local_path()`s and `praxis.lock` resolve. Mirrors
/// the OWL emitter + the USC corpus loader.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// The compiled-USC-archive cache directory: `<workspace_root>/.prx-cache/usc`.
/// `pr4xis compile` writes one `{name}-{version}.prx.gz` here per registered
/// title, and [`loaded`][super::loaded] reads them as its fast rkyv load path
/// (falling back to the XML parse when an archive is absent). Gitignored build
/// output — never committed; CI restores it from the `praxis.lock`-keyed data
/// cache. The filesystem analogue of the OWL emitter's `dist/ontologies`.
pub fn usc_prx_cache_dir(workspace_root: &std::path::Path) -> std::path::PathBuf {
    workspace_root.join(".prx-cache").join("usc")
}

/// The compiled COMPACT-USC-archive cache directory:
/// `<workspace_root>/.prx-cache/usc-compact`. `pr4xis compile` writes one
/// `{name}-{version}.cprx.gz` here per registered title (a sibling of
/// [`usc_prx_cache_dir`] so the compact and envelope caches never collide), and
/// [`loaded`][super::loaded] prefers it as the fast, content-address-gated load
/// path. Gitignored build output — never committed.
pub fn usc_compact_prx_cache_dir(workspace_root: &std::path::Path) -> std::path::PathBuf {
    workspace_root.join(".prx-cache").join("usc-compact")
}

/// Emit a USC `.prx.gz` artifact for EVERY registered [`UsCodeTitle`] source
/// on disk into `out_dir`, round-trip-validating each before returning it —
/// the USC analogue of `owl::prx::emit_all_prx_gz`, the release-distribution
/// entry point. Registry-driven (never a hardcoded title set).
///
/// A registered title whose USLM XML is NOT on disk is skipped gracefully —
/// the same graceful skip `loaded()` and the OWL emitter make. USC titles are
/// large (Title 18 ~12 MB … Title 42 ~108 MB) and externally provisioned (via
/// `pr4xis update`), not bundled in a source checkout, so a plain checkout
/// emits nothing here; a data-provisioned environment (release CI) emits and
/// pins each. A title that emits but fails to round-trip-load is a defect
/// (Err), never a skip.
///
/// Each emitted file is round-trip-validated through the fail-closed gate
/// against the `MerkleRoot` this emit just produced and the source's
/// `praxis.lock` `[hashes]` pin — proving the published artifact is loadable,
/// content-anchored, and source-faithful before the operator pins its
/// `archive_address` into `[archive_signatures]`.
pub fn emit_all_usc_prx_gz(out_dir: &std::path::Path) -> Result<Vec<EmittedArtifact>, PrxError> {
    use crate::applied::data_provisioning::registry::{data_sources, lock_hashes};
    use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;
    let root = workspace_root();
    let mut emitted = Vec::new();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let src_path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&src_path) else {
            // Registered but not on disk — skip gracefully.
            continue;
        };
        let prx_gz = emit_usc_prx_gz(&source, &entry.name, &entry.version, &entry.url)?;
        let archive_address = prx_archive_address(&prx_gz)?;
        let path = out_dir.join(format!("{}-{}.prx.gz", entry.name, entry.version));
        std::fs::write(&path, &prx_gz)
            .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

        let key = format!("{}@{}", entry.name, entry.version);
        let source_pin = lock_hashes()
            .get(&key)
            .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?;
        let read_back = std::fs::read(&path)
            .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
        load_usc_prx_gz(&read_back, &archive_address, source_pin)?;

        emitted.push(EmittedArtifact {
            name: entry.name.clone(),
            version: entry.version.clone(),
            path,
            byte_len: prx_gz.len() as u64,
            archive_address,
        });
    }
    Ok(emitted)
}

/// Emit the COMPACT `.prx.gz` for EVERY registered on-disk [`UsCodeTitle`] into
/// `out_dir`, content-addressing and round-trip-validating each through the
/// fail-closed gate before returning it — the compact sibling of
/// [`emit_all_usc_prx_gz`]. Registry-driven; a title not on disk is skipped
/// gracefully. Each artifact's `archive_address` is the portable
/// [`compact_prx_archive_address`] the operator pins into
/// `[compact_archive_signatures]`; the round-trip load through
/// [`load_compact_usc_prx_gz_gated`] proves the published archive is loadable
/// and content-anchored.
pub fn emit_all_compact_usc_prx_gz(
    out_dir: &std::path::Path,
) -> Result<Vec<EmittedArtifact>, PrxError> {
    use crate::applied::data_provisioning::registry::data_sources;
    use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

    std::fs::create_dir_all(out_dir)
        .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;
    let root = workspace_root();
    let mut emitted = Vec::new();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let src_path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&src_path) else {
            continue;
        };
        let cprx_gz = emit_compact_usc_prx_gz(&source)?;
        let archive_address = compact_prx_archive_address(&cprx_gz)?;
        let key = format!("{}@{}", entry.name, entry.version);
        let path = out_dir.join(format!("{}-{}.cprx.gz", entry.name, entry.version));
        std::fs::write(&path, &cprx_gz)
            .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

        // Round-trip-validate the written file through the fail-closed compact
        // gate against the address this emit just produced.
        let read_back = std::fs::read(&path)
            .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
        load_compact_usc_prx_gz_gated(&read_back, &archive_address, &key)?;

        emitted.push(EmittedArtifact {
            name: entry.name.clone(),
            version: entry.version.clone(),
            path,
            byte_len: cprx_gz.len() as u64,
            archive_address,
        });
    }
    Ok(emitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kind lowering is lossless AND XSD-free: every [`SubdivisionKind`]
    /// round-trips `parse(tag(k)) == Some(k)`, so the owned mirror can lower
    /// `kind` to a string on emit and recover the typed variant on load
    /// without a loaded XSD ontology on the load path.
    #[test]
    fn usc_subdivision_kind_round_trips() {
        use SubdivisionKind::*;
        for k in [
            Subsection,
            Paragraph,
            Subparagraph,
            Clause,
            Subclause,
            Item,
            Subitem,
        ] {
            assert_eq!(
                SubdivisionKind::parse(k.tag()),
                Some(k),
                "kind {k:?} must round-trip via tag()/parse()"
            );
        }
    }

    /// A 3-deep owned aux fixture (`/a` → `/a/1` → `/a/1/A`) promotes to the
    /// `&'static` corpus aux losslessly: tree shape, node count, typed URNs,
    /// re-typed kinds, and the child→parent relations all survive the leak.
    fn fixture() -> OwnedUscSectionAux {
        OwnedUscSectionAux {
            urn: "/us/usc/t18/s1514A".into(),
            subdivisions: alloc::vec![OwnedUscSubdivision {
                urn: "/us/usc/t18/s1514A/a".into(),
                kind: "subsection".into(),
                num: "a".into(),
                heading: None,
                chapeau: Some("In general—".into()),
                content: None,
                children: alloc::vec![OwnedUscSubdivision {
                    urn: "/us/usc/t18/s1514A/a/1".into(),
                    kind: "paragraph".into(),
                    num: "1".into(),
                    heading: None,
                    chapeau: None,
                    content: Some("No company may discriminate.".into()),
                    children: alloc::vec![OwnedUscSubdivision {
                        urn: "/us/usc/t18/s1514A/a/1/A".into(),
                        kind: "subparagraph".into(),
                        num: "A".into(),
                        heading: None,
                        chapeau: None,
                        content: Some("by reason of lawful acts".into()),
                        children: alloc::vec![],
                    }],
                }],
            }],
            relations: alloc::vec![
                ("/us/usc/t18/s1514A/a".into(), "/us/usc/t18/s1514A".into()),
                (
                    "/us/usc/t18/s1514A/a/1".into(),
                    "/us/usc/t18/s1514A/a".into()
                ),
                (
                    "/us/usc/t18/s1514A/a/1/A".into(),
                    "/us/usc/t18/s1514A/a/1".into()
                ),
            ],
        }
    }

    fn count(subs: &[UscSubdivision]) -> usize {
        subs.iter().map(|s| 1 + count(s.children)).sum()
    }

    #[test]
    fn owned_aux_leaks_to_static_tree() {
        let aux = to_aux_leaked(&[fixture()]);
        assert_eq!(aux.len(), 1);
        let s = &aux[0];
        assert_eq!(s.urn, "/us/usc/t18/s1514A");
        assert_eq!(s.subdivisions.len(), 1, "one top-level subsection");
        assert_eq!(count(s.subdivisions), 3, "a, a/1, a/1/A");
        assert_eq!(s.relations.len(), 3, "three child→parent edges");

        let a = &s.subdivisions[0];
        assert_eq!(a.kind, SubdivisionKind::Subsection);
        assert_eq!(a.num, "a");
        assert_eq!(a.urn.value(), "/us/usc/t18/s1514A/a");
        assert_eq!(a.urn.format, IdentifierFormatConcept::UslmUrn);
        assert_eq!(a.chapeau, Some("In general—"));

        let a1 = &a.children[0];
        assert_eq!(a1.kind, SubdivisionKind::Paragraph);
        let a1a = &a1.children[0];
        assert_eq!(a1a.kind, SubdivisionKind::Subparagraph);
        assert_eq!(a1a.content, Some("by reason of lawful acts"));

        // Relations are child → parent, document-order DFS.
        assert_eq!(s.relations[0].from_urn, "/us/usc/t18/s1514A/a");
        assert_eq!(s.relations[0].to_urn, "/us/usc/t18/s1514A");
        assert_eq!(s.relations[2].from_urn, "/us/usc/t18/s1514A/a/1/A");
        assert_eq!(s.relations[2].to_urn, "/us/usc/t18/s1514A/a/1");
    }

    // ── envelope / gate fixture (no XML parse needed) ────────────────────

    /// A complete, deterministic [`UsCodePrxEnvelope`] over the
    /// [`fixture`] aux tree (one section, 3 subdivision nodes) plus a
    /// `RawBytesComplementFloor` raw leaf — codegen-free, parse-free. The
    /// `&[ontology_archive axioms]` widened in §4 reuse this same helper.
    fn witness_usc_envelope(name: &str, version: &str) -> UsCodePrxEnvelope {
        fn n(subs: &[OwnedUscSubdivision]) -> u64 {
            subs.iter().map(|s| 1 + n(&s.children)).sum()
        }
        let blob = b"<uslm>deterministic USC fixture source bytes</uslm>".to_vec();
        let source_sha256 = source_content_hash(&blob);
        let aux = alloc::vec![fixture()];
        let number_of_subdivisions = aux.iter().map(|a| n(&a.subdivisions)).sum();
        let data = OwnedCodegenData {
            entity_count: 1,
            entity_ids: alloc::vec!["/us/usc/t18/s1514A".to_string()],
            entity_kind: alloc::vec!["section".to_string()],
            entity_labels: alloc::vec![
                "Civil action to protect against retaliation in fraud cases".to_string()
            ],
            entity_defs: alloc::vec![
                "In general— No company may discriminate. by reason of lawful acts".to_string()
            ],
            word_index: Vec::new(),
            taxonomy: Vec::new(),
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        };
        UsCodePrxEnvelope {
            metadata: UscPrxMetadata {
                name: name.to_string(),
                version: version.to_string(),
                corpus_uri: super::super::USLM_NAMESPACE_URI.to_string(),
                source_url: "https://uscode.house.gov/download/usc.zip".to_string(),
                source_sha256: source_sha256.clone(),
                number_of_sections: aux.len() as u64,
                number_of_subdivisions,
            },
            data,
            aux,
            mode: RoundTripFidelity::RawBytesComplementFloor,
            // This fixture rides the universal floor (the raw blob); the
            // graph-faithful payload is absent in this tier.
            graph: None,
            raw: Some(RawSource {
                content_address: source_sha256,
                blob,
            }),
        }
    }

    #[test]
    fn usc_envelope_bytes_round_trip_and_deterministic() {
        let e = witness_usc_envelope("usc_title_18", "pl-119-90");
        let a = usc_envelope_to_bytes(&e).expect("serialize a");
        let b = usc_envelope_to_bytes(&e).expect("serialize b");
        assert_eq!(a, b, "rkyv serialization must be deterministic");
        let back = usc_envelope_from_bytes(&a).expect("deserialize");
        assert_eq!(
            e, back,
            "rkyv round-trip must be lossless incl. the aux tree"
        );
    }

    #[test]
    fn usc_raw_leaf_reconstructs_source_byte_exact() {
        let e = witness_usc_envelope("usc_title_18", "pl-119-90");
        let src = usc_reconstruct_source(&e).expect("reconstruct");
        assert_eq!(&src, &e.raw.as_ref().unwrap().blob);
        assert_eq!(source_content_hash(&src), e.metadata.source_sha256);
    }

    /// The load gate, given the genuine pins for THIS fixture, materializes a
    /// `UsCode` and — crucially — preserves subdivision DEPTH (the archive
    /// path goes through `from_codegen_with_aux`, not `from_codegen`).
    #[test]
    fn usc_load_preserves_subdivision_depth() {
        let e = witness_usc_envelope("usc_title_18", "pl-119-90");
        let bytes = usc_envelope_to_bytes(&e).expect("serialize");
        let archive_pin = source_content_hash(&bytes);
        let source_pin = e.metadata.source_sha256.clone();
        let gz = gzip(&bytes).expect("gzip");

        let usc = load_usc_prx_gz(&gz, &archive_pin, &source_pin).expect("must load + validate");
        assert_eq!(usc.section_count(), 1);
        let s = &usc.all_sections()[0];
        assert_eq!(
            s.subdivision_count(),
            3,
            "a, a/1, a/1/A survive the archive"
        );
        assert_eq!(s.relations.len(), 3);
    }

    /// The MerkleRoot leg's reason to exist: an envelope carrying a genuine
    /// source label + honest raw leaf but ONE poisoned subdivision `num` is
    /// rejected. The source leg alone would pass (raw is honest); the
    /// MerkleRoot leg binds the recursive aux tree, so poisoning it changes
    /// the content address and the gate refuses it. Mirrors the OWL
    /// `load_rejects_poisoned_data_under_honest_label`.
    #[test]
    fn usc_load_rejects_poisoned_aux_under_honest_label() {
        let honest = witness_usc_envelope("usc_title_18", "pl-119-90");
        let honest_archive_pin = source_content_hash(&usc_envelope_to_bytes(&honest).unwrap());
        let source_pin = honest.metadata.source_sha256.clone();

        // Same genuine source label + raw leaf, only one aux node poisoned.
        let mut poisoned = honest;
        poisoned.aux[0].subdivisions[0].num = "POISON".to_string();
        let gz = gzip(&usc_envelope_to_bytes(&poisoned).unwrap()).expect("gzip");

        let err = load_usc_prx_gz(&gz, &honest_archive_pin, &source_pin)
            .expect_err("poisoned aux must be rejected by the MerkleRoot leg");
        assert!(
            matches!(err, PrxError::HashMismatch { .. }),
            "expected HashMismatch from the MerkleRoot leg, got {err:?}"
        );
    }

    #[test]
    fn usc_load_rejects_corrupted_blob() {
        let any = "0".repeat(64);
        // Valid gzip wrapping garbage rkyv → bytecheck fails closed.
        let garbage = gzip(b"not a valid USC rkyv envelope at all").expect("gzip");
        let err = load_usc_prx_gz(&garbage, &any, &any).expect_err("garbage rkyv must fail");
        assert!(matches!(err, PrxError::Rkyv(_)), "got {err:?}");
        // Truncated gzip stream.
        let e = witness_usc_envelope("usc_title_18", "pl-119-90");
        let gz = gzip(&usc_envelope_to_bytes(&e).unwrap()).expect("gzip");
        assert!(
            load_usc_prx_gz(&gz[..gz.len() / 2], &any, &any).is_err(),
            "truncated must fail"
        );
    }

    /// The source leg — not the archive leg — rejects a floor envelope with
    /// no raw complement: a genuine MerkleRoot pin passes the archive check,
    /// then `usc_reconstruct_source` refuses `raw = None`.
    #[test]
    fn usc_load_rejects_floor_envelope_missing_raw_leaf() {
        let mut e = witness_usc_envelope("usc_title_18", "pl-119-90");
        e.raw = None;
        let bytes = usc_envelope_to_bytes(&e).expect("serialize");
        let archive_pin = source_content_hash(&bytes);
        let source_pin = e.metadata.source_sha256.clone();
        let gz = gzip(&bytes).expect("gzip");
        let err = load_usc_prx_gz(&gz, &archive_pin, &source_pin)
            .expect_err("floor envelope without raw leaf must be rejected");
        assert!(
            matches!(err, PrxError::SourceNotReconstructible { .. }),
            "got {err:?}"
        );
    }

    // ── emit from real USLM source (parse path) ─────────────────────────

    /// A minimal but full-shape USLM title: `<uscDoc>` wrapper, `<meta>`,
    /// `<main>`, one `<title>`, one `<section>` rooting a `subsection →
    /// paragraph` subdivision tree. `read_uslm_title` accepts it; emit
    /// projects it through `title_to_owned`.
    const SAMPLE_USC_TITLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 18</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t18">
      <num value="18">Title 18</num>
      <heading>CRIMES AND CRIMINAL PROCEDURE</heading>
      <section identifier="/us/usc/t18/s1514A">
        <num value="1514A">§ 1514A.</num>
        <heading>Civil action to protect against retaliation in fraud cases</heading>
        <subsection identifier="/us/usc/t18/s1514A/a">
          <num value="a">(a)</num>
          <chapeau>No company may discriminate against an employee—</chapeau>
          <paragraph identifier="/us/usc/t18/s1514A/a/1">
            <num value="1">(1)</num>
            <content>to provide information.</content>
          </paragraph>
        </subsection>
      </section>
    </title>
  </main>
</uscDoc>
"##;

    const FX_NAME: &str = "usc_title_18";
    const FX_VERSION: &str = "pl-119-90";
    const FX_URL: &str = "https://uscode.house.gov/download/usc.zip";

    /// Two independent `emit_usc_prx_gz` runs over the same source MUST
    /// produce byte-identical `.prx.gz`: the document-order DFS pins the aux
    /// and edge order, so the rkyv layout (deterministic put) and the gzip
    /// wrapper are bit-for-bit stable (no HashMap iteration leaks into bytes).
    #[test]
    fn usc_emit_is_byte_reproducible() {
        let a = emit_usc_prx_gz(SAMPLE_USC_TITLE.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("emit a");
        let b = emit_usc_prx_gz(SAMPLE_USC_TITLE.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("emit b");
        assert_eq!(
            a, b,
            "two independent emits must yield byte-identical .prx.gz"
        );
    }

    /// emit → load (genuine pins computed from the emit) reconstructs the
    /// corpus value: one section, the subsection→paragraph subdivision tree
    /// preserved (depth survives the archive), URN lookup + Composes edges
    /// intact.
    #[test]
    fn usc_emit_then_load_equals_corpus() {
        let src = SAMPLE_USC_TITLE.as_bytes();
        let prx_gz = emit_usc_prx_gz(src, FX_NAME, FX_VERSION, FX_URL).expect("emit");
        let archive_pin =
            crate::social::software::markup::xml::owl::prx::prx_archive_address(&prx_gz)
                .expect("archive address");
        let source_pin = source_content_hash(src);

        let usc = load_usc_prx_gz(&prx_gz, &archive_pin, &source_pin).expect("load + validate");
        assert_eq!(usc.section_count(), 1);

        let urn =
            Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, "/us/usc/t18/s1514A");
        let s = usc.section_by_urn(&urn).expect("section present by URN");
        assert_eq!(
            s.heading,
            "Civil action to protect against retaliation in fraud cases"
        );
        // Subdivision depth preserved through the archive (from_codegen_with_aux).
        assert_eq!(s.subdivision_count(), 2, "subsection a + paragraph a/1");
        assert_eq!(s.subdivisions.len(), 1);
        assert_eq!(s.subdivisions[0].num, "a");
        assert_eq!(s.subdivisions[0].kind, SubdivisionKind::Subsection);
        assert_eq!(s.subdivisions[0].children[0].num, "1");
        assert_eq!(
            s.subdivisions[0].children[0].kind,
            SubdivisionKind::Paragraph
        );
        // (a → section) and (a/1 → a) Composes edges.
        assert_eq!(s.relations.len(), 2);
    }

    // ── compact runtime codec (data + aux, no source-reconstruction payload) ──

    /// The compact codec is LOSSLESS over the `(data, aux)` reasoning view
    /// (`from(to(d,a)) == (d,a)`, including the re-derived `Composes` edges), the
    /// compact `.prx.gz` materializes back to the same corpus (section + depth +
    /// edges), and it is smaller than fetching the source. Inline fixture — runs
    /// in any checkout with no provisioned title.
    #[test]
    fn compact_usc_codec_roundtrips_smaller_and_reasoning_equivalent() {
        let src = SAMPLE_USC_TITLE.as_bytes();
        let title = read_uslm_title(SAMPLE_USC_TITLE).expect("parse sample title");
        let (data, aux) = title_to_owned(&title);

        // (1) lossless over the reasoning view, INCLUDING the re-derived edges.
        let succ = compact_usc_to_succinct(&data, &aux);
        let (data_back, aux_back) = compact_usc_from_succinct(&succ);
        assert_eq!(data_back, data, "compact codec: data column not lossless");
        assert_eq!(
            aux_back, aux,
            "compact codec: aux tree / re-derived Composes edges not lossless"
        );

        // (2) the compact .prx.gz materializes to the same corpus as the envelope.
        let prx_gz = emit_compact_usc_prx_gz(src).expect("emit compact");
        let usc = load_compact_usc_prx_gz(&prx_gz).expect("load compact");
        assert_eq!(usc.section_count(), 1);
        let urn =
            Identifier::from_codegen_static(IdentifierFormatConcept::UslmUrn, "/us/usc/t18/s1514A");
        let s = usc.section_by_urn(&urn).expect("section present by URN");
        assert_eq!(
            s.heading,
            "Civil action to protect against retaliation in fraud cases"
        );
        assert_eq!(s.subdivision_count(), 2, "subsection a + paragraph a/1");
        assert_eq!(s.subdivisions[0].num, "a");
        assert_eq!(s.subdivisions[0].kind, SubdivisionKind::Subsection);
        assert_eq!(s.subdivisions[0].children[0].num, "1");
        assert_eq!(s.relations.len(), 2, "(a→section) + (a/1→a) Composes edges");

        // (3) smaller than fetching the source.
        let source_dl = gzip(src).expect("gzip source").len();
        assert!(
            prx_gz.len() < source_dl,
            "compact .prx.gz ({}) not smaller than gzip(source) ({})",
            prx_gz.len(),
            source_dl
        );
    }

    /// COMPACTNESS GATE — for every registered, on-disk USC title small enough
    /// for the per-test CI budget, the compact runtime `.prx.gz` is smaller than
    /// fetching its source (`gzip(source)`), the codec round-trips `(data, aux)`
    /// losslessly, and it materializes to a corpus with the same section count.
    /// Registry-driven (no hardcoded title set); a title not on disk is skipped
    /// gracefully (USC titles are externally provisioned, not git-committed). The
    /// giants (> the cap) are measured by the ignored `deep_dive_all_usc_titles`.
    #[test]
    fn compact_usc_prx_gz_smaller_than_source() {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        // Same proven-CI-safe cap the archive-anchor test uses — covers the small
        // + mid titles (1/28/18/29/50) while staying well under the 30 s lane.
        const CAP: u64 = 16 * 1024 * 1024;

        let root = workspace_root();
        let mut measured = 0usize;
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
                continue;
            }
            let path = root.join(entry.local_path());
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            if meta.len() > CAP {
                continue;
            }
            let source = std::fs::read(&path).expect("read USC title");
            let text = core::str::from_utf8(&source).expect("UTF-8");
            let title = read_uslm_title(text).expect("parse title");
            let (data, aux) = title_to_owned(&title);

            let succ = compact_usc_to_succinct(&data, &aux);
            let (data_back, aux_back) = compact_usc_from_succinct(&succ);
            assert_eq!(data_back, data, "{}: data not lossless", entry.name);
            assert_eq!(aux_back, aux, "{}: aux not lossless", entry.name);

            let prx_gz = gzip(&succ).expect("gzip");
            let source_dl = gzip(&source).expect("gzip source").len();
            eprintln!(
                "USC-COMPACT {}@{}: .prx = {:.2}MB ({:.2}MB gz)  vs  SOURCE = {:.2}MB xml \
                 ({:.2}MB .xml.gz)  ->  .prx.gz is {:.2}x the download",
                entry.name,
                entry.version,
                succ.len() as f64 / 1e6,
                prx_gz.len() as f64 / 1e6,
                source.len() as f64 / 1e6,
                source_dl as f64 / 1e6,
                prx_gz.len() as f64 / source_dl.max(1) as f64,
            );
            assert!(
                prx_gz.len() < source_dl,
                "{}: compact .prx.gz ({} B) NOT smaller than gzip(source) ({} B)",
                entry.name,
                prx_gz.len(),
                source_dl,
            );

            let usc = load_compact_usc_prx_gz(&prx_gz).expect("load compact");
            assert_eq!(
                usc.section_count(),
                data.entity_count as usize,
                "{}: compact-loaded section count differs",
                entry.name
            );
            measured += 1;
        }
        // No on-disk title within the cap (a plain checkout) — correctness is
        // still covered by the inline-fixture test above.
        if measured == 0 {
            eprintln!("compact_usc gate: no on-disk USC title within the CI cap — skipped");
        }
    }

    /// The OMV/PROV-O metadata's USC structural metrics are computed
    /// FAITHFULLY from the projection — `number_of_sections` = the section
    /// count, `number_of_subdivisions` = the total subdivision-node count.
    /// Self-description (not a load-gate check), but proven correct here so
    /// the fields are not written-only.
    #[test]
    fn usc_metadata_metrics_match_corpus() {
        let e = build_usc_envelope(SAMPLE_USC_TITLE.as_bytes(), FX_NAME, FX_VERSION, FX_URL)
            .expect("build envelope");
        assert_eq!(e.metadata.number_of_sections, 1);
        assert_eq!(
            e.metadata.number_of_subdivisions, 2,
            "subsection a + paragraph a/1"
        );
        // Consistent with the archived data + the USLM markup namespace.
        assert_eq!(e.metadata.number_of_sections, e.data.entity_count);
        assert_eq!(e.metadata.corpus_uri, super::super::USLM_NAMESPACE_URI);
    }

    // ── emit-all + lock gate ────────────────────────────────────────────

    /// Every on-disk pinned USC title small enough to emit within CI's per-test
    /// budget reproduces the exact `MerkleRoot` pinned for it in `praxis.lock`
    /// `[archive_signatures]` — the emit→address→lock anchor the fail-closed load
    /// gate relies on. Checking EVERY (not just the smallest) pinned title means
    /// a stale or wrong pin is caught for that title — including the titles whose
    /// archive shifts when a section heading carries a footnote (`prose_text`).
    ///
    /// Titles larger than [`ANCHOR_EMIT_SIZE_CAP`] are skipped here so the test
    /// stays WELL under the nextest `ci` profile's 30 s terminate-after — re-
    /// emitting the whole corpus (Title 42 ≈ 108 MB) is what tipped it over
    /// before. The cap covers the small + mid pinned titles, including the
    /// footnote-heading titles 18 and 28 whose `[archive_signatures]` shift with
    /// the `prose_text` heading projection — so that regression class is self-
    /// verified in-suite (not just the smallest, unaffected, title). The larger
    /// titles (5/49/15/42) are anchored by the full-corpus `pr4xis compile` CI
    /// step (see issue tracker: wire `compile --verify` to re-derive EVERY pin)
    /// and by `loaded()`'s fail-closed prx gate; each title's emit→load round-
    /// trip is exercised by `usc_emit_then_load_equals_corpus`.
    ///
    /// USC titles are externally provisioned (fetched via `pr4xis update`, NOT
    /// git-committed), so in a plain checkout none are on disk and the test skips
    /// gracefully — the same graceful-skip doctrine `loaded()` and the emitter use.
    #[test]
    fn usc_archive_anchors_match_lock() {
        use crate::applied::data_provisioning::registry::{data_sources, lock_archive_signature};
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        /// Skip titles whose XML exceeds this — keeps the per-test emit WELL
        /// under CI's 30 s terminate-after while still covering the footnote-
        /// heading titles (18 ≈ 12 MB, 28 ≈ 8 MB) whose archive shifts.
        const ANCHOR_EMIT_SIZE_CAP: u64 = 16 * 1024 * 1024;
        let root = workspace_root();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
                continue;
            }
            let Some(pinned) = lock_archive_signature(&entry.name, &entry.version) else {
                continue; // not pinned — nothing to anchor
            };
            let path = root.join(entry.local_path());
            let Ok(meta) = std::fs::metadata(&path) else {
                continue; // not provisioned on disk — skip gracefully
            };
            if meta.len() > ANCHOR_EMIT_SIZE_CAP {
                continue; // too large for the per-test CI budget — see doc
            }
            let src = std::fs::read(&path).expect("read pinned USC title");
            let prx_gz = emit_usc_prx_gz(&src, &entry.name, &entry.version, &entry.url)
                .expect("emit pinned USC title");
            let addr = prx_archive_address(&prx_gz).expect("derive MerkleRoot");
            assert_eq!(
                addr, pinned,
                "{}@{} .prx MerkleRoot must equal its [archive_signatures] pin",
                entry.name, entry.version
            );
        }
    }

    /// COMPACT ARCHIVE ANCHOR — for every on-disk title within the CI budget
    /// that has a `[compact_archive_signatures]` pin, a fresh
    /// `emit_compact_usc_prx_gz` re-derives EXACTLY that pin. The portable
    /// (toolchain-independent) compact sibling of `usc_archive_anchors_match_lock`;
    /// keeps the committed compact pins honest — a stale or wrong pin (or a codec
    /// change that shifts the bytes) fails closed here. Skips titles > the cap and
    /// titles not on disk (USC is externally provisioned).
    #[test]
    fn compact_usc_archive_anchors_match_lock() {
        use crate::applied::data_provisioning::registry::{
            data_sources, lock_compact_archive_signature,
        };
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        const CAP: u64 = 16 * 1024 * 1024;
        let root = workspace_root();
        let mut checked = 0usize;
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
                continue;
            }
            let Some(pinned) = lock_compact_archive_signature(&entry.name, &entry.version) else {
                continue;
            };
            let path = root.join(entry.local_path());
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            if meta.len() > CAP {
                continue;
            }
            let src = std::fs::read(&path).expect("read pinned USC title");
            let cprx_gz = emit_compact_usc_prx_gz(&src).expect("emit compact");
            let addr = compact_prx_archive_address(&cprx_gz).expect("derive compact address");
            assert_eq!(
                addr, pinned,
                "{}@{} compact .prx address must equal its \
                 [compact_archive_signatures] pin (codec or source drift?)",
                entry.name, entry.version
            );
            checked += 1;
        }
        if checked == 0 {
            eprintln!("compact anchor: no pinned on-disk USC title within the cap — skipped");
        }
    }

    /// The content gate is a well-behaved lens: a compact archive loads under its
    /// genuine address, and a SINGLE flipped byte is rejected (the fail-closed
    /// `HashMismatch`) before any data is materialized.
    #[test]
    fn compact_usc_gated_load_round_trips_and_rejects_tampering() {
        let key = "usc_title_18@pl-119-90";
        let cprx_gz = emit_compact_usc_prx_gz(SAMPLE_USC_TITLE.as_bytes()).expect("emit");
        let addr = compact_prx_archive_address(&cprx_gz).expect("address");

        // Genuine address → loads.
        let usc = load_compact_usc_prx_gz_gated(&cprx_gz, &addr, key).expect("gated load");
        assert_eq!(usc.section_count(), 1);

        // A wrong pin (one hex char flipped) → fail-closed HashMismatch.
        let mut bad = addr.clone().into_bytes();
        bad[0] = if bad[0] == b'0' { b'1' } else { b'0' };
        let bad_pin = String::from_utf8(bad).unwrap();
        let err = load_compact_usc_prx_gz_gated(&cprx_gz, &bad_pin, key)
            .expect_err("a pin mismatch must be rejected");
        assert!(matches!(err, PrxError::HashMismatch { .. }), "got {err:?}");
    }

    /// The lock-driven load path fails closed for a USC envelope whose
    /// `"{name}@{version}"` has no `[archive_signatures]` pin — the MerkleRoot
    /// pin is looked up first, so an unregistered title is refused there
    /// (mirrors the OWL `load_validation_rejects_unpinned_source`).
    #[test]
    fn usc_load_from_lock_rejects_unpinned() {
        let prx_gz = emit_usc_prx_gz(
            SAMPLE_USC_TITLE.as_bytes(),
            "not_a_registered_title",
            "9.9.9",
            FX_URL,
        )
        .expect("emit");
        let err =
            load_usc_prx_gz_from_lock(&prx_gz).expect_err("unpinned USC source must be rejected");
        assert!(matches!(err, PrxError::NoArchivePin { .. }), "got {err:?}");
    }

    // ── SLICE U6: graph-faithful .prx for the proven `usc_title_1` ──────────

    /// The registry `(name, version)` of the PROVEN graph-faithful title.
    const T1_NAME: &str = "usc_title_1";
    const T1_VERSION: &str = "pl-119-90";
    const T1_URL: &str =
        "https://uscode.house.gov/download/releasepoints/us/pl/119/90/xml_usc01@119-90.zip";

    /// The LITERAL on-disk Title 1 USLM file — the raw published bytes EXACTLY,
    /// CRLFs included (the same file the writer's
    /// `real_title1_full_uscdoc_reconstruct_is_byte_exact` gate runs over).
    /// `None` when the corpus is not provisioned (graceful skip).
    fn real_title1_source() -> Option<Vec<u8>> {
        let path = workspace_root()
            .join("crates/domains/data/legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml");
        std::fs::read(&path).ok()
    }

    /// MEASUREMENT (not a gate yet) — the go/no-go for the generative-serialization
    /// redesign (retro 2026-06-06). For each provisioned USC title, split the
    /// graph-faithful `.prx` into its two layers and gzip each:
    ///
    /// - SEMANTIC graph = `UsCodeTitle` (the meaning — the master ~compact path).
    /// - COMPLEMENT = `UslmSyntaxComplement` (the per-element concrete-syntax
    ///   residue — the byte-exact tax that bloated the artifact).
    ///
    /// It then reports the per-element residue POPULATION (whitespace /
    /// attribute-order / child-order entries) — the exception-rate proxy. The
    /// hypothesis the bidi-transform literature endorses: the semantic graph is
    /// compact (< gzip(source)) and the complement is dominated by REGULAR
    /// per-element residue (whitespace = f(depth), attr order = schema order)
    /// that a generative serialization ontology regenerates — so moving it out
    /// reclaims the size. This test prints the numbers that decide whether to
    /// build that; it asserts only non-vacuity + the semantic layer being
    /// compact. Kept small (titles ≤ ~20 MB) to stay resource-light.
    #[test]
    fn prx_compactness_breakdown_measurement() {
        use std::io::Write as _;
        fn gz_len(bytes: &[u8]) -> usize {
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            e.write_all(bytes).expect("gz write");
            e.finish().expect("gz finish").len()
        }

        // Small → moderate titles only (skip the >20 MB giants for CI budget).
        let titles = ["usc_title_1", "usc_title_28", "usc_title_5"];
        let mut measured = 0usize;
        for name in titles {
            let path = workspace_root().join(format!(
                "crates/domains/data/legal/uscode/{name}/{name}-pl-119-90.xml"
            ));
            let Ok(source) = std::fs::read(&path) else {
                continue;
            };
            let envelope = build_usc_envelope(&source, name, "pl-119-90", T1_URL)
                .unwrap_or_else(|e| panic!("build {name}: {e}"));
            let g = envelope
                .graph
                .as_ref()
                .unwrap_or_else(|| panic!("{name} must be graph-faithful for this measurement"));

            let source_gz = gz_len(&source);
            let total_gz = gz_len(&usc_envelope_to_bytes(&envelope).expect("envelope bytes"));
            let semantic_gz =
                gz_len(&rkyv::to_bytes::<rkyv::rancor::Error>(&g.title).expect("rkyv title"));
            let complement_gz = gz_len(
                &rkyv::to_bytes::<rkyv::rancor::Error>(&g.complement).expect("rkyv complement"),
            );
            let ws = g.complement.regenerated.content_whitespace.len();
            let ao = g.complement.regenerated.attribute_overrides.len();
            let co = g.complement.regenerated.child_order.len();

            eprintln!(
                "COMPACTNESS {name}: source.xml={:.2}MB | gzip(source)={:.2}MB \
                 total.prx.gz={:.2}MB || semantic(graph)={:.2}MB  complement(residue)={:.2}MB \
                 || residue entries: whitespace={ws} attr_overrides={ao} child_order={co} sections={} \
                 || semantic<gzip(source)? {}",
                source.len() as f64 / 1e6,
                source_gz as f64 / 1e6,
                total_gz as f64 / 1e6,
                semantic_gz as f64 / 1e6,
                complement_gz as f64 / 1e6,
                envelope.aux.len(),
                semantic_gz < source_gz,
            );
            measured += 1;
        }
        assert!(
            measured >= 1,
            "no USC title provisioned on disk — cannot measure .prx compactness"
        );
    }

    /// The graph-faithful build over the literal Title 1 source carries the typed
    /// ontology + concrete-syntax complement (NO raw blob), and
    /// `usc_reconstruct_source` regenerates the EXACT source bytes from the GRAPH
    /// alone. The cheap in-memory cousin of the rkyv round-trip gate below.
    #[test]
    fn usc_title1_graph_faithful_reconstructs_source_byte_exact() {
        let Some(source) = real_title1_source() else {
            return; // corpus not provisioned — skip gracefully
        };
        let envelope = build_usc_envelope(&source, T1_NAME, T1_VERSION, T1_URL)
            .expect("build graph-faithful envelope over the literal Title 1 source");
        // The tier is graph-faithful: graph payload present, NO raw blob.
        assert_eq!(envelope.mode, RoundTripFidelity::ByteExactGraphFaithful);
        assert!(
            envelope.graph.is_some(),
            "graph-faithful envelope carries the ontology + complement payload"
        );
        assert!(
            envelope.raw.is_none(),
            "graph-faithful envelope stores NO raw blob"
        );
        let out = usc_reconstruct_source(&envelope).expect("reconstruct");
        assert_eq!(
            out, source,
            "usc_reconstruct_source must regenerate the exact literal Title 1 bytes from the graph"
        );
        assert_eq!(
            source_content_hash(&out),
            envelope.metadata.source_sha256,
            "reconstructed bytes must hash to the pinned source content address"
        );
    }

    /// Fail-closed: a graph-faithful envelope with no graph payload cannot
    /// reconstruct its source — `usc_reconstruct_source` refuses `graph = None`
    /// rather than fabricating bytes.
    #[test]
    fn usc_reconstruct_refuses_missing_graph_payload() {
        let Some(source) = real_title1_source() else {
            return;
        };
        let mut envelope =
            build_usc_envelope(&source, T1_NAME, T1_VERSION, T1_URL).expect("build envelope");
        envelope.graph = None;
        let err = usc_reconstruct_source(&envelope)
            .expect_err("graph-faithful envelope without its payload must be rejected");
        assert!(
            matches!(err, PrxError::SourceNotReconstructible { .. }),
            "got {err:?}"
        );
    }

    /// Fail-closed: a drifted source pin (the graph still reconstructs the true
    /// source, but it no longer matches the metadata pin) is rejected by the
    /// in-envelope honesty gate rather than returning wrong bytes — the
    /// graph-faithful analogue of the floor arm's tampered-blob test.
    #[test]
    fn usc_title1_graph_faithful_rejects_pin_drift() {
        let Some(source) = real_title1_source() else {
            return;
        };
        let mut envelope =
            build_usc_envelope(&source, T1_NAME, T1_VERSION, T1_URL).expect("build envelope");
        envelope.metadata.source_sha256 = "0".repeat(64);
        let err = usc_reconstruct_source(&envelope)
            .expect_err("pin drift must fail closed (HashMismatch)");
        assert!(matches!(err, PrxError::HashMismatch { .. }), "got {err:?}");
    }

    /// THE SLICE-U6 HARD GATE. The LITERAL on-disk `usc_title_1-pl-119-90.xml`
    /// (CRLFs included) emits as a `ByteExactGraphFaithful` envelope, serializes
    /// to rkyv bytes, loads back THROUGH the bytecheck-validated rkyv decode
    /// ([`usc_envelope_from_bytes`]), reconstructs via [`usc_reconstruct_source`]
    /// (the graph-faithful arm — typed [`UsCodeTitle`] ontology + concrete-syntax
    /// complement, NO stored raw blob), and the regenerated bytes equal the source
    /// BYTE-FOR-BYTE. This is the only non-vacuous proof that `usc_title_1`'s
    /// `.prx` is graph-faithful end-to-end: the source bytes survive the FULL
    /// serialize → bytecheck → reconstruct path, not just the in-memory
    /// capture/reconstruct of slices U1–U5.
    ///
    /// AND the completeness meter reports THIS title (`usc_title_1`, the test's
    /// own subject) graph-faithful: its declared tier is `ByteExactGraphFaithful`
    /// via the registered `UslmGraphFaithfulLens`, and it carries NO `write_uslm`
    /// gap. The cross-source consistency of EVERY other title's tier is the
    /// source-agnostic concern of the `prx_source_round_trip` meta-tests (read from
    /// the lens registry, enumerating no title), not this focused unit test —
    /// which asserts only its own subject so it never churns as other titles flip.
    /// Gated behind the on-disk file with a graceful skip — a plain checkout that
    /// hasn't provisioned Title 1 skips, the same doctrine the emitters use.
    #[test]
    fn usc_title1_graph_faithful_prx_round_trip_over_real_corpus() {
        use crate::formal::meta::well_behaved_lens::{
            CompletenessReport, DecompileKind, RoundTripFidelity as Tier, completeness_meter,
        };

        let Some(source) = real_title1_source() else {
            return; // not provisioned on disk — skip gracefully
        };

        // Emit the graph-faithful envelope: typed ontology + concrete-syntax
        // complement, NO raw blob.
        let envelope = build_usc_envelope(&source, T1_NAME, T1_VERSION, T1_URL)
            .expect("build graph-faithful envelope over the literal Title 1 source");
        assert_eq!(
            envelope.mode,
            Tier::ByteExactGraphFaithful,
            "the literal Title 1 source emits the graph-faithful tier"
        );
        assert!(envelope.graph.is_some(), "graph payload present");
        assert!(envelope.raw.is_none(), "NO stored raw blob in this tier");

        // Serialize → rkyv bytes → bytecheck-validated decode (the full path, not
        // just the in-memory capture/reconstruct of slices U1–U5).
        let rkyv_bytes = usc_envelope_to_bytes(&envelope).expect("serialize envelope to rkyv");
        let decoded =
            usc_envelope_from_bytes(&rkyv_bytes).expect("bytecheck-validated rkyv decode");

        // Reconstruct from the DECODED envelope's graph + complement.
        let out = usc_reconstruct_source(&decoded).expect("graph-faithful reconstruct");

        // BYTE-FOR-BYTE over the whole literal file. Report the EXACT first
        // byte-diff for an honest failure (a bounded 80-byte window).
        if out != source {
            let first = out
                .iter()
                .zip(source.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(out.len().min(source.len()));
            let lo = first.saturating_sub(40);
            let hi_out = (first + 40).min(out.len());
            let hi_src = (first + 40).min(source.len());
            panic!(
                "graph-faithful .prx round-trip is NOT byte-exact: out.len()={}, \
                 source.len()={}, first diff at byte {first}\n  out[..]: {:?}\n  src[..]: {:?}",
                out.len(),
                source.len(),
                String::from_utf8_lossy(&out[lo..hi_out]),
                String::from_utf8_lossy(&source[lo..hi_src]),
            );
        }
        assert_eq!(
            source_content_hash(&out),
            decoded.metadata.source_sha256,
            "the regenerated bytes must hash to the pinned source content address"
        );

        // The completeness meter reports usc_title_1 graph-faithful: declared tier
        // == ByteExactGraphFaithful and NO write_uslm gap remains.
        let meter = completeness_meter();
        let t1_row: &CompletenessReport = meter
            .iter()
            .find(|r| r.source == "usc_title_1@pl-119-90")
            .expect("usc_title_1 must have a completeness row");
        assert_eq!(
            t1_row.kind,
            DecompileKind::UsCode,
            "usc_title_1 routes through the USC decompile leaf"
        );
        assert_eq!(
            t1_row.declared,
            Tier::ByteExactGraphFaithful,
            "usc_title_1 DECLARES graph-faithful (via the registered UslmGraphFaithfulLens)"
        );
        assert!(
            t1_row.graph_faithful_gap.is_none(),
            "usc_title_1 carries NO write_uslm gap — it IS graph-faithful, got gap {:?}",
            t1_row.graph_faithful_gap
        );
        // With the title provisioned the harness MEASURES the achieved tier by
        // running the byte-exact law; it must agree with the declaration (the
        // anti-lie cross-check), never silently fall to the floor.
        assert_eq!(
            t1_row.achieved,
            Some(Tier::ByteExactGraphFaithful),
            "with the title on disk the harness must MEASURE graph-faithful (achieved == declared)"
        );
        // Every OTHER title's declared tier is the source-agnostic concern of the
        // `prx_source_round_trip` meta-tests (registry-derived, no title named);
        // this focused unit test deliberately asserts only its own subject.
    }
}
