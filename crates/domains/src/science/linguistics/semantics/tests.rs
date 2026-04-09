use super::interpret;
use super::meaning::*;
use crate::science::linguistics::grammar::phrase::*;
use crate::science::linguistics::lexicon::pos::*;
use crate::science::linguistics::lexicon::{function_words, vocabulary};

fn word(text: &str) -> LexicalEntry {
    function_words::lookup(text)
        .or_else(|| vocabulary::lookup(text))
        .unwrap_or_else(|| panic!("word '{}' not in lexicon", text))
}

// =============================================================================
// Meaning representation tests
// =============================================================================

#[test]
fn predicate_arity() {
    assert_eq!(Predicate::unary("run").arity, 1);
    assert_eq!(Predicate::binary("see").arity, 2);
    assert_eq!(Predicate::ternary("give").arity, 3);
}

#[test]
fn proposition_well_formed() {
    let prop = SemanticProposition::new(
        Predicate::binary("see"),
        vec![
            EntityRef {
                name: "dog".into(),
                role: SemanticRole::Agent,
            },
            EntityRef {
                name: "cat".into(),
                role: SemanticRole::Patient,
            },
        ],
    );
    assert!(prop.is_well_formed());
    assert_eq!(prop.agent().unwrap().name, "dog");
    assert_eq!(prop.patient().unwrap().name, "cat");
}

#[test]
fn proposition_not_well_formed() {
    let prop = SemanticProposition::new(
        Predicate::binary("see"),
        vec![EntityRef {
            name: "dog".into(),
            role: SemanticRole::Agent,
        }],
    );
    assert!(!prop.is_well_formed()); // binary predicate with 1 argument
}

#[test]
fn proposition_negation() {
    let prop = SemanticProposition::new(Predicate::unary("run"), vec![]).negated();
    assert!(prop.negated);
    assert!(prop.describe().starts_with("NOT "));
}

#[test]
fn meaning_rep_composition() {
    let a = MeaningRep::Atomic(SemanticProposition::new(
        Predicate::unary("run"),
        vec![EntityRef {
            name: "dog".into(),
            role: SemanticRole::Agent,
        }],
    ));
    let b = MeaningRep::Atomic(SemanticProposition::new(
        Predicate::unary("bark"),
        vec![EntityRef {
            name: "dog".into(),
            role: SemanticRole::Agent,
        }],
    ));

    let combined = a.and(b);
    assert_eq!(combined.propositions().len(), 2);
    assert!(combined.is_well_formed());
}

#[test]
fn meaning_rep_describe() {
    let prop = MeaningRep::Atomic(SemanticProposition::new(
        Predicate::binary("see"),
        vec![
            EntityRef {
                name: "dog".into(),
                role: SemanticRole::Agent,
            },
            EntityRef {
                name: "cat".into(),
                role: SemanticRole::Patient,
            },
        ],
    ));
    assert_eq!(prop.describe(), "see(dog:Agent, cat:Patient)");
}

// =============================================================================
// Interpretation tests — syntax tree → meaning
// =============================================================================

fn build_sentence(words: &[&str]) -> SyntaxNode {
    // Simple parser: build "NP VP" where first noun-containing segment is NP
    let entries: Vec<LexicalEntry> = words.iter().map(|w| word(w)).collect();

    let mut np_children = Vec::new();
    let mut vp_children = Vec::new();
    let mut in_vp = false;

    for entry in entries {
        if !in_vp {
            match entry.pos_tag() {
                PosTag::Verb => {
                    in_vp = true;
                    vp_children.push(SyntaxNode::Leaf { entry });
                }
                _ => np_children.push(SyntaxNode::Leaf { entry }),
            }
        } else {
            match entry.pos_tag() {
                PosTag::Noun | PosTag::Determiner | PosTag::Adjective => {
                    // Object NP inside VP
                    vp_children.push(SyntaxNode::Leaf { entry });
                }
                _ => vp_children.push(SyntaxNode::Leaf { entry }),
            }
        }
    }

    // Wrap object nouns in VP into an NP sub-branch
    let mut final_vp_children = Vec::new();
    let mut object_np_children = Vec::new();
    for child in vp_children {
        match &child {
            SyntaxNode::Leaf { entry }
                if matches!(
                    entry.pos_tag(),
                    PosTag::Noun | PosTag::Determiner | PosTag::Adjective
                ) =>
            {
                object_np_children.push(child);
            }
            _ => {
                if !object_np_children.is_empty() {
                    final_vp_children.push(SyntaxNode::Branch {
                        phrase: PhraseType::NounPhrase,
                        children: std::mem::take(&mut object_np_children),
                    });
                }
                final_vp_children.push(child);
            }
        }
    }
    if !object_np_children.is_empty() {
        final_vp_children.push(SyntaxNode::Branch {
            phrase: PhraseType::NounPhrase,
            children: object_np_children,
        });
    }

    SyntaxNode::Branch {
        phrase: PhraseType::Sentence,
        children: vec![
            SyntaxNode::Branch {
                phrase: PhraseType::NounPhrase,
                children: np_children,
            },
            SyntaxNode::Branch {
                phrase: PhraseType::VerbPhrase,
                children: final_vp_children,
            },
        ],
    }
}

#[test]
fn interpret_intransitive() {
    // "the dog runs" → run(dog:Agent)
    let tree = build_sentence(&["the", "dog", "runs"]);
    let meaning = interpret::interpret(&tree).unwrap();
    let desc = meaning.describe();
    assert!(desc.contains("run"));
    assert!(desc.contains("dog:Agent"));
}

#[test]
fn interpret_transitive() {
    // "the dog sees the cat" → see(dog:Agent, cat:Patient)
    let tree = build_sentence(&["the", "dog", "sees", "the", "cat"]);
    let meaning = interpret::interpret(&tree).unwrap();
    let desc = meaning.describe();
    assert!(desc.contains("see"));
    assert!(desc.contains("dog:Agent"));
    assert!(desc.contains("cat:Patient"));
}

#[test]
fn interpret_well_formed() {
    let tree = build_sentence(&["the", "dog", "runs"]);
    let meaning = interpret::interpret(&tree).unwrap();
    assert!(meaning.is_well_formed());
}

#[test]
fn semantic_roles_correct() {
    let tree = build_sentence(&["the", "dog", "sees", "the", "cat"]);
    let meaning = interpret::interpret(&tree).unwrap();
    match &meaning {
        MeaningRep::Atomic(prop) => {
            assert_eq!(prop.agent().unwrap().name, "dog");
            assert_eq!(prop.patient().unwrap().name, "cat");
            assert_eq!(prop.predicate.lemma, "see");
        }
        _ => panic!("expected atomic proposition"),
    }
}
