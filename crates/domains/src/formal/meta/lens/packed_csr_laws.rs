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
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2/§3 — the
//!   well-behaved-lens laws (GetPut / PutGet).
//! - **Hill, D.** *rkyv: zero-copy deserialization framework for Rust*, v0.8,
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
/// dictionaries. Foster, Greenwald, Moore, Pierce & Schmitt (2007) §2.2.
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(PackedCsrGetPut, constructor);

/// PutGet leg of the `PackedCsr` well-behaved lens: `unpack(pack(m)) == m` — an
/// owned map round-trips through the packed form with its full query image (every
/// key, every run in order) intact, over run / scalar / checked-enum dictionaries.
/// Foster, Greenwald, Moore, Pierce & Schmitt (2007) §2.2.
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(PackedCsrPutGet, constructor);

/// The archived zero-copy read is faithful to the owned `HashMap` read for EVERY
/// key (dict, all three value columns) and every `(label, id)` (family) — the
/// property each store's now-deleted `*_is_identical_to_the_owned_fallback` unit
/// test asserted, proven once. The zero-copy `&[…]` cast returns exactly what the
/// owned map would. Foster et al. (2007) §2.2 (the PutGet counit at zero-copy read
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(PackedCsrZeroCopyFaithful, constructor);

/// Every family label reads its OWN column and no other's: with one distinct edge
/// `label k → [100 + k]` at id `k` per label, `column(tag_k, k) == [100 + k]` and
/// every other label at id `k` is empty. The generalization of `relation_store`'s
/// `every_kind_returns_its_own_distinct_edge` — a mislabelled column would read a
/// different label's map and fail. Foster et al. (2007) §2.2.
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
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
}
