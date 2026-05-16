//! English morphological rules — the canonical inflectional and
//! a small derivational set. Each rule pairs an
//! [`super::super::Affix`] with the input/output part-of-speech and
//! the [`super::super::SemanticEffect`] the affix produces.
//!
//! Coverage is closed-class inflectional plus the highest-frequency
//! productive derivational affixes documented in Bauer (1983) §6–§7
//! and Marchand (1969). The eventual replacement is a load from a
//! derivational-variant database (CatVar — Habash & Dorr 2003;
//! WordNet morphosemantic DB — Fellbaum et al. 2009); until then
//! the literature-cited rules live here.
//!
//! **SemanticEffect mapping for productive derivational suffixes is
//! approximate.** The shared enum lacks dedicated variants for
//! relational-adjective (-ory, -ive), abstract-noun (-ability,
//! -ment, -tion), and temporal-prefix (pre-, post-) categories; we
//! reuse the closest existing variants (PosChange, QualityNoun,
//! Negation) so the inversion logic works without growing the enum
//! into English-specific categories. The OLiA cross-functor at
//! `lexicon::olia::semantic_effect_to_olia_fragments` will need
//! refinement when these are properly typed.
//!
//! # Literature
//!
//! - **Bauer, Laurie (1983)** *English Word-formation*, Cambridge
//!   University Press — affix catalogue.
//! - **Quirk, Greenbaum, Leech & Svartvik (1985)** *A
//!   Comprehensive Grammar of the English Language*, Longman —
//!   canonical English inflectional system.
//! - **Marchand, Hans (1969)** *The Categories and Types of
//!   Present-Day English Word-Formation*, Beck — derivational
//!   morphology reference (for the future source-loaded layer).

#[allow(unused_imports)]
use alloc::{vec, vec::Vec};

use super::super::{Affix, MorphologicalRule, Prefix, SemanticEffect, Suffix};
use crate::cognitive::linguistics::lexicon::pos::PosTag;

/// The English morphological-rule set. Returned by value so callers
/// can iterate without reaching into module state.
pub fn english_rules() -> Vec<MorphologicalRule> {
    vec![
        // Prefixes
        MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "un".into(),
                effect: SemanticEffect::Negation,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adjective,
            effect: SemanticEffect::Negation,
        },
        MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "re".into(),
                effect: SemanticEffect::Repetition,
            }),
            input_pos: PosTag::Verb,
            output_pos: PosTag::Verb,
            effect: SemanticEffect::Repetition,
        },
        // Suffixes
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ly".into(),
                effect: SemanticEffect::PosChange,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adverb,
            effect: SemanticEffect::PosChange,
        },
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "s".into(),
                effect: SemanticEffect::NumberChange,
            }),
            input_pos: PosTag::Noun,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::NumberChange,
        },
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ed".into(),
                effect: SemanticEffect::TenseChange,
            }),
            input_pos: PosTag::Verb,
            output_pos: PosTag::Verb,
            effect: SemanticEffect::TenseChange,
        },
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ing".into(),
                effect: SemanticEffect::Progressive,
            }),
            input_pos: PosTag::Verb,
            output_pos: PosTag::Verb,
            effect: SemanticEffect::Progressive,
        },
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "er".into(),
                effect: SemanticEffect::AgentNoun,
            }),
            input_pos: PosTag::Verb,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::AgentNoun,
        },
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ness".into(),
                effect: SemanticEffect::QualityNoun,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::QualityNoun,
        },
        // ── Productive derivational affixes (Bauer 1983 §6.1, §6.2) ──
        //
        // The `effect` field is an approximation: SemanticEffect lacks
        // dedicated variants for temporal-prefix and deverbal-abstract-
        // noun categories. The closest existing variants are used and
        // both map to empty OLiA fragments via the cross-functor
        // (so no misleading morphosyntactic-category claim escapes).
        // Inversion is text-only and unaffected by the labels.
        //
        // Temporal prefix pre-: "predispute" → "dispute".
        // Bauer (1983) §6.1.6 "Locative/temporal prefixes".
        MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "pre".into(),
                effect: SemanticEffect::Repetition,
            }),
            input_pos: PosTag::Noun,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::Repetition,
        },
        // Deverbal abstract noun -ability: "waivability" → "waive".
        // Bauer (1983) §6.2.2 "Nominalizations from verbs".
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ability".into(),
                effect: SemanticEffect::QualityNoun,
            }),
            input_pos: PosTag::Verb,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::QualityNoun,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_has_10_rules() {
        assert_eq!(english_rules().len(), 10);
    }

    #[test]
    fn every_rule_well_typed() {
        for rule in english_rules() {
            match &rule.affix {
                Affix::Prefix(p) => assert!(!p.text.is_empty()),
                Affix::Suffix(s) => assert!(!s.text.is_empty()),
            }
        }
    }
}
