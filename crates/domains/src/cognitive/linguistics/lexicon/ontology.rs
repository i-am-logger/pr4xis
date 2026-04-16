use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;
use pr4xis::ontology::Quality;
use pr4xis::ontology::upper::being::Being;
use pr4xis::ontology::upper::classify::Classified;

use super::pos::*;

/// Relationship between parts of speech: which POS can modify/complement which.
/// E.g., Adjective modifies Noun, Adverb modifies Verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifies {
    pub modifier: PosTag,
    pub head: PosTag,
}

impl Relationship for Modifies {
    type Object = PosTag;
    fn source(&self) -> PosTag {
        self.modifier
    }
    fn target(&self) -> PosTag {
        self.head
    }
}

/// The lexical category: parts of speech and their modification relationships.
pub struct LexicalCategory;

impl Category for LexicalCategory {
    type Object = PosTag;
    type Morphism = Modifies;

    fn identity(obj: &PosTag) -> Modifies {
        Modifies {
            modifier: *obj,
            head: *obj,
        }
    }

    fn compose(f: &Modifies, g: &Modifies) -> Option<Modifies> {
        if f.head != g.modifier {
            return None;
        }
        Some(Modifies {
            modifier: f.modifier,
            head: g.head,
        })
    }

    fn morphisms() -> Vec<Modifies> {
        let mut m = Vec::new();

        // Identity for each POS
        for pos in PosTag::variants() {
            m.push(Modifies {
                modifier: pos,
                head: pos,
            });
        }

        // Modification rules:
        // Adjective modifies Noun
        m.push(Modifies {
            modifier: PosTag::Adjective,
            head: PosTag::Noun,
        });
        // Adverb modifies Verb
        m.push(Modifies {
            modifier: PosTag::Adverb,
            head: PosTag::Verb,
        });
        // Adverb modifies Adjective ("very big")
        m.push(Modifies {
            modifier: PosTag::Adverb,
            head: PosTag::Adjective,
        });
        // Determiner modifies Noun
        m.push(Modifies {
            modifier: PosTag::Determiner,
            head: PosTag::Noun,
        });

        // Transitive closure: Adverb → Adjective → Noun
        m.push(Modifies {
            modifier: PosTag::Adverb,
            head: PosTag::Noun,
        });
        // Auxiliary modifies Verb (OLiA: AuxiliaryVerb governs MainVerb)
        m.push(Modifies {
            modifier: PosTag::Auxiliary,
            head: PosTag::Verb,
        });
        // Auxiliary modifies Copula ("has been")
        m.push(Modifies {
            modifier: PosTag::Auxiliary,
            head: PosTag::Copula,
        });
        // Particle modifies Verb ("not run", "to go")
        m.push(Modifies {
            modifier: PosTag::Particle,
            head: PosTag::Verb,
        });
        // Numeral modifies Noun ("three dogs")
        m.push(Modifies {
            modifier: PosTag::Numeral,
            head: PosTag::Noun,
        });
        // Article modifies Noun (Article is-a Determiner)
        m.push(Modifies {
            modifier: PosTag::Article,
            head: PosTag::Noun,
        });

        m
    }
}

impl Classified for LexicalCategory {
    fn being() -> Being {
        Being::SocialObject
    }
    fn classification_reason() -> &'static str {
        "language categories are evolved social conventions"
    }
}

/// Quality: is this POS a content word or a function word?
#[derive(Debug, Clone)]
pub struct IsContentWord;

impl Quality for IsContentWord {
    type Individual = PosTag;
    type Value = bool;

    fn get(&self, pos: &PosTag) -> Option<bool> {
        Some(pos.is_content())
    }
}
