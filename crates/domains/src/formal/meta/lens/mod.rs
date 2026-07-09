//! Lens ontology — the bidirectional transformation `(get, put)`, the
//! well-behaved-lens laws, and the category lenses form under sequential
//! composition, made a first-class praxis ontology with runnable axioms
//! (North Star W3 slice 4; `feedback_praxis_as_compiler_self_describing`).
//!
//! The lens is the central abstraction of praxis's load / emit / projection
//! architecture, yet — unlike the codec ([`super::canonical_codec`]) — it was
//! named in no `ontology!`: `Lens`, `WellBehavedLens`, `GetPut`, `PutGet`,
//! `PutPut`, `SequentialComposition` and `RoundTripFidelity` were not
//! discoverable concepts, and the general lens *algebra* (that lenses form a
//! category — Foster et al. 2007 §3) was proven only in `#[test]` helpers,
//! registered as no axiom. This module brings the lens INTO the constitution
//! partition:
//!
//! - [`ontology`] — the nine concepts and their kinded dependency morphisms.
//!   Always compiled; the general [`Lens`](super::lens_composition::Lens)
//!   trait it describes is unconditional domains code (no `feature = "prx"`
//!   gate — unlike [`super::ontology_archive`]).
//! - [`axioms`] — the six runnable predicates that lift the general lens
//!   algebra into registered, discoverable axioms: the well-behaved-lens laws
//!   `LensGetPutLaw` / `LensPutGetLaw` / `LensPutPutLaw` and the category laws
//!   `LensCompositionWellBehaved` / `LensCompositionAssociative` /
//!   `LensIdentityUnit`, each verifying over REAL lens values with teeth.
//!
//! The byte-anchored [`WellBehavedLens`](super::well_behaved_lens) round-trip
//! (`RoundTripHarnessAllVerified`) and the archive emit/load leg
//! (`EmitLoadWellBehaved`) are owned by their realisation ontologies and are
//! NOT re-run here; the ontology's discoverability test resolves them through
//! the same registry, so the whole lens-law family answers one graph query.

pub mod axioms;
pub mod ontology;

/// Runnable lens-law axioms for the shared `PackedCsrDict` / `PackedCsrFamily`
/// zero-copy CSR representation (the five English M1 stores). Gated where the
/// zero-copy form exists.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub mod packed_csr_laws;

/// Runnable lens-law axioms for the four rich English M2 stores that are
/// instances of the shared `RkyvLens` (concept / function-word / morphology /
/// writing-system). Gated where the archived stores (and their mirror roots)
/// exist.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub mod rkyv_lens_laws;
