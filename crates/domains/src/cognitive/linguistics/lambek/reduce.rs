use super::types::{LambekType, reduce};

/// A typed token — a word with its Lambek type assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedToken {
    pub word: String,
    pub lambek_type: LambekType,
}

/// A reduction step — records how two types combined.
#[derive(Debug, Clone)]
pub struct ReductionStep {
    pub left: String,
    pub right: String,
    pub left_type: LambekType,
    pub right_type: LambekType,
    pub result_type: LambekType,
    pub direction: ReductionDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum ReductionDirection {
    /// Forward application: A/B + B → A
    Forward,
    /// Backward application: A + A\B → B
    Backward,
}

/// Result of attempting to reduce a sequence of typed tokens.
#[derive(Debug, Clone)]
pub struct ReductionResult {
    pub success: bool,
    pub final_type: Option<LambekType>,
    pub steps: Vec<ReductionStep>,
    pub remaining: Vec<TypedToken>,
}

/// Reduce a sequence of typed tokens by applying forward and backward
/// application rules until no more reductions are possible.
///
/// This IS the syntax category in action — each reduction is a morphism
/// (type composition), and the final type is the result of composing
/// all the word types together.
///
/// Returns a sentence type (S) if the input is a grammatical sentence.
pub fn reduce_sequence(tokens: &[TypedToken]) -> ReductionResult {
    let mut current: Vec<TypedToken> = tokens.to_vec();
    let mut steps = Vec::new();

    // Keep reducing until no more reductions possible
    loop {
        let mut reduced = false;

        // Try to reduce adjacent pairs (left to right, then right to left)
        for i in 0..current.len().saturating_sub(1) {
            if let Some(result) = reduce(&current[i].lambek_type, &current[i + 1].lambek_type) {
                steps.push(ReductionStep {
                    left: current[i].word.clone(),
                    right: current[i + 1].word.clone(),
                    left_type: current[i].lambek_type.clone(),
                    right_type: current[i + 1].lambek_type.clone(),
                    result_type: result.clone(),
                    direction: if matches!(current[i].lambek_type, LambekType::RightDiv(_, _)) {
                        ReductionDirection::Forward
                    } else {
                        ReductionDirection::Backward
                    },
                });

                let combined_word = format!("[{} {}]", current[i].word, current[i + 1].word);
                let new_token = TypedToken {
                    word: combined_word,
                    lambek_type: result,
                };

                current.splice(i..=i + 1, [new_token]);
                reduced = true;
                break;
            }
        }

        if !reduced {
            break;
        }
    }

    let success = current.len() == 1
        && matches!(
            current[0].lambek_type,
            LambekType::Atom(super::types::AtomicType::S(_))
        );
    let final_type = if current.len() == 1 {
        Some(current[0].lambek_type.clone())
    } else {
        None
    };

    ReductionResult {
        success,
        final_type,
        steps,
        remaining: current,
    }
}

/// CYK chart parser for Lambek grammars with lexical ambiguity.
///
/// Standard algorithm from the literature:
/// - Goodman, "Semiring Parsing" (1999)
/// - Hepple, "Chart Parsing Lambek Grammars" (1992)
/// - Moroz, "A Savateev-Style Parsing Algorithm for Pregroup Grammars" (2009)
///
/// Each word has a SET of possible types (from the lexicon).
/// The chart tries ALL combinations simultaneously via dynamic programming.
/// A sentence is grammatical iff S ∈ chart[0, n].
///
/// Complexity: O(n³ × K²) where K = max types per word.
/// For natural language (K ≤ 10, n ≤ 20): trivially real-time.
///
/// `type_sets` provides all possible types for each token position.
/// type_sets[i] = all Lambek types that word_i could have.
pub fn chart_reduce(words: &[String], type_sets: &[Vec<LambekType>]) -> ReductionResult {
    use std::collections::HashMap;
    use std::collections::HashSet;
    let n = words.len();
    if n == 0 {
        return ReductionResult {
            success: false,
            final_type: None,
            steps: Vec::new(),
            remaining: Vec::new(),
        };
    }

    // chart[i][j] = set of types derivable for span words[i..j]
    let mut chart: Vec<Vec<HashSet<LambekType>>> = vec![vec![HashSet::new(); n + 1]; n + 1];

    // Backpointer: for each (i, j, result_type) → (split_k, left_type, right_type)
    let mut back: Vec<Vec<BackPointer>> = vec![vec![HashMap::new(); n + 1]; n + 1];

    // Step 1: Initialize — all types for each word
    for i in 0..n {
        for t in &type_sets[i] {
            chart[i][i + 1].insert(t.clone());
        }
    }

    // Step 2: Fill chart bottom-up (CYK)
    for span in 2..=n {
        for i in 0..=(n - span) {
            let j = i + span;
            for k in (i + 1)..j {
                let left_types: Vec<LambekType> = chart[i][k].iter().cloned().collect();
                let right_types: Vec<LambekType> = chart[k][j].iter().cloned().collect();

                for t_left in &left_types {
                    for t_right in &right_types {
                        if let Some(t_result) = reduce(t_left, t_right)
                            && chart[i][j].insert(t_result.clone())
                        {
                            back[i][j].insert(t_result, (k, t_left.clone(), t_right.clone()));
                        }
                    }
                }
            }
        }
    }

    // Step 3: Check if S ∈ chart[0, n]
    // Prefer featured S types (S[q], S[wq], S[dcl]) over bare S(None),
    // because featured types carry more information.
    let sentence_type = chart[0][n]
        .iter()
        .filter(|t| matches!(t, LambekType::Atom(super::types::AtomicType::S(_))))
        .max_by_key(|t| match t {
            LambekType::Atom(super::types::AtomicType::S(Some(_))) => 1,
            _ => 0,
        });

    let success = sentence_type.is_some();
    let final_type = sentence_type.cloned();

    // Step 4: Backtrack to find winning lexical types
    let remaining = if let Some(st) = &final_type {
        let mut winning_types = vec![None; n];
        extract_lexical_types(0, n, st, &back, &mut winning_types);

        words
            .iter()
            .enumerate()
            .map(|(i, w)| TypedToken {
                word: w.clone(),
                lambek_type: winning_types[i]
                    .clone()
                    .unwrap_or_else(|| type_sets[i][0].clone()),
            })
            .collect()
    } else {
        words
            .iter()
            .zip(type_sets.iter())
            .map(|(w, types)| TypedToken {
                word: w.clone(),
                lambek_type: types
                    .first()
                    .cloned()
                    .unwrap_or(LambekType::Atom(super::types::AtomicType::N)),
            })
            .collect()
    };

    ReductionResult {
        success,
        final_type,
        steps: Vec::new(),
        remaining,
    }
}

/// Backpointer: split point + left/right types that produced a result.
type BackPointer = std::collections::HashMap<LambekType, (usize, LambekType, LambekType)>;

/// Backtrack through the chart to extract which lexical type was used at each position.
fn extract_lexical_types(
    i: usize,
    j: usize,
    target: &LambekType,
    back: &[Vec<BackPointer>],
    result: &mut [Option<LambekType>],
) {
    if j == i + 1 {
        // Leaf: this is a lexical item at position i
        result[i] = Some(target.clone());
        return;
    }
    if let Some((k, left_type, right_type)) = back[i][j].get(target) {
        extract_lexical_types(i, *k, left_type, back, result);
        extract_lexical_types(*k, j, right_type, back, result);
    }
}

/// Reduce with ambiguity using the CYK chart parser.
///
/// Combines the primary type and all alternatives into type sets,
/// then runs the chart parser over all combinations simultaneously.
pub fn reduce_with_alternatives(
    tokens: &[TypedToken],
    alternatives: &[Vec<LambekType>],
) -> ReductionResult {
    let words: Vec<String> = tokens.iter().map(|t| t.word.clone()).collect();

    // Build type sets: primary type + all alternatives
    let type_sets: Vec<Vec<LambekType>> = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut types = vec![t.lambek_type.clone()];
            if let Some(alts) = alternatives.get(i) {
                for alt in alts {
                    if !types.contains(alt) {
                        types.push(alt.clone());
                    }
                }
            }
            types
        })
        .collect();

    chart_reduce(&words, &type_sets)
}
