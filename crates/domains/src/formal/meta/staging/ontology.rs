//! Staging — the theory of multi-stage computation and partial evaluation.
//!
//! Formalizes Futamura's partial-evaluation framework as a meta-ontology.
//! The central operator is α (a `Specializer`): given a program π and a
//! static input c, α produces a residual program that accepts only the
//! remaining dynamic input r and returns the same observable result as
//! running π on (c, r) directly.
//!
//! # The three Futamura projections
//!
//! | # | Equation | Meaning |
//! |---|---|---|
//! | 1 | α(int, s) = compile(s) | Specializing an interpreter with respect to a source program yields an object program |
//! | 2 | α(α, int) = compiler | Specializing α with respect to an interpreter yields a compiler |
//! | 3 | α(α, α) = cogen | Specializing α with respect to itself yields a compiler-compiler |
//!
//! # Why this lives in `formal/meta/`
//!
//! This ontology is meta because it describes how *other* ontologies and
//! programs relate to each other under the dynamic/static axis. pr4xis has
//! several instances of the `freeze: Dynamic → Static` functor already:
//!
//! - **codegen** = partial evaluation of an ontology loader at build time
//!   with respect to a fixed ontology dataset, producing a const-table
//!   residual program (the compiled binary).
//! - **async ontology load** = total evaluation of the same loader with no
//!   partial-evaluation stage — the ontology stays dynamic until runtime.
//! - **report generation** (PDF, HTML snapshot, SVG) = partial evaluation
//!   of an interactive visualization with respect to a specific point in
//!   time, producing a frozen rendered artifact.
//! - **session archive** = freezing a live chat session into a static log.
//!
//! The memory `project_async_functor.md` (codegen and async are functors)
//! is the informal statement of Futamura's second projection applied to
//! pr4xis's ontology loading.
//!
//! # Sources
//!
//! - Futamura 1971: *Partial Evaluation of Computation Process — an Approach
//!   to a Compiler-Compiler* (Systems, Computers, Controls Vol. 2 No. 5).
//!   The paper that introduced the three projections.
//! - Jones, Gomard, Sestoft 1993: *Partial Evaluation and Automatic Program
//!   Generation* (Prentice Hall). Book-length treatment of the theory and
//!   its algorithms.
//! - Taha & Sheard 1997: *Multi-Stage Programming with Explicit Annotations*
//!   (PEPM 1997). The staged-computation lineage, direct descendant.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::{causation, taxonomy};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// Core concepts of multi-stage computation.
///
/// A program is a computation process (Futamura's π). Interpreters, compilers,
/// and specializers are kinds of programs that operate on other programs. The
/// specializer is the hero of the paper — it is the algorithm α that moves
/// work from "total evaluation time" to "partial evaluation time", producing
/// a residual program that is equivalent to the original but with the static
/// parts already evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum StageConcept {
    // --- Programs ---
    /// A computation process: a program or procedure that transforms inputs
    /// into outputs. Futamura's π.
    Program,

    /// A program that evaluates another program given its source code and
    /// runtime input. Takes (source, dynamic) and produces the observable
    /// output.
    Interpreter,

    /// A program that transforms a source program into an object program.
    /// The object program can then be run directly without interpretation.
    Compiler,

    /// A program that performs partial evaluation — Futamura's α. Given a
    /// program π and a static input c, it produces a residual program that
    /// accepts only the remaining dynamic input and preserves the semantics
    /// of π.
    Specializer,

    /// A program that generates a compiler from an interpreter. The output
    /// of the third Futamura projection: α(α, α) = cogen.
    CompilerGenerator,

    // --- Program artifacts ---
    /// The input to an interpreter or compiler: the program being processed.
    SourceProgram,

    /// The output of a compiler: a compiled program ready for direct
    /// execution without interpretation.
    ObjectProgram,

    /// The output of a specializer: a program equivalent to the original
    /// but with the static parts already evaluated.
    ResidualProgram,

    // --- Inputs ---
    /// Values known at partial-evaluation time (c₁..cₘ in Futamura).
    /// Paired with a program, they become the specialization target.
    StaticInput,

    /// Values known only at total-evaluation time (r₁..rₙ in Futamura).
    /// The residual program accepts these and nothing else.
    DynamicInput,
}

// ---------------------------------------------------------------------------
// Futamura causal graph
// ---------------------------------------------------------------------------

/// The step-ordering of how Futamura's projections compose to derive
/// compilers from interpreters, and compiler-generators from specializers.
///
/// This is not a lifecycle — it is the direction of causation: each step
/// depends on the prior step's output being available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum FutamuraStep {
    /// We have an interpreter `int` and a source program `s`.
    WriteInterpreter,

    /// Apply α to (int, s). By equation (1): α(int, s)(r) = int(s, r).
    /// The result is a residual program that only needs `r`.
    SpecializeInterpreter,

    /// That residual program is, by definition, the object code for `s`.
    /// This is Futamura's first projection.
    ProduceObjectProgram,

    /// Apply α to (α, int). The result is a program that, given any source
    /// `s`, produces the object program for `s`. This is Futamura's second
    /// projection — a compiler.
    GenerateCompiler,

    /// Apply α to (α, α). The result is a program that, given any
    /// interpreter, produces a compiler. This is Futamura's third projection
    /// — a compiler-generator.
    GenerateCompilerGenerator,
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

define_ontology! {
    /// Dense category over stage concepts, with taxonomy (program kinds),
    /// causation (Futamura projection chain), and opposition (static vs
    /// dynamic).
    ///
    /// Every `Interpreter`, `Compiler`, `Specializer`, and `CompilerGenerator`
    /// is-a `Program`. Every `SourceProgram`, `ObjectProgram`, and
    /// `ResidualProgram` is-a `Program`. The causation graph is the ordered
    /// chain of Futamura's three projections. The opposition pair
    /// `StaticInput` / `DynamicInput` is the dynamic/static axis that the
    /// whole ontology is about.
    pub StagingOntology for StagingCategory {
        entity: StageConcept,
        relation: StagingRelation,
        being: AbstractObject,
        source: "Futamura (1971); Jones, Gomard & Sestoft (1993)",

        taxonomy: StagingTaxonomy [
            // Program kinds
            (Interpreter, Program),
            (Compiler, Program),
            (Specializer, Program),
            (CompilerGenerator, Program),
            // Program artifacts are also programs (they can be run)
            (SourceProgram, Program),
            (ObjectProgram, Program),
            (ResidualProgram, Program),
        ],

        causation: FutamuraCausalGraph for FutamuraStep [
            // Futamura first projection: specialize int wrt s yields object code
            (WriteInterpreter, SpecializeInterpreter),
            (SpecializeInterpreter, ProduceObjectProgram),
            // Second projection: specializing α wrt int yields a compiler
            (ProduceObjectProgram, GenerateCompiler),
            // Third projection: specializing α wrt α yields a compiler-generator
            (GenerateCompiler, GenerateCompilerGenerator),
        ],

        opposition: StagingOpposition [
            // The dynamic/static axis — the whole ontology is about moving
            // computation between these two sides.
            (StaticInput, DynamicInput),
            // Interpretation vs compilation: same semantics, different
            // staging. Interpretation keeps everything dynamic; compilation
            // lifts as much as possible to static.
            (Interpreter, Compiler),
        ],
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Temporality tag: does a concept represent dynamic (runtime) or static
/// (build-time) computation? `Mixed` for programs that operate on both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Temporality {
    /// Evaluated only at total evaluation time (runtime).
    Dynamic,
    /// Known at partial evaluation time (build time).
    Static,
    /// A program that takes both static and dynamic inputs — the general case.
    Mixed,
}

/// Each concept carries a temporality tag. `StaticInput` is Static,
/// `DynamicInput` is Dynamic, programs are Mixed.
#[derive(Debug, Clone)]
pub struct TemporalityTag;

impl Quality for TemporalityTag {
    type Individual = StageConcept;
    type Value = Temporality;

    fn get(&self, concept: &StageConcept) -> Option<Temporality> {
        use StageConcept::*;
        Some(match concept {
            StaticInput => Temporality::Static,
            DynamicInput => Temporality::Dynamic,
            Program | Interpreter | Compiler | Specializer | CompilerGenerator | SourceProgram
            | ObjectProgram | ResidualProgram => Temporality::Mixed,
        })
    }
}

/// Staging level: how many partial-evaluation stages have been applied
/// before total evaluation runs. Futamura's projections correspond to
/// specific staging-level transitions:
///
/// - Interpreter has staging level 0 (everything dynamic).
/// - Object program has staging level 1 (one partial evaluation).
/// - Compiler (from second projection) has staging level 2.
/// - Cogen (from third projection) has staging level 3.
#[derive(Debug, Clone)]
pub struct StagingLevel;

impl Quality for StagingLevel {
    type Individual = StageConcept;
    type Value = usize;

    fn get(&self, concept: &StageConcept) -> Option<usize> {
        use StageConcept::*;
        Some(match concept {
            Program => 0,
            Interpreter => 0,
            SourceProgram => 0,
            DynamicInput => 0,
            ObjectProgram => 1,
            ResidualProgram => 1,
            StaticInput => 1,
            Compiler => 2,
            Specializer => 2,
            CompilerGenerator => 3,
        })
    }
}

// ---------------------------------------------------------------------------
// Domain axioms — verifiable properties of the Futamura framework
// ---------------------------------------------------------------------------

/// Every program kind sits under the `Program` taxonomy root.
///
/// This is the structural prerequisite of the whole ontology: if a concept
/// does not descend from `Program`, it cannot participate in specialization.
pub struct EveryProgramKindIsAProgram;

impl Axiom for EveryProgramKindIsAProgram {
    fn description(&self) -> &str {
        "every program-like concept is-a Program in the taxonomy"
    }
    fn holds(&self) -> bool {
        use StageConcept::*;
        let program_kinds = [
            Interpreter,
            Compiler,
            Specializer,
            CompilerGenerator,
            SourceProgram,
            ObjectProgram,
            ResidualProgram,
        ];
        program_kinds
            .iter()
            .all(|k| taxonomy::ancestors::<StagingTaxonomy>(k).contains(&Program))
    }
}
pr4xis::register_axiom!(EveryProgramKindIsAProgram);

/// The Futamura projection chain is complete: starting from
/// `WriteInterpreter` you can causally reach `GenerateCompilerGenerator`.
///
/// Source: Futamura 1971, Section 3. The three projections compose — a
/// compiler-generator is derivable from a specializer applied to itself,
/// which requires the prior projections to be meaningful.
pub struct FutamuraChainIsComplete;

impl Axiom for FutamuraChainIsComplete {
    fn description(&self) -> &str {
        "the three Futamura projections form a connected causal chain"
    }
    fn holds(&self) -> bool {
        use FutamuraStep::*;
        let reach = causation::effects_of::<FutamuraCausalGraph>(&WriteInterpreter);
        reach.contains(&ProduceObjectProgram)
            && reach.contains(&GenerateCompiler)
            && reach.contains(&GenerateCompilerGenerator)
    }
}
pr4xis::register_axiom!(FutamuraChainIsComplete);

/// Compilation is downstream of specialization.
///
/// Source: Futamura 1971, Equation (2) — int(s', r') = α(int, s')(r').
/// The left side (interpretation) is ordinary; the right side
/// (specialization) is the new operation. Producing an object program
/// requires specializing the interpreter first, not the other way around.
pub struct CompilationFollowsSpecialization;

impl Axiom for CompilationFollowsSpecialization {
    fn description(&self) -> &str {
        "producing an object program is caused by specializing the interpreter (Futamura Eq. 2)"
    }
    fn holds(&self) -> bool {
        use FutamuraStep::*;
        let causes = causation::causes_of::<FutamuraCausalGraph>(&ProduceObjectProgram);
        causes.contains(&SpecializeInterpreter)
    }
}
pr4xis::register_axiom!(CompilationFollowsSpecialization);

/// Each Futamura projection raises the staging level by exactly 1.
///
/// - Interpreter at level 0, object program at level 1: first projection adds 1.
/// - Object program at level 1, compiler at level 2: second projection adds 1.
/// - Compiler at level 2, cogen at level 3: third projection adds 1.
///
/// This is the formal content of "each projection moves more work to an
/// earlier stage". Quantitatively: Δ staging_level = +1 per projection.
pub struct EachProjectionRaisesStagingByOne;

impl Axiom for EachProjectionRaisesStagingByOne {
    fn description(&self) -> &str {
        "each Futamura projection raises the staging level by exactly 1"
    }
    fn holds(&self) -> bool {
        let q = StagingLevel;
        // interpreter → object program via first projection: 0 → 1
        let int = q.get(&StageConcept::Interpreter).unwrap_or_default();
        let obj = q.get(&StageConcept::ObjectProgram).unwrap_or_default();
        if obj != int + 1 {
            return false;
        }
        // object program → compiler via second projection: 1 → 2
        let cmp = q.get(&StageConcept::Compiler).unwrap_or_default();
        if cmp != obj + 1 {
            return false;
        }
        // compiler → compiler-generator via third projection: 2 → 3
        let cogen = q.get(&StageConcept::CompilerGenerator).unwrap_or_default();
        cmp + 1 == cogen
    }
}
pr4xis::register_axiom!(EachProjectionRaisesStagingByOne);

/// Static and dynamic inputs partition the input space of any program.
///
/// A program's arguments are split into exactly two groups: those known at
/// partial evaluation time (static) and those known only at total evaluation
/// time (dynamic). There is no third category. In the ontology this shows
/// up as opposition between `StaticInput` and `DynamicInput`.
///
/// Source: Futamura 1971, Section 2. "A computation process π containing m+n
/// variables c₁..cₘ, r₁..rₙ."
pub struct StaticDynamicPartitionsInputs;

impl Axiom for StaticDynamicPartitionsInputs {
    fn description(&self) -> &str {
        "static and dynamic inputs form the only two temporality classes"
    }
    fn holds(&self) -> bool {
        let q = TemporalityTag;
        // StaticInput is Static, DynamicInput is Dynamic, nothing in between
        q.get(&StageConcept::StaticInput) == Some(Temporality::Static)
            && q.get(&StageConcept::DynamicInput) == Some(Temporality::Dynamic)
    }
}
pr4xis::register_axiom!(StaticDynamicPartitionsInputs);

// ---------------------------------------------------------------------------
// Ontology trait impl
// ---------------------------------------------------------------------------

impl Ontology for StagingOntology {
    type Cat = StagingCategory;
    type Qual = TemporalityTag;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EveryProgramKindIsAProgram),
            Box::new(FutamuraChainIsComplete),
            Box::new(CompilationFollowsSpecialization),
            Box::new(EachProjectionRaisesStagingByOne),
            Box::new(StaticDynamicPartitionsInputs),
        ]
    }
}
