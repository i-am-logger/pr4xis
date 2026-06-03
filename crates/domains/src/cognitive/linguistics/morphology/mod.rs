#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod allomorphy;
pub mod english;
pub mod irregular;
pub mod lemmatizer;
pub mod tense;

use pr4xis::category::{Concept, FinitelyGenerated};

use super::lexicon::pos::PosTag;

/// An affix — a morpheme added to a word to change its meaning or function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Affix {
    /// Added before the root: un-, re-, pre-, dis-.
    Prefix(Prefix),
    /// Added after the root: -ing, -ed, -s, -ly, -ness.
    Suffix(Suffix),
}

/// A prefix morpheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix {
    pub text: String,
    pub effect: SemanticEffect,
}

/// A suffix morpheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Suffix {
    pub text: String,
    pub effect: SemanticEffect,
}

/// What an affix DOES to meaning — connects to reasoning ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticEffect {
    /// Negates meaning: un-happy → NOT happy (connects to OppositionDef).
    Negation,
    /// Repeats: re-do → do again.
    Repetition,
    /// Changes POS: quick → quick-ly (Adj → Adv).
    PosChange,
    /// Changes number: dog → dog-s (Singular → Plural).
    NumberChange,
    /// Changes tense: walk → walk-ed (Present → Past).
    TenseChange,
    /// Creates ongoing action: walk → walk-ing (Progressive).
    Progressive,
    /// Creates agent: teach → teach-er (the one who does).
    AgentNoun,
    /// Creates quality: happy → happi-ness (Adj → Noun).
    QualityNoun,
}

impl Concept for SemanticEffect {}
impl FinitelyGenerated for SemanticEffect {
    fn variants() -> Vec<Self> {
        vec![
            Self::Negation,
            Self::Repetition,
            Self::PosChange,
            Self::NumberChange,
            Self::TenseChange,
            Self::Progressive,
            Self::AgentNoun,
            Self::QualityNoun,
        ]
    }
}

/// A morphological rule — how an affix transforms a word.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MorphologicalRule {
    pub affix: Affix,
    pub input_pos: PosTag,
    pub output_pos: PosTag,
    pub effect: SemanticEffect,
}

impl MorphologicalRule {
    /// Apply this rule to a word stem, producing the derived form.
    pub fn apply(&self, stem: &str) -> String {
        match &self.affix {
            Affix::Prefix(p) => format!("{}{}", p.text, stem),
            Affix::Suffix(s) => format!("{}{}", stem, s.text),
        }
    }

    /// Invert this rule: given a derived surface form, return the
    /// bare candidate stem produced by stripping the affix.
    ///
    /// Returns an empty `Vec` when `surface` does not match this
    /// rule's affix pattern at all, and a one-element `Vec` with
    /// the bare-strip candidate otherwise.
    ///
    /// **Language-agnostic.** This method performs only the
    /// mechanical inversion of [`Self::apply`]. Language-specific
    /// allomorphic alternations — silent-e restoration, doubled-
    /// consonant un-doubling, y/i alternation — live in
    /// [`crate::cognitive::linguistics::morphology::allomorphy::AllomorphyRule`]
    /// instances and are consulted by the lemmatizer separately.
    ///
    /// Cite: Beesley & Karttunen (2003) *Finite-State Morphology*,
    /// CSLI, Ch. 3 — bidirectional FSTs treat forward (`apply`) and
    /// inverse (`invert`) as two readings of the same relation.
    pub fn invert(&self, surface: &str) -> Vec<String> {
        match &self.affix {
            Affix::Prefix(p) => Self::invert_prefix(surface, &p.text),
            Affix::Suffix(s) => Self::invert_suffix(surface, &s.text),
        }
    }

    fn invert_prefix(surface: &str, pre: &str) -> Vec<String> {
        if surface.len() <= pre.len() || !surface.starts_with(pre) {
            return vec![];
        }
        vec![surface[pre.len()..].to_string()]
    }

    fn invert_suffix(surface: &str, suf: &str) -> Vec<String> {
        if surface.len() <= suf.len() || !surface.ends_with(suf) {
            return vec![];
        }
        vec![surface[..surface.len() - suf.len()].to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_prefix() {
        let rule = MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "un".into(),
                effect: SemanticEffect::Negation,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adjective,
            effect: SemanticEffect::Negation,
        };
        assert_eq!(rule.apply("happy"), "unhappy");
    }

    #[test]
    fn apply_suffix() {
        let rule = MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "ly".into(),
                effect: SemanticEffect::PosChange,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adverb,
            effect: SemanticEffect::PosChange,
        };
        assert_eq!(rule.apply("quick"), "quickly");
    }

    #[test]
    fn invert_prefix_strips_match() {
        let rule = MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "un".into(),
                effect: SemanticEffect::Negation,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adjective,
            effect: SemanticEffect::Negation,
        };
        assert_eq!(rule.invert("unhappy"), vec!["happy".to_string()]);
    }

    #[test]
    fn invert_prefix_no_match_returns_empty() {
        let rule = MorphologicalRule {
            affix: Affix::Prefix(Prefix {
                text: "un".into(),
                effect: SemanticEffect::Negation,
            }),
            input_pos: PosTag::Adjective,
            output_pos: PosTag::Adjective,
            effect: SemanticEffect::Negation,
        };
        assert!(rule.invert("happy").is_empty());
    }

    #[test]
    fn invert_suffix_strips_match() {
        let rule = MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "s".into(),
                effect: SemanticEffect::NumberChange,
            }),
            input_pos: PosTag::Noun,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::NumberChange,
        };
        assert_eq!(rule.invert("dogs"), vec!["dog".to_string()]);
    }

    #[test]
    fn invert_never_returns_empty_string() {
        // `s` itself is len == suffix len — must not produce "".
        let rule = MorphologicalRule {
            affix: Affix::Suffix(Suffix {
                text: "s".into(),
                effect: SemanticEffect::NumberChange,
            }),
            input_pos: PosTag::Noun,
            output_pos: PosTag::Noun,
            effect: SemanticEffect::NumberChange,
        };
        assert!(rule.invert("s").is_empty());
    }

    // ── proptest property-based laws for the universal invert ───────
    //
    // Beesley & Karttunen (2003) Ch. 3 — the bidirectional-FST
    // identity. For every rule the inverse must:
    //   (a) be partial (return [] when the surface form has no match)
    //   (b) never return an empty candidate
    //   (c) return only candidates strictly shorter than the surface
    //   (d) recover the original stem when applied to apply(stem)

    use proptest::prelude::*;

    fn arb_simple_stem() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop::char::range('a', 'z'), 3..10)
            .prop_filter("no-empty-result-trap", |chars| !chars.is_empty())
            .prop_map(|chars| chars.into_iter().collect())
    }

    fn arb_surface() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop::char::range('a', 'z'), 2..16)
            .prop_map(|chars| chars.into_iter().collect())
    }

    fn sample_rules() -> Vec<MorphologicalRule> {
        vec![
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
        ]
    }

    proptest! {
        #[test]
        fn property_invert_no_empty(surface in arb_surface()) {
            for rule in sample_rules() {
                for cand in rule.invert(&surface) {
                    prop_assert!(!cand.is_empty());
                }
            }
        }

        #[test]
        fn property_invert_candidates_are_shorter(surface in arb_surface()) {
            for rule in sample_rules() {
                for cand in rule.invert(&surface) {
                    prop_assert!(cand.len() < surface.len());
                }
            }
        }

        #[test]
        fn property_apply_then_invert_recovers_stem(stem in arb_simple_stem()) {
            for rule in sample_rules() {
                let derived = rule.apply(&stem);
                if derived == stem {
                    continue;
                }
                let candidates = rule.invert(&derived);
                prop_assert!(
                    candidates.iter().any(|c| c == &stem),
                    "round-trip fail: rule {:?}, stem {stem}, derived {derived}, invert={candidates:?}",
                    rule.effect
                );
            }
        }
    }
}
