//! Staging — the theory of multi-stage computation and partial
//! evaluation. Formalises Futamura's framework as a meta-ontology.
//!
//! The central operator is α (a `Specializer`): given a program π and a
//! static input c, α produces a residual program that accepts only the
//! remaining dynamic input r and returns the same observable result as
//! running π on (c, r) directly.
//!
//! # The three Futamura projections
//!
//! | # | Equation | Meaning |
//! |---|---|---|
//! | 1 | α(int, s) = compile(s) | Specialising an interpreter with respect to a source program yields an object program |
//! | 2 | α(α, int) = compiler  | Specialising α with respect to an interpreter yields a compiler |
//! | 3 | α(α, α) = cogen        | Specialising α with respect to itself yields a compiler-compiler |
//!
//! # Literature
//!
//! - **Futamura (1971)** "Partial Evaluation of Computation Process — An
//!   Approach to a Compiler-Compiler", *Systems, Computers, Controls*
//!   2(5):45-50 — the three projections.
//! - **Jones, Gomard & Sestoft (1993)** *Partial Evaluation and
//!   Automatic Program Generation*, Prentice Hall — book-length
//!   treatment of the theory and its algorithms.
//! - **Taha & Sheard (1997)** "Multi-Stage Programming with Explicit
//!   Annotations", *PEPM 1997* — the staged-computation lineage.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Staging",
    source: "Futamura (1971) Partial Evaluation of Computation Process - an Approach to a Compiler-Compiler, Systems Computers Controls 2(5):45-50; Jones, Gomard & Sestoft (1993) Partial Evaluation and Automatic Program Generation, Prentice Hall; Taha & Sheard (1997) Multi-Stage Programming with Explicit Annotations, PEPM",

    concepts: [
        // === Programs ===
        Program,
        Interpreter,
        Compiler,
        Specializer,
        CompilerGenerator,
        // === Program artifacts ===
        SourceProgram,
        ObjectProgram,
        ResidualProgram,
        // === Inputs ===
        StaticInput,
        DynamicInput,
        // === Pipeline stages (Futamura 1971 §3) ===
        WriteInterpreter,
        SpecializeInterpreter,
        ProduceObjectProgram,
        GenerateCompiler,
        GenerateCompilerGenerator,
    ],

    labels: {
        Program: ("en", "Program", "Futamura (1971): a computation process π that transforms inputs into outputs."),
        Interpreter: ("en", "Interpreter", "Futamura (1971): a program that evaluates another given its source code and runtime input."),
        Compiler: ("en", "Compiler", "Futamura (1971) Eq. (2): a program that transforms source into object code."),
        Specializer: ("en", "Specializer", "Futamura (1971) α: given π and static c, produces a residual program accepting only dynamic r."),
        CompilerGenerator: ("en", "Compiler generator", "Futamura (1971) Eq. (3) third projection: α(α, α) = cogen - a program that generates compilers."),
        SourceProgram: ("en", "Source program", "The input to an interpreter or compiler."),
        ObjectProgram: ("en", "Object program", "The output of a compiler - ready for direct execution."),
        ResidualProgram: ("en", "Residual program", "The output of α - equivalent to the original but with static parts evaluated."),
        StaticInput: ("en", "Static input", "Futamura (1971) c₁..cₘ: values known at partial-evaluation time."),
        DynamicInput: ("en", "Dynamic input", "Futamura (1971) r₁..rₙ: values known only at total-evaluation time."),
        WriteInterpreter: ("en", "Write interpreter", "Methodology stage 1: have an interpreter int and a source program s."),
        SpecializeInterpreter: ("en", "Specialize interpreter", "Methodology stage 2: apply α to (int, s); residual needs only r. Futamura Eq. (1)."),
        ProduceObjectProgram: ("en", "Produce object program", "Methodology stage 3: residual = object code. Futamura first projection."),
        GenerateCompiler: ("en", "Generate compiler", "Methodology stage 4: apply α to (α, int). Futamura second projection."),
        GenerateCompilerGenerator: ("en", "Generate compiler-generator", "Methodology stage 5: apply α to (α, α). Futamura third projection - cogen."),
    },

    is_a: [
        // Program kinds.
        (Interpreter, Program),
        (Compiler, Program),
        (Specializer, Program),
        (CompilerGenerator, Program),
        // Program artifacts are programs (runnable).
        (SourceProgram, Program),
        (ObjectProgram, Program),
        (ResidualProgram, Program),
    ],

    causes: [
        // Futamura first projection.
        (WriteInterpreter, SpecializeInterpreter),
        (SpecializeInterpreter, ProduceObjectProgram),
        // Second projection.
        (ProduceObjectProgram, GenerateCompiler),
        // Third projection.
        (GenerateCompiler, GenerateCompilerGenerator),
    ],

    opposes: [
        // The dynamic/static axis - the whole ontology is about moving
        // computation between these.
        (StaticInput, DynamicInput),
        (DynamicInput, StaticInput),
        // Interpretation vs compilation - same semantics, different staging.
        (Interpreter, Compiler),
        (Compiler, Interpreter),
    ],
}

/// Temporality tag. Futamura (1971): the variables of π split into
/// static c₁..cₘ and dynamic r₁..rₙ. `Mixed` is for programs that
/// operate on both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Temporality {
    Dynamic,
    Static,
    Mixed,
}

/// Quality: temporality tag for each staging concept.
#[derive(Debug, Clone)]
pub struct TemporalityTag;

impl Quality for TemporalityTag {
    type Individual = StagingConcept;
    type Value = Temporality;

    fn get(&self, c: &StagingConcept) -> Option<Temporality> {
        use StagingConcept as S;
        match c {
            S::StaticInput => Some(Temporality::Static),
            S::DynamicInput => Some(Temporality::Dynamic),
            S::Program
            | S::Interpreter
            | S::Compiler
            | S::Specializer
            | S::CompilerGenerator
            | S::SourceProgram
            | S::ObjectProgram
            | S::ResidualProgram => Some(Temporality::Mixed),
            _ => None,
        }
    }
}

/// A grade in the Futamura-projection staging hierarchy. Declared in
/// ascending order so the derived `Ord` matches the projection count:
/// `Baseline < FirstProjection < SecondProjection < ThirdProjection`.
/// Named after Futamura's own terms for α applied once/twice/thrice
/// (Futamura 1971 §3, see the module-level projection table); `Baseline`
/// is the unstaged interpreter/program before any projection is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FutamuraStagingLevel {
    Baseline,
    FirstProjection,
    SecondProjection,
    ThirdProjection,
}

impl FutamuraStagingLevel {
    /// The level after one more Futamura projection is applied — Futamura
    /// (1971) §3: each application of α raises the level by exactly one.
    /// `None` past the third projection (cogen), the top of the hierarchy
    /// this ontology models.
    pub fn successor(&self) -> Option<Self> {
        match self {
            Self::Baseline => Some(Self::FirstProjection),
            Self::FirstProjection => Some(Self::SecondProjection),
            Self::SecondProjection => Some(Self::ThirdProjection),
            Self::ThirdProjection => None,
        }
    }

    /// The raw ordinal rank (0..=3), matching Futamura's projection count.
    pub fn rank(&self) -> usize {
        match self {
            Self::Baseline => 0,
            Self::FirstProjection => 1,
            Self::SecondProjection => 2,
            Self::ThirdProjection => 3,
        }
    }
}

/// Quality: staging level. Futamura projection arithmetic — each
/// projection raises the level by 1.
#[derive(Debug, Clone)]
pub struct StagingLevel;

impl Quality for StagingLevel {
    type Individual = StagingConcept;
    type Value = FutamuraStagingLevel;

    fn get(&self, c: &StagingConcept) -> Option<FutamuraStagingLevel> {
        use FutamuraStagingLevel as L;
        use StagingConcept as S;
        match c {
            S::Program | S::Interpreter | S::SourceProgram | S::DynamicInput => Some(L::Baseline),
            S::ObjectProgram | S::ResidualProgram | S::StaticInput => Some(L::FirstProjection),
            S::Compiler | S::Specializer => Some(L::SecondProjection),
            S::CompilerGenerator => Some(L::ThirdProjection),
            _ => None,
        }
    }
}

// Legacy alias.
pub type StageConcept = StagingConcept;

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Futamura (1971) §3: the projection chain is connected - starting from
/// WriteInterpreter we causally reach GenerateCompilerGenerator.
pub struct FutamuraChainIsComplete;

impl Axiom for FutamuraChainIsComplete {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let caus: Vec<_> = StagingCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == StagingRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        let ok = caus.contains(&(
            StagingConcept::WriteInterpreter,
            StagingConcept::GenerateCompilerGenerator,
        ));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FutamuraChainIsComplete",
        "WriteInterpreter transitively causes GenerateCompilerGenerator (Futamura projections compose)",
        "Futamura (1971) Partial Evaluation of Computation Process, Systems Computers Controls 2(5):45-50"
    );
}

pr4xis::register_axiom!(
    FutamuraChainIsComplete,
    "Futamura (1971) Partial Evaluation of Computation Process, Systems Computers Controls 2(5):45-50"
);

/// Each Futamura projection raises the staging level by exactly 1.
pub struct EachProjectionRaisesStagingByOne;

impl Axiom for EachProjectionRaisesStagingByOne {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let q = StagingLevel;
        let base = FutamuraStagingLevel::Baseline;
        let int = q.get(&StagingConcept::Interpreter).unwrap_or(base);
        let obj = q.get(&StagingConcept::ObjectProgram).unwrap_or(base);
        let cmp = q.get(&StagingConcept::Compiler).unwrap_or(base);
        let cogen = q.get(&StagingConcept::CompilerGenerator).unwrap_or(base);
        if int.successor() == Some(obj)
            && obj.successor() == Some(cmp)
            && cmp.successor() == Some(cogen)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EachProjectionRaisesStagingByOne",
        "interpreter → object program → compiler → cogen has staging levels 0 → 1 → 2 → 3",
        "Futamura (1971) Partial Evaluation of Computation Process, Systems Computers Controls 2(5):45-50"
    );
}

pr4xis::register_axiom!(
    EachProjectionRaisesStagingByOne,
    "Futamura (1971) Partial Evaluation of Computation Process, Systems Computers Controls 2(5):45-50"
);

/// Futamura (1971) §2: a program's arguments are partitioned into static
/// and dynamic - there is no third temporality.
pub struct StaticDynamicPartitionsInputs;

impl Axiom for StaticDynamicPartitionsInputs {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let q = TemporalityTag;
        let ok = q.get(&StagingConcept::StaticInput) == Some(Temporality::Static)
            && q.get(&StagingConcept::DynamicInput) == Some(Temporality::Dynamic);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StaticDynamicPartitionsInputs",
        "StaticInput is Static; DynamicInput is Dynamic - no third class",
        "Futamura (1971) Partial Evaluation of Computation Process §2"
    );
}

pr4xis::register_axiom!(
    StaticDynamicPartitionsInputs,
    "Futamura (1971) Partial Evaluation of Computation Process §2"
);

impl Ontology for StagingOntology {
    type Cat = StagingCategory;
    type Qual = TemporalityTag;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(FutamuraChainIsComplete));
        axioms.push(Box::new(EachProjectionRaisesStagingByOne));
        axioms.push(Box::new(StaticDynamicPartitionsInputs));
        axioms
    }
}
