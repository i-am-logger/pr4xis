//! English-specific morphology data.
//!
//! Universal types — [`super::MorphologicalRule`],
//! [`super::Affix`], [`super::SemanticEffect`],
//! [`super::allomorphy::AllomorphyRule`],
//! [`super::irregular::IrregularForm`],
//! [`super::irregular::IrregularKind`] — live in the parent
//! `morphology` module. This submodule is the English instance:
//! the actual rules, allomorphic alternations, and irregular-form
//! table that the lemmatizer applies when `Language::English`.
//!
//! Sibling submodules (Latin, German, …) will live alongside this
//! one, each carrying their own `rules()`/`allomorphy_rules()`/
//! `irregulars()` and consulted by the lemmatizer's
//! language-dispatch switch.
//!
//! # Praxis-way notes
//!
//! The lexical data here is currently hand-coded against published
//! references (Bauer 1983, Quirk et al. 1985, Pinker 1991). The
//! `bottom_up_loaded_not_encoded` rule says open-corpus data
//! should be loaded from a registered authoritative source at
//! build time; these tables therefore hold the role only until
//! AGID (Atkinson 2003), WordNet's morph-exception files, or
//! OLiA (Chiarcos & Sukhareva 2015) are wired into `praxis.toml`.
//! See the per-file module docs for the migration plan.

pub mod allomorphy;
pub mod irregular;
pub mod rules;

pub use allomorphy::english_allomorphy_rules;
pub use irregular::{english_irregulars, lookup_irregular};
pub use rules::english_rules;
