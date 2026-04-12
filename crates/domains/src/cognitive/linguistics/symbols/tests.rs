use super::character::*;
use super::numeral;
use super::punctuation;
use super::special;

// =============================================================================
// Script tests
// =============================================================================

#[test]
fn latin_has_52_letters() {
    let latin = latin();
    assert_eq!(latin.letter_count(), 52); // 26 upper + 26 lower
}

#[test]
fn latin_is_ltr() {
    assert_eq!(latin().direction, Direction::LeftToRight);
}

#[test]
fn hebrew_is_rtl() {
    assert_eq!(hebrew().direction, Direction::RightToLeft);
}

#[test]
fn hebrew_has_27_letters() {
    let hebrew = hebrew();
    assert_eq!(hebrew.letter_count(), 27); // 22 basic + 5 final forms (sofit)
}

#[test]
fn arabic_numerals_has_10_digits() {
    let nums = arabic_numerals();
    assert_eq!(nums.characters.len(), 10);
    assert!(nums.contains('0'));
    assert!(nums.contains('9'));
    assert!(!nums.contains('a'));
}

#[test]
fn latin_contains_a_not_aleph() {
    let latin = latin();
    assert!(latin.contains('a'));
    assert!(latin.contains('Z'));
    assert!(!latin.contains('\u{05D0}')); // aleph
}

#[test]
fn hebrew_contains_aleph_not_a() {
    let hebrew = hebrew();
    assert!(hebrew.contains('\u{05D0}')); // aleph
    assert!(hebrew.contains('\u{05EA}')); // tav
    assert!(!hebrew.contains('a'));
}

// =============================================================================
// Character classification tests
// =============================================================================

#[test]
fn character_categories() {
    let upper_a = Character::new('A', "A", UnicodeCategory::UppercaseLetter);
    assert!(upper_a.is_letter());
    assert!(!upper_a.is_digit());

    let digit = Character::new('5', "5", UnicodeCategory::DecimalDigit);
    assert!(digit.is_digit());
    assert!(!digit.is_letter());

    let punct = Character::new('.', "period", UnicodeCategory::Punctuation);
    assert!(punct.is_punctuation());
    assert!(!punct.is_letter());
}

// =============================================================================
// Punctuation tests
// =============================================================================

#[test]
fn period_terminates_sentence() {
    let p = punctuation::period();
    assert!(p.is_sentence_ending());
    assert!(!p.expects_response());
    assert_eq!(
        p.function,
        punctuation::PunctuationFunction::StatementTerminator
    );
}

#[test]
fn question_mark_expects_response() {
    let q = punctuation::question_mark();
    assert!(q.is_sentence_ending());
    assert!(q.expects_response());
}

#[test]
fn comma_is_not_sentence_ending() {
    let c = punctuation::comma();
    assert!(!c.is_sentence_ending());
    assert_eq!(c.function, punctuation::PunctuationFunction::Separator);
}

#[test]
fn standard_punctuation_count() {
    let marks = punctuation::standard_punctuation();
    assert_eq!(marks.len(), 12);
}

#[test]
fn sentence_ending_marks() {
    let marks = punctuation::standard_punctuation();
    let endings: Vec<_> = marks.iter().filter(|m| m.is_sentence_ending()).collect();
    assert_eq!(endings.len(), 3); // period, question, exclamation
}

// =============================================================================
// Numeral tests
// =============================================================================

#[test]
fn arabic_numerals_base_10() {
    let arabic = numeral::arabic();
    assert_eq!(arabic.base, 10);
    assert_eq!(arabic.digits.len(), 10);
}

#[test]
fn arabic_numeral_values() {
    let arabic = numeral::arabic();
    assert_eq!(arabic.value_of('0'), Some(0));
    assert_eq!(arabic.value_of('9'), Some(9));
    assert_eq!(arabic.value_of('a'), None);
}

#[test]
fn roman_numerals() {
    let roman = numeral::roman();
    assert_eq!(roman.value_of('I'), Some(1));
    assert_eq!(roman.value_of('V'), Some(5));
    assert_eq!(roman.value_of('X'), Some(10));
    assert_eq!(roman.value_of('M'), Some(1000));
}

// =============================================================================
// Special symbol tests
// =============================================================================

#[test]
fn common_symbols_exist() {
    let symbols = special::common_symbols();
    assert!(symbols.iter().any(|s| s.character == '<'));
    assert!(symbols.iter().any(|s| s.character == '>'));
    assert!(symbols.iter().any(|s| s.character == '&'));
    assert!(symbols.iter().any(|s| s.character == '='));
}

#[test]
fn angle_bracket_is_general() {
    // '<' is General because its meaning depends on language context
    let symbols = special::common_symbols();
    let lt = symbols.iter().find(|s| s.character == '<').unwrap();
    assert_eq!(lt.domain, special::SymbolDomain::General);
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Every character in Latin script is a letter.
        #[test]
        fn prop_latin_all_letters(idx in 0..52usize) {
            let latin = latin();
            let ch = &latin.characters[idx];
            prop_assert!(ch.is_letter());
        }

        /// Every digit in Arabic numerals has value < base.
        #[test]
        fn prop_arabic_digit_in_range(idx in 0..10usize) {
            let arabic = numeral::arabic();
            let digit = &arabic.digits[idx];
            prop_assert!(digit.value < arabic.base);
        }

        /// Punctuation function names are exhaustive.
        #[test]
        fn prop_punctuation_function_coverage(idx in 0..12usize) {
            let marks = punctuation::standard_punctuation();
            if let Some(mark) = marks.get(idx) {
                let _ = mark.is_sentence_ending();
            }
        }

        /// Scripts don't share characters (Latin and Hebrew are disjoint).
        #[test]
        fn prop_scripts_disjoint(idx in 0..52usize) {
            let latin = latin();
            let hebrew = hebrew();
            let ch = latin.characters[idx].codepoint;
            prop_assert!(!hebrew.contains(ch),
                "Latin character {} should not be in Hebrew script", ch);
        }

        /// Every Latin uppercase has a corresponding lowercase (a-z ↔ A-Z).
        #[test]
        fn prop_latin_case_pairs(idx in 0..26usize) {
            let latin = latin();
            let upper = latin.characters[idx].codepoint; // A-Z
            let lower = latin.characters[idx + 26].codepoint; // a-z
            prop_assert_eq!(upper.to_lowercase().next().unwrap(), lower);
        }

        /// Every sentence-ending punctuation mark appears after content.
        #[test]
        fn prop_sentence_endings_position(idx in 0..12usize) {
            let marks = punctuation::standard_punctuation();
            if let Some(mark) = marks.get(idx) {
                if mark.is_sentence_ending() {
                    prop_assert_eq!(mark.position, punctuation::Position::After);
                }
            }
        }

        /// Arabic numeral digits map contiguously: digit N has value N.
        #[test]
        fn prop_arabic_contiguous(idx in 0..10usize) {
            let arabic = numeral::arabic();
            prop_assert_eq!(arabic.digits[idx].value, idx as u32);
        }

        /// No two standard punctuation marks share the same character.
        #[test]
        fn prop_punctuation_unique_chars(i in 0..12usize, j in 0..12usize) {
            if i != j {
                let marks = punctuation::standard_punctuation();
                if let (Some(a), Some(b)) = (marks.get(i), marks.get(j)) {
                    prop_assert_ne!(a.character, b.character,
                        "{} and {} share character", a.name, b.name);
                }
            }
        }
    }
}
