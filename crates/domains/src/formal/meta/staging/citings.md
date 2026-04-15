# Citings — Staging (Multi-stage computation and partial evaluation)

Every published source the staging ontology stands on. Each entry includes the full citation, a DOI/URL where available, and a one-line annotation of which concepts, axioms, or relations the source grounds.

This is the per-ontology bibliography. The workspace-wide bibliography of all sources cited by all ontologies lives at [`docs/papers/references.md`](../../../../../../docs/papers/references.md).

## Primary sources

### Futamura 1971 — *Partial Evaluation of Computation Process — an Approach to a Compiler-Compiler*

- **Citation**: Futamura, Y. (1971). *Partial Evaluation of Computation Process — an Approach to a Compiler-Compiler*. Systems, Computers, Controls, 2(5). Translated from Denshi Tsushin Gakkai Ronbunshi, 54-C(8), 721–728 (August 1971). Republished in *Higher-Order and Symbolic Computation* 12(4): 381–391 (1999).
- **Local PDF**: [`papers/futamura-1971-partial-evaluation.pdf`](papers/futamura-1971-partial-evaluation.pdf)
- **URL**: [fi.ftmr.info/PE-Museum/PE-Original-English1971.pdf](https://fi.ftmr.info/PE-Museum/PE-Original-English1971.pdf) (English translation)
- **Grounds**: the entire ontology. The three Futamura projections, the partial-evaluation equation (π(c,r) = α(π,c)(r)), the distinction between partial-evaluation variables and remaining variables, properties p1 and p2, and the claim that a compiler-generator is derivable from a specializer applied to itself. Every entity and every domain axiom traces to a specific passage in this six-page paper.
- **Cited at**: `ontology.rs:1` (module doc comment), the doc comment on `StageConcept`, and all 5 domain axiom `description()` strings

### Jones, Gomard & Sestoft 1993 — *Partial Evaluation and Automatic Program Generation*

- **Citation**: Jones, N. D., Gomard, C. K., & Sestoft, P. (1993). *Partial Evaluation and Automatic Program Generation*. Prentice Hall International Series in Computer Science. ISBN 0-13-020249-5.
- **URL**: [`http://www.itu.dk/people/sestoft/pebook/`](http://www.itu.dk/people/sestoft/pebook/) (free PDF from one of the authors)
- **Grounds**: the modern formalization of partial evaluation as a program transformation. Provides the termination criteria, binding-time analysis, and offline vs online partial evaluation distinctions that pr4xis's staging ontology abstracts away from but should eventually be referenced for implementation detail.
- **Cited at**: `ontology.rs:1` (module doc comment)

### Taha & Sheard 1997 — *Multi-Stage Programming with Explicit Annotations*

- **Citation**: Taha, W. & Sheard, T. (1997). *Multi-Stage Programming with Explicit Annotations*. In Proceedings of the 1997 ACM SIGPLAN Symposium on Partial Evaluation and Semantics-Based Program Manipulation (PEPM '97), pp. 203–217.
- **DOI**: [10.1145/258993.259019](https://doi.org/10.1145/258993.259019)
- **Grounds**: the staged-computation lineage descended from Futamura. Introduces explicit brackets and escapes for staging-level annotations — the formal machinery behind `StagingLevel` as a first-class quality. The ontology's use of `usize` staging levels follows this paper's convention (level 0 = unstaged, each bracket raises the level by 1).
- **Cited at**: `ontology.rs:1` (module doc comment), `StagingLevel` quality

## Supporting sources

- **Burstall & Darlington 1977** — *A Transformation System for Developing Recursive Programs* (JACM 24(1)). Earlier program-transformation work that partial evaluation descended from. Not directly cited in the ontology but relevant for the broader program-transformation literature.
- **Ershov 1977** — *On the Partial Computation Principle* (Information Processing Letters 6(2)). Independent formulation of partial evaluation around the same time as Futamura.
- **Kleene 1952** — *Introduction to Metamathematics*. The s-m-n theorem that partial evaluation concretely realizes. Foundational theoretical underpinning (Kleene's first recursion theorem and the s-m-n theorem together guarantee that partial evaluation is always possible in principle).

## Pending verification

Each of the following would deepen the ontology but requires the human to read the source and confirm the mapping:

- [ ] **Andersen 1994** — *Program Analysis and Specialization for the C Programming Language*. PhD dissertation. Would ground a concrete instance of partial evaluation applied to a non-trivial language (C).
- [ ] **Consel & Danvy 1993** — *Tutorial Notes on Partial Evaluation* (POPL 1993). Good introductory framing; could inform the README.
- [ ] **Sheard & Peyton Jones 2002** — *Template Meta-programming for Haskell*. Establishes Template Haskell as a staged computation system; useful for the "Rust macro as partial evaluator" analogy pr4xis already uses.

---

- **Document date:** 2026-04-14
- **How this file is maintained**: source citations live in `ontology.rs` doc comments and in this file. When a new source is added, add a `// Source: ...` comment in the relevant Rust file, then update this file. The `per-ontology-citings` skill automates the extraction from doc comments.
