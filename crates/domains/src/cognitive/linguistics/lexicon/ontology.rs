use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::Quality;
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::pos::*;

/// Arrow between parts of speech: which POS can modify/complement which.
/// E.g., Adjective modifies Noun, Adverb modifies Verb.
///
/// Per OBO-RO (Smith 2005), every arrow carries a relation-kind tag.
/// Here we use a single `Modification` kind — the lexical category's
/// only relation is syntactic modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexicalRelationKind {
    Modification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifies {
    pub modifier: PosTag,
    pub head: PosTag,
}

impl Arrow for Modifies {
    type Object = PosTag;
    type Kind = LexicalRelationKind;

    fn source(&self) -> PosTag {
        self.modifier
    }
    fn target(&self) -> PosTag {
        self.head
    }
    fn kind(&self) -> LexicalRelationKind {
        LexicalRelationKind::Modification
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("Modifies"),
            description: Label::new_static(
                "syntactic modification — modifier POS modifies head POS (Lambek 1958)",
            ),
            citation: Citation::parse_static("Lambek (1958); Chiarcos & Sukhareva OLiA (2015)"),
            module_path: ModulePath::new_static(module_path!()),
        }
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
