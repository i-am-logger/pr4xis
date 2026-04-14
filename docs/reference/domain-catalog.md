# Domain Catalog

> **Note (2026-04-14):** This page is in transition. The previous catalog enumerated ~21 domains using a `science::math` / `games::chess` / `systems::*` organization that no longer matches the codebase. The current workspace contains 106 ontologies under a different structure (`formal/applied/social/natural/cognitive`), and a hand-maintained catalog of all 106 entries is not the right shape going forward. Instead, this page now points at the **canonical source**, the **current organization**, and the **two pieces of in-flight work** that will replace it: per-ontology READMEs ([#57](https://github.com/i-am-logger/pr4xis/issues/57)) and the source-of-truth report pipeline ([#60](https://github.com/i-am-logger/pr4xis/issues/60)).

## Canonical source

Every ontology lives at exactly one path under `crates/domains/src/`. The full list is re-derivable by:

```
find crates/domains/src -name ontology.rs
```

Total: 106 ontologies.

Every ontology directory contains an `ontology.rs` file with the `define_ontology!` invocation that declares its concepts, relations, reasoning systems, axioms, and metadata. To understand any specific ontology, read its `ontology.rs` directly. Per-ontology `README.md` and `citings.md` files are pending [#57](https://github.com/i-am-logger/pr4xis/issues/57).

## Current organization

```
crates/domains/src/
├── formal/                      formal sciences — math, information, calculator, meta
│   ├── math/                    linear algebra, geometry, probability, statistics, signal
│   │                            processing, control theory, rotation, temporal, quantity
│   ├── information/             communication, concurrency, dialogue, events, knowledge,
│   │                            measurement, provenance, schema, storage, systems, diagnostics
│   ├── calculator/              scientific calculator with exact rationals
│   └── meta/                    ontology diagnostics, gap analysis
│
├── applied/                     applied engineering domains
│   ├── sensor_fusion/           Kalman filter, observation, state, time, frame, fusion
│   ├── navigation/              AHRS, GNSS, IMU, INS-GNSS, celestial, odometry
│   ├── perception/              occupancy grid, lidar-camera, radar-camera fusion
│   ├── tracking/                single-target, multi-target, radar tracking
│   ├── space/                   orbital mechanics, attitude determination
│   ├── underwater/              sonar, AUV control
│   ├── industrial/              process, structural engineering
│   ├── localization/            SLAM, terrain
│   ├── hardware/                elevator dispatch, traffic signal control
│   └── theming/                 base16 color theme validation, WCAG contrast
│
├── social/                      social and human-system domains
│   ├── games/                   chess, rubik, tetris, simon
│   ├── software/                HTTP state machine, XML, OWL, RDF, LMF
│   ├── judicial/                case lifecycle, motion workflow, evidence, burden of proof
│   ├── compliance/              escalation ladder
│   └── military/                electronic warfare, situation awareness
│
├── natural/                     natural sciences
│   ├── physics/                 kinematics, relativity, energy
│   ├── biomedical/              biology, molecular, bioelectricity, biochemistry,
│   │                            biophysics, mechanobiology, immunology, pharmacology,
│   │                            pathology, hematology, electrophysiology, regeneration,
│   │                            chemistry, acoustics
│   ├── hearing/                 acoustics, anatomy, audiology, neuroscience, psychoacoustics,
│   │                            transduction, vestibular, music perception, signal processing,
│   │                            speech, environmental, devices, pathology, bone conduction
│   ├── geodesy/                 coordinate systems, reference frames
│   ├── colors/                  RGB, WCAG contrast, color theory
│   ├── physics/                 mechanics, electromagnetism, energy
│   └── music/                   scales, chords, intervals
│
└── cognitive/                   cognitive and linguistic domains
    ├── linguistics/             english (WordNet), grammar, lambek pregroup, lexicon,
    │                            morphology, orthography, pragmatics, semantics, symbols
    └── cognition/               epistemics, metacognition, self-model
```

This tree is the schematic view; it is not exhaustive (some sub-modules are omitted for brevity). For the complete list of every ontology directory, run `find crates/domains/src -name ontology.rs`.

## Cross-domain functors

Domains are not isolated. They compose through proven functors — structure-preserving maps verified at test time. The current workspace contains 61 functor implementations:

```
grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/ | wc -l
```

The most distinctive use of the functor machinery is in the biomedical stack, where three categorical adjunctions (Molecular ⊣ Bioelectric, Biology ⊣ Bioelectric, Pharmacology ⊣ Molecular) automatically detect missing distinctions in the source ontologies. To see the live percentages of how much information is lost in each round-trip:

```
cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
```

For the explanation of what these numbers mean, see the [README](../../README.md#a-concrete-result-gap-detection-in-biomedical-ontologies) and [Concepts](../understand/concepts.md#adjunctions-and-gap-detection).

## Why this page is short

A domain catalog written by hand goes stale immediately. Every commit that adds, removes, or renames an ontology silently invalidates a hand-maintained list. The right shape for this page is **automated generation** from the codebase, plus per-ontology pages for depth. Both are in flight:

- **[#57](https://github.com/i-am-logger/pr4xis/issues/57)** — every ontology directory gets a `README.md` (one paragraph: what it models, its scope, its functor connections, its status) and a `citings.md` (the ontology's bibliography). The README is the abstract, citings.md is the bibliography.
- **[#59](https://github.com/i-am-logger/pr4xis/issues/59)** — every per-ontology README also gets two mermaid diagrams: an internal structure view (concepts and relations) and an external connections view (functors and adjunctions to other ontologies). Auto-generated from `Entity::variants()` and the existing functor implementations.
- **[#60](https://github.com/i-am-logger/pr4xis/issues/60)** — a CI-generated `pr4xis-report.json` that captures every numerical metric the codebase produces (test counts, ontology counts, functor counts, adjunction collapse percentages, per-ontology entity/relation counts) and publishes it to GitHub Pages so the README and this catalog can pull live numbers instead of hand-typing them.

When all three land, this page becomes either a redirect to a generated catalog or a thin wrapper around the live JSON.

## Related

- [README](../../README.md) — the project entry point
- [Architecture](../understand/architecture.md) — the five-layer Rust stack
- [Concepts](../understand/concepts.md) — what ontologies are and how they compose
- [Foundations](../understand/foundations.md) — academic lineage

---

- **Document date:** 2026-04-14
- **Verification:** the ontology count and functor count above are re-derivable by the cited `find` and `grep` commands. The structural tree is hand-typed and may drift; the canonical source of truth is `crates/domains/src/` itself.
