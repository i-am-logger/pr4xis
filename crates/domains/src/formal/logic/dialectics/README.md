# Dialectics — reasoning through opposition

Categorical and ontological encoding of dialectical reasoning from Aristotle's *Peri Hermeneias* (~350 BCE) through Hegel, Marx, Adorno, and Priest's dialetheism. Serves as the literature-grounded target for `Syntrometry::Dialektik` (Heim's binary-opposition-on-a-Predikatrix primitive) and as a reusable formal substrate for any pr4xis ontology that reasons via opposition + synthesis.

## Verification

```
cargo test -p pr4xis-domains -- dialectics
```

Runs category laws, ontology validation, and six first-class domain axioms covering the Hegelian triad, Aristotelian Square of Opposition, Hegel's Sublation mechanism, the canonical Thesis↔Antithesis opposition, Adorno's refusal of Synthesis, and Priest's dialetheism-demands-paraconsistency link.

## Entities (18)

| Tradition | Entities |
|---|---|
| Aristotle — Square of Opposition (5) | `SquareOfOpposition`, `Contrary`, `Contradictory`, `Subaltern`, `Subcontrary` |
| Aristotle — Dialectical argument (2) | `DialecticalArgument`, `Endoxa` |
| Hegelian triad (4) | `DialecticalMoment`, `Thesis`, `Antithesis`, `Synthesis` |
| Hegelian mechanisms (3) | `DeterminateNegation`, `Sublation`, `Contradiction` |
| Marx (1) | `InternalContradiction` |
| Adorno (2) | `NegativeDialectics`, `NonIdentity` |
| Priest (2) | `TrueContradiction`, `Paraconsistent` |

## Axioms

All six are first-class `Axiom.holds()` tests:

| Axiom | Source | Claim |
|---|---|---|
| `HegelianTriad` | Hegel 1807 | Direct children of `DialecticalMoment` are exactly `{Thesis, Antithesis, Synthesis}` |
| `AristotelianSquareHasFourVertices` | Aristotle / Apuleius | Direct children of `SquareOfOpposition` are exactly `{Contrary, Contradictory, Subaltern, Subcontrary}` |
| `SynthesisHasSublation` | Hegel | `Sublation` produces `Synthesis` via the `Produces` edge |
| `ThesisAntithesisOppose` | Hegel | Thesis and Antithesis stand in the generic `opposes` relation |
| `AdornoRefusesSynthesis` | Adorno 1966 | `NegativeDialectics` opposes `Synthesis` (non-reconciliation encoded) |
| `DialetheismNeedsParaconsistency` | Priest 1987 | `TrueContradiction` requires `Paraconsistent` logic via `Requires` edge |

## Cross-functors in

Dialectics is the target of cross-functors from older/adjacent ontologies:

- `Syntrometry → Dialectics` (in `formal::meta::syntrometry::dialectics_functor`) — Heim's `Dialektik` ↦ `DialecticalMoment`; Synkolator/Korporator/Maxime/Transzendenzstufe ↦ `Sublation`; Aspekt/Telecenter ↦ `Synthesis`. Verified by `syntrometry_to_dialectics_laws_pass`.

Future cross-functors: `Distinction → Dialectics` (Spencer-Brown `Boundary` ↦ `DeterminateNegation`); `Dialectics → Opposition`-reasoning module consumption (already handled via the `opposes:` block in `ontology.rs`).

## Files

- `ontology.rs` — the ontology, 6 domain axioms, tradition quality
- `mod.rs` — module wiring
- `README.md`, `citings.md` — this file + bibliography
