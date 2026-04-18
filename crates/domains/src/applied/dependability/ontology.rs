//! Dependability ontology — the canonical taxonomy of dependable computing.
//!
//! Grounded in the foundational paper:
//!
//! > Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004).
//! > "Basic Concepts and Taxonomy of Dependable and Secure Computing".
//! > IEEE Transactions on Dependable and Secure Computing, 1(1), 11–33.
//! > DOI: 10.1109/TDSC.2004.2
//!
//! The central construct is the **Fault → Error → Failure chain**:
//!
//! - A **Fault** is the adjudged or hypothesized cause of an Error.
//!   Faults can be dormant (not yet activated) or active.
//! - An **Error** is the part of the system state that may lead to a
//!   subsequent service Failure. Errors can be latent (not yet detected)
//!   or detected.
//! - A **Failure** is an event that occurs when the delivered service
//!   deviates from correct service. Failures are visible to the user.
//!
//! Three **Threats** to dependability: Fault, Error, Failure.
//! Six core **Attributes**: Availability, Reliability, Safety,
//! Confidentiality, Integrity, Maintainability.
//! Four **Means**: Fault Prevention, Fault Tolerance, Fault Removal,
//! Fault Forecasting.
//!
//! Secondary sources cited within concept definitions:
//! - Cristian, F. (1991). "Understanding Fault-Tolerant Distributed
//!   Systems". CACM 34(2). — fault models (crash, omission, timing, byzantine).
//! - Lyu, M.R. (1995). *Software Fault Tolerance*. Wiley.
//! - Lamport, Shostak, Pease (1982). "The Byzantine Generals Problem".
//!   ACM TOPLAS 4(3). — byzantine failure mode.
//! - Patterson et al. (2002). "Recovery-Oriented Computing". UC Berkeley.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Dependability",
    source: "Avizienis, Laprie, Randell, Landwehr (2004) IEEE TDSC; Cristian (1991); Lamport et al. (1982)",
    being: AbstractObject,

    concepts: [
        // === Service (the delivered behaviour) ===
        Service,
        CorrectService,
        ServiceFailure,
        ServiceRestoration,

        // === The Fault → Error → Failure chain (Avizienis §2.2) ===
        Threat,
        Fault,
        Error,
        Failure,

        // === Fault classes (Avizienis §3.2) ===
        // Dormant vs Active is an orthogonal classification of every Fault.
        DormantFault,
        ActiveFault,
        // Persistence
        PermanentFault,
        TransientFault,
        // Phenomenological cause
        NaturalFault,
        HumanMadeFault,
        // Phase of creation
        DevelopmentFault,
        OperationalFault,
        // Specific fault models from Cristian (1991)
        CrashFault,
        OmissionFault,
        TimingFault,
        ByzantineFault,

        // === Error classes (Avizienis §3.3) ===
        DetectedError,
        LatentError,

        // === Failure modes (Avizienis §3.4) ===
        ContentFailure,
        TimingFailure,
        HaltFailure,
        ErraticFailure,

        // === Attributes (Avizienis §4) — what dependability requires ===
        Attribute,
        Availability,
        Reliability,
        Safety,
        Confidentiality,
        Integrity,
        Maintainability,

        // === Means (Avizienis §5) — how to achieve dependability ===
        Means,
        FaultPrevention,
        FaultTolerance,
        FaultRemoval,
        FaultForecasting,

        // === Fault tolerance sub-mechanisms (Avizienis §5.3) ===
        ErrorDetection,
        ErrorRecovery,
        ErrorHandling,
        FaultHandling,

        // === Activation / propagation events ===
        Activation,
        Propagation,
    ],

    labels: {
        Service: ("en", "Service", "The behaviour as perceived by its users (Avizienis §2.1)."),
        CorrectService: ("en", "Correct service", "Service that implements the system function correctly."),
        ServiceFailure: ("en", "Service failure", "The Failure that occurs when delivered service deviates from correct service. A specialization of Failure scoped to the service-delivery boundary."),
        ServiceRestoration: ("en", "Service restoration", "Transition from failed to correct service."),

        Threat: ("en", "Threat", "An impairment to dependability — fault, error, or failure."),
        Fault: ("en", "Fault", "Adjudged or hypothesized cause of an error (Avizienis §2.2)."),
        Error: ("en", "Error", "Part of the system state that may lead to a subsequent failure."),
        Failure: ("en", "Failure", "Event when service deviates from correct service. Visible to user."),

        DormantFault: ("en", "Dormant fault", "A fault that has not yet been activated. Latent capacity for harm."),
        ActiveFault: ("en", "Active fault", "A fault that has been activated and produced an error."),
        PermanentFault: ("en", "Permanent fault", "A fault whose presence is continuous in time."),
        TransientFault: ("en", "Transient fault", "A fault whose presence is bounded in time."),
        NaturalFault: ("en", "Natural fault", "Fault caused by natural phenomena (cosmic ray, wear, etc.)."),
        HumanMadeFault: ("en", "Human-made fault", "Fault caused by human action (bug, misuse, attack)."),
        DevelopmentFault: ("en", "Development fault", "Fault introduced during system development."),
        OperationalFault: ("en", "Operational fault", "Fault occurring during system operation."),

        CrashFault: ("en", "Crash fault", "Cristian (1991): system halts; observable to others."),
        OmissionFault: ("en", "Omission fault", "Cristian (1991): system fails to send/receive a message."),
        TimingFault: ("en", "Timing fault", "Cristian (1991): output arrives outside its specified interval."),
        ByzantineFault: ("en", "Byzantine fault", "Lamport-Shostak-Pease (1982): arbitrary, possibly adversarial behaviour, including inconsistent reports to different observers."),

        DetectedError: ("en", "Detected error", "Error whose presence has been signalled by a detection mechanism."),
        LatentError: ("en", "Latent error", "Error not yet detected — present in state but unreported."),

        ContentFailure: ("en", "Content failure", "Service delivers wrong content (incorrect output value)."),
        TimingFailure: ("en", "Timing failure", "Service delivered at wrong time (early, late, or never)."),
        HaltFailure: ("en", "Halt failure", "Service delivery ceases entirely."),
        ErraticFailure: ("en", "Erratic failure", "Service oscillates between correct and incorrect modes."),

        Attribute: ("en", "Attribute", "A required property of dependability."),
        Availability: ("en", "Availability", "Readiness for correct service. Often expressed as MTBF/(MTBF+MTTR)."),
        Reliability: ("en", "Reliability", "Continuity of correct service over an interval."),
        Safety: ("en", "Safety", "Absence of catastrophic consequences on users and environment."),
        Confidentiality: ("en", "Confidentiality", "Absence of unauthorized disclosure of information."),
        Integrity: ("en", "Integrity", "Absence of improper system alterations."),
        Maintainability: ("en", "Maintainability", "Ability to undergo modifications and repairs."),

        Means: ("en", "Means", "A category of techniques for achieving dependability."),
        FaultPrevention: ("en", "Fault prevention", "Means to prevent the occurrence or introduction of faults (Avizienis §5.1)."),
        FaultTolerance: ("en", "Fault tolerance", "Means to deliver correct service in the presence of faults (Avizienis §5.2)."),
        FaultRemoval: ("en", "Fault removal", "Means to reduce the number and severity of faults (Avizienis §5.3)."),
        FaultForecasting: ("en", "Fault forecasting", "Means to estimate present number, future incidence, likely consequences of faults (Avizienis §5.4)."),

        ErrorDetection: ("en", "Error detection", "Identifying that an error has occurred (precondition for recovery)."),
        ErrorRecovery: ("en", "Error recovery", "Replacing erroneous state with error-free state."),
        ErrorHandling: ("en", "Error handling", "Eliminating errors from the system state — rollback, rollforward, compensation."),
        FaultHandling: ("en", "Fault handling", "Preventing located faults from being activated again — diagnosis, isolation, reconfiguration."),

        Activation: ("en", "Activation", "Transition from dormant fault to active fault, producing an error."),
        Propagation: ("en", "Propagation", "Process by which an error transforms into a failure (or further errors)."),
    },

    is_a: [
        // === Threat hierarchy ===
        (Fault, Threat),
        (Error, Threat),
        (Failure, Threat),

        // === Service modes ===
        // CorrectService is a Service mode. ServiceFailure is the EVENT
        // when delivered service deviates — classified as a Failure
        // (not as a Service mode), per Avizienis §2.1.
        (CorrectService, Service),
        (ServiceFailure, Failure),

        // === Fault classes (orthogonal sub-classes of Fault) ===
        (DormantFault, Fault),
        (ActiveFault, Fault),
        (PermanentFault, Fault),
        (TransientFault, Fault),
        (NaturalFault, Fault),
        (HumanMadeFault, Fault),
        (DevelopmentFault, Fault),
        (OperationalFault, Fault),
        // Cristian fault models: each is an OperationalFault
        (CrashFault, OperationalFault),
        (OmissionFault, OperationalFault),
        (TimingFault, OperationalFault),
        (ByzantineFault, OperationalFault),

        // === Error sub-classes ===
        (DetectedError, Error),
        (LatentError, Error),

        // === Failure modes (sub-classes of Failure) ===
        (ContentFailure, Failure),
        (TimingFailure, Failure),
        (HaltFailure, Failure),
        (ErraticFailure, Failure),

        // === Attribute hierarchy ===
        (Availability, Attribute),
        (Reliability, Attribute),
        (Safety, Attribute),
        (Confidentiality, Attribute),
        (Integrity, Attribute),
        (Maintainability, Attribute),

        // === Means hierarchy ===
        (FaultPrevention, Means),
        (FaultTolerance, Means),
        (FaultRemoval, Means),
        (FaultForecasting, Means),

        // === Fault tolerance sub-mechanisms (Avizienis §5.2 hierarchy) ===
        (ErrorDetection, FaultTolerance),
        (ErrorRecovery, FaultTolerance),
        (ErrorHandling, ErrorRecovery),
        (FaultHandling, ErrorRecovery),
    ],

    causes: [
        // === The fundamental F→E→F chain (Avizienis §2.2) ===
        // Activation: Fault → Error.
        (Fault, Error),
        // Propagation: Error → Failure.
        (Error, Failure),
        // === Service transitions ===
        (Failure, ServiceFailure),
        (ServiceRestoration, CorrectService),
        // NOTE: failure recursion (Avizienis §2.4) — "a Failure at layer
        // N becomes a Fault at layer N+1" — is intentionally NOT a causal
        // edge. Causation is asymmetric (framework axiom), and Failure→
        // Fault would close a cycle Fault→Error→Failure→Fault. The
        // recursion is an inter-layer claim about composed systems, not a
        // causal arrow within one ontology. The FailureRecursionDocumented
        // axiom records this property without violating asymmetry.
    ],

    opposes: [
        // === Service oppositions ===
        (CorrectService, ServiceFailure),
        // === Fault state oppositions ===
        (DormantFault, ActiveFault),
        (PermanentFault, TransientFault),
        // === Error detectability ===
        (DetectedError, LatentError),
        // === Cause attribution ===
        (NaturalFault, HumanMadeFault),
        // === Phase ===
        (DevelopmentFault, OperationalFault),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: the dependability category each concept belongs to.
#[derive(Debug, Clone)]
pub struct DependabilityCategoryOf;

impl Quality for DependabilityCategoryOf {
    type Individual = DependabilityConcept;
    type Value = &'static str;

    fn get(&self, c: &DependabilityConcept) -> Option<&'static str> {
        use DependabilityConcept as D;
        Some(match c {
            D::Service | D::CorrectService | D::ServiceFailure | D::ServiceRestoration => "service",
            D::Threat | D::Fault | D::Error | D::Failure => "threat",
            D::DormantFault
            | D::ActiveFault
            | D::PermanentFault
            | D::TransientFault
            | D::NaturalFault
            | D::HumanMadeFault
            | D::DevelopmentFault
            | D::OperationalFault
            | D::CrashFault
            | D::OmissionFault
            | D::TimingFault
            | D::ByzantineFault => "fault-class",
            D::DetectedError | D::LatentError => "error-class",
            D::ContentFailure | D::TimingFailure | D::HaltFailure | D::ErraticFailure => {
                "failure-mode"
            }
            D::Attribute
            | D::Availability
            | D::Reliability
            | D::Safety
            | D::Confidentiality
            | D::Integrity
            | D::Maintainability => "attribute",
            D::Means
            | D::FaultPrevention
            | D::FaultTolerance
            | D::FaultRemoval
            | D::FaultForecasting
            | D::ErrorDetection
            | D::ErrorRecovery
            | D::ErrorHandling
            | D::FaultHandling => "means",
            D::Activation | D::Propagation => "event",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — invariants of the Avizienis taxonomy
// ---------------------------------------------------------------------------

/// Axiom: the direct children of `Threat` are exactly {Fault, Error, Failure}
/// (Avizienis §2.2 — the *three* and *only* threats).
pub struct ThreeThreats;

impl Axiom for ThreeThreats {
    fn description(&self) -> &str {
        "the direct children of Threat are exactly {Fault, Error, Failure} (Avizienis et al. 2004 §2.2)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(DependabilityConcept::Threat);
        let expected = [
            DependabilityConcept::Fault,
            DependabilityConcept::Error,
            DependabilityConcept::Failure,
        ];
        actual.len() == expected.len() && expected.iter().all(|t| actual.contains(t))
    }
}
pr4xis::register_axiom!(
    ThreeThreats,
    "> Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004)."
);

/// Helper: collect direct (non-transitive) children of a concept under
/// the dependability taxonomy.
fn direct_children_of(parent: DependabilityConcept) -> Vec<DependabilityConcept> {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    DependabilityTaxonomy::relations()
        .into_iter()
        .filter_map(|(child, p)| if p == parent { Some(child) } else { None })
        .collect()
}

/// Axiom: the Fault → Error → Failure chain (Avizienis §2.2).
/// Every Fault activates into an Error; every Error propagates into a Failure.
pub struct FaultErrorFailureChain;

impl Axiom for FaultErrorFailureChain {
    fn description(&self) -> &str {
        "Fault activates into Error, Error propagates into Failure (Avizienis et al. 2004 §2.2)"
    }
    fn holds(&self) -> bool {
        let m = DependabilityCategory::morphisms();
        let activates = m
            .iter()
            .any(|r| r.from == DependabilityConcept::Fault && r.to == DependabilityConcept::Error);
        let propagates = m.iter().any(|r| {
            r.from == DependabilityConcept::Error && r.to == DependabilityConcept::Failure
        });
        activates && propagates
    }
}
pr4xis::register_axiom!(
    FaultErrorFailureChain,
    "> Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004)."
);

/// Axiom: failure recursion (Avizienis §2.4).
/// A Failure observed by another component IS a Fault for that component.
/// This is what makes dependability composable across system layers.
///
/// Note: this is an INTER-LAYER claim about composed systems, NOT a
/// causal edge within this ontology (which would violate causation
/// asymmetry — Fault → Error → Failure → Fault would be a cycle).
/// The recursion is a property of how Dependability instances compose,
/// expressed structurally as "both Failure and Fault exist in the
/// taxonomy" + the meta-claim that the same `Failure` instance at layer
/// N becomes a `Fault` instance at layer N+1.
pub struct FailureRecursionDocumented;

impl Axiom for FailureRecursionDocumented {
    fn description(&self) -> &str {
        "Failure and Fault are both Threats AND no direct Failure→Fault causal edge exists (Avizienis §2.4 inter-layer recursion preserved without breaking causation asymmetry)"
    }
    fn holds(&self) -> bool {
        // Precondition: both concepts exist as Threats.
        use pr4xis::ontology::reasoning::causation::CausalDef;
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;

        let rels = DependabilityTaxonomy::relations();
        let failure_is_threat = rels.iter().any(|(c, p)| {
            *c == DependabilityConcept::Failure && *p == DependabilityConcept::Threat
        });
        let fault_is_threat = rels
            .iter()
            .any(|(c, p)| *c == DependabilityConcept::Fault && *p == DependabilityConcept::Threat);

        // Guard: no Failure → Fault causal edge (would close the cycle
        // Fault → Error → Failure → Fault, violating asymmetry).
        let no_direct_recursion = !DependabilityCausation::relations()
            .iter()
            .any(|(c, e)| *c == DependabilityConcept::Failure && *e == DependabilityConcept::Fault);

        failure_is_threat && fault_is_threat && no_direct_recursion
    }
}
pr4xis::register_axiom!(
    FailureRecursionDocumented,
    "> Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004)."
);

/// Axiom: the direct children of `Attribute` are exactly the six core
/// attributes (Avizienis §4 — excluding security extensions). Adding a
/// seventh direct child would invalidate this axiom.
pub struct SixCoreAttributes;

impl Axiom for SixCoreAttributes {
    fn description(&self) -> &str {
        "the direct children of Attribute are exactly {Availability, Reliability, Safety, Confidentiality, Integrity, Maintainability} (Avizienis et al. 2004 §4)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(DependabilityConcept::Attribute);
        let expected = [
            DependabilityConcept::Availability,
            DependabilityConcept::Reliability,
            DependabilityConcept::Safety,
            DependabilityConcept::Confidentiality,
            DependabilityConcept::Integrity,
            DependabilityConcept::Maintainability,
        ];
        actual.len() == expected.len() && expected.iter().all(|a| actual.contains(a))
    }
}
pr4xis::register_axiom!(
    SixCoreAttributes,
    "> Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004)."
);

/// Axiom: the direct children of `Means` are exactly the four means
/// (Avizienis §5). A fifth direct child would invalidate this axiom.
pub struct FourMeans;

impl Axiom for FourMeans {
    fn description(&self) -> &str {
        "the direct children of Means are exactly {FaultPrevention, FaultTolerance, FaultRemoval, FaultForecasting} (Avizienis et al. 2004 §5)"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(DependabilityConcept::Means);
        let expected = [
            DependabilityConcept::FaultPrevention,
            DependabilityConcept::FaultTolerance,
            DependabilityConcept::FaultRemoval,
            DependabilityConcept::FaultForecasting,
        ];
        actual.len() == expected.len() && expected.iter().all(|m| actual.contains(m))
    }
}
pr4xis::register_axiom!(
    FourMeans,
    "> Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004)."
);

/// Axiom: the four Cristian (1991) operational fault models are all
/// classified as OperationalFault. (The literature also describes a
/// tolerance-subsumption ordering Crash ≼ Omission ≼ Timing ≼ Byzantine
/// — meaning a Byzantine-tolerant system handles all weaker cases — but
/// that is a property of fault-TOLERANT systems, not a classification
/// in the fault taxonomy itself, so it isn't asserted here.)
pub struct CristianFaultModelsExist;

impl Axiom for CristianFaultModelsExist {
    fn description(&self) -> &str {
        "Cristian (1991) operational fault models {Crash, Omission, Timing, Byzantine} are all classified as OperationalFault"
    }
    fn holds(&self) -> bool {
        let actual = direct_children_of(DependabilityConcept::OperationalFault);
        let expected = [
            DependabilityConcept::CrashFault,
            DependabilityConcept::OmissionFault,
            DependabilityConcept::TimingFault,
            DependabilityConcept::ByzantineFault,
        ];
        // OperationalFault may have additional direct children in the
        // future (e.g. domain-specific operational faults), so we check
        // membership rather than exactness here — unlike the closed sets
        // {Threat children} and {Attribute/Means children} above.
        expected.iter().all(|m| actual.contains(m))
    }
}
pr4xis::register_axiom!(
    CristianFaultModelsExist,
    "> Avizienis, A., Laprie, J.-C., Randell, B., Landwehr, C. (2004)."
);

impl Ontology for DependabilityOntology {
    type Cat = DependabilityCategory;
    type Qual = DependabilityCategoryOf;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        DependabilityOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ThreeThreats),
            Box::new(FaultErrorFailureChain),
            Box::new(FailureRecursionDocumented),
            Box::new(SixCoreAttributes),
            Box::new(FourMeans),
            Box::new(CristianFaultModelsExist),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<DependabilityCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        DependabilityOntology::validate().unwrap();
    }

    #[test]
    fn concept_count() {
        // Service (4) + Threat (4) + FaultClasses (12) + ErrorClasses (2)
        // + FailureModes (4) + Attributes (7) + Means (5) + FtMechanisms (4)
        // + Events (2) = 44
        assert_eq!(DependabilityConcept::variants().len(), 44);
    }

    /// Per `feedback_ontological_assertions.md`: tests wrap Axiom.holds().
    #[test]
    fn three_threats_axiom_holds() {
        assert!(ThreeThreats.holds(), "{}", ThreeThreats.description());
    }

    #[test]
    fn fault_error_failure_chain_axiom_holds() {
        assert!(
            FaultErrorFailureChain.holds(),
            "{}",
            FaultErrorFailureChain.description()
        );
    }

    #[test]
    fn failure_recursion_axiom_holds() {
        assert!(
            FailureRecursionDocumented.holds(),
            "{}",
            FailureRecursionDocumented.description()
        );
    }

    #[test]
    fn six_core_attributes_axiom_holds() {
        assert!(
            SixCoreAttributes.holds(),
            "{}",
            SixCoreAttributes.description()
        );
    }

    #[test]
    fn four_means_axiom_holds() {
        assert!(FourMeans.holds(), "{}", FourMeans.description());
    }

    #[test]
    fn cristian_fault_models_axiom_holds() {
        assert!(
            CristianFaultModelsExist.holds(),
            "{}",
            CristianFaultModelsExist.description()
        );
    }
}
