//! English morphological rules — the canonical inflectional and
//! a small derivational set. Each rule pairs an
//! [`super::super::Affix`] with the input/output part-of-speech and
//! the [`super::super::SemanticEffect`] the affix produces.
//!
//! Coverage is closed-class inflectional plus the highest-frequency
//! derivational suffixes (`-ly`, `-er`, `-ness`). Productive
//! derivational morphology (`-ory`, `-ability`, `-ment`, `pre-`,
//! `post-`) is intentionally *not* included here — it lives in the
//! source-loaded layer once AGID / UniMorph / WordNet's
//! `derivationally_related_form` pointers are wired up.
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_has_8_rules() {
        assert_eq!(english_rules().len(), 8);
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
