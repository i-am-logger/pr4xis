#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::*;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use proptest::prelude::*;

// =============================================================================
// Proptest strategies
// =============================================================================

/// A dimensionless UNITLESS quantity, for comparing against [`Game::round`]
/// and [`Game::sequence_length`]'s typed return values in these tests.
fn q(n: u32) -> Quantity {
    Quantity::from_unit(f64::from(n), &unit::UNITLESS)
}

fn arb_color() -> impl Strategy<Value = SimonColor> {
    prop_oneof![
        Just(SimonColor::Red),
        Just(SimonColor::Blue),
        Just(SimonColor::Green),
        Just(SimonColor::Yellow),
    ]
}

fn arb_seed() -> impl Strategy<Value = u64> {
    0..10000u64
}

// =============================================================================
// Basic game tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn test_new_game_starts_showing() {
    let game = Game::new(42);
    assert!(matches!(game.state(), GameState::Showing));
    assert_eq!(game.round(), q(1));
    assert_eq!(game.sequence_length(), q(1));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn test_sequence_grows_each_round() {
    let mut game = Game::new(42);
    assert_eq!(game.sequence_length(), q(1));

    game.start_input().unwrap();
    let color = game.sequence()[0];
    game.input(color);
    game.next_round().unwrap();
    assert_eq!(game.sequence_length(), q(2));
    assert_eq!(game.round(), q(2));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn test_correct_input_advances() {
    let mut game = Game::new(42);
    game.start_input().unwrap();
    let color = game.sequence()[0];
    let result = game.input(color);
    assert!(matches!(result, RoundResult::RoundComplete { round: 1 }));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn test_wrong_input_game_over() {
    let mut game = Game::new(42);
    game.start_input().unwrap();
    let correct = game.sequence()[0];
    let wrong = SimonColor::all()
        .into_iter()
        .find(|&c| c != correct)
        .unwrap();
    let result = game.input(wrong);
    assert!(matches!(result, RoundResult::Wrong { .. }));
    assert!(matches!(game.state(), GameState::GameOver { .. }));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn test_cannot_input_during_showing() {
    let mut game = Game::new(42);
    let result = game.input(SimonColor::Red);
    assert_eq!(result, RoundResult::InvalidState);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn test_cannot_advance_during_input() {
    let mut game = Game::new(42);
    game.start_input().unwrap();
    assert!(game.next_round().is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn test_cannot_advance_after_game_over() {
    let mut game = Game::new(42);
    game.start_input().unwrap();
    let correct = game.sequence()[0];
    let wrong = SimonColor::all()
        .into_iter()
        .find(|&c| c != correct)
        .unwrap();
    game.input(wrong);
    assert!(game.next_round().is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn test_replay_gives_correct_sequence() {
    let game = Game::new(42);
    let inputs = game.replay_correct();
    assert_eq!(inputs.len() as f64, game.sequence_length().value);
    for (i, input) in inputs.iter().enumerate() {
        assert_eq!(input.color, game.sequence()[i]);
        assert_eq!(input.position, i);
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn test_multi_round_game() {
    let mut game = Game::new(42);
    for round in 1..=5u32 {
        assert_eq!(game.round(), q(round));
        game.start_input().unwrap();
        let seq: Vec<SimonColor> = game.sequence().to_vec();
        for &color in &seq {
            let result = game.input(color);
            match result {
                RoundResult::Correct { .. } | RoundResult::RoundComplete { .. } => {}
                other => panic!("unexpected result at round {}: {:?}", round, other),
            }
        }
        assert!(matches!(game.state(), GameState::RoundComplete));
        if round < 5 {
            game.next_round().unwrap();
        }
    }
    assert_eq!(game.round(), q(5));
    assert_eq!(game.sequence_length(), q(5));
}

// =============================================================================
// Property-based tests — ontology enforcement
// =============================================================================

proptest! {
    /// New game always starts in Showing state
    #[test]
    fn prop_new_game_is_showing(seed in arb_seed()) {
        let game = Game::new(seed);
        prop_assert!(matches!(game.state(), GameState::Showing));
    }

    /// New game always has exactly 1 in the sequence
    #[test]
    fn prop_new_game_one_element(seed in arb_seed()) {
        let game = Game::new(seed);
        prop_assert_eq!(game.sequence_length(), q(1));
        prop_assert_eq!(game.round(), q(1));
    }

    /// Correct input on round 1 always completes the round
    #[test]
    fn prop_correct_first_input_completes(seed in arb_seed()) {
        let mut game = Game::new(seed);
        game.start_input().unwrap();
        let color = game.sequence()[0];
        let result = game.input(color);
        let is_complete = matches!(result, RoundResult::RoundComplete { round: 1 });
        prop_assert!(is_complete);
    }

    /// Wrong input always causes game over
    #[test]
    fn prop_wrong_input_game_over(seed in arb_seed()) {
        let mut game = Game::new(seed);
        game.start_input().unwrap();
        let correct = game.sequence()[0];
        // Find any wrong color
        let wrong = SimonColor::all().into_iter().find(|&c| c != correct).unwrap();
        let result = game.input(wrong);
        let is_wrong = matches!(result, RoundResult::Wrong { .. });
        prop_assert!(is_wrong);
        let is_over = matches!(game.state(), GameState::GameOver { .. });
        prop_assert!(is_over);
    }

    /// Game over state reports correct expected color
    #[test]
    fn prop_game_over_reports_expected(seed in arb_seed()) {
        let mut game = Game::new(seed);
        game.start_input().unwrap();
        let correct = game.sequence()[0];
        let wrong = SimonColor::all().into_iter().find(|&c| c != correct).unwrap();
        game.input(wrong);
        if let GameState::GameOver { expected, got, position, .. } = game.state() {
            prop_assert_eq!(*expected, correct);
            prop_assert_eq!(*got, wrong);
            prop_assert_eq!(*position, 0);
        } else {
            prop_assert!(false, "should be game over");
        }
    }

    /// Sequence grows by exactly 1 each round
    #[test]
    fn prop_sequence_grows_by_one(seed in arb_seed(), rounds in 1..10usize) {
        let mut game = Game::new(seed);
        for _ in 1..rounds {
            game.start_input().unwrap();
            let seq: Vec<SimonColor> = game.sequence().to_vec();
            for &color in &seq {
                game.input(color);
            }
            let len_before = game.sequence_length().value;
            game.next_round().unwrap();
            prop_assert_eq!(game.sequence_length().value, len_before + 1.0);
        }
    }

    /// Previous sequence is preserved when growing
    #[test]
    fn prop_sequence_prefix_preserved(seed in arb_seed()) {
        let mut game = Game::new(seed);
        let first = game.sequence().to_vec();

        game.start_input().unwrap();
        game.input(first[0]);
        game.next_round().unwrap();

        let second = game.sequence().to_vec();
        prop_assert_eq!(&second[..first.len()], &first[..]);
    }

    /// Deterministic: same seed produces same sequence
    #[test]
    fn prop_deterministic(seed in arb_seed()) {
        let game1 = Game::new(seed);
        let game2 = Game::new(seed);
        prop_assert_eq!(game1.sequence(), game2.sequence());
    }

    /// Cannot input during Showing state
    #[test]
    fn prop_no_input_during_showing(seed in arb_seed(), color in arb_color()) {
        let mut game = Game::new(seed);
        prop_assert_eq!(game.input(color), RoundResult::InvalidState);
    }

    /// Cannot start input after game over
    #[test]
    fn prop_no_restart_after_game_over(seed in arb_seed()) {
        let mut game = Game::new(seed);
        game.start_input().unwrap();
        let correct = game.sequence()[0];
        let wrong = SimonColor::all().into_iter().find(|&c| c != correct).unwrap();
        game.input(wrong);
        prop_assert!(game.start_input().is_err());
    }

    /// Cannot advance round from Showing state
    #[test]
    fn prop_no_advance_from_showing(seed in arb_seed()) {
        let game = Game::new(seed);
        let mut game = game;
        prop_assert!(game.next_round().is_err());
    }

    /// All colors in sequence are valid SimonColors
    #[test]
    fn prop_all_colors_valid(seed in arb_seed(), rounds in 1..10usize) {
        let mut game = Game::new(seed);
        for _ in 1..rounds {
            game.start_input().unwrap();
            let seq: Vec<SimonColor> = game.sequence().to_vec();
            for &color in &seq {
                prop_assert!(SimonColor::all().contains(&color));
                game.input(color);
            }
            game.next_round().unwrap();
        }
    }

    /// Replay always matches the actual sequence
    #[test]
    fn prop_replay_matches_sequence(seed in arb_seed()) {
        let game = Game::new(seed);
        let replay = game.replay_correct();
        let seq = game.sequence();
        prop_assert_eq!(replay.len(), seq.len());
        for (input, &color) in replay.iter().zip(seq.iter()) {
            prop_assert_eq!(input.color, color);
        }
    }
}

pr4xis::register_praxis_value!(prop_new_game_is_showing, Verifiable);
pr4xis::register_praxis_value!(prop_new_game_one_element, Verifiable);
pr4xis::register_praxis_value!(prop_correct_first_input_completes, Verifiable);
pr4xis::register_praxis_value!(prop_wrong_input_game_over, Honest);
pr4xis::register_praxis_value!(prop_game_over_reports_expected, Verifiable);
pr4xis::register_praxis_value!(prop_sequence_grows_by_one, Verifiable);
pr4xis::register_praxis_value!(prop_sequence_prefix_preserved, Verifiable);
pr4xis::register_praxis_value!(prop_deterministic, Deterministic);
pr4xis::register_praxis_value!(prop_no_input_during_showing, Honest);
pr4xis::register_praxis_value!(prop_no_restart_after_game_over, Honest);
pr4xis::register_praxis_value!(prop_no_advance_from_showing, Honest);
pr4xis::register_praxis_value!(prop_all_colors_valid, Verifiable);
pr4xis::register_praxis_value!(prop_replay_matches_sequence, Verifiable);

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn engine_play_round_one() {
    let e = new_simon(42);
    // Start input (transition from Showing → Inputting)
    let e = e.next(SimonAction::StartInput).unwrap();
    // Press the correct first color from the sequence
    let correct_color = Game::new(42).sequence()[0];
    let e = e.next(SimonAction::Press(correct_color)).unwrap();
    // Round complete → next round
    let e = e.next(SimonAction::NextRound).unwrap();
    assert_eq!(e.step(), 3);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn engine_invalid_state_rejected() {
    let e = new_simon(42);
    // Can't press before StartInput
    let result = e.next(SimonAction::Press(SimonColor::Red));
    assert!(result.is_err());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn engine_back_forward() {
    let e = new_simon(42);
    let e = e.next(SimonAction::StartInput).unwrap();
    let e = e.back().unwrap();
    assert_eq!(e.step(), 0);
    let e = e.forward().unwrap();
    assert_eq!(e.step(), 1);
}

#[pr4xis::praxis_value(Explainable)]
#[test]
fn engine_trace() {
    let e = new_simon(42);
    let e = e.next(SimonAction::StartInput).unwrap();
    assert_eq!(e.trace().entries().len(), 1);
    assert!(e.trace().entries()[0].applied());
}
