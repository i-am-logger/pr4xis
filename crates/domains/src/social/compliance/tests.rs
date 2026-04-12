use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::social::compliance::classification::*;
use crate::social::compliance::escalation::*;
use crate::social::compliance::law::*;
use crate::social::compliance::ontology::*;

// ---------------------------------------------------------------------------
// Category and ontology validation
// ---------------------------------------------------------------------------

#[test]
fn compliance_category_laws() {
    check_category_laws::<ComplianceCategory>().unwrap();
}

#[test]
fn compliance_ontology_validates() {
    ComplianceOntology::validate().unwrap();
}

// ---------------------------------------------------------------------------
// LOAC axioms — individual proofs
// ---------------------------------------------------------------------------

#[test]
fn loac_distinction_principle() {
    assert!(DistinctionPrinciple.holds());
}

#[test]
fn loac_civilian_presumption() {
    assert!(CivilianPresumption.holds());
}

#[test]
fn loac_human_in_the_loop() {
    assert!(HumanInTheLoop.holds());
}

#[test]
fn loac_sequential_escalation() {
    assert!(SequentialEscalation.holds());
}

#[test]
fn loac_advance_warning() {
    assert!(AdvanceWarning.holds());
}

#[test]
fn loac_abort_always_available() {
    assert!(AbortAlwaysAvailable.holds());
}

// ---------------------------------------------------------------------------
// Classification proofs
// ---------------------------------------------------------------------------

#[test]
fn unknown_entity_is_not_engageable() {
    let entity = ClassifiedEntity::unknown();
    assert!(!entity.meets_engagement_threshold());
}

#[test]
fn person_never_meets_engagement_threshold() {
    // Even fully classified hostile person requires human PositiveId via escalation
    let person = ClassifiedEntity {
        entity_type: EntityType::Person,
        iff: IffClassification::Hostile,
        protected_status: ProtectedStatus::MilitaryObjective,
        confidence: Confidence::High,
    };
    assert!(!person.meets_engagement_threshold());
}

#[test]
fn protected_entity_cannot_be_engaged() {
    let civilian = ClassifiedEntity {
        entity_type: EntityType::Person,
        iff: IffClassification::Neutral,
        protected_status: ProtectedStatus::Protected,
        confidence: Confidence::PositiveId,
    };
    assert!(civilian.is_protected());
    assert!(!civilian.meets_engagement_threshold());

    // Even with commander auth, escalation to Engage is denied
    let result = can_transition(
        EscalationLevel::WarningAction,
        EscalationLevel::Engage,
        &civilian,
        Authorization::CommanderAuthorized,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EscalationDenial::ProtectedEntity
    ));
}

#[test]
fn hospital_cannot_be_engaged() {
    let hospital = ClassifiedEntity {
        entity_type: EntityType::Structure,
        iff: IffClassification::Neutral,
        protected_status: ProtectedStatus::ProtectedObject,
        confidence: Confidence::PositiveId,
    };
    assert!(hospital.is_protected());
    let result = can_transition(
        EscalationLevel::WarningAction,
        EscalationLevel::Engage,
        &hospital,
        Authorization::CommanderAuthorized,
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Escalation proofs
// ---------------------------------------------------------------------------

#[test]
fn cannot_skip_from_observe_to_engage() {
    let target = ClassifiedEntity {
        entity_type: EntityType::GroundVehicle,
        iff: IffClassification::Hostile,
        protected_status: ProtectedStatus::MilitaryObjective,
        confidence: Confidence::High,
    };
    let result = can_transition(
        EscalationLevel::Observe,
        EscalationLevel::Engage,
        &target,
        Authorization::CommanderAuthorized,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EscalationDenial::SkippedLevel { .. }
    ));
}

#[test]
fn valid_sequential_escalation_permitted() {
    let target = ClassifiedEntity {
        entity_type: EntityType::GroundVehicle,
        iff: IffClassification::Hostile,
        protected_status: ProtectedStatus::MilitaryObjective,
        confidence: Confidence::High,
    };

    // Each sequential step should be permitted with appropriate auth
    let steps = [
        (
            EscalationLevel::Observe,
            EscalationLevel::Identify,
            Authorization::Autonomous,
        ),
        (
            EscalationLevel::Identify,
            EscalationLevel::Classify,
            Authorization::OperatorNotified,
        ),
        (
            EscalationLevel::Classify,
            EscalationLevel::Alert,
            Authorization::OperatorNotified,
        ),
        (
            EscalationLevel::Alert,
            EscalationLevel::Warn,
            Authorization::OperatorApproved,
        ),
        (
            EscalationLevel::Warn,
            EscalationLevel::ShowForce,
            Authorization::OperatorApproved,
        ),
        (
            EscalationLevel::ShowForce,
            EscalationLevel::NonLethal,
            Authorization::OperatorApproved,
        ),
        (
            EscalationLevel::NonLethal,
            EscalationLevel::WarningAction,
            Authorization::CommanderAuthorized,
        ),
        (
            EscalationLevel::WarningAction,
            EscalationLevel::Engage,
            Authorization::CommanderAuthorized,
        ),
    ];

    for (from, to, auth) in &steps {
        let result = can_transition(*from, *to, &target, *auth);
        assert!(
            result.is_ok(),
            "transition {:?} -> {:?} should be permitted, got {:?}",
            from,
            to,
            result
        );
    }
}

#[test]
fn abort_from_any_level() {
    let entity = ClassifiedEntity::unknown();
    let levels = [
        EscalationLevel::Observe,
        EscalationLevel::Identify,
        EscalationLevel::Classify,
        EscalationLevel::Alert,
        EscalationLevel::Warn,
        EscalationLevel::ShowForce,
        EscalationLevel::NonLethal,
        EscalationLevel::WarningAction,
        EscalationLevel::Engage,
    ];
    for &level in &levels {
        assert!(
            can_transition(
                level,
                EscalationLevel::Abort,
                &entity,
                Authorization::Autonomous
            )
            .is_ok(),
            "abort from {:?} should always be permitted",
            level
        );
    }
}

#[test]
fn insufficient_auth_denied() {
    let target = ClassifiedEntity {
        entity_type: EntityType::GroundVehicle,
        iff: IffClassification::Hostile,
        protected_status: ProtectedStatus::MilitaryObjective,
        confidence: Confidence::High,
    };
    // Try to engage with only operator approval (need commander)
    let result = can_transition(
        EscalationLevel::WarningAction,
        EscalationLevel::Engage,
        &target,
        Authorization::OperatorApproved,
    );
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EscalationDenial::InsufficientAuthorization { .. }
    ));
}

#[test]
fn person_with_positive_id_and_military_objective_still_blocked_by_meets_threshold() {
    // meets_engagement_threshold returns false for persons — this is the
    // structural guarantee. Persons ALWAYS go through the human-in-the-loop path.
    let person = ClassifiedEntity {
        entity_type: EntityType::Person,
        iff: IffClassification::Hostile,
        protected_status: ProtectedStatus::MilitaryObjective,
        confidence: Confidence::PositiveId,
    };
    assert!(!person.meets_engagement_threshold());
}

// ---------------------------------------------------------------------------
// H9: Authorization level() is monotonic and consistent
// ---------------------------------------------------------------------------

#[test]
fn authorization_level_ordering() {
    assert!(Authorization::Autonomous.level() < Authorization::OperatorNotified.level());
    assert!(Authorization::OperatorNotified.level() < Authorization::OperatorApproved.level());
    assert!(Authorization::OperatorApproved.level() < Authorization::CommanderAuthorized.level());
}

#[test]
fn authorization_level_matches_required() {
    // Commander authorization should satisfy all requirements
    let cmd = Authorization::CommanderAuthorized;
    for &level in &[
        EscalationLevel::Observe,
        EscalationLevel::Identify,
        EscalationLevel::Classify,
        EscalationLevel::Alert,
        EscalationLevel::Warn,
        EscalationLevel::ShowForce,
        EscalationLevel::NonLethal,
        EscalationLevel::WarningAction,
        EscalationLevel::Engage,
    ] {
        let req = required_authorization(level);
        assert!(
            cmd.level() >= req.level(),
            "CommanderAuthorized should satisfy {:?} which requires {:?}",
            level,
            req
        );
    }
}

// ---------------------------------------------------------------------------
// Property-based proofs — compliance must be DETERMINISTIC
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    fn arb_entity_type() -> impl Strategy<Value = EntityType> {
        prop_oneof![
            Just(EntityType::Unknown),
            Just(EntityType::Person),
            Just(EntityType::GroundVehicle),
            Just(EntityType::Aircraft),
            Just(EntityType::Watercraft),
            Just(EntityType::Structure),
            Just(EntityType::SignalSource),
            Just(EntityType::UnmannedSystem),
            Just(EntityType::Equipment),
        ]
    }

    fn arb_iff() -> impl Strategy<Value = IffClassification> {
        prop_oneof![
            Just(IffClassification::Unknown),
            Just(IffClassification::Friend),
            Just(IffClassification::Hostile),
            Just(IffClassification::Neutral),
            Just(IffClassification::AssumedCivilian),
        ]
    }

    fn arb_protected() -> impl Strategy<Value = ProtectedStatus> {
        prop_oneof![
            Just(ProtectedStatus::Undetermined),
            Just(ProtectedStatus::Protected),
            Just(ProtectedStatus::ProtectedObject),
            Just(ProtectedStatus::MilitaryObjective),
        ]
    }

    fn arb_confidence() -> impl Strategy<Value = Confidence> {
        prop_oneof![
            Just(Confidence::None),
            Just(Confidence::Low),
            Just(Confidence::Moderate),
            Just(Confidence::High),
            Just(Confidence::PositiveId),
        ]
    }

    fn arb_entity() -> impl Strategy<Value = ClassifiedEntity> {
        (
            arb_entity_type(),
            arb_iff(),
            arb_protected(),
            arb_confidence(),
        )
            .prop_map(|(et, iff, ps, conf)| ClassifiedEntity {
                entity_type: et,
                iff,
                protected_status: ps,
                confidence: conf,
            })
    }

    fn arb_escalation() -> impl Strategy<Value = EscalationLevel> {
        prop_oneof![
            Just(EscalationLevel::Observe),
            Just(EscalationLevel::Identify),
            Just(EscalationLevel::Classify),
            Just(EscalationLevel::Alert),
            Just(EscalationLevel::Warn),
            Just(EscalationLevel::ShowForce),
            Just(EscalationLevel::NonLethal),
            Just(EscalationLevel::WarningAction),
            Just(EscalationLevel::Engage),
        ]
    }

    fn arb_auth() -> impl Strategy<Value = Authorization> {
        prop_oneof![
            Just(Authorization::Autonomous),
            Just(Authorization::OperatorNotified),
            Just(Authorization::OperatorApproved),
            Just(Authorization::CommanderAuthorized),
        ]
    }

    proptest! {
        // --- DETERMINISM ---

        #[test]
        fn escalation_is_deterministic(
            from in arb_escalation(),
            to in arb_escalation(),
            auth in arb_auth(),
            entity in arb_entity(),
        ) {
            let r1 = can_transition(from, to, &entity, auth);
            let r2 = can_transition(from, to, &entity, auth);
            prop_assert_eq!(r1, r2, "escalation must be deterministic");
        }

        // --- PERSONS ARE ALWAYS BLOCKED FROM AUTONOMOUS ENGAGEMENT ---

        #[test]
        fn person_never_meets_engagement_threshold(
            iff in arb_iff(),
            ps in arb_protected(),
            conf in arb_confidence(),
        ) {
            let entity = ClassifiedEntity {
                entity_type: EntityType::Person,
                iff,
                protected_status: ps,
                confidence: conf,
            };
            prop_assert!(!entity.meets_engagement_threshold(),
                "Person must NEVER meet engagement threshold regardless of classification");
        }

        // --- PROTECTED ENTITIES CANNOT BE ENGAGED ---

        #[test]
        fn protected_entity_blocked_from_engage(
            entity_type in arb_entity_type(),
            iff in arb_iff(),
            auth in arb_auth(),
            conf in arb_confidence(),
        ) {
            let entity = ClassifiedEntity {
                entity_type,
                iff,
                protected_status: ProtectedStatus::Protected,
                confidence: conf,
            };
            let result = can_transition(
                EscalationLevel::WarningAction,
                EscalationLevel::Engage,
                &entity,
                auth,
            );
            prop_assert!(result.is_err(),
                "protected entity must never reach Engage");
        }

        #[test]
        fn protected_object_blocked_from_engage(
            entity_type in arb_entity_type(),
            iff in arb_iff(),
            auth in arb_auth(),
            conf in arb_confidence(),
        ) {
            let entity = ClassifiedEntity {
                entity_type,
                iff,
                protected_status: ProtectedStatus::ProtectedObject,
                confidence: conf,
            };
            let result = can_transition(
                EscalationLevel::WarningAction,
                EscalationLevel::Engage,
                &entity,
                auth,
            );
            prop_assert!(result.is_err(),
                "protected object must never reach Engage");
        }

        // --- ABORT IS ALWAYS AVAILABLE ---

        #[test]
        fn abort_always_available(
            from in arb_escalation(),
            entity in arb_entity(),
        ) {
            let result = can_transition(from, EscalationLevel::Abort, &entity, Authorization::Autonomous);
            prop_assert!(result.is_ok(),
                "abort must ALWAYS be available from {:?}", from);
        }

        // --- DEESCALATE IS ALWAYS AVAILABLE ---

        #[test]
        fn deescalate_always_available(
            from in arb_escalation(),
            entity in arb_entity(),
        ) {
            let result = can_transition(from, EscalationLevel::Deescalate, &entity, Authorization::Autonomous);
            prop_assert!(result.is_ok(),
                "deescalate must ALWAYS be available from {:?}", from);
        }

        // --- CANNOT SKIP LEVELS ---

        #[test]
        fn cannot_skip_to_engage(
            from in prop_oneof![
                Just(EscalationLevel::Observe),
                Just(EscalationLevel::Identify),
                Just(EscalationLevel::Classify),
                Just(EscalationLevel::Alert),
                Just(EscalationLevel::Warn),
                Just(EscalationLevel::ShowForce),
            ],
            entity in arb_entity(),
        ) {
            // Skipping more than 1 level to Engage should be rejected
            let result = can_transition(
                from,
                EscalationLevel::Engage,
                &entity,
                Authorization::CommanderAuthorized,
            );
            prop_assert!(result.is_err(),
                "must not skip from {:?} directly to Engage", from);
        }

        // --- UNKNOWN ENTITIES CANNOT BE ENGAGED ---

        #[test]
        fn unknown_entity_cannot_be_engaged(
            entity_type in arb_entity_type(),
            auth in arb_auth(),
        ) {
            let entity = ClassifiedEntity {
                entity_type,
                iff: IffClassification::Unknown,
                protected_status: ProtectedStatus::Undetermined,
                confidence: Confidence::None,
            };
            let result = can_transition(
                EscalationLevel::WarningAction,
                EscalationLevel::Engage,
                &entity,
                auth,
            );
            prop_assert!(result.is_err(),
                "unknown entity must never reach Engage");
        }

        // --- FRIENDLY ENTITIES CANNOT BE ENGAGED ---

        #[test]
        fn friendly_cannot_be_engaged(
            entity_type in arb_entity_type(),
            conf in arb_confidence(),
        ) {
            let entity = ClassifiedEntity {
                entity_type,
                iff: IffClassification::Friend,
                protected_status: ProtectedStatus::MilitaryObjective, // even if classified as military
                confidence: conf,
            };
            prop_assert!(!entity.meets_engagement_threshold(),
                "friendly entity must never meet engagement threshold");
        }
    }
}

// ---------------------------------------------------------------------------
// Meta-axiom: ClassifiedEntity::new() enforces cross-field consistency.
//
// The ontology claims no contradictory classification states can exist.
// The validated constructor rejects contradictory combinations.
// ---------------------------------------------------------------------------

#[test]
fn meta_axiom_classified_entity_rejects_none_confidence_with_known_iff() {
    // Confidence::None + non-Unknown IFF is contradictory:
    // "no information" cannot imply "known identification"
    let result = ClassifiedEntity::new(
        EntityType::Unknown,
        IffClassification::Hostile,
        ProtectedStatus::MilitaryObjective,
        Confidence::None,
    );
    assert!(
        result.is_none(),
        "None confidence with Hostile IFF must be rejected"
    );

    let result2 = ClassifiedEntity::new(
        EntityType::Person,
        IffClassification::Friend,
        ProtectedStatus::Protected,
        Confidence::None,
    );
    assert!(
        result2.is_none(),
        "None confidence with Friend IFF must be rejected"
    );
}

#[test]
fn meta_axiom_classified_entity_rejects_low_confidence_protected_hostile() {
    // Protected + Hostile at low confidence is rejected
    // (edge case requires at least High confidence)
    let result = ClassifiedEntity::new(
        EntityType::Person,
        IffClassification::Hostile,
        ProtectedStatus::Protected,
        Confidence::Low,
    );
    assert!(
        result.is_none(),
        "Protected + Hostile at Low confidence must be rejected"
    );

    let result2 = ClassifiedEntity::new(
        EntityType::Person,
        IffClassification::Hostile,
        ProtectedStatus::Protected,
        Confidence::Moderate,
    );
    assert!(
        result2.is_none(),
        "Protected + Hostile at Moderate confidence must be rejected"
    );
}

#[test]
fn meta_axiom_classified_entity_allows_valid_combinations() {
    // Unknown entity with no confidence: valid
    let r1 = ClassifiedEntity::new(
        EntityType::Unknown,
        IffClassification::Unknown,
        ProtectedStatus::Undetermined,
        Confidence::None,
    );
    assert!(r1.is_some(), "fully unknown entity must be accepted");

    // Hostile military at high confidence: valid
    let r2 = ClassifiedEntity::new(
        EntityType::GroundVehicle,
        IffClassification::Hostile,
        ProtectedStatus::MilitaryObjective,
        Confidence::High,
    );
    assert!(
        r2.is_some(),
        "hostile military at high confidence must be accepted"
    );

    // Protected hostile at high confidence: edge case, allowed
    let r3 = ClassifiedEntity::new(
        EntityType::Person,
        IffClassification::Hostile,
        ProtectedStatus::Protected,
        Confidence::High,
    );
    assert!(
        r3.is_some(),
        "protected hostile at High confidence must be accepted (edge case)"
    );

    // Protected hostile at PositiveId: also allowed
    let r4 = ClassifiedEntity::new(
        EntityType::Person,
        IffClassification::Hostile,
        ProtectedStatus::Protected,
        Confidence::PositiveId,
    );
    assert!(
        r4.is_some(),
        "protected hostile at PositiveId must be accepted"
    );
}

#[test]
fn meta_axiom_classified_entity_unknown_constructor_is_consistent() {
    // The unknown() default constructor must pass all invariants
    let u = ClassifiedEntity::unknown();
    // Verify it would pass the validated constructor
    let validated = ClassifiedEntity::new(u.entity_type, u.iff, u.protected_status, u.confidence);
    assert!(
        validated.is_some(),
        "unknown() default must be a valid combination"
    );
    assert_eq!(
        validated.unwrap(),
        u,
        "validated construction must produce same entity"
    );
}

mod prop {
    use super::*;
    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_escalation() -> impl Strategy<Value = EscalationLevel> {
        prop_oneof![
            Just(EscalationLevel::Observe),
            Just(EscalationLevel::Identify),
            Just(EscalationLevel::Classify),
            Just(EscalationLevel::Alert),
            Just(EscalationLevel::Warn),
            Just(EscalationLevel::ShowForce),
            Just(EscalationLevel::NonLethal),
            Just(EscalationLevel::WarningAction),
            Just(EscalationLevel::Engage),
            Just(EscalationLevel::Deescalate),
            Just(EscalationLevel::Abort),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(level in arb_escalation()) {
            let id = ComplianceCategory::identity(&level);
            prop_assert_eq!(ComplianceCategory::compose(&id, &id), Some(id));
        }

        /// Every level has an identity morphism.
        #[test]
        fn prop_identity_exists(level in arb_escalation()) {
            let id = ComplianceCategory::identity(&level);
            prop_assert_eq!(id.from, level);
            prop_assert_eq!(id.to, level);
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(level in arb_escalation()) {
            let m = ComplianceCategory::morphisms();
            let id = ComplianceCategory::identity(&level);
            for morph in m.iter().filter(|r| r.from == level) {
                let composed = ComplianceCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }

        /// LOAC: protected entities can never meet engagement threshold.
        #[test]
        fn prop_protected_never_engageable(
            entity_type in prop_oneof![
                Just(EntityType::Person),
                Just(EntityType::Structure),
                Just(EntityType::GroundVehicle),
                Just(EntityType::Aircraft),
                Just(EntityType::Watercraft),
            ],
            iff in prop_oneof![
                Just(IffClassification::Unknown),
                Just(IffClassification::Friend),
                Just(IffClassification::Neutral),
                Just(IffClassification::Hostile),
            ]
        ) {
            let entity = ClassifiedEntity {
                entity_type,
                iff,
                protected_status: ProtectedStatus::Protected,
                confidence: Confidence::PositiveId,
            };
            prop_assert!(entity.is_protected());
            prop_assert!(!entity.meets_engagement_threshold());
        }

        /// Abort always available from any escalation level.
        #[test]
        fn prop_abort_always_available(_dummy in 0..1i32) {
            prop_assert!(AbortAlwaysAvailable.holds());
        }

        /// All 6 LOAC axioms hold.
        #[test]
        fn prop_all_axioms_hold(_dummy in 0..1i32) {
            prop_assert!(DistinctionPrinciple.holds());
            prop_assert!(CivilianPresumption.holds());
            prop_assert!(HumanInTheLoop.holds());
            prop_assert!(SequentialEscalation.holds());
            prop_assert!(AdvanceWarning.holds());
            prop_assert!(AbortAlwaysAvailable.holds());
        }
    }
}
