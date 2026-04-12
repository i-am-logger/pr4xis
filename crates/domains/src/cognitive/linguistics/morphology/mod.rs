pub mod tense;

use pr4xis::category::Entity;

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

impl Entity for SemanticEffect {
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
}

/// English morphological rules.
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
    fn prefix_negation() {
        let rules = english_rules();
        let un_rule = rules
            .iter()
            .find(|r| r.effect == SemanticEffect::Negation)
            .unwrap();
        assert_eq!(un_rule.apply("happy"), "unhappy");
        assert_eq!(un_rule.input_pos, PosTag::Adjective);
        assert_eq!(un_rule.output_pos, PosTag::Adjective);
    }

    #[test]
    fn suffix_adverb() {
        let rules = english_rules();
        let ly_rule = rules
            .iter()
            .find(|r| r.effect == SemanticEffect::PosChange)
            .unwrap();
        assert_eq!(ly_rule.apply("quick"), "quickly");
        assert_eq!(ly_rule.input_pos, PosTag::Adjective);
        assert_eq!(ly_rule.output_pos, PosTag::Adverb);
    }

    #[test]
    fn suffix_plural() {
        let rules = english_rules();
        let s_rule = rules
            .iter()
            .find(|r| r.effect == SemanticEffect::NumberChange)
            .unwrap();
        assert_eq!(s_rule.apply("dog"), "dogs");
    }

    #[test]
    fn suffix_past_tense() {
        let rules = english_rules();
        let ed_rule = rules
            .iter()
            .find(|r| r.effect == SemanticEffect::TenseChange)
            .unwrap();
        assert_eq!(ed_rule.apply("walk"), "walked");
    }

    #[test]
    fn suffix_agent() {
        let rules = english_rules();
        let er_rule = rules
            .iter()
            .find(|r| r.effect == SemanticEffect::AgentNoun)
            .unwrap();
        assert_eq!(er_rule.apply("teach"), "teacher");
        assert_eq!(er_rule.input_pos, PosTag::Verb);
        assert_eq!(er_rule.output_pos, PosTag::Noun);
    }

    #[test]
    fn english_has_8_rules() {
        assert_eq!(english_rules().len(), 8);
    }

    #[test]
    fn negation_connects_to_opposition() {
        // un- creates opposition: happy ↔ unhappy
        // This is the OppositionDef in action at the morphological level
        let rules = english_rules();
        let neg = rules
            .iter()
            .find(|r| r.effect == SemanticEffect::Negation)
            .unwrap();
        assert!(matches!(neg.affix, Affix::Prefix(_)));
    }
}
