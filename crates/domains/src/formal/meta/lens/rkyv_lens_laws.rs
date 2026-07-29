//! Runnable, cited lens-law axioms for the four rich English M2 stores that are
//! instances of the shared
//! [`RkyvLens`](pr4xis_runtime::lens::rkyv_lens::RkyvLens) — `concept_store`,
//! `function_word_store`, `morphology_store` and `writing_system_store`.
//!
//! Each store supplies only its leaf lens (a hand-authored `*Record` mirror plus
//! the two [`RkyvMirror`](pr4xis_runtime::lens::rkyv_lens::RkyvMirror) /
//! [`RkyvOwned`](pr4xis_runtime::lens::rkyv_lens::RkyvOwned) conversions); the
//! generic PUT / GET / ACCESS lives once in `pr4xis-runtime`. This module lifts
//! the three generic lens-law predicates — proven ONCE over the shared lens —
//! into registered, discoverable `Axiom`s that run each predicate over ALL
//! FOUR store instances' witness corpora, mirroring `packed_csr_laws` (the M1
//! stores) and `pr4xis_runtime`'s `ArchiveLensGetPut` (the runtime `Archive`
//! instance):
//!
//! - `RkyvLensGetPut` — `put(get(b)) == b`: the `rkyv` cache blob is stable
//!   under a decode/re-encode round-trip.
//! - `RkyvLensPutGet` — `get(put(o)) == o`: an owned value round-trips through
//!   the cache form with its full query image intact.
//! - `RkyvLensDeterminism` — `put(o) == put(o)`: the cache bytes are a
//!   deterministic function of the owned value alone (the law underwriting
//!   GetPut).
//! - `RkyvLensOwnedPutAgrees` — `put_aligned_owned(x.clone()) == put_aligned(&x)`:
//!   the owned-consuming (MOVE) PUT leg the store builds through is byte-identical
//!   to the borrowing (clone) PUT leg, so consuming the owned build to halve the
//!   load-time transient peak changes not one archived byte.
//!
//! Gated on `prx` + little-endian, where the archived stores (and their mirror
//! roots) exist. The three axioms self-register through `register_axiom!`, so
//! they resolve by name through the same registry as every other lens law — the
//! whole lens-law family (general algebra, byte-anchored round-trip, packed-CSR
//! stores, rkyv rich stores, runtime archive) answers one graph query.
//!
//! # Literature
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §3, Definition 3.2 — the
//!   well-behaved-lens laws (GetPut / PutGet).
//! - **Koloski, D.** *rkyv: zero-copy deserialization framework for Rust*, v0.8,
//!   <https://github.com/rkyv/rkyv> — the archived form the lens serializes to.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use hashbrown::HashMap;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis_runtime::lens::rkyv_lens::{
    determinism_holds, getput_holds, owned_put_agrees_holds, putget_holds,
};

use crate::cognitive::linguistics::english::concept_store::ConceptRecords;
use crate::cognitive::linguistics::english::function_word_store::FunctionWordRecords;
use crate::cognitive::linguistics::english::morphology_store::MorphologicalRuleRecords;
use crate::cognitive::linguistics::english::writing_system_store::WritingSystemRecord;
use crate::cognitive::linguistics::english::{Concept, ConceptId};
use crate::cognitive::linguistics::lexicon::pos::{
    Determiner, DeterminerKind, LexicalEntry, Number, Person, Pronoun, PronounKind, WhReferentRole,
};
use crate::cognitive::linguistics::morphology::MorphologicalRule;
use crate::cognitive::linguistics::morphology::english::english_rules;
use crate::cognitive::linguistics::orthography::{WritingSystem, english_writing_system};
use crate::social::software::markup::xml::lmf::ontology::LmfPos;

// ── witness corpora, one per store instance ──────────────────────────────────

/// Concept-store witnesses: an empty store and a rich two-record store. Each
/// record's `id` IS its index (the mirror drops `id`, the GET leg re-derives it
/// from position — see `concept_store`), so the fixtures set `id` to the slot.
fn concept_witnesses() -> Vec<Vec<Concept>> {
    vec![
        Vec::new(),
        vec![
            Concept {
                id: ConceptId::new(0),
                original_id: String::from("oewn-00001740-n"),
                pos: LmfPos::Noun,
                lemmas: vec![String::from("entity")],
                definitions: vec![String::from("that which is perceived or known")],
                examples: Vec::new(),
            },
            Concept {
                id: ConceptId::new(1),
                original_id: String::from("oewn-02604760-v"),
                pos: LmfPos::Verb,
                lemmas: vec![String::from("be"), String::from("exist")],
                definitions: vec![
                    String::from("have the quality of being"),
                    String::from("occupy a certain position"),
                ],
                examples: vec![String::from("there is a God")],
            },
        ],
    ]
}

/// Function-word-store witnesses: an empty lexicon and a rich one — a
/// single-reading determiner, a multi-reading surface (order must survive), and
/// a reading carrying a `None` field.
fn function_word_witnesses() -> Vec<HashMap<String, Vec<LexicalEntry>>> {
    let empty: HashMap<String, Vec<LexicalEntry>> = HashMap::new();

    let mut rich: HashMap<String, Vec<LexicalEntry>> = HashMap::new();
    rich.insert(
        String::from("the"),
        vec![LexicalEntry::Determiner(Determiner {
            text: String::from("the"),
            kind: DeterminerKind::Definite,
            number: None,
            olia_class: None,
            referent_role: None,
        })],
    );
    rich.insert(
        String::from("who"),
        vec![LexicalEntry::Pronoun(Pronoun {
            text: String::from("who"),
            number: Number::Singular,
            person: Person::Third,
            kind: PronounKind::Interrogative,
            olia_class: None,
            referent_role: Some(WhReferentRole::Person),
        })],
    );
    rich.insert(
        String::from("what"),
        vec![
            LexicalEntry::Pronoun(Pronoun {
                text: String::from("what"),
                number: Number::Singular,
                person: Person::Third,
                kind: PronounKind::Interrogative,
                olia_class: Some(String::from("InterrogativePronoun")),
                referent_role: Some(WhReferentRole::Thing),
            }),
            LexicalEntry::Determiner(Determiner {
                text: String::from("what"),
                kind: DeterminerKind::Indefinite,
                number: None,
                olia_class: Some(String::from("InterrogativeDeterminer")),
                referent_role: Some(WhReferentRole::Thing),
            }),
        ],
    );

    vec![empty, rich]
}

/// Morphology-store witnesses: an empty rule set and the real English rule set.
fn morphology_witnesses() -> Vec<Vec<MorphologicalRule>> {
    vec![Vec::new(), english_rules()]
}

/// Writing-system-store witnesses: the real English writing system (the single
/// deep value the store archives).
fn writing_system_witnesses() -> Vec<WritingSystem> {
    vec![english_writing_system()]
}

// ── the three axioms — each predicate over all four instances ────────────────

/// GetPut leg of the shared `RkyvLens` over the four rich English M2 stores: for
/// bytes `b` canonically produced by `put`, `put(get(b)) == b` — the `rkyv`
/// cache blob is stable under a decode/re-encode round-trip, over the concept,
/// function-word, morphology and writing-system instances. Foster, Greenwald,
/// Moore, Pierce & Schmitt (2007) §3, Definition 3.2.
pub struct RkyvLensGetPut;

impl Axiom for RkyvLensGetPut {
    fn verify(&self) -> Verdict {
        let holds = getput_holds::<Vec<Concept>, ConceptRecords>(&concept_witnesses())
            && getput_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(
                &function_word_witnesses(),
            )
            && getput_holds::<Vec<MorphologicalRule>, MorphologicalRuleRecords>(
                &morphology_witnesses(),
            )
            && getput_holds::<WritingSystem, WritingSystemRecord>(&writing_system_witnesses());
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RkyvLensGetPut",
        "put(get(b)) == b for the rkyv cache bytes of the four rich English stores (concept/function-word/morphology/writing-system)",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(RkyvLensGetPut, constructor);

/// PutGet leg of the shared `RkyvLens` over the four rich English M2 stores:
/// `get(put(o)) == o` — an owned value round-trips through the `rkyv` cache form
/// with its full query image intact, over all four instances. Foster, Greenwald,
/// Moore, Pierce & Schmitt (2007) §3, Definition 3.2.
pub struct RkyvLensPutGet;

impl Axiom for RkyvLensPutGet {
    fn verify(&self) -> Verdict {
        let holds = putget_holds::<Vec<Concept>, ConceptRecords>(&concept_witnesses())
            && putget_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(
                &function_word_witnesses(),
            )
            && putget_holds::<Vec<MorphologicalRule>, MorphologicalRuleRecords>(
                &morphology_witnesses(),
            )
            && putget_holds::<WritingSystem, WritingSystemRecord>(&writing_system_witnesses());
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RkyvLensPutGet",
        "get(put(o)) == o: each rich English store's owned value round-trips through the rkyv cache form with its full query image intact",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(RkyvLensPutGet, constructor);

/// Determinism leg of the shared `RkyvLens` over the four rich English M2
/// stores: `put(o) == put(o)` — the `rkyv` cache bytes are a deterministic
/// function of the owned value alone (no build-order or address
/// nondeterminism), the property that underwrites [`RkyvLensGetPut`]. Foster,
/// Greenwald, Moore, Pierce & Schmitt (2007) §3, Definition 3.2.
pub struct RkyvLensDeterminism;

impl Axiom for RkyvLensDeterminism {
    fn verify(&self) -> Verdict {
        let holds = determinism_holds::<Vec<Concept>, ConceptRecords>(&concept_witnesses())
            && determinism_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(
                &function_word_witnesses(),
            )
            && determinism_holds::<Vec<MorphologicalRule>, MorphologicalRuleRecords>(
                &morphology_witnesses(),
            )
            && determinism_holds::<WritingSystem, WritingSystemRecord>(&writing_system_witnesses());
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RkyvLensDeterminism",
        "put(o) == put(o): each rich English store's rkyv cache bytes are a deterministic function of the owned value alone",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(RkyvLensDeterminism, constructor);

/// Owned-PUT-agreement leg of the shared `RkyvLens` over the four rich English
/// M2 stores: `put_aligned_owned(x.clone()) == put_aligned(&x)` — the
/// owned-consuming (MOVE) PUT leg each store's `build` now takes is byte-identical
/// to the borrowing (clone) PUT leg, over the concept, function-word, morphology
/// and writing-system instances. This is the law that licenses consuming the
/// owned build to MOVE its heap payloads into the archive (halving the load-time
/// transient peak — the concept store's ~10⁵ records are never duplicated) as a
/// pure optimization: it cannot change a single archived byte. Foster, Greenwald,
/// Moore, Pierce & Schmitt (2007) §3, Definition 3.2 (PUT is a function of its argument alone).
pub struct RkyvLensOwnedPutAgrees;

impl Axiom for RkyvLensOwnedPutAgrees {
    fn verify(&self) -> Verdict {
        let holds = owned_put_agrees_holds::<Vec<Concept>, ConceptRecords>(&concept_witnesses())
            && owned_put_agrees_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(
                &function_word_witnesses(),
            )
            && owned_put_agrees_holds::<Vec<MorphologicalRule>, MorphologicalRuleRecords>(
                &morphology_witnesses(),
            )
            && owned_put_agrees_holds::<WritingSystem, WritingSystemRecord>(
                &writing_system_witnesses(),
            );
        if holds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RkyvLensOwnedPutAgrees",
        "put_aligned_owned(x.clone()) == put_aligned(&x): each rich English store's owned-consuming (move) PUT leg is byte-identical to its borrowing (clone) PUT leg",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §3, Definition 3.2"
    );
}

pr4xis::register_axiom!(RkyvLensOwnedPutAgrees, constructor);

// ── discoverability + the laws hold ──────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cognitive::linguistics::lexicon::pos::WhAdverbRole;
    use pr4xis::ontology::registry::axiom_by_name;

    /// The four lens-law axioms hold over the four rich English store instances.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rkyv_lens_laws_hold() {
        assert!(RkyvLensGetPut.verify().is_ok(), "put(get(b)) == b");
        assert!(RkyvLensPutGet.verify().is_ok(), "get(put(o)) == o");
        assert!(RkyvLensDeterminism.verify().is_ok(), "put(o) == put(o)");
        assert!(
            RkyvLensOwnedPutAgrees.verify().is_ok(),
            "put_aligned_owned(x.clone()) == put_aligned(&x)"
        );
    }

    /// The four axioms re-bind by name through the registry — discoverable as
    /// any statute's law is (the load-time rebind gate).
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn laws_discoverable_via_registry() {
        for name in [
            "RkyvLensGetPut",
            "RkyvLensPutGet",
            "RkyvLensDeterminism",
            "RkyvLensOwnedPutAgrees",
        ] {
            assert!(
                axiom_by_name(name).is_some(),
                "rkyv lens axiom {name} must re-bind through the registry"
            );
        }
    }

    // ── ∀-strengthening: the laws over GENERATED values ──────────────────────
    //
    // The registered axioms verify over FIXED witness corpora; these properties
    // run the same four predicates over ARBITRARY generated values for the two
    // structurally interesting instances — the `Vec<Concept>` store (arbitrary
    // strings + the full `LmfPos` enumeration, with the id-IS-index invariant
    // the mirror's GET leg re-derives) and the
    // `HashMap<String, Vec<LexicalEntry>>` store (generated entries across ALL
    // 13 `LexicalEntry` variants, multi-reading surfaces, `None`-carrying
    // fields) — the archive_lens `prop_archive_lens_round_trips` pattern.

    use proptest::prelude::*;

    use crate::cognitive::linguistics::lexicon::pos::{
        Adjective, Adverb, Auxiliary, Conjunction, Copula, Countability, Interjection,
        InterjectionKind, Noun, NounKind, Numeral, Particle, Polarity, Preposition, Tense,
        Transitivity, Verb,
    };

    /// Every `LmfPos` variant — the full WN-LMF/UD part-of-speech enumeration.
    fn arb_lmf_pos() -> impl Strategy<Value = LmfPos> {
        prop::sample::select(alloc::vec![
            LmfPos::Noun,
            LmfPos::Verb,
            LmfPos::Adjective,
            LmfPos::SatelliteAdjective,
            LmfPos::Adverb,
            LmfPos::Determiner,
            LmfPos::Pronoun,
            LmfPos::Preposition,
            LmfPos::Conjunction,
            LmfPos::Particle,
            LmfPos::Copula,
            LmfPos::Auxiliary,
            LmfPos::Interjection,
            LmfPos::Numeral,
            LmfPos::Other,
        ])
    }

    /// A generated concept store: 0..6 concepts with arbitrary printable
    /// strings and any `LmfPos`. The record's `id` IS its index (the mirror
    /// drops `id`; the GET leg re-derives it from position — the store's
    /// documented invariant), so ids are assigned by slot after generation.
    fn arb_concepts() -> impl Strategy<Value = Vec<Concept>> {
        prop::collection::vec(
            (
                "[ -~]{0,16}",                                    // original_id
                arb_lmf_pos(),                                    // pos
                prop::collection::vec("[a-zA-Z' -]{0,10}", 0..3), // lemmas
                prop::collection::vec("[ -~]{0,24}", 0..3),       // definitions
                prop::collection::vec("[ -~]{0,24}", 0..2),       // examples
            ),
            0..6,
        )
        .prop_map(|rows| {
            rows.into_iter()
                .enumerate()
                .map(
                    |(i, (original_id, pos, lemmas, definitions, examples))| Concept {
                        id: ConceptId::new(i as u64),
                        original_id,
                        pos,
                        lemmas,
                        definitions,
                        examples,
                    },
                )
                .collect()
        })
    }

    fn arb_number() -> impl Strategy<Value = Number> {
        prop::sample::select(alloc::vec![Number::Singular, Number::Plural])
    }

    fn arb_person() -> impl Strategy<Value = Person> {
        prop::sample::select(alloc::vec![Person::First, Person::Second, Person::Third])
    }

    fn arb_tense() -> impl Strategy<Value = Tense> {
        prop::sample::select(alloc::vec![Tense::Present, Tense::Past, Tense::Future])
    }

    /// One generated `LexicalEntry`, drawn across ALL 13 variants, with every
    /// enum field ranging over its full variant set and every `Option` field
    /// over both `Some` and `None`.
    fn arb_lexical_entry() -> impl Strategy<Value = LexicalEntry> {
        let text = "[a-z' -]{1,10}";
        let olia = prop::option::of("[A-Za-z]{1,16}".prop_map(String::from));
        let wh_referent_role = prop::option::of(prop::sample::select(alloc::vec![
            WhReferentRole::Person,
            WhReferentRole::Thing,
            WhReferentRole::Selection,
        ]));
        let wh_adverb_role = prop::option::of(prop::sample::select(alloc::vec![
            WhAdverbRole::Manner,
            WhAdverbRole::Reason,
            WhAdverbRole::Place,
            WhAdverbRole::Time,
        ]));
        prop_oneof![
            (
                text,
                arb_number(),
                arb_person(),
                prop::sample::select(alloc::vec![
                    Countability::Countable,
                    Countability::Uncountable
                ]),
                prop::sample::select(alloc::vec![NounKind::Common, NounKind::Proper]),
            )
                .prop_map(|(text, number, person, countability, kind)| {
                    LexicalEntry::Noun(Noun {
                        text,
                        number,
                        person,
                        countability,
                        kind,
                    })
                }),
            (
                text,
                text,
                arb_number(),
                arb_person(),
                arb_tense(),
                prop::sample::select(alloc::vec![
                    Transitivity::Transitive,
                    Transitivity::Intransitive,
                    Transitivity::Ditransitive,
                ]),
            )
                .prop_map(|(text, lemma, number, person, tense, transitivity)| {
                    LexicalEntry::Verb(Verb {
                        text,
                        lemma,
                        number,
                        person,
                        tense,
                        transitivity,
                        olia_class: None,
                    })
                }),
            (
                text,
                prop::sample::select(alloc::vec![
                    DeterminerKind::Definite,
                    DeterminerKind::Indefinite,
                    DeterminerKind::Demonstrative,
                    DeterminerKind::Quantifier,
                ]),
                prop::option::of(arb_number()),
                olia.clone(),
                wh_referent_role.clone(),
            )
                .prop_map(|(text, kind, number, olia_class, referent_role)| {
                    LexicalEntry::Determiner(Determiner {
                        text,
                        kind,
                        number,
                        olia_class,
                        referent_role,
                    })
                }),
            text.prop_map(|text| LexicalEntry::Adjective(Adjective { text })),
            (text, olia.clone(), wh_adverb_role).prop_map(|(text, olia_class, role)| {
                LexicalEntry::Adverb(Adverb {
                    text,
                    olia_class,
                    role,
                })
            }),
            text.prop_map(|text| LexicalEntry::Preposition(Preposition { text })),
            (text, olia.clone()).prop_map(|(text, olia_class)| {
                LexicalEntry::Conjunction(Conjunction { text, olia_class })
            }),
            (
                text,
                arb_number(),
                arb_person(),
                prop::sample::select(alloc::vec![
                    PronounKind::Personal,
                    PronounKind::Interrogative,
                    PronounKind::Demonstrative,
                    PronounKind::Relative,
                    PronounKind::Reflexive,
                    PronounKind::Indefinite,
                    PronounKind::Possessive,
                ]),
                olia.clone(),
                wh_referent_role,
            )
                .prop_map(|(text, number, person, kind, olia_class, referent_role)| {
                    LexicalEntry::Pronoun(Pronoun {
                        text,
                        number,
                        person,
                        kind,
                        olia_class,
                        referent_role,
                    })
                }),
            (text, arb_number(), arb_person(), arb_tense()).prop_map(
                |(text, number, person, tense)| {
                    LexicalEntry::Copula(Copula {
                        text,
                        number,
                        person,
                        tense,
                    })
                }
            ),
            (
                text,
                prop::option::of(arb_number()),
                prop::option::of(arb_tense())
            )
                .prop_map(|(text, number, tense)| {
                    LexicalEntry::Auxiliary(Auxiliary {
                        text,
                        number,
                        tense,
                    })
                }),
            (
                text,
                prop::sample::select(alloc::vec![
                    InterjectionKind::Greeting,
                    InterjectionKind::Farewell,
                    InterjectionKind::Expressive,
                    InterjectionKind::Response,
                    InterjectionKind::Politeness,
                    InterjectionKind::Conative,
                ]),
                prop::option::of(prop::sample::select(alloc::vec![
                    Polarity::Affirmative,
                    Polarity::Negative,
                ])),
            )
                .prop_map(|(text, kind, polarity)| {
                    LexicalEntry::Interjection(Interjection {
                        text,
                        kind,
                        polarity,
                    })
                }),
            (text, olia).prop_map(|(text, olia_class)| {
                LexicalEntry::Particle(Particle { text, olia_class })
            }),
            text.prop_map(|text| LexicalEntry::Numeral(Numeral { text })),
        ]
    }

    /// A generated function-word lexicon: arbitrary surface keys, each with
    /// 0..3 readings (multi-reading order must survive the round-trip).
    fn arb_lexicon() -> impl Strategy<Value = HashMap<String, Vec<LexicalEntry>>> {
        proptest::collection::hash_map(
            "[a-z' -]{1,8}",
            prop::collection::vec(arb_lexical_entry(), 0..3),
            0..4,
        )
        .prop_map(|std_map| std_map.into_iter().collect())
    }

    proptest! {
        /// ∀ generated concept stores: the four lens laws hold — not just over
        /// the fixed witness corpus the registered axiom ranges over.
        #[test]
        fn prop_concept_store_lens_laws_hold(concepts in arb_concepts()) {
            let witnesses = [concepts];
            prop_assert!(
                getput_holds::<Vec<Concept>, ConceptRecords>(&witnesses),
                "put(get(b)) == b for a generated concept store"
            );
            prop_assert!(
                putget_holds::<Vec<Concept>, ConceptRecords>(&witnesses),
                "get(put(o)) == o for a generated concept store"
            );
            prop_assert!(
                determinism_holds::<Vec<Concept>, ConceptRecords>(&witnesses),
                "put(o) == put(o) for a generated concept store"
            );
            prop_assert!(
                owned_put_agrees_holds::<Vec<Concept>, ConceptRecords>(&witnesses),
                "put_aligned_owned(x.clone()) == put_aligned(&x) for a generated concept store"
            );
        }

        /// ∀ generated function-word lexica (entries across all 13
        /// `LexicalEntry` variants): the four lens laws hold.
        #[test]
        fn prop_function_word_store_lens_laws_hold(lexicon in arb_lexicon()) {
            let witnesses = [lexicon];
            prop_assert!(
                getput_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(&witnesses),
                "put(get(b)) == b for a generated lexicon"
            );
            prop_assert!(
                putget_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(&witnesses),
                "get(put(o)) == o for a generated lexicon"
            );
            prop_assert!(
                determinism_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(
                    &witnesses
                ),
                "put(o) == put(o) for a generated lexicon"
            );
            prop_assert!(
                owned_put_agrees_holds::<HashMap<String, Vec<LexicalEntry>>, FunctionWordRecords>(
                    &witnesses
                ),
                "put_aligned_owned(x.clone()) == put_aligned(&x) for a generated lexicon"
            );
        }
    }

    pr4xis::register_praxis_value!(prop_concept_store_lens_laws_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_function_word_store_lens_laws_hold, Verifiable);
}
