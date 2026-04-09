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

    let success = current.len() == 1 && current[0].lambek_type == LambekType::s();
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
