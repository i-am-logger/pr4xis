//! Benchmark ontology — the process of measuring system performance.
//!
//! A benchmark is a structured measurement process with specific phases,
//! validity requirements, and statistical rigor. It produces results
//! that can be compared across runs for regression detection.
//!
//! References:
//! - Georges, Buytaert & Eeckhout, "Statistically Rigorous Java Performance
//!   Evaluation" (2007, OOPSLA) — steady-state detection, multiple invocations
//! - Kalibera & Jones, "Rigorous Benchmarking in Reasonable Time"
//!   (2013, ISSTA) — hierarchical design, variance decomposition
//! - SPEC CPU2017 documentation — run rules, reporting requirements
//! - ISO/IEC 14756:1999 — measurement and rating of computer performance

pr4xis::ontology! {
    name: "Benchmark",
    source: "JCGM 200:2012 (VIM)",
    being: AbstractObject,

    concepts: [
        Benchmark,
        Setup,
        Warmup,
        SteadyState,
        Iteration,
        Invocation,
        Baseline,
        Candidate,
        Regression,
        Improvement,
        EffectSize,
        ConfidenceInterval,
    ],

    labels: {
        Benchmark: ("en", "Benchmark", "The benchmark as a whole — a structured measurement protocol. ISO/IEC 14756: a complete specification of workload + measurement."),
        Setup: ("en", "Setup", "Configure the system under test, initialize workload. ISO/IEC 14756: workload characterization."),
        Warmup: ("en", "Warmup", "The non-stationary period before steady state. Georges et al. (2007): JIT compilation, cache warming."),
        SteadyState: ("en", "Steady state", "The stationary period where measurements are valid. Georges et al. (2007): coefficient of variation test."),
        Iteration: ("en", "Iteration", "A single execution of the benchmark workload. Kalibera & Jones (2013): the atomic unit of measurement."),
        Invocation: ("en", "Invocation", "A complete process start-to-finish (may contain many iterations). Georges et al. (2007): multiple invocations needed to avoid bias from memory layout, JIT decisions, etc."),
        Baseline: ("en", "Baseline", "The reference distribution of measurements from a known-good version."),
        Candidate: ("en", "Candidate", "The distribution from the version under test."),
        Regression: ("en", "Regression", "A statistically significant degradation in performance."),
        Improvement: ("en", "Improvement", "A statistically significant improvement in performance."),
        EffectSize: ("en", "Effect size", "The magnitude of difference between baseline and candidate. Cohen's d or similar effect size measure."),
        ConfidenceInterval: ("en", "Confidence interval", "The range of plausible values for the true performance. Georges et al. (2007): 'a benchmark result without a confidence interval is meaningless.'"),
    },

    edges: [
        (Benchmark, Setup, Contains),
        (Benchmark, Warmup, Contains),
        (Benchmark, SteadyState, Contains),
        (Benchmark, Invocation, Contains),
        (Setup, Warmup, Precedes),
        (Warmup, SteadyState, Precedes),
        (Invocation, Iteration, ContainsIterations),
        (Baseline, EffectSize, ComparesTo),
        (Candidate, EffectSize, ComparesTo),
        (EffectSize, Regression, Determines),
        (EffectSize, Improvement, Determines),
        (Baseline, ConfidenceInterval, Requires),
        (Candidate, ConfidenceInterval, Requires),
    ],
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<BenchmarkCategory>().unwrap();
    }

    #[test]
    fn has_twelve_concepts() {
        assert_eq!(BenchmarkConcept::variants().len(), 12);
    }

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
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Setup
            && r.to == BenchmarkConcept::SteadyState));
    }

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

    #[test]
    fn invocation_contains_iterations() {
        let m = BenchmarkCategory::morphisms();
        assert!(m.iter().any(|r| r.from == BenchmarkConcept::Invocation
            && r.to == BenchmarkConcept::Iteration
            && r.kind == BenchmarkRelationKind::ContainsIterations));
    }

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

    #[test]
    fn baseline_reaches_regression() {
        let m = BenchmarkCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == BenchmarkConcept::Baseline
                    && r.to == BenchmarkConcept::Regression)
        );
    }

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
