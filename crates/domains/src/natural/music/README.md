# Music -- Notes, intervals, scales, chords

Models Western tonal music as 12 pitch classes plus derived structures (intervals, scales, chord kinds) with an engine that enforces scale membership and MIDI range as preconditions on every state transition. The ontology layer exposes `PitchClass` as the entity with qualities for note name, naturalness, and consonance; the engine, scale, and chord machinery validate their invariants through Situation/Action/Precondition and property-based tests.

Key references:
- MIDI 1.0 specification (note numbering, middle C = 60, A4 = 69)
- Standard music theory (interval names, diatonic scales, chord qualities)

## Entities (12)

| Category | Entities |
|---|---|
| Pitch classes (12) | C, C#, D, D#, E, F, F#, G, G#, A, A#, B |

## Category

`PitchClass` is defined as an `Entity` with 12 variants; music does not yet wire a `define_ontology!` category. Interval composition (a monoid on `Interval` with `UNISON` as identity) and scale/chord structure live in the derived types and are exercised by property tests in `tests.rs`.

## Qualities

| Quality | Type | Description |
|---|---|---|
| NoteName | String | Letter name for the pitch class (C, C#, D, …, B) |
| IsNatural | bool | True for C, D, E, F, G, A, B (no sharp/flat) |
| IsConsonant | bool | True when the pitch class interpreted as an interval from the root is consonant |

## Axioms

Property-based tests in `tests.rs` carry the domain content: interval composition is associative with `UNISON` as identity, interval inversion sums to an octave, the tritone is self-inverse under inversion, octave transposition preserves pitch class, scales sum to 12 semitones and start ascending from the root, chords start on the root and are ascending, the chromatic scale admits every chord as diatonic, major and minor triads contain no tritone, and diminished triads always contain one. These are verified for every note in MIDI range and every scale / chord kind.

Engine preconditions:

| Precondition | Description | Source |
|---|---|---|
| ScaleEnforcement | Notes must be in the current scale (if set) | standard |
| RangeCheck | Notes must be within MIDI range 0–127 | MIDI 1.0 |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one. Music perception and psychoacoustics under `natural/hearing` would be natural targets for morphisms into or out of this ontology.

## Files

- `ontology.rs` -- `PitchClass` entity, `NoteName`/`IsNatural`/`IsConsonant` qualities
- `note.rs` -- `Note(u8)` MIDI-number type with pitch-class, octave, name, transpose, distance
- `interval.rs` -- `Interval` type, interval constants, consonance classification, composition
- `scale.rs` -- `Scale` and `ScaleKind` (Major, NaturalMinor, HarmonicMinor, MelodicMinor, Pentatonic, Blues, Chromatic, WholeTone, Dorian, Phrygian, Lydian, Mixolydian)
- `chord.rs` -- `Chord`, `ChordKind`, `ChordResult` (Diatonic / Chromatic) against a scale
- `engine.rs` -- `MusicState`/`MusicAction`/`ScaleEnforcement`/`RangeCheck` Situation-Action-Precondition wiring
- `tests.rs` -- unit and property-based tests over notes, intervals, scales, chords, engine
- `mod.rs` -- module declarations
