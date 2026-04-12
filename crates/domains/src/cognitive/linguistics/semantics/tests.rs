use super::interpret;
use super::meaning::*;
use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::grammar::phrase::*;
use crate::cognitive::linguistics::language::Language;
use crate::cognitive::linguistics::lexicon::pos::*;

fn sample_lang() -> English {
    let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource><Lexicon id="t" label="T" language="en" email="" license="" version="1.0" url="">
<LexicalEntry id="e1"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="s1" synset="ss1"/></LexicalEntry>
<LexicalEntry id="e2"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="s2" synset="ss2"/></LexicalEntry>
<LexicalEntry id="e3"><Lemma writtenForm="sees" partOfSpeech="v"/><Sense id="s3" synset="ss3"/></LexicalEntry>
<LexicalEntry id="e4"><Lemma writtenForm="big" partOfSpeech="a"/><Sense id="s4" synset="ss4"/></LexicalEntry>
<LexicalEntry id="e5"><Lemma writtenForm="runs" partOfSpeech="v"/><Sense id="s5" synset="ss5"/></LexicalEntry>
<LexicalEntry id="e6"><Lemma writtenForm="quickly" partOfSpeech="r"/><Sense id="s6" synset="ss6"/></LexicalEntry>
<Synset id="ss1" partOfSpeech="n" members="e1"><Definition>dog</Definition></Synset>
<Synset id="ss2" partOfSpeech="n" members="e2"><Definition>cat</Definition></Synset>
<Synset id="ss3" partOfSpeech="v" members="e3"><Definition>see</Definition></Synset>
<Synset id="ss4" partOfSpeech="a" members="e4"><Definition>big</Definition></Synset>
<Synset id="ss5" partOfSpeech="v" members="e5"><Definition>run</Definition></Synset>
<Synset id="ss6" partOfSpeech="r" members="e6"><Definition>quickly</Definition></Synset>
</Lexicon></LexicalResource>"#,
    )
    .unwrap();
    English::from_wordnet(&wn)
}

fn word(text: &str) -> LexicalEntry {
    let lang = sample_lang();
    lang.lexical_lookup(text)
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
            // WordNet-backed entries use the surface form as lemma.
            // Proper lemmatization requires morphological analysis.
            assert!(prop.predicate.lemma == "see" || prop.predicate.lemma == "sees");
        }
        _ => panic!("expected atomic proposition"),
    }
}
