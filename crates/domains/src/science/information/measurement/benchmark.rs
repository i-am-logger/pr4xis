use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Benchmark ontology — the process of measuring system performance.
//
// A benchmark is a structured measurement process with specific phases,
// validity requirements, and statistical rigor. It produces results
// that can be compared across runs for regression detection.
//
// References:
// - Georges, Buytaert & Eeckhout, "Statistically Rigorous Java Performance
//   Evaluation" (2007, OOPSLA) — steady-state detection, multiple invocations
// - Kalibera & Jones, "Rigorous Benchmarking in Reasonable Time"
//   (2013, ISSTA) — hierarchical design, variance decomposition
// - SPEC CPU2017 documentation — run rules, reporting requirements
// - ISO/IEC 14756:1999 — measurement and rating of computer performance

/// Concepts in the benchmark process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BenchmarkConcept {
    /// The benchmark as a whole — a structured measurement protocol.
    /// ISO/IEC 14756: a complete specification of workload + measurement.
    Benchmark,

    /// Configure the system under test, initialize workload.
    /// ISO/IEC 14756: workload characterization.
    Setup,

    /// The non-stationary period before steady state.
    /// Georges et al. (2007): JIT compilation, cache warming.
    Warmup,

    /// The stationary period where measurements are valid.
    /// Georges et al. (2007): coefficient of variation test.
    SteadyState,

    /// A single execution of the benchmark workload.
    /// Kalibera & Jones (2013): the atomic unit of measurement.
    Iteration,

    /// A complete process start-to-finish (may contain many iterations).
    /// Georges et al. (2007): multiple invocations needed to avoid
    /// bias from memory layout, JIT decisions, etc.
    Invocation,

    /// The reference distribution of measurements from a known-good version.
    Baseline,

    /// The distribution from the version under test.
    Candidate,

    /// A statistically significant degradation in performance.
    Regression,

    /// A statistically significant improvement in performance.
    Improvement,

    /// The magnitude of difference between baseline and candidate.
    /// Cohen's d or similar effect size measure.
    EffectSize,

    /// The range of plausible values for the true performance.
    /// Georges et al. (2007): "a benchmark result without a
    /// confidence interval is meaningless."
    ConfidenceInterval,
}

impl Entity for BenchmarkConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Benchmark,
            Self::Setup,
            Self::Warmup,
            Self::SteadyState,
            Self::Iteration,
            Self::Invocation,
            Self::Baseline,
            Self::Candidate,
            Self::Regression,
            Self::Improvement,
            Self::EffectSize,
            Self::ConfidenceInterval,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BenchmarkRelation {
    pub from: BenchmarkConcept,
    pub to: BenchmarkConcept,
    pub kind: BenchmarkRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BenchmarkRelationKind {
    Identity,
    /// Benchmark contains this phase/component.
    Contains,
    /// Phase precedes another in the process.
    Precedes,
    /// Invocation contains Iterations.
    ContainsIterations,
    /// Baseline/Candidate produce EffectSize when compared.
    ComparesTo,
    /// EffectSize determines Regression or Improvement.
    Determines,
    /// Result requires ConfidenceInterval (Georges axiom).
    Requires,
    Composed,
}

impl Relationship for BenchmarkRelation {
    type Object = BenchmarkConcept;
    fn source(&self) -> BenchmarkConcept {
        self.from
    }
    fn target(&self) -> BenchmarkConcept {
        self.to
    }
}

pub struct BenchmarkCategory;

impl Category for BenchmarkCategory {
    type Object = BenchmarkConcept;
    type Morphism = BenchmarkRelation;

    fn identity(obj: &BenchmarkConcept) -> BenchmarkRelation {
        BenchmarkRelation {
            from: *obj,
            to: *obj,
            kind: BenchmarkRelationKind::Identity,
        }
    }

    fn compose(f: &BenchmarkRelation, g: &BenchmarkRelation) -> Option<BenchmarkRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == BenchmarkRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == BenchmarkRelationKind::Identity {
            return Some(f.clone());
        }
        Some(BenchmarkRelation {
            from: f.from,
            to: g.to,
            kind: BenchmarkRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<BenchmarkRelation> {
        use BenchmarkConcept as C;
        use BenchmarkRelationKind as R;
        let mut m = Vec::new();

        for c in BenchmarkConcept::variants() {
            m.push(BenchmarkRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // Benchmark contains all phases
        for phase in [C::Setup, C::Warmup, C::SteadyState, C::Invocation] {
            m.push(BenchmarkRelation {
                from: C::Benchmark,
                to: phase,
                kind: R::Contains,
            });
        }

        // The process state machine: Setup → Warmup → SteadyState
        // Georges et al. (2007): you MUST reach steady state before measuring.
        m.push(BenchmarkRelation {
            from: C::Setup,
            to: C::Warmup,
            kind: R::Precedes,
        });
        m.push(BenchmarkRelation {
            from: C::Warmup,
            to: C::SteadyState,
            kind: R::Precedes,
        });

        // Invocation contains Iterations (Kalibera hierarchical design)
        m.push(BenchmarkRelation {
            from: C::Invocation,
            to: C::Iteration,
            kind: R::ContainsIterations,
        });

        // Baseline and Candidate are compared to produce EffectSize
        m.push(BenchmarkRelation {
            from: C::Baseline,
            to: C::EffectSize,
            kind: R::ComparesTo,
        });
        m.push(BenchmarkRelation {
            from: C::Candidate,
            to: C::EffectSize,
            kind: R::ComparesTo,
        });

        // EffectSize determines Regression or Improvement
        m.push(BenchmarkRelation {
            from: C::EffectSize,
            to: C::Regression,
            kind: R::Determines,
        });
        m.push(BenchmarkRelation {
            from: C::EffectSize,
            to: C::Improvement,
            kind: R::Determines,
        });

        // Every result requires ConfidenceInterval (Georges axiom)
        m.push(BenchmarkRelation {
            from: C::Baseline,
            to: C::ConfidenceInterval,
            kind: R::Requires,
        });
        m.push(BenchmarkRelation {
            from: C::Candidate,
            to: C::ConfidenceInterval,
            kind: R::Requires,
        });

        // Composed: Benchmark → SteadyState (through Setup → Warmup)
        m.push(BenchmarkRelation {
            from: C::Benchmark,
            to: C::Iteration,
            kind: R::Composed,
        });
        m.push(BenchmarkRelation {
            from: C::Setup,
            to: C::SteadyState,
            kind: R::Composed,
        });
        m.push(BenchmarkRelation {
            from: C::Baseline,
            to: C::Regression,
            kind: R::Composed,
        });
        m.push(BenchmarkRelation {
            from: C::Candidate,
            to: C::Regression,
            kind: R::Composed,
        });

        for c in BenchmarkConcept::variants() {
            m.push(BenchmarkRelation {
                from: c,
                to: c,
                kind: R::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<BenchmarkCategory>().unwrap();
    }

    #[test]
    fn has_twelve_concepts() {
        assert_eq!(BenchmarkConcept::variants().len(), 12);
    }

    // --- Georges et al. (2007): steady-state MUST be reached before measuring ---

    #[test]
    fn setup_precedes_warmup() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Setup
            && r.to == BenchmarkConcept::Warmup
            && r.kind == BenchmarkRelationKind::Precedes));
    }

    #[test]
    fn warmup_precedes_steady_state() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Warmup
            && r.to == BenchmarkConcept::SteadyState
            && r.kind == BenchmarkRelationKind::Precedes));
    }

    #[test]
    fn steady_state_reachable_from_setup() {
        // Setup → Warmup → SteadyState (transitively)
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Setup
            && r.to == BenchmarkConcept::SteadyState));
    }

    // --- Georges et al. (2007): "a benchmark result without a confidence interval is meaningless" ---

    #[test]
    fn baseline_requires_confidence_interval() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Baseline
            && r.to == BenchmarkConcept::ConfidenceInterval
            && r.kind == BenchmarkRelationKind::Requires));
    }

    #[test]
    fn candidate_requires_confidence_interval() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Candidate
            && r.to == BenchmarkConcept::ConfidenceInterval
            && r.kind == BenchmarkRelationKind::Requires));
    }

    // --- Kalibera & Jones (2013): hierarchical — invocations contain iterations ---

    #[test]
    fn invocation_contains_iterations() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Invocation
            && r.to == BenchmarkConcept::Iteration
            && r.kind == BenchmarkRelationKind::ContainsIterations));
    }

    // --- Regression detection: Baseline × Candidate → EffectSize → Verdict ---

    #[test]
    fn baseline_and_candidate_produce_effect_size() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Baseline
            && r.to == BenchmarkConcept::EffectSize
            && r.kind == BenchmarkRelationKind::ComparesTo));
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Candidate
            && r.to == BenchmarkConcept::EffectSize
            && r.kind == BenchmarkRelationKind::ComparesTo));
    }

    #[test]
    fn effect_size_determines_verdict() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::EffectSize
            && r.to == BenchmarkConcept::Regression
            && r.kind == BenchmarkRelationKind::Determines));
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::EffectSize
            && r.to == BenchmarkConcept::Improvement
            && r.kind == BenchmarkRelationKind::Determines));
    }

    // --- Full pipeline: Baseline → EffectSize → Regression ---

    #[test]
    fn baseline_reaches_regression() {
        let m = BenchmarkCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == BenchmarkConcept::Baseline
                    && r.to == BenchmarkConcept::Regression)
        );
    }

    // --- SPEC: benchmark contains all phases ---

    #[test]
    fn benchmark_contains_phases() {
        let m = BenchmarkCategory::morphisms();
        for phase in [
            BenchmarkConcept::Setup,
            BenchmarkConcept::Warmup,
            BenchmarkConcept::SteadyState,
            BenchmarkConcept::Invocation,
        ] {
            assert!(
                m.iter().any(|r| r.from == BenchmarkConcept::Benchmark
                    && r.to == phase
                    && r.kind == BenchmarkRelationKind::Contains),
                "Benchmark should contain {phase:?}"
            );
        }
    }
}
