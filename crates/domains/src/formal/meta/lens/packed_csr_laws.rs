//! Runnable, cited lens-law axioms for the shared
//! [`PackedCsrDict`](crate::formal::meta::packed_csr::PackedCsrDict) /
//! [`PackedCsrFamily`](crate::formal::meta::packed_csr::PackedCsrFamily) — the
//! zero-copy CSR representation the five English M1 stores share.
//!
//! `pack` (build the packed buffer from an owned `HashMap`) is the lens PUT;
//! `unpack` (reconstruct the owned map from the buffer) is the GET. This module
//! lifts the two well-behaved-lens legs plus the two faithfulness properties into
//! registered, discoverable `Axiom`s, each verifying over synthetic witnesses
//! with teeth, mirroring `pr4xis_runtime`'s `ArchiveLensGetPut` / `ArchiveLensPutGet`:
//!
//! - `PackedCsrGetPut` — `pack(unpack(b)) == b`: the packed bytes are stable
//!   under a decode/re-encode round-trip (the cache blob is canonical for the map).
//! - `PackedCsrPutGet` — `unpack(pack(m)) == m`: an owned map round-trips through
//!   the packed form with its full query image intact.
//! - `PackedCsrZeroCopyFaithful` — the archived zero-copy read equals the owned
//!   `HashMap` read for EVERY key (dict) and every `(label, id)` (family); the one
//!   property the five stores' now-deleted `*_is_identical_to_the_owned_fallback`
//!   unit tests each asserted, proven once here over all three value columns.
//! - `PackedCsrFamilyLabelFaithful` — every family label reads its OWN column and
//!   no other's; the generalization of `relation_store`'s
//!   `every_kind_returns_its_own_distinct_edge`.
//!
//! Gated on `prx` + little-endian, where the zero-copy `ArchivedCsrDict` /
//! `ArchivedCsrFamily` exist (the owned fallback needs no lens laws — its `pack`
//! is the identity). The four axioms self-register through `register_axiom!`, so
//! they resolve by name through the same registry as every other lens law.
//!
//! # Literature
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §3, Definition 3.2 — the
//!   well-behaved-lens laws (GetPut / PutGet).
//! - **Koloski, D.** *rkyv: zero-copy deserialization framework for Rust*, v0.8,
//!   <https://github.com/rkyv/rkyv> — the `AlignedVec` the buffer reuses.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use hashbrown::HashMap;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::formal::information::ontology::Ref;
use crate::formal::meta::packed_csr::{
    ArchivedCsrDict, ArchivedCsrFamily, CheckedEnumRun, DenseKey, EnumBound, LabelKind,
    OwnedCsrDict, OwnedCsrFamily, PodElem, PodRun, PodScalar, SortedKeys, ValueColumn,
};

// ── witness POD / label types (local — no domain-store import) ───────────────

/// A witness dense id.
fn r(i: u64) -> Ref<4> {
    Ref::new(i)
}

/// A witness `#[repr(u8)]` enum for the [`CheckedEnumRun`] column — the analogue
/// of `Transitivity`, kept local so the laws import no domain store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Tri {
    A,
    B,
    C,
}

impl PodElem for Tri {
    const SIZE: usize = 1;
    fn le_bytes(&self) -> [u8; 8] {
        [*self as u8, 0, 0, 0, 0, 0, 0, 0]
    }
}

impl EnumBound for Tri {
    const MAX_DISCRIMINANT: u8 = Tri::C as u8;
}

/// A witness 4-label family tag — the analogue of `Direction` / `RelationKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quad {
    W,
    X,
    Y,
    Z,
}

impl LabelKind for Quad {
    const COUNT: usize = 4;
    fn index(self) -> usize {
        self as usize
    }
    fn all() -> &'static [Self] {
        &[Quad::W, Quad::X, Quad::Y, Quad::Z]
    }
}

// ── witness data ─────────────────────────────────────────────────────────────

/// Witness run-dictionaries: an empty dict, a single-run dict, and a rich dict
/// with a multi-id run whose order must be preserved.
fn run_witnesses() -> Vec<HashMap<String, Vec<Ref<4>>>> {
    let empty: HashMap<String, Vec<Ref<4>>> = HashMap::new();
    let mut one: HashMap<String, Vec<Ref<4>>> = HashMap::new();
    one.insert(String::from("alpha"), alloc::vec![r(7)]);
    let mut rich: HashMap<String, Vec<Ref<4>>> = HashMap::new();
    rich.insert(String::from("gamma"), alloc::vec![r(3), r(9), r(1)]);
    rich.insert(String::from("beta"), alloc::vec![r(42)]);
    rich.insert(String::from("alpha"), alloc::vec![r(7)]);
    alloc::vec![empty, one, rich]
}

/// Witness scalar-dictionaries (one id per key).
fn scalar_witnesses() -> Vec<HashMap<String, Ref<4>>> {
    let empty: HashMap<String, Ref<4>> = HashMap::new();
    let mut rich: HashMap<String, Ref<4>> = HashMap::new();
    rich.insert(String::from("oewn-99-v"), r(2));
    rich.insert(String::from("oewn-00-n"), r(0));
    rich.insert(String::from("oewn-02-n"), r(42));
    alloc::vec![empty, rich]
}

/// Witness checked-enum-run dictionaries.
fn enum_witnesses() -> Vec<HashMap<String, Vec<Tri>>> {
    let mut rich: HashMap<String, Vec<Tri>> = HashMap::new();
    rich.insert(String::from("walk"), alloc::vec![Tri::B]);
    rich.insert(String::from("run"), alloc::vec![Tri::B, Tri::A]);
    rich.insert(String::from("see"), alloc::vec![Tri::A, Tri::C]);
    alloc::vec![rich]
}

/// Witness family: `Quad::COUNT` columns over a common row space, each column
/// carrying ONE distinct edge `label k → [100 + k]` keyed at id `k`.
fn label_witness_bundle() -> Vec<(usize, HashMap<DenseKey, Vec<Ref<4>>>)> {
    let n = Quad::COUNT;
    (0..n)
        .map(|k| {
            let mut m: HashMap<DenseKey, Vec<Ref<4>>> = HashMap::new();
            m.insert(r(k as u64), alloc::vec![r(100 + k as u64)]);
            (n, m)
        })
        .collect()
}

// ── generic law helpers over the value-column axis ───────────────────────────

/// GetPut leg: the packed bytes are stable under decode/re-encode.
fn getput_ok<V>(maps: Vec<HashMap<String, V::Owned>>) -> bool
where
    V: ValueColumn,
{
    for m in maps {
        let d = ArchivedCsrDict::<SortedKeys, V>::build(m.clone());
        let bytes = d.as_bytes().to_vec();
        let reencoded = ArchivedCsrDict::<SortedKeys, V>::build(d.unpack());
        if reencoded.as_bytes() != bytes.as_slice() {
            return false;
        }
    }
    true
}

/// PutGet leg: an owned map round-trips through the packed form.
fn putget_ok<V>(maps: Vec<HashMap<String, V::Owned>>) -> bool
where
    V: ValueColumn,
    V::Owned: PartialEq,
{
    for m in maps {
        let d = ArchivedCsrDict::<SortedKeys, V>::build(m.clone());
        if d.unpack() != m {
            return false;
        }
    }
    true
}

/// ZeroCopyFaithful (dict leg): the archived read equals the owned read for every
/// key in the map plus the given miss probes.
fn faithful_dict<V>(maps: Vec<HashMap<String, V::Owned>>, misses: &[&str]) -> bool
where
    V: ValueColumn,
    for<'a> V::Read<'a>: PartialEq,
{
    for m in maps {
        let archived = ArchivedCsrDict::<SortedKeys, V>::build(m.clone());
        let owned = OwnedCsrDict::<SortedKeys, V>::build(m.clone());
        let keys: Vec<String> = m.keys().cloned().collect();
        for k in keys
            .iter()
            .map(String::as_str)
            .chain(misses.iter().copied())
        {
            if archived.lookup(k) != owned.lookup(k) {
                return false;
            }
        }
    }
    true
}

// ── the four axioms ──────────────────────────────────────────────────────────

/// GetPut leg of the `PackedCsr` well-behaved lens: for buffer bytes `b`
/// canonically produced by `pack`, `pack(unpack(b)) == b` — the packed cache blob
/// is stable under a decode/re-encode round-trip, over run / scalar / checked-enum
/// dictionaries. Foster, Greenwald, Moore, Pierce & Schmitt (2007) §3, Definition 3.2.
pub struct PackedCsrGetPut;

impl Axiom for PackedCsrGetPut {
    fn verify(&self) -> Verdict {
        let holds = getput_ok::<PodRun<Ref<4>>>(run_witnesses())
            && getput_ok::<PodScalar<Ref<4>>>(scalar_witnesses())
            && getput_ok::<CheckedEnumRun<Tri>>(enum_witnesses());
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PackedCsrGetPut",
        "pack(unpack(b)) == b for canonically-produced packed CSR bytes, over run/scalar/checked-enum dictionaries",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(PackedCsrGetPut, constructor);

/// PutGet leg of the `PackedCsr` well-behaved lens: `unpack(pack(m)) == m` — an
/// owned map round-trips through the packed form with its full query image (every
/// key, every run in order) intact, over run / scalar / checked-enum dictionaries.
/// Foster, Greenwald, Moore, Pierce & Schmitt (2007) §3, Definition 3.2.
pub struct PackedCsrPutGet;

impl Axiom for PackedCsrPutGet {
    fn verify(&self) -> Verdict {
        let holds = putget_ok::<PodRun<Ref<4>>>(run_witnesses())
            && putget_ok::<PodScalar<Ref<4>>>(scalar_witnesses())
            && putget_ok::<CheckedEnumRun<Tri>>(enum_witnesses());
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PackedCsrPutGet",
        "unpack(pack(m)) == m: an owned map round-trips through the packed CSR form with its full query image intact",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(PackedCsrPutGet, constructor);

/// The archived zero-copy read is faithful to the owned `HashMap` read for EVERY
/// key (dict, all three value columns) and every `(label, id)` (family) — the
/// property each store's now-deleted `*_is_identical_to_the_owned_fallback` unit
/// test asserted, proven once. The zero-copy `&[…]` cast returns exactly what the
/// owned map would. Foster et al. (2007) §3, Definition 3.2 (the PutGet counit at zero-copy read
/// identity).
pub struct PackedCsrZeroCopyFaithful;

impl Axiom for PackedCsrZeroCopyFaithful {
    fn verify(&self) -> Verdict {
        let dict_ok = faithful_dict::<PodRun<Ref<4>>>(run_witnesses(), &["delta", "alph", ""])
            && faithful_dict::<PodScalar<Ref<4>>>(scalar_witnesses(), &["absent", "oewn-00"])
            && faithful_dict::<CheckedEnumRun<Tri>>(enum_witnesses(), &["swim", "ru"]);

        // Family leg: archived column == owned column for every (label, id),
        // including out-of-range ids.
        let archived = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(label_witness_bundle());
        let owned = OwnedCsrFamily::<Quad, PodRun<Ref<4>>>::build(label_witness_bundle());
        let mut family_ok = true;
        for &tag in Quad::all() {
            for i in 0..(Quad::COUNT as u64 + 2) {
                if archived.column(tag, r(i)) != owned.column(tag, r(i)) {
                    family_ok = false;
                }
            }
        }

        if dict_ok && family_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PackedCsrZeroCopyFaithful",
        "the archived zero-copy read equals the owned HashMap read for every dict key and every family (label, id)",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(PackedCsrZeroCopyFaithful, constructor);

/// Every family label reads its OWN column and no other's: with one distinct edge
/// `label k → [100 + k]` at id `k` per label, `column(tag_k, k) == [100 + k]` and
/// every other label at id `k` is empty. The generalization of `relation_store`'s
/// `every_kind_returns_its_own_distinct_edge` — a mislabelled column would read a
/// different label's map and fail. Foster et al. (2007) §3, Definition 3.2.
pub struct PackedCsrFamilyLabelFaithful;

impl Axiom for PackedCsrFamilyLabelFaithful {
    fn verify(&self) -> Verdict {
        let family = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(label_witness_bundle());
        let mut holds = true;
        for (k, &tag) in Quad::all().iter().enumerate() {
            let id = r(k as u64);
            if family.column(tag, id) != [r(100 + k as u64)].as_slice() {
                holds = false;
            }
            for (j, &other) in Quad::all().iter().enumerate() {
                if j != k && !family.column(other, id).is_empty() {
                    holds = false;
                }
            }
        }
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PackedCsrFamilyLabelFaithful",
        "every packed CSR family label reads its own column and no other's (distinct per-label edges are never crossed)",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(PackedCsrFamilyLabelFaithful, constructor);

// ── query-behaviour proptest (the teeth) + discoverability ───────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use pr4xis::ontology::registry::axiom_by_name;
    use proptest::prelude::*;

    /// The four lens-law axioms hold over their witnesses.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn packed_csr_laws_hold() {
        assert!(PackedCsrGetPut.verify().is_ok(), "pack(unpack(b)) == b");
        assert!(PackedCsrPutGet.verify().is_ok(), "unpack(pack(m)) == m");
        assert!(
            PackedCsrZeroCopyFaithful.verify().is_ok(),
            "archived read == owned read"
        );
        assert!(
            PackedCsrFamilyLabelFaithful.verify().is_ok(),
            "every label reads its own column"
        );
    }

    /// The four axioms re-bind by name through the registry — discoverable as any
    /// statute's law is (the load-time rebind gate).
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn laws_discoverable_via_registry() {
        for name in [
            "PackedCsrGetPut",
            "PackedCsrPutGet",
            "PackedCsrZeroCopyFaithful",
            "PackedCsrFamilyLabelFaithful",
        ] {
            assert!(
                axiom_by_name(name).is_some(),
                "packed-csr axiom {name} must re-bind through the registry"
            );
        }
    }

    /// Generated dictionary entries (lowercase keys), plus a disjoint uppercase
    /// miss probe. Entries are collected into a `hashbrown::HashMap` in the body
    /// (proptest's own `hash_map` combinator yields a `std` map).
    fn dict_strategy() -> impl Strategy<Value = (Vec<(String, Vec<Ref<4>>)>, String)> {
        (
            prop::collection::vec(
                (
                    "[a-z]{1,5}",
                    prop::collection::vec((0u64..1000).prop_map(r), 0..4),
                ),
                0..8,
            ),
            "[A-Z]{1,5}", // uppercase → guaranteed absent from the lowercase keys
        )
    }

    proptest! {
        /// A key absent from the map reads empty (dict) — the exact binary search
        /// never returns a neighbouring key's run.
        #[test]
        fn prop_lookup_miss_is_empty((entries, miss) in dict_strategy()) {
            let map: HashMap<String, Vec<Ref<4>>> = entries.into_iter().collect();
            let dict = ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::build(map.clone());
            prop_assume!(!map.contains_key(&miss));
            prop_assert!(dict.lookup(&miss).is_empty());
        }

        /// An id at or beyond a family column's row count reads empty (no OOB read).
        #[test]
        fn prop_out_of_range_is_empty(rows in 0usize..6, over in 0u64..4) {
            // A `Quad` family whose `W` column is populated over `0..rows`.
            let bundle: Vec<(usize, HashMap<DenseKey, Vec<Ref<4>>>)> = Quad::all()
                .iter()
                .map(|&tag| {
                    let mut m: HashMap<DenseKey, Vec<Ref<4>>> = HashMap::new();
                    if tag == Quad::W {
                        for i in 0..rows {
                            m.insert(r(i as u64), alloc::vec![r(1000 + i as u64)]);
                        }
                    }
                    (rows, m)
                })
                .collect();
            let fam = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle);
            let id = r(rows as u64 + over);
            prop_assert!(fam.column(Quad::W, id).is_empty());
        }

        /// Each present key's run is returned in its exact insertion order — the
        /// pack/cast never reorders a run.
        #[test]
        fn prop_order_preserved((entries, _miss) in dict_strategy()) {
            let map: HashMap<String, Vec<Ref<4>>> = entries.into_iter().collect();
            let dict = ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::build(map.clone());
            for (k, v) in &map {
                prop_assert_eq!(dict.lookup(k), v.as_slice());
            }
        }
    }

    pr4xis::register_praxis_value!(prop_lookup_miss_is_empty, Verifiable);
    pr4xis::register_praxis_value!(prop_out_of_range_is_empty, Verifiable);
    pr4xis::register_praxis_value!(prop_order_preserved, Verifiable);

    // ── the UNTRUSTED construction path: adversarial-byte properties ─────────

    use crate::formal::meta::packed_csr::PackedCsrError;
    use rkyv::util::AlignedVec;

    /// Copy raw bytes into a fresh 16-aligned buffer — the shape an untrusted
    /// wire payload arrives in.
    fn aligned(bytes: &[u8]) -> AlignedVec<16> {
        let mut v = AlignedVec::<16>::with_capacity(bytes.len());
        v.extend_from_slice(bytes);
        v
    }

    /// The ∀-byte-mutation totality+sanity check, generic over the value
    /// column: pack a real dict, flip ONE byte, and require
    /// `from_untrusted_buf` to be TOTAL — `Err`, or an `Ok` dict whose every
    /// read (unpack, all-key lookups, miss probes, key iteration) completes
    /// without panicking. Unlike the pinned raw-source envelopes, a mutated
    /// CSR buffer may still be VALID bytes (e.g. a flipped value byte is just
    /// a different id) — the property is totality + sanity, not rejection.
    fn mutated_untrusted_is_total<V>(
        map: HashMap<String, V::Owned>,
        byte_idx: usize,
        xor: u8,
        misses: &[&str],
    ) -> Result<(), TestCaseError>
    where
        V: ValueColumn,
        V::Owned: PartialEq + core::fmt::Debug,
    {
        let packed = ArchivedCsrDict::<SortedKeys, V>::pack(&map);
        let mut bad = packed.as_slice().to_vec();
        let i = byte_idx % bad.len(); // pack always emits ≥ the 16-byte header
        bad[i] ^= xor;
        let keys: Vec<String> = map.keys().cloned().collect();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || -> Result<(), TestCaseError> {
                match ArchivedCsrDict::<SortedKeys, V>::from_untrusted_buf(aligned(&bad)) {
                    Err(_) => Ok(()), // fail-closed refusal: correct
                    Ok(dict) => {
                        // Accepted ⇒ every read is total (bounds hold by validation).
                        let _ = dict.unpack();
                        for k in keys
                            .iter()
                            .map(String::as_str)
                            .chain(misses.iter().copied())
                        {
                            let _ = dict.lookup(k);
                        }
                        prop_assert_eq!(dict.keys().count(), dict.len());
                        Ok(())
                    }
                }
            },
        ));
        match outcome {
            Ok(r) => r,
            Err(_) => Err(TestCaseError::fail(format!(
                "from_untrusted_buf PANICKED on byte {i} ^= {xor}"
            ))),
        }
    }

    /// Generated scalar dictionary entries (one id per key).
    fn scalar_dict_strategy() -> impl Strategy<Value = Vec<(String, Ref<4>)>> {
        prop::collection::vec(("[a-z]{1,5}", (0u64..1000).prop_map(r)), 0..8)
    }

    /// Generated checked-enum-run dictionary entries (`Tri` runs — the local
    /// analogue of `Transitivity`'s `CheckedEnumRun`).
    fn enum_dict_strategy() -> impl Strategy<Value = Vec<(String, Vec<Tri>)>> {
        prop::collection::vec(
            (
                "[a-z]{1,5}",
                prop::collection::vec(prop_oneof![Just(Tri::A), Just(Tri::B), Just(Tri::C)], 0..4),
            ),
            0..8,
        )
    }

    proptest! {
        /// ∀ single-byte mutation of a real packed `PodRun<Ref<4>>` dict, the
        /// validated path is total and sane — Err, or an Ok whose reads never
        /// panic and never read out of bounds.
        #[test]
        fn prop_mutated_untrusted_run_dict_is_total(
            (entries, miss) in dict_strategy(),
            byte_idx in 0usize..4096,
            xor in 1u8..=255,
        ) {
            let map: HashMap<String, Vec<Ref<4>>> = entries.into_iter().collect();
            mutated_untrusted_is_total::<PodRun<Ref<4>>>(map, byte_idx, xor, &[&miss])?;
        }

        /// The same ∀-mutation property over the `PodScalar<Ref<4>>` column
        /// (`synset_index`'s shape).
        #[test]
        fn prop_mutated_untrusted_scalar_dict_is_total(
            entries in scalar_dict_strategy(),
            byte_idx in 0usize..4096,
            xor in 1u8..=255,
        ) {
            let map: HashMap<String, Ref<4>> = entries.into_iter().collect();
            mutated_untrusted_is_total::<PodScalar<Ref<4>>>(map, byte_idx, xor, &["ZZ"])?;
        }

        /// The same ∀-mutation property over the `CheckedEnumRun` column
        /// (`verb_transitivity_index`'s shape) — additionally, an accepted
        /// buffer can never carry an out-of-range discriminant (the payload
        /// sweep), so the zero-copy `&[Tri]` cast stays sound on hostile bytes.
        #[test]
        fn prop_mutated_untrusted_enum_dict_is_total(
            entries in enum_dict_strategy(),
            byte_idx in 0usize..4096,
            xor in 1u8..=255,
        ) {
            let map: HashMap<String, Vec<Tri>> = entries.into_iter().collect();
            mutated_untrusted_is_total::<CheckedEnumRun<Tri>>(map, byte_idx, xor, &["ZZ"])?;
        }

        /// POSITIVE CONTROL: every canonically packed buffer — all three value
        /// columns — is ACCEPTED by the validated path, and its reads equal the
        /// owned baseline (the untrusted path is not "reject everything").
        #[test]
        fn prop_untrusted_accepts_canonical_pack(
            (run_entries, miss) in dict_strategy(),
            scalar_entries in scalar_dict_strategy(),
            enum_entries in enum_dict_strategy(),
        ) {
            let run_map: HashMap<String, Vec<Ref<4>>> = run_entries.into_iter().collect();
            let d = ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::from_untrusted_buf(
                ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::pack(&run_map),
            ).map_err(|e| TestCaseError::fail(format!("run dict refused: {e}")))?;
            let owned = OwnedCsrDict::<SortedKeys, PodRun<Ref<4>>>::build(run_map.clone());
            for k in run_map.keys().map(String::as_str).chain([miss.as_str()]) {
                prop_assert_eq!(d.lookup(k), owned.lookup(k));
            }

            let scalar_map: HashMap<String, Ref<4>> = scalar_entries.into_iter().collect();
            let d = ArchivedCsrDict::<SortedKeys, PodScalar<Ref<4>>>::from_untrusted_buf(
                ArchivedCsrDict::<SortedKeys, PodScalar<Ref<4>>>::pack(&scalar_map),
            ).map_err(|e| TestCaseError::fail(format!("scalar dict refused: {e}")))?;
            prop_assert_eq!(d.unpack(), scalar_map);

            let enum_map: HashMap<String, Vec<Tri>> = enum_entries.into_iter().collect();
            let d = ArchivedCsrDict::<SortedKeys, CheckedEnumRun<Tri>>::from_untrusted_buf(
                ArchivedCsrDict::<SortedKeys, CheckedEnumRun<Tri>>::pack(&enum_map),
            ).map_err(|e| TestCaseError::fail(format!("enum dict refused: {e}")))?;
            prop_assert_eq!(d.unpack(), enum_map);
        }

        /// FAMILY MULTI-KIND generator widening: archived reads equal owned
        /// reads over GENERATED multi-label bundles (every `Quad` label
        /// populated independently), not just the one-edge-per-label witness.
        #[test]
        fn prop_family_multi_kind_faithful(
            cols in prop::collection::vec(
                prop::collection::vec(
                    prop::collection::vec((0u64..1000).prop_map(r), 0..3),
                    0..6,
                ),
                Quad::COUNT..=Quad::COUNT,
            ),
            probe in 0u64..8,
        ) {
            let rows = cols.iter().map(Vec::len).max().unwrap_or(0);
            let bundle: Vec<(usize, HashMap<DenseKey, Vec<Ref<4>>>)> = cols
                .iter()
                .map(|runs| {
                    let mut m: HashMap<DenseKey, Vec<Ref<4>>> = HashMap::new();
                    for (i, run) in runs.iter().enumerate() {
                        if !run.is_empty() {
                            m.insert(r(i as u64), run.clone());
                        }
                    }
                    (rows, m)
                })
                .collect();
            let archived = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle.clone());
            let owned = OwnedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle);
            for &tag in Quad::all() {
                prop_assert_eq!(archived.edge_count(tag), owned.edge_count(tag));
                prop_assert_eq!(archived.row_count(tag), owned.row_count(tag));
                for i in 0..(rows as u64 + probe + 1) {
                    prop_assert_eq!(archived.column(tag, r(i)), owned.column(tag, r(i)));
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_mutated_untrusted_run_dict_is_total, Honest);
    pr4xis::register_praxis_value!(prop_mutated_untrusted_scalar_dict_is_total, Honest);
    pr4xis::register_praxis_value!(prop_mutated_untrusted_enum_dict_is_total, Honest);
    pr4xis::register_praxis_value!(prop_untrusted_accepts_canonical_pack, Verifiable);
    pr4xis::register_praxis_value!(prop_family_multi_kind_faithful, Verifiable);

    /// A generated `Quad` family bundle (each label's per-id runs, independent).
    fn family_bundle_strategy()
    -> impl Strategy<Value = Vec<(usize, HashMap<DenseKey, Vec<Ref<4>>>)>> {
        prop::collection::vec(
            prop::collection::vec(prop::collection::vec((0u64..1000).prop_map(r), 0..3), 0..6),
            Quad::COUNT..=Quad::COUNT,
        )
        .prop_map(|cols| {
            let rows = cols.iter().map(Vec::len).max().unwrap_or(0);
            cols.iter()
                .map(|runs| {
                    let mut m: HashMap<DenseKey, Vec<Ref<4>>> = HashMap::new();
                    for (i, run) in runs.iter().enumerate() {
                        if !run.is_empty() {
                            m.insert(r(i as u64), run.clone());
                        }
                    }
                    (rows, m)
                })
                .collect()
        })
    }

    proptest! {
        /// ∀ single-byte mutation of a real packed FAMILY buffer (byte payload
        /// AND declared column table), `ArchivedCsrFamily::from_untrusted_buf`
        /// is TOTAL — `Err`, or an `Ok` family whose every read (all labels,
        /// all rows + an over-range probe, edge counts) completes without
        /// panicking and without an out-of-bounds read. The frame's
        /// `(row_count, edge_count)` header is the genuinely new attack
        /// surface the store bundle introduces; this is its hostile-bytes
        /// property (the wave-4 dict discipline, applied to the family).
        #[test]
        fn prop_mutated_untrusted_family_is_total(
            bundle in family_bundle_strategy(),
            byte_idx in 0usize..4096,
            xor in 1u8..=255,
            col_tamper in 0usize..3,
            col_delta in 1usize..4,
        ) {
            let fam = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle);
            let bytes = fam.as_bytes().to_vec();
            let mut cols = fam.col_layout();
            let rows = cols.iter().map(|&(rc, _)| rc).max().unwrap_or(0);
            // Tamper leg 0: flip a payload byte. Leg 1: inflate a column's
            // edge_count. Leg 2: inflate a column's row_count.
            let mut bad = bytes.clone();
            match col_tamper {
                0 if !bad.is_empty() => {
                    let i = byte_idx % bad.len();
                    bad[i] ^= xor;
                }
                1 => cols[byte_idx % Quad::COUNT].1 += col_delta,
                _ => cols[byte_idx % Quad::COUNT].0 += col_delta,
            }
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> Result<(), TestCaseError> {
                    match ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::from_untrusted_buf(
                        aligned(&bad),
                        &cols,
                    ) {
                        Err(_) => Ok(()), // fail-closed refusal: correct
                        Ok(f) => {
                            for &tag in Quad::all() {
                                let _ = f.edge_count(tag);
                                for i in 0..(rows as u64 + 2) {
                                    let _ = f.column(tag, r(i));
                                }
                            }
                            Ok(())
                        }
                    }
                },
            ));
            match outcome {
                Ok(res) => res?,
                Err(_) => return Err(TestCaseError::fail(
                    "family from_untrusted_buf PANICKED on tampered input".to_string(),
                )),
            }
        }

        /// POSITIVE CONTROL: every canonically built family round-trips through
        /// `as_bytes` + `col_layout` + `from_untrusted_buf`, and the accepted
        /// family reads label-identically to the owned baseline.
        #[test]
        fn prop_untrusted_family_accepts_canonical_build(
            bundle in family_bundle_strategy(),
            probe in 0u64..8,
        ) {
            let built = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle.clone());
            let accepted = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::from_untrusted_buf(
                aligned(built.as_bytes()),
                &built.col_layout(),
            ).map_err(|e| TestCaseError::fail(format!("canonical family refused: {e}")))?;
            let owned = OwnedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle);
            let rows = built.col_layout().iter().map(|&(rc, _)| rc).max().unwrap_or(0);
            for &tag in Quad::all() {
                prop_assert_eq!(accepted.edge_count(tag), owned.edge_count(tag));
                prop_assert_eq!(accepted.row_count(tag), owned.row_count(tag));
                for i in 0..(rows as u64 + probe + 1) {
                    prop_assert_eq!(accepted.column(tag, r(i)), owned.column(tag, r(i)));
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_mutated_untrusted_family_is_total, Honest);
    pr4xis::register_praxis_value!(prop_untrusted_family_accepts_canonical_build, Verifiable);

    /// Every NAMED forgery of a real packed FAMILY buffer / column table is
    /// refused with its typed verdict — the per-invariant teeth behind the
    /// family ∀-mutation property.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn untrusted_family_refuses_each_named_forgery() {
        // A Quad family: W gets 2 rows ([7], [3, 9]), the rest empty over 2 rows.
        let bundle: Vec<(usize, HashMap<DenseKey, Vec<Ref<4>>>)> = Quad::all()
            .iter()
            .map(|&tag| {
                let mut m: HashMap<DenseKey, Vec<Ref<4>>> = HashMap::new();
                if tag == Quad::W {
                    m.insert(r(0), alloc::vec![r(7)]);
                    m.insert(r(1), alloc::vec![r(3), r(9)]);
                }
                (2usize, m)
            })
            .collect();
        let fam = ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::build(bundle);
        let bytes = fam.as_bytes().to_vec();
        let cols = fam.col_layout();
        let refuse = |bad: &[u8], cols: &[(usize, usize)]| match ArchivedCsrFamily::<
            Quad,
            PodRun<Ref<4>>,
        >::from_untrusted_buf(
            aligned(bad), cols
        ) {
            Err(e) => e,
            Ok(_) => panic!("forged family must be refused"),
        };

        // Wrong column count (one entry short / one extra).
        assert!(matches!(
            refuse(&bytes, &cols[..Quad::COUNT - 1]),
            PackedCsrError::WrongColumnCount { .. }
        ));
        let mut extra = cols.clone();
        extra.push((0, 0));
        assert!(matches!(
            refuse(&bytes, &extra),
            PackedCsrError::WrongColumnCount { .. }
        ));
        // Inflated edge_count / row_count: the declared layout no longer equals
        // the buffer length exactly.
        let mut bad_cols = cols.clone();
        bad_cols[0].1 += 1;
        assert!(matches!(
            refuse(&bytes, &bad_cols),
            PackedCsrError::LengthMismatch { .. }
        ));
        let mut bad_cols = cols.clone();
        bad_cols[0].0 += 1;
        assert!(matches!(
            refuse(&bytes, &bad_cols),
            PackedCsrError::LengthMismatch { .. }
        ));
        // A count past u32 addressability is an overflow refusal, never an
        // allocation or a truncating cast.
        let mut bad_cols = cols.clone();
        bad_cols[0].1 = u32::MAX as usize + 1;
        assert!(matches!(
            refuse(&bytes, &bad_cols),
            PackedCsrError::LayoutOverflow
        ));
        let mut bad_cols = cols.clone();
        bad_cols[0].0 = usize::MAX - 1;
        assert!(matches!(
            refuse(&bytes, &bad_cols),
            PackedCsrError::LayoutOverflow
        ));
        // Trailing garbage after the exact layout.
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(matches!(
            refuse(&bad, &cols),
            PackedCsrError::LengthMismatch { .. }
        ));
        // Non-monotone CSR offsets in the W column: W is the first column, its
        // offsets array starts right after ALL columns' targets (3 edges × 8).
        let offsets_at = 3 * 8;
        let mut bad = bytes.clone();
        bad[offsets_at + 4] = 0xf0; // offset[1] jumps past edge_count
        assert!(matches!(
            refuse(&bad, &cols),
            PackedCsrError::ValueOffsetsNotMonotone { .. }
        ));

        // CheckedEnumRun family: an out-of-range discriminant in the targets
        // region is InvalidPayload (the payload sweep guards the zero-copy cast).
        let ebundle: Vec<(usize, HashMap<DenseKey, Vec<Tri>>)> = Quad::all()
            .iter()
            .map(|&tag| {
                let mut m: HashMap<DenseKey, Vec<Tri>> = HashMap::new();
                if tag == Quad::W {
                    m.insert(r(0), alloc::vec![Tri::A, Tri::C]);
                }
                (1usize, m)
            })
            .collect();
        let efam = ArchivedCsrFamily::<Quad, CheckedEnumRun<Tri>>::build(ebundle);
        let mut ebad = efam.as_bytes().to_vec();
        ebad[0] = Tri::C as u8 + 1; // first targets byte out of range
        let err = match ArchivedCsrFamily::<Quad, CheckedEnumRun<Tri>>::from_untrusted_buf(
            aligned(&ebad),
            &efam.col_layout(),
        ) {
            Err(e) => e,
            Ok(_) => panic!("out-of-range family discriminant must be refused"),
        };
        assert!(matches!(err, PackedCsrError::InvalidPayload));

        // Positive control: the untouched canonical bytes + table are accepted
        // and read identically to the source family.
        let ok =
            ArchivedCsrFamily::<Quad, PodRun<Ref<4>>>::from_untrusted_buf(aligned(&bytes), &cols)
                .expect("canonical family must be accepted");
        assert_eq!(ok.column(Quad::W, r(0)), &[r(7)]);
        assert_eq!(ok.column(Quad::W, r(1)), &[r(3), r(9)]);
        assert_eq!(ok.edge_count(Quad::W), 3);
    }

    /// Every NAMED forgery of a real packed buffer is refused with its typed
    /// verdict — the per-invariant teeth behind the ∀-mutation property (which
    /// alone could pass with a validator that accepts everything).
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn untrusted_path_refuses_each_named_forgery() {
        let mut map: HashMap<String, Vec<Ref<4>>> = HashMap::new();
        map.insert(String::from("alpha"), alloc::vec![r(7)]);
        map.insert(String::from("beta"), alloc::vec![r(3), r(9)]);
        let good = ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::pack(&map);
        let bytes = good.as_slice().to_vec();
        let refuse = |bad: &[u8]| {
            ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::from_untrusted_buf(aligned(bad))
                .expect_err("forged buffer must be refused")
        };

        // Truncated header.
        assert!(matches!(
            refuse(&bytes[..8]),
            PackedCsrError::TruncatedHeader { len: 8 }
        ));
        // Non-zero pad word.
        let mut bad = bytes.clone();
        bad[12] = 1;
        assert!(matches!(refuse(&bad), PackedCsrError::NonZeroPad { .. }));
        // Forged n (over-declares the offset arrays past the buffer).
        let mut bad = bytes.clone();
        bad[0] = 0xff;
        assert!(matches!(
            refuse(&bad),
            PackedCsrError::LengthMismatch { .. } | PackedCsrError::LayoutOverflow
        ));
        // Forged val_count (over-declares the value array).
        let mut bad = bytes.clone();
        bad[4] = 0xff;
        assert!(matches!(
            refuse(&bad),
            PackedCsrError::LengthMismatch { .. } | PackedCsrError::LayoutOverflow
        ));
        // Trailing garbage after the exact layout.
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(matches!(
            refuse(&bad),
            PackedCsrError::LengthMismatch { .. }
        ));
        // Non-monotone value offsets: swap the interior offset above the total
        // (the offsets live right after the value array — locate via the same
        // layout the validator derives).
        let n = 2usize;
        let es = 8usize; // Ref<4>::SIZE
        let val_count = 3usize;
        let val_offsets_at = 16 + val_count * es;
        let mut bad = bytes.clone();
        bad[val_offsets_at + 4] = 0xf0; // offset[1] jumps past val_count
        assert!(matches!(
            refuse(&bad),
            PackedCsrError::ValueOffsetsNotMonotone { .. }
        ));
        // Non-UTF-8 key byte inside the key blob.
        let key_offsets_at = val_offsets_at + (n + 1) * 4;
        let key_blob_at = key_offsets_at + (n + 1) * 4;
        let mut bad = bytes.clone();
        bad[key_blob_at] = 0xff; // "alpha" → \xfflpha (invalid UTF-8 start)
        assert!(matches!(
            refuse(&bad),
            PackedCsrError::KeyNotUtf8 { index: 0 }
        ));
        // Unsorted keys: make key 0 lexically greater than key 1.
        let mut bad = bytes.clone();
        bad[key_blob_at] = b'z'; // "alpha" → "zlpha" > "beta"
        assert!(matches!(
            refuse(&bad),
            PackedCsrError::KeysNotSorted { index: 1 }
        ));

        // PodScalar arity: equal adjacent value offsets (an EMPTY run) must be
        // refused — a scalar column requires exactly one element per key, and
        // `to_owned`'s `elems[0]` would panic on the empty run (the exact case
        // the master-CI proptest seed found: monotone-but-empty offsets were
        // accepted, then the GET leg panicked instead of erroring).
        let mut smap: HashMap<String, Ref<4>> = HashMap::new();
        smap.insert(String::from("alpha"), r(7));
        smap.insert(String::from("beta"), r(3));
        let sgood = ArchivedCsrDict::<SortedKeys, PodScalar<Ref<4>>>::pack(&smap);
        let sbytes = sgood.as_slice().to_vec();
        // scalar layout: n=2, val_count=2 elems × 8B → offsets at 16 + 2*8 = 32,
        // offsets = [0, 1, 2]; forge offset[1] 1 → 0 (still monotone, run 0 empty).
        let s_val_offsets_at = 16 + 2 * 8;
        let mut sbad = sbytes.clone();
        sbad[s_val_offsets_at + 4] = 0;
        let err =
            ArchivedCsrDict::<SortedKeys, PodScalar<Ref<4>>>::from_untrusted_buf(aligned(&sbad))
                .expect_err("an empty scalar run must be refused, not panic the GET leg");
        assert!(matches!(
            err,
            PackedCsrError::InvalidRunLength { index: 0, len: 0 }
        ));

        // CheckedEnumRun: an out-of-range discriminant is InvalidPayload.
        let mut emap: HashMap<String, Vec<Tri>> = HashMap::new();
        emap.insert(String::from("walk"), alloc::vec![Tri::A, Tri::C]);
        let egood = ArchivedCsrDict::<SortedKeys, CheckedEnumRun<Tri>>::pack(&emap);
        let mut ebad = egood.as_slice().to_vec();
        ebad[16] = Tri::C as u8 + 1; // first payload byte out of range
        let err =
            ArchivedCsrDict::<SortedKeys, CheckedEnumRun<Tri>>::from_untrusted_buf(aligned(&ebad))
                .expect_err("out-of-range discriminant must be refused");
        assert!(matches!(err, PackedCsrError::InvalidPayload));

        // Positive control: the untouched canonical bytes are accepted.
        let ok = ArchivedCsrDict::<SortedKeys, PodRun<Ref<4>>>::from_untrusted_buf(good)
            .expect("canonical pack must be accepted");
        assert_eq!(ok.unpack(), map);
    }
}
