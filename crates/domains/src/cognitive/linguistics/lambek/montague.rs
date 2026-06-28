#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::reduce::TypedToken;
use super::types::LambekType;
use crate::cognitive::linguistics::english::{ConceptId, LexicalReasoner};

// The Montague functor: Syntax → Semantics.
//
// Type-driven interpretation: each Lambek type maps to a semantic domain.
// The mapping IS a functor — composition in syntax maps to composition in semantics.
//
// Atomic types → Semantic domains:
//   NP → Entity (a reference to a thing)
//   S  → Proposition (a truth-evaluable statement)
//   N  → Predicate (a property: λx.dog(x))
//
// Complex types → Function spaces:
//   A/B → (B-domain → A-domain)
//   A\B → (A-domain → B-domain)
//
// Reduction → Function application:
//   (A/B) + B → A  ≡  f(x) where f: B→A, x: B
//
// References:
// - Montague, The Proper Treatment of Quantification (1973)
// - Coecke, Sadrzadeh, Clark, DisCoCat (2010)

/// A semantic value — lives in the semantic domain determined by its Lambek type.
#[derive(Debug, Clone, PartialEq)]
pub enum Sem {
    /// Entity domain (NP): a reference to something in the world.
    Concept {
        word: String,
        concepts: Vec<ConceptId>,
    },
    /// Predicate domain (N): a property that can be true of entities.
    Pred { word: String },
    /// Proposition domain (S): a complete truth-evaluable statement.
    Prop {
        predicate: String,
        arguments: Vec<Sem>,
    },
    /// Question domain (Q): a proposition that asks for truth value or information.
    Question {
        predicate: String,
        arguments: Vec<Sem>,
    },
    /// Function domain (A/B or A\B): a function waiting for an argument.
    Func { word: String, body: Box<Sem> },
}

impl Sem {
    pub fn describe(&self) -> String {
        match self {
            Sem::Concept { word, .. } => word.clone(),
            Sem::Pred { word } => format!("λx.{}(x)", word),
            Sem::Prop {
                predicate,
                arguments,
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
                format!("{}({})", predicate, args.join(", "))
            }
            Sem::Question {
                predicate,
                arguments,
            } => {
                let args: Vec<String> = arguments.iter().map(|a| a.describe()).collect();
                format!("?{}({})", predicate, args.join(", "))
            }
            Sem::Func { word, .. } => format!("λ.{}", word),
        }
    }

    /// Is this a question?
    pub fn is_question(&self) -> bool {
        matches!(self, Sem::Question { .. })
    }

    /// Is this a proposition?
    pub fn is_proposition(&self) -> bool {
        matches!(self, Sem::Prop { .. })
    }
}

/// Assign a lexical semantic value to a word based on its Lambek type.
/// This is the LEXICAL part of the functor — mapping words to their semantic domains.
fn lex(word: &str, ty: &LambekType, en: &dyn LexicalReasoner) -> Sem {
    let concepts: Vec<ConceptId> = en.lookup(word).to_vec();

    match ty {
        // NP → Entity domain
        LambekType::Atom(super::types::AtomicType::NP) => Sem::Concept {
            word: word.into(),
            concepts,
        },
        // N → Predicate domain
        LambekType::Atom(super::types::AtomicType::N) => Sem::Pred { word: word.into() },
        // A/B or A\B → Function domain
        // The function takes a B-domain value and produces an A-domain value
        LambekType::RightDiv(_, _) | LambekType::LeftDiv(_, _) => Sem::Func {
            word: word.into(),
            body: Box::new(Sem::Pred { word: word.into() }),
        },
        // S or other atoms — predicate as default
        _ => Sem::Pred { word: word.into() },
    }
}

/// Apply the functor: reduce semantic values in parallel with type reductions.
/// Each type reduction (function application) corresponds to semantic function application.
pub fn interpret(tokens: &[TypedToken], en: &dyn LexicalReasoner) -> Sem {
    // The reduction loop below is O(n²) in the token count, so a pathologically
    // long utterance is a resource-exhaustion DoS. Real sentences are far under
    // this bound (matching chart_reduce's MAX_CHART_WIDTH); abstain past it.
    const MAX_INTERPRET_WIDTH: usize = 256;
    if tokens.is_empty() || tokens.len() > MAX_INTERPRET_WIDTH {
        return Sem::Pred {
            word: "empty".into(),
        };
    }

    let mut values: Vec<(LambekType, Sem)> = tokens
        .iter()
        .map(|t| (t.lambek_type.clone(), lex(&t.word, &t.lambek_type, en)))
        .collect();

    // Reduce: each type reduction triggers the corresponding semantic composition
    loop {
        let mut reduced = false;
        for i in 0..values.len().saturating_sub(1) {
            if let Some(result_type) = super::types::reduce(&values[i].0, &values[i + 1].0) {
                let is_forward = matches!(values[i].0, LambekType::RightDiv(_, _));
                let sem = if is_forward {
                    // Forward: f(x) — left is function, right is argument
                    apply(&values[i].1, &values[i + 1].1, &result_type, en)
                } else {
                    // Backward: f(x) — right is function, left is argument
                    apply(&values[i + 1].1, &values[i].1, &result_type, en)
                };
                values.splice(i..=i + 1, [(result_type, sem)]);
                reduced = true;
                break;
            }
        }
        if !reduced {
            break;
        }
    }

    values
        .into_iter()
        .next()
        .map(|(_, s)| s)
        .unwrap_or(Sem::Pred { word: "?".into() })
}

/// Semantic function application — the ONLY composition rule.
/// When types reduce via A/B + B → A, the semantics is f(x).
/// The result domain is determined by the result type.
fn apply(func: &Sem, arg: &Sem, result_type: &LambekType, en: &dyn LexicalReasoner) -> Sem {
    match result_type {
        // Result is S (any feature) — check if question or proposition
        LambekType::Atom(super::types::AtomicType::S(feature)) => {
            // When the argument is a RELATIONAL predicative complement — a `Func`
            // whose head surface is a LOADED relation ("part of") — the asserted
            // relation comes from the COMPLEMENT, not the copula: lift its surface
            // to the predicate and flatten its object into the arguments. So
            // "is X part of Y" → Question{ "part of", [X, Y] }, whereas the plain
            // copula "is X a Y" keeps the function's predicate ("is" → the
            // Subsumption default). The discriminator is loaded data
            // (`relation_for_surface`), not a hardcoded "part of" match.
            let (predicate, arguments) = match arg {
                Sem::Func { word, body } if en.relation_for_surface(word).is_some() => {
                    let mut arguments = extract_arguments(func);
                    arguments.push((**body).clone());
                    (word.clone(), arguments)
                }
                _ => {
                    let mut arguments = extract_arguments(func);
                    arguments.push(arg.clone());
                    (extract_predicate(func), arguments)
                }
            };
            match feature {
                Some(super::types::SentenceFeature::Q | super::types::SentenceFeature::Wq) => {
                    Sem::Question {
                        predicate,
                        arguments,
                    }
                }
                _ => Sem::Prop {
                    predicate,
                    arguments,
                },
            }
        }
        // Result is NP (entity)
        LambekType::Atom(super::types::AtomicType::NP) => match arg {
            Sem::Pred { word } => Sem::Concept {
                word: word.clone(),
                concepts: Vec::new(),
            },
            _ => arg.clone(),
        },
        // Result is N (predicate) — modifier applied to predicate
        LambekType::Atom(super::types::AtomicType::N) => {
            let func_word = extract_word(func);
            let arg_word = extract_word(arg);
            Sem::Pred {
                word: format!("{} {}", func_word, arg_word),
            }
        }
        // Result is a function type — partial application
        LambekType::RightDiv(_, _) | LambekType::LeftDiv(_, _) => {
            let predicate = extract_predicate(func);
            Sem::Func {
                word: predicate,
                body: Box::new(arg.clone()),
            }
        }
        _ => func.clone(),
    }
}

fn extract_predicate(sem: &Sem) -> String {
    match sem {
        Sem::Pred { word } => word.clone(),
        Sem::Func { word, .. } => word.clone(),
        Sem::Concept { word, .. } => word.clone(),
        Sem::Prop { predicate, .. } | Sem::Question { predicate, .. } => predicate.clone(),
    }
}

fn extract_word(sem: &Sem) -> String {
    match sem {
        Sem::Pred { word } => word.clone(),
        Sem::Func { word, .. } => word.clone(),
        Sem::Concept { word, .. } => word.clone(),
        Sem::Prop { predicate, .. } | Sem::Question { predicate, .. } => predicate.clone(),
    }
}

fn extract_arguments(sem: &Sem) -> Vec<Sem> {
    match sem {
        Sem::Func { body, .. } => vec![*body.clone()],
        Sem::Prop { arguments, .. } | Sem::Question { arguments, .. } => arguments.clone(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::composed::ComposedReasoner;
    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::lambek::types::svo;

    /// The relational-question SEMANTICS in isolation (no parse plumbing): given
    /// the typed tokens for "is X part of Y", `interpret` lifts the relation from
    /// the COMPLEMENT ("part of") into the `Question` predicate and flattens its
    /// object into the arguments — `Question{ "part of", [X, Y] }`. The lift fires
    /// only because `relation_for_surface("part of")` is loaded (ComposedReasoner
    /// carries the relation lexicon); on plain English it would not.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_x_part_of_y_interprets_to_a_relational_question() {
        let en = ComposedReasoner::new(English::sample(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                word: "is".into(),
                lambek_type: svo::question_copula_pred(),
            },
            TypedToken {
                word: "alpha".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                word: "part of".into(),
                lambek_type: svo::relational_predicate(),
            },
            TypedToken {
                word: "beta".into(),
                lambek_type: svo::proper_noun(),
            },
        ];

        match interpret(&tokens, &en) {
            Sem::Question {
                predicate,
                arguments,
            } => {
                assert_eq!(
                    predicate, "part of",
                    "the relation comes from the complement, not the copula 'is'"
                );
                let names: Vec<String> = arguments.iter().map(extract_entity_name).collect();
                assert_eq!(
                    names,
                    alloc::vec!["alpha".to_string(), "beta".to_string()],
                    "subject and object are flattened into the arguments, in order"
                );
            }
            other => panic!("expected a relational Question, got {other:?}"),
        }
    }

    /// The plain copula "is X a Y" is UNCHANGED: the argument is a bare entity NP
    /// (not a relational `Func`), so the predicate stays the copula "is" (→ the
    /// Subsumption default at dispatch). Proves the lift does not fire spuriously.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn is_x_a_y_keeps_the_copula_predicate() {
        let en = ComposedReasoner::new(English::sample(), Vec::new());
        let tokens = alloc::vec![
            TypedToken {
                word: "is".into(),
                lambek_type: svo::question_copula(),
            },
            TypedToken {
                word: "alpha".into(),
                lambek_type: svo::proper_noun(),
            },
            TypedToken {
                word: "beta".into(),
                lambek_type: svo::proper_noun(),
            },
        ];
        match interpret(&tokens, &en) {
            Sem::Question { predicate, .. } => assert_eq!(
                predicate, "is",
                "a plain copula question keeps the copula predicate"
            ),
            other => panic!("expected a Question, got {other:?}"),
        }
    }

    fn extract_entity_name(sem: &Sem) -> String {
        match sem {
            Sem::Concept { word, .. } | Sem::Pred { word } => word.clone(),
            _ => String::new(),
        }
    }
}
