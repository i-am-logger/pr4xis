//! Binary archival of the codegen interchange shape.
//!
//! [`OwnedCodegenData`] is the owned, self-contained mirror of
//! [`crate::codegen_data::CodegenData`]. Where `CodegenData<P>` holds
//! `&'static` slices (it is the build-time→runtime *static* interchange,
//! the first Futamura projection), `OwnedCodegenData` owns its strings
//! and edge tables so it can be serialized to a content-addressed
//! `.rkyv` blob and reconstructed elsewhere.
//!
//! This is the binary delivery format for the *non-embedded* stagings.
//! A registered source is parsed from its authoritative format (USLM
//! XML, WordNet LMF, …) exactly once — at build time — projected into an
//! `OwnedCodegenData`, and frozen as a `.rkyv` archive. At runtime the
//! archive is materialized with no re-parse of the original format: the
//! `Async` staging fetches the blob over the network, the `Mmap`
//! staging memory-maps it, the `Embedded` staging `include_bytes!`-bakes
//! it into the binary. Staging is *where the archive bytes come from*;
//! the materialization functor is the same regardless.
//!
//! The archive is a [`crate::ontology`]-agnostic lens: bytes ⇄
//! `OwnedCodegenData`, with `to_archive_bytes` the *put* and
//! `from_archive_bytes` the *get*. The PutGet / GetPut round-trip laws
//! (Foster et al. 2007, "Combinators for Bidirectional Tree
//! Transformations", ACM TOPLAS 29(3) §2.2) hold up to byte equality
//! because rkyv's serialization is deterministic.
//!
//! Format: rkyv 0.8 (zero-copy archival). The archived form is laid out
//! for direct access; `from_archive_bytes` validates with `bytecheck`
//! before materializing, so a corrupted or truncated blob fails closed
//! rather than producing unsound references.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::EntityRef;
use crate::codegen_data::CodegenData;

/// Owned, serializable mirror of [`CodegenData`].
///
/// Field-for-field identical to `CodegenData<P>` except that every
/// `&'static str` becomes an owned [`String`] and every typed
/// [`crate::EntityRef`] becomes its raw `u64` handle (the phantom marker
/// `P` is reconstructed when a typed [`CodegenData`] view is rebuilt by
/// the materialization functor — the integer handle is identical machine
/// data either way).
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
    /// shape. The forgetful direction of the embed/forget adjunction:
    /// the phantom marker `P` is dropped (handles become raw integers).
    pub fn from_codegen_data<P: 'static>(data: &CodegenData<P>) -> Self {
        let edges = |s: &[(crate::EntityRef<P>, crate::EntityRef<P>)]| -> Vec<(u64, u64)> {
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
    /// `static`, and the same trade made by the on-disk USLM loader and
    /// the `OnceLock`-cached corpus singleton.
    ///
    /// This is the join point that lets the existing `from_codegen`
    /// functors (`English::from_codegen`, `UsCode::from_codegen`, …)
    /// serve every staging unchanged: an archive-delivered ontology and
    /// a build-time-codegen'd one reach the runtime through the *same*
    /// materializer — only the byte source differs.
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
}

/// Serialize to a content-addressable archive blob (the lens *put*).
///
/// Deterministic: the same `OwnedCodegenData` always produces the same
/// bytes, so the blob's SHA-256 is a stable content address (the
/// `canonical_signature` recorded in `praxis.lock`).
pub fn to_archive_bytes(data: &OwnedCodegenData) -> Result<Vec<u8>, rkyv::rancor::Error> {
    rkyv::to_bytes::<rkyv::rancor::Error>(data).map(|v| v.to_vec())
}

/// Materialize from an archive blob (the lens *get*).
///
/// Copies the input into an aligned buffer first, so callers may pass a
/// byte slice of any alignment — e.g. a `Vec<u8>` from a network fetch
/// or `std::fs::read`, neither of which guarantees rkyv's archive
/// alignment. Validates with `bytecheck` before materializing.
pub fn from_archive_bytes(bytes: &[u8]) -> Result<OwnedCodegenData, rkyv::rancor::Error> {
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<OwnedCodegenData, rkyv::rancor::Error>(&aligned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    fn sample() -> OwnedCodegenData {
        OwnedCodegenData {
            entity_count: 3,
            entity_ids: vec_of(&["oewn-02084071-n", "oewn-01317541-n", "oewn-00015388-n"]),
            entity_kind: vec_of(&["n", "n", "n"]),
            entity_labels: vec_of(&["dog", "animal", "being"]),
            entity_defs: vec_of(&["a domesticated carnivore", "a living organism", ""]),
            word_index: alloc::vec![
                ("animal".to_string(), alloc::vec![1u64]),
                ("dog".to_string(), alloc::vec![0u64]),
            ],
            taxonomy: alloc::vec![(0, 1), (1, 2)],
            mereology: alloc::vec![],
            opposition: alloc::vec![],
            equivalence: alloc::vec![],
            causation: alloc::vec![],
            references: alloc::vec![(0, 2)],
        }
    }

    fn vec_of(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn round_trip_concrete_sample() {
        let data = sample();
        let bytes = to_archive_bytes(&data).expect("serialize");
        let back = from_archive_bytes(&bytes).expect("deserialize");
        assert_eq!(data, back, "rkyv round-trip must be lossless");
    }

    #[test]
    fn deterministic_bytes_are_a_stable_content_address() {
        // Two independent serializations of equal data yield equal bytes
        // — the property that lets the blob's hash be its content address.
        let a = to_archive_bytes(&sample()).expect("serialize a");
        let b = to_archive_bytes(&sample()).expect("serialize b");
        assert_eq!(a, b, "rkyv serialization must be deterministic");
    }

    #[test]
    fn corrupted_blob_fails_closed() {
        let mut bytes = to_archive_bytes(&sample()).expect("serialize");
        // Flip the leading byte; bytecheck validation must reject it
        // rather than materialize an unsound structure.
        bytes[0] ^= 0xFF;
        assert!(
            from_archive_bytes(&bytes).is_err(),
            "validated get must reject a corrupted blob"
        );
    }

    #[test]
    fn from_codegen_data_projects_static_into_owned() {
        struct TestMarker;
        use crate::EntityRef;
        static IDS: &[&str] = &["a", "b"];
        static KIND: &[&str] = &["n", "v"];
        static LABELS: &[&str] = &["alpha", "beta"];
        static DEFS: &[&str] = &["first", "second"];
        static WORDS: &[(&str, &[EntityRef<TestMarker>])] = &[
            ("alpha", &[EntityRef::new(0)]),
            ("beta", &[EntityRef::new(1)]),
        ];
        static TAX: &[(EntityRef<TestMarker>, EntityRef<TestMarker>)] =
            &[(EntityRef::new(1), EntityRef::new(0))];
        static EMPTY: &[(EntityRef<TestMarker>, EntityRef<TestMarker>)] = &[];
        let cg: CodegenData<TestMarker> = CodegenData {
            entity_count: 2,
            entity_ids: IDS,
            entity_kind: KIND,
            entity_labels: LABELS,
            entity_defs: DEFS,
            word_index: WORDS,
            taxonomy: TAX,
            mereology: EMPTY,
            opposition: EMPTY,
            equivalence: EMPTY,
            causation: EMPTY,
            references: EMPTY,
        };
        let owned = OwnedCodegenData::from_codegen_data(&cg);
        assert_eq!(owned.entity_count, 2);
        assert_eq!(owned.entity_labels, vec_of(&["alpha", "beta"]));
        assert_eq!(owned.taxonomy, alloc::vec![(1u64, 0u64)]);
        assert_eq!(
            owned.word_index,
            alloc::vec![
                ("alpha".to_string(), alloc::vec![0u64]),
                ("beta".to_string(), alloc::vec![1u64]),
            ]
        );
        // And the projection survives a binary round-trip.
        let bytes = to_archive_bytes(&owned).expect("serialize");
        assert_eq!(owned, from_archive_bytes(&bytes).expect("deserialize"));
    }

    #[test]
    fn to_codegen_data_leaked_reconstructs_the_view() {
        struct P;
        let owned = sample();
        let cg: CodegenData<P> = owned.to_codegen_data_leaked();
        assert_eq!(cg.entity_count, 3);
        assert_eq!(cg.entity_ids.len(), 3);
        assert_eq!(cg.entity_labels[0], "dog");
        // word_index is queried by binary search — `lookup` must resolve.
        assert_eq!(cg.lookup("dog").len(), 1);
        assert_eq!(cg.lookup("dog")[0].value(), 0);
        assert_eq!(cg.lookup("animal")[0].value(), 1);
        assert!(cg.lookup("missing").is_empty());
        assert_eq!(cg.taxonomy.len(), 2);
        assert_eq!(cg.taxonomy[0].0.value(), 0);
        assert_eq!(cg.taxonomy[0].1.value(), 1);
        assert_eq!(cg.references[0].1.value(), 2);
    }

    /// bytes → owned → typed view: the full archive-delivery path
    /// reconstructs the same handles the owned data carried.
    #[test]
    fn bytes_to_leaked_view_preserves_handles() {
        struct P;
        let bytes = to_archive_bytes(&sample()).expect("serialize");
        let owned = from_archive_bytes(&bytes).expect("deserialize");
        let cg: CodegenData<P> = owned.to_codegen_data_leaked();
        assert_eq!(cg.entity_count, 3);
        assert_eq!(cg.lookup("dog")[0].value(), 0);
    }

    fn arb_pairs() -> impl Strategy<Value = Vec<(u64, u64)>> {
        vec((any::<u64>(), any::<u64>()), 0..8)
    }

    prop_compose! {
        fn arb_owned()(
            entity_count in any::<u64>(),
            entity_ids in vec(".*", 0..6),
            entity_kind in vec(".*", 0..6),
            entity_labels in vec(".*", 0..6),
            entity_defs in vec(".*", 0..6),
            word_index in vec((".*", vec(any::<u64>(), 0..4)), 0..6),
            taxonomy in arb_pairs(),
            mereology in arb_pairs(),
            opposition in arb_pairs(),
            equivalence in arb_pairs(),
            causation in arb_pairs(),
            references in arb_pairs(),
        ) -> OwnedCodegenData {
            OwnedCodegenData {
                entity_count,
                entity_ids,
                entity_kind,
                entity_labels,
                entity_defs,
                word_index,
                taxonomy,
                mereology,
                opposition,
                equivalence,
                causation,
                references,
            }
        }
    }

    proptest! {
        /// GetPut law (Foster et al. 2007 §2.2): `get(put(t)) == t` for
        /// every archival value `t`.
        #[test]
        fn prop_round_trip_lossless(data in arb_owned()) {
            let bytes = to_archive_bytes(&data).expect("serialize");
            let back = from_archive_bytes(&bytes).expect("deserialize");
            prop_assert_eq!(data, back);
        }

        /// The serialization is a function of the value alone — equal
        /// values serialize to equal bytes (stable content addressing).
        #[test]
        fn prop_serialization_deterministic(data in arb_owned()) {
            let a = to_archive_bytes(&data).expect("serialize a");
            let b = to_archive_bytes(&data).expect("serialize b");
            prop_assert_eq!(a, b);
        }
    }
}
