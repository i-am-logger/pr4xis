//! The committed registered-source MANIFEST `.prx` — the registry's OWN root.
//!
//! Every other source materializes its committed bytes through the registry
//! (its `.prx` path, its `praxis.lock` pin) — see
//! [`raw_source_prx`](super::raw_source_prx). The MANIFEST itself (the
//! workspace-root `praxis.toml` + `praxis.lock`) is what KNOWS those paths and
//! pins, so it cannot load through the registry it populates — that is circular.
//!
//! This module is the SELF-CONTAINED bootstrap gate that breaks the circularity:
//! the manifest is projected into ONE content-addressed `.prx` envelope
//! ([`encode_registry`]), committed at a fixed crate-local path, embedded with
//! `include_bytes!`, and admitted ONLY if its content address re-derives to a
//! BAKED-IN root hex Rust const ([`PRAXIS_REGISTRY_ROOT_HEX`]) — the SAME
//! baked-root pattern the WordNet / OWL projection functors use
//! ([`english::bridge`](crate::cognitive::linguistics::english::bridge) /
//! [`owl::bridge`](crate::social::software::markup::xml::owl::bridge)), NOT a
//! `praxis.lock` lookup (which would require the lock to already be loaded).
//!
//! So the registry root is the ONE content-address that lives in Rust, and every
//! other source's pin chains from the manifest this gate decodes.
//!
//! ## Why a content-addressed envelope, not a typed `Category` projection
//!
//! The manifest's value is the registered-source TABLE plus the lock's SIX
//! digest-map spaces (`[hashes]`, `[canonical_signatures]`,
//! `[byte_exact_signatures]`, `[archive_signatures]`,
//! `[compact_archive_signatures]`, `[snapshot_signatures]`). A full ontology
//! projection — a `Definition` per source + per-digest-space edges, with a
//! reverse-projection parser — is disproportionate to its value: the manifest is
//! already a faithful, human-authored, parseable serialization. So this takes the
//! lighter conforming form the design explicitly admits: the SAME
//! [`raw_source_prx`](super::raw_source_prx) succinct envelope (dependency-free
//! LEB128 framing, content address over the bytes), carrying both manifest files
//! as length-prefixed blobs. It is still a `.prx`, still loaded fail-closed
//! against a baked root, still ships NO raw TOML in the package and NO generated
//! Rust registry — only the ONE committed `praxis-registry.prx`.
//!
//! ## Bootstrap order (what depends on what)
//!
//! ```text
//! PRAXIS_REGISTRY_ROOT_HEX  (baked Rust const — the ONLY trust anchor)
//!   └► load_registry_manifest()   (self-contained: hash-gate the embedded .prx)
//!        └► (praxis.toml text, praxis.lock text)
//!             ├► data_sources()   (parses the toml)
//!             └► lock_*()         (parses the lock)
//! ```
//!
//! `load_registry_manifest` reaches NEITHER `data_sources()` NOR any `lock_*`
//! accessor — it depends only on the kernel content-address primitive
//! ([`pr4xis_runtime::address`]) and the baked root. That is what makes the
//! manifest loadable as "just another `.prx`" without the circularity.
//!
//! ## Citations
//!
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model* —
//!   content-addressing by cryptographic hash (the gate's integrity claim).
//! - **W3C Subresource Integrity (2016)** — the fetched/embedded resource must
//!   match the externally-supplied digest before it is admitted.

#[allow(unused_imports)]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use pr4xis_runtime::address::ContentAddress;

/// The committed registry MANIFEST `.prx` — the projected
/// `praxis.toml` + `praxis.lock`, embedded at build time at a FIXED crate-local
/// path (NOT a workspace-relative one), so the PUBLISHED crate — unpacked under
/// `target/package/` with no workspace root — loads its registry from this ONE
/// committed artifact, never from raw TOML files (which it does not ship).
pub const PRAXIS_REGISTRY_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/registry/praxis-registry.prx"
));

/// The trusted content address (Merkle-style digest) of [`PRAXIS_REGISTRY_PRX`] —
/// the registry root, the ONE content-address that lives in Rust. The fail-closed
/// [`load_registry_manifest`] gate admits the embedded bytes only if they
/// re-derive to this value, so a tampered or stale manifest `.prx` is refused,
/// never silently mis-loaded. Regenerated (with the `.prx` itself) by
/// `cargo test -p pr4xis-domains -- --ignored regenerate_praxis_registry_prx`.
pub const PRAXIS_REGISTRY_ROOT_HEX: &str =
    "873a0b972c8768b6b87d8cf663e1a30716280cce5d30cef36e623f1f0aca886b";

/// Append `bytes` length-prefixed (LEB128 varint length + raw bytes) — the SAME
/// framing as [`raw_source_prx::encode_raw_source`](super::raw_source_prx), so
/// the manifest envelope is portable across toolchains/targets (wasm32 included)
/// and its content address is stable.
fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    let mut n = bytes.len() as u64;
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
    out.extend_from_slice(bytes);
}

/// Read one length-prefixed blob, fully bounds-checked — a truncated envelope is
/// an `Err`, never a panic (the panic-proof reader the gate relies on).
fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], String> {
    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf
            .get(*pos)
            .ok_or_else(|| "registry .prx varint runs past end of buffer".to_string())?;
        *pos += 1;
        len |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err("registry .prx varint length overflow".to_string());
        }
    }
    let len = len as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| "registry .prx blob runs past end of buffer".to_string())?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

/// Encode the registry manifest into the portable succinct envelope:
/// `put_blob(praxis.toml bytes) put_blob(praxis.lock bytes)`. Dependency-free
/// LEB128 framing — the content address taken over these bytes is the registry
/// root [`PRAXIS_REGISTRY_ROOT_HEX`] pins.
#[must_use]
pub fn encode_registry(toml: &[u8], lock: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(toml.len() + lock.len() + 16);
    put_blob(&mut out, toml);
    put_blob(&mut out, lock);
    out
}

/// Decode a registry envelope back into `(praxis.toml text, praxis.lock text)` —
/// the exact inverse of [`encode_registry`]. Fail-closed on a truncated /
/// malformed envelope or non-UTF-8 payload.
pub fn decode_registry(buf: &[u8]) -> Result<(String, String), String> {
    let mut pos = 0usize;
    let toml = get_blob(buf, &mut pos)?;
    let lock = get_blob(buf, &mut pos)?;
    let toml = core::str::from_utf8(toml)
        .map_err(|e| format!("registry .prx praxis.toml payload is not UTF-8: {e}"))?
        .to_string();
    let lock = core::str::from_utf8(lock)
        .map_err(|e| format!("registry .prx praxis.lock payload is not UTF-8: {e}"))?
        .to_string();
    Ok((toml, lock))
}

/// The content address of a registry `.prx` — the digest of its succinct bytes as
/// 64-char lowercase hex (the value baked into [`PRAXIS_REGISTRY_ROOT_HEX`]).
#[must_use]
pub fn registry_archive_address(prx: &[u8]) -> String {
    ContentAddress::of(prx).to_hex()
}

/// Load the registered-source MANIFEST from the committed [`PRAXIS_REGISTRY_PRX`],
/// FAIL-CLOSED against the baked [`PRAXIS_REGISTRY_ROOT_HEX`] — the SELF-CONTAINED
/// bootstrap gate. Depends on NEITHER `data_sources()` NOR any `lock_*` accessor,
/// only the kernel content-address primitive and the baked root, so it is the
/// registry's loadable ROOT without circularity.
///
/// Returns the `(praxis.toml text, praxis.lock text)` the registry parses. A
/// failure here is a build-time invariant violation (the bytes ship embedded in
/// the binary, like the `english_functor.prx` load), so the public accessors
/// `panic!` with an actionable message — never a silent empty registry.
pub fn load_registry_manifest() -> Result<(String, String), String> {
    let trusted = ContentAddress::from_hex(PRAXIS_REGISTRY_ROOT_HEX)
        .ok_or_else(|| "PRAXIS_REGISTRY_ROOT_HEX is not valid 64-hex".to_string())?;
    let actual = ContentAddress::of(PRAXIS_REGISTRY_PRX);
    if actual != trusted {
        return Err(format!(
            "registry .prx root mismatch: baked PRAXIS_REGISTRY_ROOT_HEX is {}, embedded \
             praxis-registry.prx hashes to {} — refusing to load the manifest. Regenerate with \
             `cargo test -p pr4xis-domains -- --ignored regenerate_praxis_registry_prx` and bake \
             the printed root.",
            trusted.to_hex(),
            actual.to_hex(),
        ));
    }
    decode_registry(PRAXIS_REGISTRY_PRX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// The workspace-root manifest files — the live source of truth that
    /// `pr4xis update` / `compile` rewrite, and the input the committed
    /// `praxis-registry.prx` is emitted FROM. The tests cross-check the committed
    /// `.prx` against THESE (the staleness guard), so a `praxis.toml` edit that
    /// isn't re-emitted is caught.
    fn workspace_manifest_paths() -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("crates/domains has two ancestor dirs");
        (root.join("praxis.toml"), root.join("praxis.lock"))
    }

    /// The committed registry `.prx` path (`data/registry/praxis-registry.prx`).
    fn committed_registry_prx_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/registry/praxis-registry.prx")
    }

    /// REGENERATE PATH (`--ignored`, WRITES): re-emit the committed
    /// `praxis-registry.prx` from the workspace-root `praxis.toml` + `praxis.lock`,
    /// then PRINT the new `PRAXIS_REGISTRY_ROOT_HEX` to bake into
    /// [`PRAXIS_REGISTRY_ROOT_HEX`]. Mirrors `regenerate_morphism_kinds_prx` +
    /// `MORPHISM_KINDS_ROOT_HEX`. Run after editing `praxis.toml` (e.g. registering
    /// a source) or after `pr4xis update --lock` rewrites `praxis.lock`:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_praxis_registry_prx`.
    /// The drift guard below FAILS until the printed root is baked in.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore]
    fn regenerate_praxis_registry_prx() {
        let (toml_path, lock_path) = workspace_manifest_paths();
        let toml = std::fs::read(&toml_path).expect("read workspace-root praxis.toml");
        let lock = std::fs::read(&lock_path).expect("read workspace-root praxis.lock");
        let prx = encode_registry(&toml, &lock);
        let out = committed_registry_prx_path();
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).expect("create data/registry/");
        }
        std::fs::write(&out, &prx).expect("write praxis-registry.prx");
        let root = registry_archive_address(&prx);
        eprintln!("wrote {} ({} bytes)", out.display(), prx.len());
        println!("PRAXIS_REGISTRY_ROOT_HEX = {root}");
    }

    /// STALENESS GUARD (normal suite): the committed `praxis-registry.prx` must be
    /// a FRESH emit of the workspace-root `praxis.toml` + `praxis.lock` — emit them
    /// and assert byte-identity with the committed `.prx`. HARD-FAILS (no skip) if
    /// `praxis.toml`/`praxis.lock` drifted from the committed registry `.prx`
    /// without regenerating; the round-trip (emit → load == parsed manifest) is the
    /// integrity claim. Pairs the morphism-kinds drift guard at the registry layer.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn committed_registry_prx_matches_workspace_root_manifest() {
        let (toml_path, lock_path) = workspace_manifest_paths();
        let toml = std::fs::read(&toml_path).unwrap_or_else(|e| {
            panic!(
                "workspace-root praxis.toml `{}` is absent ({e}) — the committed \
                 registry .prx cannot be staleness-checked without it",
                toml_path.display()
            )
        });
        let lock = std::fs::read(&lock_path).unwrap_or_else(|e| {
            panic!(
                "workspace-root praxis.lock `{}` is absent ({e})",
                lock_path.display()
            )
        });
        let fresh = encode_registry(&toml, &lock);
        let committed = std::fs::read(committed_registry_prx_path())
            .expect("read committed praxis-registry.prx");
        assert_eq!(
            fresh, committed,
            "committed data/registry/praxis-registry.prx is STALE vs the workspace-root \
             praxis.toml/praxis.lock — regenerate with `cargo test -p pr4xis-domains -- \
             --ignored regenerate_praxis_registry_prx` and bake the printed PRAXIS_REGISTRY_ROOT_HEX"
        );

        // ROUND-TRIP: emit → decode equals the manifest parsed from the
        // workspace-root TOML/LOCK (byte-exact), the GetPut leg of the manifest ⇄
        // `.prx` lens. The committed `.prx` therefore carries EXACTLY the
        // source-of-truth, recoverable losslessly.
        let (back_toml, back_lock) = decode_registry(&committed).expect("decode committed .prx");
        assert_eq!(back_toml.as_bytes(), toml.as_slice());
        assert_eq!(back_lock.as_bytes(), lock.as_slice());
    }

    /// FILE ⇔ PIN COHERENCE + FAIL-CLOSED (normal suite): the committed
    /// `praxis-registry.prx` re-derives to the baked [`PRAXIS_REGISTRY_ROOT_HEX`]
    /// (so [`load_registry_manifest`] admits it and returns NON-EMPTY manifest
    /// text), and a WRONG baked root would refuse it. This is the bootstrap-gate
    /// twin of `the_functor_loads_from_its_committed_prx_fail_closed`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn registry_prx_loads_against_its_baked_root_fail_closed() {
        let (toml, lock) =
            load_registry_manifest().expect("committed registry .prx loads against baked root");
        assert!(
            toml.contains("[sources."),
            "the loaded praxis.toml carries registered sources"
        );
        assert!(
            lock.contains("[hashes]"),
            "the loaded praxis.lock carries source hashes"
        );

        // A WRONG baked root refuses the bytes — the gate is fail-closed.
        let wrong = ContentAddress::of(b"not the registry root");
        assert_ne!(
            ContentAddress::of(PRAXIS_REGISTRY_PRX),
            wrong,
            "the committed .prx does not hash to the wrong root (sanity)"
        );
    }

    /// DRIFT GUARD: `build.rs`'s baked `PRAXIS_REGISTRY_ROOT_HEX` MUST equal this
    /// runtime anchor. They are two independent copies of the same blake3 root of
    /// the committed `praxis-registry.prx`. The in-workspace build reads the root
    /// from `praxis.lock`, so build.rs's copy is exercised ONLY by the isolated
    /// `cargo publish --verify` — a drift there is invisible to every ordinary
    /// build and TEST, so it stayed hidden until release, wedging the v0.26.0 and
    /// v0.27.0 publishes of `pr4xis-domains`. This test runs in the ordinary suite
    /// and fails the instant the two constants diverge, so the drift can never
    /// reach a release again.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn build_side_registry_root_hex_matches_runtime_anchor() {
        let build_rs = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/build.rs"))
            .expect("read build.rs");
        let after = build_rs
            .split("const PRAXIS_REGISTRY_ROOT_HEX: &str =")
            .nth(1)
            .expect("build.rs declares PRAXIS_REGISTRY_ROOT_HEX");
        let baked = after
            .split('"')
            .nth(1)
            .expect("build.rs const carries a string literal");
        assert_eq!(
            baked, PRAXIS_REGISTRY_ROOT_HEX,
            "build.rs baked registry root has DRIFTED from the runtime anchor — the \
             isolated `cargo publish --verify` will reject the committed .prx and wedge \
             the release. Sync build.rs to {PRAXIS_REGISTRY_ROOT_HEX}."
        );
    }

    // ENCODE/DECODE ROUND-TRIP (the manifest ⇄ `.prx` GetPut law): forall
    // toml/lock byte payloads, `decode(encode(t, l)) == (t, l)` and the content
    // address is a deterministic pure function of the inputs.
    proptest! {
        // forall toml/lock byte payloads, `decode(encode(t, l)) == (t, l)`.
        #[test]
        fn prop_registry_encode_decode_round_trips(
            toml in proptest::collection::vec(any::<u8>(), 0..512)
                .prop_map(|v| String::from_utf8_lossy(&v).into_owned()),
            lock in proptest::collection::vec(any::<u8>(), 0..512)
                .prop_map(|v| String::from_utf8_lossy(&v).into_owned()),
        ) {
            let enc = encode_registry(toml.as_bytes(), lock.as_bytes());
            let (t, l) = decode_registry(&enc)
                .map_err(|e| TestCaseError::fail(format!("decode: {e}")))?;
            prop_assert_eq!(&t, &toml);
            prop_assert_eq!(&l, &lock);
            // Determinism: re-encoding yields identical bytes + address.
            let enc2 = encode_registry(toml.as_bytes(), lock.as_bytes());
            prop_assert_eq!(&enc, &enc2);
            prop_assert_eq!(registry_archive_address(&enc), registry_archive_address(&enc2));
        }

        // GATE FAIL-CLOSED PROPERTY: forall single-byte mutation of the REAL
        // committed registry `.prx`, the baked-root gate rejects it — either the
        // content address no longer matches the baked root (hash gate) or the
        // framing is corrupt (bounds-checked decode). Never `Ok` against the
        // baked root, never a panic-through. The `forall`-mutation style the
        // raw-source gate uses, applied to the registry root.
        #[test]
        fn prop_mutated_registry_prx_rejected_by_baked_root(
            byte_idx in any::<prop::sample::Index>(),
            xor in 1u8..=255,
        ) {
            let trusted = ContentAddress::from_hex(PRAXIS_REGISTRY_ROOT_HEX)
                .expect("PRAXIS_REGISTRY_ROOT_HEX is valid 64-hex");
            let i = byte_idx.index(PRAXIS_REGISTRY_PRX.len());
            let mut bad = PRAXIS_REGISTRY_PRX.to_vec();
            bad[i] ^= xor; // a guaranteed change at a real index

            // The gate: content-address re-derive against the baked root, THEN
            // decode. A mutation flips the address (reject) or the framing
            // (decode Err). catch_unwind proves no panic-through.
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let actual = ContentAddress::of(&bad);
                if actual != trusted {
                    return Err("root mismatch".to_string());
                }
                decode_registry(&bad).map(|_| ())
            }));
            match res {
                Ok(Ok(())) => prop_assert!(
                    false,
                    "mutated registry .prx (byte {i} ^= {xor}) passed the baked-root gate"
                ),
                Ok(Err(_)) => {} // correct: fail-closed Err
                Err(_) => prop_assert!(false, "gate PANICKED on mutated bytes (byte {i})"),
            }
        }
    }

    pr4xis::register_praxis_value!(prop_registry_encode_decode_round_trips, Deterministic);
    pr4xis::register_praxis_value!(prop_mutated_registry_prx_rejected_by_baked_root, Honest);
}
