//! `RkyvLens<Owned, Mirror>` — the generic `rkyv` local-cache/query lens.
//!
//! This is the generalization of [`ArchiveLens`](super::archive_lens::ArchiveLens):
//! every hand-authored `rkyv` mirror store in the workspace — the runtime
//! [`Archive`](crate::archive::Archive) itself AND the four rich English M2
//! stores in `pr4xis-domains` (`concept_store`, `function_word_store`,
//! `morphology_store`, `writing_system_store`) — is an INSTANCE of this one
//! lens. Each instance supplies only the irreducible *leaf lens*: a
//! purpose-built `*Record` mirror tree plus the two conversions
//! [`RkyvMirror::from_owned`] (the PUT leg, owned → mirror) and
//! [`RkyvOwned::from_mirror`] (the GET leg, mirror → owned, FALLIBLE in
//! general). The `rkyv` serialize / `bytecheck`-validate-once / zero-copy access
//! / owning-deserialize boilerplate lives here, once.
//!
//! ## Two serialized forms — this is NOT the content-address form
//!
//! Exactly as [`ArchiveLens`](super::archive_lens) documents: the `rkyv` byte
//! layout is `rkyv`-version- and target-bound, so it is a private local cache,
//! never a content address (that stays DAG-CBOR in [`crate::load`]). The lens
//! trades address stability for zero-copy access speed on the local query path.
//!
//! ## The leaf lens — a hand-authored mirror, per project precedent
//!
//! The `*Record` mirror + its two conversions are NOT a facade leak: they are
//! the irreducible per-store leaf lens (the OWL `prx.rs` / `ArchiveLens`
//! precedent — a purpose-built serializable shadow keeps the address-bearing /
//! layout-free domain type free of `rkyv`'s coupling). What generalizes is
//! everything ABOVE the leaf: PUT (serialize the mirror to a 16-aligned buffer),
//! GET (re-align, `bytecheck`-validate, materialize the owned value fail-closed),
//! ACCESS (validate-once, borrow the archived view zero-copy).
//!
//! ## Lens laws
//!
//! `put`/`get` form a well-behaved lens (Foster, Greenwald, Moore, Pierce &
//! Schmitt 2007, "Combinators for Bidirectional Tree Transformations", *ACM
//! TOPLAS* 29(3) §2.2) between an owned value and its `rkyv` bytes. The three
//! runnable predicates [`getput_holds`], [`putget_holds`] and
//! [`determinism_holds`] verify the GetPut leg, the PutGet leg, and the
//! determinism of PUT (a later law underwriting GetPut) over a per-instance
//! witness corpus; the registered [`Axiom`](pr4xis::ontology::Axiom)s that wrap
//! them live at each instance (the runtime `Archive` in
//! [`super::archive_lens`], the four stores in
//! `pr4xis_domains::formal::meta::lens::rkyv_lens_laws`), so the whole lens-law
//! family resolves through the one Lens-ontology registry.
//!
//! ## Citations
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2 — the lens
//!   laws (GetPut / PutGet).
//! - **Hill, D.** *rkyv: zero-copy deserialization framework for Rust*, v0.8,
//!   <https://github.com/rkyv/rkyv>.

/// The PUT leg of a leaf lens: build a serializable `Mirror` from a borrowed
/// owned value. Infallible — projecting an owned value into its mirror shadow
/// never fails (it may clone, drop an index-derived field, or hex-encode an
/// opaque address, but it always succeeds).
pub trait RkyvMirror<Owned> {
    /// Project `owned` into its `rkyv` mirror.
    fn from_owned(owned: &Owned) -> Self;
}

/// The GET leg of a leaf lens: rebuild an owned value from a materialized
/// `Mirror`. FALLIBLE in general — a mirror can carry a lossy encoding whose
/// decode has a failure mode (e.g. a grounded-atom hex that must parse back to a
/// content address); such an instance fails closed rather than fabricating a
/// value. Instances with a total decode set `Error = core::convert::Infallible`.
pub trait RkyvOwned<Mirror>: Sized {
    /// The failure mode of the decode leg (`Infallible` for total decodes).
    type Error;
    /// Rebuild the owned value from `mirror`, failing closed on a malformed one.
    fn from_mirror(mirror: Mirror) -> Result<Self, Self::Error>;
}

/// Why an [`RkyvLens`] GET / ACCESS refused a blob. Fail-closed: `get` returns
/// the owned value only when the bytes both `bytecheck`-validate AND the leaf
/// [`RkyvOwned::from_mirror`] decode succeeds. Generalizes
/// `ArchiveLensError`, splitting its `Rkyv` leg (owned here) from the
/// instance-specific conversion leg (carried in [`Conversion`](Self::Conversion)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RkyvLensError<E> {
    /// `rkyv` deserialization or `bytecheck` validation failed — a corrupted,
    /// truncated, or misaligned blob.
    Rkyv(String),
    /// The leaf [`RkyvOwned::from_mirror`] decode failed — a well-formed archive
    /// carrying an un-decodable payload (fail-closed, never fabricated).
    Conversion(E),
}

impl<E: core::fmt::Display> core::fmt::Display for RkyvLensError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RkyvLensError::Rkyv(m) => write!(f, "rkyv lens error: {m}"),
            RkyvLensError::Conversion(e) => write!(f, "rkyv lens conversion error: {e}"),
        }
    }
}

impl<E: core::fmt::Debug + core::fmt::Display> std::error::Error for RkyvLensError<E> {}

/// The generic `rkyv` local-cache/query lens between an `Owned` value and its
/// zero-copy `Mirror` bytes. A ZST carrier for the associated PUT / GET / ACCESS
/// functions — never instantiated; used only as `RkyvLens::<O, M>::put(..)`. See
/// the [module docs](self) for why this is NOT the content-address form.
pub struct RkyvLens<Owned, Mirror> {
    #[allow(dead_code)]
    _marker: core::marker::PhantomData<(Owned, Mirror)>,
}

impl<Owned, Mirror> RkyvLens<Owned, Mirror>
where
    Mirror: RkyvMirror<Owned>
        + for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<Mirror>: rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
        + rkyv::Deserialize<Mirror, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
    Owned: RkyvOwned<Mirror>,
{
    /// The lens PUT, keeping `rkyv`'s own 16-aligned buffer: project the owned
    /// value into its mirror and `rkyv`-serialize it into the local cache/query
    /// bytes as an [`AlignedVec<16>`](rkyv::util::AlignedVec) — the alignment
    /// [`access`](Self::access) / [`access_unchecked`](Self::access_unchecked)
    /// require. **Not** the DAG-CBOR content-address form.
    ///
    /// Infallible for these owned mirror types: `rkyv`'s default serializer over
    /// `String`/`Vec`/`Option`/tuple/enum data has no fallible leg, so a
    /// serialization error here would be a `rkyv` bug, not a data condition.
    pub fn put_aligned(owned: &Owned) -> rkyv::util::AlignedVec<16> {
        let mirror = Mirror::from_owned(owned);
        rkyv::to_bytes::<rkyv::rancor::Error>(&mirror)
            .expect("rkyv serialization of the owned mirror is infallible")
    }

    /// The lens PUT as a plain `Vec<u8>` — [`put_aligned`](Self::put_aligned)
    /// with the alignment guarantee dropped, for callers that only round-trip the
    /// bytes through [`get`](Self::get) (which re-aligns) or compare them (the
    /// lens-law predicates). The zero-copy query path uses `put_aligned` instead.
    pub fn put(owned: &Owned) -> Vec<u8> {
        Self::put_aligned(owned).to_vec()
    }

    /// The ZERO-COPY GET: `bytecheck`-validate `bytes` and return a borrowed
    /// [`rkyv::Archived<Mirror>`] over them — NO owned rebuild (contrast
    /// [`get`](Self::get)). `bytes` must be 16-aligned (an
    /// [`AlignedVec<16>`](rkyv::util::AlignedVec) as
    /// [`put_aligned`](Self::put_aligned) produces). Fail-closed on a corrupted /
    /// truncated / misaligned blob, so the returned view never borrows unsound
    /// bytes. Validate ONCE here at materialize, then serve every hot query
    /// through [`access_unchecked`](Self::access_unchecked).
    pub fn access(bytes: &[u8]) -> Result<&rkyv::Archived<Mirror>, RkyvLensError<Owned::Error>> {
        rkyv::access::<rkyv::Archived<Mirror>, rkyv::rancor::Error>(bytes)
            .map_err(|e| RkyvLensError::Rkyv(e.to_string()))
    }

    /// The ZERO-COPY GET without re-validation — the hot query path. Returns a
    /// borrowed [`rkyv::Archived<Mirror>`] over `bytes` with no `bytecheck` pass.
    ///
    /// # Safety
    ///
    /// `bytes` must be a 16-aligned buffer previously accepted by
    /// [`access`](Self::access) (bytecheck-validated) and kept immutable since.
    /// This is the deliberate `access_unchecked` an instance uses to pay
    /// bytecheck exactly once: validate at materialize, never mutate, then every
    /// query-path call is sound.
    pub unsafe fn access_unchecked(bytes: &[u8]) -> &rkyv::Archived<Mirror> {
        // SAFETY: forwarded to the caller's contract above — validated once,
        // immutable since.
        unsafe { rkyv::access_unchecked::<rkyv::Archived<Mirror>>(bytes) }
    }

    /// The lens GET: `bytecheck`-validate the bytes and materialize the owned
    /// value. Copies into a 16-aligned buffer first (a fetched/mmapped `&[u8]`
    /// carries no alignment guarantee), then `rkyv::from_bytes` validates before
    /// materializing, and the leaf [`RkyvOwned::from_mirror`] decode runs — so a
    /// corrupted/truncated blob OR an un-decodable payload fails closed rather
    /// than producing an unsound or fabricated value.
    pub fn get(bytes: &[u8]) -> Result<Owned, RkyvLensError<Owned::Error>> {
        let mut aligned = rkyv::util::AlignedVec::<16>::new();
        aligned.extend_from_slice(bytes);
        let mirror = rkyv::from_bytes::<Mirror, rkyv::rancor::Error>(&aligned)
            .map_err(|e| RkyvLensError::Rkyv(e.to_string()))?;
        Owned::from_mirror(mirror).map_err(RkyvLensError::Conversion)
    }
}

// =============================================================================
// Generic lens-law predicates — proven ONCE, run per-instance over a witness
// corpus (the generalization of `ArchiveLens`'s `witness_archives`). Pure
// `bool` — no `pr4xis` dependency; the registered `Axiom`s that wrap these into
// `Verdict`s (with citation + discoverability) live at each instance.
// =============================================================================

/// GetPut leg: for bytes `b` canonically produced by [`RkyvLens::put`],
/// `put(get(b)) == b` — the serialized cache blob is stable under a
/// decode/re-encode round-trip. Foster et al. (2007) §2.2.
pub fn getput_holds<Owned, Mirror>(witnesses: &[Owned]) -> bool
where
    Mirror: RkyvMirror<Owned>
        + for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<Mirror>: rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
        + rkyv::Deserialize<Mirror, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
    Owned: RkyvOwned<Mirror>,
{
    for owned in witnesses {
        let b = RkyvLens::<Owned, Mirror>::put(owned);
        let Ok(decoded) = RkyvLens::<Owned, Mirror>::get(&b) else {
            return false;
        };
        if RkyvLens::<Owned, Mirror>::put(&decoded) != b {
            return false;
        }
    }
    true
}

/// PutGet leg: `get(put(o)) == o` — an owned value round-trips through the
/// `rkyv` cache form with its full query image intact. Foster et al. (2007)
/// §2.2.
pub fn putget_holds<Owned, Mirror>(witnesses: &[Owned]) -> bool
where
    Mirror: RkyvMirror<Owned>
        + for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<Mirror>: rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
        + rkyv::Deserialize<Mirror, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
    Owned: RkyvOwned<Mirror> + PartialEq,
{
    for owned in witnesses {
        match RkyvLens::<Owned, Mirror>::get(&RkyvLens::<Owned, Mirror>::put(owned)) {
            Ok(decoded) if &decoded == owned => {}
            _ => return false,
        }
    }
    true
}

/// Determinism of PUT: `put(o) == put(o)` — the serialized bytes are a
/// deterministic function of the owned value alone (no build-order or address
/// nondeterminism). This underwrites GetPut: a stable decode/re-encode requires
/// PUT be a function. Foster et al. (2007) §2.2 (the well-behaved-lens PUT is a
/// function of its arguments).
pub fn determinism_holds<Owned, Mirror>(witnesses: &[Owned]) -> bool
where
    Mirror: RkyvMirror<Owned>
        + for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<Mirror>: rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
        + rkyv::Deserialize<Mirror, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
    Owned: RkyvOwned<Mirror>,
{
    for owned in witnesses {
        if RkyvLens::<Owned, Mirror>::put(owned) != RkyvLens::<Owned, Mirror>::put(owned) {
            return false;
        }
    }
    true
}
