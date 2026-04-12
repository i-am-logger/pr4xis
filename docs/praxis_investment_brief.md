# Pr4xis — Axiomatic Intelligence

github.com/i-am-logger/pr4xis

Aristotle classified knowledge into three kinds: **episteme** (knowing how
things are), **techne** (knowing how to make things), and **praxis** (knowing
how to do the right thing). This is praxis — a system that doesn't just
compute, it understands what it's doing and can prove it's correct.

> "Every good regulator of a system must be a model of that system."
> — Conant & Ashby (1970)

All intelligence lives in composable ontologies — formal descriptions of
what exists and how things relate. Every transformation between domains is
a mathematically proven functor. Every claim has a proof. 3,592 of them.

## What It Is

Pr4xis is an axiomatic artificial intelligence.

It reasons from axioms — not from data, not from weights, not from
probabilities. 44 domains of knowledge, from physics to law to medicine,
encoded as mathematical categories where every claim is a theorem and
every theorem has a proof. If the reasoning is unsound, the proof fails.
If a domain is incomplete, the adjunctions detect what's missing.
If an action violates a precondition, it's blocked with an explanation.

Category theory is the foundation. Rust is the language. Proofs are
the output.

## The Platform

### 44 domains across science, technology, and governance:

**Science (10 + 19 new)**

Existing: physics (mechanics, electromagnetism, relativity, quantum), math
(linear algebra, probability, statistics, signal processing, control theory,
geometry), music (scales, chords, consonance), colors (RGB, WCAG contrast),
calculator (exact arithmetic, unit conversion), cognition (epistemics),
geodesy (coordinate systems), information (knowledge bases, concurrency,
dialogue), linguistics (English, pregroup grammar, Montague semantics),
systems (systems thinking, feedback, homeostasis)

New (pending integration): biology, molecular biology, biochemistry,
bioelectricity, regeneration, pharmacology, immunology, electrophysiology,
pathology, biophysics, acoustics, mechanobiology, chemistry, hematology,
analytical methods, recommendation, optimization, derivation,
ontology diagnostics

**Technology (12)**

Games (chess with full rules, Rubik's cube via group theory, Tetris, Simon),
hardware (traffic lights, elevators with dispatch), software (HTTP protocol
state machine, XML/RDF/OWL markup), sensor fusion (Kalman filter, covariance
intersection, observation models, validation gating, track management),
perception (occupancy grids, radar-camera fusion), tracking (Kalman
filtering), navigation (path planning), space (orbital mechanics),
underwater (sonar, buoyancy), industrial (structural engineering),
theming (hardware color profiles), localization

**Governance (3)**

Compliance (escalation ladder), judicial (case lifecycle, motion workflow,
evidence rules, burden of proof), military (electronic warfare)

### What connects them

24 functors (structure-preserving maps between domains) prove that one
domain's structure IS another's — not by analogy, by mathematical proof.
3 adjunctions capture scale-bridging relationships and automatically
detect missing knowledge.

## By the Numbers

| Metric | Praxis core | New domains | Combined |
|--------|-------------|-------------|----------|
| Domains | 25 | 19 | 44 |
| Tests | 2,471 | 1,121 | 3,592 |
| Axioms | ~400 | 226 | ~626 |
| Functors | ~30 | 24 | ~54 |
| Lines of Rust | ~50,000 | 25,429 | ~75,000 |

Plus: 5,120 property-based test permutations, 20 opposition sets,
21 taxonomies, 19 causal graphs, 3 adjunctions, 1 ContextDef.

## What We Discovered

When we connected molecular biology to bioelectricity via an adjunction,
the math detected a missing distinction: potassium channels serve two
roles (homeostatic vs therapeutic) that the ontology had collapsed into
one entity — 85.2% of molecular entities lost their identity in the
round-trip.

This is the first time categorical adjunctions have been used to
automatically detect missing distinctions in scientific ontologies. Every
gap we found corresponds to a published scientific fact. The methodology
is general — it works on any pair of domains.

We also measured the full abstraction ladder from acoustics to
bioelectricity: 26 concepts compress to 2 across four domains (92.3%
end-to-end collapse). Each domain is a lossy compression of the one
below — known qualitatively in biology, quantified for the first time.

## Praxis vs LLMs

| | LLMs | Praxis |
|---|---|---|
| **How it knows** | Learned from training data | Derived from axioms |
| **Correctness** | Probable (next-token prediction) | Provable (category laws + axioms) |
| **Hallucination** | Inherent — no ground truth | Impossible — every output traces to a proof |
| **Determinism** | Stochastic (temperature, sampling) | Absolute — same input, same proof, always |
| **Traceability** | Opaque (billions of weights) | Full proof path from conclusion back to axiom(with citing) |
| **Cross-domain** | Blends domains implicitly | Functors PROVE domain connections explicitly |
| **When wrong** | Confidently wrong, hard to find why | Axiom violation: tells you WHICH claim failed and WHY |
| **Missing knowledge** | Doesn't know what it doesn't know | Adjunctions detect gaps automatically |
| **Scale** | GPU clusters, terabytes | Single core, megabytes, <1 second |
| **Proofs** | 0 | 3,592 |

LLMs answer questions. Praxis proves theorems.

An LLM can tell you "Piezo1 is a mechanosensitive ion channel." Praxis
proves it: `taxonomy::is_a(Piezo1, Mechanosensitive)` returns true because
Piezo1 → Mechanosensitive → IonChannel in the taxonomy DAG, verified by
category laws, grounded in Coste et al. 2010 (Nobel 2021). The LLM might
be right. Praxis IS right — or the tests fail.

An LLM can't tell you what's MISSING from your knowledge. Praxis can:
the adjunction between molecular biology and bioelectricity detected that
Kv channels serve dual roles — a distinction documented in literature but
absent from the ontology until the math found it.

## Why It Matters

**Cross-domain reasoning.** Functors prove that pharmacology → molecular
→ bioelectricity composes correctly. The chain is verified end-to-end.
Change one domain's structure and dependent proofs break immediately —
you know exactly what's affected.

**Gap detection.** Adjunctions find what's MISSING in your knowledge,
not just what's wrong. No other system does this.

**Any domain.** Chess, law, physics, medicine, linguistics — same
framework, same guarantees. The 44 domains demonstrate breadth.
Category theory is universal.

## Three Papers (in draft)

1. Categorical formalization of bioelectric morphogenesis (19 domains, 1121 proofs)
2. Adjunction-based information loss across biological scales
3. Ontology diagnostics: automated gap detection via unit/counit analysis

## Tech

Rust. Category theory. CC-BY-NC-SA-4.0.

github.com/i-am-logger/pr4xis
