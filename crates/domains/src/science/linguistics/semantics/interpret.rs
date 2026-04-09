use super::super::grammar::phrase::{PhraseType, SyntaxNode};
use super::super::lexicon::pos::*;
use super::meaning::*;

/// Interpret a syntax tree into a semantic meaning representation.
/// Extracts predicates from verbs, agents from subject NPs, patients from object NPs.
pub fn interpret(tree: &SyntaxNode) -> Option<MeaningRep> {
    match tree {
        SyntaxNode::Branch {
            phrase: PhraseType::Sentence,
            children,
        } => interpret_sentence(children),
        _ => None,
    }
}

fn interpret_sentence(children: &[SyntaxNode]) -> Option<MeaningRep> {
    let mut agent: Option<EntityRef> = None;
    let mut predicate: Option<Predicate> = None;
    let mut patient: Option<EntityRef> = None;
    let mut seen_verb = false;

    for child in children {
        match child {
            SyntaxNode::Branch {
                phrase: PhraseType::NounPhrase,
                ..
            } => {
                let name = extract_head_text(child)?;
                if !seen_verb {
                    agent = Some(EntityRef {
                        name,
                        role: SemanticRole::Agent,
                    });
                } else {
                    patient = Some(EntityRef {
                        name,
                        role: SemanticRole::Patient,
                    });
                }
            }
            SyntaxNode::Branch {
                phrase: PhraseType::VerbPhrase,
                children: vp_children,
            } => {
                seen_verb = true;
                for vc in vp_children {
                    match vc {
                        SyntaxNode::Leaf {
                            entry: LexicalEntry::Verb(v),
                        } => {
                            let arity = match v.transitivity {
                                Transitivity::Intransitive => 1,
                                Transitivity::Transitive => 2,
                                Transitivity::Ditransitive => 3,
                            };
                            predicate = Some(Predicate {
                                lemma: v.lemma.clone(),
                                arity,
                            });
                        }
                        SyntaxNode::Branch {
                            phrase: PhraseType::NounPhrase,
                            ..
                        } => {
                            let name = extract_head_text(vc)?;
                            patient = Some(EntityRef {
                                name,
                                role: SemanticRole::Patient,
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let pred = predicate?;
    let mut args = Vec::new();
    if let Some(a) = agent {
        args.push(a);
    }
    if let Some(p) = patient {
        args.push(p);
    }

    Some(MeaningRep::Atomic(SemanticProposition::new(pred, args)))
}

fn extract_head_text(node: &SyntaxNode) -> Option<String> {
    node.head().map(|entry| entry.text().to_string())
}
