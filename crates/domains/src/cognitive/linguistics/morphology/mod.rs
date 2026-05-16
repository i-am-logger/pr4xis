#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod allomorphy;
pub mod irregular;
pub mod lemmatizer;
pub mod tense;

use pr4xis::category::Concept;

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

impl Concept for SemanticEffect {
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
    /// candidate stems that could have produced it.
    ///
    /// Returns an empty `Vec` when `surface` does not match this
    /// rule's affix pattern at all. Returns one candidate for the
    /// straight strip plus additional candidates for the documented
    /// English allomorphic variants:
    /// silent-e restoration (`bak` + `e` → `bake` from `baked`),
    /// doubled-consonant un-doubling (`runn` → `run` from `running`),
    /// y/i alternation (`cri` + `y` → `cry` from `cried`,
    /// `citi` + `y` → `city` from `cities`).
    ///
    /// Cite: Beesley & Karttunen (2003) *Finite-State Morphology*,
    /// CSLI, Ch. 3 — bidirectional FSTs treat forward (`apply`) and
    /// inverse (`invert`) as two readings of the same relation.
    /// Spencer (1991) *Morphological Theory*, Blackwell, Ch. 5 — the
    /// English allomorphy patches encoded below.
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
        let mut candidates: Vec<String> = Vec::new();

        // Direct stripping.
        if surface.len() > suf.len() && surface.ends_with(suf) {
            let bare = &surface[..surface.len() - suf.len()];
            candidates.push(bare.to_string());

            // Silent-e restoration (Spencer 1991 §5.2):
            // bake + ed → baked (drop-e then add suffix). When the
            // suffix is -ed / -ing / -er and the bare stem ends in
            // a consonant, the original stem may end in -e.
            let restore_e = matches!(suf, "ed" | "ing" | "er") && bare_ends_in_consonant(bare);
            if restore_e {
                candidates.push(format!("{bare}e"));
            }

            // Doubled-consonant un-doubling (Spencer 1991 §5.2):
            // run + ing → running. When the bare stem ends in a
            // doubled consonant, the original stem may end in a
            // single one. -ed / -ing / -er only.
            if matches!(suf, "ed" | "ing" | "er") && ends_in_doubled_consonant(bare) {
                candidates.push(bare[..bare.len() - 1].to_string());
            }
        }

        // y/i alternation (Spencer 1991 §5.3):
        // cried → cry (from -ed rule), cities → city (from -s rule).
        // The bare stem "cri" / "citi" is replaced by stem-without-i + "y".
        if suf == "ed" && surface.ends_with("ied") && surface.len() > 3 {
            let bare = &surface[..surface.len() - 3];
            candidates.push(format!("{bare}y"));
        }
        if suf == "s" && surface.ends_with("ies") && surface.len() > 3 {
            let bare = &surface[..surface.len() - 3];
            candidates.push(format!("{bare}y"));
        }

        // -es → e restoration: boxes → box (already covered by
        // direct -s strip yielding "boxe"); also try the un-restored
        // form so "box" is a candidate.
        if suf == "s" && surface.ends_with("es") && surface.len() > 2 {
            let bare = &surface[..surface.len() - 2];
            candidates.push(bare.to_string());
        }

        candidates
    }
}

fn bare_ends_in_consonant(bare: &str) -> bool {
    // Word-final 'y' counts as a vowel here, per Spencer (1991) §5.2 —
    // "played" → "play" must not over-generate "playe".
    match bare.chars().last() {
        Some(c) => c.is_ascii_alphabetic() && !"aeiouy".contains(c.to_ascii_lowercase()),
        None => false,
    }
}

fn ends_in_doubled_consonant(bare: &str) -> bool {
    let chars: Vec<char> = bare.chars().collect();
    if chars.len() < 2 {
        return false;
    }
    let last = chars[chars.len() - 1];
    let prev = chars[chars.len() - 2];
    last == prev && last.is_ascii_alphabetic() && !"aeiouy".contains(last.to_ascii_lowercase())
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

    // ── invert ───────────────────────────────────────────────────────

    fn rule_by_suffix(text: &str) -> MorphologicalRule {
        english_rules()
            .into_iter()
            .find(|r| matches!(&r.affix, Affix::Suffix(s) if s.text == text))
            .expect("rule exists")
    }

    fn rule_by_prefix(text: &str) -> MorphologicalRule {
        english_rules()
            .into_iter()
            .find(|r| matches!(&r.affix, Affix::Prefix(p) if p.text == text))
            .expect("rule exists")
    }

    #[test]
    fn invert_plural_simple() {
        // rights → right
        let s_rule = rule_by_suffix("s");
        let candidates = s_rule.invert("rights");
        assert!(
            candidates.contains(&"right".to_string()),
            "expected `right` in {candidates:?}"
        );
    }

    #[test]
    fn invert_plural_y_to_ies() {
        // cities → city, remedies → remedy
        let s_rule = rule_by_suffix("s");
        let candidates = s_rule.invert("cities");
        assert!(
            candidates.contains(&"city".to_string()),
            "expected `city` in {candidates:?}"
        );
        let candidates = s_rule.invert("remedies");
        assert!(
            candidates.contains(&"remedy".to_string()),
            "expected `remedy` in {candidates:?}"
        );
    }

    #[test]
    fn invert_past_simple() {
        // walked → walk
        let ed_rule = rule_by_suffix("ed");
        let candidates = ed_rule.invert("walked");
        assert!(
            candidates.contains(&"walk".to_string()),
            "expected `walk` in {candidates:?}"
        );
    }

    #[test]
    fn invert_past_silent_e() {
        // filed → file, baked → bake
        let ed_rule = rule_by_suffix("ed");
        let candidates = ed_rule.invert("filed");
        assert!(
            candidates.contains(&"file".to_string()),
            "expected `file` in {candidates:?}"
        );
        let candidates = ed_rule.invert("baked");
        assert!(
            candidates.contains(&"bake".to_string()),
            "expected `bake` in {candidates:?}"
        );
    }

    #[test]
    fn invert_past_ied_to_y() {
        // cried → cry, testified → testify
        let ed_rule = rule_by_suffix("ed");
        let candidates = ed_rule.invert("cried");
        assert!(
            candidates.contains(&"cry".to_string()),
            "expected `cry` in {candidates:?}"
        );
        let candidates = ed_rule.invert("testified");
        assert!(
            candidates.contains(&"testify".to_string()),
            "expected `testify` in {candidates:?}"
        );
    }

    #[test]
    fn invert_progressive_silent_e() {
        // providing → provide
        let ing_rule = rule_by_suffix("ing");
        let candidates = ing_rule.invert("providing");
        assert!(
            candidates.contains(&"provide".to_string()),
            "expected `provide` in {candidates:?}"
        );
    }

    #[test]
    fn invert_progressive_doubled_consonant() {
        // running → run, sitting → sit
        let ing_rule = rule_by_suffix("ing");
        let candidates = ing_rule.invert("running");
        assert!(
            candidates.contains(&"run".to_string()),
            "expected `run` in {candidates:?}"
        );
        let candidates = ing_rule.invert("sitting");
        assert!(
            candidates.contains(&"sit".to_string()),
            "expected `sit` in {candidates:?}"
        );
    }

    #[test]
    fn invert_prefix_negation() {
        // unhappy → happy
        let un_rule = rule_by_prefix("un");
        let candidates = un_rule.invert("unhappy");
        assert_eq!(candidates, vec!["happy".to_string()]);
    }

    #[test]
    fn invert_returns_empty_when_no_match() {
        let s_rule = rule_by_suffix("s");
        assert!(s_rule.invert("dog").is_empty());
        let ed_rule = rule_by_suffix("ed");
        assert!(ed_rule.invert("walk").is_empty());
    }

    #[test]
    fn invert_never_returns_empty_string_for_short_input() {
        // "s" alone — len == suffix len — must not produce "".
        let s_rule = rule_by_suffix("s");
        let candidates = s_rule.invert("s");
        assert!(
            !candidates.iter().any(|c| c.is_empty()),
            "got empty candidate from {:?}",
            candidates
        );
    }

    // ── proptest property-based laws for invert ─────────────────────
    //
    // Beesley & Karttunen (2003) Ch. 3 — the bidirectional-FST
    // identity. For every rule the inverse must:
    //   (a) be partial (return [] when the surface form has no match)
    //   (b) never return an empty candidate
    //   (c) return only candidates strictly shorter than the surface
    //   (d) recover the original stem when applied to apply(stem)
    //       (with allomorphy-free stems)

    use proptest::prelude::*;

    fn arb_simple_stem() -> impl Strategy<Value = String> {
        // 3..10 lowercase ASCII consonants and vowels, but ending in
        // a configuration that doesn't trigger allomorphy (avoid e,
        // y, doubled-final-consonants). Keeps the round-trip law
        // checkable without modelling all of English orthography.
        proptest::collection::vec(prop::char::range('a', 'z'), 3..10)
            .prop_filter("no-allomorphy-trap", |chars| {
                if chars.is_empty() {
                    return false;
                }
                let last = chars[chars.len() - 1];
                if "ey".contains(last) {
                    return false;
                }
                if chars.len() >= 2 && chars[chars.len() - 2] == last {
                    return false;
                }
                true
            })
            .prop_map(|chars| chars.into_iter().collect())
    }

    fn arb_surface() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop::char::range('a', 'z'), 2..16)
            .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        #[test]
        fn property_invert_no_empty(surface in arb_surface()) {
            for rule in english_rules() {
                for cand in rule.invert(&surface) {
                    prop_assert!(!cand.is_empty(),
                        "rule {:?} produced empty candidate from {surface}",
                        rule.effect);
                }
            }
        }

        #[test]
        fn property_invert_candidates_are_shorter(surface in arb_surface()) {
            for rule in english_rules() {
                for cand in rule.invert(&surface) {
                    prop_assert!(cand.len() < surface.len(),
                        "rule {:?} produced same-or-longer candidate {cand} from {surface}",
                        rule.effect);
                }
            }
        }

        #[test]
        fn property_apply_then_invert_recovers_stem(stem in arb_simple_stem()) {
            // For every rule whose apply changes the stem (i.e., the
            // affix is non-empty), invert must recover the stem.
            for rule in english_rules() {
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

    #[test]
    fn apply_then_invert_recovers_stem_for_regular_forms() {
        // The fundamental Beesley & Karttunen (2003) Ch. 3 property:
        // for any stem that the rule's apply produces a derived form
        // for, invert(apply(stem)) must contain stem as a candidate.
        let cases: Vec<(&str, &str)> = vec![
            ("s", "right"),
            ("s", "dog"),
            ("ed", "walk"),
            ("ing", "walk"),
            ("er", "teach"),
            ("ly", "quick"),
            ("ness", "happy"),
            ("un", "happy"),
            ("re", "do"),
        ];
        for (affix, stem) in cases {
            let rule = english_rules()
                .into_iter()
                .find(|r| match &r.affix {
                    Affix::Prefix(p) => p.text == affix,
                    Affix::Suffix(s) => s.text == affix,
                })
                .unwrap();
            let derived = rule.apply(stem);
            let candidates = rule.invert(&derived);
            assert!(
                candidates.iter().any(|c| c == stem),
                "apply({affix}, {stem}) = {derived}; invert = {candidates:?}; expected to contain `{stem}`"
            );
        }
    }
}
