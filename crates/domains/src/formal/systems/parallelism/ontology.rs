//! Parallelism — the theory of executing computation simultaneously to
//! reduce completion time, and of measuring how much that helps.
//!
//! Parallelism is *not* concurrency (see the sibling
//! [`super::super::concurrency`] ontology and the adjunction gap in
//! [`super::concurrency_functor`]): concurrency is about *composing*
//! interacting activities and their nondeterministic interleaving;
//! parallelism is about *speeding up* one computation with more
//! hardware, and is deterministic by default (Bocchino et al. 2009).
//!
//! Source traditions:
//!
//! - **Flynn (1966)** *Very high-speed computing systems*, Proc. IEEE
//!   54(12):1901–1909, and **Flynn (1972)** *Some Computer Organizations
//!   and Their Effectiveness*, IEEE Trans. Computers C-21(9):948–960 —
//!   the SISD/SIMD/MISD/MIMD machine taxonomy.
//! - **Amdahl (1967)** *Validity of the single processor approach…*,
//!   AFIPS 30:483–485 — the serial-fraction bound on speedup.
//! - **Gustafson (1988)** *Reevaluating Amdahl's Law*, CACM
//!   31(5):532–533 — fixed-time scaled speedup.
//! - **Brent (1974)** *The parallel evaluation of general arithmetic
//!   expressions*, JACM 21(2):201–206, and **Graham (1966; 1969)** —
//!   greedy scheduling and its bound.
//! - **Blelloch (1996)** *Programming Parallel Algorithms*, CACM
//!   39(3):85–97 — work/span and the parallelism bound.
//! - **Valiant (1990)** *A Bridging Model for Parallel Computation*,
//!   CACM 33(8):103–111 — cost models (with PRAM and LogP).
//!
//! The eight domain axioms are discharged against pure numeric grids and
//! the CLRS `P-FIB(4)` work-span fixture in [`super::engine`].

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::quantity::value::Quantity;

use super::engine::{
    ComputationDag, FIB_INDEX, PROCESSOR_GRID_BASE, SPAN_FIB4, WORK_FIB4, evaluate, evaluate_along,
    fib_dag, fibonacci, greedy_processor_counts, greedy_schedule, span, unit_time_step,
};

pr4xis::ontology! {
    name: "Parallelism",
    source: "Flynn (1966, 1972); Amdahl (1967); Gustafson (1988); Brent (1974); Graham (1966, 1969); Blelloch (1996); Valiant (1990)",

    concepts: [
        // === Execution + hardware (Brent 1974; Flynn 1966; CLRS Ch. 27) ===
        ParallelExecution,
        ProcessingElement,
        ParallelTask,

        // === Flynn's machine taxonomy (Flynn 1966, 1972) ===
        MachineOrganization,
        SISD,
        SIMD,
        MISD,
        MIMD,

        // === Forms of parallelism ===
        DataParallelism,
        TaskParallelism,
        PipelineParallelism,

        // === Cost measures (CLRS Ch. 27; Blelloch 1996; Amdahl; Gustafson) ===
        Work,
        Span,
        Speedup,
        Efficiency,
        SequentialFraction,
        ScaledSpeedup,

        // === Scheduling (Graham 1966, 1969; Brent 1974) ===
        GreedyScheduler,

        // === Cost models (Valiant 1990; Fortune & Wyllie 1978; Culler et al. 1993) ===
        CostModel,
        PRAM,
        BSP,
        LogP,

        // === Determinism (Bocchino et al. 2009) ===
        DeterministicParallelism,
    ],

    labels: {
        ParallelExecution: ("en", "Parallel execution", "Brent (1974) JACM 21(2):201-206: the simultaneous execution of computation to reduce completion time."),
        ProcessingElement: ("en", "Processing element", "Flynn (1966) Proc. IEEE 54(12): a physical executor of instructions - a hardware endurant, not a behaviour."),
        ParallelTask: ("en", "Parallel task", "Cormen, Leiserson, Rivest & Stein (2009) Introduction to Algorithms 3e, Ch. 27: the behavioural unit of parallel decomposition (the strand)."),
        MachineOrganization: ("en", "Machine organization", "Flynn (1966) Proc. IEEE 54(12):1901-1909: the abstract parent of the stream classes (the fourfold taxonomy first appears here); elaborated in Flynn (1972) IEEE Trans. Computers C-21(9):948-960."),
        SISD: ("en", "SISD", "Flynn (1966; 1972): single instruction stream, single data stream - the sequential von Neumann machine."),
        SIMD: ("en", "SIMD", "Flynn (1966; 1972): single instruction stream, multiple data streams - one operation applied across many data lanes."),
        MISD: ("en", "MISD", "Flynn (1966; 1972): multiple instruction streams, single data stream - defined by the 2x2 stream product; no commercial MISD machine was built (Hennessy & Patterson, Computer Architecture: A Quantitative Approach); the systolic-array reading (Kung 1982, IEEE Computer 15(1):37-46) is a contested classification."),
        MIMD: ("en", "MIMD", "Flynn (1966; 1972): multiple instruction streams, multiple data streams - independent processors on independent data (the general multiprocessor)."),
        DataParallelism: ("en", "Data parallelism", "Hillis & Steele (1986) CACM 29(12):1170-1183: the same operation applied across a data collection."),
        TaskParallelism: ("en", "Task parallelism", "Cormen, Leiserson, Rivest & Stein (2009) Introduction to Algorithms 3e, Ch. 27: distinct tasks executing simultaneously."),
        PipelineParallelism: ("en", "Pipeline parallelism", "Ramamoorthy & Li (1977) ACM Computing Surveys 9(1):61-102: the overlapped execution of staged computation."),
        Work: ("en", "Work", "Cormen, Leiserson, Rivest & Stein (2009) Ch. 27; Blelloch (1996) CACM 39(3):85-97: the total operation count T1 - the execution time on one processing element."),
        Span: ("en", "Span", "Cormen, Leiserson, Rivest & Stein (2009) Ch. 27; Blelloch (1996): the critical-path length T-infinity - the longest chain of dependent operations (the depth)."),
        Speedup: ("en", "Speedup", "Amdahl (1967); Cormen, Leiserson, Rivest & Stein (2009) Ch. 27: the ratio S_p = T1 / T_p of serial time to p-processor time."),
        Efficiency: ("en", "Efficiency", "Karp & Flatt (1990) CACM 33(5):539-543: E_p = S_p / p - speedup per processing element; the experimentally-determined serial fraction is e = (1/S - 1/p) / (1 - 1/p)."),
        SequentialFraction: ("en", "Sequential fraction", "Amdahl (1967) AFIPS 30:483-485: the fraction f of a fixed-size computation that is inherently serial."),
        ScaledSpeedup: ("en", "Scaled speedup", "Gustafson (1988) CACM 31(5):532-533: fixed-time scaled speedup - the speedup when problem size grows with the processor count."),
        GreedyScheduler: ("en", "Greedy scheduler", "Graham (1966) Bell System Technical Journal 45(9):1563-1581; Graham (1969) SIAM J. Appl. Math. 17(2):416-429; Brent (1974) Lemma 2: a scheduler that never idles a processing element while a ready task exists."),
        CostModel: ("en", "Cost model", "Valiant (1990) CACM 33(8):103-111: the abstract parent of the machine cost models that predict parallel running time."),
        PRAM: ("en", "PRAM", "Fortune & Wyllie (1978) STOC '78:114-118: the parallel random-access machine - shared memory, unit-cost synchronous steps."),
        BSP: ("en", "BSP", "Valiant (1990) CACM 33(8):103-111: the bulk-synchronous parallel bridging model - supersteps of computation, communication, and barrier."),
        LogP: ("en", "LogP", "Culler, Karp, Patterson, Sahay, Schauser, Santos, Subramonian & von Eicken (1993) PPoPP '93:1-12: a cost model parameterised by latency, overhead, gap, and processor count."),
        DeterministicParallelism: ("en", "Deterministic parallelism", "Bocchino, Adve, Adve & Snir (2009) HotPar '09: parallelism whose observable behaviour equals its sequential elaboration."),
    },

    is_a: [
        // Flynn's four stream classes specialise the machine organization.
        (SISD, MachineOrganization),
        (SIMD, MachineOrganization),
        (MISD, MachineOrganization),
        (MIMD, MachineOrganization),
        // The three forms are kinds of parallel execution.
        (DataParallelism, ParallelExecution),
        (TaskParallelism, ParallelExecution),
        (PipelineParallelism, ParallelExecution),
        // The three cost models specialise the abstract cost model.
        (PRAM, CostModel),
        (BSP, CostModel),
        (LogP, CostModel),
        // Gustafson's scaled speedup is a speedup notion.
        (ScaledSpeedup, Speedup),
    ],

    edges: [
        // Amdahl (1967): the serial fraction bounds the achievable speedup.
        (SequentialFraction, Speedup, Bounds),
        // CLRS Ch. 27 / Blelloch (1996): S_p <= T1 / T-infinity, the span law.
        (Span, Speedup, Bounds),
        // Graham (1966; 1969); Brent (1974): a greedy scheduler achieves
        // T_p <= T1/p + T-infinity.
        (GreedyScheduler, Span, Achieves),
        // CLRS Ch. 27: a task runs on a processing element.
        (ParallelTask, ProcessingElement, ExecutesOn),
        // Bocchino et al. (2009): data parallelism is deterministic by default.
        (DataParallelism, DeterministicParallelism, Exhibits),
        // Valiant (1990): a cost model predicts parallel execution.
        (CostModel, ParallelExecution, Models),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The multiplicity of a Flynn stream — Flynn (1966; 1972): each of the
/// two axes (instruction, data) is either single or multiple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMultiplicity {
    /// One stream.
    Single,
    /// More than one stream.
    Multiple,
}

impl StreamMultiplicity {
    /// The closed two-element set of multiplicities.
    pub const ALL: [StreamMultiplicity; 2] =
        [StreamMultiplicity::Single, StreamMultiplicity::Multiple];
}

/// A Flynn class — Flynn (1966; 1972): the generative `2×2` product of an
/// instruction-stream multiplicity and a data-stream multiplicity that
/// yields exactly SISD/SIMD/MISD/MIMD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlynnClass {
    /// The instruction-stream multiplicity.
    pub instruction: StreamMultiplicity,
    /// The data-stream multiplicity.
    pub data: StreamMultiplicity,
}

impl FlynnClass {
    /// The full `2×2` product of Flynn's two axes — the closed set of
    /// four classes the taxonomy generates (Flynn 1966; 1972).
    pub fn product() -> Vec<FlynnClass> {
        let mut all = Vec::new();
        for instruction in StreamMultiplicity::ALL {
            for data in StreamMultiplicity::ALL {
                all.push(FlynnClass { instruction, data });
            }
        }
        all
    }
}

/// Which Flynn class a machine organization realises — Flynn (1966;
/// 1972). `Some` for exactly the four stream classes (each a distinct
/// point of Flynn's own generative `2×2` product); `None` for every other
/// concept, including the abstract `MachineOrganization` parent.
#[derive(Debug, Clone)]
pub struct MachineClass;

impl Quality for MachineClass {
    type Individual = ParallelismConcept;
    type Value = FlynnClass;

    fn get(&self, c: &ParallelismConcept) -> Option<FlynnClass> {
        use ParallelismConcept as C;
        use StreamMultiplicity::{Multiple, Single};
        let class = |instruction, data| FlynnClass { instruction, data };
        match c {
            C::SISD => Some(class(Single, Single)),
            C::SIMD => Some(class(Single, Multiple)),
            C::MISD => Some(class(Multiple, Single)),
            C::MIMD => Some(class(Multiple, Multiple)),
            _ => None,
        }
    }
}

/// The unit-time-step cost quantum a cost measure accumulates — Cormen,
/// Leiserson, Rivest & Stein (2009) Ch. 27: `Work` (T1) and `Span`
/// (T-infinity) are *dimensionless* counts of unit-time strands, not
/// physical durations. `Some(1.0)` (the dimensionless unit step) for
/// exactly `Work` and `Span`; `None` for concepts that are not counted
/// cost measures.
#[derive(Debug, Clone)]
pub struct CostCarrier;

impl Quality for CostCarrier {
    type Individual = ParallelismConcept;
    type Value = Quantity;

    fn get(&self, c: &ParallelismConcept) -> Option<Quantity> {
        use ParallelismConcept as C;
        match c {
            C::Work | C::Span => Some(unit_time_step()),
            _ => None,
        }
    }
}

/// Whether a form of parallelism is deterministic by default — Bocchino,
/// Adve, Adve & Snir (2009): data parallelism is deterministic by
/// default (`true`); task parallelism admits nondeterministic task
/// interaction (`false`, Lee 2006 IEEE Computer 39(5):33-42). `None` for
/// concepts that are not forms of parallelism.
#[derive(Debug, Clone)]
pub struct IsDeterministicByDefault;

impl Quality for IsDeterministicByDefault {
    type Individual = ParallelismConcept;
    type Value = bool;

    fn get(&self, c: &ParallelismConcept) -> Option<bool> {
        use ParallelismConcept as C;
        match c {
            C::DataParallelism => Some(true),
            C::TaskParallelism => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Numeric parameters (all named + cited — no magic numbers)
// ---------------------------------------------------------------------------

/// Serial fractions `f` used to probe Amdahl's bound. Amdahl (1967) AFIPS
/// 30:483-485 estimates the data-management "housekeeping" fraction — the
/// part of a production workload that resists speedup — at ~40% of executed
/// instructions, reducible by a factor of two (~20%) in a dedicated
/// environment but, in his words, "highly improbable" to reduce by a factor
/// of three (~13%): a cited serial band of roughly 13-40%. The grid keeps two
/// points inside that band (0.10 near the improbable ~13% floor, 0.25
/// interior) and adds 0.05 and 0.50 as extremes bracketing it below and
/// above, so the monotonicity/boundedness of `S(p,f)=1/(f+(1-f)/p)` is
/// exercised across and beyond the cited regime.
pub const AMDAHL_SERIAL_FRACTIONS: [f64; 4] = [0.05, 0.10, 0.25, 0.50];

/// Serial time fractions `s'` measured *on the parallel system*, used to
/// probe Gustafson's scaled speedup — Gustafson (1988) CACM 31(5):532-533
/// (distinct in meaning from Amdahl's fixed-size fraction).
pub const GUSTAFSON_SERIAL_FRACTIONS: [f64; 4] = [0.05, 0.10, 0.25, 0.50];

/// Number of [`PROCESSOR_GRID_BASE`] doublings in the processor-count grid:
/// probes `p ∈ {1, 2, …, 1024}` — Amdahl (1967) and Gustafson (1988) both
/// plot speedup against exponentially growing processor counts.
pub const PROCESSOR_GRID_DOUBLINGS: u32 = 10;

/// Pipeline stage counts `k` to probe the pipeline speedup law over. The
/// speedup law itself is Ramamoorthy & Li (1977) ACM Computing Surveys
/// 9(1):61-102. For depth, Hennessy & Patterson (*Computer Architecture: A
/// Quantitative Approach*) take the 5-stage MIPS datapath (IF/ID/EX/MEM/WB)
/// as the classic RISC pipeline, and the MIPS R4000's 8-stage integer
/// pipeline as the canonical *superpipeline*. The grid is the doubling
/// sequence {2, 4, 8}: its top rung reaches the cited 8-stage R4000
/// superpipeline depth, and it brackets the 5-stage classic RISC pipeline
/// between its 4 and 8 rungs.
pub const PIPELINE_STAGE_COUNTS: [u64; 3] = [2, 4, 8];

/// Float comparison tolerance — the round-off slack for the algebraic
/// identities below; Goldberg (1991) *What Every Computer Scientist
/// Should Know About Floating-Point Arithmetic*, ACM Computing Surveys
/// 23(1):5-48.
pub const NUMERIC_TOLERANCE: f64 = 1e-9;

/// The minimum processing-element count for *parallel* execution:
/// simultaneity requires at least two elements executing at once —
/// Marlow (2012) LNCS 7241 §1.2; Lee (2006) IEEE Computer 39(5):33-42.
pub const MIN_PARALLEL_DEGREE: usize = 2;

/// The serial processing-element count — one element, on which
/// *concurrent* execution is a total interleaving (Marlow 2012 §1.2).
pub const SERIAL_PROCESSOR_COUNT: usize = 1;

/// The processor-count grid:
/// `1, PROCESSOR_GRID_BASE, …, PROCESSOR_GRID_BASE^PROCESSOR_GRID_DOUBLINGS`
/// — structurally derived from the shared cited grid base, not hand-listed.
fn processor_grid() -> Vec<u64> {
    (0..=PROCESSOR_GRID_DOUBLINGS)
        .map(|k| (PROCESSOR_GRID_BASE as u64).pow(k))
        .collect()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn direct_children_of(parent: ParallelismConcept) -> Vec<ParallelismConcept> {
    use pr4xis::category::{Arrow, Category};
    ParallelismCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == ParallelismRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(
    from: ParallelismConcept,
    to: ParallelismConcept,
    kind: ParallelismRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    ParallelismCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

/// Amdahl's speedup `S(p, f) = 1 / (f + (1−f)/p)` — the algebraic form
/// (see [`AmdahlBound`] for the provenance of the formula itself).
fn amdahl_speedup(p: u64, f: f64) -> f64 {
    1.0 / (f + (1.0 - f) / p as f64)
}

/// Gustafson's scaled speedup in the paper's verbatim form
/// `s' + p'·N = N + (1−N)·s'`, with `s' + p' = 1` — Gustafson (1988).
fn gustafson_scaled_speedup(n: u64, serial: f64) -> f64 {
    let parallel = 1.0 - serial;
    serial + parallel * n as f64
}

/// Pipeline speedup `S(n, k) = n·k / (k + n − 1)` for `n` tasks through a
/// `k`-stage pipeline — Ramamoorthy & Li (1977).
fn pipeline_speedup(n: u64, k: u64) -> f64 {
    (n * k) as f64 / (k + n - 1) as f64
}

/// The `P-FIB(FIB_INDEX)` fixture DAG (CLRS Ch. 27), shared by the
/// engine-discharged axioms.
fn fixture() -> ComputationDag {
    fib_dag(FIB_INDEX)
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Amdahl (1967): the speedup `S(p, f) = 1 / (f + (1−f)/p)` of a
/// fixed-size computation with serial fraction `f` is monotone
/// nondecreasing in the processor count `p` and bounded above by `1/f`.
///
/// The closed-form formula does **not** appear in Amdahl's 1967 paper,
/// which gives only a prose argument; the algebraic formalization is
/// commonly credited to Ware (1972). The category carries the matching
/// `Bounds` edge from `SequentialFraction` to `Speedup`.
pub struct AmdahlBound;

impl Axiom for AmdahlBound {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let grid = processor_grid();
        let mut monotone = true;
        let mut bounded = true;
        for &f in &AMDAHL_SERIAL_FRACTIONS {
            let limit = 1.0 / f;
            let mut prev = f64::NEG_INFINITY;
            for &p in &grid {
                let s = amdahl_speedup(p, f);
                if s < prev - NUMERIC_TOLERANCE {
                    monotone = false;
                }
                if s > limit + NUMERIC_TOLERANCE {
                    bounded = false;
                }
                prev = s;
            }
        }
        let edge = kinded_edge_exists(
            ParallelismConcept::SequentialFraction,
            ParallelismConcept::Speedup,
            ParallelismRelationKind::Bounds,
        );
        if monotone && bounded && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AmdahlBound",
        "S(p,f)=1/(f+(1-f)/p) is monotone nondecreasing in p and bounded above by 1/f over the processor grid",
        "Amdahl (1967) AFIPS Conf. Proc. 30, 483-485 (prose argument); algebraic formalization later, commonly credited to Ware (1972) IEEE Spectrum 9(3):84-91"
    );
}
pr4xis::register_axiom!(
    AmdahlBound,
    "Amdahl (1967) AFIPS Conf. Proc. 30, 483-485 (prose argument); algebraic formalization later, commonly credited to Ware (1972) IEEE Spectrum 9(3):84-91"
);

/// Gustafson (1988): fixed-time *scaled* speedup, in the paper's verbatim
/// form `s' + p'·N = N + (1−N)·s'` where `s' + p' = 1` are the serial and
/// parallel time fractions measured **on the parallel system** (not
/// Amdahl's fixed-size serial fraction). Verified linear in `N`, and
/// algebraically equal to the `N − (N−1)·s'` form.
pub struct GustafsonScaledSpeedup;

impl Axiom for GustafsonScaledSpeedup {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let grid = processor_grid();
        let mut identity_holds = true;
        let mut linear = true;
        for &s_serial in &GUSTAFSON_SERIAL_FRACTIONS {
            let slope = 1.0 - s_serial; // p' = 1 − s'
            let mut prev: Option<(u64, f64)> = None;
            for &n in &grid {
                let scaled = gustafson_scaled_speedup(n, s_serial);
                // Algebraic identity: s' + p'·N == N + (1−N)·s' == N − (N−1)·s'.
                let form_b = n as f64 + (1.0 - n as f64) * s_serial;
                let form_c = n as f64 - (n as f64 - 1.0) * s_serial;
                if (scaled - form_b).abs() > NUMERIC_TOLERANCE
                    || (scaled - form_c).abs() > NUMERIC_TOLERANCE
                {
                    identity_holds = false;
                }
                // Linearity: successive differences equal the constant slope p'
                // per unit of N.
                if let Some((prev_n, prev_scaled)) = prev {
                    let expected = slope * (n as f64 - prev_n as f64);
                    if (scaled - prev_scaled - expected).abs() > NUMERIC_TOLERANCE {
                        linear = false;
                    }
                }
                prev = Some((n, scaled));
            }
        }
        if identity_holds && linear {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GustafsonScaledSpeedup",
        "scaled speedup s'+p'*N = N+(1-N)s' = N-(N-1)s' is linear in N with slope p' (fixed-time scaling on the parallel system)",
        "Gustafson (1988) CACM 31(5):532-533"
    );
}
pr4xis::register_axiom!(
    GustafsonScaledSpeedup,
    "Gustafson (1988) CACM 31(5):532-533"
);

/// Graham (1966; 1969) / Brent (1974) greedy bound: on the `P-FIB(4)`
/// fixture the greedy scheduler's makespan satisfies
/// `max(⌈T1/p⌉, T∞) ≤ T_p ≤ ⌊T1/p⌋ + T∞` for every probed processor
/// count `p`. Discharged via the engine's greedy scheduler. The category
/// carries the matching `Achieves` edge from `GreedyScheduler` to `Span`.
pub struct GreedySchedulerBound;

impl Axiom for GreedySchedulerBound {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let dag = fixture();
        let t1 = dag.work();
        let t_inf = span(&dag);
        let mut ok = t1 == WORK_FIB4 && t_inf == SPAN_FIB4;
        for p in greedy_processor_counts(t_inf) {
            let t_p = greedy_schedule(&dag, p).makespan();
            // Lower bound: T_p ≥ T∞ and p·T_p ≥ T1 (i.e. T_p ≥ ⌈T1/p⌉).
            let lower = t_p >= t_inf && p * t_p >= t1;
            // Upper bound: T_p ≤ ⌊T1/p⌋ + T∞ (T∞ integer, so this is the
            // integer floor of the Brent bound T1/p + T∞).
            let upper = t_p <= t1 / p + t_inf;
            ok = ok && lower && upper;
        }
        let edge = kinded_edge_exists(
            ParallelismConcept::GreedyScheduler,
            ParallelismConcept::Span,
            ParallelismRelationKind::Achieves,
        );
        if ok && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GreedySchedulerBound",
        "on the P-FIB(4) fixture max(ceil(T1/p),Tinf) <= T_p <= floor(T1/p)+Tinf for every probed p",
        "Graham (1966) BSTJ 45(9); Graham (1969) SIAM J. Appl. Math. 17(2); Brent (1974) JACM 21(2) Lemma 2; modern statement Blumofe & Leiserson (1999) JACM 46(5):720-748"
    );
}
pr4xis::register_axiom!(
    GreedySchedulerBound,
    "Graham (1966) BSTJ 45(9); Graham (1969) SIAM J. Appl. Math. 17(2); Brent (1974) JACM 21(2) Lemma 2; modern statement Blumofe & Leiserson (1999) JACM 46(5):720-748"
);

/// Blelloch (1996) / CLRS Ch. 27 parallelism bound: `S_p ≤ T1/T∞` for
/// every probed `p` on the fixture — no scheduler can beat the
/// parallelism `T1/T∞`. Equivalent to `T_p ≥ T∞`; discharged via the
/// engine. The category carries the matching `Bounds` edge from `Span`
/// to `Speedup`.
pub struct SpeedupBoundedByParallelism;

impl Axiom for SpeedupBoundedByParallelism {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let dag = fixture();
        let t1 = dag.work();
        let t_inf = span(&dag);
        let parallelism = t1 as f64 / t_inf as f64;
        let mut ok = t_inf > 0;
        for p in greedy_processor_counts(t_inf) {
            let t_p = greedy_schedule(&dag, p).makespan();
            let speedup = t1 as f64 / t_p as f64;
            if speedup > parallelism + NUMERIC_TOLERANCE {
                ok = false;
            }
        }
        let edge = kinded_edge_exists(
            ParallelismConcept::Span,
            ParallelismConcept::Speedup,
            ParallelismRelationKind::Bounds,
        );
        if ok && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SpeedupBoundedByParallelism",
        "on the P-FIB(4) fixture S_p = T1/T_p <= T1/Tinf for every probed p",
        "Blelloch (1996) CACM 39(3):85-97; Cormen, Leiserson, Rivest & Stein (2009) Introduction to Algorithms 3e, Ch. 27"
    );
}
pr4xis::register_axiom!(
    SpeedupBoundedByParallelism,
    "Blelloch (1996) CACM 39(3):85-97; Cormen, Leiserson, Rivest & Stein (2009) Introduction to Algorithms 3e, Ch. 27"
);

/// Flynn (1966; 1972): the Subsumption-children of `MachineOrganization`
/// are in bijection with the `2×2` product `StreamMultiplicity ×
/// StreamMultiplicity` via the `MachineClass` quality — every child has
/// `Some` and a distinct Flynn class, and every point of the product is
/// realised by exactly one child. This is a *bijection*, not a count.
pub struct FlynnBijection;

impl Axiom for FlynnBijection {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let children = direct_children_of(ParallelismConcept::MachineOrganization);
        let product = FlynnClass::product();
        let q = MachineClass;
        // Every child has Some(class); collect them.
        let classes: Option<Vec<FlynnClass>> = children.iter().map(|c| q.get(c)).collect();
        let Some(classes) = classes else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        // Injective: no two children share a class.
        let injective = classes
            .iter()
            .enumerate()
            .all(|(i, a)| classes.iter().skip(i + 1).all(|b| a != b));
        // Surjective onto the 2x2 product: every product point is realised.
        let surjective = product.iter().all(|point| classes.contains(point));
        // Sizes match the product (four classes).
        let sized = children.len() == product.len() && classes.len() == product.len();
        if injective && surjective && sized {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "FlynnBijection",
        "the Subsumption-children of MachineOrganization are in bijection with StreamMultiplicity x StreamMultiplicity via MachineClass",
        "Flynn (1966) Proc. IEEE 54(12):1901-1909; Flynn (1972) IEEE Trans. Computers C-21(9):948-960"
    );
}
pr4xis::register_axiom!(
    FlynnBijection,
    "Flynn (1966) Proc. IEEE 54(12):1901-1909; Flynn (1972) IEEE Trans. Computers C-21(9):948-960"
);

/// Ramamoorthy & Li (1977): the pipeline speedup `S(n, k) = n·k /
/// (k + n − 1)` of `n` tasks through a `k`-stage pipeline is monotone
/// nondecreasing in `n` and bounded above by the stage count `k`
/// (the ideal fill/drain-free limit).
pub struct PipelineSpeedupLaw;

impl Axiom for PipelineSpeedupLaw {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let task_grid = processor_grid(); // reuse the exponential n-grid
        let mut monotone = true;
        let mut bounded = true;
        for &k in &PIPELINE_STAGE_COUNTS {
            let mut prev = f64::NEG_INFINITY;
            for &n in &task_grid {
                let s = pipeline_speedup(n, k);
                if s < prev - NUMERIC_TOLERANCE {
                    monotone = false;
                }
                if s > k as f64 + NUMERIC_TOLERANCE {
                    bounded = false;
                }
                prev = s;
            }
        }
        if monotone && bounded {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PipelineSpeedupLaw",
        "S(n,k)=n*k/(k+n-1) is monotone nondecreasing in n and bounded above by the stage count k",
        "Ramamoorthy & Li (1977) ACM Computing Surveys 9(1):61-102; Hennessy & Patterson, Computer Architecture: A Quantitative Approach (pipelining)"
    );
}
pr4xis::register_axiom!(
    PipelineSpeedupLaw,
    "Ramamoorthy & Li (1977) ACM Computing Surveys 9(1):61-102; Hennessy & Patterson, Computer Architecture: A Quantitative Approach (pipelining)"
);

/// Bocchino et al. (2009) / Blelloch (1996): on the `P-FIB(4)` fixture,
/// every greedy schedule (over every probed `p`) yields the same
/// computation result as the sequential elaboration `fib(4) = 3` — the
/// engine performs real additions, so the equality has bite.
///
/// Deterministic **by default**, not universally: some parallel
/// algorithms (branch-and-bound, some search) are intrinsically
/// nondeterministic. The category carries the matching `Exhibits` edge
/// from `DataParallelism` to `DeterministicParallelism`.
pub struct DeterministicParallelismIsSequentialSemantics;

impl Axiom for DeterministicParallelismIsSequentialSemantics {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let dag = fixture();
        let sequential = fibonacci(FIB_INDEX);
        let intrinsic = evaluate(&dag);
        let mut ok = intrinsic == sequential;
        for p in greedy_processor_counts(span(&dag)) {
            let schedule = greedy_schedule(&dag, p);
            let along = evaluate_along(&dag, &schedule.flatten());
            if along != sequential {
                ok = false;
            }
        }
        let edge = kinded_edge_exists(
            ParallelismConcept::DataParallelism,
            ParallelismConcept::DeterministicParallelism,
            ParallelismRelationKind::Exhibits,
        );
        if ok && edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeterministicParallelismIsSequentialSemantics",
        "every greedy schedule of the P-FIB(4) fixture yields the sequential result fib(4)=3, for every probed p",
        "Bocchino, Adve, Adve & Snir (2009) HotPar '09; Blelloch (1996) CACM 39(3):85-97"
    );
}
pr4xis::register_axiom!(
    DeterministicParallelismIsSequentialSemantics,
    "Bocchino, Adve, Adve & Snir (2009) HotPar '09; Blelloch (1996) CACM 39(3):85-97"
);

/// Marlow (2012) / Lee (2006): the operational distinction between
/// parallel and concurrent execution, on the `P-FIB(4)` fixture:
///
/// - *parallel* execution uses ≥ 2 processing elements simultaneously —
///   with `p ≥ 2` some greedy step dispatches two strands at once;
/// - *concurrent* execution admits interleaving on one element — the
///   `p = 1` greedy schedule is a valid total interleaving (one strand
///   per step, all `T1` strands, computing the same result).
pub struct ParallelExecutionRequiresMultiplicity;

impl Axiom for ParallelExecutionRequiresMultiplicity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let dag = fixture();
        let sequential = fibonacci(FIB_INDEX);
        // Parallel: with p ≥ 2 some step runs two strands at once.
        let parallel =
            greedy_schedule(&dag, MIN_PARALLEL_DEGREE).max_parallelism() >= MIN_PARALLEL_DEGREE;
        // Concurrent: the p = 1 schedule is a single-element total
        // interleaving over all T1 strands, computing the same result.
        let serial = greedy_schedule(&dag, SERIAL_PROCESSOR_COUNT);
        let one_per_step = serial.steps.iter().all(|s| s.tasks.len() == 1);
        let covers_all = serial.makespan() == dag.work();
        let same_result = evaluate_along(&dag, &serial.flatten()) == sequential;
        if parallel && one_per_step && covers_all && same_result {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ParallelExecutionRequiresMultiplicity",
        "on the fixture p>=2 yields a step with two strands (parallel), while the p=1 schedule is a valid one-strand-per-step total interleaving (concurrent) of the same computation",
        "Marlow (2012) Parallel and Concurrent Programming in Haskell, CEFP 2011, LNCS 7241, 339-401, §1.2; Lee (2006) The Problem with Threads, IEEE Computer 39(5):33-42"
    );
}
pr4xis::register_axiom!(
    ParallelExecutionRequiresMultiplicity,
    "Marlow (2012) LNCS 7241 §1.2; Lee (2006) IEEE Computer 39(5):33-42"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for ParallelismOntology {
    type Cat = ParallelismCategory;
    type Qual = MachineClass;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AmdahlBound));
        axioms.push(Box::new(GustafsonScaledSpeedup));
        axioms.push(Box::new(GreedySchedulerBound));
        axioms.push(Box::new(SpeedupBoundedByParallelism));
        axioms.push(Box::new(FlynnBijection));
        axioms.push(Box::new(PipelineSpeedupLaw));
        axioms.push(Box::new(DeterministicParallelismIsSequentialSemantics));
        axioms.push(Box::new(ParallelExecutionRequiresMultiplicity));
        axioms
    }
}

/// The four Flynn stream classes — direct Subsumption-children of
/// `MachineOrganization` (Flynn 1966; 1972). Grounded in the category's
/// edges, used by tests.
pub fn flynn_classes() -> Vec<ParallelismConcept> {
    direct_children_of(ParallelismConcept::MachineOrganization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ParallelismCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ParallelismOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn amdahl_bound_holds() {
        assert!(AmdahlBound.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn gustafson_scaled_speedup_holds() {
        assert!(GustafsonScaledSpeedup.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn greedy_scheduler_bound_holds() {
        assert!(GreedySchedulerBound.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn speedup_bounded_by_parallelism_holds() {
        assert!(SpeedupBoundedByParallelism.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn flynn_bijection_holds() {
        assert!(FlynnBijection.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pipeline_speedup_law_holds() {
        assert!(PipelineSpeedupLaw.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn deterministic_parallelism_is_sequential_semantics_holds() {
        assert!(
            DeterministicParallelismIsSequentialSemantics
                .verify()
                .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parallel_execution_requires_multiplicity_holds() {
        assert!(ParallelExecutionRequiresMultiplicity.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fixture_matches_cited_work_and_span() {
        let dag = fixture();
        assert_eq!(dag.work(), WORK_FIB4, "work T1 must equal the cited 17");
        assert_eq!(
            span(&dag),
            SPAN_FIB4,
            "span T-infinity must equal the cited 8"
        );
        assert_eq!(
            evaluate(&dag),
            fibonacci(FIB_INDEX),
            "the DAG must compute fib(4)"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn flynn_taxonomy_is_the_four_classes() {
        let classes = flynn_classes();
        let expected = [
            ParallelismConcept::SISD,
            ParallelismConcept::SIMD,
            ParallelismConcept::MISD,
            ParallelismConcept::MIMD,
        ];
        assert_eq!(classes.len(), expected.len());
        for c in expected {
            assert!(classes.contains(&c), "{c:?} should be a Flynn class");
            assert!(
                MachineClass.get(&c).is_some(),
                "{c:?} must carry a Flynn class"
            );
        }
        assert_eq!(
            MachineClass.get(&ParallelismConcept::MachineOrganization),
            None,
            "the abstract parent carries no Flynn class of its own"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cost_measures_are_dimensionless() {
        for c in [ParallelismConcept::Work, ParallelismConcept::Span] {
            let q = CostCarrier
                .get(&c)
                .unwrap_or_else(|| panic!("{c:?} should carry a cost quantum"));
            assert!(
                q.is_dimensionless(),
                "{c:?} cost quantum must be dimensionless (CLRS Ch. 27)"
            );
        }
        assert_eq!(CostCarrier.get(&ParallelismConcept::Speedup), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn determinism_by_default_classification() {
        assert_eq!(
            IsDeterministicByDefault.get(&ParallelismConcept::DataParallelism),
            Some(true),
            "data parallelism is deterministic by default (Bocchino et al. 2009)"
        );
        assert_eq!(
            IsDeterministicByDefault.get(&ParallelismConcept::TaskParallelism),
            Some(false),
            "task parallelism admits nondeterministic interaction (Lee 2006)"
        );
        assert_eq!(
            IsDeterministicByDefault.get(&ParallelismConcept::PipelineParallelism),
            None
        );
    }
}
