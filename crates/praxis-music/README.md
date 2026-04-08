# praxis-music

[![crates.io](https://img.shields.io/crates/v/praxis-music.svg)](https://crates.io/crates/praxis-music)
[![docs.rs](https://img.shields.io/docsrs/praxis-music)](https://docs.rs/praxis-music)

Music theory with rule enforcement -- notes, intervals, scales, and chords.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models the foundations of Western music theory as typed, enforceable structures. Notes are MIDI-backed (0-127), intervals carry consonance/dissonance properties, scales define pitch-class membership, and chords validate against scales to distinguish diatonic from chromatic usage. The engine layer enforces music-theoretic constraints on every transformation.

## Key Types

| Type | Description |
|---|---|
| `Note` | A MIDI note (0-127) with pitch class, octave, transposition, and enharmonic comparison |
| `Interval` | Distance in semitones with named constants (unison through octave), consonance checks, and inversion |
| `Scale` | Root note + scale kind (major, minor, modes, pentatonic, blues, etc.) with membership testing |
| `ScaleKind` | 12 scale types: Major, NaturalMinor, HarmonicMinor, Dorian, Phrygian, Lydian, Mixolydian, and more |
| `Chord` | Root + chord kind with note generation, tritone detection, and scale validation |
| `ChordKind` | 10 chord types: Major, Minor, Diminished, Augmented, Dominant7, Sus2, Sus4, and more |
| `ChordResult` | Validation result -- `Diatonic` or `Chromatic { out_of_scale }` |

## Example

```rust
use praxis_music::{Note, Scale, ScaleKind, Chord, ChordKind, ChordResult};

let c_major = Scale::new(Note::C4, ScaleKind::Major);
let chord = Chord::new(Note::C4, ChordKind::Major);

assert_eq!(chord.name(), "C");
assert_eq!(chord.validate_against(&c_major), ChordResult::Diatonic);
assert!(!chord.has_tritone());

let dim = Chord::new(Note::B4, ChordKind::Diminished);
assert!(dim.has_tritone());
```

## License

CC BY-NC-SA 4.0
