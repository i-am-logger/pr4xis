use crate::decision::*;
use crate::element::*;
use crate::fact::*;
use crate::lifecycle::*;
use crate::ontology::*;
use crate::source::*;
use chrono::NaiveDate;
use proptest::prelude::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn test_court() -> crate::entity::Entity {
    crate::entity::Entity::Court(crate::entity::Court {
        name: "District Court".into(),
        district: None,
        circuit: None,
    })
}

fn test_judge() -> crate::entity::Entity {
    crate::entity::Entity::Person(crate::entity::Person {
        name: "Judge Smith".into(),
        title: Some("Judge".into()),
        organization: None,
        bar_admissions: vec![],
        source: None,
    })
}

fn test_movant() -> crate::entity::Entity {
    crate::entity::Entity::Person(crate::entity::Person {
        name: "Plaintiff".into(),
        title: None,
        organization: None,
        bar_admissions: vec![],
        source: None,
    })
}

fn test_respondent() -> crate::entity::Entity {
    crate::entity::Entity::Person(crate::entity::Person {
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
        statute: crate::authority::Authority::Constitution {
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
        statute: crate::authority::Authority::Constitution {
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
        authority: crate::authority::Authority::Constitution {
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
        prop_assert!(a <= b || a > b);
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
            statute: crate::authority::Authority::Constitution { provision: "".into() },
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
            statute: crate::authority::Authority::Constitution { provision: "".into() },
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
        let auth = crate::authority::Authority::Constitution { provision: "test".into() };
        prop_assert!(auth.weight() >= 3 && auth.weight() <= 10);
    }

    /// Constitution has highest weight
    #[test]
    fn prop_constitution_highest(_x in 0..1u8) {
        let c = crate::authority::Authority::Constitution { provision: "".into() };
        let t = crate::authority::Authority::TrialCourt {
            court: test_court(),
            case: crate::authority::CaseLaw { name: "".into(), citation: crate::authority::Citation::Statute { code: "".into(), section: "".into(), subsection: None, short: None }, year: 2024, court: "".into(), holding: "".into(), quote: None },
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
