//! The fail-closed load gate — admit a `.prx` only after re-deriving its root
//! and checking it against the trusted root.
//!
//! This is the runtime's verify-before-interpret kernel primitive. The runtime
//! never trusts a self-asserted identity: it decodes the bytes, re-derives the
//! archive's Merkle root from the content it is about to admit, and accepts the
//! archive only if that root equals the externally-trusted root (the pin a peer
//! or the lock supplies). A tampered, stale, or mis-addressed archive is
//! refused, not loaded.
//!
//! The loaded [`Archive`] is the OPEN form — generators + relations + connections
//! as data, the free-category presentation. Rebinding it into the closed-world
//! compiled `Category` (via `FreeExtension`, driven by each connection's
//! action-on-generators) is the next layer.

use crate::address::ContentAddress;
use crate::archive::Archive;
use crate::codec::{self, CodecError};

/// Why a `.prx` failed to load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The bytes could not be decoded as a `.prx` archive.
    Decode(CodecError),
    /// The archive decoded, but its re-derived Merkle root does not match the
    /// trusted root — a tampered, stale, or wrong archive. Refused.
    RootMismatch {
        expected: ContentAddress,
        actual: ContentAddress,
    },
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadError::Decode(e) => write!(f, "decode .prx: {e}"),
            LoadError::RootMismatch { expected, actual } => write!(
                f,
                ".prx root mismatch: expected {}, got {} — refused",
                expected.to_hex(),
                actual.to_hex()
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// Emit an [`Archive`] to its canonical `.prx` bytes (DAG-CBOR). The archive's
/// identity is its [`Archive::root`], derived from the content — NOT these bytes
/// — so the gate re-derives the root on load rather than hashing the wire.
pub fn emit(archive: &Archive) -> Result<Vec<u8>, CodecError> {
    codec::canonical_encode(archive)
}

/// Load a `.prx` archive from its canonical bytes, FAIL-CLOSED against
/// `trusted_root`: decode, re-derive the Merkle root, and admit the archive only
/// if it matches. The trusted root comes from OUTSIDE the bytes (a peer's claim,
/// a lock pin) — that is what makes the check meaningful.
pub fn load(bytes: &[u8], trusted_root: ContentAddress) -> Result<Archive, LoadError> {
    let archive: Archive = codec::canonical_decode(bytes).map_err(LoadError::Decode)?;
    let actual = archive.root().map_err(LoadError::Decode)?;
    if actual != trusted_root {
        return Err(LoadError::RootMismatch {
            expected: trusted_root,
            actual,
        });
    }
    Ok(archive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{Connection, GeneratorAction};
    use crate::definition::Definition;

    fn sample() -> Archive {
        Archive {
            nodes: vec![Definition {
                kind: "Concept".into(),
                name: "Employer".into(),
                edges: vec![("Subsumption".into(), "Agent".into())],
                axioms: vec!["EmployerIsAgent".into()],
                lexical: Some("employer".into()),
            }],
            connections: vec![Connection {
                kind: "FullyFaithful".into(),
                source: "A".into(),
                target: "B".into(),
                action: GeneratorAction::Functor {
                    map_object: vec![("A".into(), "B".into())],
                    map_morphism: vec![],
                },
                laws: vec!["PreservesComposition".into()],
            }],
        }
    }

    #[test]
    fn emit_then_load_round_trips() {
        let a = sample();
        let bytes = emit(&a).unwrap();
        let loaded = load(&bytes, a.root().unwrap()).unwrap();
        assert_eq!(loaded, a);
    }

    #[test]
    fn load_refuses_a_wrong_root() {
        let a = sample();
        let bytes = emit(&a).unwrap();
        let err = load(&bytes, ContentAddress::of(b"not the root")).unwrap_err();
        assert!(matches!(err, LoadError::RootMismatch { .. }));
    }

    #[test]
    fn load_refuses_tampered_bytes() {
        let a = sample();
        let trusted = a.root().unwrap();
        let mut bytes = emit(&a).unwrap();
        // Flip a byte: either decode fails, or it decodes to a different archive
        // whose re-derived root no longer matches. Either way — refused.
        *bytes.last_mut().unwrap() ^= 0xff;
        assert!(load(&bytes, trusted).is_err());
    }

    #[test]
    fn load_refuses_garbage() {
        let err = load(b"not dag-cbor at all", ContentAddress::of(b"x")).unwrap_err();
        assert!(matches!(err, LoadError::Decode(_)));
    }

    mod adversarial {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// GATE FAIL-CLOSED PROPERTY (the `raw_source_prx`
            /// `prop_mutated_prx_always_rejected` pattern, ported to the typed
            /// public `.prx` entry): ∀ single-byte mutation of a real emitted
            /// archive's canonical bytes, `load` is TOTAL and the root
            /// re-derivation is sound — the mutation is refused (a decode `Err`
            /// or a `RootMismatch`), or, if the mutated bytes still decode to
            /// content whose re-derived root equals the trusted root, the
            /// admitted archive IS the original content (the root pins the
            /// CONTENT, not the wire bytes). Never a panic-through.
            #[test]
            fn prop_mutated_prx_refused_or_content_identical(
                byte_idx in any::<prop::sample::Index>(),
                xor in 1u8..=255,
            ) {
                let a = sample();
                let trusted = a.root().unwrap();
                let bytes = emit(&a).unwrap();
                let i = byte_idx.index(bytes.len());
                let mut bad = bytes.clone();
                bad[i] ^= xor; // a guaranteed change at a real index

                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    load(&bad, trusted)
                }));
                match res {
                    // Admitted ⇒ the gate re-derived the same root, which is
                    // only sound if the CONTENT is the original archive.
                    Ok(Ok(loaded)) => prop_assert_eq!(
                        loaded, a,
                        "mutated bytes (byte {} ^= {}) admitted DIFFERENT content — \
                         the root re-derivation is broken", i, xor
                    ),
                    Ok(Err(_)) => {} // correct: fail-closed refusal
                    Err(_) => prop_assert!(
                        false,
                        "load PANICKED on mutated bytes (byte {i} ^= {xor})"
                    ),
                }
            }

            /// TOTALITY over arbitrary bytes: `load` never panics on garbage —
            /// any byte vector is a typed `Err` (or an admitted archive whose
            /// re-derived root matches, unreachable for random bytes), never an
            /// unwind. The decoder + root gate are total.
            #[test]
            fn prop_load_is_total_over_arbitrary_bytes(
                bytes in proptest::collection::vec(any::<u8>(), 0..2048),
            ) {
                let trusted = sample().root().unwrap();
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    load(&bytes, trusted)
                }));
                prop_assert!(res.is_ok(), "load PANICKED on arbitrary bytes");
            }
        }
    }
}
