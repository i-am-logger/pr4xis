use super::engine::*;
use super::phrase::*;
use crate::science::linguistics::lexicon::pos::*;
use crate::science::linguistics::lexicon::{function_words, vocabulary};

fn word(text: &str) -> LexicalEntry {
    function_words::lookup(text)
        .or_else(|| vocabulary::lookup(text))
        .unwrap_or_else(|| panic!("word '{}' not in lexicon", text))
}

// =============================================================================
// Parse tree structure tests
// =============================================================================

#[test]
fn syntax_node_text() {
    let tree = SyntaxNode::Branch {
        phrase: PhraseType::NounPhrase,
        children: vec![
            SyntaxNode::Leaf { entry: word("the") },
            SyntaxNode::Leaf { entry: word("dog") },
        ],
    };
    assert_eq!(tree.text(), "the dog");
}

#[test]
fn syntax_node_head_noun() {
    let tree = SyntaxNode::Branch {
        phrase: PhraseType::NounPhrase,
        children: vec![
            SyntaxNode::Leaf { entry: word("the") },
            SyntaxNode::Leaf { entry: word("big") },
            SyntaxNode::Leaf { entry: word("dog") },
        ],
    };
    let head = tree.head().unwrap();
    assert_eq!(head.text(), "dog");
    assert_eq!(head.pos_tag(), PosTag::Noun);
}

#[test]
fn syntax_node_depth() {
    let leaf = SyntaxNode::Leaf { entry: word("dog") };
    assert_eq!(leaf.depth(), 0);

    let np = SyntaxNode::Branch {
        phrase: PhraseType::NounPhrase,
        children: vec![SyntaxNode::Leaf { entry: word("dog") }],
    };
    assert_eq!(np.depth(), 1);
}

#[test]
fn syntax_node_word_count() {
    let tree = SyntaxNode::Branch {
        phrase: PhraseType::Sentence,
        children: vec![
            SyntaxNode::Branch {
                phrase: PhraseType::NounPhrase,
                children: vec![
                    SyntaxNode::Leaf { entry: word("the") },
                    SyntaxNode::Leaf { entry: word("dog") },
                ],
            },
            SyntaxNode::Branch {
                phrase: PhraseType::VerbPhrase,
                children: vec![SyntaxNode::Leaf {
                    entry: word("runs"),
                }],
            },
        ],
    };
    assert_eq!(tree.word_count(), 3);
}

// =============================================================================
// Engine tests — valid parses
// =============================================================================

#[test]
fn parse_simple_sentence() {
    // "the dog runs"
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("the") })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("dog") })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // close NP
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::VerbPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord {
            entry: word("runs"),
        })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // close VP
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // close S
    assert!(e.situation().completed.is_some());
    assert_eq!(
        e.situation().completed.as_ref().unwrap().text(),
        "the dog runs"
    );
}

#[test]
fn parse_with_adjective() {
    // "the big dog runs quickly"
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("the") })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("big") })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("dog") })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // close NP
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::VerbPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord {
            entry: word("runs"),
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord {
            entry: word("quickly"),
        })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // close VP
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // close S
    assert!(e.is_terminal());
    assert_eq!(
        e.situation().completed.as_ref().unwrap().text(),
        "the big dog runs quickly"
    );
}

#[test]
fn parse_plural_agreement() {
    // "the dogs run" — plural subject + plural verb
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("the") })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord {
            entry: word("dogs"),
        })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::VerbPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("run") })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    assert!(e.is_terminal());
}

// =============================================================================
// Engine tests — violations
// =============================================================================

#[test]
fn reject_verb_in_noun_phrase() {
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    // Can't put a verb in an NP
    let result = e.try_next(ParseAction::AddWord {
        entry: word("runs"),
    });
    assert!(result.is_err());
}

#[test]
fn reject_agreement_violation() {
    // "the dog run" — singular subject + plural verb
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("the") })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("dog") })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::VerbPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("run") })
        .unwrap(); // "run" is plural
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    // Closing the sentence should fail: "dog" (sg) + "run" (pl)
    let result = e.try_next(ParseAction::ClosePhrase);
    assert!(result.is_err());
}

#[test]
fn reject_close_empty_stack() {
    let e = new_parse();
    let result = e.try_next(ParseAction::ClosePhrase);
    assert!(result.is_err());
}

#[test]
fn reject_action_after_complete() {
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("dog") })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::VerbPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord {
            entry: word("runs"),
        })
        .unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap();
    let e = e.try_next(ParseAction::ClosePhrase).unwrap(); // complete
    assert!(e.is_terminal());

    // Can't add more after completion
    let result = e.try_next(ParseAction::OpenPhrase {
        phrase: PhraseType::NounPhrase,
    });
    assert!(result.is_err());
}

#[test]
fn parse_back_forward() {
    let e = new_parse();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::Sentence,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::OpenPhrase {
            phrase: PhraseType::NounPhrase,
        })
        .unwrap();
    let e = e
        .try_next(ParseAction::AddWord { entry: word("dog") })
        .unwrap();

    // Undo the word add
    let e = e.back().unwrap();
    assert!(e.situation().stack.last().unwrap().children.is_empty());

    // Redo
    let e = e.forward().unwrap();
    assert_eq!(e.situation().stack.last().unwrap().children.len(), 1);
}
