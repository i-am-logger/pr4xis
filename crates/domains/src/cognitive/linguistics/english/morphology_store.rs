//! The English morphological rule set as an immutable, zero-copy archive.
//!
//! `English` holds 17 [`MorphologicalRule`]s (from
//! [`english_rules`](crate::cognitive::linguistics::morphology::english::english_rules))
//! — an affix (with its text) plus the input/output POS and semantic effect. Held
//! naively as an owned `Vec<MorphologicalRule>` this is a small owned collection;
//! this module reclaims it as one `rkyv`-archived buffer, matching the
//! [`concept_store`](super::concept_store) discipline.
//!
//! # The representation
//!
//! An authored [`MorphologicalRuleRecord`] mirror tree (kept OFF the live
//! [`MorphologicalRule`] domain types per the
//! [`concept_store`](super::concept_store) / OWL `prx` precedent) is
//! `rkyv`-serialized into a single [`AlignedVec<16>`], `bytecheck`-validated ONCE
//! at build.
//!
//! # Reads — one warm, one cold
//!
//! * [`suffix_texts`](archived::MorphologyStore::suffix_texts) — the WARM reader:
//!   the stemming loop in
//!   [`lexical_lookup_all`](super::ontology::English) iterates the suffix affixes
//!   and strips them from the surface. This reads the archive ZERO-COPY (each
//!   suffix's `text` as an [`ArchivedString`](rkyv::string::ArchivedString)
//!   `&str`), no allocation, byte-identical stripping.
//! * [`rules`](archived::MorphologyStore::rules) — the COLD reader: the
//!   [`Language::morphological_rules`] trait method (TEST-ONLY — grep-confirmed no
//!   tokenizer/runtime caller). It deserializes the 17 records; the archive yields
//!   a structurally distinct `Archived*` type, so a `&[MorphologicalRule]` borrow
//!   is impossible without an owned copy anyway. This is why the trait signature is
//!   `-> Vec<MorphologicalRule>`, not `-> &[…]`.
//!
//! # Endianness invariant
//!
//! `rkyv`'s archived layout is little-endian by construction; the archived variant
//! is compiled only under `cfg(target_endian = "little")` (a big-endian target
//! falls back to the owned `Vec`).
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — see <https://github.com/rkyv/rkyv>.

use alloc::vec::Vec;

use crate::cognitive::linguistics::morphology::MorphologicalRule;

/// The zero-copy, `prx`-gated morphology store (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::MorphologyStore;

/// The morphology-store leaf-lens mirror root — exposed so the shared lens-law
/// axioms (`formal::meta::lens::rkyv_lens_laws`) can name it as the `Mirror` of
/// the `RkyvLens<Vec<MorphologicalRule>, MorphologicalRuleRecords>` instance.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::MorphologicalRuleRecords;

/// The owned fallback store (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::MorphologyStore;

impl core::fmt::Debug for MorphologyStore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MorphologyStore")
            .field("len", &self.rules().len())
            .finish()
    }
}

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: a plain `Vec<MorphologicalRule>`. The mandatory non-`prx`
/// (and big-endian) path.
///
/// Compiled ALSO under `test` on the archived path so the cast unit tests can build
/// both representations from the same rules and assert identical reads.
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    use crate::cognitive::linguistics::morphology::Affix;

    /// The morphological rules, held owned.
    pub struct MorphologyStore {
        rules: Vec<MorphologicalRule>,
    }

    impl MorphologyStore {
        /// Retain the owned build as-is.
        pub fn build(rules: Vec<MorphologicalRule>) -> Self {
            Self { rules }
        }

        /// Every rule (by value — matches the archived reader's signature).
        pub fn rules(&self) -> Vec<MorphologicalRule> {
            self.rules.clone()
        }

        /// The suffix affix texts, in rule order — the warm stemming-loop reader
        /// (borrowed `&str`, matching the archived reader's signature).
        pub fn suffix_texts(&self) -> impl Iterator<Item = &str> {
            self.rules.iter().filter_map(|r| match &r.affix {
                Affix::Suffix(s) => Some(s.text.as_str()),
                Affix::Prefix(_) => None,
            })
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived store: the [`MorphologicalRuleRecord`] mirror vector
/// `rkyv`-serialized into one [`AlignedVec<16>`].
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use alloc::string::String;

    use rkyv::util::AlignedVec;

    use pr4xis_runtime::lens::rkyv_lens::{RkyvLens, RkyvMirror, RkyvOwned};

    use super::*;
    use crate::cognitive::linguistics::lexicon::pos::PosTag;
    use crate::cognitive::linguistics::morphology::{Affix, Prefix, SemanticEffect, Suffix};

    /// The concrete lens for the morphology store instance.
    type MorphologyLens = RkyvLens<Vec<MorphologicalRule>, MorphologicalRuleRecords>;

    const _: () = assert!(cfg!(target_endian = "little"));

    /// `MorphologyStore` is `Sync`: its only field is an [`AlignedVec`] (rkyv
    /// declares it `Send + Sync`) — no interior mutability.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MorphologyStore>();
    };

    // ── mirror records ────────────────────────────────────────────────────────

    /// Mirror of a [`Prefix`]/[`Suffix`] morpheme body (both share the same shape).
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct AffixMorphRecord {
        pub text: String,
        pub effect: SemanticEffect,
    }

    /// Mirror of [`Affix`].
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub enum AffixRecord {
        Prefix(AffixMorphRecord),
        Suffix(AffixMorphRecord),
    }

    /// Mirror of [`MorphologicalRule`] — the archive's element type.
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct MorphologicalRuleRecord {
        pub affix: AffixRecord,
        pub input_pos: PosTag,
        pub output_pos: PosTag,
        pub effect: SemanticEffect,
    }

    /// The archive root: the morphological-rule records. A local newtype (not a
    /// bare `Vec<MorphologicalRuleRecord>`) so the [`RkyvMirror`] / [`RkyvOwned`]
    /// leaf-lens conversions land on a domains-local type (the orphan rule forbids
    /// impl'ing the foreign traits for a bare foreign `Vec<_>`).
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct MorphologicalRuleRecords {
        /// The rule records, in rule order.
        pub rules: Vec<MorphologicalRuleRecord>,
    }

    type ArchivedRecords = rkyv::Archived<MorphologicalRuleRecords>;

    // ── owned → record (build side) ───────────────────────────────────────────

    impl From<&MorphologicalRule> for MorphologicalRuleRecord {
        fn from(r: &MorphologicalRule) -> Self {
            let affix = match &r.affix {
                Affix::Prefix(p) => AffixRecord::Prefix(AffixMorphRecord {
                    text: p.text.clone(),
                    effect: p.effect,
                }),
                Affix::Suffix(s) => AffixRecord::Suffix(AffixMorphRecord {
                    text: s.text.clone(),
                    effect: s.effect,
                }),
            };
            Self {
                affix,
                input_pos: r.input_pos,
                output_pos: r.output_pos,
                effect: r.effect,
            }
        }
    }

    // ── record → owned (read side, after deserialize) ─────────────────────────

    impl From<MorphologicalRuleRecord> for MorphologicalRule {
        fn from(r: MorphologicalRuleRecord) -> Self {
            let affix = match r.affix {
                AffixRecord::Prefix(m) => Affix::Prefix(Prefix {
                    text: m.text,
                    effect: m.effect,
                }),
                AffixRecord::Suffix(m) => Affix::Suffix(Suffix {
                    text: m.text,
                    effect: m.effect,
                }),
            };
            MorphologicalRule {
                affix,
                input_pos: r.input_pos,
                output_pos: r.output_pos,
                effect: r.effect,
            }
        }
    }

    // ── leaf lens: Vec<MorphologicalRule> ⇄ MorphologicalRuleRecords ──────────

    /// PUT leg: project the owned rules into their record mirror (per-element via
    /// the [`From<&MorphologicalRule>`](MorphologicalRuleRecord) leaf conversion).
    impl RkyvMirror<Vec<MorphologicalRule>> for MorphologicalRuleRecords {
        fn from_owned(rules: &Vec<MorphologicalRule>) -> Self {
            MorphologicalRuleRecords {
                rules: rules.iter().map(MorphologicalRuleRecord::from).collect(),
            }
        }
    }

    /// GET leg: rebuild the owned rules (per-element via the
    /// [`From<MorphologicalRuleRecord>`](MorphologicalRule) leaf conversion).
    /// Total — the record → rule decode cannot fail.
    impl RkyvOwned<MorphologicalRuleRecords> for Vec<MorphologicalRule> {
        type Error = core::convert::Infallible;
        fn from_mirror(
            mirror: MorphologicalRuleRecords,
        ) -> Result<Self, core::convert::Infallible> {
            Ok(mirror
                .rules
                .into_iter()
                .map(MorphologicalRule::from)
                .collect())
        }
    }

    /// The morphological rules, held as one `rkyv`-archived, immutable buffer.
    pub struct MorphologyStore {
        buf: AlignedVec<16>,
    }

    impl MorphologyStore {
        /// Transcode the owned rule vector into the archived buffer ONCE (PUT
        /// through the shared [`RkyvLens`]), validated once here so the reads are
        /// sound.
        pub fn build(rules: Vec<MorphologicalRule>) -> Self {
            let buf = MorphologyLens::put_aligned(&rules);
            MorphologyLens::access(buf.as_slice())
                .expect("freshly-serialized morphology records must bytecheck-validate");
            Self { buf }
        }

        /// The archived record root, borrowed zero-copy from the buffer.
        #[inline]
        fn root(&self) -> &ArchivedRecords {
            // SAFETY: `buf` was produced by `RkyvLens::put_aligned` (16-aligned,
            // sound), `bytecheck`-validated ONCE in `build`, and never mutated (no
            // interior mutability). The validate-once / access_unchecked-many
            // discipline.
            unsafe { MorphologyLens::access_unchecked(self.buf.as_slice()) }
        }

        /// The suffix affix texts, in rule order, read ZERO-COPY out of the archive
        /// — the warm stemming-loop reader. Each `&str` borrows the buffer; no
        /// allocation, so the tokenizer's per-token stemming stays allocation-free.
        pub fn suffix_texts(&self) -> impl Iterator<Item = &str> {
            self.root().rules.iter().filter_map(|r| match &r.affix {
                ArchivedAffixRecord::Suffix(m) => Some(m.text.as_str()),
                ArchivedAffixRecord::Prefix(_) => None,
            })
        }

        /// Every rule, materialized from the archive via the OWNING GET
        /// ([`RkyvLens::get`]) — the cold, test-only trait reader (see the
        /// [module docs](super)).
        pub fn rules(&self) -> Vec<MorphologicalRule> {
            MorphologyLens::get(self.buf.as_slice())
                .expect("the once-validated morphology archive must deserialize")
        }
    }
}

// ── record cast unit tests (archived path) ───────────────────────────────────
//
// Direct coverage of the mirror round-trip + the zero-copy suffix read: build from
// the KNOWN English rule set, assert the deserialized rules and the suffix texts
// are identical to the owned fallback built from the SAME rules.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod cast_tests {
    use super::MorphologyStore; // the archived store
    use super::owned::MorphologyStore as OwnedStore; // the owned fallback
    use crate::cognitive::linguistics::morphology::Affix;
    use crate::cognitive::linguistics::morphology::english::english_rules;
    use alloc::string::String;
    use alloc::vec::Vec;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_rules_match_the_known_set() {
        let store = MorphologyStore::build(english_rules());
        let rules = store.rules();
        let source = english_rules();
        assert_eq!(rules.len(), source.len());
        assert_eq!(
            rules, source,
            "deserialized rules must equal english_rules()"
        );

        // The suffix texts the stemming loop reads, zero-copy.
        let suffixes: Vec<&str> = store.suffix_texts().collect();
        let expected: Vec<String> = source
            .iter()
            .filter_map(|r| match &r.affix {
                Affix::Suffix(s) => Some(s.text.clone()),
                Affix::Prefix(_) => None,
            })
            .collect();
        assert_eq!(suffixes, expected);
        // Sanity: the core inflectional suffixes are present.
        for s in ["s", "ed", "ing", "ly"] {
            assert!(suffixes.contains(&s), "missing suffix {s}");
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_is_identical_to_the_owned_fallback() {
        let archived = MorphologyStore::build(english_rules());
        let owned = OwnedStore::build(english_rules());
        assert_eq!(archived.rules(), owned.rules());
        let a: Vec<&str> = archived.suffix_texts().collect();
        let o: Vec<&str> = owned.suffix_texts().collect();
        assert_eq!(a, o);
    }
}
