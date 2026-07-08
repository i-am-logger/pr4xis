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
//! address; the succinct codec is the COMPACT form whose guarantee is
//! information-near-minimal size at zero information loss. Neither codec's axioms
//! cover the other.
//!
//! # Literature
//!
//! - **Jacobson (1989)** *Space-efficient static trees and graphs*, FOCS '89 —
//!   the succinct-data-structure program: store a structure in space close to
//!   its information-theoretic minimum while keeping it usable without
//!   decompressing.
//! - **Elias (1974)** *Efficient storage and retrieval by content and address of
//!   static files*, JACM 21(2) §II, and **Fano (1971)** *On the number of bits
//!   required to implement an associative memory*, MIT Project MAC memo — the
//!   monotone-integer-sequence compression the `put_ef` column is named after,
//!   realized here as dependency-free gap coding.
//! - **Witten, Moffat & Bell (1999)** *Managing Gigabytes*, 2nd ed., §3.3
//!   (gap/delta coding) and §4.2 (front coding) — the string-dictionary and
//!   offset compressions.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** *Combinators for
//!   bidirectional tree transformations*, ACM TOPLAS 29(3) §2.2 — the get/put
//!   fidelity law the round-trip axiom instantiates.
//! - **Smith et al. (2005)** *Relations in biomedical ontologies* (OBO Relation
//!   Ontology), *Genome Biology* 6:R46 — the `part of` (RO:0000050) and
//!   `depends on` (RO:0002502) relations the codec's morphisms are kinded by.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SuccinctCodec",
    source: "Jacobson (1989) Space-efficient static trees and graphs, FOCS '89 (succinct data structures); Elias (1974) Efficient storage and retrieval by content and address of static files, JACM 21(2) §II; Fano (1971) On the number of bits required to implement an associative memory, MIT Project MAC memo; Witten, Moffat & Bell (1999) Managing Gigabytes, 2nd ed., §3.3, §4.2; Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2; Smith et al. (2005) Relations in biomedical ontologies (OBO Relation Ontology), Genome Biology 6:R46",

    concepts: [
        SuccinctEncoding,
        BitPackedColumn,
        MonotoneGapColumn,
        FrontCodedDictionary,
        SuccinctRoundTrip,
    ],

    labels: {
        SuccinctEncoding: ("en", "Succinct encoding",
            "Jacobson (1989): the compact bit-packed .prx byte encoding of the runtime reasoning view — a shared front-coded dictionary plus bit-packed index columns and gap-coded CSR offsets — in space close to the information-theoretic minimum, and usable (decodable on any target incl. wasm32) without an external index."),
        BitPackedColumn: ("en", "Bit-packed column",
            "A column of unsigned integers stored at exactly bits(max) bits per value (the minimal fixed width that represents the largest value), LSB-first into a byte stream. The kernel the other columns build on: gap-coded offsets and dictionary indices are all bit-packed columns."),
        MonotoneGapColumn: ("en", "Monotone gap column",
            "Elias (1974); Fano (1971): a monotone non-decreasing offset sequence (a CSR offset array) stored as its consecutive gaps, then bit-packed. Because per-node gaps are small even when the cumulative offsets span a large range, the gap column is far narrower than the absolute one — the compression Elias-Fano gives on offsets, realized dependency-free."),
        FrontCodedDictionary: ("en", "Front-coded dictionary",
            "Witten, Moffat & Bell (1999) §4.2: a string dictionary storing each entry as (shared-prefix-length, suffix) against the previous entry, so a prefix shared with the previous entry (heavy for sorted IRIs under a common namespace) is written once, not per entry. Lossless for any input order; sorting maximizes the elided prefix."),
        SuccinctRoundTrip: ("en", "Succinct round-trip",
            "Foster et al. (2007) get/put fidelity, here the total inverse law of the succinct codec: from_succinct(to_succinct(d)) == d, so the compact wire form loses nothing across the encode/decode boundary the runtime and the wasm/web demo load the corpus and registry through."),
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
                "compact bit-packed .prx bytes near the information-theoretic minimum (Jacobson 1989)"
            }
            C::BitPackedColumn => {
                "integers at bits(max) bits per value, LSB-first — the packing kernel"
            }
            C::MonotoneGapColumn => {
                "monotone offsets stored as gaps; narrow width despite a wide range (Elias 1974)"
            }
            C::FrontCodedDictionary => {
                "sorted string dict with shared prefixes written once (Witten-Moffat-Bell 1999)"
            }
            C::SuccinctRoundTrip => {
                "from_succinct(to_succinct(d)) == d — the compact codec loses nothing (Foster 2007)"
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
            FrontCodingSharesPrefixes, MonotoneOffsetsCompact, SuccinctCodecRoundTrip,
        };
        axioms.push(alloc::boxed::Box::new(SuccinctCodecRoundTrip));
        axioms.push(alloc::boxed::Box::new(MonotoneOffsetsCompact));
        axioms.push(alloc::boxed::Box::new(FrontCodingSharesPrefixes));
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_concepts() {
        assert_eq!(SuccinctCodecConcept::variants().len(), 5);
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
        ] {
            assert!(
                axiom_by_name(axiom).is_some(),
                "codec axiom {axiom} must re-bind through the registry (axiom_by_name)"
            );
        }
    }
}
