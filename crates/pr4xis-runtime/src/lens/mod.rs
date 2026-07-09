//! Bidirectional lenses between the runtime graph and its serialized forms.
//!
//! A `.prx` [`Archive`](crate::archive::Archive) has TWO serialized shapes, and
//! they are different lenses with different laws:
//!
//! - The **content-address form** ([`load`](crate::load)) — a canonical,
//!   toolchain-independent DAG-CBOR encoding whose BLAKE3 digest IS the archive's
//!   identity. This is the wire/agreement form: a peer rebinds a node only when
//!   the DAG-CBOR bytes of its definition hash to the same address.
//! - The **local cache / query form** ([`archive_lens`]) — a `rkyv` zero-copy
//!   layout for fast local materialization. Its byte layout is `rkyv`-version- and
//!   target-bound (never an address), so it is a private cache, NOT the content
//!   address. This is the runtime's analogue of the OWL leaf's `.prx.gz` rkyv
//!   envelope (`domains/.../owl/prx.rs`).
//!
//! Both are well-behaved lenses (Foster, Greenwald, Moore, Pierce & Schmitt 2007);
//! they focus on different views and therefore hold different round-trip laws.

pub mod archive_lens;
pub mod rkyv_lens;
