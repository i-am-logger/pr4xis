//! The OLiA-class → CCG-category projection, carried as a loaded
//! [`Connection`] and interpreted — the lexical-category assignment done the
//! praxis way (projection-as-data), NOT a Rust `match`.
//!
//! This is the [`wordnet_to_praxis_functor`](crate::cognitive::linguistics::english::bridge::wordnet_to_praxis_functor)
//! pattern applied to grammar: the map `OLiA-class ↦ CCG-category` is a
//! [`GeneratorAction::Functor`] whose rows load from a cited CCGbank artifact
//! (`data/grammar/olia-ccg-categories.tsv`), content-addressed and re-emittable,
//! NOT a hand-written table in Rust. The class identity is a loaded OLiA Concept
//! (resolved against [`olia::reference_model`]); the category is its standard
//! CCGbank notation, lowered to a [`LambekType`] by the
//! [`notation parser`](super::notation_parser) — the one codec crossing.
//!
//! ## Why not `apply`
//!
//! [`apply`](pr4xis_runtime::apply) relabels an archive's node/edge *kinds* by a
//! string lookup; it cannot mint a structured category from a lexical lookup and
//! there is no word-as-node archive to relabel. So this functor's `kind` is
//! `"LexicalAssignment"` (not a relabeling `"Faithful"`), and its interpreter is
//! [`assign_categories`] — which consults the same `map_object` data `apply`
//! would, but produces a [`LambekType`] via the parser instead of relabeling.
//!
//! Language-agnostic in its KEY (universal OLiA classes), grammar-specific in
//! its VALUE (CCGbank is English-grammar-grounded); only the word→class binding
//! (`function-words/english.xml`) is language data.

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use super::notation_parser::parse_category;
use super::types::LambekType;

#[cfg(feature = "std")]
use crate::cognitive::linguistics::lexicon::olia;
#[cfg(feature = "std")]
use pr4xis_runtime::connection::{Connection, GeneratorAction};

/// The loaded OLiA→CCG lexical-category functor, cached for the process.
///
/// Each row is validated at load (a build-time invariant — the artifact ships
/// with praxis): the OLiA class must resolve in the loaded Reference Model
/// ([`olia::is_loaded_class`]) and the CCG category must parse. A failure panics
/// rather than silently dropping a generator.
#[cfg(feature = "std")]
pub fn olia_ccg_functor() -> &'static Connection {
    use std::sync::OnceLock;
    static FUNCTOR: OnceLock<Connection> = OnceLock::new();
    FUNCTOR.get_or_init(|| {
        const TSV: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/grammar/olia-ccg-categories.tsv"
        ));
        let mut map_object: Vec<(String, String)> = Vec::new();
        for line in TSV.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut cols = line.split('\t').map(str::trim);
            let fragment = cols.next().expect("a row carries an OLiA class fragment");
            let notation = cols.next().expect("a row carries a CCG category notation");
            assert!(
                olia::is_loaded_class(fragment),
                "OLiA class {fragment:?} does not resolve in the loaded Reference Model"
            );
            assert!(
                parse_category(notation).is_some(),
                "CCG category {notation:?} for {fragment:?} does not parse"
            );
            map_object.push((olia::class_iri(fragment), notation.to_string()));
        }
        Connection {
            kind: "LexicalAssignment".to_string(),
            source: "OLiA".to_string(),
            target: "CCG".to_string(),
            action: GeneratorAction::Functor {
                map_object,
                map_morphism: Vec::new(),
            },
            laws: vec!["functor".to_string()],
        }
    })
}

/// Interpret the functor: the CCG categories a loaded OLiA class IRI projects
/// to. Consults the `map_object` rows (the `apply` idiom — match the source
/// generator) and lowers each target notation through the parser. A class with
/// several rows yields several readings (the chart explores them); an unmapped
/// class yields none.
#[cfg(feature = "std")]
pub fn assign_categories(olia_iri: &str, functor: &Connection) -> Vec<LambekType> {
    let GeneratorAction::Functor { map_object, .. } = &functor.action else {
        return Vec::new();
    };
    map_object
        .iter()
        .filter(|(source, _)| source == olia_iri)
        .filter_map(|(_, notation)| parse_category(notation))
        .collect()
}

/// The CCG categories the OLiA class named by `fragment` projects to, via the
/// loaded functor — the one call the lexicon/tokenizer needs. Empty on
/// `no_std` (the loaded functor is `std`-only; wh-words degrade to the default
/// type, a tracked transitional residue).
pub fn categories_for_class(fragment: &str) -> Vec<LambekType> {
    #[cfg(feature = "std")]
    {
        assign_categories(&olia::class_iri(fragment), olia_ccg_functor())
    }
    #[cfg(not(feature = "std"))]
    {
        let _ = fragment;
        Vec::new()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::types::{reduce, svo};

    #[test]
    fn the_functor_loads_and_is_a_lexical_assignment() {
        let f = olia_ccg_functor();
        assert_eq!(f.kind, "LexicalAssignment");
        assert_eq!(f.source, "OLiA");
        assert_eq!(f.target, "CCG");
        let GeneratorAction::Functor { map_object, .. } = &f.action else {
            panic!("a lexical-assignment functor is a GeneratorAction::Functor");
        };
        assert_eq!(
            map_object.len(),
            3,
            "the interrogative slice has three generators"
        );
        // Keys are full OLiA Reference-Model IRIs, not bare fragments.
        assert!(
            map_object
                .iter()
                .all(|(k, _)| k.starts_with("http://purl.org/olia/olia.owl#"))
        );
    }

    #[test]
    fn the_projection_recovers_the_cited_categories() {
        assert_eq!(
            categories_for_class("InterrogativePronoun"),
            vec![svo::wh_what()]
        );
        assert_eq!(
            categories_for_class("InterrogativeDeterminer"),
            vec![svo::wh_determiner()]
        );
        assert_eq!(
            categories_for_class("InterrogativeAdverb"),
            vec![svo::wh_adverb()]
        );
        // An unmapped class projects nothing (fails closed).
        assert!(categories_for_class("PersonalPronoun").is_empty());
    }

    #[test]
    fn projected_categories_reduce_real_constituents() {
        // The slice earns its keep: the categories selected THROUGH THE LOADED
        // FUNCTOR (not a Rust constant) reduce real constituents.
        let predicate = LambekType::left_div(LambekType::np(), LambekType::s()); // NP\S

        // pronoun + (subject-missing predicate) → S[wq]   ("what is a dog")
        let pron = categories_for_class("InterrogativePronoun").remove(0);
        assert_eq!(reduce(&pron, &predicate), Some(LambekType::wq()));

        // determiner + N → pronoun category, then + predicate → S[wq]   ("which dog is a mammal")
        let det = categories_for_class("InterrogativeDeterminer").remove(0);
        let after_n = reduce(&det, &LambekType::n()).expect("det + N");
        assert_eq!(reduce(&after_n, &predicate), Some(LambekType::wq()));

        // adverb + (S[q]/PP clause) → S[wq]   ("where is the dog")
        let adv = categories_for_class("InterrogativeAdverb").remove(0);
        let clause =
            reduce(&svo::question_copula_pp(), &LambekType::np()).expect("copula + NP → S[q]/PP");
        assert_eq!(reduce(&adv, &clause), Some(LambekType::wq()));
        // ...and the adverb does NOT reduce a bare predicate (genuinely distinct).
        assert_eq!(reduce(&adv, &predicate), None);
    }
}
