//! The English writing system as an immutable, zero-copy archive.
//!
//! `English` holds one [`WritingSystem`] — a script (its characters), a numeral
//! system, a punctuation table, and a writing direction. Held naively as an owned
//! `WritingSystem` this is a small but deep tree of owned `String`/`Vec`
//! allocations (~60 `Character`s plus digits and punctuation marks). This module
//! reclaims it exactly as [`concept_store`](super::concept_store) reclaims the
//! concept records: one `rkyv`-archived buffer, built ONCE at load, the owned tree
//! dropped.
//!
//! # The representation
//!
//! An authored `WritingSystemRecord` mirror tree (kept OFF the live
//! [`WritingSystem`] domain types per the [`concept_store`](super::concept_store) /
//! OWL `prx` precedent, so the domain stays free of `rkyv`'s layout coupling) is
//! `rkyv`-serialized into a single `AlignedVec<16>` and `bytecheck`-validated
//! ONCE at build. The one public reader — the `Language::writing_system` trait
//! method — is TEST-ONLY (grep-confirmed: no tokenizer/runtime caller), so it
//! deserializes the one small value on call and returns it by value; the archive
//! yields a structurally distinct `Archived*` type, so a `&WritingSystem` borrow is
//! impossible without materializing an owned copy anyway. Deserialize-on-call is
//! the honest, minimal choice for a single cold reader.
//!
//! # Endianness invariant
//!
//! `rkyv`'s archived layout is little-endian by construction, so the archived
//! variant is compiled only under `cfg(target_endian = "little")`; a big-endian
//! target falls back to the owned `WritingSystem`, exactly as a non-`prx` build
//! does.
//!
//! Reference: Koloski, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — `AlignedVec` + `access` / `from_bytes` are rkyv's own little-endian
//! primitives. See <https://github.com/rkyv/rkyv>.

use crate::cognitive::linguistics::orthography::WritingSystem;

/// The zero-copy, `prx`-gated writing-system store (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::WritingSystemStore;

/// The writing-system-store leaf-lens mirror root — exposed so the shared
/// lens-law axioms (`formal::meta::lens::rkyv_lens_laws`) can name it as the
/// `Mirror` of the `RkyvLens<WritingSystem, WritingSystemRecord>` instance.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::WritingSystemRecord;

/// The owned fallback store (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::WritingSystemStore;

impl core::fmt::Debug for WritingSystemStore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WritingSystemStore").finish()
    }
}

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: a plain [`WritingSystem`]. The mandatory non-`prx` (and
/// big-endian) path.
///
/// Compiled ALSO under `test` on the archived path so the cast unit tests can build
/// both representations from the same value and assert they reconstruct an
/// identical `WritingSystem`.
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    /// The writing system, held owned.
    pub struct WritingSystemStore {
        writing: WritingSystem,
    }

    impl WritingSystemStore {
        /// Retain the owned build as-is.
        pub fn build(writing: WritingSystem) -> Self {
            Self { writing }
        }

        /// The writing system (by value — matches the archived reader's signature).
        pub fn writing_system(&self) -> WritingSystem {
            self.writing.clone()
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived store: the [`WritingSystemRecord`] mirror `rkyv`-
/// serialized into one [`AlignedVec<16>`], deserialized on the cold read.
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use alloc::string::String;
    use alloc::vec::Vec;

    use rkyv::util::AlignedVec;

    use pr4xis_runtime::lens::rkyv_lens::{RkyvLens, RkyvMirror, RkyvMirrorOwned, RkyvOwned};

    use super::*;
    use crate::cognitive::linguistics::symbols::character::{
        Character, Direction, Script, UnicodeCategory,
    };
    use crate::cognitive::linguistics::symbols::numeral::{Digit, NumeralSystem};
    use crate::cognitive::linguistics::symbols::punctuation::{
        Position, PunctuationFunction, PunctuationMark,
    };

    const _: () = assert!(cfg!(target_endian = "little"));

    /// `WritingSystemStore` is `Sync`: its only field is an [`AlignedVec`] (rkyv
    /// declares it `Send + Sync`) — no interior mutability. Keeps `English`'s
    /// process-wide `OnceLock<English>` static valid.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<WritingSystemStore>();
    };

    // ── mirror records (rkyv-serializable shadows of the domain tree) ─────────

    /// Mirror of [`Character`].
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct CharacterRecord {
        pub codepoint: char,
        pub name: String,
        pub category: UnicodeCategory,
    }

    /// Mirror of [`Script`].
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct ScriptRecord {
        pub name: String,
        pub characters: Vec<CharacterRecord>,
        pub direction: Direction,
    }

    /// Mirror of [`Digit`].
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct DigitRecord {
        pub character: char,
        pub value: u32,
    }

    /// Mirror of [`NumeralSystem`].
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct NumeralSystemRecord {
        pub name: String,
        pub base: u32,
        pub digits: Vec<DigitRecord>,
    }

    /// Mirror of [`PunctuationMark`].
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct PunctuationMarkRecord {
        pub character: char,
        pub name: String,
        pub function: PunctuationFunction,
        pub position: Position,
    }

    /// Mirror of [`WritingSystem`] — the buffer's root type.
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct WritingSystemRecord {
        pub name: String,
        pub script: ScriptRecord,
        pub numerals: NumeralSystemRecord,
        pub punctuation: Vec<PunctuationMarkRecord>,
        pub direction: Direction,
    }

    // ── owned → record (build side) ───────────────────────────────────────────

    impl From<&Character> for CharacterRecord {
        fn from(c: &Character) -> Self {
            Self {
                codepoint: c.codepoint,
                name: c.name.clone(),
                category: c.category,
            }
        }
    }
    impl From<&Script> for ScriptRecord {
        fn from(s: &Script) -> Self {
            Self {
                name: s.name.clone(),
                characters: s.characters.iter().map(CharacterRecord::from).collect(),
                direction: s.direction,
            }
        }
    }
    impl From<&Digit> for DigitRecord {
        fn from(d: &Digit) -> Self {
            Self {
                character: d.character,
                value: d.value,
            }
        }
    }
    impl From<&NumeralSystem> for NumeralSystemRecord {
        fn from(n: &NumeralSystem) -> Self {
            Self {
                name: n.name.clone(),
                base: n.base,
                digits: n.digits.iter().map(DigitRecord::from).collect(),
            }
        }
    }
    impl From<&PunctuationMark> for PunctuationMarkRecord {
        fn from(p: &PunctuationMark) -> Self {
            Self {
                character: p.character,
                name: p.name.clone(),
                function: p.function,
                position: p.position,
            }
        }
    }
    impl From<&WritingSystem> for WritingSystemRecord {
        fn from(w: &WritingSystem) -> Self {
            Self {
                name: w.name.clone(),
                script: ScriptRecord::from(&w.script),
                numerals: NumeralSystemRecord::from(&w.numerals),
                punctuation: w
                    .punctuation
                    .iter()
                    .map(PunctuationMarkRecord::from)
                    .collect(),
                direction: w.direction,
            }
        }
    }

    // ── owned → record (build side, MOVE) ─────────────────────────────────────
    //
    // By-value twins of the borrowing `From<&T>` conversions above: they MOVE
    // each `name` / `characters` / `digits` / `punctuation` heap payload into the
    // record instead of cloning it, for the owned PUT leg that consumes the
    // writing-system tree at build. Each is byte-identical in result to its
    // borrowing sibling (moving preserves the value).

    impl From<Character> for CharacterRecord {
        fn from(c: Character) -> Self {
            Self {
                codepoint: c.codepoint,
                name: c.name,
                category: c.category,
            }
        }
    }
    impl From<Script> for ScriptRecord {
        fn from(s: Script) -> Self {
            Self {
                name: s.name,
                characters: s
                    .characters
                    .into_iter()
                    .map(CharacterRecord::from)
                    .collect(),
                direction: s.direction,
            }
        }
    }
    impl From<Digit> for DigitRecord {
        fn from(d: Digit) -> Self {
            Self {
                character: d.character,
                value: d.value,
            }
        }
    }
    impl From<NumeralSystem> for NumeralSystemRecord {
        fn from(n: NumeralSystem) -> Self {
            Self {
                name: n.name,
                base: n.base,
                digits: n.digits.into_iter().map(DigitRecord::from).collect(),
            }
        }
    }
    impl From<PunctuationMark> for PunctuationMarkRecord {
        fn from(p: PunctuationMark) -> Self {
            Self {
                character: p.character,
                name: p.name,
                function: p.function,
                position: p.position,
            }
        }
    }
    impl From<WritingSystem> for WritingSystemRecord {
        fn from(w: WritingSystem) -> Self {
            Self {
                name: w.name,
                script: ScriptRecord::from(w.script),
                numerals: NumeralSystemRecord::from(w.numerals),
                punctuation: w
                    .punctuation
                    .into_iter()
                    .map(PunctuationMarkRecord::from)
                    .collect(),
                direction: w.direction,
            }
        }
    }

    // ── record → owned (read side, after deserialize) ─────────────────────────

    impl From<CharacterRecord> for Character {
        fn from(c: CharacterRecord) -> Self {
            Character {
                codepoint: c.codepoint,
                name: c.name,
                category: c.category,
            }
        }
    }
    impl From<ScriptRecord> for Script {
        fn from(s: ScriptRecord) -> Self {
            Script {
                name: s.name,
                characters: s.characters.into_iter().map(Character::from).collect(),
                direction: s.direction,
            }
        }
    }
    impl From<DigitRecord> for Digit {
        fn from(d: DigitRecord) -> Self {
            Digit {
                character: d.character,
                value: d.value,
            }
        }
    }
    impl From<NumeralSystemRecord> for NumeralSystem {
        fn from(n: NumeralSystemRecord) -> Self {
            NumeralSystem {
                name: n.name,
                base: n.base,
                digits: n.digits.into_iter().map(Digit::from).collect(),
            }
        }
    }
    impl From<PunctuationMarkRecord> for PunctuationMark {
        fn from(p: PunctuationMarkRecord) -> Self {
            PunctuationMark {
                character: p.character,
                name: p.name,
                function: p.function,
                position: p.position,
            }
        }
    }
    impl From<WritingSystemRecord> for WritingSystem {
        fn from(w: WritingSystemRecord) -> Self {
            WritingSystem {
                name: w.name,
                script: Script::from(w.script),
                numerals: NumeralSystem::from(w.numerals),
                punctuation: w
                    .punctuation
                    .into_iter()
                    .map(PunctuationMark::from)
                    .collect(),
                direction: w.direction,
            }
        }
    }

    // ── leaf lens: WritingSystem ⇄ WritingSystemRecord ────────────────────────

    /// PUT leg: project the owned writing system into its record mirror (via the
    /// [`From<&WritingSystem>`](WritingSystemRecord) leaf conversion).
    impl RkyvMirror<WritingSystem> for WritingSystemRecord {
        fn from_owned(writing: &WritingSystem) -> Self {
            WritingSystemRecord::from(writing)
        }
    }

    /// Owned PUT leg: CONSUME the writing-system tree, MOVING its `String`/`Vec`
    /// payloads into the record (via the by-value [`From<WritingSystem>`] leaf
    /// conversion) instead of cloning them. Byte-identical to
    /// [`from_owned`](RkyvMirror::from_owned).
    impl RkyvMirrorOwned<WritingSystem> for WritingSystemRecord {
        fn from_owned_value(writing: WritingSystem) -> Self {
            WritingSystemRecord::from(writing)
        }
    }

    /// GET leg: rebuild the owned writing system (via the
    /// [`From<WritingSystemRecord>`](WritingSystem) leaf conversion). Total — the
    /// record → value decode cannot fail.
    impl RkyvOwned<WritingSystemRecord> for WritingSystem {
        type Error = core::convert::Infallible;
        fn from_mirror(mirror: WritingSystemRecord) -> Result<Self, core::convert::Infallible> {
            Ok(WritingSystem::from(mirror))
        }
    }

    /// The concrete lens for the writing-system store instance.
    type WritingSystemLens = RkyvLens<WritingSystem, WritingSystemRecord>;

    /// The writing system, held as one `rkyv`-archived, immutable buffer.
    pub struct WritingSystemStore {
        buf: AlignedVec<16>,
    }

    impl WritingSystemStore {
        /// Transcode the owned [`WritingSystem`] into the archived buffer ONCE (PUT
        /// through the shared [`RkyvLens`]), validated once here so the read path
        /// is sound.
        pub fn build(writing: WritingSystem) -> Self {
            // CONSUME `writing` through the OWNED PUT leg, moving its String/Vec
            // payloads into the mirror rather than cloning them (byte-identical to
            // the borrowing `put_aligned(&writing)` by `RkyvLensOwnedPutAgrees`).
            let buf = WritingSystemLens::put_aligned_owned(writing);
            WritingSystemLens::access(buf.as_slice())
                .expect("freshly-serialized writing-system record must bytecheck-validate");
            Self { buf }
            // the transient mirror drops here — only `buf` survives.
        }

        /// The archived buffer bytes — the store's complete serialized form,
        /// framed verbatim into the English store bundle.
        pub fn as_bytes(&self) -> &[u8] {
            self.buf.as_slice()
        }

        /// Recover a writing-system store from an already-serialized `rkyv`
        /// buffer this process did NOT produce (a store-bundle frame): run the
        /// ONE `bytecheck` validation pass ([`RkyvLens::access`]) the trusted
        /// `build` path runs before admitting the buffer. Fail-closed.
        pub fn from_validated_buf(buf: AlignedVec<16>) -> Result<Self, alloc::string::String> {
            WritingSystemLens::access(buf.as_slice())
                .map_err(|e| alloc::format!("writing-system store frame failed bytecheck: {e}"))?;
            Ok(Self { buf })
        }

        /// The writing system, materialized from the archive via the OWNING GET
        /// ([`RkyvLens::get`]) — the cold, test-only reader (see the
        /// [module docs](super)). Returns an owned value reconstructed
        /// field-identically to the source.
        pub fn writing_system(&self) -> WritingSystem {
            WritingSystemLens::get(self.buf.as_slice())
                .expect("the once-validated writing-system archive must deserialize")
        }
    }
}

// ── record cast unit tests (archived path) ───────────────────────────────────
//
// Direct coverage of the mirror round-trip: build the store from the KNOWN English
// writing system, assert the deserialized value is field-identical to the owned
// fallback built from the SAME value (recognizes the same characters, same
// direction, same punctuation).
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod cast_tests {
    use super::WritingSystemStore; // the archived store
    use super::owned::WritingSystemStore as OwnedStore; // the owned fallback
    use crate::cognitive::linguistics::orthography::english_writing_system;
    use crate::cognitive::linguistics::symbols::character::Direction;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_writing_system_matches_the_known_value() {
        let store = WritingSystemStore::build(english_writing_system());
        let ws = store.writing_system();
        assert_eq!(ws.name, "English");
        assert_eq!(ws.direction, Direction::LeftToRight);
        assert!(ws.recognizes('a'));
        assert!(ws.recognizes('Z'));
        assert!(ws.recognizes('5'));
        assert!(ws.recognizes('.'));
        assert!(!ws.recognizes('\u{05D0}')); // aleph — not Latin
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_is_identical_to_the_owned_fallback() {
        let archived = WritingSystemStore::build(english_writing_system());
        let owned = OwnedStore::build(english_writing_system());
        let a = archived.writing_system();
        let o = owned.writing_system();
        assert_eq!(a.name, o.name);
        assert_eq!(a.direction, o.direction);
        assert_eq!(a.script.name, o.script.name);
        assert_eq!(a.script.characters.len(), o.script.characters.len());
        assert_eq!(a.numerals.base, o.numerals.base);
        assert_eq!(a.numerals.digits.len(), o.numerals.digits.len());
        assert_eq!(a.punctuation.len(), o.punctuation.len());
        // Full structural equality (WritingSystem derives PartialEq via its tree).
        for c in ['a', 'Z', '5', '.', '?', '\u{05D0}', ' '] {
            assert_eq!(a.recognizes(c), o.recognizes(c), "recognizes {c:?}");
        }
    }
}
