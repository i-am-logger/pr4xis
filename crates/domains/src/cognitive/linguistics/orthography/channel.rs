use pr4xis::category::Entity;
use pr4xis::define_ontology;

// Noisy Channel Model — spelling correction as categorical adjunction.
//
// Shannon (1948): communication through a noisy channel.
// Kernighan, Church & Gale (1990): applied to spelling correction.
// Brill & Moore (2000): string-to-string partition model.
//
// The channel and its Bayesian inverse form an ADJUNCTION, not an isomorphism:
//   F: Lang → Obs  (the channel functor — words become misspellings)
//   G: Obs → Lang  (Bayesian right adjoint — correction)
//   G∘F ≠ Id       (information loss through channel)
//   η: Id → G∘F    (unit: the "correction accuracy" natural transformation)
//
// This is NOT a simple inverse functor because:
// - The channel destroys information (many words can produce the same misspelling)
// - Correction is probabilistic (argmax, not exact inverse)
// - G∘F approaches Id as the language model and error model improve

/// Concepts in the noisy channel model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ChannelConcept {
    /// A word in the language, with its prior probability P(w).
    Word,
    /// An observed string (possibly misspelled).
    Observation,
    /// The error model P(x|w) — probability of misspelling x given intended word w.
    /// Parameterized by confusion matrices (Kernighan, Church & Gale 1990).
    ErrorModel,
    /// The language model P(w) — prior probability of the word.
    LanguageModel,
    /// The correction: argmax_w P(x|w) * P(w).
    Correction,
    /// A confusion matrix — edit probabilities for a specific operation type.
    /// del[x,y], ins[x,y], sub[x,y], trans[x,y].
    ConfusionMatrix,
}

define_ontology! {
    /// Noisy Channel Model — spelling correction as categorical adjunction (Shannon 1948).
    pub ChannelOntology for ChannelCategory {
        concepts: ChannelConcept,
        relation: ChannelRelation,
        kind: ChannelRelationKind,
        kinds: [
            /// Word → Observation: the channel corrupts (F: Lang → Obs).
            Corrupts,
            /// Observation → Word: Bayesian inverse correction (G: Obs → Lang).
            Corrects,
            /// ErrorModel parameterizes the corruption.
            Parameterizes,
            /// LanguageModel provides prior probabilities.
            Weights,
            /// ConfusionMatrix provides edit-level probabilities.
            Provides,
            /// Correction uses both ErrorModel and LanguageModel.
            Uses,
        ],
        edges: [
            // The channel: Word → Observation (corruption)
            (Word, Observation, Corrupts),
            // The inverse: Observation → Word (correction / adjoint)
            (Observation, Word, Corrects),
            // ErrorModel parameterizes the channel
            (ErrorModel, Observation, Parameterizes),
            // LanguageModel weights the prior
            (LanguageModel, Word, Weights),
            // ConfusionMatrix provides edit probabilities to ErrorModel
            (ConfusionMatrix, ErrorModel, Provides),
            // Correction uses both models
            (Correction, ErrorModel, Uses),
            (Correction, LanguageModel, Uses),
            // The adjunction: Observation → Correction → Word
            (Correction, Word, Corrects),
        ],
        composed: [
            // The adjunction: G∘F ≈ Id (correction after corruption ≈ identity)
            (Observation, Correction),
            (Word, Word),
            (ConfusionMatrix, Observation),
        ],

        being: AbstractObject,
        source: "Shannon (1948); Kernighan et al. (1990)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ChannelCategory>().unwrap();
    }

    #[test]
    fn six_concepts() {
        assert_eq!(ChannelConcept::variants().len(), 6);
    }

    #[test]
    fn channel_corrupts_word_to_observation() {
        let morphisms = ChannelCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ChannelConcept::Word
            && m.to == ChannelConcept::Observation
            && m.kind == ChannelRelationKind::Corrupts));
    }

    #[test]
    fn correction_is_inverse() {
        let morphisms = ChannelCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ChannelConcept::Observation
                    && m.to == ChannelConcept::Word
                    && m.kind == ChannelRelationKind::Corrects)
        );
    }

    #[test]
    fn adjunction_composes() {
        // F: Word → Observation, G: Observation → Word
        // G∘F: Word → Word (should exist as composition)
        let f = ChannelRelation {
            from: ChannelConcept::Word,
            to: ChannelConcept::Observation,
            kind: ChannelRelationKind::Corrupts,
        };
        let g = ChannelRelation {
            from: ChannelConcept::Observation,
            to: ChannelConcept::Word,
            kind: ChannelRelationKind::Corrects,
        };
        let composed = ChannelCategory::compose(&f, &g);
        assert!(composed.is_some());
        let c = composed.unwrap();
        assert_eq!(c.from, ChannelConcept::Word);
        assert_eq!(c.to, ChannelConcept::Word);
        // G∘F ≠ Id — it's Composed, not Identity (information loss!)
        assert_eq!(c.kind, ChannelRelationKind::Composed);
    }
}
