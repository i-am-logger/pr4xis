use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for ChannelConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Word,
            Self::Observation,
            Self::ErrorModel,
            Self::LanguageModel,
            Self::Correction,
            Self::ConfusionMatrix,
        ]
    }
}

/// Relationships in the noisy channel model.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelRelation {
    pub from: ChannelConcept,
    pub to: ChannelConcept,
    pub kind: ChannelRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelRelationKind {
    Identity,
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
    Composed,
}

impl Relationship for ChannelRelation {
    type Object = ChannelConcept;
    fn source(&self) -> ChannelConcept {
        self.from
    }
    fn target(&self) -> ChannelConcept {
        self.to
    }
}

pub struct ChannelCategory;

impl Category for ChannelCategory {
    type Object = ChannelConcept;
    type Morphism = ChannelRelation;

    fn identity(obj: &ChannelConcept) -> ChannelRelation {
        ChannelRelation {
            from: *obj,
            to: *obj,
            kind: ChannelRelationKind::Identity,
        }
    }

    fn compose(f: &ChannelRelation, g: &ChannelRelation) -> Option<ChannelRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ChannelRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ChannelRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ChannelRelation {
            from: f.from,
            to: g.to,
            kind: ChannelRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ChannelRelation> {
        use ChannelConcept::*;
        use ChannelRelationKind::*;

        let mut m = Vec::new();

        for c in ChannelConcept::variants() {
            m.push(ChannelRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // The channel: Word → Observation (corruption)
        m.push(ChannelRelation {
            from: Word,
            to: Observation,
            kind: Corrupts,
        });

        // The inverse: Observation → Word (correction / adjoint)
        m.push(ChannelRelation {
            from: Observation,
            to: Word,
            kind: Corrects,
        });

        // ErrorModel parameterizes the channel
        m.push(ChannelRelation {
            from: ErrorModel,
            to: Observation,
            kind: Parameterizes,
        });

        // LanguageModel weights the prior
        m.push(ChannelRelation {
            from: LanguageModel,
            to: Word,
            kind: Weights,
        });

        // ConfusionMatrix provides edit probabilities to ErrorModel
        m.push(ChannelRelation {
            from: ConfusionMatrix,
            to: ErrorModel,
            kind: Provides,
        });

        // Correction uses both models
        m.push(ChannelRelation {
            from: Correction,
            to: ErrorModel,
            kind: Uses,
        });
        m.push(ChannelRelation {
            from: Correction,
            to: LanguageModel,
            kind: Uses,
        });

        // The adjunction: G∘F ≈ Id (correction after corruption ≈ identity)
        // Observation → Correction → Word
        m.push(ChannelRelation {
            from: Observation,
            to: Correction,
            kind: Composed,
        });
        m.push(ChannelRelation {
            from: Correction,
            to: Word,
            kind: Corrects,
        });

        // Transitive
        m.push(ChannelRelation {
            from: Word,
            to: Word,
            kind: Composed,
        });
        m.push(ChannelRelation {
            from: ConfusionMatrix,
            to: Observation,
            kind: Composed,
        });

        // Self-composed closure
        for c in ChannelConcept::variants() {
            m.push(ChannelRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::Category;
    use praxis::category::validate::check_category_laws;

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
