use super::*;
use proptest::prelude::*;

fn arb_note() -> impl Strategy<Value = Note> {
    (0..128u8).prop_map(Note)
}

fn arb_midi_note() -> impl Strategy<Value = Note> {
    (21..109u8).prop_map(Note) // piano range
}

fn arb_scale_kind() -> impl Strategy<Value = ScaleKind> {
    (0..12usize).prop_map(|i| ScaleKind::all()[i])
}

fn arb_chord_kind() -> impl Strategy<Value = ChordKind> {
    (0..10usize).prop_map(|i| ChordKind::all()[i])
}

// =============================================================================
// Note tests
// =============================================================================

#[test]
fn test_middle_c() {
    assert_eq!(Note::C4.pitch_class(), 0);
    assert_eq!(Note::C4.octave(), 4);
    assert_eq!(Note::C4.name(), "C");
}

#[test]
fn test_a440() {
    assert_eq!(Note::A4.0, 69);
    assert_eq!(Note::A4.pitch_class(), 9);
}

#[test]
fn test_transpose() {
    assert_eq!(Note::C4.transpose(12), Some(Note(72))); // C5
    assert_eq!(Note::C4.transpose(-12), Some(Note(48))); // C3
    assert_eq!(Note(0).transpose(-1), None);
    assert_eq!(Note(127).transpose(1), None);
}

#[test]
fn test_fifth_plus_fourth_is_octave() {
    let result = Interval::PERFECT_FIFTH.compose(Interval::PERFECT_FOURTH);
    assert_eq!(result, Interval::OCTAVE);
}

#[test]
fn test_tritone_self_inverse() {
    assert_eq!(Interval::TRITONE.invert(), Interval::TRITONE);
}

#[test]
fn test_consonance() {
    assert!(Interval::PERFECT_FIFTH.is_consonant());
    assert!(Interval::MINOR_SECOND.is_dissonant());
}

#[test]
fn test_c_major_scale() {
    let scale = Scale::new(Note::C4, ScaleKind::Major);
    let names: Vec<&str> = scale.notes().iter().map(|n| n.name()).collect();
    assert_eq!(names, vec!["C", "D", "E", "F", "G", "A", "B", "C"]);
}

#[test]
fn test_c_major_chord() {
    let chord = Chord::new(Note::C4, ChordKind::Major);
    let names: Vec<&str> = chord.notes().iter().map(|n| n.name()).collect();
    assert_eq!(names, vec!["C", "E", "G"]);
}

#[test]
fn test_diatonic_validation() {
    let chord = Chord::new(Note::C4, ChordKind::Major);
    let scale = Scale::new(Note::C4, ScaleKind::Major);
    assert_eq!(chord.validate_against(&scale), ChordResult::Diatonic);
}

#[test]
fn test_chromatic_detection() {
    let chord = Chord::new(Note(61), ChordKind::Major); // C# major in C major scale
    let scale = Scale::new(Note::C4, ScaleKind::Major);
    assert!(matches!(
        chord.validate_against(&scale),
        ChordResult::Chromatic { .. }
    ));
}

#[test]
fn test_diminished_has_tritone() {
    assert!(Chord::new(Note::C4, ChordKind::Diminished).has_tritone());
}

#[test]
fn test_major_no_tritone() {
    assert!(!Chord::new(Note::C4, ChordKind::Major).has_tritone());
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    #[test]
    fn prop_pitch_class_range(note in arb_note()) {
        prop_assert!(note.pitch_class() < 12);
    }

    #[test]
    fn prop_transpose_zero(note in arb_note()) {
        prop_assert_eq!(note.transpose(0), Some(note));
    }

    #[test]
    fn prop_transpose_octave_preserves_class(note in arb_midi_note()) {
        if let Some(up) = note.transpose(12) {
            prop_assert_eq!(up.pitch_class(), note.pitch_class());
        }
    }

    #[test]
    fn prop_transpose_inverse(note in arb_midi_note(), semitones in 1..12i16) {
        if let Some(up) = note.transpose(semitones) {
            if let Some(back) = up.transpose(-semitones) {
                prop_assert_eq!(back, note);
            }
        }
    }

    #[test]
    fn prop_enharmonic_reflexive(note in arb_note()) {
        prop_assert!(note.is_enharmonic(note));
    }

    #[test]
    fn prop_octave_enharmonic(note in arb_midi_note()) {
        if let Some(up) = note.transpose(12) {
            prop_assert!(note.is_enharmonic(up));
        }
    }

    #[test]
    fn prop_inversion_sum_octave(semitones in 0..13u8) {
        let interval = Interval(semitones);
        prop_assert_eq!(interval.0 + interval.invert().0, 12);
    }

    #[test]
    fn prop_interval_associative(a in 0..6u8, b in 0..6u8, c in 0..6u8) {
        let left = Interval(a).compose(Interval(b)).compose(Interval(c));
        let right = Interval(a).compose(Interval(b).compose(Interval(c)));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn prop_unison_identity(semitones in 0..24u8) {
        let i = Interval(semitones);
        prop_assert_eq!(i.compose(Interval::UNISON), i);
        prop_assert_eq!(Interval::UNISON.compose(i), i);
    }

    #[test]
    fn prop_scale_sums_to_12(kind in arb_scale_kind()) {
        prop_assert_eq!(Scale::new(Note::C4, kind).total_semitones(), 12);
    }

    #[test]
    fn prop_scale_starts_with_root(root in arb_midi_note(), kind in arb_scale_kind()) {
        prop_assert_eq!(Scale::new(root, kind).notes()[0], root);
    }

    #[test]
    fn prop_scale_contains_root(root in arb_midi_note(), kind in arb_scale_kind()) {
        prop_assert!(Scale::new(root, kind).contains(root));
    }

    #[test]
    fn prop_scale_ascending(root in arb_midi_note(), kind in arb_scale_kind()) {
        let notes = Scale::new(root, kind).notes();
        for i in 1..notes.len() {
            prop_assert!(notes[i].0 > notes[i-1].0);
        }
    }

    #[test]
    fn prop_major_7_pitch_classes(root in arb_midi_note()) {
        let pcs: std::collections::HashSet<u8> = Scale::new(root, ScaleKind::Major)
            .notes().iter().take(7).map(|n| n.pitch_class()).collect();
        prop_assert_eq!(pcs.len(), 7);
    }

    #[test]
    fn prop_chord_note_count(root in arb_midi_note(), kind in arb_chord_kind()) {
        let chord = Chord::new(root, kind);
        prop_assert_eq!(chord.notes().len(), chord.note_count());
    }

    #[test]
    fn prop_chord_starts_with_root(root in arb_midi_note(), kind in arb_chord_kind()) {
        prop_assert_eq!(Chord::new(root, kind).notes()[0], root);
    }

    #[test]
    fn prop_chord_ascending(root in arb_midi_note(), kind in arb_chord_kind()) {
        let notes = Chord::new(root, kind).notes();
        for i in 1..notes.len() {
            prop_assert!(notes[i].0 > notes[i-1].0);
        }
    }

    #[test]
    fn prop_chromatic_always_diatonic(root in arb_midi_note(), kind in arb_chord_kind()) {
        let chord = Chord::new(root, kind);
        let scale = Scale::new(root, ScaleKind::Chromatic);
        prop_assert_eq!(chord.validate_against(&scale), ChordResult::Diatonic);
    }

    #[test]
    fn prop_tonic_is_diatonic(root in arb_midi_note()) {
        let chord = Chord::new(root, ChordKind::Major);
        let scale = Scale::new(root, ScaleKind::Major);
        prop_assert_eq!(chord.validate_against(&scale), ChordResult::Diatonic);
    }

    #[test]
    fn prop_diminished_has_tritone(root in arb_midi_note()) {
        prop_assert!(Chord::new(root, ChordKind::Diminished).has_tritone());
    }

    #[test]
    fn prop_major_no_tritone(root in arb_midi_note()) {
        prop_assert!(!Chord::new(root, ChordKind::Major).has_tritone());
    }

    #[test]
    fn prop_minor_no_tritone(root in arb_midi_note()) {
        prop_assert!(!Chord::new(root, ChordKind::Minor).has_tritone());
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

use praxis::engine::Precondition;
use praxis::engine::*;

#[test]
fn engine_transpose_in_scale() {
    let e = new_music(Note(60)); // Middle C
    let e = e
        .try_next(MusicAction::SetScale {
            kind: ScaleKind::Major,
        })
        .unwrap();
    // Transpose up 2 semitones (C → D, which is in C Major)
    let e = e.try_next(MusicAction::Transpose { semitones: 2 }).unwrap();
    assert_eq!(e.situation().note, Note(62));
}

#[test]
fn engine_transpose_outside_scale_rejected() {
    let e = new_music(Note(60)); // Middle C
    let e = e
        .try_next(MusicAction::SetScale {
            kind: ScaleKind::Major,
        })
        .unwrap();
    // Transpose up 1 semitone (C → C#, NOT in C Major)
    let result = e.try_next(MusicAction::Transpose { semitones: 1 });
    assert!(result.is_err());
}

#[test]
fn engine_out_of_range_rejected() {
    let e = new_music(Note(125));
    // Transpose beyond MIDI range
    let result = e.try_next(MusicAction::Transpose { semitones: 10 });
    assert!(result.is_err());
}

#[test]
fn engine_back_forward() {
    let e = new_music(Note(60));
    let e = e.try_next(MusicAction::Transpose { semitones: 7 }).unwrap();
    let e = e.try_next(MusicAction::Transpose { semitones: 5 }).unwrap();
    let e = e.back().unwrap();
    assert_eq!(e.situation().note, Note(67));
    let e = e.forward().unwrap();
    assert_eq!(e.situation().note, Note(72));
}

#[test]
fn engine_clear_scale_allows_any() {
    let e = new_music(Note(60));
    let e = e
        .try_next(MusicAction::SetScale {
            kind: ScaleKind::Major,
        })
        .unwrap();
    let e = e.try_next(MusicAction::ClearScale).unwrap();
    // Now C# should be allowed (no scale)
    let e = e.try_next(MusicAction::Transpose { semitones: 1 }).unwrap();
    assert_eq!(e.situation().note, Note(61));
}

// =============================================================================
// Note::distance_to tests
// =============================================================================

#[test]
fn test_distance_to_same_note() {
    assert_eq!(Note::C4.distance_to(Note::C4), 0);
}

#[test]
fn test_distance_to_ascending() {
    // C4 to G4 = 7 semitones
    assert_eq!(Note::C4.distance_to(Note::G4), 7);
    // C4 to E4 = 4 semitones (major third)
    assert_eq!(Note::C4.distance_to(Note::E4), 4);
}

#[test]
fn test_distance_to_descending() {
    // G4 to C4 = -7 semitones
    assert_eq!(Note::G4.distance_to(Note::C4), -7);
}

#[test]
fn test_distance_to_octave() {
    assert_eq!(Note::C4.distance_to(Note(72)), 12); // C4 to C5
    assert_eq!(Note(72).distance_to(Note::C4), -12); // C5 to C4
}

#[test]
fn test_distance_to_extremes() {
    assert_eq!(Note(0).distance_to(Note(127)), 127);
    assert_eq!(Note(127).distance_to(Note(0)), -127);
}

// =============================================================================
// Interval::name tests
// =============================================================================

#[test]
fn test_interval_name_all_twelve() {
    assert_eq!(Interval::UNISON.name(), "unison/octave");
    assert_eq!(Interval::MINOR_SECOND.name(), "minor 2nd");
    assert_eq!(Interval::MAJOR_SECOND.name(), "major 2nd");
    assert_eq!(Interval::MINOR_THIRD.name(), "minor 3rd");
    assert_eq!(Interval::MAJOR_THIRD.name(), "major 3rd");
    assert_eq!(Interval::PERFECT_FOURTH.name(), "perfect 4th");
    assert_eq!(Interval::TRITONE.name(), "tritone");
    assert_eq!(Interval::PERFECT_FIFTH.name(), "perfect 5th");
    assert_eq!(Interval::MINOR_SIXTH.name(), "minor 6th");
    assert_eq!(Interval::MAJOR_SIXTH.name(), "major 6th");
    assert_eq!(Interval::MINOR_SEVENTH.name(), "minor 7th");
    assert_eq!(Interval::MAJOR_SEVENTH.name(), "major 7th");
}

#[test]
fn test_interval_name_octave() {
    // Octave (12) should map to 12 % 12 = 0, same as unison
    assert_eq!(Interval::OCTAVE.name(), "unison/octave");
}

#[test]
fn test_interval_name_wraps_beyond_octave() {
    // 13 semitones = minor 2nd an octave up
    assert_eq!(Interval(13).name(), "minor 2nd");
    // 24 semitones = two octaves, maps to unison/octave
    assert_eq!(Interval(24).name(), "unison/octave");
}

// =============================================================================
// Chord::note_count tests
// =============================================================================

#[test]
fn test_chord_note_count_triads() {
    assert_eq!(Chord::new(Note::C4, ChordKind::Major).note_count(), 3);
    assert_eq!(Chord::new(Note::C4, ChordKind::Minor).note_count(), 3);
    assert_eq!(Chord::new(Note::C4, ChordKind::Diminished).note_count(), 3);
    assert_eq!(Chord::new(Note::C4, ChordKind::Augmented).note_count(), 3);
    assert_eq!(Chord::new(Note::C4, ChordKind::Sus2).note_count(), 3);
    assert_eq!(Chord::new(Note::C4, ChordKind::Sus4).note_count(), 3);
}

#[test]
fn test_chord_note_count_sevenths() {
    assert_eq!(Chord::new(Note::C4, ChordKind::Major7).note_count(), 4);
    assert_eq!(Chord::new(Note::C4, ChordKind::Minor7).note_count(), 4);
    assert_eq!(Chord::new(Note::C4, ChordKind::Dominant7).note_count(), 4);
    assert_eq!(Chord::new(Note::C4, ChordKind::Diminished7).note_count(), 4);
}

// =============================================================================
// Chord::name tests
// =============================================================================

#[test]
fn test_chord_name_all_kinds() {
    let root = Note::C4;
    assert_eq!(Chord::new(root, ChordKind::Major).name(), "C");
    assert_eq!(Chord::new(root, ChordKind::Minor).name(), "Cm");
    assert_eq!(Chord::new(root, ChordKind::Diminished).name(), "Cdim");
    assert_eq!(Chord::new(root, ChordKind::Augmented).name(), "Caug");
    assert_eq!(Chord::new(root, ChordKind::Major7).name(), "Cmaj7");
    assert_eq!(Chord::new(root, ChordKind::Minor7).name(), "Cm7");
    assert_eq!(Chord::new(root, ChordKind::Dominant7).name(), "C7");
    assert_eq!(Chord::new(root, ChordKind::Diminished7).name(), "Cdim7");
    assert_eq!(Chord::new(root, ChordKind::Sus2).name(), "Csus2");
    assert_eq!(Chord::new(root, ChordKind::Sus4).name(), "Csus4");
}

#[test]
fn test_chord_name_with_sharp_root() {
    let fsharp = Note(66); // F#4
    assert_eq!(Chord::new(fsharp, ChordKind::Minor).name(), "F#m");
    assert_eq!(Chord::new(fsharp, ChordKind::Major7).name(), "F#maj7");
}

#[test]
fn test_chord_name_various_roots() {
    assert_eq!(Chord::new(Note::A4, ChordKind::Minor).name(), "Am");
    assert_eq!(Chord::new(Note::G4, ChordKind::Dominant7).name(), "G7");
    assert_eq!(Chord::new(Note::D4, ChordKind::Sus4).name(), "Dsus4");
    assert_eq!(Chord::new(Note::E4, ChordKind::Augmented).name(), "Eaug");
    assert_eq!(Chord::new(Note::B4, ChordKind::Diminished).name(), "Bdim");
}

// =============================================================================
// Scale::degree_count tests
// =============================================================================

#[test]
fn test_scale_degree_count_seven_note_scales() {
    let root = Note::C4;
    assert_eq!(Scale::new(root, ScaleKind::Major).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::NaturalMinor).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::HarmonicMinor).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::MelodicMinor).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::Dorian).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::Phrygian).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::Lydian).degree_count(), 7);
    assert_eq!(Scale::new(root, ScaleKind::Mixolydian).degree_count(), 7);
}

#[test]
fn test_scale_degree_count_pentatonic() {
    assert_eq!(
        Scale::new(Note::C4, ScaleKind::Pentatonic).degree_count(),
        5
    );
}

#[test]
fn test_scale_degree_count_blues() {
    assert_eq!(Scale::new(Note::C4, ScaleKind::Blues).degree_count(), 6);
}

#[test]
fn test_scale_degree_count_chromatic() {
    assert_eq!(
        Scale::new(Note::C4, ScaleKind::Chromatic).degree_count(),
        12
    );
}

#[test]
fn test_scale_degree_count_whole_tone() {
    assert_eq!(Scale::new(Note::C4, ScaleKind::WholeTone).degree_count(), 6);
}

// =============================================================================
// ScaleEnforcement::describe and RangeCheck::describe tests
// =============================================================================

#[test]
fn test_scale_enforcement_describe() {
    let precondition = ScaleEnforcement;
    assert_eq!(
        precondition.describe(),
        "notes must be in the current scale (if set)"
    );
}

#[test]
fn test_range_check_describe() {
    let precondition = RangeCheck;
    assert_eq!(
        precondition.describe(),
        "notes must be within MIDI range 0-127"
    );
}

// =============================================================================
// Additional engine tests for MoveTo action paths
// =============================================================================

#[test]
fn engine_move_to_in_scale() {
    let e = new_music(Note(60));
    let e = e
        .try_next(MusicAction::SetScale {
            kind: ScaleKind::Major,
        })
        .unwrap();
    // Move to E4 (64), which is in C Major
    let e = e.try_next(MusicAction::MoveTo { note: Note::E4 }).unwrap();
    assert_eq!(e.situation().note, Note::E4);
}

#[test]
fn engine_move_to_outside_scale_rejected() {
    let e = new_music(Note(60));
    let e = e
        .try_next(MusicAction::SetScale {
            kind: ScaleKind::Major,
        })
        .unwrap();
    // Move to C#4 (61), which is NOT in C Major
    let result = e.try_next(MusicAction::MoveTo { note: Note(61) });
    assert!(result.is_err());
}
