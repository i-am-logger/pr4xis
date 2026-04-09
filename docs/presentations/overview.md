---
marp: true
theme: default
paginate: true
backgroundColor: #0d1117
color: #c9d1d9
style: |
  section {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
    font-size: 28px;
  }
  h1 { color: #58a6ff; font-size: 1.6em; }
  h2 { color: #c9d1d9; font-size: 1.3em; }
  a { color: #58a6ff; }
  table { font-size: 0.85em; width: 100%; border-collapse: collapse; background: transparent; }
  th { color: #58a6ff; font-size: 0.9em; border-bottom: 2px solid #30363d; background: #161b22; padding: 0.5em 0.7em; }
  td { color: #e6edf3; padding: 0.5em 0.7em; border-bottom: 1px solid #21262d; background: #0d1117; }
  tr:nth-child(even) td { background: #161b22; }
  code { background: #161b22; color: #e6edf3; padding: 0.1em 0.4em; border-radius: 4px; }
  pre { background: #161b22 !important; border: 1px solid #30363d; border-radius: 6px; padding: 1em; }
  pre code { background: transparent !important; }
  strong { color: #f0f6fc; }
  em { color: #8b949e; }
  .pass { color: #3fb950; }
  .fail { color: #f85149; }
  .ignore { color: #d29922; }
---

# Praxis

**Axiomatic intelligence.**

*Knowing how to do the right thing — with proof.*

> "Every good regulator of a system must be a model of that system."
> — Conant & Ashby (1970)

---

# What is Praxis?

Aristotle classified knowledge into three kinds:

- **Episteme** — knowing how things ARE (science)
- **Techne** — knowing how to MAKE things (technology)
- **Praxis** — knowing how to DO the right thing (action with understanding)

**The code contains zero domain knowledge.**
All intelligence lives in composable ontologies.
Every transformation is a proven functor.
Every claim has a proof. **1,822 of them.**

---

# How it works

```
Define rules (Ontology) → Check rules (Engine) → Prove rules hold (Tests)
         ↑                                              |
         └──────────── feedback ────────────────────────┘
```

The ontology IS the model. The engine IS a control system.
Conant-Ashby (1970) proven in code: `Model → Ontology`.

---

# The Architecture

| Layer | What it does |
|---|---|
| **Logic** | Axioms, propositions, inference |
| **Category** | Entities, relationships, composition, functors |
| **Ontology** | Domain knowledge, reasoning patterns, DOLCE |
| **Engine** | Situations, actions, preconditions, enforcement |
| **Codegen** | Build-time ontology generation from data sources |

Zero domain logic in framework code. Adding chess, physics, or English changes nothing in the engine.

---

# Domains compose via functors

If Chess IS EventDriven (functor) and EventDriven IS Concurrent (functor),
then Chess IS Concurrent (composition). **The proof is automatic.**

| Functor | Proof |
|---|---|
| Traffic → Systems | Identity + composition preserved |
| Chess → EventDriven | Identity + composition preserved |
| Chess → Concurrent | Composed from above |
| Dialogue → Communication | Identity + composition preserved |
| Control → Engine | Plant→Situation, Model→Ontology |
| Lambek → Pregroup | Parsing preserved across type systems |

---

# Natural Language Pipeline

```
Text → Language::lexical_lookup → Pregroup types → Contract → Semantics → Response
```

- **Language trait** — language-agnostic. English, Hebrew, same code.
- **Pregroup grammar** — parsing as group algebra (Lambek 1999)
- **Montague functor** — syntax → semantics (proven)
- **Zero hardcoded words** — everything through the ontology

`np · n^l · n · np^r · s` → contract → `s` — "the dog runs" parsed by algebra.

---

# Turing Test Benchmark

Questions from real competitions (Loebner Prize, Winograd Schema Challenge).

| Level | Status | What's needed |
|---|---|---|
| Taxonomy ("is a dog a mammal?") | <span class="pass">3 PASS</span> | WordNet (107K concepts) |
| Grammar ("the dog runs") | <span class="ignore">2 PENDING</span> | Pregroup pipeline |
| Factual ("capital of France?") | <span class="ignore">3 PENDING</span> | Geography, Literature, Mereology |
| Reasoning ("brick weight puzzle") | <span class="ignore">2 PENDING</span> | Arithmetic ontology |
| Winograd ("trophy too big") | <span class="ignore">3 PENDING</span> | Physical world + DRT |
| Common sense ("drop an egg?") | <span class="ignore">2 PENDING</span> | Material/sensation ontology |
| Social ("how are you?") | <span class="ignore">1 PENDING</span> | Social dialogue ontology |
| Meta ("are you a computer?") | <span class="ignore">2 PENDING</span> | Self-model + metacognition |

**Every pending test = a research task. When the ontology is built, the test passes.**

---

# Praxis vs LLMs

| | LLMs | Praxis |
|---|---|---|
| **Method** | Statistical prediction | Ontological reasoning |
| **Knowledge** | Training data (terabytes) | Formal ontologies (megabytes) |
| **Correctness** | Probable | Provable |
| **Hardware** | GPU clusters | Single core, <5 seconds |
| **Hallucination** | Frequent | Impossible (axiom-checked) |
| **Explainability** | Opaque weights | Full trace of every step |
| **Language** | Python/CUDA | Rust |
| **Proofs** | 0 | 1,822 |

---

# Novel Contribution

**Praxis Architecture** — a synthesis that doesn't exist in the literature:

1. **Zero domain logic in code** — all in ontologies (Guarino 1998 proposed, never built)
2. **Functorial composition** — between behavioral ontologies (Spivak did data, not behavior)
3. **Conant-Ashby as design principle** — never applied to software architecture
4. **DOLCE + category theory unified** — always used separately before
5. **Composable proof chains** — if A IS B and B IS C, then A IS C (automatic)

---

# Academic Foundation

50+ papers, all downloaded and cited:

- **Category theory** — Mac Lane, Awodey, Spivak (ologs)
- **Control systems** — Wiener, Ashby, Conant-Ashby, Powers
- **Formal ontology** — DOLCE, Guarino, Gangemi (ODPs)
- **Linguistics** — Lambek, Montague, Kamp (DRT), Steedman (CCG)
- **Information** — Shannon, Damerau, Brill & Moore
- **Metacognition** — von Foerster, Spencer-Brown

Full lineage: `docs/foundations.md`

---

# Live Demo

<iframe src="../chat/" width="100%" height="500" style="border: 1px solid #30363d; border-radius: 8px; background: #0d1117;"></iframe>

*107,000 concepts. 4.2MB. Running in your browser right now. No server.*

---

# What's next

- **More ontologies** — each Turing test question that fails tells us what to build
- **The paper** — "Praxis Architecture: Zero-Knowledge Code via Composable Domain Ontologies with Categorical Proof"
- **Hebrew** — second language proves the architecture is language-agnostic

**The name praxis is not marketing. It is a claim backed by 1,822 proofs.**
