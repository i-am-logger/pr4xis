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
    if !matches!(suffix, "ed" | "ing" | "er") {
        return vec![];
    }
    if !ends_in_consonant(bare) {
        return vec![];
    }
    vec![format!("{bare}e")]
}

fn doubled_consonant_undoubling(bare: &str, _surface: &str, suffix: &str) -> Vec<String> {
    if !matches!(suffix, "ed" | "ing" | "er") {
        return vec![];
    }
    if !ends_in_doubled_consonant(bare) {
        return vec![];
    }
    vec![bare[..bare.len() - 1].to_string()]
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
    fn y_to_i_plural_recovers_city() {
        let out = y_to_i_alternation_plural("citi", "cities", "s");
        assert_eq!(out, vec!["city".to_string()]);
    }

    #[test]
    fn es_restoration_recovers_box() {
        let out = es_to_e_restoration("box", "boxes", "s");
        assert_eq!(out, vec!["box".to_string()]);
    }

    #[test]
    fn all_rules_callable_through_table() {
        for rule in english_allomorphy_rules() {
            let _ = (rule.expand)("bak", "baked", "ed");
        }
    }
}
