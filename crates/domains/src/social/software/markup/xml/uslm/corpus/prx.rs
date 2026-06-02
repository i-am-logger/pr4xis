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
//! ([`source_content_hash`](crate::social::software::markup::xml::owl::prx::source_content_hash),
//! [`gzip`](crate::social::software::markup::xml::owl::prx::gzip) /
//! [`gunzip`](crate::social::software::markup::xml::owl::prx::gunzip),
//! `raw_hash::verify`, [`RoundTripFidelity`], [`PrxMetadata`],
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
//! [`UscSectionAux`](super::section_aux::UscSectionAux) side-channel
//! [`UsCode::from_codegen_with_aux`](super::UsCode::from_codegen_with_aux)
//! attaches. The archive mirrors it with an owned, rkyv-serializable
//! [`OwnedUscSectionAux`] / [`OwnedUscSubdivision`] tree.
//!
//! ## Corpus-faithful, not parse-faithful (the round-trip target)
//!
//! [`OwnedUscSubdivision`] mirrors the **corpus**
//! [`UscSubdivision`](super::section_aux::UscSubdivision) (the 7-field
//! node the corpus API exposes), NOT the richer parse
//! [`UsCodeSubdivision`](super::runtime_types::UsCodeSubdivision) (whose
//! `heading_runs` / `chapeau_runs` / `content_runs` / `refs` / `def_blocks`
//! / `markers` / `amendments` are inline-run detail). Those parse-only
//! fields are deliberately NOT archived: the corpus projection
//! ([`UsCode::from_uslm_titles_owned`](super::UsCode::from_uslm_titles_owned)
//! → `subdivisions_to_static`) already discards them, so the round-trip
//! equivalence target is the **`UsCode` corpus value** (`section_count`,
//! `subdivision_count`, `section_by_urn`, every `UscComposesEdge`,
//! `to_statute`) — not the `UsCodeTitle` parse.
//!
//! Byte-exact source fidelity (`hash(out) == hash(in)`, the #186 invariant)
//! is discharged ENTIRELY by [`RawSource::blob`] — the whole unzipped USLM
//! XML, content-addressed to the `praxis.lock` `[hashes]` pin — exactly as
//! the OWL leaf's [`RoundTripFidelity::RawBytesComplementFloor`] does. The
//! parse-only fields survive in `raw.blob`, so source regeneration is
//! lossless; `data` + `aux` are never re-hashed against the source pin.
//! USLM has no byte-exact writer + canonicalization (the analogue of the
//! OWL `write_owl` + RDFC #258 gap), so USC is permanently
//! `RawBytesComplementFloor` today; `ByteExactGraphFaithful` stays
//! unemitted, not stubbed.
//!
//! ## Citations
//!
//! - LRC, *USLM XML User Guide* §V (USC URN hierarchy / subdivision depth).
//! - Foster, Greenwald, Moore, Pierce & Schmitt (2007) "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2.
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

// =============================================================================
// Owned aux mirror — the rkyv-serializable twin of the corpus subdivision tree.
// =============================================================================

/// Owned, rkyv-serializable mirror of the corpus
/// [`UscSubdivision`](super::section_aux::UscSubdivision).
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
/// [`UscSectionAux`](super::section_aux::UscSectionAux): one section's
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
/// section corpus, its subdivision-depth aux side-channel, and the
/// OMV/PROV-O-grounded metadata. Serialized to rkyv bytes and gzip-wrapped
/// to form the `.prx.gz` artifact.
///
/// Structurally the OWL
/// [`PrxEnvelope`](crate::social::software::markup::xml::owl::prx::PrxEnvelope)
/// plus exactly ONE field — [`Self::aux`] — because the U.S. Code carries
/// subdivision DEPTH the OWL flat edge tables do not.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct UsCodePrxEnvelope {
    /// OMV/PROV-O-grounded self-description, incl. the source content hash
    /// the load gate validates.
    pub metadata: UscPrxMetadata,
    /// The archived section corpus — the owned mirror of the
    /// `CodegenData<UsCode>` interchange (flat section leaves).
    pub data: OwnedCodegenData,
    /// The subdivision-depth side-channel — the owned mirror of the
    /// [`UscSectionAux`] table [`UsCode::from_codegen_with_aux`] attaches.
    /// This is the one structural delta from the OWL envelope.
    pub aux: Vec<OwnedUscSectionAux>,
    /// The source lens's [`RoundTripFidelity`] — `RawBytesComplementFloor`
    /// for USC today (no byte-exact USLM writer + canonicalization yet).
    pub mode: RoundTripFidelity,
    /// The content-addressed source bytes (the constant-complement) — `Some`
    /// iff `mode == RawBytesComplementFloor`.
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
/// `.prx → xml` leg of the #186 byte-hash invariant. Mirrors the OWL
/// `reconstruct_source` over the USC envelope.
///
/// For [`RoundTripFidelity::RawBytesComplementFloor`] (USC today): return the
/// stored `raw.blob` after enforcing the in-envelope honesty doctrine
/// (`sha256(blob) == raw.content_address == metadata.source_sha256`). A
/// tampered blob is rejected. [`RoundTripFidelity::ByteExactGraphFaithful`]
/// would regenerate from `data`+`aux` via a byte-exact `write_uslm` + USLM
/// canonicalization — the USC analogue of the OWL `write_owl`+RDFC gap
/// (#258), unimplemented, so no envelope is emitted in that mode today.
pub fn usc_reconstruct_source(envelope: &UsCodePrxEnvelope) -> Result<Vec<u8>, PrxError> {
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
            let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
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
        RoundTripFidelity::ByteExactGraphFaithful => Err(PrxError::SourceNotReconstructible {
            reason: "byte-exact USLM graph→source regeneration (write_uslm + USLM \
                     canonicalization) is not yet implemented"
                .to_string(),
        }),
    }
}

/// Verify the source-identity leg: reconstruct the source and bind it to the
/// trusted `SourcePin` (`praxis.lock` `[hashes]`). Mirrors OWL `verify_source_leg`.
fn usc_verify_source_leg(
    envelope: &UsCodePrxEnvelope,
    source_pin: &str,
    key: &str,
) -> Result<(), PrxError> {
    match envelope.mode {
        RoundTripFidelity::RawBytesComplementFloor => {
            let source_bytes = usc_reconstruct_source(envelope)?;
            usc_verify_content_address(&source_bytes, source_pin, key)
        }
        RoundTripFidelity::ByteExactGraphFaithful => Ok(()),
    }
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
        entity_labels.push(section.heading.clone());
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
/// `(name, version, url)`. Parses via `read_uslm_title`, projects with
/// `title_to_owned`, attaches the OMV/PROV-O metadata, and carries the exact
/// source bytes as the `RawBytesComplementFloor` raw leaf (content-addressed).
pub fn build_usc_envelope(
    source: &[u8],
    name: &str,
    version: &str,
    url: &str,
) -> Result<UsCodePrxEnvelope, PrxError> {
    let text = core::str::from_utf8(source)
        .map_err(|e| PrxError::Read(format!("source is not UTF-8: {e}")))?;
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
    Ok(UsCodePrxEnvelope {
        metadata,
        data,
        aux,
        mode: RoundTripFidelity::RawBytesComplementFloor,
        raw: Some(RawSource {
            content_address: source_sha256,
            blob: source.to_vec(),
        }),
    })
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

    /// `emit_all_usc_prx_gz` round-trip-validates every on-disk USC title it
    /// emits, and any emitted title that carries a `praxis.lock`
    /// `[archive_signatures]` pin must match it.
    ///
    /// USC titles are externally provisioned (Title 18 ~12 MB … Title 42
    /// ~108 MB; fetched via `pr4xis update`, NOT git-committed), so the emitted
    /// set is environment-dependent — empty in a plain checkout, populated in a
    /// data-provisioned one. `emit_all_usc_prx_gz` returning `Ok` already means
    /// every emitted title passed the fail-closed gate (gunzip → bytecheck →
    /// `MerkleRoot` + the committed `[hashes]` source pin) on the way out; this
    /// additionally cross-checks any `[archive_signatures]` pin that exists.
    /// `#271` pins none (the runtime load-from-lock path that consumes them is
    /// the split-out perf rewire); the operator pins at release time, exactly
    /// as the OWL `#256` path does. (The per-title emit→load round-trip is
    /// exercised non-vacuously by `usc_emit_then_load_equals_corpus`.)
    #[test]
    fn usc_archive_anchors_match_lock() {
        use crate::applied::data_provisioning::registry::lock_archive_signature;
        let dir = std::env::temp_dir().join("usc_prx_archive_anchor");
        let arts = emit_all_usc_prx_gz(&dir).expect("emit all USC archives must round-trip");
        for a in &arts {
            if let Some(pinned) = lock_archive_signature(&a.name, &a.version) {
                assert_eq!(
                    a.archive_address, pinned,
                    "{}@{} .prx MerkleRoot must equal its [archive_signatures] pin",
                    a.name, a.version
                );
            }
        }
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
}
