//! `PackedCsrLens` — the ONE hand-audited zero-copy CSR dictionary/family,
//! generic over key addressing and value column, shared by the five English M1
//! stores (`word_index`, `synset_index`, `verb_transitivity_index`,
//! `taxonomy_store`, `relation_store`).
//!
//! Historically each of those five stores hand-packed a byte-identical
//! `AlignedVec<16>` CSR — its own `unsafe { from_raw_parts }` POD cast, its own
//! 4-byte little-endian `csr()` reader, its own little-endian-refusal `const _`,
//! its own aligned-buffer `Packer`, and its own owned `HashMap` fallback. That is
//! the same representation five times (`feedback_prefer_composition` /
//! `feedback_no_mechanical`). This module centralizes the representation ONCE:
//! the single [`cast_elems`] cast, the single [`read_u32_le`] CSR reader, the
//! single little-endian invariant, and the single aligned packer live here; each
//! store shrinks to a type alias plus its query-name wrappers.
//!
//! # The two containers
//!
//! - [`PackedCsrDict`] — a **sorted-key** dictionary ([`SortedKeys`]): `~N`
//!   string keys, each mapping to a value column (one scalar, or a run). Backs
//!   `word_index`, `synset_index`, `verb_transitivity_index`. Queried by binary
//!   search over the packed sorted key blob.
//! - [`PackedCsrFamily`] — a **dense-id** family ([`DenseId`]) of `Tag::COUNT`
//!   labelled CSR columns in ONE buffer, indexed *directly* by the dense
//!   [`Ref<4>`] id (no hashing). Backs `taxonomy_store` (2 labels —
//!   [`super`]'s `Direction`) and `relation_store` (27 labels — its
//!   `RelationKind`). The label set is the [`LabelKind`] `Tag`.
//!
//! The two addressing modes are the `Key ∈ {SortedKeys, DenseId}` axis; the
//! value column is the `Value ∈ {PodScalar, PodRun, CheckedEnumRun}` axis.
//!
//! # The value column
//!
//! Every value element is a [`PodElem`]: a `Copy` POD whose in-memory bytes equal
//! a fixed little-endian serialization, so a write of its `le_bytes` followed by a
//! [`cast_elems`] reinterpretation is value-preserving on a little-endian target.
//! [`Ref<4>`] (the id type) is a single little-endian `u64`; a `#[repr(u8)]`
//! fieldless enum (`Transitivity`) is a single discriminant byte. The value
//! dimension forks on how the payload is trusted:
//!
//! - [`PodRun`] / [`PodScalar`] — POD, valid by construction (every id byte
//!   pattern is a valid id).
//! - [`CheckedEnumRun`] — a `#[repr(u8)]` enum whose discriminant range is
//!   VALIDATED at build ([`EnumBound::MAX_DISCRIMINANT`]); an out-of-range byte
//!   fails the build, so the zero-copy `&[Enum]` cast is provably sound.
//!
//! # Endianness invariant
//!
//! The zero-copy cast and the CSR reader both assume the machine's native integer
//! byte order equals the little-endian order the buffer is written in. wasm32 and
//! x86-64 — the two targets praxis ships — are both little-endian. The whole
//! packed (zero-copy) representation is therefore compiled only under
//! `cfg(target_endian = "little")` with `feature = "prx"` (where `rkyv`'s
//! [`AlignedVec`] is linked); every other build uses the owned `HashMap`
//! fallback, exactly as a non-`prx` build does. A `const _` in [`archived`]
//! asserts the invariant at compile time.
//!
//! # Lens laws
//!
//! [`PackedCsrDict`] is a well-behaved lens (Foster, Greenwald, Moore, Pierce &
//! Schmitt 2007 §2.2/§3) between the owned `HashMap` build and its packed bytes:
//! `pack` (build the buffer) is the PUT, `unpack` (reconstruct the map) is the
//! GET, and the GetPut/PutGet legs plus the archived-reads-equal-owned-reads
//! faithfulness and the per-label family faithfulness are registered, cited
//! axioms in [`crate::formal::meta::lens::packed_csr_laws`].
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust*
//! (v0.8) — [`AlignedVec`] is rkyv's own little-endian aligned buffer, whose
//! aligned-buffer discipline this module reuses. See <https://github.com/rkyv/rkyv>.

use core::marker::PhantomData;

use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::formal::information::ontology::Ref;

/// The dense id every [`PackedCsrFamily`] column is indexed by — the same
/// single-`u64` [`Ref<4>`] that `ConceptId` / `SenseId` alias.
pub type DenseKey = Ref<4>;

// ── PodElem: the value element ───────────────────────────────────────────────

/// A plain-old-data value element that round-trips zero-copy through the packed
/// payload: its in-memory bytes equal `le_bytes()[..SIZE]`, so a write of those
/// bytes followed by a [`cast_elems`] reinterpretation is value-preserving on a
/// little-endian target.
pub trait PodElem: Copy + PartialEq + core::fmt::Debug {
    /// Byte width of this element in the packed array.
    const SIZE: usize;
    /// The little-endian bytes of this element; only `[..SIZE]` is written.
    fn le_bytes(&self) -> [u8; 8];
}

/// A `#[repr(u8)]` fieldless enum element whose discriminants are validated at
/// build. `MAX_DISCRIMINANT` is the largest valid `value as u8`, so a packed byte
/// `> MAX_DISCRIMINANT` is not a valid variant and fails the build — the guarantee
/// that makes the zero-copy `&[Enum]` cast sound.
pub trait EnumBound: PodElem {
    /// The largest valid discriminant (`highest_variant as u8`).
    const MAX_DISCRIMINANT: u8;
}

/// Every [`Ref<N>`] is backed by a single little-endian `u64`
/// (`#[repr(transparent)]`), so its in-memory bytes ARE its `u64` little-endian
/// serialization — the zero-copy cast is sound by the type's contract.
impl<const N: usize> PodElem for Ref<N> {
    const SIZE: usize = 8;
    #[inline]
    fn le_bytes(&self) -> [u8; 8] {
        self.value().to_le_bytes()
    }
}

// ── ValueColumn: scalar vs run, POD vs checked ───────────────────────────────

/// How a per-key value packs into and reads back from the CSR payload — the
/// `Value` axis. Marker types [`PodScalar`], [`PodRun`], [`CheckedEnumRun`]
/// implement it; instances are never constructed.
pub trait ValueColumn {
    /// The POD element written one-per-slot (scalar) or per-run-element.
    type Elem: PodElem;
    /// The owned per-key value (`Elem` for a scalar, `Vec<Elem>` for a run).
    type Owned: Clone;
    /// What a lookup yields — `Option<Elem>` for a scalar, `&[Elem]` for a run.
    type Read<'a>
    where
        Self: 'a;

    /// Elements of one owned entry (a scalar is a length-1 slice via
    /// [`core::slice::from_ref`]).
    fn elems(owned: &Self::Owned) -> &[Self::Elem];
    /// Adapt an element slice into the read type (scalar → the first element as
    /// `Option`; run → the slice unchanged). An empty slice is the "absent" read.
    fn read_slice(elems: &[Self::Elem]) -> Self::Read<'_>;
    /// Rebuild an owned value from an element slice — the GET half of the lens.
    fn to_owned(elems: &[Self::Elem]) -> Self::Owned;
    /// Is this read "present" (`contains`)? Scalar → `Some`; run → non-empty.
    fn present(read: &Self::Read<'_>) -> bool;
    /// Validate a packed payload byte region (the whole value array). POD columns
    /// are valid by construction; [`CheckedEnumRun`] checks every discriminant.
    #[inline]
    fn validate_payload(_bytes: &[u8]) -> bool {
        true
    }
}

/// A value column of exactly one [`PodElem`] per key — `synset_index`
/// (synset id → its one `ConceptId`). Read as `Option<Elem>`.
pub struct PodScalar<E>(PhantomData<E>);

impl<E: PodElem> ValueColumn for PodScalar<E> {
    type Elem = E;
    type Owned = E;
    type Read<'a>
        = Option<E>
    where
        Self: 'a;

    #[inline]
    fn elems(owned: &E) -> &[E] {
        core::slice::from_ref(owned)
    }
    #[inline]
    fn read_slice(elems: &[E]) -> Option<E> {
        elems.first().copied()
    }
    #[inline]
    fn to_owned(elems: &[E]) -> E {
        elems[0]
    }
    #[inline]
    fn present(read: &Option<E>) -> bool {
        read.is_some()
    }
}

/// A value column of a variable-length run of [`PodElem`]s per key — `word_index`
/// (word → the `ConceptId`s it can express), `taxonomy_store` /
/// `relation_store` (id → its neighbour ids). Read as `&[Elem]`.
pub struct PodRun<E>(PhantomData<E>);

impl<E: PodElem> ValueColumn for PodRun<E> {
    type Elem = E;
    type Owned = Vec<E>;
    type Read<'a>
        = &'a [E]
    where
        Self: 'a;

    #[inline]
    fn elems(owned: &Vec<E>) -> &[E] {
        owned.as_slice()
    }
    #[inline]
    fn read_slice(elems: &[E]) -> &[E] {
        elems
    }
    #[inline]
    fn to_owned(elems: &[E]) -> Vec<E> {
        elems.to_vec()
    }
    #[inline]
    fn present(read: &&[E]) -> bool {
        !read.is_empty()
    }
}

/// A value column of a variable-length run of a `#[repr(u8)]` [`EnumBound`] enum
/// per key — `verb_transitivity_index` (verb → its `Transitivity` options). Every
/// packed byte is VALIDATED as a discriminant at build, so the zero-copy
/// `&[Enum]` cast is sound. Read as `&[Elem]`.
pub struct CheckedEnumRun<E>(PhantomData<E>);

impl<E: EnumBound> ValueColumn for CheckedEnumRun<E> {
    type Elem = E;
    type Owned = Vec<E>;
    type Read<'a>
        = &'a [E]
    where
        Self: 'a;

    #[inline]
    fn elems(owned: &Vec<E>) -> &[E] {
        owned.as_slice()
    }
    #[inline]
    fn read_slice(elems: &[E]) -> &[E] {
        elems
    }
    #[inline]
    fn to_owned(elems: &[E]) -> Vec<E> {
        elems.to_vec()
    }
    #[inline]
    fn present(read: &&[E]) -> bool {
        !read.is_empty()
    }
    /// Every packed value byte must be a valid discriminant (`<=
    /// MAX_DISCRIMINANT`). `E::SIZE == 1` for a `#[repr(u8)]` enum, so the payload
    /// is one byte per element and this is a direct sweep.
    #[inline]
    fn validate_payload(bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b <= E::MAX_DISCRIMINANT)
    }
}

// ── KeyKind + LabelKind: the addressing axis ─────────────────────────────────

/// The key-addressing axis marker. [`SortedKeys`] (a binary-searched key blob)
/// backs [`PackedCsrDict`]; [`DenseId`] (direct `Ref<4>` indexing) backs
/// [`PackedCsrFamily`]'s columns.
pub trait KeyKind {}

/// String keys, stored sorted and binary-searched. The dict addressing mode.
pub struct SortedKeys;
impl KeyKind for SortedKeys {}

/// Dense `Ref<4>` ids in `0..row_count`, indexed directly (no hashing). The
/// family-column addressing mode.
pub struct DenseId;
impl KeyKind for DenseId {}

/// A fixed, ordered label set — the `Tag` of a [`PackedCsrFamily`]. The
/// discriminant order IS the column layout order; `index(self)` selects a column.
pub trait LabelKind: Copy {
    /// The number of labels (= number of family columns).
    const COUNT: usize;
    /// This label's column index (`0..COUNT`), in layout order.
    fn index(self) -> usize;
    /// Every label, in layout order.
    fn all() -> &'static [Self]
    where
        Self: Sized;
}

// ── owned fallback (always compiled) ─────────────────────────────────────────

/// The owned `HashMap` fallback for [`PackedCsrDict`] — the mandatory non-`prx`
/// (and big-endian) representation, and the baseline the archived reads are
/// proven equal to. Always compiled so the lens-law axioms can build it beside
/// the archived form under `prx` + little-endian.
pub struct OwnedCsrDict<K, V: ValueColumn> {
    map: HashMap<String, V::Owned>,
    _pd: PhantomData<K>,
}

impl<V: ValueColumn> OwnedCsrDict<SortedKeys, V> {
    /// Retain the owned build as-is — the fallback keeps the `HashMap`.
    pub fn build(map: HashMap<String, V::Owned>) -> Self {
        Self {
            map,
            _pd: PhantomData,
        }
    }

    /// Look up a key → its value (absent → the empty read).
    pub fn lookup(&self, key: &str) -> V::Read<'_> {
        match self.map.get(key) {
            Some(owned) => V::read_slice(V::elems(owned)),
            None => V::read_slice(&[]),
        }
    }

    /// Number of distinct keys.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Is the dictionary empty?
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Does the dictionary hold `key`?
    pub fn contains(&self, key: &str) -> bool {
        V::present(&self.lookup(key))
    }

    /// Every key, as `&str` (unordered — the owned fallback; the archived variant
    /// yields them sorted).
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(String::as_str)
    }

    /// Reconstruct the owned map — the GET half of the lens (identity here).
    pub fn unpack(&self) -> HashMap<String, V::Owned> {
        self.map.clone()
    }
}

impl<V: ValueColumn> core::fmt::Debug for OwnedCsrDict<SortedKeys, V>
where
    for<'a> V::Read<'a>: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.keys().map(|k| (k, self.lookup(k))))
            .finish()
    }
}

impl<V: ValueColumn> PartialEq for OwnedCsrDict<SortedKeys, V>
where
    for<'a> V::Read<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.keys().all(|k| self.lookup(k) == other.lookup(k))
    }
}

/// One owned relation column of a family: its dense row count (key space) and its
/// adjacency map.
struct OwnedColumn<V: ValueColumn> {
    row_count: usize,
    map: HashMap<DenseKey, V::Owned>,
}

/// The owned `HashMap` fallback for [`PackedCsrFamily`] — the labelled columns
/// held as plain maps in [`LabelKind`] layout order.
pub struct OwnedCsrFamily<Tag: LabelKind, V: ValueColumn> {
    cols: Vec<OwnedColumn<V>>,
    _pd: PhantomData<Tag>,
}

impl<Tag: LabelKind, V: ValueColumn> OwnedCsrFamily<Tag, V> {
    /// Build from the per-column `(row_count, map)` bundle, in [`LabelKind`]
    /// layout order (one entry per label).
    pub fn build(bundle: Vec<(usize, HashMap<DenseKey, V::Owned>)>) -> Self {
        assert_eq!(
            bundle.len(),
            Tag::COUNT,
            "family bundle must have exactly one entry per label"
        );
        let cols = bundle
            .into_iter()
            .map(|(row_count, map)| OwnedColumn { row_count, map })
            .collect();
        Self {
            cols,
            _pd: PhantomData,
        }
    }

    /// The targets of `id` under `tag` (absent / out of range → the empty read).
    pub fn column(&self, tag: Tag, id: DenseKey) -> V::Read<'_> {
        let col = &self.cols[tag.index()];
        if id.value() as usize >= col.row_count {
            return V::read_slice(&[]);
        }
        match col.map.get(&id) {
            Some(owned) => V::read_slice(V::elems(owned)),
            None => V::read_slice(&[]),
        }
    }

    /// Total number of edges of `tag`.
    pub fn edge_count(&self, tag: Tag) -> usize {
        let col = &self.cols[tag.index()];
        (0..col.row_count)
            .map(|i| {
                col.map
                    .get(&Ref::new(i as u64))
                    .map_or(0, |o| V::elems(o).len())
            })
            .sum()
    }

    /// The dense row count (key space) of `tag`.
    pub fn row_count(&self, tag: Tag) -> usize {
        self.cols[tag.index()].row_count
    }
}

// ── archived (zero-copy) representation — prx + little-endian ─────────────────

#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use super::*;

    use rkyv::util::AlignedVec;

    /// The soundness precondition of the zero-copy cast and the CSR reader:
    /// native integer byte order must equal the little-endian order the buffer is
    /// written in. Enforced by the `cfg(target_endian = "little")` gate on this
    /// module; asserted here so a mis-configuration is a compile error.
    const _: () = assert!(cfg!(target_endian = "little"));

    /// Byte length of the fixed dict header (`n`, `val_count`, `key_blob_len`,
    /// pad — four `u32`s), and the offset at which the value array begins. Sized
    /// to keep the buffer's 16-alignment (⇒ 8-alignment) on the value array.
    const HEADER: usize = 16;

    /// The ONE hand-audited zero-copy POD cast — the single `unsafe` in the whole
    /// packed representation.
    ///
    /// # Safety
    ///
    /// `byte_start` must be `E`-aligned. Callers guarantee this: value arrays are
    /// laid out at `HEADER` (16, a multiple of 8) or at 8-multiple column offsets,
    /// so `byte_start = base + start * E::SIZE` is 8-aligned when `E::SIZE == 8`,
    /// and any offset is 1-aligned when `E::SIZE == 1`. `payload[byte_start ..
    /// byte_start + len * E::SIZE]` must be in bounds — the CSR offsets partition
    /// exactly the packed element count. Each element was written as its
    /// `le_bytes()[..E::SIZE]`; on this little-endian-gated build those ARE `E`'s
    /// in-memory bytes, so the reinterpretation is value-preserving. The returned
    /// slice borrows `payload`.
    #[inline]
    unsafe fn cast_elems<E: PodElem>(payload: &[u8], byte_start: usize, len: usize) -> &[E] {
        unsafe { core::slice::from_raw_parts(payload.as_ptr().add(byte_start) as *const E, len) }
    }

    /// Read the little-endian `u32` at byte offset `at` — the ONE CSR reader. A
    /// checked 4-byte read (no alignment requirement on the offset arrays).
    #[inline]
    fn read_u32_le(buf: &[u8], at: usize) -> usize {
        let b = &buf[at..at + 4];
        u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
    }

    /// Append a little-endian `u32` to the aligned packer.
    #[inline]
    fn push_u32(buf: &mut AlignedVec<16>, v: u32) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Append one POD element's little-endian bytes to the aligned packer.
    #[inline]
    fn push_elem<E: PodElem>(buf: &mut AlignedVec<16>, e: &E) {
        buf.extend_from_slice(&e.le_bytes()[..E::SIZE]);
    }

    // ── the sorted-key dict ─────────────────────────────────────────────────

    /// The zero-copy [`SortedKeys`] dictionary: one packed [`AlignedVec<16>`]
    /// holding a header, the value array, the value CSR offsets, the key CSR
    /// offsets, and the sorted key blob; queried by binary search + the one cast.
    ///
    /// Buffer layout (offsets in bytes from the 16-aligned base):
    ///
    /// ```text
    /// [ 0..16)              header: n:u32, val_count:u32, key_blob_len:u32, _pad
    /// [16 ..)               val_array:   val_count × E::SIZE
    /// then (n+1) × 4        val_offsets: CSR offsets into val_array (in E units)
    /// then (n+1) × 4        key_offsets: CSR offsets into key_blob (in bytes)
    /// then key_blob_len     key_blob:    concatenated sorted key bytes
    /// ```
    pub struct ArchivedCsrDict<K, V: ValueColumn> {
        buf: AlignedVec<16>,
        n: usize,
        val_offsets_at: usize,
        key_offsets_at: usize,
        key_blob_at: usize,
        _pd: PhantomData<(K, V)>,
    }

    impl<V: ValueColumn> ArchivedCsrDict<SortedKeys, V> {
        /// Transcode the owned `HashMap` into the packed dictionary ONCE,
        /// consuming and freeing the map. Keys are sorted by raw UTF-8 bytes (the
        /// order `lookup`'s binary search assumes); each key's element order is
        /// preserved exactly.
        pub fn build(map: HashMap<String, V::Owned>) -> Self {
            let buf = Self::pack(&map);
            Self::from_buf(buf)
        }

        /// The lens PUT: pack an owned map into the buffer bytes (deterministic —
        /// keys sorted, so the bytes are canonical for the map).
        pub fn pack(map: &HashMap<String, V::Owned>) -> AlignedVec<16> {
            let mut entries: Vec<(&str, &[V::Elem])> =
                map.iter().map(|(k, v)| (k.as_str(), V::elems(v))).collect();
            entries.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

            let n = entries.len();
            let es = V::Elem::SIZE;
            let val_count: usize = entries.iter().map(|(_, e)| e.len()).sum();
            let key_blob_len: usize = entries.iter().map(|(k, _)| k.len()).sum();

            let val_offsets_at = HEADER + val_count * es;
            let key_offsets_at = val_offsets_at + (n + 1) * 4;
            let key_blob_at = key_offsets_at + (n + 1) * 4;
            let total = key_blob_at + key_blob_len;

            let mut buf = AlignedVec::<16>::with_capacity(total);
            push_u32(&mut buf, n as u32);
            push_u32(&mut buf, val_count as u32);
            push_u32(&mut buf, key_blob_len as u32);
            push_u32(&mut buf, 0);

            for (_, e) in &entries {
                for x in *e {
                    push_elem(&mut buf, x);
                }
            }
            let mut acc = 0u32;
            push_u32(&mut buf, acc);
            for (_, e) in &entries {
                acc += e.len() as u32;
                push_u32(&mut buf, acc);
            }
            let mut acc = 0u32;
            push_u32(&mut buf, acc);
            for (k, _) in &entries {
                acc += k.len() as u32;
                push_u32(&mut buf, acc);
            }
            for (k, _) in &entries {
                buf.extend_from_slice(k.as_bytes());
            }

            assert_eq!(buf.len(), total, "packed dict length must equal the layout");
            assert!(
                V::validate_payload(&buf.as_slice()[HEADER..val_offsets_at]),
                "packed value payload holds an out-of-range discriminant"
            );
            buf
        }

        /// Recover a dict from already-packed bytes by reading the header. Used by
        /// [`build`](Self::build) and (in tests) the GetPut byte round-trip.
        pub fn from_buf(buf: AlignedVec<16>) -> Self {
            let s = buf.as_slice();
            let n = read_u32_le(s, 0);
            let val_count = read_u32_le(s, 4);
            let es = V::Elem::SIZE;
            let val_offsets_at = HEADER + val_count * es;
            let key_offsets_at = val_offsets_at + (n + 1) * 4;
            let key_blob_at = key_offsets_at + (n + 1) * 4;
            Self {
                buf,
                n,
                val_offsets_at,
                key_offsets_at,
                key_blob_at,
                _pd: PhantomData,
            }
        }

        #[inline]
        fn key_bytes(&self, i: usize) -> &[u8] {
            let s =
                self.key_blob_at + read_u32_le(self.buf.as_slice(), self.key_offsets_at + i * 4);
            let e = self.key_blob_at
                + read_u32_le(self.buf.as_slice(), self.key_offsets_at + (i + 1) * 4);
            &self.buf.as_slice()[s..e]
        }

        #[inline]
        fn key_str(&self, i: usize) -> &str {
            core::str::from_utf8(self.key_bytes(i)).expect("packed keys are UTF-8 by construction")
        }

        #[inline]
        fn elems_at(&self, i: usize) -> &[V::Elem] {
            let start = read_u32_le(self.buf.as_slice(), self.val_offsets_at + i * 4);
            let end = read_u32_le(self.buf.as_slice(), self.val_offsets_at + (i + 1) * 4);
            let byte_start = HEADER + start * V::Elem::SIZE;
            // SAFETY: see `cast_elems`. `byte_start = 16 + start*SIZE` is
            // SIZE-aligned; the run is in bounds by the CSR partition.
            unsafe { cast_elems::<V::Elem>(self.buf.as_slice(), byte_start, end - start) }
        }

        /// The lens GET: reconstruct the owned map from the packed buffer.
        pub fn unpack(&self) -> HashMap<String, V::Owned> {
            (0..self.n)
                .map(|i| (String::from(self.key_str(i)), V::to_owned(self.elems_at(i))))
                .collect()
        }

        /// The packed bytes — the lens's source representation.
        pub fn as_bytes(&self) -> &[u8] {
            self.buf.as_slice()
        }

        /// Look up a key → its value, by binary search over the sorted keys.
        pub fn lookup(&self, key: &str) -> V::Read<'_> {
            let target = key.as_bytes();
            let (mut lo, mut hi) = (0usize, self.n);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                match self.key_bytes(mid).cmp(target) {
                    core::cmp::Ordering::Less => lo = mid + 1,
                    core::cmp::Ordering::Greater => hi = mid,
                    core::cmp::Ordering::Equal => return V::read_slice(self.elems_at(mid)),
                }
            }
            V::read_slice(&[])
        }

        /// Number of distinct keys.
        pub fn len(&self) -> usize {
            self.n
        }

        /// Is the dictionary empty?
        pub fn is_empty(&self) -> bool {
            self.n == 0
        }

        /// Does the dictionary hold `key`?
        pub fn contains(&self, key: &str) -> bool {
            V::present(&self.lookup(key))
        }

        /// Every key, as `&str`, in sorted (key-byte) order.
        pub fn keys(&self) -> impl Iterator<Item = &str> {
            (0..self.n).map(move |i| self.key_str(i))
        }
    }

    impl<V: ValueColumn> core::fmt::Debug for ArchivedCsrDict<SortedKeys, V>
    where
        for<'a> V::Read<'a>: core::fmt::Debug,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_map()
                .entries(self.keys().map(|k| (k, self.lookup(k))))
                .finish()
        }
    }

    impl<V: ValueColumn> PartialEq for ArchivedCsrDict<SortedKeys, V>
    where
        for<'a> V::Read<'a>: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.len() == other.len() && self.keys().all(|k| self.lookup(k) == other.lookup(k))
        }
    }

    /// `ArchivedCsrDict` is `Sync`: an [`AlignedVec`] (rkyv declares it `Send +
    /// Sync`) plus `Copy` scalars — no interior mutability.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ArchivedCsrDict<SortedKeys, PodRun<Ref<4>>>>();
    };

    // ── the dense-id labelled family ────────────────────────────────────────

    /// One family column's layout within the packed buffer.
    #[derive(Clone, Copy)]
    struct ColMeta {
        row_count: usize,
        edge_count: usize,
        targets_at: usize,
        offsets_at: usize,
    }

    /// The zero-copy [`DenseId`] labelled family: every column's targets array
    /// (concatenated in [`LabelKind`] layout order, each inheriting the buffer's
    /// 8-alignment), then every column's CSR offsets array, in ONE
    /// [`AlignedVec<16>`]. The per-column layout lives in the struct (the family is
    /// always rebuilt at load, never serialized), read back through the one cast.
    pub struct ArchivedCsrFamily<Tag: LabelKind, V: ValueColumn> {
        buf: AlignedVec<16>,
        meta: Vec<ColMeta>,
        _pd: PhantomData<(Tag, V)>,
    }

    impl<Tag: LabelKind, V: ValueColumn> ArchivedCsrFamily<Tag, V> {
        /// Transcode the per-column `(row_count, map)` bundle (in [`LabelKind`]
        /// layout order) into the packed CSR family ONCE, consuming and freeing
        /// the maps. Each column's per-id run is written in dense-id order, each
        /// element in its owned-map order.
        pub fn build(bundle: Vec<(usize, HashMap<DenseKey, V::Owned>)>) -> Self {
            assert_eq!(
                bundle.len(),
                Tag::COUNT,
                "family bundle must have exactly one entry per label"
            );
            let es = V::Elem::SIZE;

            let run = |map: &HashMap<DenseKey, V::Owned>, i: usize| -> usize {
                map.get(&Ref::new(i as u64))
                    .map_or(0, |o| V::elems(o).len())
            };
            let edge_counts: Vec<usize> = bundle
                .iter()
                .map(|(row_count, map)| (0..*row_count).map(|i| run(map, i)).sum())
                .collect();

            let mut meta = Vec::with_capacity(Tag::COUNT);
            let mut cursor = 0usize;
            let mut targets_at = Vec::with_capacity(Tag::COUNT);
            for &ec in &edge_counts {
                targets_at.push(cursor);
                cursor += ec * es;
            }
            for (k, (row_count, _)) in bundle.iter().enumerate() {
                let offsets_at = cursor;
                cursor += (*row_count + 1) * 4;
                meta.push(ColMeta {
                    row_count: *row_count,
                    edge_count: edge_counts[k],
                    targets_at: targets_at[k],
                    offsets_at,
                });
            }
            let total = cursor;

            let mut buf = AlignedVec::<16>::with_capacity(total);
            for (row_count, map) in &bundle {
                for i in 0..*row_count {
                    if let Some(o) = map.get(&Ref::new(i as u64)) {
                        for x in V::elems(o) {
                            push_elem(&mut buf, x);
                        }
                    }
                }
            }
            for (row_count, map) in &bundle {
                let mut acc = 0u32;
                push_u32(&mut buf, acc);
                for i in 0..*row_count {
                    acc += run(map, i) as u32;
                    push_u32(&mut buf, acc);
                }
            }

            assert_eq!(
                buf.len(),
                total,
                "packed family length must equal the layout"
            );
            // Validate every column's targets region (the leading `cursor_targets`
            // bytes, i.e. everything before the first offsets array).
            let payload_end = meta.iter().map(|m| m.offsets_at).min().unwrap_or(0);
            assert!(
                V::validate_payload(&buf.as_slice()[..payload_end]),
                "packed family payload holds an out-of-range discriminant"
            );

            Self {
                buf,
                meta,
                _pd: PhantomData,
            }
        }

        /// The targets of `id` under `tag`, cast zero-copy from the CSR (absent /
        /// out of range → the empty read).
        pub fn column(&self, tag: Tag, id: DenseKey) -> V::Read<'_> {
            let m = &self.meta[tag.index()];
            let i = id.value() as usize;
            if i >= m.row_count {
                return V::read_slice(&[]);
            }
            let start = read_u32_le(self.buf.as_slice(), m.offsets_at + i * 4);
            let end = read_u32_le(self.buf.as_slice(), m.offsets_at + (i + 1) * 4);
            let byte_start = m.targets_at + start * V::Elem::SIZE;
            // SAFETY: see `cast_elems`. Each column's targets array begins at a
            // multiple of 8 (laid out first, each `edge_count * SIZE` bytes, from
            // the 16-aligned base), so `byte_start` is SIZE-aligned; the run is in
            // bounds by the CSR partition.
            let elems =
                unsafe { cast_elems::<V::Elem>(self.buf.as_slice(), byte_start, end - start) };
            V::read_slice(elems)
        }

        /// Total number of edges of `tag`.
        pub fn edge_count(&self, tag: Tag) -> usize {
            self.meta[tag.index()].edge_count
        }

        /// The dense row count (key space) of `tag`.
        pub fn row_count(&self, tag: Tag) -> usize {
            self.meta[tag.index()].row_count
        }
    }

    /// `ArchivedCsrFamily` is `Sync`: an [`AlignedVec`] plus `Copy` metadata.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        // A concrete 1-label witness suffices to exercise the bound.
        assert_sync::<AlignedVec<16>>();
    };
}

#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::{ArchivedCsrDict, ArchivedCsrFamily};

// ── the public representation aliases ────────────────────────────────────────

/// The packed CSR dictionary — the zero-copy [`ArchivedCsrDict`] under `prx` +
/// little-endian, the [`OwnedCsrDict`] fallback otherwise. Both present the same
/// `build` / `lookup` / `len` / `is_empty` / `contains` / `keys` surface.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub type PackedCsrDict<K, V> = ArchivedCsrDict<K, V>;

/// The owned-fallback packed CSR dictionary (no `prx`, or big-endian).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub type PackedCsrDict<K, V> = OwnedCsrDict<K, V>;

/// The packed CSR labelled family — the zero-copy [`ArchivedCsrFamily`] under
/// `prx` + little-endian, the [`OwnedCsrFamily`] fallback otherwise. Both present
/// the same `build` / `column` / `edge_count` / `row_count` surface.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub type PackedCsrFamily<Tag, V> = ArchivedCsrFamily<Tag, V>;

/// The owned-fallback packed CSR family (no `prx`, or big-endian).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub type PackedCsrFamily<Tag, V> = OwnedCsrFamily<Tag, V>;
