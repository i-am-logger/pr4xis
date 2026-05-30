#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::pos::PosTag;

// OLiA → PosTag functor.
//
// Maps OLiA (Ontologies of Linguistic Annotation) class IRIs to our PosTag
// category. This is a functor from the OLiA category to our lexical category.
//
// OLiA defines 1,300+ linguistic classes in OWL/DL. We map the ones relevant
// to morphosyntactic categories to our PosTag enum.
//
// Reference: Chiarcos & Sukhareva, OLiA (Semantic Web journal, 2015)
// OLiA reference model: http://purl.org/olia/olia.owl

const OLIA_NS: &str = "http://purl.org/olia/olia.owl#";

/// Map an OLiA class IRI to a PosTag.
/// This is the object map of the functor: OLiA → PosTag.
pub fn olia_to_pos(iri: &str) -> Option<PosTag> {
    let fragment = iri_fragment(iri)?;
    from_fragment(fragment)
}

/// Map an OLiA class fragment (the part after #) to a PosTag.
fn from_fragment(fragment: &str) -> Option<PosTag> {
    match fragment {
        // Noun hierarchy
        "Noun"
        | "CommonNoun"
        | "ProperNoun"
        | "ClassifierNoun"
        | "PartitiveNoun"
        | "QuantifierNoun"
        | "NominalizedAdjective"
        | "VerbalNoun"
        | "Gerund" => Some(PosTag::Noun),

        // Verb hierarchy
        "Verb" | "MainVerb" | "FiniteVerb" | "NonFiniteVerb" | "Infinitive" | "Participle"
        | "PresentParticiple" | "PastParticiple" | "LightVerb" => Some(PosTag::Verb),

        // Copula (OLiA distinguishes this from MainVerb)
        "Copula" => Some(PosTag::Copula),

        // Auxiliary verb hierarchy
        "AuxiliaryVerb"
        | "StrictAuxiliaryVerb"
        | "HaveAuxiliary"
        | "BeAuxiliary"
        | "ModalVerb"
        | "AspectMarkingAuxiliary"
        | "TenseMarkingAuxiliary" => Some(PosTag::Auxiliary),

        // Determiner hierarchy
        "Determiner"
        | "PossessiveDeterminer"
        | "DemonstrativeDeterminer"
        | "ReflexiveDeterminer"
        | "PronounOrDeterminer"
        | "WHDeterminer"
        | "InterrogativeDeterminer"
        | "RelativeDeterminer" => Some(PosTag::Determiner),

        // Article (subclass of Determiner in OLiA)
        "Article"
        | "DefiniteArticle"
        | "IndefiniteArticle"
        | "PartitiveArticle"
        | "IndefinitenessMarker" => Some(PosTag::Article),

        // Adjective hierarchy
        "Adjective" | "QualifierAdjective" | "RelativeAdjective" | "OrdinalAdjective"
        | "CardinalAdjective" => Some(PosTag::Adjective),

        // Adverb hierarchy
        "Adverb"
        | "RelativeAdverb"
        | "InterrogativeAdverb"
        | "DegreeAdverb"
        | "MannerAdverb"
        | "NegativeAdverb" => Some(PosTag::Adverb),

        // Pronoun hierarchy
        "Pronoun"
        | "PersonalPronoun"
        | "PossessivePronoun"
        | "ReflexivePronoun"
        | "DemonstrativePronoun"
        | "RelativePronoun"
        | "InterrogativePronoun"
        | "ReciprocalPronoun"
        | "IndefinitePronoun"
        | "WHPronoun" => Some(PosTag::Pronoun),

        // Preposition/Adposition hierarchy
        "Preposition" | "Adposition" | "Postposition" | "Circumposition" => {
            Some(PosTag::Preposition)
        }

        // Conjunction hierarchy
        "Conjunction" | "CoordinatingConjunction" | "SubordinatingConjunction" => {
            Some(PosTag::Conjunction)
        }

        // Interjection
        "Interjection" => Some(PosTag::Interjection),

        // Particle hierarchy
        "Particle"
        | "NegativeParticle"
        | "InfinitiveParticle"
        | "ComparativeParticle"
        | "VerbalParticle"
        | "QuestionParticle"
        | "FocusParticle" => Some(PosTag::Particle),

        // Numeral hierarchy
        "Numeral"
        | "CardinalNumber"
        | "OrdinalNumber"
        | "FractionNumber"
        | "MultiplicativeNumeral"
        | "CollectiveNumeral" => Some(PosTag::Numeral),

        _ => None,
    }
}

/// Extract the fragment (after #) from an IRI.
fn iri_fragment(iri: &str) -> Option<&str> {
    iri.rsplit_once('#').map(|(_, frag)| frag)
}

/// Check if an IRI belongs to the OLiA namespace.
pub fn is_olia_iri(iri: &str) -> bool {
    iri.starts_with(OLIA_NS)
}

// ── Cross-functor: SemanticEffect → OLiA ─────────────────────────────

/// Map a [`SemanticEffect`](super::super::morphology::SemanticEffect)
/// to the OLiA class fragments that realise it.
///
/// Cross-functor from praxis's morphology ontology into OLiA's
/// Morphosyntax sub-ontology. Lets downstream tooling reason about
/// each morphological rule's *grammatical category* through the
/// shared OLiA vocabulary rather than the praxis-internal enum
/// variant names.
///
/// Returns an empty `Vec` for effects with no direct OLiA mapping
/// (e.g. `SemanticEffect::PosChange` — POS changes are structural
/// edges, not OLiA feature classes; `SemanticEffect::Repetition`
/// — re-prefixing is derivational without a Morphosyntax slot).
///
/// Reference: Chiarcos & Sukhareva (2015), OLiA Morphosyntax model
/// at <http://purl.org/olia/olia.owl>.
pub fn semantic_effect_to_olia_fragments(
    effect: super::super::morphology::SemanticEffect,
) -> Vec<&'static str> {
    use super::super::morphology::SemanticEffect;
    match effect {
        // Morphological negation — closest OLiA class is the
        // operator-level `NegativeParticle` since OLiA's
        // Morphosyntax model doesn't carry a Negation feature
        // directly on the affixed word.
        SemanticEffect::Negation => vec!["NegativeParticle"],
        // No OLiA Morphosyntax class for derivational repetition.
        SemanticEffect::Repetition => vec![],
        // POS change is a category transition, not an OLiA feature.
        SemanticEffect::PosChange => vec![],
        // OLiA Number → {Singular, Plural, Dual, ...}. We emit the
        // category here; the rule's input vs output Number values
        // live separately on each instance.
        SemanticEffect::NumberChange => vec!["Number", "Singular", "Plural"],
        // OLiA Tense → {Past, Present, Future, ...}.
        SemanticEffect::TenseChange => vec!["Tense", "Past", "Present", "Future"],
        // OLiA Aspect with the Progressive class specifically.
        SemanticEffect::Progressive => vec!["Aspect", "ProgressiveAspect"],
        // OLiA covers agent-nominal derivation under
        // AgentiveNoun (in the Lexinfo extension; the base OLiA
        // does not carry it). Returned for downstream resolution.
        SemanticEffect::AgentNoun => vec!["AgentiveNoun"],
        // De-adjectival quality nouns: OLiA does not have a
        // direct class — `StativeVerb` is the closest sibling.
        // Empty until a richer derivational extension is loaded.
        SemanticEffect::QualityNoun => vec![],
    }
}

/// Get all OLiA class fragments that map to a given PosTag.
/// Inverse of the functor: PosTag → {OLiA fragments}.
pub fn pos_to_olia_fragments(pos: PosTag) -> Vec<&'static str> {
    match pos {
        PosTag::Noun => vec!["Noun", "CommonNoun", "ProperNoun"],
        PosTag::Verb => vec!["Verb", "MainVerb", "FiniteVerb"],
        PosTag::Copula => vec!["Copula"],
        PosTag::Auxiliary => vec!["AuxiliaryVerb", "StrictAuxiliaryVerb", "ModalVerb"],
        PosTag::Determiner => vec![
            "Determiner",
            "PossessiveDeterminer",
            "DemonstrativeDeterminer",
        ],
        PosTag::Article => vec!["Article", "DefiniteArticle", "IndefiniteArticle"],
        PosTag::Adjective => vec!["Adjective", "QualifierAdjective"],
        PosTag::Adverb => vec!["Adverb", "DegreeAdverb", "MannerAdverb"],
        PosTag::Pronoun => vec!["Pronoun", "PersonalPronoun", "ReflexivePronoun"],
        PosTag::Preposition => vec!["Preposition", "Adposition", "Postposition"],
        PosTag::Conjunction => vec![
            "Conjunction",
            "CoordinatingConjunction",
            "SubordinatingConjunction",
        ],
        PosTag::Interjection => vec!["Interjection"],
        PosTag::Particle => vec!["Particle", "NegativeParticle", "InfinitiveParticle"],
        PosTag::Numeral => vec!["Numeral", "CardinalNumber", "OrdinalNumber"],
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::morphology::SemanticEffect;
    use super::*;

    #[test]
    fn olia_iri_round_trip() {
        let iri = "http://purl.org/olia/olia.owl#Noun";
        assert!(is_olia_iri(iri));
        assert_eq!(olia_to_pos(iri), Some(PosTag::Noun));
    }

    #[test]
    fn pos_to_olia_round_trip() {
        for pos in [
            PosTag::Noun,
            PosTag::Verb,
            PosTag::Adjective,
            PosTag::Adverb,
        ] {
            let frags = pos_to_olia_fragments(pos);
            assert!(!frags.is_empty(), "no OLiA fragments for {pos:?}");
            for f in &frags {
                assert_eq!(
                    from_fragment(f),
                    Some(pos),
                    "fragment `{f}` did not map back to {pos:?}"
                );
            }
        }
    }

    // ── SemanticEffect → OLiA cross-functor ─────────────────────────

    #[test]
    fn number_change_maps_to_olia_number_singular_plural() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::NumberChange);
        assert!(frags.contains(&"Number"));
        assert!(frags.contains(&"Singular"));
        assert!(frags.contains(&"Plural"));
    }

    #[test]
    fn tense_change_maps_to_olia_tense_axis() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::TenseChange);
        assert!(frags.contains(&"Tense"));
        assert!(frags.contains(&"Past"));
        assert!(frags.contains(&"Present"));
        assert!(frags.contains(&"Future"));
    }

    #[test]
    fn progressive_maps_to_olia_progressive_aspect() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::Progressive);
        assert!(frags.contains(&"Aspect"));
        assert!(frags.contains(&"ProgressiveAspect"));
    }

    #[test]
    fn negation_maps_to_olia_negative_particle() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::Negation);
        assert!(frags.contains(&"NegativeParticle"));
    }

    #[test]
    fn pos_change_has_no_olia_feature_mapping() {
        // PosChange is a category-edge, not an OLiA feature.
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::PosChange);
        assert!(frags.is_empty());
    }

    #[test]
    fn repetition_has_no_olia_feature_mapping() {
        let frags = semantic_effect_to_olia_fragments(SemanticEffect::Repetition);
        assert!(frags.is_empty());
    }

    #[test]
    fn every_semantic_effect_handled() {
        // Coverage: every variant must be callable through the
        // cross-functor without panicking. Concept::variants() lists
        // the full set so adding a new variant forces a deliberate
        // mapping decision here.
        use pr4xis::category::Concept;
        for effect in SemanticEffect::variants() {
            let _ = semantic_effect_to_olia_fragments(effect);
        }
    }

    // ── Property-based laws for the OLiA cross-functor ─────────────

    use proptest::prelude::*;

    fn arb_semantic_effect() -> impl Strategy<Value = SemanticEffect> {
        prop_oneof![
            Just(SemanticEffect::Negation),
            Just(SemanticEffect::Repetition),
            Just(SemanticEffect::PosChange),
            Just(SemanticEffect::NumberChange),
            Just(SemanticEffect::TenseChange),
            Just(SemanticEffect::Progressive),
            Just(SemanticEffect::AgentNoun),
            Just(SemanticEffect::QualityNoun),
        ]
    }

    proptest! {
        #[test]
        fn property_olia_fragments_are_valid_class_names(
            effect in arb_semantic_effect(),
        ) {
            // Every emitted fragment must parse as a valid OLiA
            // class fragment via `from_fragment` — either resolving
            // to a PosTag (when the fragment is a POS class) or
            // being a documented OLiA Morphosyntax class. Empty
            // returns are valid for variants with no OLiA mapping.
            //
            // Documented OLiA Morphosyntax + structural fragments
            // we emit (per Chiarcos & Sukhareva 2015):
            const OLIA_MORPHOSYNTAX_CLASSES: &[&str] = &[
                "Number", "Singular", "Plural", "Dual",
                "Tense", "Past", "Present", "Future",
                "Aspect", "ProgressiveAspect", "PerfectiveAspect", "ImperfectiveAspect",
                "Mood", "Indicative", "Subjunctive", "Imperative",
                "Case", "Nominative", "Accusative", "Genitive", "Dative",
                "Person", "FirstPerson", "SecondPerson", "ThirdPerson",
                "Gender", "Masculine", "Feminine", "Neuter",
                "NegativeParticle", "AgentiveNoun", "StativeVerb",
            ];

            for fragment in semantic_effect_to_olia_fragments(effect) {
                let is_pos = from_fragment(fragment).is_some();
                let is_morphosyntax = OLIA_MORPHOSYNTAX_CLASSES.contains(&fragment);
                prop_assert!(
                    is_pos || is_morphosyntax,
                    "fragment `{fragment}` from {effect:?} is neither a known PosTag class nor a documented OLiA Morphosyntax class"
                );
            }
        }

        #[test]
        fn property_olia_fragments_deterministic(
            effect in arb_semantic_effect(),
        ) {
            let a = semantic_effect_to_olia_fragments(effect);
            let b = semantic_effect_to_olia_fragments(effect);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn property_olia_fragments_are_unique(
            effect in arb_semantic_effect(),
        ) {
            // No effect should emit duplicate fragments.
            let frags = semantic_effect_to_olia_fragments(effect);
            let unique: alloc::collections::BTreeSet<&&str> = frags.iter().collect();
            prop_assert_eq!(unique.len(), frags.len());
        }
    }
}
