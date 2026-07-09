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
//! into registered, discoverable [`Axiom`]s that run each predicate over ALL
//! FOUR store instances' witness corpora, mirroring `packed_csr_laws` (the M1
//! stores) and `pr4xis_runtime`'s `ArchiveLensGetPut` (the runtime `Archive`
//! instance):
//!
//! - [`RkyvLensGetPut`] — `put(get(b)) == b`: the `rkyv` cache blob is stable
//!   under a decode/re-encode round-trip.
//! - [`RkyvLensPutGet`] — `get(put(o)) == o`: an owned value round-trips through
//!   the cache form with its full query image intact.
//! - [`RkyvLensDeterminism`] — `put(o) == put(o)`: the cache bytes are a
//!   deterministic function of the owned value alone (the law underwriting
//!   GetPut).
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
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2 — the
//!   well-behaved-lens laws (GetPut / PutGet).
//! - **Hill, D.** *rkyv: zero-copy deserialization framework for Rust*, v0.8,
//!   <https://github.com/rkyv/rkyv> — the archived form the lens serializes to.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use hashbrown::HashMap;

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis_runtime::lens::rkyv_lens::{determinism_holds, getput_holds, putget_holds};

use crate::cognitive::linguistics::english::concept_store::ConceptRecords;
use crate::cognitive::linguistics::english::function_word_store::FunctionWordRecords;
use crate::cognitive::linguistics::english::morphology_store::MorphologicalRuleRecords;
use crate::cognitive::linguistics::english::writing_system_store::WritingSystemRecord;
use crate::cognitive::linguistics::english::{Concept, ConceptId};
use crate::cognitive::linguistics::lexicon::pos::{
    Determiner, DeterminerKind, LexicalEntry, Number, Person, Pronoun, PronounKind,
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
            }),
            LexicalEntry::Determiner(Determiner {
                text: String::from("what"),
                kind: DeterminerKind::Indefinite,
                number: None,
                olia_class: Some(String::from("InterrogativeDeterminer")),
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
/// Moore, Pierce & Schmitt (2007) §2.2.
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(RkyvLensGetPut, constructor);

/// PutGet leg of the shared `RkyvLens` over the four rich English M2 stores:
/// `get(put(o)) == o` — an owned value round-trips through the `rkyv` cache form
/// with its full query image intact, over all four instances. Foster, Greenwald,
/// Moore, Pierce & Schmitt (2007) §2.2.
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(RkyvLensPutGet, constructor);

/// Determinism leg of the shared `RkyvLens` over the four rich English M2
/// stores: `put(o) == put(o)` — the `rkyv` cache bytes are a deterministic
/// function of the owned value alone (no build-order or address
/// nondeterminism), the property that underwrites [`RkyvLensGetPut`]. Foster,
/// Greenwald, Moore, Pierce & Schmitt (2007) §2.2.
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
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations, ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(RkyvLensDeterminism, constructor);

// ── discoverability + the laws hold ──────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use pr4xis::ontology::registry::axiom_by_name;

    /// The three lens-law axioms hold over the four rich English store instances.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rkyv_lens_laws_hold() {
        assert!(RkyvLensGetPut.verify().is_ok(), "put(get(b)) == b");
        assert!(RkyvLensPutGet.verify().is_ok(), "get(put(o)) == o");
        assert!(RkyvLensDeterminism.verify().is_ok(), "put(o) == put(o)");
    }

    /// The three axioms re-bind by name through the registry — discoverable as
    /// any statute's law is (the load-time rebind gate).
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn laws_discoverable_via_registry() {
        for name in ["RkyvLensGetPut", "RkyvLensPutGet", "RkyvLensDeterminism"] {
            assert!(
                axiom_by_name(name).is_some(),
                "rkyv lens axiom {name} must re-bind through the registry"
            );
        }
    }
}
