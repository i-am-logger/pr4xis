# Dependability ontology — bibliography

## Primary source

**Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004).** *"Basic Concepts and Taxonomy of Dependable and Secure Computing"*. IEEE Transactions on Dependable and Secure Computing, 1(1), 11–33. DOI: [10.1109/TDSC.2004.2](https://doi.org/10.1109/TDSC.2004.2)

The foundational paper of dependable-systems engineering. Provides:
- §2 — basic concepts: system, service, function, behaviour
- §2.2 — the Fault → Error → Failure chain (the central construct)
- §2.4 — failure recursion (Failure at layer N becomes Fault at layer N+1)
- §3 — threats (Faults, Errors, Failures); §3.2 fault classes; §3.3 error classes; §3.4 failure modes
- §4 — attributes: Availability, Reliability, Safety, Confidentiality, Integrity, Maintainability
- §5 — means: Prevention, Tolerance, Removal, Forecasting

Every concept in `ontology.rs` is grounded in a specific section of this paper.

## Cited within concept definitions

**Cristian, F. (1991).** *"Understanding Fault-Tolerant Distributed Systems"*. Communications of the ACM, 34(2), 56–78. DOI: [10.1145/102792.102801](https://doi.org/10.1145/102792.102801)

Defines the operational fault models used in distributed systems:
- Crash failures (system halts; observable to others)
- Omission failures (fails to send/receive)
- Timing failures (output outside specified interval)
- Byzantine / arbitrary failures (no constraints on behaviour)

The four `*Fault` concepts under `OperationalFault` derive from this.

**Lamport, L., Shostak, R., Pease, M. (1982).** *"The Byzantine Generals Problem"*. ACM Transactions on Programming Languages and Systems, 4(3), 382–401. DOI: [10.1145/357172.357176](https://doi.org/10.1145/357172.357176)

Origin of the term "Byzantine fault" — arbitrary, possibly adversarial behaviour including inconsistent reports to different observers. Cited for the `ByzantineFault` concept.

## Related (consulted but not directly extracted)

**Lyu, M.R. (Ed.) (1995).** *Software Fault Tolerance*. Wiley series in Software Engineering Practice. ISBN 0-471-95068-8.

Surveys recovery blocks, N-version programming, and consensus protocols. Concepts feed into the upcoming Resilience ontology (#123) rather than this one.

**Patterson, D., Brown, A., Broadwell, P., Candea, G., Chen, M., Cutler, J., Enriquez, P., Fox, A., Kıcıman, E., Merzbacher, M., Oppenheimer, D., Sastry, N., Tetzlaff, W., Traupman, J., Treuhaft, N. (2002).** *"Recovery-Oriented Computing (ROC): Motivation, Definition, Techniques, and Case Studies"*. UC Berkeley + Stanford technical report.

Argues "failures are inevitable; design for fast recovery." Concepts feed into #123.

**IFIP WG 10.4 on Dependable Computing and Fault Tolerance.** Working group that produced the canonical vocabulary which Avizienis et al. (2004) consolidated.

## Related ontologies in this workspace

The Dependability ontology composes with — but does not yet have strict-Functor mappings to — these existing ontologies. The cross-domain relationships are documented in `mod.rs` and tracked as a follow-up (lax-functor framework support needed):

- `formal/information/diagnostics` — Reiter (1987) diagnostic cycle. Failure ↔ Symptom, Fault ↔ FaultMode, ErrorDetection ↔ Test.
- `formal/systems/control` — Wiener (1948) cybernetics. FaultTolerance IS feedback control on Errors.
- `applied/sensor_fusion/observation` — JDL Level 0. Failure detection IS observation.

## Future / dependent work

- **#123** — Resilience ontology (Nygard, Brooker, Erlang OTP, Patterson ROC). Defines HOW to recover; this defines WHAT is being recovered from.
- **#124** — Endofunctor trait. Recovery transformations like ExponentialBackoff: Duration → Duration are endofunctors on a single category.
- Lax-functor framework support — to allow Dependability → Diagnostics and similar dense-to-kinded structure-preserving mappings.
- `Ontology::validate()` migration from `Result<(), Vec<String>>` to a typed dependability error.
