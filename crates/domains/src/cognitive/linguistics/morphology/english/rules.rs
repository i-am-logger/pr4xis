//! English morphological rules — the canonical inflectional and
//! a small derivational set. Each rule pairs an
//! [`super::super::Affix`] with the input/output part-of-speech and
//! the [`super::super::SemanticEffect`] the affix produces.
//!
//! Coverage is closed-class inflectional plus the highest-frequency
//! productive derivational affixes documented in Bauer (1983) ch.7
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
        // ── Productive derivational affixes (Bauer 1983 ch.7) ──
        //
        // The `effect` field is an approximation: SemanticEffect lacks
        // dedicated variants for temporal-prefix and deverbal-abstract-
        // noun categories. The closest existing variants are used and
        // both map to empty OLiA fragments via the cross-functor
        // (so no misleading morphosyntactic-category claim escapes).
        // Inversion is text-only and unaffected by the labels.
        //
        // Temporal prefix pre-: "predispute" → "dispute".
        // Bauer (1983) ch.7 (locative/temporal prefixes).
        MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "pre".into(),
                effect: SemanticEffect::Repetition,
            }),
            input_pos: PosTag::Noun,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::Repetition,
        },
        // Negation prefix non-: "nonenforceability" → "enforceability".
        // Quirk, Greenbaum, Leech & Svartvik (1985) *A Comprehensive
        // Grammar of the English Language* §I.21 (negative prefixes);
        // Bauer (1983) ch.7 (negative prefixes). Productive on
        // both nouns and adjectives in English; per the unique-affix
        // convention in this rule set, declared once with Noun POS
        // — the text-only inverter applies the same surgery
        // regardless of POS.
        MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "non".into(),
                effect: SemanticEffect::Negation,
            }),
            input_pos: PosTag::Noun,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::Negation,
        },
        // Deverbal abstract noun -ability: "waivability" → "waive".
        // Bauer (1983) ch.7 (nominalizations from verbs).
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ability".into(),
                effect: SemanticEffect::QualityNoun,
            }),
            input_pos: PosTag::Verb,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::QualityNoun,
        },
        // Relational/locative prefix inter-: "interparliamentary" →
        // "parliamentary"; "interstate" → "state". "Between / among /
        // mutually." POS-preserving; reuses the locative-prefix effect
        // approximation documented for pre- above (text-only inversion;
        // maps to empty OLiA fragments, so no false morphosyntactic
        // claim escapes). Bauer (1983) ch.7; Quirk, Greenbaum, Leech &
        // Svartvik (1985) §I.21 (locative prefixes).
        MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "inter".into(),
                effect: SemanticEffect::Repetition,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adjective,
            effect: SemanticEffect::Repetition,
        },
        // Verb-forming suffix -ize: "annualize" → "annual". A genuine
        // denominal / de-adjectival verbalizer (Noun/Adj → Verb), so
        // the effect is an honest PosChange. Combined with the -ed
        // inflection + e-restoration allomorphy, the fixed-point
        // lemmatizer recovers "annualized" → "annualize" → "annual".
        // Bauer (1983) ch.7 (verb-forming suffixes); Quirk et al.
        // (1985) §I.43.
        MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ize".into(),
                effect: SemanticEffect::PosChange,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Verb,
            effect: SemanticEffect::PosChange,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_has_13_rules() {
        // 10 base + "non-" (M5.D) + the inter- locative prefix and the
        // -ize verb-forming suffix added for the Title-2 gap audit
        // ("interparliamentary" → "parliamentary"; "annualized" →
        // "annualize" → "annual").
        assert_eq!(english_rules().len(), 13);
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

    // ── Property-based laws for the English rule set ───────────────
    //
    // The universal `MorphologicalRule::invert` round-trip law lives
    // in morphology/mod.rs. These properties verify English-specific
    // invariants that hold for every rule in english_rules().

    use proptest::prelude::*;

    fn arb_rule_index() -> impl Strategy<Value = usize> {
        let count = english_rules().len();
        0..count
    }

    fn arb_stem() -> impl Strategy<Value = String> {
        // Avoid stems ending in 'e' or 'y' so the universal round-
        // trip property doesn't trip over English allomorphy. The
        // English-specific allomorphy lives in english/allomorphy.rs
        // and is tested there.
        proptest::collection::vec(prop::char::range('a', 'z'), 3..8)
            .prop_filter("no-allomorphy-trap", |chars| {
                if chars.is_empty() {
                    return false;
                }
                let last = chars[chars.len() - 1];
                !"eyiu".contains(last)
                    && (chars.len() < 2 || chars[chars.len() - 1] != chars[chars.len() - 2])
            })
            .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        #[test]
        fn property_every_english_rule_apply_then_invert_recovers_stem(
            i in arb_rule_index(),
            stem in arb_stem(),
        ) {
            let rule = english_rules()[i].clone();
            let derived = rule.apply(&stem);
            if derived == stem {
                return Ok(());
            }
            let candidates = rule.invert(&derived);
            prop_assert!(
                candidates.iter().any(|c| c == &stem),
                "round-trip fail for rule {:?} on stem `{stem}`: derived={derived}, invert={candidates:?}",
                rule.effect
            );
        }

        #[test]
        fn property_english_rules_have_unique_affixes(_ in 0..1) {
            // Each affix text appears at most once in english_rules.
            // Duplicate affixes would mean the lemmatizer does the
            // same work twice.
            let mut affixes: Vec<String> = Vec::new();
            for rule in english_rules() {
                let text = match &rule.affix {
                    Affix::Prefix(p) => format!("pre:{}", p.text),
                    Affix::Suffix(s) => format!("suf:{}", s.text),
                };
                prop_assert!(
                    !affixes.contains(&text),
                    "duplicate affix `{text}` in english_rules"
                );
                affixes.push(text);
            }
        }

        #[test]
        fn property_english_rule_affixes_are_lowercase_ascii(
            i in arb_rule_index(),
        ) {
            let rule = english_rules()[i].clone();
            let text = match rule.affix {
                Affix::Prefix(p) => p.text,
                Affix::Suffix(s) => s.text,
            };
            for c in text.chars() {
                prop_assert!(
                    c.is_ascii_lowercase(),
                    "non-lowercase-ASCII char `{c}` in affix `{text}`"
                );
            }
        }
    }
}
