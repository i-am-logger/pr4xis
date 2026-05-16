//! Allomorphy — phonological / orthographic alternations between
//! surface variants of the same morpheme.
//!
//! Allomorphy is the relation that holds between two surface forms
//! of one underlying morpheme: the plural morpheme {-z} appears as
//! `/-s/` (cats), `/-z/` (dogs), `/-ɪz/` (boxes); the past-tense
//! morpheme {-d} appears as `/-t/` (walked), `/-d/` (loved), `/-ɪd/`
//! (waited). Orthographically the alternations show up as `-s` vs
//! `-es`, `-y` vs `-ies`, etc. — what English-speaker schoolbooks
//! call "spelling rules". They are the same conceptual layer as
//! phonological allomorphy, with the underlying form being the
//! orthographic stem.
//!
//! Praxis encodes allomorphy as a small registry of typed
//! [`AllomorphyRule`]s that the [`MorphologicalRule::invert`] path
//! consults. Each rule names the alternation, cites the literature,
//! and exposes a deterministic `candidate_stems` mapping from a
//! surface to the underlying forms it could correspond to.
//!
//! # Literature
//!
//! - **Spencer, Andrew (1991)** *Morphological Theory*, Blackwell,
//!   Ch. 5 — the surface/underlying distinction and the canonical
//!   English allomorphy patches (silent-e, doubling, y/i).
//! - **Aronoff, Mark (1976)** *Word Formation in Generative
//!   Grammar*, MIT Press — readjustment rules.
//! - **Beesley & Karttunen (2003)** *Finite-State Morphology*, CSLI,
//!   Ch. 3.5 — alternation rules in two-level morphology.
//!
//! # Praxis-way notes
//!
//! Each rule is data, not a hand-coded `match`: the public
//! [`english_allomorphy_rules`] function returns a `Vec` so callers
//! iterate, cite, and combine without conditional code paths.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Named allomorphy alternation between a surface pattern and the
/// underlying-stem pattern it can correspond to.
///
/// The `name` identifies the rule for citation and reporting. The
/// `citation` is the literature reference (Spencer 1991 §5 in the
/// English-language case). The `expand` callable maps a candidate
/// bare stem (the form produced by stripping the affix surface) to
/// the *additional* candidate stems the alternation predicts; the
/// caller is expected to have already considered the direct strip
/// itself.
#[derive(Clone)]
pub struct AllomorphyRule {
    pub name: &'static str,
    pub citation: &'static str,
    pub expand: fn(bare: &str, surface: &str, suffix: &str) -> Vec<String>,
}

impl core::fmt::Debug for AllomorphyRule {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AllomorphyRule")
            .field("name", &self.name)
            .field("citation", &self.citation)
            .finish()
    }
}

impl PartialEq for AllomorphyRule {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.citation == other.citation
    }
}

impl Eq for AllomorphyRule {}

/// The canonical English allomorphy rules consulted by
/// [`crate::cognitive::linguistics::morphology::MorphologicalRule::invert`].
///
/// Adding a new alternation = adding a [`AllomorphyRule`] here; no
/// callers change.
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
        // The rule only fires after a consonant. A vowel-ending bare
        // stem (e.g. "play") doesn't trigger silent-e restoration.
        let out = silent_e_restoration("play", "played", "ed");
        assert!(
            out.is_empty(),
            "vowel-ending stem should not restore-e: {out:?}"
        );
    }

    #[test]
    fn silent_e_inert_for_unrelated_suffix() {
        // The rule fires only for -ed/-ing/-er. -ly leaves the bare stem alone.
        let out = silent_e_restoration("quick", "quickly", "ly");
        assert!(out.is_empty(), "ly suffix should not restore-e: {out:?}");
    }

    #[test]
    fn doubled_consonant_undoubles_for_ing() {
        let out = doubled_consonant_undoubling("runn", "running", "ing");
        assert_eq!(out, vec!["run".to_string()]);
        let out = doubled_consonant_undoubling("sitt", "sitting", "ing");
        assert_eq!(out, vec!["sit".to_string()]);
    }

    #[test]
    fn y_to_i_past_recovers_cry() {
        let out = y_to_i_alternation_past("cri", "cried", "ed");
        assert_eq!(out, vec!["cry".to_string()]);
        let out = y_to_i_alternation_past("testifi", "testified", "ed");
        assert_eq!(out, vec!["testify".to_string()]);
    }

    #[test]
    fn y_to_i_plural_recovers_city() {
        let out = y_to_i_alternation_plural("citi", "cities", "s");
        assert_eq!(out, vec!["city".to_string()]);
        let out = y_to_i_alternation_plural("remedi", "remedies", "s");
        assert_eq!(out, vec!["remedy".to_string()]);
    }

    #[test]
    fn es_restoration_recovers_box() {
        let out = es_to_e_restoration("box", "boxes", "s");
        assert_eq!(out, vec!["box".to_string()]);
    }

    #[test]
    fn all_rules_callable_through_table() {
        // Every registered rule must be invokable with the canonical
        // signature without panicking.
        for rule in english_allomorphy_rules() {
            let _ = (rule.expand)("bak", "baked", "ed");
        }
    }
}
