use super::*;
use chrono::{NaiveDate, Utc};
use praxis::category::{Category, Entity as CategoryEntity};
use praxis::engine::{Action, EngineError, Precondition, Situation};
use praxis::logic::Axiom;
use praxis::ontology::Quality;
use proptest::prelude::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn test_court() -> super::entity::Entity {
    super::entity::Entity::Court(super::entity::Court {
        name: "District Court".into(),
        district: None,
        circuit: None,
    })
}

fn test_judge() -> super::entity::Entity {
    super::entity::Entity::Person(super::entity::Person {
        name: "Judge Smith".into(),
        title: Some("Judge".into()),
        organization: None,
        bar_admissions: vec![],
        source: None,
    })
}

fn test_movant() -> super::entity::Entity {
    super::entity::Entity::Person(super::entity::Person {
        name: "Plaintiff".into(),
        title: None,
        organization: None,
        bar_admissions: vec![],
        source: None,
    })
}

fn test_respondent() -> super::entity::Entity {
    super::entity::Entity::Person(super::entity::Person {
        name: "Defendant".into(),
        title: None,
        organization: None,
        bar_admissions: vec![],
        source: None,
    })
}

fn test_pending_motion() -> Decision {
    Decision {
        question: "test".into(),
        motion_type: MotionType::MotionToDismiss {
            grounds: "failure to state a claim".into(),
        },
        status: MotionStatus::Pending {
            filed: date(2024, 3, 1),
            movant: test_movant(),
        },
        arguments: vec![],
        assessment: Assessment {
            summary: "".into(),
            risk_comparison: None,
            checklist_summary: None,
        },
    }
}

fn arb_status_tag() -> impl Strategy<Value = StatusTag> {
    prop_oneof![
        Just(StatusTag::Pending),
        Just(StatusTag::Opposed),
        Just(StatusTag::UnderAdvisement),
        Just(StatusTag::Granted),
        Just(StatusTag::Denied),
        Just(StatusTag::GrantedInPart),
        Just(StatusTag::Moot),
        Just(StatusTag::Withdrawn),
    ]
}

fn arb_phase_tag() -> impl Strategy<Value = PhaseTag> {
    prop_oneof![
        Just(PhaseTag::PreFiling),
        Just(PhaseTag::Filed),
        Just(PhaseTag::Discovery),
        Just(PhaseTag::Motions),
        Just(PhaseTag::PreTrial),
        Just(PhaseTag::Trial),
        Just(PhaseTag::PostTrial),
        Just(PhaseTag::Appeal),
        Just(PhaseTag::Closed),
    ]
}

fn arb_severity() -> impl Strategy<Value = Severity> {
    prop_oneof![
        Just(Severity::Info),
        Just(Severity::Low),
        Just(Severity::Medium),
        Just(Severity::High),
        Just(Severity::Critical),
    ]
}

// =============================================================================
// Source tests
// =============================================================================

#[test]
fn test_source_tier_ordering() {
    assert!(SourceTier::tier1() < SourceTier::tier2());
    assert!(SourceTier::tier2() < SourceTier::tier3());
}

#[test]
fn test_severity_ordering() {
    assert!(Severity::Info < Severity::Critical);
}

// =============================================================================
// Rich enum tests — Answer
// =============================================================================

#[test]
fn test_answer_yes_carries_context() {
    let answer = Answer::Yes {
        confidence: 0.95,
        basis: "direct testimony".into(),
    };
    assert!(answer.is_met());
    assert_eq!(answer.tag(), AnswerTag::Yes);
}

#[test]
fn test_answer_partial_carries_context() {
    let answer = Answer::Partial {
        met: "timing".into(),
        unmet: "intent".into(),
    };
    assert!(!answer.is_met());
    assert_eq!(answer.tag(), AnswerTag::Partial);
}

// =============================================================================
// Rich enum tests — MotionStatus
// =============================================================================

#[test]
fn test_motion_pending_carries_filed_date() {
    let status = MotionStatus::Pending {
        filed: date(2024, 3, 1),
        movant: test_movant(),
    };
    assert!(!status.is_terminal());
    assert_eq!(status.tag(), StatusTag::Pending);
}

#[test]
fn test_motion_granted_carries_judge() {
    let status = MotionStatus::Granted {
        ruling_date: date(2024, 6, 15),
        judge: test_judge(),
        order: "Motion granted. Defendant's counsel disqualified.".into(),
    };
    assert!(status.is_terminal());
}

#[test]
fn test_motion_lifecycle_with_act() {
    let motion = test_pending_motion();
    // Oppose
    let motion = motion
        .act(MotionAction::Oppose {
            date: date(2024, 3, 15),
            by: test_respondent(),
        })
        .unwrap();
    assert_eq!(motion.status.tag(), StatusTag::Opposed);

    // Take under advisement
    let motion = motion
        .act(MotionAction::TakeUnderAdvisement {
            date: date(2024, 4, 1),
        })
        .unwrap();
    assert_eq!(motion.status.tag(), StatusTag::UnderAdvisement);

    // Grant
    let motion = motion
        .act(MotionAction::Grant {
            date: date(2024, 5, 1),
            judge: test_judge(),
            order: "Granted.".into(),
        })
        .unwrap();
    assert!(motion.status.is_terminal());
}

#[test]
fn test_cant_grant_pending_motion() {
    let motion = test_pending_motion();
    let result = motion.act(MotionAction::Grant {
        date: date(2024, 5, 1),
        judge: test_judge(),
        order: "".into(),
    });
    assert!(result.is_err());
}

// =============================================================================
// Rich enum tests — CasePhase
// =============================================================================

#[test]
fn test_case_filed_carries_court() {
    let phase = CasePhase::Filed {
        court: test_court(),
        date: date(2024, 1, 15),
    };
    assert_eq!(phase.tag(), PhaseTag::Filed);
    assert!(!phase.is_terminal());
}

#[test]
fn test_case_closed_carries_reason() {
    let phase = CasePhase::Closed {
        reason: CloseReason::Settlement {
            terms: "confidential".into(),
        },
        date: date(2024, 6, 1),
    };
    assert!(phase.is_terminal());
}

// =============================================================================
// Case lifecycle tests
// =============================================================================

#[test]
fn test_full_case_lifecycle() {
    let mut case = Case::new("Smith v. Corp");
    assert!(
        case.act(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1)
        })
        .is_ok()
    );
    assert!(
        case.act(CaseAction::BeginDiscovery {
            date: date(2024, 2, 1)
        })
        .is_ok()
    );
    assert!(
        case.act(CaseAction::SetForTrial {
            date: date(2024, 12, 1)
        })
        .is_ok()
    );
    assert!(
        case.act(CaseAction::BeginTrial {
            date: date(2024, 12, 1)
        })
        .is_ok()
    );
    assert!(
        case.act(CaseAction::Verdict {
            outcome: "plaintiff".into(),
            date: date(2024, 12, 15)
        })
        .is_ok()
    );
    assert_eq!(case.phase.tag(), PhaseTag::PostTrial);
}

#[test]
fn test_settlement_with_terms() {
    let mut case = Case::new("Test");
    case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    case.act(CaseAction::Settle {
        terms: "$500k confidential".into(),
        date: date(2024, 3, 1),
    });
    if let CasePhase::Closed { reason, .. } = &case.phase {
        let is_settlement = matches!(reason, CloseReason::Settlement { .. });
        assert!(is_settlement);
    } else {
        panic!("case should be closed");
    }
}

#[test]
fn test_dismissal_with_prejudice() {
    let mut case = Case::new("Test");
    case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    case.act(CaseAction::Dismiss {
        reason: "failure to state a claim".into(),
        with_prejudice: true,
        date: date(2024, 2, 1),
    });
    if let CasePhase::Closed { reason, .. } = &case.phase {
        if let CloseReason::Dismissal { with_prejudice, .. } = reason {
            assert!(with_prejudice);
        } else {
            panic!("should be dismissal");
        }
    }
}

#[test]
fn test_cant_act_on_closed_case() {
    let mut case = Case::new("Test");
    case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    case.act(CaseAction::Settle {
        terms: "terms".into(),
        date: date(2024, 2, 1),
    });
    let result = case.act(CaseAction::BeginDiscovery {
        date: date(2024, 3, 1),
    });
    assert!(!result.is_ok());
}

// =============================================================================
// Element tests
// =============================================================================

#[test]
fn test_element_all_met() {
    let check = ElementCheck {
        statute: super::authority::Authority::Constitution {
            provision: "test".into(),
        },
        elements: vec![
            Element {
                name: "A".into(),
                description: "".into(),
                answer: Answer::Yes {
                    confidence: 1.0,
                    basis: "doc".into(),
                },
                evidence: vec![],
                analysis: None,
            },
            Element {
                name: "B".into(),
                description: "".into(),
                answer: Answer::Yes {
                    confidence: 0.9,
                    basis: "testimony".into(),
                },
                evidence: vec![],
                analysis: None,
            },
        ],
    };
    assert!(check.all_met());
    assert_eq!(check.tally(), (2, 0, 0, 0));
}

#[test]
fn test_element_not_all_met() {
    let check = ElementCheck {
        statute: super::authority::Authority::Constitution {
            provision: "test".into(),
        },
        elements: vec![
            Element {
                name: "A".into(),
                description: "".into(),
                answer: Answer::Yes {
                    confidence: 1.0,
                    basis: "".into(),
                },
                evidence: vec![],
                analysis: None,
            },
            Element {
                name: "B".into(),
                description: "".into(),
                answer: Answer::No {
                    reason: "no evidence".into(),
                },
                evidence: vec![],
                analysis: None,
            },
        ],
    };
    assert!(!check.all_met());
    assert_eq!(check.tally(), (1, 1, 0, 0));
}

// =============================================================================
// Ontology tests
// =============================================================================

#[test]
fn test_ontology_registry() {
    let mut registry = OntologyRegistry::new();
    registry.register(LegalCategory {
        name: "test".into(),
        description: "test".into(),
        authority: super::authority::Authority::Constitution {
            provision: "test".into(),
        },
        terms: vec![LegalTerm {
            id: "test:term1".into(),
            name: "Term".into(),
            definition: "A term".into(),
            source_text: None,
            valence: Valence::Supportive,
            subsection: None,
            required_evidence: vec![],
            obligations: vec![],
            deadlines: vec![],
            rights: vec![],
            remedies: vec![],
            burdens: vec![],
            exceptions: vec![],
        }],
        relations: vec![],
    });
    assert!(registry.get_term("test:term1").is_some());
    assert!(registry.get_term("nonexistent").is_none());
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// Terminal status tags have no transitions
    #[test]
    fn prop_terminal_no_transitions(tag in arb_status_tag()) {
        if tag.is_terminal() {
            prop_assert!(tag.valid_transitions().is_empty());
        }
    }

    /// Non-terminal status tags have at least one transition
    #[test]
    fn prop_non_terminal_has_transitions(tag in arb_status_tag()) {
        if !tag.is_terminal() {
            prop_assert!(!tag.valid_transitions().is_empty());
        }
    }

    /// Terminal phase tags have no transitions
    #[test]
    fn prop_phase_terminal_no_transitions(tag in arb_phase_tag()) {
        if tag.is_terminal() {
            prop_assert!(tag.valid_transitions().is_empty());
        }
    }

    /// Closed is the only terminal phase
    #[test]
    fn prop_only_closed_terminal(tag in arb_phase_tag()) {
        prop_assert_eq!(tag.is_terminal(), tag == PhaseTag::Closed);
    }

    /// Severity ordering is total and transitive
    #[test]
    fn prop_severity_total(a in arb_severity(), b in arb_severity()) {
        // Total ordering: partial_cmp always returns Some
        prop_assert!(a.partial_cmp(&b).is_some());
    }

    /// Source tier ordering
    #[test]
    fn prop_tier_ordering(a in 1..5u8, b in 1..5u8) {
        prop_assert_eq!(SourceTier(a) < SourceTier(b), a < b);
    }

    /// Answer::Yes is the only met answer
    #[test]
    fn prop_only_yes_is_met(_x in 0..1u8) {
        let yes = Answer::Yes { confidence: 1.0, basis: "".into() };
        let no = Answer::No { reason: "".into() };
        let partial = Answer::Partial { met: "".into(), unmet: "".into() };
        let unknown = Answer::Unknown { needs: "".into() };
        prop_assert!(yes.is_met());
        prop_assert!(!no.is_met());
        prop_assert!(!partial.is_met());
        prop_assert!(!unknown.is_met());
    }

    /// ElementCheck tally sums to element count
    #[test]
    fn prop_tally_sums(yes in 0..5usize, no in 0..5usize, partial in 0..5usize, unknown in 0..5usize) {
        let mut elements = Vec::new();
        for _ in 0..yes { elements.push(Element { name: "".into(), description: "".into(), answer: Answer::Yes { confidence: 1.0, basis: "".into() }, evidence: vec![], analysis: None }); }
        for _ in 0..no { elements.push(Element { name: "".into(), description: "".into(), answer: Answer::No { reason: "".into() }, evidence: vec![], analysis: None }); }
        for _ in 0..partial { elements.push(Element { name: "".into(), description: "".into(), answer: Answer::Partial { met: "".into(), unmet: "".into() }, evidence: vec![], analysis: None }); }
        for _ in 0..unknown { elements.push(Element { name: "".into(), description: "".into(), answer: Answer::Unknown { needs: "".into() }, evidence: vec![], analysis: None }); }
        let check = ElementCheck {
            statute: super::authority::Authority::Constitution { provision: "".into() },
            elements,
        };
        let (y, n, p, u) = check.tally();
        prop_assert_eq!(y + n + p + u, yes + no + partial + unknown);
    }

    /// all_met iff all answers are Yes
    #[test]
    fn prop_all_met_iff_all_yes(answers in proptest::collection::vec(
        prop_oneof![
            Just(AnswerTag::Yes), Just(AnswerTag::No),
            Just(AnswerTag::Partial), Just(AnswerTag::Unknown),
        ], 1..10
    )) {
        let elements: Vec<Element> = answers.iter().map(|t| Element {
            name: "".into(), description: "".into(),
            answer: match t {
                AnswerTag::Yes => Answer::Yes { confidence: 1.0, basis: "".into() },
                AnswerTag::No => Answer::No { reason: "".into() },
                AnswerTag::Partial => Answer::Partial { met: "".into(), unmet: "".into() },
                AnswerTag::Unknown => Answer::Unknown { needs: "".into() },
            },
            evidence: vec![], analysis: None,
        }).collect();
        let check = ElementCheck {
            statute: super::authority::Authority::Constitution { provision: "".into() },
            elements,
        };
        let all_yes = answers.iter().all(|a| *a == AnswerTag::Yes);
        prop_assert_eq!(check.all_met(), all_yes);
    }

    /// New case always starts PreFiling
    #[test]
    fn prop_new_case_prefiling(caption in "[a-z]{3,10} v\\. [a-z]{3,10}") {
        let case = Case::new(&caption);
        prop_assert_eq!(case.phase.tag(), PhaseTag::PreFiling);
    }

    /// Filing moves to Filed
    #[test]
    fn prop_filing_transitions(_x in 0..1u8) {
        let mut case = Case::new("Test");
        let result = case.act(CaseAction::File { court: test_court(), date: date(2024, 1, 1) });
        prop_assert!(result.is_ok());
        prop_assert_eq!(case.phase.tag(), PhaseTag::Filed);
    }

    /// Settlement always closes
    #[test]
    fn prop_settlement_closes(_x in 0..1u8) {
        let mut case = Case::new("Test");
        case.act(CaseAction::File { court: test_court(), date: date(2024, 1, 1) });
        case.act(CaseAction::Settle { terms: "terms".into(), date: date(2024, 2, 1) });
        prop_assert_eq!(case.phase.tag(), PhaseTag::Closed);
    }

    /// Dismissal always closes
    #[test]
    fn prop_dismissal_closes(_x in 0..1u8) {
        let mut case = Case::new("Test");
        case.act(CaseAction::File { court: test_court(), date: date(2024, 1, 1) });
        case.act(CaseAction::Dismiss { reason: "moot".into(), with_prejudice: false, date: date(2024, 2, 1) });
        prop_assert_eq!(case.phase.tag(), PhaseTag::Closed);
    }

    /// Authority weight is bounded
    #[test]
    fn prop_authority_weight(_x in 0..1u8) {
        let auth = super::authority::Authority::Constitution { provision: "test".into() };
        prop_assert!(auth.weight() >= 3 && auth.weight() <= 10);
    }

    /// Constitution has highest weight
    #[test]
    fn prop_constitution_highest(_x in 0..1u8) {
        let c = super::authority::Authority::Constitution { provision: "".into() };
        let t = super::authority::Authority::TrialCourt {
            court: test_court(),
            case: super::authority::CaseLaw { name: "".into(), citation: super::authority::Citation::Statute { code: "".into(), section: "".into(), subsection: None, short: None }, year: 2024, court: "".into(), holding: "".into(), quote: None },
        };
        prop_assert!(c.weight() > t.weight());
    }

    /// Motion action on terminal status always fails
    #[test]
    fn prop_terminal_motion_rejects_all(_x in 0..1u8) {
        let motion = Decision {
            question: "".into(),
            motion_type: MotionType::MotionToDismiss { grounds: "".into() },
            status: MotionStatus::Granted {
                ruling_date: date(2024, 5, 1), judge: test_judge(), order: "".into(),
            },
            arguments: vec![],
            assessment: Assessment { summary: "".into(), risk_comparison: None, checklist_summary: None },
        };
        let result = motion.act(MotionAction::Oppose { date: date(2024, 6, 1), by: test_respondent() });
        prop_assert!(result.is_err());
    }

    /// Pending motion can be opposed
    #[test]
    fn prop_pending_can_oppose(_x in 0..1u8) {
        let motion = test_pending_motion();
        let result = motion.act(MotionAction::Oppose { date: date(2024, 3, 15), by: test_respondent() });
        prop_assert!(result.is_ok());
        let advanced = result.unwrap();
        prop_assert_eq!(advanced.status.tag(), StatusTag::Opposed);
    }

    /// Motion preserves filed date through transitions
    #[test]
    fn prop_filed_date_preserved(_x in 0..1u8) {
        let motion = test_pending_motion();
        let opposed = motion.act(MotionAction::Oppose { date: date(2024, 3, 15), by: test_respondent() }).unwrap();
        if let MotionStatus::Opposed { filed, .. } = &opposed.status {
            prop_assert_eq!(*filed, date(2024, 3, 1));
        }
    }
}

// =============================================================================
// Helper factories for new tests
// =============================================================================

fn now() -> chrono::DateTime<Utc> {
    Utc::now()
}

fn test_source() -> Source {
    Source {
        url: Some("https://example.com/doc".into()),
        document: DocumentType::Other {
            description: "test document".into(),
        },
        tier: SourceTier::tier1(),
        retrieved_at: now(),
        archive: None,
        verification: Verification::Unverified { reason: None },
    }
}

fn test_source_ref() -> SourceRef {
    SourceRef {
        url: Some("https://example.com".into()),
        description: "test ref".into(),
        tier: SourceTier::tier2(),
    }
}

fn test_fact(claim: &str) -> Fact {
    Fact {
        claim: claim.into(),
        value: None,
        date: Some(date(2024, 1, 15)),
        date_precision: DatePrecision::Exact,
        source: test_source(),
        actors: vec![],
        narrative: None,
        severity: Some(Severity::Medium),
    }
}

fn test_finding(title: &str) -> Finding {
    Finding {
        title: title.into(),
        facts: vec![test_fact("supporting fact")],
        analysis: "test analysis".into(),
        severity: Severity::High,
        subject: Some(test_movant()),
        analyzed_at: Some(now()),
    }
}

fn test_authority() -> Authority {
    Authority::Constitution {
        provision: "14th Amendment".into(),
    }
}

fn test_statute() -> Statute {
    Statute {
        citation: Citation::Statute {
            code: "42 U.S.C.".into(),
            section: "1983".into(),
            subsection: None,
            short: Some("Section 1983".into()),
        },
        title: Some("Civil Rights Act".into()),
        text: None,
        effective_date: Some(date(1871, 4, 20)),
    }
}

fn test_case_law() -> CaseLaw {
    CaseLaw {
        name: "Miranda v. Arizona".into(),
        citation: Citation::SupremeCourt {
            name: "Miranda v. Arizona".into(),
            reporter_volume: 384,
            page: 436,
            year: 1966,
        },
        year: 1966,
        court: "Supreme Court".into(),
        holding: "Suspects must be informed of their rights".into(),
        quote: Some("You have the right to remain silent...".into()),
    }
}

fn test_agency() -> Entity {
    Entity::Agency(Agency {
        name: "SEC".into(),
        jurisdiction: "Federal".into(),
    })
}

fn test_corporation() -> Entity {
    Entity::Corporation(Corporation {
        name: "Acme Corp".into(),
        structure: CorporateStructure::PublicCompany,
        jurisdiction: "Delaware".into(),
        source: None,
    })
}

fn test_law_firm() -> Entity {
    Entity::LawFirm(LawFirm {
        name: "Smith & Associates".into(),
        source: None,
    })
}

// =============================================================================
// fact.rs tests
// =============================================================================

#[test]
fn test_fact_value_currency() {
    let v = FactValue::Currency {
        amount: 1_000_000.0,
        currency: "USD".into(),
    };
    if let FactValue::Currency { amount, currency } = &v {
        assert!(*amount > 999_999.0);
        assert_eq!(currency, "USD");
    } else {
        panic!("expected Currency");
    }
}

#[test]
fn test_fact_value_percentage() {
    let v = FactValue::Percentage {
        value: 42.5,
        of_what: "revenue".into(),
    };
    assert_eq!(
        v,
        FactValue::Percentage {
            value: 42.5,
            of_what: "revenue".into(),
        }
    );
}

#[test]
fn test_fact_value_count() {
    let v = FactValue::Count {
        n: 7,
        of_what: "violations".into(),
    };
    if let FactValue::Count { n, of_what } = &v {
        assert_eq!(*n, 7);
        assert_eq!(of_what, "violations");
    }
}

#[test]
fn test_fact_value_duration() {
    let v = FactValue::Duration {
        days: 90,
        from: date(2024, 1, 1),
        to: date(2024, 3, 31),
    };
    if let FactValue::Duration { days, from, to } = &v {
        assert_eq!(*days, 90);
        assert!(to > from);
    }
}

#[test]
fn test_fact_value_rating() {
    let v = FactValue::Rating {
        score: 8.5,
        scale: (0.0, 10.0),
    };
    if let FactValue::Rating { score, scale } = &v {
        assert!(*score >= scale.0 && *score <= scale.1);
    }
}

#[test]
fn test_fact_value_text() {
    let v = FactValue::Text("arbitrary text".into());
    assert_eq!(v, FactValue::Text("arbitrary text".into()));
}

#[test]
fn test_date_precision_variants() {
    assert_eq!(DatePrecision::Exact, DatePrecision::Exact);
    assert_ne!(DatePrecision::Exact, DatePrecision::Month);
    assert_ne!(DatePrecision::Month, DatePrecision::Year);
    assert_ne!(DatePrecision::Year, DatePrecision::Approximate);
}

#[test]
fn test_narrative_step_and_narrative() {
    let step = NarrativeStep {
        actor: test_movant(),
        action: "filed complaint".into(),
        detail: Some("in district court".into()),
        quote: Some("The plaintiff alleges...".into()),
        source: test_source(),
        date: Some(date(2024, 1, 15)),
    };
    let narrative = Narrative {
        chain: vec![step],
        significance: "establishes timeline".into(),
    };
    assert_eq!(narrative.chain.len(), 1);
    assert_eq!(narrative.significance, "establishes timeline");
}

#[test]
fn test_fact_with_all_fields() {
    let fact = Fact {
        claim: "Company backdated documents".into(),
        value: Some(FactValue::Count {
            n: 3,
            of_what: "documents".into(),
        }),
        date: Some(date(2024, 2, 1)),
        date_precision: DatePrecision::Exact,
        source: test_source(),
        actors: vec![test_corporation()],
        narrative: Some(Narrative {
            chain: vec![],
            significance: "shows intent".into(),
        }),
        severity: Some(Severity::Critical),
    };
    assert_eq!(fact.claim, "Company backdated documents");
    assert!(fact.value.is_some());
    assert_eq!(fact.actors.len(), 1);
    assert!(fact.narrative.is_some());
    assert_eq!(fact.severity, Some(Severity::Critical));
}

#[test]
fn test_temporal_proximity() {
    let from = test_fact("complaint filed");
    let to = test_fact("documents shredded");
    let proximity = TemporalProximity {
        from,
        to,
        days: 3,
        significance: "suspicious timing".into(),
    };
    assert_eq!(proximity.days, 3);
    assert_eq!(proximity.significance, "suspicious timing");
}

#[test]
fn test_integrity_issue_backdated() {
    let issue = IntegrityIssue::Backdated {
        document_date: date(2024, 1, 1),
        actual_timestamp: date(2024, 3, 15),
        gap_days: 74,
    };
    if let IntegrityIssue::Backdated { gap_days, .. } = &issue {
        assert_eq!(*gap_days, 74);
    }
}

#[test]
fn test_integrity_issue_created_after_litigation() {
    let issue = IntegrityIssue::CreatedAfterLitigation {
        document_date: date(2024, 4, 1),
        litigation_filed: date(2024, 3, 1),
    };
    if let IntegrityIssue::CreatedAfterLitigation {
        document_date,
        litigation_filed,
    } = &issue
    {
        assert!(document_date > litigation_filed);
    }
}

#[test]
fn test_integrity_issue_unsigned_and_incomplete() {
    let unsigned = IntegrityIssue::Unsigned;
    assert_eq!(unsigned, IntegrityIssue::Unsigned);

    let incomplete = IntegrityIssue::Incomplete {
        missing: "signature page".into(),
    };
    if let IntegrityIssue::Incomplete { missing } = &incomplete {
        assert_eq!(missing, "signature page");
    }
}

#[test]
fn test_document_integrity() {
    let integrity = DocumentIntegrity {
        issues: vec![
            IntegrityIssue::Unsigned,
            IntegrityIssue::Incomplete {
                missing: "exhibits".into(),
            },
        ],
    };
    assert_eq!(integrity.issues.len(), 2);
}

#[test]
fn test_severity_all_orderings() {
    assert!(Severity::Info < Severity::Low);
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
    assert_eq!(Severity::Medium, Severity::Medium);
}

// =============================================================================
// rule.rs tests
// =============================================================================

#[test]
fn test_rule_construction() {
    let rule = Rule {
        name: "Spoliation Inference".into(),
        description: "Negative inference from evidence destruction".into(),
        authority: vec![test_authority()],
        conditions: vec![Condition::FactExists(FactMatcher::ClaimContains(
            "destroyed".into(),
        ))],
        consequence: Consequence {
            finding_type: "spoliation".into(),
            severity: Severity::Critical,
            recommendation: Recommendation::Sanction,
            explanation: "evidence was destroyed".into(),
        },
    };
    assert_eq!(rule.name, "Spoliation Inference");
    assert_eq!(rule.conditions.len(), 1);
    assert_eq!(rule.consequence.recommendation, Recommendation::Sanction);
}

#[test]
fn test_condition_fact_exists() {
    let cond = Condition::FactExists(FactMatcher::ClaimContains("fraud".into()));
    if let Condition::FactExists(FactMatcher::ClaimContains(s)) = &cond {
        assert_eq!(s, "fraud");
    } else {
        panic!("expected FactExists(ClaimContains)");
    }
}

#[test]
fn test_condition_temporal_proximity_within() {
    let cond = Condition::TemporalProximityWithin {
        from: FactMatcher::ClaimContains("filing".into()),
        to: FactMatcher::ClaimContains("destruction".into()),
        max_days: 30,
    };
    if let Condition::TemporalProximityWithin { max_days, .. } = &cond {
        assert_eq!(*max_days, 30);
    }
}

#[test]
fn test_condition_severity_at_least() {
    let cond = Condition::SeverityAtLeast(Severity::High);
    assert_eq!(cond, Condition::SeverityAtLeast(Severity::High));
}

#[test]
fn test_condition_all_of() {
    let cond = Condition::AllOf(vec![
        Condition::FactExists(FactMatcher::Any),
        Condition::SeverityAtLeast(Severity::Medium),
    ]);
    if let Condition::AllOf(inner) = &cond {
        assert_eq!(inner.len(), 2);
    }
}

#[test]
fn test_condition_any_of() {
    let cond = Condition::AnyOf(vec![
        Condition::FactExists(FactMatcher::ClaimContains("A".into())),
        Condition::FactExists(FactMatcher::ClaimContains("B".into())),
    ]);
    if let Condition::AnyOf(inner) = &cond {
        assert_eq!(inner.len(), 2);
    }
}

#[test]
fn test_condition_not() {
    let inner = Condition::FactExists(FactMatcher::Any);
    let cond = Condition::Not(Box::new(inner.clone()));
    if let Condition::Not(boxed) = &cond {
        assert_eq!(**boxed, inner);
    }
}

#[test]
fn test_condition_term_satisfied() {
    let cond = Condition::TermSatisfied("retaliation:protected_activity".into());
    assert_eq!(
        cond,
        Condition::TermSatisfied("retaliation:protected_activity".into())
    );
}

#[test]
fn test_fact_matcher_variants() {
    let _ = FactMatcher::ClaimContains("test".into());
    let _ = FactMatcher::MinSeverity(Severity::Low);
    let _ = FactMatcher::DateRange {
        from: Some(date(2024, 1, 1)),
        to: Some(date(2024, 12, 31)),
    };
    let _ = FactMatcher::DateRange {
        from: None,
        to: None,
    };
    let _ = FactMatcher::InvolvesEntity("Acme Corp".into());
    let _ = FactMatcher::Any;
}

#[test]
fn test_consequence_and_recommendation() {
    let consequence = Consequence {
        finding_type: "conflict_of_interest".into(),
        severity: Severity::High,
        recommendation: Recommendation::Investigate,
        explanation: "potential conflict".into(),
    };
    assert_eq!(consequence.severity, Severity::High);
    assert_eq!(consequence.recommendation, Recommendation::Investigate);

    // All recommendation variants
    assert_ne!(Recommendation::Investigate, Recommendation::Disclose);
    assert_ne!(Recommendation::Monitor, Recommendation::Compel);
    assert_ne!(Recommendation::Sanction, Recommendation::NoAction);
}

#[test]
fn test_rule_evaluation_triggered() {
    let eval = RuleEvaluation {
        rule_name: "test rule".into(),
        condition_results: vec![ConditionResult {
            description: "fact exists".into(),
            status: ConditionStatus::Met,
            evidence: vec![test_fact("evidence")],
        }],
        triggered: Triggered::Yes {
            recommendation: Recommendation::Disclose,
        },
    };
    if let Triggered::Yes { recommendation } = &eval.triggered {
        assert_eq!(*recommendation, Recommendation::Disclose);
    }
}

#[test]
fn test_rule_evaluation_not_triggered() {
    let eval = RuleEvaluation {
        rule_name: "test rule".into(),
        condition_results: vec![ConditionResult {
            description: "severity check".into(),
            status: ConditionStatus::NotMet {
                reason: "severity too low".into(),
            },
            evidence: vec![],
        }],
        triggered: Triggered::No {
            unmet: vec!["severity >= High".into()],
        },
    };
    if let Triggered::No { unmet } = &eval.triggered {
        assert_eq!(unmet.len(), 1);
    }
}

#[test]
fn test_rule_evaluation_partial() {
    let eval = RuleEvaluation {
        rule_name: "test rule".into(),
        condition_results: vec![],
        triggered: Triggered::Partial {
            met: vec!["timing".into()],
            unmet: vec!["intent".into()],
        },
    };
    if let Triggered::Partial { met, unmet } = &eval.triggered {
        assert_eq!(met.len(), 1);
        assert_eq!(unmet.len(), 1);
    }
}

#[test]
fn test_condition_status_variants() {
    let met = ConditionStatus::Met;
    assert_eq!(met, ConditionStatus::Met);

    let not_met = ConditionStatus::NotMet {
        reason: "no evidence".into(),
    };
    if let ConditionStatus::NotMet { reason } = &not_met {
        assert_eq!(reason, "no evidence");
    }

    let partial = ConditionStatus::Partial {
        what_met: "timing".into(),
        what_remains: "intent".into(),
    };
    if let ConditionStatus::Partial {
        what_met,
        what_remains,
    } = &partial
    {
        assert_eq!(what_met, "timing");
        assert_eq!(what_remains, "intent");
    }

    let unknown = ConditionStatus::Unknown {
        needs: "deposition transcript".into(),
    };
    if let ConditionStatus::Unknown { needs } = &unknown {
        assert_eq!(needs, "deposition transcript");
    }
}

// =============================================================================
// finding.rs tests
// =============================================================================

#[test]
fn test_finding_construction() {
    let finding = test_finding("Backdating discovered");
    assert_eq!(finding.title, "Backdating discovered");
    assert_eq!(finding.facts.len(), 1);
    assert_eq!(finding.severity, Severity::High);
    assert!(finding.subject.is_some());
    assert!(finding.analyzed_at.is_some());
}

#[test]
fn test_finding_without_optional_fields() {
    let finding = Finding {
        title: "minor issue".into(),
        facts: vec![],
        analysis: "nothing notable".into(),
        severity: Severity::Info,
        subject: None,
        analyzed_at: None,
    };
    assert!(finding.subject.is_none());
    assert!(finding.analyzed_at.is_none());
    assert!(finding.facts.is_empty());
}

#[test]
fn test_contradiction() {
    let contradiction = Contradiction {
        claimed: test_fact("no contact with witnesses"),
        actual: test_fact("email records show contact"),
        claimed_by: test_respondent(),
        refuted_by: test_source(),
        significance: "perjury risk".into(),
    };
    assert_eq!(contradiction.significance, "perjury risk");
    assert_eq!(contradiction.claimed_by.name(), "Defendant");
}

#[test]
fn test_ruling_merits_granted() {
    let ruling = Ruling::Merits(MeritsRuling {
        date: date(2024, 6, 1),
        court: test_court(),
        judge: Some(test_judge()),
        motion: "Motion for Summary Judgment".into(),
        outcome: MeritsOutcome::Granted {
            detail: Some("no genuine issue of material fact".into()),
        },
        significance: "case-dispositive".into(),
        source: test_source(),
    });
    if let Ruling::Merits(mr) = &ruling {
        assert!(matches!(mr.outcome, MeritsOutcome::Granted { .. }));
        assert!(mr.judge.is_some());
    }
}

#[test]
fn test_ruling_merits_denied() {
    let ruling = MeritsRuling {
        date: date(2024, 6, 1),
        court: test_court(),
        judge: None,
        motion: "Motion to Dismiss".into(),
        outcome: MeritsOutcome::Denied {
            detail: Some("plaintiff stated a claim".into()),
        },
        significance: "case proceeds".into(),
        source: test_source(),
    };
    assert!(matches!(ruling.outcome, MeritsOutcome::Denied { .. }));
    assert!(ruling.judge.is_none());
}

#[test]
fn test_ruling_merits_granted_in_part() {
    let outcome = MeritsOutcome::GrantedInPart {
        granted: "count 1 dismissed".into(),
        denied: "counts 2-3 survive".into(),
    };
    if let MeritsOutcome::GrantedInPart { granted, denied } = &outcome {
        assert!(!granted.is_empty());
        assert!(!denied.is_empty());
    }
}

#[test]
fn test_ruling_procedural_dismissed_with_prejudice() {
    let ruling = Ruling::Procedural(ProceduralRuling {
        date: date(2024, 5, 15),
        court: test_court(),
        judge: Some(test_judge()),
        motion: "12(b)(6) Motion".into(),
        outcome: ProceduralOutcome::DismissedWithPrejudice {
            reason: "failure to state a claim".into(),
        },
        significance: "final dismissal".into(),
        source: test_source(),
    });
    if let Ruling::Procedural(pr) = &ruling {
        assert!(matches!(
            pr.outcome,
            ProceduralOutcome::DismissedWithPrejudice { .. }
        ));
    }
}

#[test]
fn test_procedural_outcome_variants() {
    let dwop = ProceduralOutcome::DismissedWithoutPrejudice {
        reason: "procedural defect".into(),
    };
    assert!(matches!(
        dwop,
        ProceduralOutcome::DismissedWithoutPrejudice { .. }
    ));

    let transferred = ProceduralOutcome::Transferred {
        to: Entity::Court(Court {
            name: "Southern District".into(),
            district: Some("S.D.N.Y.".into()),
            circuit: Some("2nd".into()),
        }),
    };
    if let ProceduralOutcome::Transferred { to } = &transferred {
        assert_eq!(to.name(), "Southern District");
    }

    let stayed = ProceduralOutcome::Stayed {
        duration: Some("90 days".into()),
    };
    assert!(matches!(stayed, ProceduralOutcome::Stayed { .. }));

    let granted = ProceduralOutcome::Granted {
        detail: "leave to amend".into(),
    };
    assert!(matches!(granted, ProceduralOutcome::Granted { .. }));

    let denied = ProceduralOutcome::Denied {
        detail: "untimely".into(),
    };
    assert!(matches!(denied, ProceduralOutcome::Denied { .. }));
}

// =============================================================================
// argument.rs tests
// =============================================================================

#[test]
fn test_argument_construction() {
    let arg = Argument {
        ground: "Retaliation".into(),
        title: "Protected Activity Established".into(),
        findings: vec![test_finding("whistleblower report")],
        standards: vec![test_authority()],
        explanation: "Plaintiff engaged in protected activity".into(),
        reasoning: "Filing an OSHA complaint constitutes protected activity".into(),
        counterarguments: vec![],
        checklist: vec![],
        severity: Severity::High,
    };
    assert_eq!(arg.ground, "Retaliation");
    assert_eq!(arg.findings.len(), 1);
    assert_eq!(arg.standards.len(), 1);
}

#[test]
fn test_counterargument_weak() {
    let ca = Counterargument {
        anticipated: "Performance issues justified termination".into(),
        rebuttal: "No documented performance issues before complaint".into(),
        strength: CounterStrength::Weak {
            reason: "contradicted by performance reviews".into(),
        },
    };
    assert!(matches!(ca.strength, CounterStrength::Weak { .. }));
}

#[test]
fn test_counterargument_moderate() {
    let ca = Counterargument {
        anticipated: "Business restructuring".into(),
        rebuttal: "Only plaintiff's position eliminated".into(),
        strength: CounterStrength::Moderate {
            reason: "partial documentation exists".into(),
        },
    };
    assert!(matches!(ca.strength, CounterStrength::Moderate { .. }));
}

#[test]
fn test_counterargument_strong() {
    let ca = Counterargument {
        anticipated: "Legitimate business reason".into(),
        rebuttal: "Timing still suspicious".into(),
        strength: CounterStrength::Strong {
            reason: "well-documented reorganization".into(),
        },
    };
    if let CounterStrength::Strong { reason } = &ca.strength {
        assert!(reason.contains("well-documented"));
    }
}

#[test]
fn test_check_item() {
    let item = CheckItem {
        question: "Did plaintiff file a complaint?".into(),
        answer: Answer::Yes {
            confidence: 0.95,
            basis: "OSHA filing records".into(),
        },
        evidence: vec![test_fact("OSHA complaint filed")],
    };
    assert!(item.answer.is_met());
    assert_eq!(item.evidence.len(), 1);
}

#[test]
fn test_argument_with_counterarguments_and_checklist() {
    let arg = Argument {
        ground: "Discrimination".into(),
        title: "Disparate Treatment".into(),
        findings: vec![],
        standards: vec![],
        explanation: "Plaintiff was treated differently".into(),
        reasoning: "Similarly situated employees received better treatment".into(),
        counterarguments: vec![
            Counterargument {
                anticipated: "Different qualifications".into(),
                rebuttal: "Comparable qualifications documented".into(),
                strength: CounterStrength::Moderate {
                    reason: "some differences exist".into(),
                },
            },
            Counterargument {
                anticipated: "Performance gap".into(),
                rebuttal: "Performance reviews are comparable".into(),
                strength: CounterStrength::Weak {
                    reason: "contradicted by records".into(),
                },
            },
        ],
        checklist: vec![
            CheckItem {
                question: "Protected class membership?".into(),
                answer: Answer::Yes {
                    confidence: 1.0,
                    basis: "demographic records".into(),
                },
                evidence: vec![],
            },
            CheckItem {
                question: "Adverse employment action?".into(),
                answer: Answer::Yes {
                    confidence: 0.9,
                    basis: "termination letter".into(),
                },
                evidence: vec![],
            },
            CheckItem {
                question: "Similarly situated comparators?".into(),
                answer: Answer::Partial {
                    met: "identified comparators".into(),
                    unmet: "need more data on treatment".into(),
                },
                evidence: vec![],
            },
        ],
        severity: Severity::High,
    };
    assert_eq!(arg.counterarguments.len(), 2);
    assert_eq!(arg.checklist.len(), 3);
    assert!(arg.checklist[0].answer.is_met());
    assert!(!arg.checklist[2].answer.is_met());
}

// =============================================================================
// source.rs tests
// =============================================================================

#[test]
fn test_source_tier_constructors() {
    assert_eq!(SourceTier::tier1().0, 1);
    assert_eq!(SourceTier::tier2().0, 2);
    assert_eq!(SourceTier::tier3().0, 3);
    assert_eq!(SourceTier::tier4().0, 4);
}

#[test]
fn test_source_tier_full_ordering() {
    assert!(SourceTier::tier1() < SourceTier::tier2());
    assert!(SourceTier::tier2() < SourceTier::tier3());
    assert!(SourceTier::tier3() < SourceTier::tier4());
    assert!(SourceTier::tier1() < SourceTier::tier4());
}

#[test]
fn test_source_ref_with_url() {
    let sr = test_source_ref();
    assert!(sr.url.is_some());
    assert_eq!(sr.tier, SourceTier::tier2());
}

#[test]
fn test_source_ref_without_url() {
    let sr = SourceRef {
        url: None,
        description: "oral testimony".into(),
        tier: SourceTier::tier3(),
    };
    assert!(sr.url.is_none());
}

#[test]
fn test_verification_verified() {
    let v = Verification::Verified {
        confidence: 0.99,
        verified_at: now(),
        method: "cross-reference with court records".into(),
        corroborated_by: vec![test_source_ref()],
    };
    if let Verification::Verified {
        confidence,
        corroborated_by,
        ..
    } = &v
    {
        assert!(*confidence > 0.9);
        assert_eq!(corroborated_by.len(), 1);
    }
}

#[test]
fn test_verification_partial() {
    let v = Verification::Partial {
        confidence: 0.6,
        verified_at: now(),
        what_verified: "dates match".into(),
        what_remains: "amounts unverified".into(),
    };
    assert!(matches!(v, Verification::Partial { .. }));
}

#[test]
fn test_verification_unverified() {
    let v1 = Verification::Unverified { reason: None };
    let v2 = Verification::Unverified {
        reason: Some("source unavailable".into()),
    };
    assert!(matches!(v1, Verification::Unverified { reason: None }));
    assert!(matches!(v2, Verification::Unverified { reason: Some(_) }));
}

#[test]
fn test_verification_stale() {
    let verified_at = now() - chrono::Duration::days(365);
    let stale_since = now() - chrono::Duration::days(30);
    let v = Verification::Stale {
        was_verified_at: verified_at,
        stale_since,
    };
    if let Verification::Stale {
        was_verified_at,
        stale_since,
    } = &v
    {
        assert!(was_verified_at < stale_since);
    }
}

#[test]
fn test_document_type_court_docket() {
    let dt = DocumentType::CourtDocket {
        court: "S.D.N.Y.".into(),
        case_number: "1:24-cv-01234".into(),
        docket_id: Some("ECF 42".into()),
    };
    if let DocumentType::CourtDocket {
        court,
        case_number,
        docket_id,
    } = &dt
    {
        assert_eq!(court, "S.D.N.Y.");
        assert!(case_number.contains("cv"));
        assert!(docket_id.is_some());
    }
}

#[test]
fn test_document_type_press_article() {
    let dt = DocumentType::PressArticle {
        publication: "New York Times".into(),
        date: date(2024, 3, 15),
        author: Some("Jane Reporter".into()),
    };
    assert!(matches!(dt, DocumentType::PressArticle { .. }));
}

#[test]
fn test_document_type_legal_opinion() {
    let dt = DocumentType::LegalOpinion {
        court: "9th Circuit".into(),
        case_name: "Smith v. Jones".into(),
    };
    assert!(matches!(dt, DocumentType::LegalOpinion { .. }));
}

#[test]
fn test_document_type_exhibit() {
    let dt = DocumentType::Exhibit {
        label: "Exhibit A".into(),
        filed_with: "Complaint".into(),
        proves: vec!["damages".into(), "causation".into()],
    };
    if let DocumentType::Exhibit { proves, .. } = &dt {
        assert_eq!(proves.len(), 2);
    }
}

#[test]
fn test_document_type_declaration() {
    let dt = DocumentType::Declaration {
        declarant: "John Witness".into(),
        date: date(2024, 4, 1),
    };
    assert!(matches!(dt, DocumentType::Declaration { .. }));
}

#[test]
fn test_document_type_policy_report() {
    let dt = DocumentType::PolicyReport {
        organization: "GAO".into(),
        date: date(2024, 2, 28),
    };
    assert!(matches!(dt, DocumentType::PolicyReport { .. }));
}

#[test]
fn test_document_type_regulatory_filing() {
    let dt = DocumentType::RegulatoryFiling {
        agency: "SEC".into(),
        form_type: "10-K".into(),
        filed: date(2024, 3, 31),
    };
    assert!(matches!(dt, DocumentType::RegulatoryFiling { .. }));
}

#[test]
fn test_document_type_spreadsheet() {
    let dt = DocumentType::Spreadsheet {
        filename: "financials.xlsx".into(),
        sheet: "Q1 Revenue".into(),
    };
    assert!(matches!(dt, DocumentType::Spreadsheet { .. }));
}

#[test]
fn test_document_type_recording() {
    let dt = DocumentType::Recording {
        participants: vec!["CEO".into(), "CFO".into()],
        date: date(2024, 1, 10),
        transcribed: true,
    };
    if let DocumentType::Recording {
        participants,
        transcribed,
        ..
    } = &dt
    {
        assert_eq!(participants.len(), 2);
        assert!(*transcribed);
    }
}

#[test]
fn test_document_type_website() {
    let dt = DocumentType::Website {
        organization: "FDA".into(),
        page_type: "recall notice".into(),
    };
    assert!(matches!(dt, DocumentType::Website { .. }));
}

#[test]
fn test_archive() {
    let archive = Archive {
        text_path: Some("/archive/doc.txt".into()),
        image_path: Some("/archive/doc.png".into()),
        sha256: Some("abc123def456".into()),
        wayback_url: Some("https://web.archive.org/web/20240101/example.com".into()),
    };
    assert!(archive.text_path.is_some());
    assert!(archive.sha256.is_some());
}

#[test]
fn test_archive_minimal() {
    let archive = Archive {
        text_path: None,
        image_path: None,
        sha256: None,
        wayback_url: None,
    };
    assert!(archive.text_path.is_none());
}

#[test]
fn test_source_full() {
    let source = Source {
        url: Some("https://pacer.uscourts.gov/doc/123".into()),
        document: DocumentType::CourtDocket {
            court: "D.D.C.".into(),
            case_number: "1:24-cv-00001".into(),
            docket_id: None,
        },
        tier: SourceTier::tier1(),
        retrieved_at: now(),
        archive: Some(Archive {
            text_path: Some("/archive/docket.txt".into()),
            image_path: None,
            sha256: Some("deadbeef".into()),
            wayback_url: None,
        }),
        verification: Verification::Verified {
            confidence: 0.99,
            verified_at: now(),
            method: "PACER cross-check".into(),
            corroborated_by: vec![],
        },
    };
    assert_eq!(source.tier, SourceTier::tier1());
    assert!(source.archive.is_some());
}

// =============================================================================
// entity.rs tests
// =============================================================================

#[test]
fn test_entity_name_person() {
    let e = test_movant();
    assert_eq!(e.name(), "Plaintiff");
}

#[test]
fn test_entity_name_corporation() {
    let e = test_corporation();
    assert_eq!(e.name(), "Acme Corp");
}

#[test]
fn test_entity_name_law_firm() {
    let e = test_law_firm();
    assert_eq!(e.name(), "Smith & Associates");
}

#[test]
fn test_entity_name_agency() {
    let e = test_agency();
    assert_eq!(e.name(), "SEC");
}

#[test]
fn test_entity_name_court() {
    let e = test_court();
    assert_eq!(e.name(), "District Court");
}

#[test]
fn test_court_with_district_and_circuit() {
    let court = Entity::Court(Court {
        name: "U.S. District Court".into(),
        district: Some("Southern District of New York".into()),
        circuit: Some("Second Circuit".into()),
    });
    assert_eq!(court.name(), "U.S. District Court");
    if let Entity::Court(c) = &court {
        assert_eq!(c.district.as_deref(), Some("Southern District of New York"));
        assert_eq!(c.circuit.as_deref(), Some("Second Circuit"));
    }
}

#[test]
fn test_person_with_organization() {
    let firm = Entity::LawFirm(LawFirm {
        name: "Big Law LLP".into(),
        source: None,
    });
    let person = Entity::Person(Person {
        name: "Jane Attorney".into(),
        title: Some("Partner".into()),
        organization: Some(Box::new(firm)),
        bar_admissions: vec!["NY".into(), "CA".into()],
        source: None,
    });
    if let Entity::Person(p) = &person {
        assert_eq!(p.bar_admissions.len(), 2);
        assert!(p.organization.is_some());
        assert_eq!(p.organization.as_ref().unwrap().name(), "Big Law LLP");
    }
}

#[test]
fn test_corporate_structure_public() {
    let corp = Corporation {
        name: "Public Inc".into(),
        structure: CorporateStructure::PublicCompany,
        jurisdiction: "Delaware".into(),
        source: None,
    };
    assert!(matches!(corp.structure, CorporateStructure::PublicCompany));
}

#[test]
fn test_corporate_structure_subsidiary() {
    let parent = test_corporation();
    let sub = Corporation {
        name: "Sub Corp".into(),
        structure: CorporateStructure::Subsidiary {
            parent: Box::new(parent),
        },
        jurisdiction: "Delaware".into(),
        source: None,
    };
    if let CorporateStructure::Subsidiary { parent } = &sub.structure {
        assert_eq!(parent.name(), "Acme Corp");
    }
}

#[test]
fn test_corporate_structure_spinoff() {
    let parent = test_corporation();
    let spin = CorporateStructure::SpinOff {
        parent: Box::new(parent),
        date: date(2023, 6, 1),
    };
    assert!(matches!(spin, CorporateStructure::SpinOff { .. }));
}

#[test]
fn test_corporate_structure_joint_venture() {
    let jv = CorporateStructure::JointVenture {
        partners: vec![test_corporation(), test_corporation()],
    };
    if let CorporateStructure::JointVenture { partners } = &jv {
        assert_eq!(partners.len(), 2);
    }
}

#[test]
fn test_corporate_structure_partnership_and_private() {
    let _ = CorporateStructure::Partnership;
    let _ = CorporateStructure::Private;
}

#[test]
fn test_tenure() {
    let current = Tenure::Current {
        start: date(2020, 1, 1),
    };
    let former = Tenure::Former {
        start: date(2018, 1, 1),
        end: date(2022, 6, 30),
    };
    assert!(matches!(current, Tenure::Current { .. }));
    assert!(matches!(former, Tenure::Former { .. }));
}

#[test]
fn test_representation_status() {
    let current = RepresentationStatus::Current {
        since: Some(date(2023, 5, 1)),
    };
    let former = RepresentationStatus::Former {
        ended: Some(date(2024, 1, 15)),
    };
    assert!(matches!(current, RepresentationStatus::Current { .. }));
    assert!(matches!(former, RepresentationStatus::Former { .. }));
}

#[test]
fn test_relationship_corporate() {
    let rel = Relationship::Corporate {
        parent: test_corporation(),
        child: Entity::Corporation(Corporation {
            name: "Sub Corp".into(),
            structure: CorporateStructure::Subsidiary {
                parent: Box::new(test_corporation()),
            },
            jurisdiction: "Delaware".into(),
            source: None,
        }),
        source: test_source(),
    };
    if let Relationship::Corporate { parent, child, .. } = &rel {
        assert_eq!(parent.name(), "Acme Corp");
        assert_eq!(child.name(), "Sub Corp");
    }
}

#[test]
fn test_relationship_employment() {
    let rel = Relationship::Employment {
        person: test_movant(),
        organization: test_corporation(),
        role: "Senior Engineer".into(),
        tenure: Tenure::Former {
            start: date(2020, 1, 1),
            end: date(2024, 1, 15),
        },
        source: test_source(),
    };
    if let Relationship::Employment { role, tenure, .. } = &rel {
        assert_eq!(role, "Senior Engineer");
        assert!(matches!(tenure, Tenure::Former { .. }));
    }
}

#[test]
fn test_relationship_legal() {
    let rel = Relationship::Legal {
        counsel: test_law_firm(),
        client: test_movant(),
        matter: Some("Employment dispute".into()),
        status: RepresentationStatus::Current { since: None },
        source: test_source(),
    };
    if let Relationship::Legal { matter, status, .. } = &rel {
        assert!(matter.is_some());
        assert!(matches!(status, RepresentationStatus::Current { .. }));
    }
}

#[test]
fn test_relationship_supply_chain() {
    let rel = Relationship::SupplyChain {
        supplier: test_corporation(),
        customer: Entity::Corporation(Corporation {
            name: "Big Buyer Inc".into(),
            structure: CorporateStructure::PublicCompany,
            jurisdiction: "New York".into(),
            source: None,
        }),
        revenue_pct: Some(35.0),
        source: test_source(),
    };
    if let Relationship::SupplyChain { revenue_pct, .. } = &rel {
        assert_eq!(*revenue_pct, Some(35.0));
    }
}

// =============================================================================
// authority.rs tests
// =============================================================================

#[test]
fn test_authority_weight_all_variants() {
    let constitution = Authority::Constitution {
        provision: "1st Amendment".into(),
    };
    assert_eq!(constitution.weight(), 10);

    let supreme = Authority::SupremeCourt {
        case: test_case_law(),
        interprets: Box::new(constitution.clone()),
    };
    assert_eq!(supreme.weight(), 9);

    let legislature = Authority::Legislature {
        statute: test_statute(),
    };
    assert_eq!(legislature.weight(), 8);

    let appellate = Authority::AppellateCourt {
        jurisdiction: "9th Circuit".into(),
        case: test_case_law(),
        interprets: Box::new(legislature.clone()),
        precedent: None,
    };
    assert_eq!(appellate.weight(), 7);

    let regulation = Authority::Regulation {
        agency: test_agency(),
        regulation: test_statute(),
        implements: Box::new(legislature.clone()),
    };
    assert_eq!(regulation.weight(), 6);

    let agency_action = Authority::AgencyAction {
        agency: test_agency(),
        action_type: "enforcement action".into(),
        under: Box::new(regulation.clone()),
    };
    assert_eq!(agency_action.weight(), 5);

    let trial = Authority::TrialCourt {
        court: test_court(),
        case: test_case_law(),
    };
    assert_eq!(trial.weight(), 4);

    let professional = Authority::ProfessionalBody {
        body: Entity::Agency(Agency {
            name: "ABA".into(),
            jurisdiction: "National".into(),
        }),
        rule: test_statute(),
    };
    assert_eq!(professional.weight(), 3);
}

#[test]
fn test_authority_weight_decreasing() {
    let weights = [
        Authority::Constitution {
            provision: "".into(),
        }
        .weight(),
        Authority::SupremeCourt {
            case: test_case_law(),
            interprets: Box::new(Authority::Constitution {
                provision: "".into(),
            }),
        }
        .weight(),
        Authority::Legislature {
            statute: test_statute(),
        }
        .weight(),
        Authority::AppellateCourt {
            jurisdiction: "".into(),
            case: test_case_law(),
            interprets: Box::new(Authority::Constitution {
                provision: "".into(),
            }),
            precedent: None,
        }
        .weight(),
        Authority::Regulation {
            agency: test_agency(),
            regulation: test_statute(),
            implements: Box::new(Authority::Constitution {
                provision: "".into(),
            }),
        }
        .weight(),
        Authority::AgencyAction {
            agency: test_agency(),
            action_type: "".into(),
            under: Box::new(Authority::Constitution {
                provision: "".into(),
            }),
        }
        .weight(),
        Authority::TrialCourt {
            court: test_court(),
            case: test_case_law(),
        }
        .weight(),
        Authority::ProfessionalBody {
            body: test_agency(),
            rule: test_statute(),
        }
        .weight(),
    ];
    for w in weights.windows(2) {
        assert!(w[0] > w[1], "{} should be > {}", w[0], w[1]);
    }
}

#[test]
fn test_statute_construction() {
    let s = test_statute();
    assert!(s.title.is_some());
    assert!(s.effective_date.is_some());
    assert!(s.text.is_none());
}

#[test]
fn test_case_law_construction() {
    let cl = test_case_law();
    assert_eq!(cl.year, 1966);
    assert!(cl.quote.is_some());
}

#[test]
fn test_precedent() {
    let established = test_case_law();
    let follower = CaseLaw {
        name: "Dickerson v. United States".into(),
        citation: Citation::SupremeCourt {
            name: "Dickerson v. United States".into(),
            reporter_volume: 530,
            page: 428,
            year: 2000,
        },
        year: 2000,
        court: "Supreme Court".into(),
        holding: "Miranda is constitutional".into(),
        quote: None,
    };
    let precedent = Precedent {
        doctrine: "Miranda warnings".into(),
        established_by: established,
        followed_by: vec![follower],
        distinguished_by: vec![],
        overruled: None,
    };
    assert_eq!(precedent.doctrine, "Miranda warnings");
    assert_eq!(precedent.followed_by.len(), 1);
    assert!(precedent.distinguished_by.is_empty());
    assert!(precedent.overruled.is_none());
}

#[test]
fn test_citation_statute() {
    let c = Citation::Statute {
        code: "42 U.S.C.".into(),
        section: "1983".into(),
        subsection: Some("(a)".into()),
        short: Some("Section 1983".into()),
    };
    if let Citation::Statute {
        code, subsection, ..
    } = &c
    {
        assert_eq!(code, "42 U.S.C.");
        assert!(subsection.is_some());
    }
}

#[test]
fn test_citation_regulation() {
    let c = Citation::Regulation {
        code: "29 C.F.R.".into(),
        part: "1910".into(),
        section: "1200".into(),
    };
    assert!(matches!(c, Citation::Regulation { .. }));
}

#[test]
fn test_citation_case_law() {
    let c = Citation::CaseLaw {
        name: "Roe v. Wade".into(),
        reporter: "U.S.".into(),
        volume: 410,
        page: 113,
        court: "Supreme Court".into(),
        year: 1973,
    };
    if let Citation::CaseLaw { volume, page, .. } = &c {
        assert_eq!(*volume, 410);
        assert_eq!(*page, 113);
    }
}

#[test]
fn test_citation_supreme_court() {
    let c = Citation::SupremeCourt {
        name: "Brown v. Board of Education".into(),
        reporter_volume: 347,
        page: 483,
        year: 1954,
    };
    assert!(matches!(c, Citation::SupremeCourt { .. }));
}

#[test]
fn test_citation_professional_rule() {
    let c = Citation::ProfessionalRule {
        body: "ABA".into(),
        rule: "Model Rule 1.7".into(),
    };
    assert!(matches!(c, Citation::ProfessionalRule { .. }));
}

#[test]
fn test_citation_administrative_ruling() {
    let c = Citation::AdministrativeRuling {
        body: "NLRB".into(),
        date: date(2024, 3, 1),
        description: "unfair labor practice finding".into(),
    };
    assert!(matches!(c, Citation::AdministrativeRuling { .. }));
}

#[test]
fn test_binding_status() {
    assert_ne!(BindingStatus::Binding, BindingStatus::Persuasive);
}

#[test]
fn test_jurisdiction_variants() {
    let federal = Jurisdiction::Federal;
    let supreme = Jurisdiction::Supreme;
    let appellate = Jurisdiction::Appellate("9th Circuit".into());
    let state = Jurisdiction::State("California".into());
    let international = Jurisdiction::International;

    assert_eq!(federal, Jurisdiction::Federal);
    assert_eq!(supreme, Jurisdiction::Supreme);
    assert_ne!(appellate, state);
    assert_ne!(state, international);
}

#[test]
fn test_appellate_authority_with_precedent() {
    let established = test_case_law();
    let precedent = Precedent {
        doctrine: "Miranda warnings".into(),
        established_by: established.clone(),
        followed_by: vec![],
        distinguished_by: vec![],
        overruled: None,
    };
    let auth = Authority::AppellateCourt {
        jurisdiction: "9th Circuit".into(),
        case: test_case_law(),
        interprets: Box::new(Authority::Legislature {
            statute: test_statute(),
        }),
        precedent: Some(precedent),
    };
    if let Authority::AppellateCourt { precedent, .. } = &auth {
        assert!(precedent.is_some());
    }
}

// =============================================================================
// ontology.rs tests
// =============================================================================

#[test]
fn test_ontology_registry_get_category() {
    let mut registry = OntologyRegistry::new();
    let cat = LegalCategory {
        name: "employment".into(),
        description: "Employment law".into(),
        authority: test_authority(),
        terms: vec![],
        relations: vec![],
    };
    registry.register(cat);
    assert!(registry.get_category("employment").is_some());
    assert!(registry.get_category("nonexistent").is_none());
}

#[test]
fn test_ontology_registry_default() {
    let registry = OntologyRegistry::default();
    assert!(registry.categories.is_empty());
}

#[test]
fn test_ontology_registry_multiple_categories() {
    let mut registry = OntologyRegistry::new();
    registry.register(LegalCategory {
        name: "employment".into(),
        description: "Employment law".into(),
        authority: test_authority(),
        terms: vec![LegalTerm {
            id: "emp:retaliation".into(),
            name: "Retaliation".into(),
            definition: "Adverse action for protected activity".into(),
            source_text: None,
            valence: Valence::Supportive,
            subsection: None,
            required_evidence: vec![],
            obligations: vec![],
            deadlines: vec![],
            rights: vec![],
            remedies: vec![],
            burdens: vec![],
            exceptions: vec![],
        }],
        relations: vec![],
    });
    registry.register(LegalCategory {
        name: "securities".into(),
        description: "Securities law".into(),
        authority: test_authority(),
        terms: vec![LegalTerm {
            id: "sec:insider_trading".into(),
            name: "Insider Trading".into(),
            definition: "Trading on material nonpublic information".into(),
            source_text: None,
            valence: Valence::Supportive,
            subsection: None,
            required_evidence: vec![],
            obligations: vec![],
            deadlines: vec![],
            rights: vec![],
            remedies: vec![],
            burdens: vec![],
            exceptions: vec![],
        }],
        relations: vec![],
    });

    assert_eq!(registry.categories.len(), 2);
    assert!(registry.get_term("emp:retaliation").is_some());
    assert!(registry.get_term("sec:insider_trading").is_some());
    assert!(registry.get_term("nonexistent").is_none());
}

#[test]
fn test_legal_term_full() {
    let term = LegalTerm {
        id: "ret:protected_activity".into(),
        name: "Protected Activity".into(),
        definition: "Activity protected by anti-retaliation statutes".into(),
        source_text: Some("See 42 U.S.C. 2000e-3(a)".into()),
        valence: Valence::Supportive,
        subsection: Some("(a)".into()),
        required_evidence: vec![EvidenceRequirement {
            field: "complaint_date".into(),
            field_type: EvidenceType::Date,
            required: RequirementLevel::Required,
            description: Some("date complaint was filed".into()),
        }],
        obligations: vec![Obligation {
            actor: "employer".into(),
            action: "preserve records".into(),
            language: ObligationLanguage::Mandatory {
                word: "shall".into(),
            },
            source_text: "Employer shall preserve...".into(),
        }],
        deadlines: vec![Deadline {
            duration: DeadlineDuration::Days(180),
            trigger: "discriminatory act".into(),
            consequence: Some("claim barred".into()),
            source_text: "within 180 days of the alleged discriminatory act".into(),
        }],
        rights: vec!["right to file charge".into()],
        remedies: vec![Remedy {
            name: "reinstatement".into(),
            description: "restoration to former position".into(),
            source_text: "The court may order reinstatement...".into(),
        }],
        burdens: vec![BurdenOfProof {
            standard: ProofStandard::Preponderance,
            borne_by: "plaintiff".into(),
            source_text: "Plaintiff bears the burden...".into(),
        }],
        exceptions: vec![Exception {
            to_rule: "anti-retaliation".into(),
            exception: "legitimate business reason".into(),
            source_text: "unless the employer demonstrates...".into(),
        }],
    };
    assert_eq!(term.id, "ret:protected_activity");
    assert_eq!(term.valence, Valence::Supportive);
    assert_eq!(term.required_evidence.len(), 1);
    assert_eq!(term.obligations.len(), 1);
    assert_eq!(term.deadlines.len(), 1);
    assert_eq!(term.rights.len(), 1);
    assert_eq!(term.remedies.len(), 1);
    assert_eq!(term.burdens.len(), 1);
    assert_eq!(term.exceptions.len(), 1);
}

#[test]
fn test_valence_variants() {
    assert_ne!(Valence::Supportive, Valence::Defensive);
    assert_ne!(Valence::Defensive, Valence::Procedural);
    assert_ne!(Valence::Supportive, Valence::Procedural);
}

#[test]
fn test_proof_standard_variants() {
    assert_ne!(
        ProofStandard::Preponderance,
        ProofStandard::ClearAndConvincing
    );
    assert_ne!(
        ProofStandard::ClearAndConvincing,
        ProofStandard::BeyondReasonableDoubt
    );
}

#[test]
fn test_obligation_language_variants() {
    let mandatory = ObligationLanguage::Mandatory {
        word: "shall".into(),
    };
    let discretionary = ObligationLanguage::Discretionary { word: "may".into() };
    let prohibitive = ObligationLanguage::Prohibitive {
        word: "shall not".into(),
    };
    assert_ne!(mandatory, discretionary);
    assert_ne!(discretionary, prohibitive);
}

#[test]
fn test_deadline_duration_variants() {
    let days = DeadlineDuration::Days(30);
    let months = DeadlineDuration::Months(6);
    let immediate = DeadlineDuration::Immediate;
    assert_ne!(days, months);
    assert_ne!(months, immediate);
}

#[test]
fn test_evidence_type_variants() {
    let types = [
        EvidenceType::Date,
        EvidenceType::Entity,
        EvidenceType::Document,
        EvidenceType::Currency,
        EvidenceType::Duration,
        EvidenceType::Narrative,
        EvidenceType::Count,
        EvidenceType::Text,
    ];
    // All distinct
    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(types[i], types[j]);
        }
    }
}

#[test]
fn test_requirement_level_variants() {
    assert_ne!(RequirementLevel::Required, RequirementLevel::Recommended);
    assert_ne!(RequirementLevel::Recommended, RequirementLevel::Optional);
}

#[test]
fn test_validation_completeness_variants() {
    let complete = ValidationCompleteness::Complete;
    let sufficient = ValidationCompleteness::Sufficient;
    let insufficient = ValidationCompleteness::Insufficient {
        missing_required: vec!["date".into(), "entity".into()],
    };
    assert_ne!(complete, sufficient);
    if let ValidationCompleteness::Insufficient { missing_required } = &insufficient {
        assert_eq!(missing_required.len(), 2);
    }
}

#[test]
fn test_legal_relation() {
    let rel = LegalRelation {
        from: "retaliation:protected_activity".into(),
        to: "retaliation:adverse_action".into(),
        relation: RelationType::Precedes {
            max_days: Some(180),
        },
    };
    if let RelationType::Precedes { max_days } = &rel.relation {
        assert_eq!(*max_days, Some(180));
    }
}

#[test]
fn test_relation_type_variants() {
    let _ = RelationType::Requires;
    let _ = RelationType::Precedes { max_days: None };
    let _ = RelationType::Implies {
        consequence: "liability".into(),
    };
    let _ = RelationType::Contradicts;
    let _ = RelationType::Composes {
        into: "claim".into(),
    };
    let _ = RelationType::SubtypeOf;
    let _ = RelationType::Triggers {
        obligation: "disclosure".into(),
    };
    let _ = RelationType::Negates;
    let _ = RelationType::AlternativeTo;
    let _ = RelationType::Rebuts {
        burden: "causation".into(),
    };
    let _ = RelationType::AffirmativeDefenseTo;
    let _ = RelationType::SafeHarborFor;
    let _ = RelationType::ExhaustionRequiredFor;
}

#[test]
fn test_legal_category_with_relations() {
    let cat = LegalCategory {
        name: "retaliation".into(),
        description: "Anti-retaliation framework".into(),
        authority: test_authority(),
        terms: vec![
            LegalTerm {
                id: "ret:pa".into(),
                name: "Protected Activity".into(),
                definition: "".into(),
                source_text: None,
                valence: Valence::Supportive,
                subsection: None,
                required_evidence: vec![],
                obligations: vec![],
                deadlines: vec![],
                rights: vec![],
                remedies: vec![],
                burdens: vec![],
                exceptions: vec![],
            },
            LegalTerm {
                id: "ret:aa".into(),
                name: "Adverse Action".into(),
                definition: "".into(),
                source_text: None,
                valence: Valence::Supportive,
                subsection: None,
                required_evidence: vec![],
                obligations: vec![],
                deadlines: vec![],
                rights: vec![],
                remedies: vec![],
                burdens: vec![],
                exceptions: vec![],
            },
        ],
        relations: vec![LegalRelation {
            from: "ret:pa".into(),
            to: "ret:aa".into(),
            relation: RelationType::Precedes { max_days: None },
        }],
    };
    assert_eq!(cat.terms.len(), 2);
    assert_eq!(cat.relations.len(), 1);
}

// =============================================================================
// ontology.rs — Category trait, Quality, Axiom tests
// =============================================================================

#[test]
fn test_phase_tag_entity_variants() {
    let variants = <PhaseTag as CategoryEntity>::variants();
    assert_eq!(variants.len(), 9);
    assert!(variants.contains(&PhaseTag::PreFiling));
    assert!(variants.contains(&PhaseTag::Closed));
}

#[test]
fn test_phase_transition_rel_source_target() {
    use praxis::category::Relationship;
    let rel = PhaseTransitionRel {
        from: PhaseTag::Filed,
        to: PhaseTag::Discovery,
    };
    assert_eq!(rel.source(), PhaseTag::Filed);
    assert_eq!(rel.target(), PhaseTag::Discovery);
}

#[test]
fn test_case_lifecycle_category_identity() {
    let id = CaseLifecycleCategory::identity(&PhaseTag::Discovery);
    assert_eq!(id.from, PhaseTag::Discovery);
    assert_eq!(id.to, PhaseTag::Discovery);
}

#[test]
fn test_case_lifecycle_category_compose_valid() {
    let f = PhaseTransitionRel {
        from: PhaseTag::PreFiling,
        to: PhaseTag::Filed,
    };
    let g = PhaseTransitionRel {
        from: PhaseTag::Filed,
        to: PhaseTag::Discovery,
    };
    let composed = CaseLifecycleCategory::compose(&f, &g);
    assert!(composed.is_some());
    let c = composed.unwrap();
    assert_eq!(c.from, PhaseTag::PreFiling);
    assert_eq!(c.to, PhaseTag::Discovery);
}

#[test]
fn test_case_lifecycle_category_compose_invalid() {
    let f = PhaseTransitionRel {
        from: PhaseTag::PreFiling,
        to: PhaseTag::Filed,
    };
    let g = PhaseTransitionRel {
        from: PhaseTag::Discovery,
        to: PhaseTag::Motions,
    };
    // f.to (Filed) != g.from (Discovery)
    let composed = CaseLifecycleCategory::compose(&f, &g);
    assert!(composed.is_none());
}

#[test]
fn test_case_lifecycle_category_morphisms_include_identities() {
    let morphisms = CaseLifecycleCategory::morphisms();
    // Every phase should have an identity morphism
    for phase in <PhaseTag as CategoryEntity>::variants() {
        assert!(morphisms.contains(&PhaseTransitionRel {
            from: phase,
            to: phase,
        }));
    }
}

#[test]
fn test_case_lifecycle_category_morphisms_include_direct_transitions() {
    let morphisms = CaseLifecycleCategory::morphisms();
    // PreFiling -> Filed should be present
    assert!(morphisms.contains(&PhaseTransitionRel {
        from: PhaseTag::PreFiling,
        to: PhaseTag::Filed,
    }));
    // Filed -> Discovery should be present
    assert!(morphisms.contains(&PhaseTransitionRel {
        from: PhaseTag::Filed,
        to: PhaseTag::Discovery,
    }));
}

#[test]
fn test_case_lifecycle_category_morphisms_include_composites() {
    let morphisms = CaseLifecycleCategory::morphisms();
    // PreFiling -> Filed -> Discovery should compose to PreFiling -> Discovery
    assert!(morphisms.contains(&PhaseTransitionRel {
        from: PhaseTag::PreFiling,
        to: PhaseTag::Discovery,
    }));
}

#[test]
fn test_is_terminal_phase_quality() {
    let q = IsTerminalPhase;
    assert!(q.get(&PhaseTag::Closed).is_some());
    assert!(q.get(&PhaseTag::PreFiling).is_none());
    assert!(q.get(&PhaseTag::Filed).is_none());
    assert!(q.get(&PhaseTag::Trial).is_none());
}

#[test]
fn test_only_closed_is_terminal_axiom() {
    let axiom = OnlyClosedIsTerminal;
    assert!(axiom.holds());
    assert_eq!(axiom.description(), "only Closed is a terminal phase");
}

#[test]
fn test_no_dead_phases_axiom() {
    let axiom = NoDeadPhases;
    assert!(axiom.holds());
    assert_eq!(
        axiom.description(),
        "every non-terminal phase has transitions"
    );
}

// =============================================================================
// engine.rs tests
// =============================================================================

#[test]
fn test_case_situation_describe() {
    let case = Case::new("Smith v. Corp");
    let desc = case.describe();
    assert!(desc.contains("Smith v. Corp"));
    assert!(desc.contains("PreFiling"));
}

#[test]
fn test_case_situation_is_terminal() {
    let case = Case::new("Test");
    assert!(!case.is_terminal());

    let mut closed_case = Case::new("Test");
    let _ = closed_case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    let _ = closed_case.act(CaseAction::Settle {
        terms: "terms".into(),
        date: date(2024, 2, 1),
    });
    assert!(closed_case.is_terminal());
}

#[test]
fn test_legal_action_describe_variants() {
    let file_action = LegalAction(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    assert!(file_action.describe().contains("file case"));

    let discovery = LegalAction(CaseAction::BeginDiscovery {
        date: date(2024, 2, 1),
    });
    assert!(discovery.describe().contains("discovery"));

    let motion = LegalAction(CaseAction::FileMotion {
        motion: test_pending_motion(),
        date: date(2024, 3, 1),
    });
    assert!(motion.describe().contains("file motion"));

    let rule_motion = LegalAction(CaseAction::RuleOnMotion {
        motion_index: 0,
        action: MotionAction::Grant {
            date: date(2024, 5, 1),
            judge: test_judge(),
            order: "granted".into(),
        },
        date: date(2024, 5, 1),
    });
    assert!(rule_motion.describe().contains("rule on motion"));

    let set_trial = LegalAction(CaseAction::SetForTrial {
        date: date(2024, 12, 1),
    });
    assert!(set_trial.describe().contains("trial"));

    let begin_trial = LegalAction(CaseAction::BeginTrial {
        date: date(2024, 12, 1),
    });
    assert!(begin_trial.describe().contains("trial"));

    let verdict = LegalAction(CaseAction::Verdict {
        outcome: "plaintiff wins".into(),
        date: date(2024, 12, 15),
    });
    assert!(verdict.describe().contains("verdict"));

    let appeal = LegalAction(CaseAction::Appeal {
        court: test_court(),
        date: date(2025, 1, 15),
    });
    assert!(appeal.describe().contains("appeal"));

    let settle = LegalAction(CaseAction::Settle {
        terms: "confidential".into(),
        date: date(2024, 6, 1),
    });
    assert!(settle.describe().contains("settle"));

    let dismiss = LegalAction(CaseAction::Dismiss {
        reason: "moot".into(),
        with_prejudice: false,
        date: date(2024, 6, 1),
    });
    assert!(dismiss.describe().contains("dismiss"));
}

#[test]
fn test_phase_transition_precondition_on_closed() {
    let precond = PhaseTransition;

    let mut case = Case::new("Test");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    let _ = case.act(CaseAction::Settle {
        terms: "terms".into(),
        date: date(2024, 2, 1),
    });

    let action = LegalAction(CaseAction::BeginDiscovery {
        date: date(2024, 3, 1),
    });
    let result = precond.check(&case, &action);
    assert!(!result.is_satisfied());
}

#[test]
fn test_phase_transition_precondition_wrong_phase_for_file() {
    let precond = PhaseTransition;

    let mut case = Case::new("Test");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });

    // Trying to file again should violate
    let action = LegalAction(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 2),
    });
    let result = precond.check(&case, &action);
    assert!(!result.is_satisfied());
}

#[test]
fn test_phase_transition_precondition_valid_action() {
    let precond = PhaseTransition;

    let mut case = Case::new("Test");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });

    let action = LegalAction(CaseAction::BeginDiscovery {
        date: date(2024, 2, 1),
    });
    let result = precond.check(&case, &action);
    assert!(result.is_satisfied());
}

#[test]
fn test_phase_transition_precondition_invalid_transition() {
    let precond = PhaseTransition;

    // PreFiling -> can only go to Filed, not Trial
    let case = Case::new("Test");
    let action = LegalAction(CaseAction::BeginTrial {
        date: date(2024, 12, 1),
    });
    let result = precond.check(&case, &action);
    assert!(!result.is_satisfied());
}

#[test]
fn test_phase_transition_file_motion_in_wrong_phase() {
    let precond = PhaseTransition;

    // PreFiling -> can't file motion
    let case = Case::new("Test");
    let action = LegalAction(CaseAction::FileMotion {
        motion: test_pending_motion(),
        date: date(2024, 3, 1),
    });
    let result = precond.check(&case, &action);
    assert!(!result.is_satisfied());
}

#[test]
fn test_phase_transition_file_motion_in_valid_phase() {
    let precond = PhaseTransition;

    let mut case = Case::new("Test");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });

    let action = LegalAction(CaseAction::FileMotion {
        motion: test_pending_motion(),
        date: date(2024, 3, 1),
    });
    let result = precond.check(&case, &action);
    assert!(result.is_satisfied());
}

#[test]
fn test_phase_transition_describe() {
    let precond = PhaseTransition;
    assert_eq!(
        precond.describe(),
        "action must be valid for the current case phase"
    );
}

#[test]
fn test_new_case_engine() {
    let engine = new_case("Test v. Corp");
    assert_eq!(engine.situation().phase.tag(), PhaseTag::PreFiling);
    assert!(!engine.is_terminal());
    assert_eq!(engine.step(), 0);
}

#[test]
fn test_engine_next_valid() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    assert_eq!(engine.situation().phase.tag(), PhaseTag::Filed);
    assert_eq!(engine.step(), 1);
}

#[test]
fn test_engine_next_invalid() {
    let engine = new_case("Test v. Corp");
    // Try to begin trial from PreFiling
    let result = engine.next(LegalAction(CaseAction::BeginTrial {
        date: date(2024, 12, 1),
    }));
    assert!(result.is_err());
    if let Err(EngineError::Violated { engine, violations }) = result {
        assert!(!violations.is_empty());
        assert_eq!(engine.step(), 0); // unchanged
    }
}

#[test]
fn test_engine_back_and_forward() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    assert_eq!(engine.situation().phase.tag(), PhaseTag::Filed);

    let engine = engine
        .next(LegalAction(CaseAction::BeginDiscovery {
            date: date(2024, 2, 1),
        }))
        .unwrap();
    assert_eq!(engine.situation().phase.tag(), PhaseTag::Discovery);
    assert_eq!(engine.step(), 2);

    // Go back
    let engine = engine.back().unwrap();
    assert_eq!(engine.situation().phase.tag(), PhaseTag::Filed);
    assert_eq!(engine.step(), 1);
    assert_eq!(engine.forward_depth(), 1);

    // Go forward
    let engine = engine.forward().unwrap();
    assert_eq!(engine.situation().phase.tag(), PhaseTag::Discovery);
    assert_eq!(engine.step(), 2);
}

#[test]
fn test_engine_back_at_beginning() {
    let engine = new_case("Test v. Corp");
    let result = engine.back();
    assert!(result.is_err());
}

#[test]
fn test_engine_forward_without_back() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    let result = engine.forward();
    assert!(result.is_err());
}

#[test]
fn test_engine_trace() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();

    let trace = engine.trace();
    assert_eq!(trace.successful_steps(), 1);
    assert_eq!(trace.violations(), 0);
    let last = trace.last().unwrap();
    assert!(last.success);
    assert!(last.situation_after.is_some());
}

#[test]
fn test_engine_trace_with_violation() {
    let engine = new_case("Test v. Corp");
    // Invalid action
    let result = engine.next(LegalAction(CaseAction::BeginTrial {
        date: date(2024, 12, 1),
    }));
    let EngineError::Violated { engine, .. } = result.unwrap_err() else {
        panic!("expected Violated")
    };

    let trace = engine.trace();
    assert_eq!(trace.violations(), 1);
    assert_eq!(trace.successful_steps(), 0);
    let violation_entries = trace.violation_entries();
    assert_eq!(violation_entries.len(), 1);
    assert!(!violation_entries[0].success);
    assert!(violation_entries[0].situation_after.is_none());
}

#[test]
fn test_engine_trace_dump() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();

    let dump = engine.trace().dump();
    assert!(dump.contains("OK"));
    assert!(dump.contains("file case"));
}

#[test]
fn test_engine_try_next_valid() {
    let engine = new_case("Test v. Corp");
    let result = engine.try_next(LegalAction(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    }));
    assert!(result.is_ok());
}

#[test]
fn test_engine_try_next_invalid() {
    let engine = new_case("Test v. Corp");
    let result = engine.try_next(LegalAction(CaseAction::BeginTrial {
        date: date(2024, 12, 1),
    }));
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].contains("phase_transition"));
}

#[test]
fn test_engine_full_lifecycle() {
    let engine = new_case("Smith v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::BeginDiscovery {
            date: date(2024, 2, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::SetForTrial {
            date: date(2024, 10, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::BeginTrial {
            date: date(2024, 10, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::Verdict {
            outcome: "plaintiff".into(),
            date: date(2024, 10, 15),
        }))
        .unwrap();

    assert_eq!(engine.situation().phase.tag(), PhaseTag::PostTrial);
    assert_eq!(engine.step(), 5);
    assert_eq!(engine.trace().successful_steps(), 5);
    assert!(!engine.is_terminal());
}

#[test]
fn test_engine_settlement_is_terminal() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::Settle {
            terms: "$1M settlement".into(),
            date: date(2024, 6, 1),
        }))
        .unwrap();

    assert!(engine.is_terminal());
    assert_eq!(engine.situation().phase.tag(), PhaseTag::Closed);
}

#[test]
fn test_engine_no_action_after_terminal() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::Settle {
            terms: "terms".into(),
            date: date(2024, 6, 1),
        }))
        .unwrap();

    // Trying any action on a closed case should fail
    let result = engine.try_next(LegalAction(CaseAction::BeginDiscovery {
        date: date(2024, 7, 1),
    }));
    assert!(result.is_err());
}

#[test]
fn test_engine_next_clears_future() {
    let engine = new_case("Test v. Corp");
    let engine = engine
        .next(LegalAction(CaseAction::File {
            court: test_court(),
            date: date(2024, 1, 1),
        }))
        .unwrap();
    let engine = engine
        .next(LegalAction(CaseAction::BeginDiscovery {
            date: date(2024, 2, 1),
        }))
        .unwrap();

    // Go back
    let engine = engine.back().unwrap();
    assert_eq!(engine.forward_depth(), 1);

    // Do a new action (should clear future)
    let engine = engine
        .next(LegalAction(CaseAction::Settle {
            terms: "early settlement".into(),
            date: date(2024, 3, 1),
        }))
        .unwrap();
    assert_eq!(engine.forward_depth(), 0);
}

// =============================================================================
// Additional lifecycle / decision edge case tests
// =============================================================================

#[test]
fn test_motion_deny_from_under_advisement() {
    let motion = test_pending_motion();
    let motion = motion
        .act(MotionAction::Oppose {
            date: date(2024, 3, 15),
            by: test_respondent(),
        })
        .unwrap();
    let motion = motion
        .act(MotionAction::TakeUnderAdvisement {
            date: date(2024, 4, 1),
        })
        .unwrap();
    let motion = motion
        .act(MotionAction::Deny {
            date: date(2024, 5, 1),
            judge: test_judge(),
            reason: "plaintiff fails to show likelihood of success".into(),
        })
        .unwrap();
    assert_eq!(motion.status.tag(), StatusTag::Denied);
    assert!(motion.status.is_terminal());
}

#[test]
fn test_motion_grant_in_part() {
    let motion = test_pending_motion();
    let motion = motion
        .act(MotionAction::Oppose {
            date: date(2024, 3, 15),
            by: test_respondent(),
        })
        .unwrap();
    let motion = motion
        .act(MotionAction::TakeUnderAdvisement {
            date: date(2024, 4, 1),
        })
        .unwrap();
    let motion = motion
        .act(MotionAction::GrantInPart {
            date: date(2024, 5, 1),
            judge: test_judge(),
            granted: "count 1 dismissed".into(),
            denied: "counts 2-3 survive".into(),
        })
        .unwrap();
    assert_eq!(motion.status.tag(), StatusTag::GrantedInPart);
    assert!(motion.status.is_terminal());
}

#[test]
fn test_motion_declare_moot_from_pending() {
    let motion = test_pending_motion();
    let motion = motion
        .act(MotionAction::DeclareMoot {
            date: date(2024, 4, 1),
            reason: "case settled".into(),
        })
        .unwrap();
    assert_eq!(motion.status.tag(), StatusTag::Moot);
    assert!(motion.status.is_terminal());
}

#[test]
fn test_motion_withdraw_from_pending() {
    let motion = test_pending_motion();
    let motion = motion
        .act(MotionAction::Withdraw {
            date: date(2024, 3, 20),
            reason: Some("strategic withdrawal".into()),
        })
        .unwrap();
    assert_eq!(motion.status.tag(), StatusTag::Withdrawn);
    assert!(motion.status.is_terminal());
}

#[test]
fn test_motion_withdraw_without_reason() {
    let motion = test_pending_motion();
    let motion = motion
        .act(MotionAction::Withdraw {
            date: date(2024, 3, 20),
            reason: None,
        })
        .unwrap();
    if let MotionStatus::Withdrawn { reason, .. } = &motion.status {
        assert!(reason.is_none());
    }
}

#[test]
fn test_motion_type_variants() {
    let _ = MotionType::PreliminaryInjunction {
        irreparable_harm: "ongoing harm".into(),
    };
    let _ = MotionType::Disqualification {
        conflict: "prior representation".into(),
    };
    let _ = MotionType::SummaryJudgment {
        undisputed_facts: vec!["fact1".into(), "fact2".into()],
    };
    let _ = MotionType::DiscoveryMotion {
        scope: "emails from 2023".into(),
    };
    let _ = MotionType::ProtectiveOrder {
        protecting: "trade secrets".into(),
    };
    let _ = MotionType::MotionInLimine {
        exclude: "hearsay testimony".into(),
    };
    let _ = MotionType::MotionToCompel {
        compelling: "document production".into(),
    };
    let _ = MotionType::MotionForSanctions {
        basis: "spoliation".into(),
    };
}

#[test]
fn test_case_appeal_lifecycle() {
    let mut case = Case::new("Test v. Corp");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    let _ = case.act(CaseAction::BeginDiscovery {
        date: date(2024, 2, 1),
    });
    let _ = case.act(CaseAction::SetForTrial {
        date: date(2024, 10, 1),
    });
    let _ = case.act(CaseAction::BeginTrial {
        date: date(2024, 10, 1),
    });
    let _ = case.act(CaseAction::Verdict {
        outcome: "defendant".into(),
        date: date(2024, 10, 15),
    });
    assert_eq!(case.phase.tag(), PhaseTag::PostTrial);

    let _ = case.act(CaseAction::Appeal {
        court: Entity::Court(Court {
            name: "9th Circuit".into(),
            district: None,
            circuit: Some("9th".into()),
        }),
        date: date(2024, 11, 1),
    });
    assert_eq!(case.phase.tag(), PhaseTag::Appeal);
}

#[test]
fn test_case_events_are_recorded() {
    let mut case = Case::new("Test");
    assert!(case.events.is_empty());

    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    assert_eq!(case.events.len(), 1);
    assert_eq!(case.events[0].phase_tag, PhaseTag::Filed);

    let _ = case.act(CaseAction::BeginDiscovery {
        date: date(2024, 2, 1),
    });
    assert_eq!(case.events.len(), 2);
}

#[test]
fn test_action_result_variants() {
    let ok = ActionResult::Ok {
        description: "filed".into(),
    };
    assert!(ok.is_ok());

    let invalid = ActionResult::InvalidTransition {
        from: PhaseTag::PreFiling,
        action: "begin trial".into(),
    };
    assert!(!invalid.is_ok());

    let not_found = ActionResult::MotionNotFound { index: 99 };
    assert!(!not_found.is_ok());

    let error = ActionResult::MotionError {
        message: "invalid transition".into(),
    };
    assert!(!error.is_ok());
}

#[test]
fn test_close_reason_variants() {
    let settlement = CloseReason::Settlement {
        terms: "terms".into(),
    };
    assert!(matches!(settlement, CloseReason::Settlement { .. }));

    let dismissal = CloseReason::Dismissal {
        reason: "moot".into(),
        with_prejudice: false,
    };
    assert!(matches!(dismissal, CloseReason::Dismissal { .. }));

    let verdict = CloseReason::Verdict {
        outcome: "plaintiff".into(),
    };
    assert!(matches!(verdict, CloseReason::Verdict { .. }));

    let voluntary = CloseReason::Voluntary;
    assert!(matches!(voluntary, CloseReason::Voluntary));
}

#[test]
fn test_impact_variants() {
    let positive = Impact::Positive {
        detail: "favorable ruling".into(),
    };
    let minimal = Impact::Minimal {
        detail: "no change".into(),
    };
    let warning = Impact::Warning {
        detail: "potential exposure".into(),
    };
    let critical = Impact::Critical {
        detail: "case-ending".into(),
    };
    assert!(matches!(positive, Impact::Positive { .. }));
    assert!(matches!(minimal, Impact::Minimal { .. }));
    assert!(matches!(warning, Impact::Warning { .. }));
    assert!(matches!(critical, Impact::Critical { .. }));
}

#[test]
fn test_stakeholder_and_risk_comparison() {
    let stakeholder = Stakeholder {
        name: "Plaintiff".into(),
        role: "claimant".into(),
        if_granted: vec![Impact::Positive {
            detail: "claim proceeds".into(),
        }],
        if_denied: vec![Impact::Critical {
            detail: "claim dismissed".into(),
        }],
    };
    let risk = RiskComparison {
        stakeholders: vec![stakeholder],
        summary_if_granted: "case proceeds to trial".into(),
        summary_if_denied: "case dismissed with prejudice".into(),
    };
    assert_eq!(risk.stakeholders.len(), 1);
    assert_eq!(risk.stakeholders[0].if_granted.len(), 1);
    assert_eq!(risk.stakeholders[0].if_denied.len(), 1);
}

#[test]
fn test_case_file_motion_and_rule_on_it() {
    let mut case = Case::new("Test");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    let _ = case.act(CaseAction::FileMotion {
        motion: test_pending_motion(),
        date: date(2024, 2, 1),
    });
    assert_eq!(case.motions.len(), 1);
    assert_eq!(case.phase.tag(), PhaseTag::Motions);

    let result = case.act(CaseAction::RuleOnMotion {
        motion_index: 0,
        action: MotionAction::Oppose {
            date: date(2024, 3, 15),
            by: test_respondent(),
        },
        date: date(2024, 3, 15),
    });
    assert!(result.is_ok());
    assert_eq!(case.motions[0].status.tag(), StatusTag::Opposed);
}

#[test]
fn test_case_rule_on_motion_not_found() {
    let mut case = Case::new("Test");
    let _ = case.act(CaseAction::File {
        court: test_court(),
        date: date(2024, 1, 1),
    });
    let result = case.act(CaseAction::RuleOnMotion {
        motion_index: 99,
        action: MotionAction::Oppose {
            date: date(2024, 3, 15),
            by: test_respondent(),
        },
        date: date(2024, 3, 15),
    });
    assert!(!result.is_ok());
    assert!(matches!(result, ActionResult::MotionNotFound { index: 99 }));
}

// =============================================================================
// Property-based tests for new modules
// =============================================================================

proptest! {
    /// Authority weights are always in range 3..=10
    #[test]
    fn prop_authority_weight_bounded(idx in 0..8usize) {
        let authorities = [
            Authority::Constitution { provision: "".into() },
            Authority::SupremeCourt { case: test_case_law(), interprets: Box::new(Authority::Constitution { provision: "".into() }) },
            Authority::Legislature { statute: test_statute() },
            Authority::AppellateCourt { jurisdiction: "".into(), case: test_case_law(), interprets: Box::new(Authority::Constitution { provision: "".into() }), precedent: None },
            Authority::Regulation { agency: test_agency(), regulation: test_statute(), implements: Box::new(Authority::Constitution { provision: "".into() }) },
            Authority::AgencyAction { agency: test_agency(), action_type: "".into(), under: Box::new(Authority::Constitution { provision: "".into() }) },
            Authority::TrialCourt { court: test_court(), case: test_case_law() },
            Authority::ProfessionalBody { body: test_agency(), rule: test_statute() },
        ];
        let w = authorities[idx].weight();
        prop_assert!((3..=10).contains(&w));
    }

    /// Entity::name() always returns a non-empty string
    #[test]
    fn prop_entity_name_nonempty(idx in 0..5usize) {
        let entities = [
            test_movant(),
            test_corporation(),
            test_law_firm(),
            test_agency(),
            test_court(),
        ];
        prop_assert!(!entities[idx].name().is_empty());
    }

    /// SourceTier ordering is consistent
    #[test]
    fn prop_source_tier_consistent(a in 1..5u8, b in 1..5u8) {
        if a == b {
            prop_assert_eq!(SourceTier(a), SourceTier(b));
        } else if a < b {
            prop_assert!(SourceTier(a) < SourceTier(b));
        } else {
            prop_assert!(SourceTier(a) > SourceTier(b));
        }
    }

    /// Category identity law: compose(id, f) == f
    #[test]
    fn prop_category_left_identity(from_idx in 0..8usize, to_idx in 0..9usize) {
        let phases = <PhaseTag as CategoryEntity>::variants();
        let from = phases[from_idx];
        let to = phases[to_idx];
        let f = PhaseTransitionRel { from, to };
        let id = CaseLifecycleCategory::identity(&from);
        let composed = CaseLifecycleCategory::compose(&id, &f);
        prop_assert_eq!(composed, Some(f.clone()));
    }

    /// Category identity law: compose(f, id) == f
    #[test]
    fn prop_category_right_identity(from_idx in 0..9usize, to_idx in 0..9usize) {
        let phases = <PhaseTag as CategoryEntity>::variants();
        let from = phases[from_idx];
        let to = phases[to_idx];
        let f = PhaseTransitionRel { from, to };
        let id = CaseLifecycleCategory::identity(&to);
        let composed = CaseLifecycleCategory::compose(&f, &id);
        prop_assert_eq!(composed, Some(f.clone()));
    }

    /// Recommendation equality is reflexive
    #[test]
    fn prop_recommendation_eq(idx in 0..6usize) {
        let recs = [
            Recommendation::Investigate,
            Recommendation::Disclose,
            Recommendation::Monitor,
            Recommendation::Compel,
            Recommendation::Sanction,
            Recommendation::NoAction,
        ];
        prop_assert_eq!(recs[idx], recs[idx]);
    }
}
