# Parallelism ontology — bibliography

## Primary sources

- **Flynn, M. J. (1966).** *"Very high-speed computing systems"*. Proc. IEEE 54(12), 1901–1909. DOI: [10.1109/PROC.1966.5273](https://doi.org/10.1109/PROC.1966.5273). The fourfold SISD/SIMD/MISD/MIMD taxonomy first appears here. Grounds `MachineOrganization`, `SISD`, `SIMD`, `MISD`, `MIMD`, `ProcessingElement`, the `MachineClass` quality, and the `FlynnBijection` axiom.
- **Flynn, M. J. (1972).** *"Some Computer Organizations and Their Effectiveness"*. IEEE Trans. Computers C-21(9), 948–960. DOI: [10.1109/TC.1972.5009071](https://doi.org/10.1109/TC.1972.5009071). Elaboration of the taxonomy.
- **Amdahl, G. M. (1967).** *"Validity of the single processor approach to achieving large scale computing capabilities"*. AFIPS Conf. Proc. 30, 483–485. DOI: [10.1145/1465482.1465560](https://doi.org/10.1145/1465482.1465560). Grounds `SequentialFraction`, `Speedup`, the `Bounds` edge, and the `AmdahlBound` axiom. **Note:** the closed-form `S(p,f)=1/(f+(1−f)/p)` is *not* in Amdahl's paper (a prose argument); its algebraic form is commonly credited to Ware (1972).
- **Ware, W. H. (1972).** *"The ultimate computer"*. IEEE Spectrum 9(3), 84–91. DOI: [10.1109/MSPEC.1972.5218983](https://doi.org/10.1109/MSPEC.1972.5218983). The usual source for the algebraic form of Amdahl's law.
- **Gustafson, J. L. (1988).** *"Reevaluating Amdahl's Law"*. CACM 31(5), 532–533. DOI: [10.1145/42411.42415](https://doi.org/10.1145/42411.42415). Grounds `ScaledSpeedup` and the `GustafsonScaledSpeedup` axiom (fixed-time scaling, serial fraction measured on the parallel system).
- **Brent, R. P. (1974).** *"The parallel evaluation of general arithmetic expressions"*. JACM 21(2), 201–206. DOI: [10.1145/321812.321815](https://doi.org/10.1145/321812.321815). Grounds `ParallelExecution`, the greedy-scheduler bound (Lemma 2), and the `GreedySchedulerBound` axiom.
- **Graham, R. L. (1966).** *"Bounds for Certain Multiprocessing Anomalies"*. Bell System Technical Journal 45(9), 1563–1581. DOI: [10.1002/j.1538-7305.1966.tb01709.x](https://doi.org/10.1002/j.1538-7305.1966.tb01709.x). Grounds `GreedyScheduler` and the greedy bound.
- **Graham, R. L. (1969).** *"Bounds on Multiprocessing Timing Anomalies"*. SIAM J. Appl. Math. 17(2), 416–429. DOI: [10.1137/0117039](https://doi.org/10.1137/0117039). Grounds `GreedyScheduler` and the greedy bound.
- **Blelloch, G. E. (1996).** *"Programming Parallel Algorithms"*. CACM 39(3), 85–97. DOI: [10.1145/227234.227246](https://doi.org/10.1145/227234.227246). Grounds `Work`, `Span`, and the `SpeedupBoundedByParallelism` axiom.
- **Cormen, T. H., Leiserson, C. E., Rivest, R. L. & Stein, C. (2009).** *Introduction to Algorithms* (3rd ed.), Ch. 27 (Multithreaded Algorithms). MIT Press. Grounds `ParallelTask`, `TaskParallelism`, `Work`, `Span`, the `CostCarrier` quality (unit-time convention), and the engine's `P-FIB(4)` fixture (`WORK_FIB4 = 17`, `SPAN_FIB4 = 8`).
- **Valiant, L. G. (1990).** *"A Bridging Model for Parallel Computation"*. CACM 33(8), 103–111. DOI: [10.1145/79173.79181](https://doi.org/10.1145/79173.79181). Grounds `CostModel`, `BSP`, and the `Models` edge.
- **Fortune, S. & Wyllie, J. (1978).** *"Parallelism in Random Access Machines"*. STOC '78, 114–118. DOI: [10.1145/800133.804339](https://doi.org/10.1145/800133.804339). Grounds `PRAM`.
- **Culler, D., Karp, R., Patterson, D., Sahay, A., Schauser, K. E., Santos, E., Subramonian, R. & von Eicken, T. (1993).** *"LogP: Towards a Realistic Model of Parallel Computation"*. PPoPP '93, 1–12. DOI: [10.1145/155332.155333](https://doi.org/10.1145/155332.155333). Grounds `LogP`.
- **Hillis, W. D. & Steele, G. L. (1986).** *"Data Parallel Algorithms"*. CACM 29(12), 1170–1183. DOI: [10.1145/7902.7903](https://doi.org/10.1145/7902.7903). Grounds `DataParallelism`.
- **Ramamoorthy, C. V. & Li, H. F. (1977).** *"Pipeline Architecture"*. ACM Computing Surveys 9(1), 61–102. DOI: [10.1145/356683.356687](https://doi.org/10.1145/356683.356687). Grounds `PipelineParallelism` and the `PipelineSpeedupLaw` axiom.
- **Karp, A. H. & Flatt, H. P. (1990).** *"Measuring Parallel Processor Performance"*. CACM 33(5), 539–543. DOI: [10.1145/78607.78614](https://doi.org/10.1145/78607.78614). Grounds `Efficiency` (the experimentally-determined serial fraction `e = (1/S − 1/p)/(1 − 1/p)`).
- **Blumofe, R. D. & Leiserson, C. E. (1999).** *"Scheduling Multithreaded Computations by Work Stealing"*. JACM 46(5), 720–748. DOI: [10.1145/324133.324234](https://doi.org/10.1145/324133.324234). The modern statement of the greedy/work-stealing bound.
- **Bocchino, R. L., Adve, V. S., Adve, S. V. & Snir, M. (2009).** *"Parallel Programming Must Be Deterministic by Default"*. HotPar '09. Grounds `DeterministicParallelism`, the `Exhibits` edge, and the `DeterministicParallelismIsSequentialSemantics` axiom.
- **Lee, E. A. (2006).** *"The Problem with Threads"*. IEEE Computer 39(5), 33–42. DOI: [10.1109/MC.2006.180](https://doi.org/10.1109/MC.2006.180). Grounds the `IsDeterministicByDefault` classification of task parallelism (nondeterministic interaction) and the `ParallelExecutionRequiresMultiplicity` axiom.
- **Marlow, S. (2012).** *"Parallel and Concurrent Programming in Haskell"*. In *Central European Functional Programming School* (CEFP 2011), LNCS 7241, 339–401, §1.2. DOI: [10.1007/978-3-642-32096-5_7](https://doi.org/10.1007/978-3-642-32096-5_7). Grounds the parallel/concurrent operational distinction (`ParallelExecutionRequiresMultiplicity`) and the two functors' citation.

## Adjunction + gap (true-concurrency semantics)

- **Mazurkiewicz, A. (1977).** *Concurrent Program Schemes and Their Interpretations*. DAIMI Report PB-78, Aarhus University. Trace theory: the quotient of interleavings by an independence relation — exactly the structure parallelism forgets. Grounds the `InterleavingCollapseGap`.
- **Winskel, G. (1986).** *"Event Structures"*. In *Petri Nets: Applications and Relationships to Other Models of Concurrency*, LNCS 255, 325–392. DOI: [10.1007/3-540-17906-2_31](https://doi.org/10.1007/3-540-17906-2_31). True-concurrency semantics where interleaving and partial-order composition differ.
- **Pratt, V. (1986).** *"Modeling Concurrency with Partial Orders"*. Int. J. Parallel Programming 15(1), 33–71. DOI: [10.1007/BF01379149](https://doi.org/10.1007/BF01379149). Partial-order (pomset) semantics; grounds the collapse gap.
- **Mac Lane, S. (1971).** *Categories for the Working Mathematician*, Ch. IV. Springer. The adjunction (unit/counit) formalism.

## Machine-organization reference

- **Hennessy, J. L. & Patterson, D. A.** *Computer Architecture: A Quantitative Approach*. Morgan Kaufmann. The honest note that no commercial MISD machine was built, and the pipelining speedup discussion.
- **Kung, H. T. (1982).** *"Why Systolic Architectures?"*. IEEE Computer 15(1), 37–46. DOI: [10.1109/MC.1982.1653825](https://doi.org/10.1109/MC.1982.1653825). The contested systolic-array reading of MISD.

## Numeric-method reference

- **Goldberg, D. (1991).** *"What Every Computer Scientist Should Know About Floating-Point Arithmetic"*. ACM Computing Surveys 23(1), 5–48. DOI: [10.1145/103162.103163](https://doi.org/10.1145/103162.103163). Grounds the `NUMERIC_TOLERANCE` round-off slack used in the algebraic axioms.

## Honest tier — non-peer-reviewed (used for framing, not as axiom grounds)

- **Pike, R. (2012).** *"Concurrency Is Not Parallelism"*. Waza talk, [go.dev/talks/2012/waza.slide](https://go.dev/talks/2012/waza.slide). A widely-cited talk, not peer-reviewed.
- **Harper, R. (2011).** *"Parallelism is not concurrency"*. Existential Type blog. A blog post, not peer-reviewed.

## Related workspace ontologies

- `formal::systems::concurrency` — the sibling ontology; the reverse functor's source and the adjunction's other half.
- `formal::systems` — the `Parallelism → System` functor target (`SystemCategory`).
- `formal::math::quantity` — the `Quantity`/`Dimension` types behind the `CostCarrier` quality.
- `formal::meta::gap_analysis` — the `Gap`/`GapReport` harness the collapse analysis mirrors.
