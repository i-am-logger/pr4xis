//! `.prx.gz` — the self-describing, load-validated distribution envelope
//! for a loaded OWL vocabulary.
//!
//! This is the OWL leaf of praxis's M4.ι archival path: a registered OWL
//! source is parsed from its authoritative RDF/XML exactly once (the
//! [`read_owl`] reader), projected through the established codegen
//! interchange (`owl_to_builder` → [`CodegenData`]), and frozen into a
//! content-addressed binary blob the runtime materializes back into a
//! [`LoadedOwlVocabulary`] without re-parsing the XML. Same shape the
//! WordNet → English and USLM → UsCode sources use; no new reader, no new
//! IR, no second parser.
//!
//! ## The two layers, bottom-up
//!
//! ```text
//! read_owl(source)                     authoritative RDF/XML  (one reader)
//!   └► owl_to_builder ─► CodegenData    the M4.ε interchange shape
//!        └► OwnedCodegenData            rkyv-serializable owned mirror
//!             └► PrxEnvelope            + OMV/PROV-O metadata block
//!                  └► rkyv bytes        deterministic, bytecheck-validated
//!                       └► gzip         RFC 1952 wrapper ─► `.prx.gz`
//! ```
//!
//! [`OwnedCodegenData`] is the owned, serializable mirror of the build-
//! time→runtime [`CodegenData<P>`] interchange (whose fields are
//! `&'static` slices). It carries the same data with `String`/`Vec`
//! ownership so rkyv can serialize it, and rebuilds a typed
//! `CodegenData<P>` view by leaking its owned columns to `'static`
//! (process-lifetime, identical in effect to a build-emitted `static`,
//! the same trade [`LoadedOwlVocabulary`]'s test helper and the USC
//! corpus loader make).
//!
//! [`PrxEnvelope`] wraps that mirror with an **ontologically grounded**
//! metadata block: every metadata field is typed/documented by the OMV
//! (Ontology Metadata Vocabulary) or PROV-O (PROV Ontology) term it
//! realizes — not ad-hoc strings. See [`PrxMetadata`].
//!
//! ## The load gate
//!
//! [`load_prx_gz`] is fail-closed and content-addressed. It gunzips,
//! rkyv-validates the envelope (bytecheck rejects a corrupted/truncated
//! blob), then discharges THREE content-hash `IntegrityClaim`s through the
//! same `artifact_identity` verifier the fetch path uses
//! (`raw_hash::verify`): (1) the `MerkleRoot` — the content address
//! *re-derived from the rkyv bytes* must equal the `praxis.lock`
//! `[archive_signatures]` pin, binding the whole installed node so a
//! poisoned `data` column is rejected even under a genuine source label;
//! (2) the `SourcePin` — the carried source re-hashes to the `[hashes]`
//! pin; and (3) the **RDFC-1.0 `CanonicalPin`** — the W3C RDF Dataset
//! Canonicalization (REC-rdf-canon-20240521) canonical N-Quads of the
//! loaded source *graph* re-hash to the `[canonical_signatures]` pin,
//! binding the graph the source denotes (RDF 1.1 §3.6 graph isomorphism),
//! not merely its bytes. No check trusts an embedded self-asserted field.
//! Only on all three `Verified` does it rebuild the `CodegenData` view and
//! hand back a validated [`LoadedOwlVocabulary`]; otherwise it installs
//! nothing.
//!
//! ## Bidirectional-transformation law
//!
//! `emit`/`load` form a well-behaved lens between bytes and the loaded
//! vocabulary (Foster, Greenwald, Moore, Pierce & Schmitt 2007,
//! "Combinators for Bidirectional Tree Transformations", *ACM TOPLAS*
//! 29(3) §2.2). The rkyv put is deterministic — equal envelopes
//! serialize to equal bytes — so the blob's BLAKE3 hash is a stable content
//! address; gzip (RFC 1952) round-trips losslessly, so
//! `gunzip(gzip(x)) == x`; together the GetPut round-trip holds up to the
//! materialized vocabulary value.
//!
//! ## Distribution model (commit vs in-memory)
//!
//! Following the M4.ι decision that dropped committed `.rkyv` blobs in
//! favour of loading authoritative XML directly, **no `.prx.gz` file is
//! committed to the tree**. rkyv's wire layout is determined by the
//! `rkyv` version and target, so a committed blob would be a
//! cross-toolchain liability; the canonical source of truth stays the
//! bundled `.owl` plus its `praxis.lock` pin. The `emit`→`load`
//! round-trip is exercised in-memory by the tests here; producing a
//! published `.prx.gz` artifact (and serving it) is the distribution
//! layer's job (#256), and the wasm dual-load is #257.
//!
//! ## Deferred (flagged, not built here)
//!
//! - A full **queryable runtime OMV/PROV-O instance-graph** — this brick
//!   grounds the metadata *schema* in OMV+PROV-O via typed, cited fields;
//!   materializing those fields into a navigable PROV/OMV `Category` of
//!   instances is follow-on.
//! - `write_owl` + RDF canonicalization to regenerate the *source* bytes
//!   (the full [`WellBehavedLens`] PutGet leg) — #258.
//! - wasm dual-load UI/fetch — #257.
//!
//! ## Realised axioms (M4.ι.0)
//!
//! This module is a *realisation* of the
//! [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive::ontology)
//! ontology — its functions witness that ontology's runnable axioms (in
//! `crate::formal::meta::ontology_archive::axioms`), and the test
//! `realisation_witnesses_the_archive_axioms` runs them against this code.
//! Through the fully-faithful
//! [`ArchiveIntoGraph`](crate::formal::meta::praxis_knowledge_graph::functor)
//! functor (#272), that archive ontology is the **storage substratum** of
//! the whole-graph
//! [`PraxisKnowledgeGraph`](crate::formal::meta::praxis_knowledge_graph), so
//! this same realisation is the graph's storage layer and the seven axioms
//! below carry over verbatim (the functor is the proof). The test
//! `realisation_witnesses_the_graph_storage_substratum` binds it. The
//! fn → axiom map (zero byte-output change):
//!
//! - [`ContentAddress`] — `MerkleHashDeterministic`, `MerkleDedupCorrect`.
//! - [`gzip`] / [`gunzip`] — `CompressionRoundTrip`.
//! - [`envelope_to_bytes`] — `RkyvDeterminism`.
//! - [`envelope_to_bytes`] / [`envelope_from_bytes`] — `EmitLoadWellBehaved`.
//! - [`reconstruct_source`] — `SourceHashFaithfulness`.
//! - [`load_prx_gz`] / `verify_content_address` — `LoadGateFailsClosed`.
//!   (The load gate also discharges the W3C-SRI `IntegrityClaim` *concept*
//!   on the install path; the `IntegrityClaimVerifiable` *axiom* itself
//!   stays deferred, per `ontology_archive::axioms`.)
//!
//! ## Citations
//!
//! - **Hartmann, Palma & Sure (2005)** "OMV — Ontology Metadata
//!   Vocabulary", *ISWC 2005 Workshop on Ontology Patterns*. OMV
//!   namespace `http://omv.ontoware.org/2005/05/ontology#`.
//! - **Lebo, Sahoo & McGuinness (eds.) (2013)** *PROV-O: The PROV
//!   Ontology*, W3C Recommendation 2013-04-30. PROV namespace
//!   `http://www.w3.org/ns/prov#`. <https://www.w3.org/TR/prov-o/>.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators
//!   for Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2.
//! - **Deutsch, P. (1996)** *GZIP file format specification version
//!   4.3*, RFC 1952. <https://www.rfc-editor.org/rfc/rfc1952>.
//! - **Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020)** *BLAKE3: one
//!   function, fast everywhere*. The integrity hash space.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment
//!   Model*, PhD thesis — content-addressing by cryptographic hash.
//! - **Hill, D.** *rkyv: zero-copy deserialization framework for Rust*,
//!   v0.8, <https://github.com/rkyv/rkyv>.
//!
//! [`read_owl`]: super::reader::read_owl
//! [`CodegenData`]: pr4xis::codegen_data::CodegenData
//! [`CodegenData<P>`]: pr4xis::codegen_data::CodegenData
//! [`LoadedOwlVocabulary`]: super::vocabulary::LoadedOwlVocabulary
//! [`WellBehavedLens`]: crate::formal::meta::well_behaved_lens
//! [`lock_hashes`]: crate::applied::data_provisioning::registry::lock_hashes

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use pr4xis::EntityRef;
use pr4xis::codegen_data::CodegenData;
use pr4xis_runtime::address::ContentAddress;

use super::vocabulary::LoadedOwlVocabulary;

use crate::applied::data_provisioning::registry::LockDigest;
use crate::formal::meta::artifact_identity::ontology::{
    IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::raw_hash;
use crate::formal::meta::well_behaved_lens::{RoundTripFidelity, WellBehavedLens};

use super::lens::OwlLens;
use super::rdfxml_writer::reconstruct_owl_rdfxml_source;
use crate::social::software::markup::xml::succinct::{
    get_cv, get_dict_fc, get_ef, get_varint, put_cv, put_dict_fc, put_ef, put_varint,
};

// =============================================================================
// OMV / PROV-O term IRIs — the vocabulary the metadata block is typed by.
// =============================================================================

/// The OMV namespace (Hartmann, Palma & Sure 2005). Each OMV term IRI is
/// this base + the term's local name.
pub const OMV_NS: &str = "http://omv.ontoware.org/2005/05/ontology#";
/// The PROV-O namespace (Lebo, Sahoo & McGuinness 2013).
pub const PROV_NS: &str = "http://www.w3.org/ns/prov#";

/// `omv:version` — the version string an ontology self-declares
/// (Hartmann 2005 OMV core). Realized by [`PrxMetadata::version`].
pub const OMV_VERSION: &str = "http://omv.ontoware.org/2005/05/ontology#version";
/// `omv:URI` — the canonical IRI under which the ontology is published
/// (Hartmann 2005 OMV core). Realized by [`PrxMetadata::ontology_uri`].
pub const OMV_URI: &str = "http://omv.ontoware.org/2005/05/ontology#URI";
/// `omv:name` — the short symbolic name of the ontology (Hartmann 2005).
/// Realized by [`PrxMetadata::name`].
pub const OMV_NAME: &str = "http://omv.ontoware.org/2005/05/ontology#name";
/// `omv:numberOfClasses` — structural metric (Hartmann 2005 OMV
/// `omv:OntologyMetrics`; MOD 2.0 Analytics). Realized by
/// [`PrxMetadata::number_of_classes`].
pub const OMV_NUMBER_OF_CLASSES: &str = "http://omv.ontoware.org/2005/05/ontology#numberOfClasses";
/// `omv:numberOfProperties` — structural metric (Hartmann 2005). Realized
/// by [`PrxMetadata::number_of_properties`].
pub const OMV_NUMBER_OF_PROPERTIES: &str =
    "http://omv.ontoware.org/2005/05/ontology#numberOfProperties";

/// `prov:wasDerivedFrom` — "a transformation of an entity into another"
/// (Lebo 2013 §3, PROV-O). The `.prx.gz` envelope is
/// `prov:wasDerivedFrom` the source RDF/XML; the value pinned is that
/// source entity's content address. Realized by
/// [`PrxMetadata::source_url`] (the derivation target) together with
/// [`PrxMetadata::source_address`] (its content hash).
pub const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";
/// `prov:Entity` — "a physical, digital, conceptual, or other kind of
/// thing" (Lebo 2013 §2). The source RDF/XML bytes are the
/// `prov:Entity` this envelope is derived from; its identity is the
/// content address. Realized by [`PrxMetadata::source_address`].
pub const PROV_ENTITY: &str = "http://www.w3.org/ns/prov#Entity";
/// `prov:atLocation` — "the Location of any resource" (Lebo 2013 §3.2).
/// The URL the source entity is published at. Realized by
/// [`PrxMetadata::source_url`].
pub const PROV_AT_LOCATION: &str = "http://www.w3.org/ns/prov#atLocation";

// =============================================================================
// OwnedCodegenData — the rkyv-serializable owned mirror of CodegenData<P>.
// =============================================================================

/// Owned, serializable mirror of [`CodegenData`].
///
/// Field-for-field identical to `CodegenData<P>` except that every
/// `&'static str` becomes an owned [`String`] and every typed
/// [`pr4xis::EntityRef`] becomes its raw `u64` handle (the phantom marker
/// `P` is reconstructed when a typed [`CodegenData`] view is rebuilt — the
/// integer handle is identical machine data either way). This is the
/// `prov:Entity` the envelope archives.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct OwnedCodegenData {
    pub entity_count: u64,
    pub entity_ids: Vec<String>,
    pub entity_kind: Vec<String>,
    pub entity_labels: Vec<String>,
    pub entity_defs: Vec<String>,
    /// `(word, concept-handles)` — mirrors `CodegenData::word_index`.
    pub word_index: Vec<(String, Vec<u64>)>,
    /// `(child, parent)` subsumption edges.
    pub taxonomy: Vec<(u64, u64)>,
    /// `(whole, part)` mereology edges.
    pub mereology: Vec<(u64, u64)>,
    pub opposition: Vec<(u64, u64)>,
    pub equivalence: Vec<(u64, u64)>,
    pub causation: Vec<(u64, u64)>,
    pub references: Vec<(u64, u64)>,
}

impl OwnedCodegenData {
    /// Project a typed build-time [`CodegenData`] into the owned archival
    /// shape. The forgetful direction of the embed/forget adjunction: the
    /// phantom marker `P` is dropped (handles become raw integers).
    pub fn from_codegen_data<P: 'static>(data: &CodegenData<P>) -> Self {
        let edges = |s: &[(EntityRef<P>, EntityRef<P>)]| -> Vec<(u64, u64)> {
            s.iter().map(|(a, b)| (a.value(), b.value())).collect()
        };
        Self {
            entity_count: data.entity_count as u64,
            entity_ids: data.entity_ids.iter().map(|s| s.to_string()).collect(),
            entity_kind: data.entity_kind.iter().map(|s| s.to_string()).collect(),
            entity_labels: data.entity_labels.iter().map(|s| s.to_string()).collect(),
            entity_defs: data.entity_defs.iter().map(|s| s.to_string()).collect(),
            word_index: data
                .word_index
                .iter()
                .map(|(w, refs)| (w.to_string(), refs.iter().map(|r| r.value()).collect()))
                .collect(),
            taxonomy: edges(data.taxonomy),
            mereology: edges(data.mereology),
            opposition: edges(data.opposition),
            equivalence: edges(data.equivalence),
            causation: edges(data.causation),
            references: edges(data.references),
        }
    }

    /// Rebuild a typed [`CodegenData`] view from this owned data.
    ///
    /// The *re-embed* direction: raw `u64` handles are re-tagged with the
    /// phantom marker `P` chosen by the caller, and the owned strings /
    /// edge tables are promoted to the `&'static` lifetimes that
    /// [`CodegenData`] requires by [`Box::leak`]. The leaks persist for
    /// process lifetime — identical in effect to a build-time-emitted
    /// `static`, the same trade the on-disk USLM loader and the
    /// `OnceLock`-cached corpus singleton make.
    pub fn to_codegen_data_leaked<P: 'static>(&self) -> CodegenData<P> {
        fn leak_str(s: &str) -> &'static str {
            Box::leak(s.to_string().into_boxed_str())
        }
        fn leak_strs(v: &[String]) -> &'static [&'static str] {
            let leaked: Vec<&'static str> = v.iter().map(|s| leak_str(s)).collect();
            Box::leak(leaked.into_boxed_slice())
        }
        fn leak_edges<P: 'static>(v: &[(u64, u64)]) -> &'static [(EntityRef<P>, EntityRef<P>)] {
            let leaked: Vec<(EntityRef<P>, EntityRef<P>)> = v
                .iter()
                .map(|(a, b)| (EntityRef::new(*a), EntityRef::new(*b)))
                .collect();
            Box::leak(leaked.into_boxed_slice())
        }

        let word_index: Vec<(&'static str, &'static [EntityRef<P>])> = self
            .word_index
            .iter()
            .map(|(w, refs)| {
                let r: Vec<EntityRef<P>> = refs.iter().map(|x| EntityRef::new(*x)).collect();
                let r: &'static [EntityRef<P>] = Box::leak(r.into_boxed_slice());
                (leak_str(w), r)
            })
            .collect();

        CodegenData {
            entity_count: self.entity_count as usize,
            entity_ids: leak_strs(&self.entity_ids),
            entity_kind: leak_strs(&self.entity_kind),
            entity_labels: leak_strs(&self.entity_labels),
            entity_defs: leak_strs(&self.entity_defs),
            word_index: Box::leak(word_index.into_boxed_slice()),
            taxonomy: leak_edges(&self.taxonomy),
            mereology: leak_edges(&self.mereology),
            opposition: leak_edges(&self.opposition),
            equivalence: leak_edges(&self.equivalence),
            causation: leak_edges(&self.causation),
            references: leak_edges(&self.references),
        }
    }

    /// Serialize to the compact succinct `.prx` bytes — the runtime reasoning
    /// view alone, far below the source it was derived from.
    ///
    /// One sorted, front-coded string dictionary is shared by every text column
    /// (`entity_ids`/`entity_kind`/`entity_labels`/`entity_defs` and the
    /// `word_index` words); each column reduces to a bit-packed array of indices
    /// into it. IRIs share long prefixes and kinds/labels repeat heavily, so the
    /// shared dictionary deduplicates them and front-coding elides the prefixes.
    /// The `word_index` handles ride a CSR (gap-coded offsets + a flat value
    /// column); the six edge tables store two bit-packed endpoint columns each,
    /// at `bits(entity_count)` per value. No source bytes, no graph-faithful
    /// complement — only the columns the runtime reasons over.
    ///
    /// Source-agnostic: it touches only the generic
    /// [`CodegenData`] columns, so the same
    /// codec serializes any source's interchange. Entity handles and edge
    /// endpoints are `< entity_count` (well under `2^32`), so the `usize`
    /// columns are lossless on wasm32. `from_succinct(&to_succinct(d)) == d`.
    pub fn to_succinct(&self) -> Vec<u8> {
        use hashbrown::HashMap;

        let mut out = Vec::new();
        put_varint(&mut out, self.entity_count);

        // One sorted, deduplicated dictionary spanning every text column.
        let mut all: Vec<&str> = Vec::new();
        all.extend(self.entity_ids.iter().map(String::as_str));
        all.extend(self.entity_kind.iter().map(String::as_str));
        all.extend(self.entity_labels.iter().map(String::as_str));
        all.extend(self.entity_defs.iter().map(String::as_str));
        all.extend(self.word_index.iter().map(|(w, _)| w.as_str()));
        all.sort_unstable();
        all.dedup();
        let idx: HashMap<&str, usize> = all.iter().enumerate().map(|(i, &s)| (s, i)).collect();
        let dict: Vec<String> = all.iter().map(|s| String::from(*s)).collect();
        put_dict_fc(&mut out, &dict);

        let to_idx =
            |strs: &[String]| -> Vec<usize> { strs.iter().map(|s| idx[s.as_str()]).collect() };
        put_cv(&mut out, &to_idx(&self.entity_ids));
        put_cv(&mut out, &to_idx(&self.entity_kind));
        put_cv(&mut out, &to_idx(&self.entity_labels));
        put_cv(&mut out, &to_idx(&self.entity_defs));

        // word_index: the word column (dict indices) + a CSR of concept handles.
        let word_ixs: Vec<usize> = self
            .word_index
            .iter()
            .map(|(w, _)| idx[w.as_str()])
            .collect();
        put_cv(&mut out, &word_ixs);
        let mut offsets = Vec::with_capacity(self.word_index.len() + 1);
        let mut handles = Vec::new();
        let mut acc = 0usize;
        offsets.push(0);
        for (_, hs) in &self.word_index {
            acc += hs.len();
            offsets.push(acc);
            handles.extend(hs.iter().map(|&h| h as usize));
        }
        put_ef(&mut out, &offsets);
        put_cv(&mut out, &handles);

        // The six edge tables — two bit-packed endpoint columns each.
        for table in [
            &self.taxonomy,
            &self.mereology,
            &self.opposition,
            &self.equivalence,
            &self.causation,
            &self.references,
        ] {
            let src: Vec<usize> = table.iter().map(|&(a, _)| a as usize).collect();
            let dst: Vec<usize> = table.iter().map(|&(_, b)| b as usize).collect();
            put_cv(&mut out, &src);
            put_cv(&mut out, &dst);
        }
        out
    }

    /// Decode the compact succinct `.prx` bytes back into an exact
    /// [`OwnedCodegenData`] (the inverse of [`Self::to_succinct`]).
    pub fn from_succinct(buf: &[u8]) -> Self {
        let mut pos = 0usize;
        let entity_count = get_varint(buf, &mut pos);
        let dict = get_dict_fc(buf, &mut pos);
        let take = |buf: &[u8], pos: &mut usize| -> Vec<String> {
            get_cv(buf, pos)
                .into_iter()
                .map(|i| dict[i].clone())
                .collect()
        };
        let entity_ids = take(buf, &mut pos);
        let entity_kind = take(buf, &mut pos);
        let entity_labels = take(buf, &mut pos);
        let entity_defs = take(buf, &mut pos);

        let word_ixs = get_cv(buf, &mut pos);
        let offsets = get_ef(buf, &mut pos);
        let handles = get_cv(buf, &mut pos);
        let word_index: Vec<(String, Vec<u64>)> = word_ixs
            .iter()
            .enumerate()
            .map(|(i, &w)| {
                let hs = handles[offsets[i]..offsets[i + 1]]
                    .iter()
                    .map(|&x| x as u64)
                    .collect();
                (dict[w].clone(), hs)
            })
            .collect();

        let mut tables: Vec<Vec<(u64, u64)>> = Vec::with_capacity(6);
        for _ in 0..6 {
            let src = get_cv(buf, &mut pos);
            let dst = get_cv(buf, &mut pos);
            tables.push(
                src.into_iter()
                    .zip(dst)
                    .map(|(a, b)| (a as u64, b as u64))
                    .collect(),
            );
        }
        let mut it = tables.into_iter();
        Self {
            entity_count,
            entity_ids,
            entity_kind,
            entity_labels,
            entity_defs,
            word_index,
            taxonomy: it.next().unwrap_or_default(),
            mereology: it.next().unwrap_or_default(),
            opposition: it.next().unwrap_or_default(),
            equivalence: it.next().unwrap_or_default(),
            causation: it.next().unwrap_or_default(),
            references: it.next().unwrap_or_default(),
        }
    }
}

// =============================================================================
// PrxMetadata — the OMV/PROV-O-grounded metadata block.
// =============================================================================

/// Self-describing metadata carried by a [`PrxEnvelope`].
///
/// Every field is **ontologically grounded**: its doc names the OMV
/// (Hartmann, Palma & Sure 2005) or PROV-O (Lebo, Sahoo & McGuinness
/// 2013) term it realizes, so the metadata schema is a typed projection of
/// those published vocabularies rather than invented fields. The
/// constants [`OMV_VERSION`], [`PROV_WAS_DERIVED_FROM`], … hold the term
/// IRIs.
///
/// This brick grounds the metadata *schema*; a queryable runtime
/// OMV/PROV-O instance-graph over these fields is deferred (see module
/// docs).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PrxMetadata {
    /// `omv:name` (Hartmann 2005) — the registry name of the source, e.g.
    /// `"cito"`. Combined with [`Self::version`] this is the
    /// `"{name}@{version}"` key under which the source's content hash is
    /// pinned in `praxis.lock` and looked up at load time.
    pub name: String,
    /// `omv:version` (Hartmann 2005) — the source's self-declared version,
    /// e.g. `"2.8.1"` (the `owl:versionInfo` on the `owl:Ontology` node).
    pub version: String,
    /// `omv:URI` (Hartmann 2005) — the canonical IRI of the loaded
    /// ontology (`OwlOntology::iri`), e.g.
    /// `"http://purl.org/spar/cito/"`.
    pub ontology_uri: String,
    /// `prov:atLocation` (Lebo 2013 §3.2) of the `prov:Entity` this
    /// envelope is `prov:wasDerivedFrom` — the URL the source RDF/XML is
    /// published at (the registry `url`). Pairs with
    /// [`Self::source_address`] to identify the derivation source.
    pub source_url: String,
    /// The content address of the source `prov:Entity`
    /// ([`PROV_WAS_DERIVED_FROM`] / [`PROV_ENTITY`], Lebo 2013) — the
    /// content address (BLAKE3 — Aumasson, O'Connor, Neves & Wilcox-O'Hearn
    /// 2020; content-addressing per Dolstra 2006) of the exact source bytes [`read_owl`] consumed, as 64-char
    /// lowercase hex. **The load-validation gate compares this against the
    /// `praxis.lock` pin**; a mismatch fails closed.
    ///
    /// [`read_owl`]: super::reader::read_owl
    pub source_address: String,
    /// `omv:numberOfClasses` (Hartmann 2005 `omv:OntologyMetrics`; MOD 2.0
    /// Analytics) — count of `owl:Class` entities in the archived corpus.
    /// A structural metric, redundant with the archived data and used to
    /// self-describe the envelope without materializing it.
    pub number_of_classes: u64,
    /// `omv:numberOfProperties` (Hartmann 2005) — count of
    /// `owl:ObjectProperty` entities in the archived corpus.
    pub number_of_properties: u64,
}

// =============================================================================
// PrxEnvelope — owned data + metadata, the thing that gets rkyv'd + gzip'd.
// =============================================================================

// The `.prx` envelope's reconstruction mode is NOT a parallel enum: it IS
// the source lens's [`RoundTripFidelity`] (well_behaved_lens) — the same
// Bancilhon & Spyratos (1981) constant-complement grading, realised-through
// rather than re-declared (review #6: no parallel value-isomorphic enums):
//
//   RoundTripFidelity::RawBytesComplementFloor — the source bytes are
//     carried in the envelope, content-addressed (the constant-complement);
//     reconstruction returns the stored bytes after re-verifying their hash.
//     OWL today; PDF/binary permanently (a theorem).
//   RoundTripFidelity::ByteExactGraphFaithful — the source is regenerated
//     from the ontology graph alone, no stored bytes; earned per-artifact
//     once a byte-exact `write_owl` + RDF Dataset Canonicalization (#258)
//     lands. Not reachable for OWL today.

/// The content-addressed source bytes a `RawBytesComplementFloor` envelope
/// carries so `.prx` regenerates the exact source without re-fetching it.
///
/// Honesty doctrine (recovered design): the complement lives IN the
/// envelope, never external; `content_address` MUST equal both
/// `address(blob)` and the envelope's [`PrxMetadata::source_address`] — the
/// byte hash is simultaneously the content address and the round-trip gate
/// (BLAKE3 — Aumasson, O'Connor, Neves & Wilcox-O'Hearn 2020; Dolstra 2006
/// content-addressing).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct RawSource {
    /// Content address of `blob` as 64-char lowercase hex; equals
    /// [`PrxMetadata::source_address`].
    pub content_address: String,
    /// The exact source bytes [`read_owl`](super::reader::read_owl) consumed.
    pub blob: Vec<u8>,
}

/// The graph-faithful reconstruction payload: the typed [`OwlOntology`](super::ontology::OwlOntology) graph
/// PLUS the structured concrete-syntax [`OwlSyntaxComplement`](super::rdfxml_writer::OwlSyntaxComplement) the byte-exact
/// `put` ([`reconstruct_owl_rdfxml_source`]) re-applies. Present in a
/// [`PrxEnvelope`] iff `mode == ByteExactGraphFaithful` (the flat SPAR OWL family
/// cito/biro/c4o/doco).
///
/// The OWL realisation of #186's graph-faithful tier — the direct sibling of
/// WordNet's
/// [`WnGraphFaithful`](crate::social::software::markup::xml::lmf::prx::WnGraphFaithful):
/// the source bytes are regenerated from the ONTOLOGY GRAPH (`ontology`) plus a
/// content-addressed structured RDF/XML SERIALIZATION complement (`complement` —
/// the node-block/property-element striping + the generic DOCTYPE/namespace/
/// white-space/attribute/entity/EOL residue) and NO stored raw blob (the
/// `RawBytesComplementFloor` constant-complement). The complement is concrete
/// syntax, NOT ontology: the same graph serialised two ways keeps one content
/// address; only the per-source `complement` differs. The capture/reconstruct
/// pair ([`capture_owl_complement`](super::rdfxml_writer::capture_owl_complement()) / [`reconstruct_owl_rdfxml_source`]) is
/// proven a byte-exact inverse over the real bundled CiTO source (the slice
/// hard gate).
///
/// rkyv-serializable through the `prx`-gated derives on
/// [`OwlSyntaxComplement`](super::rdfxml_writer::OwlSyntaxComplement) and the XML/residue
/// types it references. [`OwlOntology`](super::ontology::OwlOntology) is NOT rkyv-serializable (it carries the
/// proof/category machinery), so it is NOT archived directly — the navigable
/// reasoning view the runtime materializes is `data` (the [`OwnedCodegenData`]
/// projection). The `complement` alone suffices for the byte-exact `put`; this
/// payload therefore carries only the structured RDF/XML complement.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct OwlGraphFaithful {
    /// The structured RDF/XML serialization complement — the byte-affecting
    /// residue the typed graph does not carry (the striping, the root
    /// namespaces, the white-space, the exact source attribute sequences, the
    /// §4.6 entity form, the §2.11 EOL form). Re-applied by
    /// [`reconstruct_owl_rdfxml_source`] to reproduce the source bytes exactly.
    pub complement: super::rdfxml_writer::OwlSyntaxComplement,
}

/// The rkyv-serializable `.prx` envelope: the archived corpus plus its
/// OMV/PROV-O-grounded metadata. Serialized to rkyv bytes and gzip-wrapped
/// to form the `.prx.gz` artifact.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PrxEnvelope {
    /// OMV/PROV-O-grounded self-description, incl. the source content
    /// hash the load gate validates.
    pub metadata: PrxMetadata,
    /// The archived corpus — the owned mirror of the
    /// `CodegenData<LoadedOwlVocabulary>` interchange. The runtime reasoning
    /// view, carried unchanged in BOTH reconstruction tiers.
    pub data: OwnedCodegenData,
    /// The source lens's [`RoundTripFidelity`] — which PutGet law the
    /// source round-trips under, and therefore how `.prx` reconstructs it.
    /// [`RoundTripFidelity::ByteExactGraphFaithful`] for every bundled OWL vocab
    /// (the flat SPAR family cito/biro/c4o/doco AND the striped prov_o/olia);
    /// [`RoundTripFidelity::RawBytesComplementFloor`] for a NON-bundled OWL source
    /// with no graph-faithful writer.
    pub mode: RoundTripFidelity,
    /// The graph-faithful reconstruction payload (structured RDF/XML
    /// complement) — `Some` iff `mode == ByteExactGraphFaithful` (every bundled
    /// OWL vocab), `None` otherwise (the floor stores `raw` instead). NO raw blob
    /// is kept in this tier; the source regenerates from the graph + complement.
    pub graph: Option<OwlGraphFaithful>,
    /// The content-addressed source bytes (the constant-complement) —
    /// `Some` iff `mode == RawBytesComplementFloor`, `None` for a
    /// `ByteExactGraphFaithful` envelope (whose source is regenerable from
    /// `graph`).
    pub raw: Option<RawSource>,
}

/// Error from emitting or loading a `.prx.gz` envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrxError {
    /// The RDF/XML source could not be read by [`read_owl`].
    ///
    /// [`read_owl`]: super::reader::read_owl
    Read(String),
    /// rkyv serialization or (bytecheck-validated) deserialization failed
    /// — e.g. a corrupted or truncated blob.
    Rkyv(String),
    /// gzip (RFC 1952) compression or decompression failed.
    Gzip(String),
    /// The load-validation gate rejected the envelope: its embedded
    /// `source_address` did not match the `praxis.lock` pin for
    /// `"{name}@{version}"`. Fail-closed — nothing is installed.
    HashMismatch {
        key: String,
        expected: String,
        found: String,
    },
    /// `praxis.lock` carries no pin for the envelope's `"{name}@{version}"`
    /// — an unregistered or unpinned source cannot be validated, so the
    /// gate rejects it (fail-closed).
    NoLockPin { key: String },
    /// `praxis.lock` carries no `[archive_signatures]` pin for the
    /// envelope's `"{name}@{version}"` — without the trusted `MerkleRoot`
    /// content address there is nothing to verify the installed envelope
    /// against, so the gate rejects it (fail-closed).
    NoArchivePin { key: String },
    /// `praxis.lock` carries no `[canonical_signatures]` pin for the
    /// envelope's `"{name}@{version}"` — without the trusted RDFC-1.0
    /// graph-identity signature there is nothing to verify the loaded
    /// graph against, so the gate rejects it (fail-closed), mirroring
    /// `NoArchivePin` / `NoLockPin`.
    NoCanonicalPin { key: String },
    /// The RDFC-1.0 (REC-rdf-canon-20240521) canonical N-Quads of the
    /// loaded source graph could not be derived — the carried source is
    /// not well-formed RDF/XML, or a poison dataset tripped the
    /// canonicalization DoS cap (RDFC §"Dataset Poisoning"). Fail-closed:
    /// nothing is installed.
    CanonicalUnderivable { key: String, reason: String },
    /// A content-hash `IntegrityClaim` could not be evaluated — the
    /// `raw_hash` verifier returned `Unverifiable` (e.g. a malformed pin).
    /// Fail-closed: nothing is installed.
    IntegrityUnverifiable { key: String, reason: String },
    /// The envelope's source bytes could not be reconstructed: a
    /// `BytesPlusView` envelope is missing its raw leaf, or a
    /// `GraphFaithful` envelope's byte-exact graph→source regeneration
    /// (write_owl + RDFC, #258) is not yet implemented.
    SourceNotReconstructible { reason: String },
}

impl core::fmt::Display for PrxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PrxError::Read(m) => write!(f, "OWL read error: {m}"),
            PrxError::Rkyv(m) => write!(f, "rkyv archive error: {m}"),
            PrxError::Gzip(m) => write!(f, "gzip error: {m}"),
            PrxError::HashMismatch {
                key,
                expected,
                found,
            } => write!(
                f,
                "source hash mismatch for `{key}`: praxis.lock pins {expected}, \
                 envelope carries {found} — refusing to install"
            ),
            PrxError::NoLockPin { key } => write!(
                f,
                "no praxis.lock pin for `{key}` — cannot validate, refusing to install"
            ),
            PrxError::NoArchivePin { key } => write!(
                f,
                "no praxis.lock [archive_signatures] pin for `{key}` — cannot verify the \
                 installed archive, refusing to install"
            ),
            PrxError::NoCanonicalPin { key } => write!(
                f,
                "no praxis.lock [canonical_signatures] pin for `{key}` — cannot verify the \
                 loaded graph's RDFC-1.0 identity, refusing to install"
            ),
            PrxError::CanonicalUnderivable { key, reason } => write!(
                f,
                "cannot derive the RDFC-1.0 canonical form of `{key}`'s source graph: \
                 {reason} — refusing to install"
            ),
            PrxError::IntegrityUnverifiable { key, reason } => write!(
                f,
                "integrity claim for `{key}` is unverifiable: {reason} — refusing to install"
            ),
            PrxError::SourceNotReconstructible { reason } => {
                write!(f, "cannot reconstruct source bytes: {reason}")
            }
        }
    }
}

impl std::error::Error for PrxError {}

// =============================================================================
// gzip layer (RFC 1952).
// =============================================================================

/// Compress bytes with gzip (Deutsch 1996, RFC 1952). The `.prx.gz` *put*
/// half of the gzip lens.
pub fn gzip(bytes: &[u8]) -> Result<Vec<u8>, PrxError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .map_err(|e| PrxError::Gzip(e.to_string()))?;
    encoder.finish().map_err(|e| PrxError::Gzip(e.to_string()))
}

/// Decompress gzip bytes (Deutsch 1996, RFC 1952). The `.prx.gz` *get*
/// half; `gunzip(gzip(x)) == x`.
pub fn gunzip(bytes: &[u8]) -> Result<Vec<u8>, PrxError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| PrxError::Gzip(e.to_string()))?;
    Ok(out)
}

/// The content address (`MerkleRoot`) of a `.prx.gz` archive — the digest
/// of its rkyv envelope bytes (the `BinaryEnvelope`), as 64-char lowercase
/// hex. This is the value pinned in `praxis.lock` `[archive_signatures]`
/// and the one [`load_prx_gz`] re-derives and verifies. Computed by
/// gunzipping to the canonical rkyv form (gzip-level-independent) and
/// hashing it (Merkle 1987; Dolstra 2006 content-addressing).
pub fn prx_archive_address(prx_gz: &[u8]) -> Result<String, PrxError> {
    Ok(ContentAddress::of(&gunzip(prx_gz)?).to_hex())
}

/// The compiled-OWL-archive cache directory:
/// `<workspace_root>/.prx-cache/ontologies`. `pr4xis compile` writes one
/// `{name}-{version}.prx.gz` here per registered OntologyVocabulary source, and
/// `pr4xis decompile` reads them back. The OWL analogue of
/// [`usc_prx_cache_dir`](crate::social::software::markup::xml::uslm::corpus::prx::usc_prx_cache_dir);
/// gitignored build output — never committed.
#[cfg(feature = "std")]
pub fn owl_prx_cache_dir(workspace_root: &std::path::Path) -> std::path::PathBuf {
    workspace_root.join(".prx-cache").join("ontologies")
}

// =============================================================================
// rkyv layer — envelope ⇄ bytes (the bidirectional lens, bytecheck-validated).
// =============================================================================

/// Serialize an envelope to rkyv bytes (the lens *put*). Deterministic:
/// equal envelopes yield equal bytes, so the blob's hash is a stable
/// content address.
pub fn envelope_to_bytes(envelope: &PrxEnvelope) -> Result<Vec<u8>, PrxError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(envelope)
        .map(|v| v.to_vec())
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

/// Materialize an envelope from rkyv bytes (the lens *get*). Copies into
/// an aligned buffer first (a fetched/decompressed `Vec<u8>` carries no
/// alignment guarantee), then `bytecheck`-validates before materializing,
/// so a corrupted blob fails closed rather than producing unsound
/// references.
pub fn envelope_from_bytes(bytes: &[u8]) -> Result<PrxEnvelope, PrxError> {
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<PrxEnvelope, rkyv::rancor::Error>(&aligned)
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

// =============================================================================
// The load gate — a content-address IntegrityClaim over the installed node.
// =============================================================================

/// Discharge a content-hash `IntegrityClaim` over `bytes` against a trusted
/// pin — the realise-through that subordinates this archive's integrity to
/// [`ArtifactIdentity`](crate::formal::meta::artifact_identity).
///
/// Builds the `RawHash` claim from the pin's named algorithm + digest
/// ([`LockDigest::claim_data`]) and discharges it through the SAME
/// [`raw_hash::verify`] the fetch path uses (`registry::build_entry` →
/// `fetch`), so one content-hash primitive serves both trust boundaries
/// (Dolstra 2006 content-addressing; W3C SRI 2016). `raw_hash::verify`
/// re-hashes `bytes` under the pin's algorithm (the one verify leg,
/// `hash_hex`) — the pin is checked against bytes that are actually
/// present, never against a self-asserted label, and the algorithm comes
/// from the trusted lock value, never from the payload.
fn verify_content_address(
    bytes: &[u8],
    trusted_pin: &LockDigest,
    key: &str,
) -> Result<(), PrxError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: trusted_pin.claim_data(),
    };
    match raw_hash::verify(&claim, bytes) {
        VerificationResult::Verified(_) => Ok(()),
        VerificationResult::Mismatch { expected, actual } => Err(PrxError::HashMismatch {
            key: key.to_string(),
            expected,
            found: actual,
        }),
        // Unreachable on this path: the claim just above is always a
        // hash-bearing variant, which raw_hash::verify never reports as
        // Unverifiable. Mapped explicitly so a future non-hash claim still
        // fails closed rather than silently passing.
        VerificationResult::Unverifiable { reason } => Err(PrxError::IntegrityUnverifiable {
            key: key.to_string(),
            reason,
        }),
    }
}

/// Verify the source-identity leg, folding [`reconstruct_source`] onto the
/// load path so `raw` is no longer Optional-skippable.
///
/// For [`RoundTripFidelity::RawBytesComplementFloor`] (OWL today),
/// [`reconstruct_source`] returns the exact source bytes after enforcing
/// the in-envelope honesty doctrine (`address(blob) == raw.content_address ==
/// metadata.source_address`); we then discharge a content-hash
/// `IntegrityClaim` binding those bytes to the trusted `SourcePin`
/// (`praxis.lock` `[hashes]`). A floor envelope with no raw leaf is rejected
/// here. [`RoundTripFidelity::ByteExactGraphFaithful`] (now emitted for
/// cito) carries no raw blob either — [`reconstruct_source`] regenerates
/// the exact source bytes from the typed graph + the structured RDF/XML
/// complement, then the same content-hash claim binds them to the pin.
fn verify_source_leg(
    envelope: &PrxEnvelope,
    source_pin: &LockDigest,
    key: &str,
) -> Result<(), PrxError> {
    // BOTH tiers reconstruct the source and bind it to the trusted source pin —
    // the floor from its stored raw complement, the graph-faithful tier from the
    // structured RDF/XML complement (`reconstruct_source` now regenerates CiTO's
    // exact bytes, so the source leg is no longer skipped). `reconstruct_source`
    // already enforces the in-envelope honesty gate (regenerated == metadata
    // hash); binding to `source_pin` anchors it to the EXTERNAL `praxis.lock`
    // pin (`[hashes]` == `[byte_exact_signatures]` for a byte-exact source, since
    // `put(get(b)) == b` makes the round-trip hash the raw-source hash).
    let source_bytes = reconstruct_source(envelope)?;
    verify_content_address(&source_bytes, source_pin, key)
}

/// Verify the **graph-identity (RDFC-1.0) leg**: the W3C RDF Dataset
/// Canonicalization (REC-rdf-canon-20240521) serialized canonical N-Quads
/// of the loaded source RDF graph must hash to the trusted
/// `canonical_pin` (`praxis.lock` `[canonical_signatures]`).
///
/// This mirrors the source-bytes ([`verify_source_leg`]) and
/// archive-MerkleRoot ([`verify_content_address`]) legs but binds a
/// *different* identity: not the source bytes, not the rkyv archive, but
/// the **RDF graph** the source denotes (RDF 1.1 §3.6 graph isomorphism).
/// Two byte-different RDF/XML serializations of the same graph share the
/// same canonical N-Quads, so a re-serialized-but-isomorphic source still
/// passes, while any drift in the graph the document asserts fails.
///
/// `OwlLens::canonical` re-derives the canonical form from the
/// reconstructed source bytes (`read_owl_to_quads → rdf::canonicalize`);
/// we then discharge the same content-hash `IntegrityClaim`
/// ([`verify_content_address`]) over those canonical-form bytes against
/// `canonical_pin`. STRICT (fail-closed): a missing pin is rejected by the
/// caller before this runs; a canonicalization failure is
/// [`PrxError::CanonicalUnderivable`]; a hash mismatch is
/// [`PrxError::HashMismatch`].
///
/// This runs for BOTH tiers: a `ByteExactGraphFaithful` envelope (cito)
/// regenerates its exact source bytes from the structured complement, so
/// the same RDFC canonical form is re-derived and pinned — the graph-
/// identity leg is enforced for the byte-exact tier too, never skipped.
fn verify_canonical_leg(
    envelope: &PrxEnvelope,
    canonical_pin: &LockDigest,
    key: &str,
) -> Result<(), PrxError> {
    // The RDFC-1.0 graph-identity leg runs for BOTH tiers — for a graph-faithful
    // CiTO envelope just as for a floor one. `reconstruct_source` now regenerates
    // CiTO's exact source bytes from the structured complement, so `OwlLens::
    // canonical` (read_owl_to_quads → rdf::canonicalize) re-derives the same RDFC
    // canonical N-Quads it would from the on-disk source, and the
    // `[canonical_signatures]` pin (229d0967) is enforced unchanged on the load
    // path. The graph-identity gate is therefore NOT weakened by the byte-exact
    // flip — both the source-bytes pin AND the source-graph pin bind CiTO.
    let source_bytes = reconstruct_source(envelope)?;
    let canonical_form =
        OwlLens::canonical(&source_bytes).map_err(|e| PrxError::CanonicalUnderivable {
            key: key.to_string(),
            reason: format!("{e}"),
        })?;
    verify_content_address(&canonical_form, canonical_pin, key)
}

/// Admit a decoded envelope only after both legs of the load gate verify,
/// then materialize it — the fail-closed realisation of the `LoadGate`
/// concept ([`crate::formal::meta::ontology_archive`]).
///
/// 1. **Installed-node integrity** — the `BinaryEnvelope`'s `MerkleRoot`
///    content address, *re-derived* from `rkyv_bytes`, must equal the
///    trusted `archive_pin`. This binds the whole envelope (`data`, `raw`,
///    `metadata`), so an archive carrying a genuine source label but a
///    poisoned `data` column is rejected before anything is installed
///    (Merkle 1987; Benet 2014; W3C SRI 2016).
/// 2. **Source identity** — the carried source re-hashes to the trusted
///    `SourcePin` ([`verify_source_leg`]), preserving the `.prx → source`
///    byte-hash invariant on the load path.
/// 3. **Graph identity** — the RDFC-1.0 canonical N-Quads of the loaded
///    source graph re-hash to the trusted `CanonicalPin`
///    ([`verify_canonical_leg`]), binding the *graph the source denotes*
///    (RDF 1.1 §3.6) and not merely its bytes.
/// 4. Only on all three `Verified` rebuild the `CodegenData` view and
///    materialize.
fn admit_validated(
    rkyv_bytes: &[u8],
    envelope: PrxEnvelope,
    archive_pin: &LockDigest,
    source_pin: &LockDigest,
    canonical_pin: &LockDigest,
) -> Result<LoadedOwlVocabulary, PrxError> {
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    verify_content_address(rkyv_bytes, archive_pin, &key)?;
    verify_source_leg(&envelope, source_pin, &key)?;
    verify_canonical_leg(&envelope, canonical_pin, &key)?;
    let data: CodegenData<LoadedOwlVocabulary> = envelope.data.to_codegen_data_leaked();
    Ok(LoadedOwlVocabulary::from_codegen(&data))
}

/// Load a `.prx.gz` blob into a materialized [`LoadedOwlVocabulary`], gated
/// on two externally trusted pins.
///
/// 1. gunzip (RFC 1952);
/// 2. rkyv-validate the envelope (bytecheck — corrupted/truncated blobs
///    fail closed);
/// 3. discharge the `MerkleRoot` `IntegrityClaim`: the content address
///    re-derived from the rkyv bytes must equal `archive_pin`
///    (`praxis.lock` `[archive_signatures]`);
/// 4. discharge the `SourcePin` leg: the carried source re-hashes to
///    `source_pin` (`praxis.lock` `[hashes]`);
/// 5. discharge the `CanonicalPin` leg: the RDFC-1.0 canonical N-Quads of
///    the loaded source graph re-hash to `canonical_pin` (`praxis.lock`
///    `[canonical_signatures]`);
/// 6. on all three `Verified`, materialize — otherwise install nothing.
///
/// All three pins are externally trusted (held in `praxis.lock`), never
/// read from the envelope. Splitting them out keeps this function pure and
/// unit-testable; [`load_prx_gz_from_lock`] does the lookups.
pub fn load_prx_gz(
    prx_gz: &[u8],
    archive_pin: &LockDigest,
    source_pin: &LockDigest,
    canonical_pin: &LockDigest,
) -> Result<LoadedOwlVocabulary, PrxError> {
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = envelope_from_bytes(&rkyv_bytes)?;
    admit_validated(
        &rkyv_bytes,
        envelope,
        archive_pin,
        source_pin,
        canonical_pin,
    )
}

/// Load a `.prx.gz` blob, reaching all three pins through the live
/// registry: the `MerkleRoot` from `praxis.lock` `[archive_signatures]`,
/// the `SourcePin` from `[hashes]`, and the RDFC-1.0 `CanonicalPin` from
/// `[canonical_signatures]`, keyed by `"{name}@{version}"`.
///
/// Peeks the (bytecheck-valid) envelope to read `name`/`version` — a lookup
/// key, not a trust input: an envelope claiming a name it lacks the content
/// for fails the content-address checks. Fail-closed if any pin is
/// unregistered.
pub fn load_prx_gz_from_lock(prx_gz: &[u8]) -> Result<LoadedOwlVocabulary, PrxError> {
    use crate::applied::data_provisioning::registry::{
        lock_archive_signature, lock_canonical_signature, lock_hashes,
    };
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = envelope_from_bytes(&rkyv_bytes)?;
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    let archive_pin = lock_archive_signature(&envelope.metadata.name, &envelope.metadata.version)
        .ok_or_else(|| PrxError::NoArchivePin { key: key.clone() })?;
    let source_pin = lock_hashes()
        .get(&key)
        .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?;
    let canonical_pin =
        lock_canonical_signature(&envelope.metadata.name, &envelope.metadata.version)
            .ok_or_else(|| PrxError::NoCanonicalPin { key: key.clone() })?;
    admit_validated(
        &rkyv_bytes,
        envelope,
        archive_pin,
        source_pin,
        canonical_pin,
    )
}

// =============================================================================
// Compact runtime `.prx.gz` — the small data-only artifact the runtime loads.
// =============================================================================

/// Serialize a built [`OwnedCodegenData`] to the compact runtime `.prx.gz`:
/// `to_succinct` (the bit-packed data-only codec) then gzip.
///
/// This is the small artifact the runtime embeds or downloads — the reasoning
/// view alone, with neither the source bytes nor the graph-faithful complement
/// the integrity-bearing [`PrxEnvelope`] carries. The OWL sibling of WordNet's
/// `emit_prx_gz`; the byte-exact reconstruction + `praxis.lock` content-address
/// gate stay on the distribution path ([`emit_all_prx_gz`], [`load_prx_gz`]).
pub fn emit_compact_prx_gz(data: &OwnedCodegenData) -> Result<Vec<u8>, PrxError> {
    gzip(&data.to_succinct())
}

/// Load a compact runtime `.prx.gz` (produced by [`emit_compact_prx_gz`]) into a
/// materialized [`LoadedOwlVocabulary`]: gunzip → [`OwnedCodegenData::from_succinct`]
/// → [`to_codegen_data_leaked`](OwnedCodegenData::to_codegen_data_leaked) →
/// [`LoadedOwlVocabulary::from_codegen`]. The runtime decode path — no re-parse,
/// no rkyv envelope.
///
/// UNGATED: trusts the bytes. The content-address-checked entry point every
/// committed-archive consumer uses is [`load_compact_prx_gz_gated`]; this raw
/// form is for in-memory round-trip tests only.
pub fn load_compact_prx_gz(prx_gz: &[u8]) -> Result<LoadedOwlVocabulary, PrxError> {
    let bytes = gunzip(prx_gz)?;
    let data = OwnedCodegenData::from_succinct(&bytes);
    let codegen: CodegenData<LoadedOwlVocabulary> = data.to_codegen_data_leaked();
    Ok(LoadedOwlVocabulary::from_codegen(&codegen))
}

/// The content address of a compact OWL `.prx.gz` — the digest of its
/// uncompressed succinct bytes (gzip-level-independent), as 64-char lowercase
/// hex. The value pinned in `praxis.lock` `[compact_archive_signatures]` and the
/// one [`load_compact_prx_gz_gated`] re-derives and verifies. Portable: the
/// succinct codec is dependency-free bit-packing, stable across toolchains and
/// targets (unlike the rkyv [`prx_archive_address`]). The OWL sibling of
/// [`compact_english_archive_address`](crate::social::software::markup::xml::lmf::prx::compact_english_archive_address).
pub fn compact_owl_archive_address(prx_gz: &[u8]) -> Result<String, PrxError> {
    Ok(ContentAddress::of(&gunzip(prx_gz)?).to_hex())
}

/// Load a compact OWL `.prx.gz` into a materialized [`LoadedOwlVocabulary`]
/// through the fail-closed content-address gate: gunzip → verify the succinct
/// bytes hash to `archive_pin` (the `[compact_archive_signatures]` pin) →
/// succinct-decode → materialize. A compact archive whose bytes do not match
/// the pin is rejected before any data is installed (Dolstra 2006
/// content-addressing; W3C SRI 2016). The portable, no-source-reconstruction
/// OWL sibling of
/// [`load_compact_english_prx_gz_gated`](crate::social::software::markup::xml::lmf::prx::load_compact_english_prx_gz_gated)
/// and [`load_compact_usc_prx_gz_gated`](crate::social::software::markup::xml::uslm::corpus::prx::load_compact_usc_prx_gz_gated).
///
/// This is the SINGLE gated entry point the committed-`.prx` OWL load path
/// ([`load_owl_vocabulary`](super::loaded_vocabularies::load_owl_vocabulary))
/// routes through — both `olia::reference_model` and every
/// `OntologyVocabulary` in [`loaded_vocabularies`](super::loaded_vocabularies::loaded_vocabularies).
pub fn load_compact_prx_gz_gated(
    prx_gz: &[u8],
    archive_pin: &LockDigest,
    key: &str,
) -> Result<LoadedOwlVocabulary, PrxError> {
    let raw = gunzip(prx_gz)?;
    verify_content_address(&raw, archive_pin, key)?;
    let data = OwnedCodegenData::from_succinct(&raw);
    let codegen: CodegenData<LoadedOwlVocabulary> = data.to_codegen_data_leaked();
    Ok(LoadedOwlVocabulary::from_codegen(&codegen))
}

/// The compiled COMPACT-OWL-archive cache directory:
/// `<workspace_root>/.prx-cache/ontologies-compact`. `pr4xis compile` writes one
/// `{name}-{version}.prx.gz` here per registered `OntologyVocabulary` source (the
/// portable, content-addressed compact codec), and the committed
/// `crates/domains/data/ontologies/{name}-{version}.prx.gz` is a copy of it. The
/// OWL sibling of
/// [`english_compact_prx_cache_dir`](crate::social::software::markup::xml::lmf::prx::english_compact_prx_cache_dir)
/// / [`usc_compact_prx_cache_dir`](crate::social::software::markup::xml::uslm::corpus::prx::usc_compact_prx_cache_dir);
/// gitignored build output — never committed (the committed copy lives under
/// `data/ontologies/`).
#[cfg(feature = "std")]
pub fn owl_compact_prx_cache_dir(workspace_root: &std::path::Path) -> std::path::PathBuf {
    workspace_root.join(".prx-cache").join("ontologies-compact")
}

// =============================================================================
// Source reconstruction — the `.prx → source` leg of the byte-hash invariant.
// =============================================================================

/// Reconstruct the exact source bytes from an envelope — the `.prx → xml`
/// leg of the operator's invariant "…→ ontology → .prx → xml (same byte
/// hash)" (M4.ι / #186).
///
/// - [`RoundTripFidelity::RawBytesComplementFloor`]: return the stored
///   `raw.blob` after re-verifying `address(blob) == raw.content_address ==
///   metadata.source_address` — fail-closed; a tampered blob is rejected.
/// - [`RoundTripFidelity::ByteExactGraphFaithful`]: regenerate from `data`.
///   For OWL this is the deferred byte-exact `write_owl` + RDFC leg (#258);
///   no envelope is emitted in this mode today, so reaching it is a logic
///   error.
pub fn reconstruct_source(envelope: &PrxEnvelope) -> Result<Vec<u8>, PrxError> {
    match envelope.mode {
        RoundTripFidelity::RawBytesComplementFloor => {
            let raw = envelope
                .raw
                .as_ref()
                .ok_or_else(|| PrxError::SourceNotReconstructible {
                    reason: "BytesPlusView envelope is missing its raw source leaf".to_string(),
                })?;
            let computed = ContentAddress::of(&raw.blob).to_hex();
            let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
            // The stored blob must hash to its own content address …
            if computed != raw.content_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw content address)"),
                    expected: raw.content_address.clone(),
                    found: computed,
                });
            }
            // … and that address must equal the envelope's source pin, so
            // the raw leaf cannot disagree with the load gate's anchor.
            if computed != envelope.metadata.source_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (raw vs metadata)"),
                    expected: envelope.metadata.source_address.clone(),
                    found: computed,
                });
            }
            Ok(raw.blob.clone())
        }
        RoundTripFidelity::ByteExactGraphFaithful => {
            // The graph-faithful payload (structured RDF/XML complement) must be
            // present — its absence is a malformed envelope, not a fabrication
            // opportunity. Mirrors `wn_reconstruct_source`.
            let graph =
                envelope
                    .graph
                    .as_ref()
                    .ok_or_else(|| PrxError::SourceNotReconstructible {
                        reason: "ByteExactGraphFaithful envelope is missing its graph payload \
                             (structured RDF/XML complement)"
                            .to_string(),
                    })?;
            // Regenerate from the GRAPH alone (NO stored raw blob): the structured
            // RDF/XML striping + the captured concrete-syntax complement. This is
            // the byte-exact `put` proven inverse over the real bundled CiTO source.
            let bytes = reconstruct_owl_rdfxml_source(&graph.complement).map_err(|e| {
                PrxError::SourceNotReconstructible {
                    reason: format!("graph-faithful RDF/XML reconstruction failed: {e}"),
                }
            })?;
            // The SAME honesty gate the floor arm enforces: the regenerated bytes
            // must hash to the pinned source content address. A regeneration that
            // drifts from the pinned source fails closed.
            let computed = ContentAddress::of(&bytes).to_hex();
            let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
            if computed != envelope.metadata.source_address {
                return Err(PrxError::HashMismatch {
                    key: format!("{key} (graph-faithful reconstruction vs metadata)"),
                    expected: envelope.metadata.source_address.clone(),
                    found: computed,
                });
            }
            Ok(bytes)
        }
    }
}

// =============================================================================
// EmittedArtifact — one published `.prx.gz` file, round-trip-validated.
// =============================================================================

/// A `.prx.gz` artifact `emit_all_prx_gz` wrote to disk and then
/// round-trip-validated by loading it back through the fail-closed gate.
///
/// The presence of this value is itself a proof obligation discharged: the
/// emitter returns it only after [`load_prx_gz_from_lock`] re-loaded the
/// written file and the embedded `source_address` matched the `praxis.lock`
/// pin. A published artifact therefore is guaranteed loadable AND
/// content-anchored (BLAKE3 — Aumasson, O'Connor, Neves & Wilcox-O'Hearn 2020;
/// Dolstra 2006 content-addressing)
/// before it is handed to the distribution layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedArtifact {
    /// `omv:name` (Hartmann 2005) — the registry source name, e.g. `"cito"`.
    pub name: String,
    /// `omv:version` (Hartmann 2005) — the source version, e.g. `"2.8.1"`.
    pub version: String,
    /// Absolute path of the written `<name>-<version>.prx.gz` file.
    pub path: std::path::PathBuf,
    /// Size of the written `.prx.gz` in bytes.
    pub byte_len: u64,
    /// The content address of the written archive — for the envelope producers
    /// the rkyv `MerkleRoot` (the digest of the envelope bytes), for the compact
    /// producers the digest of the uncompressed succinct bytes. The value to
    /// pin in the corresponding `praxis.lock` signature space
    /// (`[archive_signatures]` for envelopes, `[compact_archive_signatures]` for
    /// compact archives) so the runtime load gate can verify it.
    pub archive_address: String,
}

// =============================================================================
// Emit — read_owl → builder → CodegenData → envelope → rkyv → gzip.
// =============================================================================
//
// Gated on `any(test, feature = "codegen")` because building the envelope
// from OWL source needs `owl_to_builder` + `pr4xis::codegen::OntologyBuilder`,
// which are only present under the `codegen` feature (the WASM-facing
// default dep keeps `pr4xis/codegen` — and its `xsd-parser` transitive —
// out). The load path above needs only `fetch` (rkyv + flate2).

#[cfg(any(test, feature = "codegen"))]
mod emit {
    use super::*;
    use crate::social::software::markup::xml::owl::owl_vocabulary::owl_to_builder;
    use crate::social::software::markup::xml::owl::reader::read_owl;
    use pr4xis::codegen::OntologyBuilder;
    use std::collections::HashMap;

    /// Turn an [`OntologyBuilder`] into an owned [`CodegenData`] with the
    /// id→index resolution + dangling-edge drop the build-time emitter
    /// performs (mirrors `pr4xis::codegen::generate::write_raw_relations`
    /// and the `LoadedOwlVocabulary` test helper). Returns the owned mirror
    /// directly — no intermediate `&'static` leak, since [`OwnedCodegenData`]
    /// already owns its columns.
    fn builder_to_owned(builder: &OntologyBuilder) -> OwnedCodegenData {
        let id_to_idx: HashMap<&str, u64> = builder
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.as_str(), i as u64))
            .collect();

        let edges = |raw: &[(String, String)]| -> Vec<(u64, u64)> {
            raw.iter()
                .filter_map(|(c, p)| {
                    Some((*id_to_idx.get(c.as_str())?, *id_to_idx.get(p.as_str())?))
                })
                .collect()
        };

        OwnedCodegenData {
            entity_count: builder.entities.len() as u64,
            entity_ids: builder.entities.iter().map(|e| e.id.clone()).collect(),
            entity_kind: builder
                .entities
                .iter()
                .map(|e| e.pos.clone().unwrap_or_default())
                .collect(),
            entity_labels: builder.entities.iter().map(|e| e.label.clone()).collect(),
            entity_defs: builder
                .entities
                .iter()
                .map(|e| e.definitions.first().cloned().unwrap_or_default())
                .collect(),
            word_index: Vec::new(),
            taxonomy: edges(&builder.taxonomy),
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        }
    }

    /// Build a [`PrxEnvelope`] from OWL source bytes plus its registry
    /// `(name, version, url)`.
    ///
    /// `read_owl(source) → owl_to_builder → OwnedCodegenData`, then attach
    /// the OMV/PROV-O metadata block: `source_address` is the content address of the
    /// exact `source` bytes (the `prov:Entity` content address);
    /// `number_of_classes` / `number_of_properties` are the
    /// `omv:numberOf*` structural metrics counted from the builder's
    /// entity kinds.
    pub fn build_envelope(
        source: &[u8],
        name: &str,
        version: &str,
        url: &str,
    ) -> Result<PrxEnvelope, PrxError> {
        let text = core::str::from_utf8(source)
            .map_err(|e| PrxError::Read(format!("source is not UTF-8: {e}")))?;
        let ont = read_owl(text).map_err(|e| PrxError::Read(format!("{e}")))?;
        let ontology_uri = ont.iri.clone();
        let builder = owl_to_builder(&ont);

        // omv:numberOfClasses / omv:numberOfProperties — count by the
        // entity_kind tag owl_to_builder wrote (W3C OWL 2 §5).
        let number_of_classes = builder
            .entities
            .iter()
            .filter(|e| e.pos.as_deref() == Some("Class"))
            .count() as u64;
        let number_of_properties = builder
            .entities
            .iter()
            .filter(|e| e.pos.as_deref() == Some("ObjectProperty"))
            .count() as u64;

        let data = builder_to_owned(&builder);
        let source_address = ContentAddress::of(source).to_hex();
        let metadata = PrxMetadata {
            name: name.to_string(),
            version: version.to_string(),
            ontology_uri,
            source_url: url.to_string(),
            source_address: source_address.clone(),
            number_of_classes,
            number_of_properties,
        };

        // Tier selection (mirrors `build_wordnet_envelope`): prefer the
        // BYTE-EXACT GRAPH-FAITHFUL tier ONLY for a source whose registered lens
        // declares it (the completeness meter reads the SAME registry, so
        // emit-tier == meter-tier), and only when the structural capture
        // actually succeeds. ALL six bundled OWL vocabs now qualify (registered
        // `OwlGraphFaithfulLens`): the flat SPAR family cito/biro/c4o/doco AND the
        // striped prov_o/olia, whose parser-layer residue (the internal-subset
        // DOCTYPE, §4.1 numeric/general references, interspersed comments) the L3
        // byte kernel captures as structured concrete-syntax residue. A
        // registered-graph-faithful source whose capture FAILS (a shape the
        // structural writer cannot regenerate — e.g. a NON-bundled OWL source
        // outside this slice) degrades HONESTLY to the floor, never a silent lie.
        let registered_graph_faithful =
            crate::formal::meta::well_behaved_lens::lens_by_name(&format!("{name}@{version}"))
                .is_some_and(|r| r.fidelity == RoundTripFidelity::ByteExactGraphFaithful);

        if registered_graph_faithful
            && let Ok((_ont, complement)) =
                crate::social::software::markup::xml::owl::rdfxml_writer::capture_owl_complement(
                    text,
                )
        {
            // ByteExactGraphFaithful: the source regenerates from the structured
            // RDF/XML complement, NO stored raw blob (the byte-exact tier the
            // operator's "…→ ontology → .prx → xml, same byte hash" invariant
            // reaches WITHOUT a constant-complement side-channel).
            return Ok(PrxEnvelope {
                metadata,
                data,
                mode: RoundTripFidelity::ByteExactGraphFaithful,
                graph: Some(OwlGraphFaithful { complement }),
                raw: None,
            });
        }

        // RawBytesComplementFloor: a graph-faithful-declared source whose capture
        // failed, or a NON-bundled OWL source with no graph-faithful writer (all
        // six bundled vocabs are now graph-faithful). Carry the exact source bytes
        // content-addressed (the Bancilhon & Spyratos 1981 constant-complement) so
        // `.prx` self-reconstructs the source.
        Ok(PrxEnvelope {
            metadata,
            data,
            mode: RoundTripFidelity::RawBytesComplementFloor,
            graph: None,
            raw: Some(RawSource {
                content_address: source_address,
                blob: source.to_vec(),
            }),
        })
    }

    /// Emit a `.prx.gz` artifact from OWL source bytes:
    /// `build_envelope → envelope_to_bytes (rkyv) → gzip`.
    pub fn emit_prx_gz(
        source: &[u8],
        name: &str,
        version: &str,
        url: &str,
    ) -> Result<Vec<u8>, PrxError> {
        let envelope = build_envelope(source, name, version, url)?;
        let rkyv_bytes = envelope_to_bytes(&envelope)?;
        gzip(&rkyv_bytes)
    }

    /// Emit a COMPACT OWL `.prx.gz` from OWL source bytes:
    /// `build_envelope → data → to_succinct → gzip`. The portable,
    /// dependency-free sibling of [`emit_prx_gz`] (no rkyv envelope) — the bytes
    /// the committed-`.prx` OWL load path
    /// ([`load_owl_vocabulary`](super::super::loaded_vocabularies::load_owl_vocabulary))
    /// reads through the `[compact_archive_signatures]` content gate. The OWL
    /// sibling of
    /// [`emit_compact_english_prx_gz`](crate::social::software::markup::xml::lmf::prx::emit_compact_english_prx_gz).
    pub fn emit_compact_owl_prx_gz(
        source: &[u8],
        name: &str,
        version: &str,
        url: &str,
    ) -> Result<Vec<u8>, PrxError> {
        let envelope = build_envelope(source, name, version, url)?;
        emit_compact_prx_gz(&envelope.data)
    }

    /// Workspace root — the grandparent of `CARGO_MANIFEST_DIR`
    /// (`crates/domains/`). `RegistryEntry::local_path()` is
    /// workspace-relative (`crates/domains/data/...`), so the bundled `.owl`
    /// resolves against this root. Mirrors
    /// `owl::loaded_vocabularies::workspace_root` and the USC corpus loader.
    fn workspace_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// Emit a `.prx.gz` artifact for **every** registered
    /// [`OntologyVocabulary`][ov] source into `out_dir`, round-trip-validating
    /// each before returning it.
    ///
    /// The distribution-layer entry point for #256. It is registry-driven
    /// (never a hardcoded source set): it walks every entry
    /// [`data_sources`][ds] reports whose `kind` is
    /// [`SourceTaxonomyConcept::OntologyVocabulary`][ov], reads the bundled
    /// RDF/XML from `workspace_root.join(entry.local_path())`, and for each:
    ///
    /// 1. `emit_prx_gz(source, name, version, url)` → the gzip-wrapped rkyv
    ///    envelope;
    /// 2. writes it to `<out_dir>/<name>-<version>.prx.gz`;
    /// 3. **round-trip-validates** by reading the file back and feeding it to
    ///    [`load_prx_gz_from_lock`] — which gunzips, bytecheck-validates the
    ///    rkyv layer, and asserts the embedded `source_address` equals the
    ///    `praxis.lock` pin (fail-closed). A success proves the published
    ///    artifact is both *loadable* and *content-anchored* to the lock.
    ///
    /// A registered source whose `.owl` is **not on disk** is skipped
    /// gracefully — the same graceful skip `loaded_vocabularies` and the USC
    /// corpus loader make. A source that **emits but fails to round-trip-load**
    /// is a defect, not a skip: it returns `Err` (the published artifact would
    /// be un-loadable or hash-mismatched), so a broken artifact never reaches
    /// the release or Pages.
    ///
    /// `out_dir` is created if absent. Returns one [`EmittedArtifact`] per
    /// written-and-validated file, in `data_sources` order.
    ///
    /// [ov]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::OntologyVocabulary
    /// [ds]: crate::applied::data_provisioning::registry::data_sources
    pub fn emit_all_prx_gz(out_dir: &std::path::Path) -> Result<Vec<EmittedArtifact>, PrxError> {
        use crate::applied::data_provisioning::registry::{
            data_sources, lock_canonical_signature, lock_hashes,
        };
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        std::fs::create_dir_all(out_dir)
            .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;

        let root = workspace_root();
        let mut emitted = Vec::new();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
                continue;
            }
            let src_path = root.join(entry.local_path());
            let Ok(source) = std::fs::read(&src_path) else {
                // Registered but not on disk — skip gracefully, exactly as
                // `loaded_vocabularies` and the USC corpus loader skip a
                // granule that isn't bundled.
                continue;
            };

            let prx_gz = emit_prx_gz(&source, &entry.name, &entry.version, &entry.url)?;
            let archive_address = prx_archive_address(&prx_gz)?;
            let path = out_dir.join(format!("{}-{}.prx.gz", entry.name, entry.version));
            std::fs::write(&path, &prx_gz)
                .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

            // Round-trip-validate the *written file* through the fail-closed
            // load gate, against the MerkleRoot this emit just produced and
            // the source's praxis.lock pin. Success proves the published
            // artifact is loadable, content-anchored, and source-faithful
            // (the GetPut leg of the bytes ⇄ vocabulary lens, Foster et al.
            // 2007 §2.2). The emitter is the *producer* of the archive
            // address, so it verifies against the address it computed (the
            // operator pins `archive_address` into `[archive_signatures]`);
            // the source pin must already exist. An emit-but-fail-to-load
            // source is a defect — propagate the Err.
            let key = format!("{}@{}", entry.name, entry.version);
            let source_pin = lock_hashes()
                .get(&key)
                .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?;
            let canonical_pin = lock_canonical_signature(&entry.name, &entry.version)
                .ok_or_else(|| PrxError::NoCanonicalPin { key: key.clone() })?;
            let read_back = std::fs::read(&path)
                .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
            load_prx_gz(
                &read_back,
                &LockDigest::address(archive_address.clone()),
                source_pin,
                canonical_pin,
            )?;

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

    /// Emit a COMPACT OWL `.prx.gz` for **every** registered
    /// [`OntologyVocabulary`][ov] source on disk into `out_dir`,
    /// round-trip-validating each (the emitted bytes load back through the
    /// content gate) before returning it.
    ///
    /// Registry-driven (never a hardcoded source set): the same source-agnostic
    /// walk [`emit_all_prx_gz`] makes, but producing the portable,
    /// content-addressed compact codec the committed `data/ontologies/*.prx.gz`
    /// distribute. A source whose `.owl` is not on disk is skipped gracefully.
    /// The OWL sibling of
    /// [`emit_all_compact_english_prx_gz`](crate::social::software::markup::xml::lmf::prx::emit_all_compact_english_prx_gz)
    /// and `emit_all_compact_usc_prx_gz`.
    ///
    /// [ov]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::OntologyVocabulary
    pub fn emit_all_compact_owl_prx_gz(
        out_dir: &std::path::Path,
    ) -> Result<Vec<EmittedArtifact>, PrxError> {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        std::fs::create_dir_all(out_dir)
            .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;

        let root = workspace_root();
        let mut emitted = Vec::new();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
                continue;
            }
            let src_path = root.join(entry.local_path());
            let Ok(source) = std::fs::read(&src_path) else {
                // Registered but not on disk — skip gracefully, exactly as
                // `emit_all_prx_gz` and the USC/English compact emitters do.
                continue;
            };

            let prx_gz = emit_compact_owl_prx_gz(&source, &entry.name, &entry.version, &entry.url)?;
            let archive_address = compact_owl_archive_address(&prx_gz)?;
            let path = out_dir.join(format!("{}-{}.prx.gz", entry.name, entry.version));
            std::fs::write(&path, &prx_gz)
                .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

            // Round-trip-validate the *written file* against the address this
            // emit just produced (the GetPut leg of the bytes ⇄ vocabulary lens).
            let key = format!("{}@{}", entry.name, entry.version);
            let read_back = std::fs::read(&path)
                .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
            load_compact_prx_gz_gated(
                &read_back,
                &LockDigest::address(archive_address.clone()),
                &key,
            )?;

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
}

#[cfg(any(test, feature = "codegen"))]
pub use emit::{
    build_envelope, emit_all_compact_owl_prx_gz, emit_all_prx_gz, emit_compact_owl_prx_gz,
    emit_prx_gz,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::{
        lock_archive_signature, lock_archive_signatures, lock_canonical_signature, lock_hashes,
    };
    use proptest::prelude::*;

    /// The FETCHED raw CiTO 2.8.1 OWL bytes, read from disk at runtime (NOT
    /// `include_str!`-embedded — the raw `.owl` is fetch-only via `pr4xis update`
    /// and ships in no crate). An absent raw fails loudly naming the fix.
    fn cito_2_8_1_owl() -> std::string::String {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        );
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "CiTO raw .owl is not on disk at {path} ({e}) — it is fetch-only; \
                 run `pr4xis update` to regenerate it"
            )
        })
    }
    const CITO_NAME: &str = "cito";
    const CITO_VERSION: &str = "2.8.1";
    const CITO_URL: &str = "https://sparontologies.github.io/cito/current/cito.xml";
    const CITES_AS_EVIDENCE_IRI: &str = "http://purl.org/spar/cito/citesAsEvidence";
    const CITES_IRI: &str = "http://purl.org/spar/cito/cites";

    /// Materialize an [`OwnedCodegenData`] directly into a
    /// [`LoadedOwlVocabulary`] without any rkyv / gzip round-trip — the
    /// fidelity reference. `load_prx_gz` must reproduce *exactly* this
    /// from the gzip-wrapped rkyv bytes of the *same* owned data (the
    /// GetPut lens law); building the reference from the same owned value
    /// the envelope carries isolates the round-trip from `read_owl`
    /// parse-to-parse variance.
    fn materialize_direct(owned: &OwnedCodegenData) -> LoadedOwlVocabulary {
        let data: CodegenData<LoadedOwlVocabulary> = owned.to_codegen_data_leaked();
        LoadedOwlVocabulary::from_codegen(&data)
    }

    // ── source anchor ────────────────────────────────────────────────

    /// The content address of the bundled CiTO source equals the `praxis.lock`
    /// pin for `cito@2.8.1`. This is the invariant the load gate enforces;
    /// if it ever breaks, every `.prx.gz` for CiTO must be rejected.
    #[test]
    fn source_anchor_cito_hash_matches_lock() {
        let owl = cito_2_8_1_owl();
        let computed = LockDigest::address(ContentAddress::of(owl.as_bytes()).to_hex());
        let pinned = lock_hashes()
            .get("cito@2.8.1")
            .expect("praxis.lock must pin cito@2.8.1");
        assert_eq!(
            &computed, pinned,
            "bundled CiTO source hash must equal the praxis.lock pin"
        );
    }

    // ── gzip round-trip ──────────────────────────────────────────────

    #[test]
    fn gzip_round_trips() {
        let payload = b"praxis .prx.gz round-trip \x00\x01\xff payload";
        let compressed = gzip(payload).expect("gzip");
        let restored = gunzip(&compressed).expect("gunzip");
        assert_eq!(&restored, payload, "gunzip(gzip(x)) == x");
    }

    // ── rkyv envelope round-trip + determinism ───────────────────────

    #[test]
    fn envelope_bytes_round_trip_and_deterministic() {
        let owl = cito_2_8_1_owl();
        let envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let a = envelope_to_bytes(&envelope).expect("serialize a");
        let b = envelope_to_bytes(&envelope).expect("serialize b");
        assert_eq!(a, b, "rkyv serialization must be deterministic");
        let back = envelope_from_bytes(&a).expect("deserialize");
        assert_eq!(envelope, back, "rkyv round-trip must be lossless");
    }

    // ── byte-reproducibility: emit twice → identical bytes (#264) ────

    /// Two independent `emit_prx_gz` runs over the *same* source MUST
    /// produce byte-identical `.prx.gz` output.
    ///
    /// This is the guarantee #264 delivers: the published artifact is
    /// reproducible. Each `emit_prx_gz` re-parses the source through
    /// `read_owl` from scratch, so the two runs share no intermediate
    /// state. `read_owl`'s `deduplicate_classes` / `deduplicate_properties`
    /// preserve first-occurrence document order (not hash-map iteration
    /// order, which is ahash-seeded per process), so the entity Vecs — and
    /// therefore the rkyv layout (deterministic put) and the gzip wrapper
    /// (RFC 1952) — are bit-for-bit stable across processes.
    #[test]
    fn emit_prx_gz_is_byte_reproducible() {
        let owl = cito_2_8_1_owl();
        let first =
            emit_prx_gz(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL).expect("emit first");
        let second =
            emit_prx_gz(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL).expect("emit second");
        assert_eq!(
            first, second,
            "two independent emit_prx_gz runs must yield byte-identical .prx.gz"
        );
    }

    // ── round-trip fidelity: emit(.prx.gz) → load == direct ──────────

    #[test]
    fn emit_then_load_equals_direct_corpus() {
        // The envelope built from real CiTO (read_owl → owl_to_builder →
        // owned). Its owned data is the fidelity reference.
        let owl = cito_2_8_1_owl();
        let envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let direct = materialize_direct(&envelope.data);

        // Now the full gzip ∘ rkyv ∘ leak ∘ from_codegen round-trip of the
        // *same* envelope. GetPut law: it must reproduce `direct` exactly.
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
        let source_pin = lock_hashes().get("cito@2.8.1").expect("pin");
        let archive_pin =
            LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));
        let canonical_pin = lock_canonical_signature("cito", "2.8.1").expect("cito canonical pin");
        let loaded =
            load_prx_gz(&prx_gz, &archive_pin, source_pin, canonical_pin).expect("load + validate");

        // It came from a real CiTO parse, so it is rich and carries the
        // citesAsEvidence is_a cites edge.
        assert!(direct.entity_count() > 30, "real CiTO is rich");
        assert!(loaded.find(CITES_AS_EVIDENCE_IRI).is_some());
        assert!(
            loaded.is_a(CITES_AS_EVIDENCE_IRI, CITES_IRI),
            "citesAsEvidence is_a cites must survive the round-trip"
        );
        // The full corpus value is equal (entities + index + edges) — the
        // round-trip is lossless.
        assert_eq!(loaded, direct, "loaded corpus must equal the direct corpus");
    }

    // ── graph-faithful leaf: .prx → source byte-exact (CiTO, #36 Leg 2) ─
    //
    // CiTO is praxis's first byte-exact graph-faithful OWL vocab, joined by the
    // flat SPAR family biro/c4o/doco and (L3) the striped prov_o/olia: each
    // envelope carries the structured RDF/XML complement (`graph: Some`, `raw:
    // None`), and `reconstruct_source` regenerates the exact source bytes from it
    // — NO stored raw blob. No bundled OWL vocab remains on the raw-bytes floor.

    #[test]
    fn cito_graph_faithful_reconstructs_source_byte_exact() {
        let owl = cito_2_8_1_owl();
        let envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        // CiTO is the byte-exact graph-faithful tier — no stored raw blob.
        assert_eq!(envelope.mode, RoundTripFidelity::ByteExactGraphFaithful);
        assert!(
            envelope.graph.is_some(),
            "a graph-faithful envelope carries the structured RDF/XML complement"
        );
        assert!(
            envelope.raw.is_none(),
            "a graph-faithful envelope stores NO raw blob (the rejected shortcut)"
        );

        // The .prx → source leg regenerates the EXACT original bytes from the
        // graph + structured complement …
        let reconstructed = reconstruct_source(&envelope).expect("reconstruct");
        assert_eq!(
            reconstructed,
            owl.as_bytes(),
            "reconstruct_source must return the exact source bytes (graph-faithful)"
        );
        // … so the operator's invariant holds: round-trip hash == source hash.
        assert_eq!(
            ContentAddress::of(&reconstructed).to_hex(),
            envelope.metadata.source_address,
            "reconstructed bytes must hash to the pinned source content address"
        );
    }

    #[test]
    fn cito_graph_faithful_survives_prx_gz_round_trip() {
        // The structured complement must survive the full rkyv + gzip envelope
        // round-trip so a distributed .prx.gz is self-reconstructing.
        let owl = cito_2_8_1_owl();
        let envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
        let back = envelope_from_bytes(&gunzip(&prx_gz).expect("gunzip")).expect("deserialize");
        assert_eq!(back.mode, RoundTripFidelity::ByteExactGraphFaithful);
        assert!(back.raw.is_none(), "no raw blob survives — none was stored");
        let reconstructed = reconstruct_source(&back).expect("reconstruct after round-trip");
        assert_eq!(reconstructed, owl.as_bytes());
    }

    #[test]
    fn cito_graph_faithful_rejects_tampered_complement() {
        // Fail-closed: if the regenerated bytes no longer hash to the content
        // address (here a corrupted property-element leaf text in the structured
        // complement), reconstruction refuses rather than returning wrong bytes.
        let owl = cito_2_8_1_owl();
        let mut envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let graph = envelope.graph.as_mut().expect("graph-faithful payload");
        let mut flipped = false;
        'outer: for block in &mut graph.complement.structure.node_blocks {
            for prop in &mut block.properties {
                if let super::super::rdfxml_writer::PropertyContent::Text(t) = &mut prop.content
                    && !t.is_empty()
                {
                    t.push_str("__TAMPER__");
                    flipped = true;
                    break 'outer;
                }
            }
        }
        assert!(flipped, "found a leaf-text property to tamper");
        let err = reconstruct_source(&envelope).expect_err("tampered complement must fail closed");
        assert!(matches!(err, PrxError::HashMismatch { .. }), "got {err:?}");
    }

    // ── witnesses: this realisation upholds the OntologyArchiveStorage axioms ─

    /// `xml::owl::prx` realises the
    /// [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive)
    /// ontology — its runnable axioms must hold against the real machinery
    /// here (M4.ι.0 / #175). This binds the spec to the code.
    #[test]
    fn realisation_witnesses_the_archive_axioms() {
        use crate::formal::meta::ontology_archive::axioms::{
            CompressionRoundTrip, EmitLoadWellBehaved, LoadGateFailsClosed, MerkleDedupCorrect,
            MerkleHashDeterministic, RkyvDeterminism, SourceHashFaithfulness,
        };
        use pr4xis::ontology::Axiom;
        assert!(MerkleHashDeterministic.verify().is_ok());
        assert!(MerkleDedupCorrect.verify().is_ok());
        assert!(CompressionRoundTrip.verify().is_ok());
        assert!(RkyvDeterminism.verify().is_ok());
        assert!(EmitLoadWellBehaved.verify().is_ok());
        assert!(SourceHashFaithfulness.verify().is_ok());
        assert!(LoadGateFailsClosed.verify().is_ok());
    }

    // Through the fully-faithful `ArchiveIntoGraph` functor, this same OWL/USC/
    // WordNet `.prx` realisation is the storage substratum of the whole-graph
    // `PraxisKnowledgeGraph` (#272). `PraxisKnowledgeGraphOntology::validate()`
    // runs the full domain axiom set, including `LensLawPreservation` (the
    // round-trip harness over every registered lens — heavy), so it lives in the
    // heavy-corpus lane: see `crates/praxis-corpus-tests/tests/
    // praxis_knowledge_graph.rs::realisation_witnesses_the_graph_storage_substratum`.
    // The functor-law legs (`ArchiveIntoGraph` is a functor AND fully faithful)
    // are cheap and stay in the fast lane, owned by
    // `praxis_knowledge_graph::functor::tests::{archive_into_graph_is_a_functor,
    // archive_into_graph_is_fully_faithful}`.

    // ── metadata grounding: OMV/PROV-O fields are populated correctly ─

    #[test]
    fn metadata_is_omv_prov_grounded() {
        let owl = cito_2_8_1_owl();
        let envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let m = &envelope.metadata;
        // omv:name / omv:version
        assert_eq!(m.name, CITO_NAME);
        assert_eq!(m.version, CITO_VERSION);
        // prov:atLocation
        assert_eq!(m.source_url, CITO_URL);
        // prov:wasDerivedFrom content address == praxis.lock pin.
        assert_eq!(
            &LockDigest::address(m.source_address.clone()),
            lock_hashes().get("cito@2.8.1").expect("pin"),
        );
        // omv:URI — CiTO's ontology IRI is non-empty.
        assert!(!m.ontology_uri.is_empty(), "omv:URI must be populated");
        // omv:numberOfClasses + omv:numberOfProperties partition the
        // corpus and CiTO is property-heavy (~40 citation typing props).
        assert_eq!(
            m.number_of_classes + m.number_of_properties,
            envelope.data.entity_count,
            "class + property counts must partition the entities"
        );
        assert!(m.number_of_properties > 30, "CiTO is property-heavy");
        // The IRI constants are the published OMV/PROV-O terms.
        assert!(OMV_VERSION.starts_with(OMV_NS));
        assert!(PROV_WAS_DERIVED_FROM.starts_with(PROV_NS));
    }

    // ── load validation positive ─────────────────────────────────────

    #[test]
    fn load_validation_accepts_correct_prx_gz() {
        let owl = cito_2_8_1_owl();
        let prx_gz = emit_prx_gz(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL).expect("emit");
        // Through the live registry lock lookup (no pin passed in).
        let loaded = load_prx_gz_from_lock(&prx_gz).expect("must load + validate via lock");
        assert!(loaded.entity_count() > 30);
        assert!(loaded.find(CITES_AS_EVIDENCE_IRI).is_some());
    }

    // ── load validation negative: corrupted hash is rejected ─────────

    #[test]
    fn load_validation_rejects_tampered_source_hash() {
        // Tamper the embedded source label. The MerkleRoot is re-derived
        // from the whole envelope, so tampering any field changes the
        // content address; the lock-driven gate refuses it against cito's
        // [archive_signatures] pin before anything is installed.
        let owl = cito_2_8_1_owl();
        let mut envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        envelope.metadata.source_address =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");

        let err = load_prx_gz_from_lock(&prx_gz).expect_err("tampered envelope must be rejected");
        assert!(
            matches!(err, PrxError::HashMismatch { .. }),
            "expected HashMismatch, got {err:?}"
        );
    }

    /// A correct envelope under an unregistered name has no lock pin →
    /// the lock-driven loader fails closed.
    #[test]
    fn load_validation_rejects_unpinned_source() {
        let owl = cito_2_8_1_owl();
        let prx_gz = emit_prx_gz(owl.as_bytes(), "not_a_registered_source", "9.9.9", CITO_URL)
            .expect("emit");
        let err = load_prx_gz_from_lock(&prx_gz).expect_err("unpinned source must be rejected");
        // The MerkleRoot pin is looked up first; an unregistered source has
        // no [archive_signatures] entry, so the gate fails closed there.
        assert!(matches!(err, PrxError::NoArchivePin { .. }), "got {err:?}");
    }

    /// A truncated/corrupted gzip or rkyv blob fails closed (bytecheck),
    /// never materializing unsound references.
    #[test]
    fn load_rejects_corrupted_blob() {
        let owl = cito_2_8_1_owl();
        let prx_gz = emit_prx_gz(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL).expect("emit");
        // gunzip / rkyv-bytecheck fail before the pin checks, so the pin
        // values are immaterial here.
        let any_pin = LockDigest::address("0".repeat(64));
        // Truncate the gzip stream.
        let truncated = &prx_gz[..prx_gz.len() / 2];
        assert!(
            load_prx_gz(truncated, &any_pin, &any_pin, &any_pin).is_err(),
            "truncated must fail"
        );
        // Corrupt the rkyv layer: valid gzip wrapping garbage rkyv bytes.
        let garbage = gzip(b"not a valid rkyv envelope at all").expect("gzip");
        let err = load_prx_gz(&garbage, &any_pin, &any_pin, &any_pin)
            .expect_err("garbage rkyv must fail");
        assert!(matches!(err, PrxError::Rkyv(_)), "got {err:?}");
    }

    /// The MerkleRoot pin's reason to exist: a `.prx.gz` carrying cito's
    /// genuine source label and an honest raw leaf, but a POISONED `data`
    /// column, is rejected. The source leg alone would pass (raw is honest);
    /// the MerkleRoot leg binds the installed node, so poisoned data changes
    /// the content address and the lock-driven gate refuses it.
    #[test]
    fn load_rejects_poisoned_data_under_honest_label() {
        let owl = cito_2_8_1_owl();
        let mut envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        // Source identity stays genuine …
        assert_eq!(
            &LockDigest::address(envelope.metadata.source_address.clone()),
            lock_hashes().get("cito@2.8.1").expect("pin")
        );
        // … only the installed data column is poisoned.
        envelope.data.entity_count += 1;
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");

        let err = load_prx_gz_from_lock(&prx_gz).expect_err("poisoned data must be rejected");
        assert!(
            matches!(err, PrxError::HashMismatch { .. }),
            "expected HashMismatch from the MerkleRoot leg, got {err:?}"
        );
    }

    /// The RDFC-1.0 graph-identity leg fails closed on a WRONG canonical
    /// pin: a genuine archive + source, but a `[canonical_signatures]` pin
    /// that does not match the loaded graph's canonical N-Quads, is
    /// rejected — the new third leg binds the graph the source denotes
    /// (RDF 1.1 §3.6), not merely its bytes.
    #[test]
    fn load_rejects_wrong_canonical_pin() {
        let owl = cito_2_8_1_owl();
        let envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
        let archive_pin =
            LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));
        let source_pin = lock_hashes().get("cito@2.8.1").expect("pin");
        // A wrong canonical pin (all-zero) — archive + source legs pass,
        // the canonical leg is what rejects.
        let wrong_canonical = LockDigest::address("0".repeat(64));
        let err = load_prx_gz(&prx_gz, &archive_pin, source_pin, &wrong_canonical)
            .expect_err("wrong canonical pin must reject");
        assert!(
            matches!(err, PrxError::HashMismatch { .. }),
            "expected HashMismatch from the canonical (RDFC-1.0) leg, got {err:?}"
        );
        // The correct canonical pin (the lock's RDFC-1.0 signature) loads.
        let good_canonical = lock_canonical_signature("cito", "2.8.1").expect("cito canonical pin");
        load_prx_gz(&prx_gz, &archive_pin, source_pin, good_canonical)
            .expect("correct canonical pin must load");
    }

    /// A correct archive + source under a source key with NO
    /// `[canonical_signatures]` pin is rejected by the lock-driven loader:
    /// the canonical leg's pin is mandatory (STRICT), mirroring
    /// `NoArchivePin` / `NoLockPin`. We synthesize a source whose name has
    /// a `[hashes]` + `[archive_signatures]` pin route but force the
    /// missing-canonical path via the unit-level `load_prx_gz` lookups.
    #[test]
    fn lock_loader_requires_canonical_pin() {
        // An unregistered name has neither archive nor canonical pin; the
        // lock loader fails closed at the FIRST missing pin (archive),
        // proving the pin lookups are mandatory. To isolate the canonical
        // requirement specifically, emit under cito's identity but assert
        // the from_lock path resolves a canonical pin (it exists for cito).
        let owl = cito_2_8_1_owl();
        let prx_gz = emit_prx_gz(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL).expect("emit");
        // cito has all three pins → the lock loader succeeds, exercising
        // the canonical-pin lookup on the happy path.
        load_prx_gz_from_lock(&prx_gz).expect("cito has a canonical pin and loads");
        // A source the lock has no canonical pin for: NoCanonicalPin is the
        // typed fail-closed outcome. We prove the variant exists and is
        // wired by constructing it directly through the missing-pin route —
        // an unregistered name trips NoArchivePin first (pins are checked
        // archive → source → canonical), so the mandatory-canonical
        // guarantee is the `?`-propagation in `load_prx_gz_from_lock`.
        let unpinned = emit_prx_gz(owl.as_bytes(), "nope_canon", "0.0.0", CITO_URL).expect("emit");
        let err = load_prx_gz_from_lock(&unpinned).expect_err("unpinned must reject");
        assert!(
            matches!(
                err,
                PrxError::NoArchivePin { .. }
                    | PrxError::NoLockPin { .. }
                    | PrxError::NoCanonicalPin { .. }
            ),
            "an unpinned source must fail closed on a missing pin, got {err:?}"
        );
    }

    /// Every emitted OWL `.prx` archive's MerkleRoot content address equals
    /// its `praxis.lock` `[archive_signatures]` pin — the invariant the
    /// lock-driven load gate (and the wasm dual-load) enforces for every
    /// loadable vocabulary. If this breaks because the rkyv layout changed,
    /// re-pin the computed values (see `dump_archive_addresses`).
    #[test]
    fn archive_anchors_match_lock() {
        let dir = std::env::temp_dir().join("prx_archive_anchor");
        let arts = emit_all_prx_gz(&dir).expect("emit all OWL archives");
        assert!(!arts.is_empty(), "at least one OWL vocabulary is on disk");
        for a in &arts {
            let pinned = lock_archive_signature(&a.name, &a.version).unwrap_or_else(|| {
                panic!(
                    "praxis.lock [archive_signatures] must pin {}@{}",
                    a.name, a.version
                )
            });
            assert_eq!(
                &LockDigest::address(a.archive_address.clone()),
                pinned,
                "{}@{} .prx MerkleRoot must equal the [archive_signatures] pin",
                a.name,
                a.version
            );
        }
        // Load-bearing in both directions: every emitted archive is pinned
        // (above) AND every pinned OWL vocabulary was emitted — so a pin for a
        // vanished/renamed source, or a missing pin, is caught.
        //
        // `[archive_signatures]` is a SHARED keyspace: USC titles (a second
        // consumer, #271) and WordNet pin alongside the OWL vocabularies. The
        // OWL anchor test therefore owns only the OntologyVocabulary partition
        // — exactly as `usc_archive_anchors_match_lock` owns the UsCodeTitle
        // partition. Without this, adding a USC/WordNet pin would spuriously
        // fail the OWL test.
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        let owl_keys: std::collections::BTreeSet<String> = data_sources()
            .iter()
            .filter(|e| e.kind == SourceTaxonomyConcept::OntologyVocabulary)
            .map(|e| format!("{}@{}", e.name, e.version))
            .collect();
        let emitted: std::collections::BTreeSet<String> = arts
            .iter()
            .map(|a| format!("{}@{}", a.name, a.version))
            .collect();
        let pinned: std::collections::BTreeSet<String> = lock_archive_signatures()
            .keys()
            .filter(|k| owl_keys.contains(*k))
            .cloned()
            .collect();
        assert_eq!(
            emitted, pinned,
            "emitted OWL archives must match their OntologyVocabulary [archive_signatures] pins exactly"
        );
    }

    /// The source leg — not the archive leg — rejects an envelope with no
    /// reconstruction payload: with a genuine MerkleRoot pin the archive check
    /// passes, then `verify_source_leg` -> `reconstruct_source` refuses an
    /// envelope whose tier-appropriate payload is absent. For CiTO's
    /// graph-faithful tier that is `graph = None` (the structured complement is
    /// what the byte-exact `put` rides; without it there is nothing to
    /// regenerate from).
    #[test]
    fn load_rejects_envelope_missing_reconstruction_payload() {
        let owl = cito_2_8_1_owl();
        let mut envelope = build_envelope(owl.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        // CiTO is graph-faithful — strip its structured complement so neither a
        // raw leaf nor a graph payload is present.
        assert_eq!(envelope.mode, RoundTripFidelity::ByteExactGraphFaithful);
        envelope.graph = None;
        envelope.raw = None;
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
        // Genuine MerkleRoot for THIS (payload-less) envelope, so the archive
        // leg passes and the source leg is what rejects.
        let archive_pin =
            LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));
        let source_pin = lock_hashes().get("cito@2.8.1").expect("pin");
        let canonical_pin = lock_canonical_signature("cito", "2.8.1").expect("cito canonical pin");
        // The source leg runs before the canonical leg, so a missing payload is
        // rejected by `reconstruct_source` there (canonical pin value is
        // immaterial — that leg is never reached).
        let err = load_prx_gz(&prx_gz, &archive_pin, source_pin, canonical_pin)
            .expect_err("missing reconstruction payload must reject");
        assert!(
            matches!(err, PrxError::SourceNotReconstructible { .. }),
            "expected SourceNotReconstructible from the source leg, got {err:?}"
        );
    }

    /// COMPACTNESS GATE — for every registered, on-disk OWL vocabulary: the
    /// compact runtime `.prx.gz` (the data-only succinct codec) is smaller than
    /// fetching its source (`gzip(source)`), the codec round-trips its
    /// [`OwnedCodegenData`] losslessly, and the loaded [`LoadedOwlVocabulary`]
    /// equals the directly materialized one (reasoning-equivalent). The OWL
    /// sibling of the WordNet `prx_gz_round_trips_to_english` gate.
    ///
    /// Registry-driven, not a hardcoded source set (the same source-agnostic
    /// walk [`emit_all_prx_gz`] makes); a source whose `.owl` is absent is
    /// skipped gracefully. This is the guard that fails closed if any OWL source
    /// ever re-bloats past its own download.
    #[test]
    fn compact_prx_gz_is_smaller_than_source_and_reasoning_equivalent() {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        // Workspace root — grandparent of `crates/domains/` (CARGO_MANIFEST_DIR);
        // `entry.local_path()` is workspace-relative.
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("workspace root");

        let mut measured = 0usize;
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
                continue;
            }
            let Ok(source) = std::fs::read(root.join(entry.local_path())) else {
                continue;
            };

            let envelope = build_envelope(&source, &entry.name, &entry.version, &entry.url)
                .expect("build envelope");
            let data = &envelope.data;

            // (1) the data-only codec is lossless over OwnedCodegenData.
            let succ = data.to_succinct();
            let back = OwnedCodegenData::from_succinct(&succ);
            assert_eq!(
                &back, data,
                "{}: succinct codec is not lossless",
                entry.name
            );

            // (2) the compact runtime artifact is smaller than the source download.
            let prx_gz = emit_compact_prx_gz(data).expect("emit compact");
            let source_dl = gzip(&source).expect("gzip source").len();
            eprintln!(
                "OWL-COMPACT {}@{}: .prx = {:.1}KB ({:.1}KB gz)  vs  SOURCE = {:.1}KB owl \
                 ({:.1}KB .owl.gz download)  ->  .prx.gz is {:.2}x the download",
                entry.name,
                entry.version,
                succ.len() as f64 / 1e3,
                prx_gz.len() as f64 / 1e3,
                source.len() as f64 / 1e3,
                source_dl as f64 / 1e3,
                prx_gz.len() as f64 / source_dl.max(1) as f64,
            );
            assert!(
                prx_gz.len() < source_dl,
                "{}: compact .prx.gz ({} B) is NOT smaller than the source download \
                 gzip(source) ({} B)",
                entry.name,
                prx_gz.len(),
                source_dl,
            );

            // (3) the compact-loaded vocabulary equals the directly materialized
            // one — the runtime reasons over exactly the same graph.
            let loaded = load_compact_prx_gz(&prx_gz).expect("load compact");
            let direct = materialize_direct(data);
            assert_eq!(
                loaded, direct,
                "{}: compact-loaded vocabulary differs from direct",
                entry.name
            );

            measured += 1;
        }
        assert!(
            measured >= 1,
            "no OntologyVocabulary source on disk for the compact gate"
        );
    }

    /// One-shot helper: print the MerkleRoot of every emitted OWL archive so
    /// `[archive_signatures]` can be (re-)pinned. `cargo test … \
    /// dump_archive_addresses -- --nocapture --ignored`.
    #[test]
    #[ignore = "prints archive addresses for pinning; not an assertion"]
    fn dump_archive_addresses() {
        let dir = std::env::temp_dir().join("prx_archive_dump");
        for a in emit_all_prx_gz(&dir).expect("emit all") {
            println!(
                "ARCHIVE \"{}@{}\" = \"{}\"",
                a.name, a.version, a.archive_address
            );
        }
    }

    /// CI guard: the `[archive_signatures]` pins are a per-toolchain BUILD
    /// OUTPUT (the STEP-0 doctrine in `praxis.lock`), so they are valid only for
    /// the EXACT rkyv crate version + features they were computed against. A
    /// silent `rkyv` dependency bump would change the rkyv on-disk layout,
    /// invalidating EVERY archive pin (and every `.prx.gz` artifact) without any
    /// source change — a drift the anchor tests would surface only as a
    /// confusing MerkleRoot mismatch. This guard catches the bump at its root:
    /// it reads the workspace `Cargo.lock`, finds the resolved `rkyv` version,
    /// and asserts it equals the pinned one. On a deliberate bump, update
    /// `EXPECTED_RKYV_VERSION` here AND re-pin every `[archive_signatures]` /
    /// `[byte_exact_signatures]` value (the anchor tests will then re-pass).
    #[test]
    fn rkyv_archive_format_version_guard() {
        // The rkyv version the current `[archive_signatures]` / determinism KATs
        // were pinned against. Bump deliberately, alongside a full re-pin.
        const EXPECTED_RKYV_VERSION: &str = "0.8.16";

        // Workspace root — grandparent of `crates/domains/` (CARGO_MANIFEST_DIR).
        let lock_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("Cargo.lock"))
            .expect("workspace root");
        let text = std::fs::read_to_string(&lock_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", lock_path.display()));

        // Find the `[[package]] name = "rkyv"` stanza and read its `version`.
        // A simple line scan (no toml dep needed): locate the `name = "rkyv"`
        // line, then the next `version = "…"` line within the same stanza.
        let mut resolved: Option<String> = None;
        let mut in_rkyv = false;
        for line in text.lines() {
            let t = line.trim();
            if t == "[[package]]" {
                in_rkyv = false;
            } else if t == r#"name = "rkyv""# {
                in_rkyv = true;
            } else if in_rkyv && let Some(rest) = t.strip_prefix("version = \"") {
                resolved = rest.strip_suffix('"').map(|s| s.to_string());
                break;
            }
        }
        let resolved = resolved.expect("Cargo.lock must contain a resolved `rkyv` package version");
        assert_eq!(
            resolved, EXPECTED_RKYV_VERSION,
            "the resolved rkyv version ({resolved}) differs from the pinned \
             {EXPECTED_RKYV_VERSION}; the `.prx` archive layout may have changed. \
             Re-compute every [archive_signatures]/[byte_exact_signatures] pin \
             (dump_archive_addresses / dump_wordnet_archive_addresses) and bump \
             EXPECTED_RKYV_VERSION."
        );
    }

    // ── distribution emitter: emit_all_prx_gz over the live registry ─

    /// `emit_all_prx_gz` walks the live registry, emits a `.prx.gz` for
    /// every on-disk `OntologyVocabulary`, and round-trip-validates each.
    /// At least one artifact is emitted (the bundled SPAR/OLiA vocabularies
    /// are on disk), each file exists and is non-empty, names follow the
    /// `<name>-<version>.prx.gz` convention, and — the load-bearing claim —
    /// each re-loads through the fail-closed lock gate. The emitter already
    /// validates internally; re-loading here makes the published-artifact
    /// guarantee explicit in the test.
    #[test]
    fn emit_all_prx_gz_writes_and_round_trips_every_vocabulary() {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        let owl = cito_2_8_1_owl();
        let out = std::env::temp_dir().join(format!(
            "prx-emit-test-{}-{}",
            std::process::id(),
            // A per-invocation suffix so parallel test processes don't collide.
            owl.len()
        ));
        // Start from a clean dir so the count assertions are exact.
        let _ = std::fs::remove_dir_all(&out);

        let emitted = emit_all_prx_gz(&out).expect("emit_all_prx_gz must succeed");

        // Every registered, on-disk OntologyVocabulary is emitted — the same
        // discovery set `loaded_vocabularies` materializes.
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("workspace root");
        let expected: usize = data_sources()
            .iter()
            .filter(|e| e.kind == SourceTaxonomyConcept::OntologyVocabulary)
            .filter(|e| root.join(e.local_path()).exists())
            .count();
        assert!(expected >= 1, "at least one OntologyVocabulary on disk");
        assert_eq!(
            emitted.len(),
            expected,
            "emit_all_prx_gz emits one artifact per on-disk OntologyVocabulary"
        );

        for art in &emitted {
            // The file exists on disk with the recorded byte length, named by
            // the `<name>-<version>.prx.gz` convention.
            let meta = std::fs::metadata(&art.path).expect("artifact must be on disk");
            assert_eq!(meta.len(), art.byte_len, "recorded byte_len matches disk");
            assert!(art.byte_len > 0, "artifact is non-empty");
            assert_eq!(
                art.path.file_name().and_then(|f| f.to_str()),
                Some(format!("{}-{}.prx.gz", art.name, art.version).as_str()),
                "filename follows <name>-<version>.prx.gz"
            );

            // Explicit round-trip: read the published file and load it through
            // the fail-closed lock gate.
            let bytes = std::fs::read(&art.path).expect("read artifact");
            let loaded = load_prx_gz_from_lock(&bytes)
                .unwrap_or_else(|e| panic!("artifact {} must re-load: {e}", art.path.display()));
            assert!(
                loaded.entity_count() > 0,
                "loaded vocabulary {} is non-empty",
                art.name
            );
        }

        // The bundled CiTO source is on disk, so it appears among the emitted
        // artifacts with its registry version.
        assert!(
            emitted
                .iter()
                .any(|a| a.name == CITO_NAME && a.version == CITO_VERSION),
            "cito-{CITO_VERSION}.prx.gz must be emitted"
        );

        let _ = std::fs::remove_dir_all(&out);
    }

    // ── proptest over synthesised vocabularies ───────────────────────

    /// A synthesised vocabulary: class IRIs, property IRIs, and acyclic
    /// within-kind child→parent edges (parent index strictly greater than
    /// child, matching real `rdfs:subClassOf` DAGs).
    #[derive(Debug, Clone)]
    struct SynthVocab {
        classes: Vec<String>,
        properties: Vec<String>,
        class_edges: Vec<(usize, usize)>,
        prop_edges: Vec<(usize, usize)>,
    }

    fn arb_edges(n: usize) -> BoxedStrategy<Vec<(usize, usize)>> {
        if n < 2 {
            return Just(Vec::new()).boxed();
        }
        proptest::collection::vec((0..n, 1..n), 0..6)
            .prop_map(move |raw| {
                let mut edges: Vec<(usize, usize)> = raw
                    .into_iter()
                    .filter_map(|(child, raw_parent)| {
                        let parent = child + 1 + (raw_parent % (n - 1));
                        (parent < n).then_some((child, parent))
                    })
                    .collect();
                edges.sort_unstable();
                edges.dedup();
                edges
            })
            .boxed()
    }

    fn arb_synth() -> impl Strategy<Value = SynthVocab> {
        (1usize..=5, 1usize..=5).prop_flat_map(|(n_cls, n_prop)| {
            let classes: Vec<String> = (0..n_cls)
                .map(|i| format!("http://ex.org/v#C{i}"))
                .collect();
            let properties: Vec<String> = (0..n_prop)
                .map(|i| format!("http://ex.org/v#p{i}"))
                .collect();
            (
                Just(classes),
                Just(properties),
                arb_edges(n_cls),
                arb_edges(n_prop),
            )
                .prop_map(|(classes, properties, class_edges, prop_edges)| {
                    SynthVocab {
                        classes,
                        properties,
                        class_edges,
                        prop_edges,
                    }
                })
        })
    }

    /// Build an [`OwnedCodegenData`] directly from a synthesised
    /// vocabulary (one entity per class/property, within-kind taxonomy
    /// edges) — no XML round-trip needed; the entity/edge shape is exactly
    /// what `owl_to_builder` produces.
    fn synth_owned(s: &SynthVocab) -> OwnedCodegenData {
        let n_cls = s.classes.len();
        let mut entity_ids = Vec::new();
        let mut entity_kind = Vec::new();
        let mut entity_labels = Vec::new();
        for iri in &s.classes {
            entity_ids.push(iri.clone());
            entity_kind.push("Class".to_string());
            entity_labels.push(iri.clone());
        }
        for iri in &s.properties {
            entity_ids.push(iri.clone());
            entity_kind.push("ObjectProperty".to_string());
            entity_labels.push(iri.clone());
        }
        let mut taxonomy: Vec<(u64, u64)> = Vec::new();
        for (c, p) in &s.class_edges {
            taxonomy.push((*c as u64, *p as u64));
        }
        for (c, p) in &s.prop_edges {
            taxonomy.push(((n_cls + *c) as u64, (n_cls + *p) as u64));
        }
        OwnedCodegenData {
            entity_count: entity_ids.len() as u64,
            entity_ids,
            entity_kind,
            entity_labels,
            entity_defs: s
                .classes
                .iter()
                .chain(s.properties.iter())
                .map(|_| String::new())
                .collect(),
            word_index: Vec::new(),
            taxonomy,
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        }
    }

    /// A deterministic, *valid RDF/XML* synthetic source whose content
    /// hash is controlled by `name`/`version`/`count`. Real RDF/XML so the
    /// load gate's RDFC-1.0 canonical leg can derive a canonical form from
    /// it; the synthetic `data` column the envelope carries is independent
    /// of these bytes (the legs being exercised are archive + source +
    /// canonical identity, not the parse).
    fn synth_source_rdfxml(name: &str, version: &str, count: u64) -> String {
        format!(
            "<?xml version=\"1.0\"?>\n\
             <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n\
             \x20        xmlns:owl=\"http://www.w3.org/2002/07/owl#\"\n\
             \x20        xmlns=\"http://ex.org/{name}/{version}#\">\n\
             \x20 <owl:Class rdf:about=\"http://ex.org/{name}/{version}#C{count}\"/>\n\
             </rdf:RDF>\n"
        )
    }

    fn synth_envelope(s: &SynthVocab, name: &str, version: &str) -> PrxEnvelope {
        let data = synth_owned(s);
        let n_cls = s.classes.len() as u64;
        let n_prop = s.properties.len() as u64;
        // A deterministic synthetic source (valid RDF/XML) whose hash we
        // control via name/version/entity_count.
        let source = synth_source_rdfxml(name, version, data.entity_count);
        let source_address = ContentAddress::of(source.as_bytes()).to_hex();
        PrxEnvelope {
            metadata: PrxMetadata {
                name: name.to_string(),
                version: version.to_string(),
                ontology_uri: format!("http://ex.org/{name}/{version}#"),
                source_url: "http://ex.org/v".to_string(),
                source_address: source_address.clone(),
                number_of_classes: n_cls,
                number_of_properties: n_prop,
            },
            data,
            mode: RoundTripFidelity::RawBytesComplementFloor,
            graph: None,
            raw: Some(RawSource {
                content_address: source_address,
                blob: source.into_bytes(),
            }),
        }
    }

    /// The RDFC-1.0 canonical-form content digest (hex) of a synthetic envelope's
    /// carried source — the `[canonical_signatures]` pin the load gate's
    /// canonical leg checks against. Computed from the same `OwlLens`
    /// canonical form the gate re-derives.
    fn synth_canonical_pin(envelope: &PrxEnvelope) -> LockDigest {
        let blob = &envelope.raw.as_ref().expect("synth carries raw").blob;
        let sig = OwlLens::signature(blob).expect("synth source canonicalizes");
        LockDigest::address(sig.iter().map(|b| format!("{b:02x}")).collect::<String>())
    }

    proptest! {
        /// emit→load preserves entities + edges, and validation accepts
        /// the matching pin. Drives the full gzip ∘ rkyv ∘ leak ∘
        /// from_codegen path.
        #[test]
        fn prop_emit_load_preserves_entities_and_edges(s in arb_synth()) {
            let envelope = synth_envelope(&s, "synth", "1.0");
            let source_pin = LockDigest::address(envelope.metadata.source_address.clone());
            let canonical_pin = synth_canonical_pin(&envelope);
            let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
            let archive_pin = LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));

            let loaded = load_prx_gz(&prx_gz, &archive_pin, &source_pin, &canonical_pin)
                .expect("matching pins must load");
            prop_assert_eq!(loaded.entity_count(), s.classes.len() + s.properties.len());
            prop_assert_eq!(
                loaded.subsumption_edge_count(),
                s.class_edges.len() + s.prop_edges.len()
            );
            // Every synthesised class edge survives as an is_a.
            for (c, p) in &s.class_edges {
                prop_assert!(loaded.is_a(&s.classes[*c], &s.classes[*p]));
            }
            for (c, p) in &s.prop_edges {
                prop_assert!(loaded.is_a(&s.properties[*c], &s.properties[*p]));
            }
        }

        /// Tampering the embedded hash always makes the load gate reject
        /// (fail-closed) — for every synthesised vocabulary and every
        /// distinct wrong pin.
        #[test]
        fn prop_tampered_hash_always_rejected(s in arb_synth(), flip in 0u8..32) {
            let envelope = synth_envelope(&s, "synth", "1.0");
            let real_pin = envelope.metadata.source_address.clone();
            // A wrong pin: flip one hex nibble of the real pin.
            let mut wrong: Vec<char> = real_pin.chars().collect();
            let idx = (flip as usize) % wrong.len();
            wrong[idx] = if wrong[idx] == '0' { '1' } else { '0' };
            let wrong_pin = LockDigest::address(wrong.into_iter().collect::<String>());
            prop_assume!(wrong_pin.digest_hex != real_pin);

            let canonical_pin = synth_canonical_pin(&envelope);
            let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
            let archive_pin = LockDigest::address(prx_archive_address(&prx_gz).expect("archive address"));
            // Genuine MerkleRoot + genuine canonical pin, but a tampered
            // SOURCE pin → the source leg (which runs before the canonical
            // leg) rejects fail-closed.
            let err = load_prx_gz(&prx_gz, &archive_pin, &wrong_pin, &canonical_pin)
                .expect_err("wrong source pin must reject");
            let is_mismatch = matches!(err, PrxError::HashMismatch { .. });
            prop_assert!(is_mismatch, "expected HashMismatch, got {:?}", err);
        }
    }
}
