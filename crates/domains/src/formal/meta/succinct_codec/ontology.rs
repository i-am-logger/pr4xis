//! Succinct-codec ontology — the compact bit-packed `.prx` wire format the
//! `.cprx.gz` English corpus and the registry ship in: bit-packed columns,
//! gap-coded monotone offsets, and a front-coded string dictionary, with the
//! round-trip and compaction guarantees they rest on.
//!
//! This is the *first-class praxis ontology* the succinct codec realises. Rather
//! than describe the codec in doc-comments and prove it with one on-disk
//! integration test, praxis declares its concepts here and proves its guarantees
//! as runnable axioms (see [`super::axioms`], whose `verify()` predicates
//! exercise the real
//! [`markup::xml::succinct`](crate::social::software::markup::xml::succinct)
//! primitives and
//! [`OwnedCodegenData`](crate::social::software::markup::xml::owl::prx::OwnedCodegenData)).
//!
//! It is a DIFFERENT codec from [`super::super::canonical_codec`]: DAG-CBOR is
//! the self-describing interchange form whose guarantee is a stable content
//! address; this one is a lossless COMPRESSION of the corpus whose guarantee is
//! a smaller wire form at zero information loss. It is a compression codec (a
//! full owned `Vec` is materialized on every accessor), NOT a succinct
//! rank/select structure — there is no usable-without-decompressing claim.
//! Neither codec's axioms cover the other.
//!
//! # Literature
//!
//! - **Witten, Moffat & Bell (1999)** *Managing Gigabytes: Compressing and
//!   Indexing Documents and Images*, 2nd ed., §3.3 (gap/delta coding of
//!   monotone integer sequences) and §4.1 (Accessing the lexicon — front coding) — the offset and
//!   string-dictionary compressions, and the lossless round-trip
//!   (`from_succinct ∘ to_succinct = id`, a total inverse pair). The `put_ef`
//!   column is plain gap coding; its name is historical and does NOT denote an
//!   Elias-Fano structure (no upper/lower-bit split, no select).
//! - **Smith et al. (2005)** *Relations in biomedical ontologies* (OBO Relation
//!   Ontology), *Genome Biology* 6:R46 — the `part of` (RO:0000050) and
//!   `depends on` (RO:0002502) relations the codec's morphisms are kinded by.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SuccinctCodec",
    source: "Witten, Moffat & Bell (1999) Managing Gigabytes: Compressing and Indexing Documents and Images, 2nd ed., §3.3 (gap/delta coding of monotone integer sequences), §4.1 (Accessing the lexicon — front coding) — lossless compression + the total-inverse-pair round-trip; Deutsch (1996) RFC 1951 DEFLATE Compressed Data Format Specification 1.3 — the raw-source envelope's payload transport; Smith et al. (2005) Relations in biomedical ontologies (OBO Relation Ontology), Genome Biology 6:R46",

    concepts: [
        SuccinctEncoding,
        BitPackedColumn,
        MonotoneGapColumn,
        FrontCodedDictionary,
        SuccinctRoundTrip,
        RawSourceEnvelope,
        DeflatePayload,
    ],

    labels: {
        SuccinctEncoding: ("en", "Succinct encoding",
            "Witten, Moffat & Bell (1999): the compact bit-packed .prx byte encoding of the runtime reasoning view — a shared front-coded dictionary plus bit-packed index columns and gap-coded CSR offsets — a lossless COMPRESSION of the corpus, decodable on any target incl. wasm32 (each accessor materializes a full owned Vec; this is compression, not a rank/select succinct structure)."),
        BitPackedColumn: ("en", "Bit-packed column",
            "A column of unsigned integers stored at exactly bits(max) bits per value (the minimal fixed width that represents the largest value), LSB-first into a byte stream. The kernel the other columns build on: gap-coded offsets and dictionary indices are all bit-packed columns."),
        MonotoneGapColumn: ("en", "Monotone gap column",
            "Witten, Moffat & Bell (1999) §3.3: a monotone non-decreasing offset sequence (a CSR offset array) stored as its consecutive gaps, then bit-packed. Because per-node gaps are small even when the cumulative offsets span a large range, the gap column is far narrower than the absolute one — gap/delta coding of monotone integer sequences (the `put_ef` name is historical, not an Elias-Fano structure)."),
        FrontCodedDictionary: ("en", "Front-coded dictionary",
            "Witten, Moffat & Bell (1999) §4.1: a string dictionary storing each entry as (shared-prefix-length, suffix) against the previous entry, so a prefix shared with the previous entry (heavy for sorted IRIs under a common namespace) is written once, not per entry. Lossless for any input order; sorting maximizes the elided prefix."),
        SuccinctRoundTrip: ("en", "Succinct round-trip",
            "Witten, Moffat & Bell (1999) lossless coding, here the total inverse law of the compression codec: from_succinct(to_succinct(d)) == d (a serialization inverse pair, NOT a lens law), so the compact wire form loses nothing across the encode/decode boundary the runtime and the wasm/web demo load the corpus and registry through."),
        RawSourceEnvelope: ("en", "Raw-source envelope",
            "The versioned raw-source .prx wire form (raw_source_prx, format version 2): varint(format_version) blob(name) blob(version) varint(encoding) varint(decoded_len) blob(payload) — the Bancilhon & Spyratos (1981) raw-bytes complement floor carried under an ENUMERATED, self-described payload encoding, fail-closed on an unknown version or encoding tag and total over arbitrary bytes."),
        DeflatePayload: ("en", "DEFLATE payload",
            "Deutsch (1996) RFC 1951: the raw-source envelope's compressed payload transport — a raw DEFLATE stream (deliberately NOT the RFC 1952 gzip wrapper, whose MTIME/OS header fields would make the emitted bytes nondeterministic), STORE-IF-SMALLER (a stream that does not strictly shrink the source bytes is downgraded to the identity transport), inflated fail-closed to EXACTLY the declared decoded length and bomb-guarded by DEFLATE's ~1032:1 maximum expansion (Gailly & Adler, zlib Technical Details)."),
    },

    // Kinded morphisms (OBO-RO; Smith et al. 2005). Mereology: the succinct
    // encoding HAS-PART the bit-packed columns and the front-coded dictionary
    // (`part of`, RO:0000050). Dependency: the monotone gap column is realized
    // VIA a bit-packed column of gaps (`put_ef` calls `put_cv`), and the
    // round-trip law depends on the encoding it inverts (`depends on`,
    // RO:0002502). These are traversable, kinded edges — the graph a future
    // `reachable_subgraph(SuccinctEncoding)` walks — NOT laws (the laws are the
    // three runnable axioms in `super::axioms`).
    edges: [
        (SuccinctEncoding, BitPackedColumn, Parthood),
        (SuccinctEncoding, FrontCodedDictionary, Parthood),
        (MonotoneGapColumn, BitPackedColumn, Dependency),
        (SuccinctRoundTrip, SuccinctEncoding, Dependency),
        // The raw-source envelope HAS-PART its DEFLATE payload transport
        // (`part of`, RO:0000050) — the store-if-smaller arm of format v2.
        (RawSourceEnvelope, DeflatePayload, Parthood),
    ],
}

/// Quality: a short symbolic description of each succinct-codec concept,
/// matching the citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = SuccinctCodecConcept;
    type Value = &'static str;

    fn get(&self, c: &SuccinctCodecConcept) -> Option<&'static str> {
        use SuccinctCodecConcept as C;
        Some(match c {
            C::SuccinctEncoding => {
                "compact bit-packed .prx bytes — a lossless compression of the corpus (Witten-Moffat-Bell 1999)"
            }
            C::BitPackedColumn => {
                "integers at bits(max) bits per value, LSB-first — the packing kernel"
            }
            C::MonotoneGapColumn => {
                "monotone offsets stored as gaps; narrow width despite a wide range (Witten-Moffat-Bell 1999 §3.3)"
            }
            C::FrontCodedDictionary => {
                "sorted string dict with shared prefixes written once (Witten-Moffat-Bell 1999)"
            }
            C::SuccinctRoundTrip => {
                "from_succinct(to_succinct(d)) == d — a total inverse pair; the compact codec loses nothing (Witten-Moffat-Bell 1999)"
            }
            C::RawSourceEnvelope => {
                "the versioned raw-source .prx wire form: name/version key + enumerated payload encoding + declared decoded length (Bancilhon-Spyratos 1981 floor, fail-closed framing)"
            }
            C::DeflatePayload => {
                "raw RFC 1951 DEFLATE payload transport, store-if-smaller, length-declared and bomb-guarded (Deutsch 1996)"
            }
        })
    }
}

impl Ontology for SuccinctCodecOntology {
    type Cat = SuccinctCodecCategory;
    type Qual = ConceptDescription;

    fn axioms() -> alloc::vec::Vec<alloc::boxed::Box<dyn Axiom>> {
        // The succinct codec ([`markup::xml::succinct`] +
        // [`owl::prx::OwnedCodegenData`]) is compiled under `feature = "prx"`
        // (as is this whole module), so these predicates run against the real
        // machinery whenever the codec exists.
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        use super::axioms::{
            FrontCodingSharesPrefixes, MonotoneOffsetsCompact, RawSourceDeflateTransport,
            SuccinctCodecRoundTrip,
        };
        axioms.push(alloc::boxed::Box::new(SuccinctCodecRoundTrip));
        axioms.push(alloc::boxed::Box::new(MonotoneOffsetsCompact));
        axioms.push(alloc::boxed::Box::new(FrontCodingSharesPrefixes));
        axioms.push(alloc::boxed::Box::new(RawSourceDeflateTransport));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::ontology::registry::{axiom_by_name, describe_knowledge_base};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SuccinctCodecCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        // Runs the category laws AND all three succinct-codec axioms' `verify()`
        // against the real bit-packing kernel + `OwnedCodegenData` codec.
        SuccinctCodecOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    /// Every declared concept carries a non-empty, DISTINCT description — a
    /// structural bijection derived from the declaration itself, replacing the
    /// former hand-maintained `assert_eq!(variants().len(), N)` count that had
    /// to be re-edited on every concept addition (the exhaustive `match` in
    /// the `ConceptDescription` Quality already compile-enforces coverage;
    /// this pins non-emptiness and injectivity).
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_concept_carries_a_distinct_description() {
        let q = ConceptDescription;
        let mut seen = alloc::collections::BTreeSet::new();
        for c in SuccinctCodecConcept::variants() {
            let d = q.get(&c).expect("every declared concept is described");
            assert!(!d.is_empty(), "{c:?} has an empty description");
            assert!(
                seen.insert(d),
                "{c:?} repeats another concept's description"
            );
        }
        assert!(
            !seen.is_empty(),
            "the ontology declares at least one concept"
        );
    }

    /// The machinery is reasoned about through the SAME registry as any statute:
    /// the ontology is discoverable in the knowledge base and each of its three
    /// axioms re-binds by name through `axiom_by_name` (the load-time rebind
    /// gate). If the succinct codec were opaque runtime code, neither would
    /// resolve — this is what "self-describing" buys.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn discoverable_via_self_model() {
        assert!(
            describe_knowledge_base()
                .iter()
                .any(|v| v.name() == "SuccinctCodecOntology"),
            "SuccinctCodec must be discoverable in the ontology registry"
        );
        for axiom in [
            "SuccinctCodecRoundTrip",
            "MonotoneOffsetsCompact",
            "FrontCodingSharesPrefixes",
            "RawSourceDeflateTransport",
        ] {
            assert!(
                axiom_by_name(axiom).is_some(),
                "codec axiom {axiom} must re-bind through the registry (axiom_by_name)"
            );
        }
    }
}
