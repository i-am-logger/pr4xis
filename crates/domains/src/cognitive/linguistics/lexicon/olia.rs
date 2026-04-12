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
