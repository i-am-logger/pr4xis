//! The OLiA-class → CCG-category projection, carried as a loaded
//! [`Connection`] and interpreted — the lexical-category assignment done the
//! praxis way (projection-as-data), NOT a Rust `match`.
//!
//! This is the WordNet→praxis functor pattern (the committed
//! `english_functor.prx`, loaded by
//! [`english_runtime_ontology`](crate::cognitive::linguistics::english::bridge::english_runtime_ontology))
//! applied to grammar: the map `OLiA-class ↦ CCG-category` is a
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

use crate::cognitive::linguistics::lexicon::olia;
use pr4xis_runtime::connection::{Connection, GeneratorAction};

/// Build the loaded OLiA→CCG functor from the bundled cited TSV.
///
/// `no_std`-capable: the functor's LOOKUP (fragment → category) needs no OLiA —
/// only its build-time VALIDATION does (that each key is a real loaded OLiA
/// class, the O1 grounding), which is gated to `std`. Under `no_std` the
/// fragment is carried as a wire string, unvalidated — a tracked degradation,
/// the same one the rest of the `std`-only OLiA path has. The CCG category must
/// always parse (a build invariant on both paths).
fn build_functor() -> Connection {
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
        // Optional 2nd OLiA coordinate (a ValencyFeature class) for
        // product-keyed categories (verbs).
        let valency = cols.next().filter(|v| !v.is_empty());
        assert!(
            parse_category(notation).is_some(),
            "CCG category {notation:?} for {fragment:?} does not parse"
        );
        #[cfg(feature = "std")]
        {
            assert!(
                olia::is_loaded_class(fragment),
                "OLiA class {fragment:?} does not resolve in the loaded Reference Model"
            );
            if let Some(v) = valency {
                assert!(
                    olia::is_loaded_class(v),
                    "OLiA valency class {v:?} does not resolve in the loaded Reference Model"
                );
            }
        }
        // The functor key is the conjunction of OLiA coordinate IRIs,
        // space-joined (the wire form of the product key).
        let key = match valency {
            Some(v) => alloc::format!("{} {}", olia::class_iri(fragment), olia::class_iri(v)),
            None => olia::class_iri(fragment),
        };
        map_object.push((key, notation.to_string()));
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
}

/// The loaded OLiA→CCG lexical-category functor, cached for the process (`std`).
#[cfg(feature = "std")]
pub fn olia_ccg_functor() -> &'static Connection {
    use std::sync::OnceLock;
    static FUNCTOR: OnceLock<Connection> = OnceLock::new();
    FUNCTOR.get_or_init(build_functor)
}

/// Interpret the functor: the CCG categories a loaded OLiA class IRI projects
/// to. Consults the `map_object` rows (the `apply` idiom — match the source
/// generator) and lowers each target notation through the parser. A class with
/// several rows yields several readings (the chart explores them); an unmapped
/// class yields none.
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
    categories_for_class_valency(fragment, None)
}

/// The CCG categories an OLiA class projects to, optionally refined by a second
/// OLiA coordinate (a ValencyFeature class — `Transitive`/`Intransitive`/
/// `Ditransitive` for verbs), via the loaded functor. The product key is the
/// space-joined conjunction of the coordinate IRIs (the `derive_lambek(arity,
/// result)` pattern as loaded data). Works on `no_std` (rebuilds the small
/// functor by value, uncached, since there is no global `OnceLock`).
pub fn categories_for_class_valency(fragment: &str, valency: Option<&str>) -> Vec<LambekType> {
    let key = match valency {
        Some(v) => alloc::format!("{} {}", olia::class_iri(fragment), olia::class_iri(v)),
        None => olia::class_iri(fragment),
    };
    #[cfg(feature = "std")]
    {
        assign_categories(&key, olia_ccg_functor())
    }
    #[cfg(not(feature = "std"))]
    {
        assign_categories(&key, &build_functor())
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
        // The functor carries the WHOLE lexical-category projection (open +
        // closed class + interrogatives). Spot-check representative generators
        // across the classes rather than a brittle count (rule 5).
        let keys: alloc::vec::Vec<&str> = map_object.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.iter().any(|k| k.ends_with("#Noun")), "open-class noun");
        assert!(
            keys.iter()
                .any(|k| k.contains("#Verb") && k.contains("#Transitive")),
            "verb × valency product key"
        );
        assert!(
            keys.iter().any(|k| k.ends_with("#InterrogativeAdverb")),
            "interrogative generator"
        );
        // Keys are full OLiA Reference-Model IRIs, not bare fragments.
        assert!(
            map_object
                .iter()
                .all(|(k, _)| k.starts_with("http://purl.org/olia/olia.owl#"))
        );
    }

    /// THE ORACLE: every loaded POS row projects to exactly the `svo::`
    /// constructor the old `pos_to_lambek` match produced — pinning the TSV
    /// notation strings to the categories, so a wrong-notation row fails HERE
    /// (before any sentence test) and the `pos_to_lambek` migration is proven
    /// behavior-preserving. Bare-`S` (not `S[dcl]`) is the trap this guards.
    #[test]
    fn the_pos_rows_recover_the_svo_categories() {
        assert_eq!(categories_for_class("Noun"), vec![svo::noun()]);
        assert_eq!(
            categories_for_class_valency("Verb", Some("Intransitive")),
            vec![svo::intransitive_verb()]
        );
        assert_eq!(
            categories_for_class_valency("Verb", Some("Transitive")),
            vec![svo::transitive_verb()]
        );
        assert_eq!(
            categories_for_class_valency("Verb", Some("Ditransitive")),
            vec![svo::ditransitive_verb()]
        );
        assert_eq!(categories_for_class("Determiner"), vec![svo::determiner()]);
        assert_eq!(categories_for_class("Numeral"), vec![svo::determiner()]);
        assert_eq!(categories_for_class("Adjective"), vec![svo::adjective()]);
        assert_eq!(categories_for_class("Adverb"), vec![svo::adverb()]);
        assert_eq!(
            categories_for_class("Preposition"),
            vec![svo::preposition()]
        );
        assert_eq!(categories_for_class("Pronoun"), vec![svo::proper_noun()]);
        // PROVISIONAL rows preserve the pre-migration defaults.
        assert_eq!(
            categories_for_class("AuxiliaryVerb"),
            vec![svo::intransitive_verb()]
        );
        assert_eq!(categories_for_class("Conjunction"), vec![svo::noun()]);
        assert_eq!(categories_for_class("Interjection"), vec![svo::noun()]);
        assert_eq!(categories_for_class("Particle"), vec![svo::adverb()]);
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
