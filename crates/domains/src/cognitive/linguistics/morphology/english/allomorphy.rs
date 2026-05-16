//! English allomorphy rules — the orthographic alternations the
//! English lemmatizer applies when inverting
//! [`super::super::MorphologicalRule`]s.
//!
//! Each rule names the alternation and cites Spencer (1991) §5.
//! The shared [`super::super::allomorphy::AllomorphyRule`] type
//! lives in the parent `morphology::allomorphy` module; this
//! submodule supplies the English-specific instances.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::super::allomorphy::AllomorphyRule;

/// The English allomorphy rule set.
pub fn english_allomorphy_rules() -> Vec<AllomorphyRule> {
    vec![
        AllomorphyRule {
            name: "silent_e_restoration",
            citation: "Spencer (1991) §5.2",
            expand: silent_e_restoration,
        },
        AllomorphyRule {
            name: "doubled_consonant_undoubling",
            citation: "Spencer (1991) §5.2",
            expand: doubled_consonant_undoubling,
        },
        AllomorphyRule {
            name: "y_to_i_alternation_past",
            citation: "Spencer (1991) §5.3",
            expand: y_to_i_alternation_past,
        },
        AllomorphyRule {
            name: "y_to_i_alternation_plural",
            citation: "Spencer (1991) §5.3",
            expand: y_to_i_alternation_plural,
        },
        AllomorphyRule {
            name: "es_to_e_restoration",
            citation: "Spencer (1991) §5.2",
            expand: es_to_e_restoration,
        },
    ]
}

fn silent_e_restoration(bare: &str, _surface: &str, suffix: &str) -> Vec<String> {
    // Spencer (1991) §5.2: silent-e elides before ANY vowel-initial
    // suffix (not just -ed/-ing/-er). Examples that need the
    // generalized version: bake + ability → bakability (stem "bak"),
    // provide + ory → provisory (stem "provis"). Inversely, the
    // bare stem from any vowel-initial suffix may have lost a silent
    // -e and we should try restoring it.
    if !suffix_is_vowel_initial(suffix) {
        return vec![];
    }
    if !ends_in_consonant(bare) {
        return vec![];
    }
    vec![format!("{bare}e")]
}

fn doubled_consonant_undoubling(bare: &str, _surface: &str, suffix: &str) -> Vec<String> {
    // Spencer (1991) §5.2: consonant-doubling before vowel-initial
    // suffixes generally (run + ing → running, big + er → bigger).
    if !suffix_is_vowel_initial(suffix) {
        return vec![];
    }
    if !ends_in_doubled_consonant(bare) {
        return vec![];
    }
    vec![bare[..bare.len() - 1].to_string()]
}

fn suffix_is_vowel_initial(suffix: &str) -> bool {
    match suffix.chars().next() {
        Some(c) => "aeiouy".contains(c.to_ascii_lowercase()),
        None => false,
    }
}

fn y_to_i_alternation_past(_bare: &str, surface: &str, suffix: &str) -> Vec<String> {
    if suffix != "ed" || !surface.ends_with("ied") || surface.len() <= 3 {
        return vec![];
    }
    let stem = &surface[..surface.len() - 3];
    vec![format!("{stem}y")]
}

fn y_to_i_alternation_plural(_bare: &str, surface: &str, suffix: &str) -> Vec<String> {
    if suffix != "s" || !surface.ends_with("ies") || surface.len() <= 3 {
        return vec![];
    }
    let stem = &surface[..surface.len() - 3];
    vec![format!("{stem}y")]
}

fn es_to_e_restoration(_bare: &str, surface: &str, suffix: &str) -> Vec<String> {
    if suffix != "s" || !surface.ends_with("es") || surface.len() <= 2 {
        return vec![];
    }
    let stem = &surface[..surface.len() - 2];
    vec![stem.to_string()]
}

fn ends_in_consonant(s: &str) -> bool {
    // Word-final 'y' counts as a vowel here, per Spencer (1991) §5.2.
    match s.chars().last() {
        Some(c) => c.is_ascii_alphabetic() && !"aeiouy".contains(c.to_ascii_lowercase()),
        None => false,
    }
}

fn ends_in_doubled_consonant(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 {
        return false;
    }
    let last = chars[chars.len() - 1];
    let prev = chars[chars.len() - 2];
    last == prev && last.is_ascii_alphabetic() && !"aeiouy".contains(last.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_set_nonempty_and_named() {
        let rules = english_allomorphy_rules();
        assert!(!rules.is_empty());
        for rule in &rules {
            assert!(!rule.name.is_empty());
            assert!(
                !rule.citation.is_empty(),
                "rule {} missing citation",
                rule.name
            );
        }
    }

    #[test]
    fn silent_e_restores_for_ed_consonant() {
        let out = silent_e_restoration("bak", "baked", "ed");
        assert_eq!(out, vec!["bake".to_string()]);
        let out = silent_e_restoration("provid", "provided", "ed");
        assert_eq!(out, vec!["provide".to_string()]);
    }

    #[test]
    fn silent_e_inert_after_vowel() {
        // 'y' is word-final-vowel per Spencer §5.2 — "played" does
        // not over-generate "playe".
        let out = silent_e_restoration("play", "played", "ed");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn silent_e_inert_for_unrelated_suffix() {
        let out = silent_e_restoration("quick", "quickly", "ly");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn doubled_consonant_undoubles_for_ing() {
        let out = doubled_consonant_undoubling("runn", "running", "ing");
        assert_eq!(out, vec!["run".to_string()]);
    }

    #[test]
    fn y_to_i_past_recovers_cry() {
        let out = y_to_i_alternation_past("cri", "cried", "ed");
        assert_eq!(out, vec!["cry".to_string()]);
    }

    #[test]
    fn y_to_i_past_inert_for_non_ied_surface() {
        // "tested" + "ed" should NOT trigger the y/i alternation —
        // it doesn't end in "ied". Closes a mutation-testing gap
        // where the function would over-generate "tesy" if the
        // ied-check were inverted.
        let out = y_to_i_alternation_past("test", "tested", "ed");
        assert!(out.is_empty(), "got {out:?}");
        let out = y_to_i_alternation_past("walk", "walked", "ed");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn y_to_i_past_inert_for_wrong_suffix() {
        // suffix == "s" should NOT trigger the past-tense rule
        // even if the surface looks plausible. Closes mutation gap.
        let out = y_to_i_alternation_past("citi", "cities", "s");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn y_to_i_past_inert_for_short_surface() {
        // 3-char surface "ied" — len <= 3 must bail. Closes
        // boundary mutation.
        let out = y_to_i_alternation_past("", "ied", "ed");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn y_to_i_plural_recovers_city() {
        let out = y_to_i_alternation_plural("citi", "cities", "s");
        assert_eq!(out, vec!["city".to_string()]);
    }

    #[test]
    fn y_to_i_plural_inert_for_non_ies_surface() {
        let out = y_to_i_alternation_plural("dog", "dogs", "s");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn y_to_i_plural_inert_for_wrong_suffix() {
        let out = y_to_i_alternation_plural("cri", "cried", "ed");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn y_to_i_plural_inert_for_short_surface() {
        let out = y_to_i_alternation_plural("", "ies", "s");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn es_restoration_recovers_box() {
        let out = es_to_e_restoration("box", "boxes", "s");
        assert_eq!(out, vec!["box".to_string()]);
    }

    #[test]
    fn es_restoration_inert_for_non_es_surface() {
        let out = es_to_e_restoration("dog", "dogs", "s");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn es_restoration_inert_for_wrong_suffix() {
        let out = es_to_e_restoration("test", "tested", "ed");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn es_restoration_inert_for_short_surface() {
        // len <= 2 surface: "es" alone must bail.
        let out = es_to_e_restoration("", "es", "s");
        assert!(out.is_empty(), "got {out:?}");
    }

    #[test]
    fn all_rules_callable_through_table() {
        for rule in english_allomorphy_rules() {
            let _ = (rule.expand)("bak", "baked", "ed");
        }
    }

    // ── Property-based laws for english_allomorphy_rules ──────────

    use proptest::prelude::*;

    fn arb_bare() -> impl Strategy<Value = String> {
        proptest::collection::vec(prop::char::range('a', 'z'), 1..10)
            .prop_map(|chars| chars.into_iter().collect())
    }

    fn arb_suffix() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("s".to_string()),
            Just("es".to_string()),
            Just("ed".to_string()),
            Just("ied".to_string()),
            Just("ies".to_string()),
            Just("ing".to_string()),
            Just("er".to_string()),
            Just("ly".to_string()),
            Just("ness".to_string()),
            Just("ability".to_string()),
        ]
    }

    proptest! {
        #[test]
        fn property_allomorphy_outputs_never_empty(
            bare in arb_bare(),
            suffix in arb_suffix(),
        ) {
            // No allomorphy rule may emit an empty string candidate.
            let surface = format!("{bare}{suffix}");
            for rule in english_allomorphy_rules() {
                for cand in (rule.expand)(&bare, &surface, &suffix) {
                    prop_assert!(
                        !cand.is_empty(),
                        "{} emitted empty candidate for bare={bare}, suffix={suffix}",
                        rule.name
                    );
                }
            }
        }

        #[test]
        fn property_allomorphy_outputs_deterministic(
            bare in arb_bare(),
            suffix in arb_suffix(),
        ) {
            let surface = format!("{bare}{suffix}");
            for rule in english_allomorphy_rules() {
                let a = (rule.expand)(&bare, &surface, &suffix);
                let b = (rule.expand)(&bare, &surface, &suffix);
                prop_assert_eq!(a, b);
            }
        }

        #[test]
        fn property_silent_e_only_fires_for_vowel_initial(
            bare in arb_bare(),
            suffix in arb_suffix(),
        ) {
            // Spencer (1991) §5.2: silent-e restoration is gated on
            // vowel-initial suffixes. For consonant-initial suffixes
            // (-ly, -ness) the rule MUST return empty regardless of
            // the bare stem's shape.
            let out = silent_e_restoration(&bare, &format!("{bare}{suffix}"), &suffix);
            let starts_with_vowel = suffix
                .chars()
                .next()
                .map(|c| "aeiouy".contains(c.to_ascii_lowercase()))
                .unwrap_or(false);
            if !starts_with_vowel {
                prop_assert!(
                    out.is_empty(),
                    "silent_e fired for consonant-initial suffix `{suffix}`: {out:?}"
                );
            }
        }

        #[test]
        fn property_every_rule_has_non_empty_citation(_ in 0..1) {
            for rule in english_allomorphy_rules() {
                prop_assert!(!rule.name.is_empty());
                prop_assert!(!rule.citation.is_empty(),
                    "rule {} missing citation", rule.name);
            }
        }
    }
}
