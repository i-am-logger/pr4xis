# Domain Crates

Each domain crate implements `Situation`, `Action`, and `Precondition` from rust-praxis. This document describes what each crate enforces and how.

## praxis-chess

Full chess rules enforcement.

**Situation:** `Board` — 64 squares, castling rights, en passant target, halfmove clock, fullmove number.

**Actions:** `ChessAction { from, to }` — a move from one square to another.

**Preconditions:**
- `GameNotOver` — rejects moves after checkmate, stalemate, or 50-move rule
- `PieceExists` — source square must have a piece
- `OwnPiece` — piece must belong to the side to move
- `LegalMove` — move must follow piece movement rules including check

**Rules enforced:** Piece movement patterns, sliding piece blocking, pawn forward/capture distinction, double pawn move from starting rank, en passant, castling (kingside/queenside with path/check validation), promotion (auto-queen), check/checkmate/stalemate detection, pin detection, 50-move rule, castling rights loss on king/rook move.

## praxis-calculator

Scientific calculator with exact arithmetic and domain enforcement.

**Situation:** `Calculator` — display value, memory, angle mode, history.

**Actions:** `CalcAction` — Enter, Unary(op), Binary(op, value), Clear, AllClear, memory ops, angle mode.

**Preconditions:**
- `DomainCheck` — validates mathematical domain before operation

**Rules enforced:** Division by zero, sqrt of negative, log of non-positive, tan at undefined angles, asin/acos out of [-1,1], acosh < 1, atanh at ±1, factorial overflow (>20), MIDI range. Exact rational arithmetic with auto-simplification (2/4 → 1/2). Expression tree simplification preserving equivalence.

**Additional features:** Constants (π, e, φ), complex numbers, hyperbolic functions, bitwise operations, base conversion (bin/oct/dec/hex), unit conversion (35 units, 8 categories), combinatorics (nCr, nPr).

## praxis-rubik

Rubik's cube with group theory enforcement.

**Situation:** `Cube` — 6 faces × 9 stickers.

**Actions:** `RubikAction(Move)` — 18 moves (6 faces × CW/CCW/180°).

**Preconditions:**
- `ColorInvariant` — each color must have exactly 9 stickers

**Rules enforced:** 18 valid moves only, color conservation, center stickers fixed, move composition, inverse moves (CW inverse is CCW), 4× CW = identity, opposite faces commute.

## praxis-elevator

Multi-elevator dispatch with scheduling enforcement.

**Situation:** `Building` — N floors, M elevators with position/direction/load/doors/stops.

**Actions:** `ElevatorAction` — Request, Dispatch, Step, RunToCompletion.

**Preconditions:**
- `ValidRequest` — floor in range, destination in range, origin ≠ destination

**Rules enforced:** Capacity limits, direction commitment (going up won't accept stops below), doors must close before moving, no starvation, nearest-car dispatch, zone-based dispatch.

## praxis-traffic

Traffic signal intersection enforcement.

**Situation:** `Intersection` — N signals with state, timing, conflict matrix.

**Actions:** `TrafficAction` — AdvanceSignal, Tick, Malfunction, Recover.

**Preconditions:**
- `SafetyCheck` — conflicting directions cannot both be green

**Rules enforced:** Minimum phase timing (can't advance before minimum time), signal cycle (Green→Yellow→Red→Green), left arrow phase, malfunction/recovery (always recovers to Red), intersection conflict prevention, 4-way conflict matrix.

## praxis-tetris

Tetris with spatial constraint enforcement.

**Situation:** `Game` — 10×20 board, current piece, score, level.

**Actions:** `TetrisAction(GameAction)` — MoveLeft, MoveRight, MoveDown, HardDrop, RotateCW, RotateCCW.

**Preconditions:**
- `GameActive` — game must not be over

**Rules enforced:** Collision detection, rotation with wall kicks, line clears, gravity, game over on blocked spawn, score never decreases, walls block movement.

## praxis-simon

Simon Says sequence memory enforcement.

**Situation:** `Game` — sequence, round, game state.

**Actions:** `SimonAction` — StartInput, Press(color), NextRound.

**Preconditions:**
- `ValidState` — action must be valid for current game state (Showing/Inputting/RoundComplete)

**Rules enforced:** Correct sequence reproduction, wrong input = game over, sequence grows by 1 each round, previous sequence preserved, deterministic (same seed = same game), state machine (Showing→Inputting→RoundComplete or GameOver).

## praxis-http

HTTP connection state machine enforcement.

**Situation:** `Connection` — state, retries, max retries, keep-alive.

**Actions:** `HttpAction(ConnectionAction)` — Connect, SendRequest, ReceiveResponse, Complete, Retry, Close, Reset.

**Preconditions:**
- `ValidTransition` — action must be valid for current connection state

**Rules enforced:** State machine (Idle→Connecting→Sending→Awaiting→Receiving→Complete→Closed), can't skip states, max retries, keep-alive reuse after Complete, Closed is terminal. HTTP method semantics: safe methods are idempotent, body methods are not safe, status code ranges.

## praxis-music

Music theory enforcement.

**Situation:** `MusicState` — current note + optional scale context.

**Actions:** `MusicAction` — Transpose, SetScale, ClearScale, MoveTo.

**Preconditions:**
- `RangeCheck` — notes must be within MIDI range 0-127
- `ScaleEnforcement` — if a scale is set, notes must be scale tones

**Rules enforced:** 12-semitone octave system, interval composition, scale definitions (12 kinds including modes), chord construction (10 kinds), consonance/dissonance classification, diatonic validation, tritone detection.

## praxis-colors

Color theory enforcement.

**Situation:** `Rgb` — red, green, blue channels.

**Actions:** `ColorAction` — Mix, Blend, Invert, Grayscale, SetChannel.

**Preconditions:**
- `ValidAlpha` — blend alpha must be 0.0-1.0
- `ContrastCheck` — warns if result has very low contrast

**Rules enforced:** 4 mixing modes (additive, average, multiply, screen), WCAG contrast ratios (AA: 4.5:1, AAA: 7:1), luminance calculation, color complementation, additive mixing laws (black is identity, white absorbs in screen), commutative mixing.

## praxis-legal

Legal case lifecycle enforcement.

**Situation:** `Case` — caption, phase, motions, rulings, events.

**Actions:** `LegalAction` — File, BeginDiscovery, FileMotion, RuleOnMotion, SetForTrial, BeginTrial, Verdict, Appeal, Settle, Dismiss.

**Preconditions:**
- `PhaseTransition` — validates action against current case phase with fine-grained checks

**Rules enforced:** Case phases (PreFiling→Filed→Discovery→Motions→PreTrial→Trial→PostTrial→Appeal→Closed), motion lifecycle (Pending→Opposed→UnderAdvisement→Granted/Denied), settlement/dismissal from any active phase, closed is terminal. Rich enums carry full context (filing date, judge, ruling text). Legal ontology: authorities, elements, burden of proof, deadlines, remedies.

## praxis-math

Famous mathematical theorems as ontology-driven axioms.

**Pythagorean theorem:**
- Situation: `Triangle` (a, b, c)
- Actions: Scale, SetLegA, SetLegB (hypotenuse always derived)
- Preconditions: PythagoreanTheorem (a²+b²=c² enforced), PositiveSides
- Also: Pythagorean triple generation, triangle inequality

**Quadratic formula:**
- Situation: `Quadratic` (a, b, c with auto-computed roots)
- Actions: SetA, SetB, SetC (roots recomputed on every change)
- Preconditions: NonZeroA, RootsValid (roots satisfy equation)
- Also: Vieta's formulas, discriminant consistency

**Fibonacci:** Recurrence relation, Cassini's identity (F(n-1)F(n+1)-F(n)²=(-1)^n), golden ratio convergence.

**Primes:** Sieve of Eratosthenes, prime factorization (product = original), Goldbach conjecture (verified to 1000).

**Set theory:** Union/intersection/difference with algebraic laws — De Morgan's, distributive, inclusion-exclusion, symmetric difference.

## praxis-physics

Physics equations as ontology-driven axioms. Every law is a Precondition enforced by the Engine.

**Mechanics:**
- Situation: `Particle` (mass, position, velocity)
- Actions: ApplyForce, FreeFall
- Preconditions: MassConservation (mass never changes), PositiveDuration (time moves forward)
- Proven: F=ma (Δv = F/m⋅Δt for all random inputs), KE always non-negative

**Energy:**
- Situation: `System` (mass, velocity, height)
- Actions: Drop (PE→KE), Rise (KE→PE)
- Preconditions: EnergyConservation (KE+PE=constant), PhysicalConstraints (can't drop below ground, can't rise without enough KE)
- Proven: total energy preserved across all transformations

**Electromagnetism:**
- Situation: `Circuit` (voltage, current, resistance)
- Actions: SetVoltage, SetResistance (current auto-derived)
- Preconditions: OhmsLaw (V=IR enforced), PositiveResistance
- Also: Coulomb's law (symmetric)

**Relativity:**
- Situation: `Body` (rest mass, velocity)
- Actions: Accelerate, SetVelocity
- Preconditions: SpeedLimit (v < c enforced — nothing reaches speed of light)
- Derived: Lorentz factor, E=mc², time dilation, length contraction, relativistic momentum

**Quantum:**
- Situation: `QuantumParticle` (Δx, Δp — position and momentum uncertainties)
- Actions: MeasurePosition, MeasureMomentum
- Preconditions: HeisenbergUncertainty (ΔxΔp ≥ ℏ/2 enforced), PositiveUncertainty
- Proven: measuring position more precisely automatically increases momentum uncertainty
- Also: photon energy (E=hf), de Broglie wavelength, hydrogen energy levels
