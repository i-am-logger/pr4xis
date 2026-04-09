use super::reduce::TypedToken;
use super::types::LambekType;
use crate::science::linguistics::english::{ConceptId, English};

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
    Entity {
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
            Sem::Entity { word, .. } => word.clone(),
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
fn lex(word: &str, ty: &LambekType, en: &English) -> Sem {
    let concepts: Vec<ConceptId> = en.lookup(word).to_vec();

    match ty {
        // NP → Entity domain
        LambekType::Atom(super::types::AtomicType::NP) => Sem::Entity {
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
pub fn interpret(tokens: &[TypedToken], en: &English) -> Sem {
    if tokens.is_empty() {
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
                    apply(&values[i].1, &values[i + 1].1, &result_type)
                } else {
                    // Backward: f(x) — right is function, left is argument
                    apply(&values[i + 1].1, &values[i].1, &result_type)
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
fn apply(func: &Sem, arg: &Sem, result_type: &LambekType) -> Sem {
    match result_type {
        // Result is S (any feature) — check if question or proposition
        LambekType::Atom(super::types::AtomicType::S(feature)) => {
            let predicate = extract_predicate(func);
            let mut arguments = extract_arguments(func);
            arguments.push(arg.clone());
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
            Sem::Pred { word } => Sem::Entity {
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
        Sem::Entity { word, .. } => word.clone(),
        Sem::Prop { predicate, .. } | Sem::Question { predicate, .. } => predicate.clone(),
    }
}

fn extract_word(sem: &Sem) -> String {
    match sem {
        Sem::Pred { word } => word.clone(),
        Sem::Func { word, .. } => word.clone(),
        Sem::Entity { word, .. } => word.clone(),
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
