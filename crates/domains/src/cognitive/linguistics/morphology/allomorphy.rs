//! Allomorphy — universal type for phonological / orthographic
//! alternations between surface variants of the same morpheme.
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
//! This file hosts only the **universal** [`AllomorphyRule`] type.
//! Language-specific rule sets — `super::english::allomorphy::
//! english_allomorphy_rules`, sibling Latin, etc. — supply the
//! concrete data.
//!
//! # Literature
//!
//! - **Spencer, Andrew (1991)** *Morphological Theory*, Blackwell,
//!   Ch. 5 — the surface/underlying distinction.
//! - **Aronoff, Mark (1976)** *Word Formation in Generative
//!   Grammar*, MIT Press — readjustment rules.
//! - **Beesley & Karttunen (2003)** *Finite-State Morphology*, CSLI,
//!   Ch. 3.5 — alternation rules in two-level morphology.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

/// Named allomorphy alternation between a surface pattern and the
/// underlying-stem pattern it can correspond to.
///
/// The `name` identifies the rule for citation and reporting. The
/// `citation` is the literature reference. The `expand` callable
/// maps a candidate bare stem (the form produced by stripping the
/// affix surface) to the *additional* candidate stems the
/// alternation predicts; the caller is expected to have already
/// considered the direct strip itself.
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
