//! Engine mechanics tests — the receiver-side admittance rules.
//!
//! The domain CLAIMS (slashing exclusion, illocutionary totality, the role
//! ladder, moderation authority, ban durability, mode-change amendment, the
//! two prx axioms) are asserted by `verify()`-ing the Axiom structs in
//! `ontology.rs`; the tests here exercise the engine's concrete transition
//! mechanics around those claims.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::engine::*;

/// Fixture device ids — arbitrary distinct byte patterns standing in for
/// blake3 digests (documented structural fixtures, not protocol constants).
fn founder() -> DeviceIdValue {
    DeviceIdValue([1; DEVICE_ID_BYTES])
}
fn alice() -> DeviceIdValue {
    DeviceIdValue([2; DEVICE_ID_BYTES])
}
fn bob() -> DeviceIdValue {
    DeviceIdValue([3; DEVICE_ID_BYTES])
}

/// Fixture channel identity — a distinct content-address stand-in.
fn channel() -> ChannelIdValue {
    ChannelIdValue([0; DEVICE_ID_BYTES])
}

/// A freshly-constituted channel with the default (open) manifest modes.
fn open_channel() -> ChannelSituation {
    ChannelSituation::founded(channel(), founder(), ChannelModeSet::default())
}

/// A freshly-constituted channel whose manifest sets the `op_can_grant_op`
/// mode (RFC 2811 §4.1: operators may act at the operator tier).
fn op_grantable_channel() -> ChannelSituation {
    ChannelSituation::founded(
        channel(),
        founder(),
        ChannelModeSet {
            op_can_grant_op: true,
        },
    )
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn founding_seats_the_founder() {
    // Hart (1961): the founding act creates the seed Founder authority.
    let s = open_channel();
    assert!(s.is_member(&founder()));
    assert_eq!(s.role_of(&founder()), Some(ChannelRole::Founder));
    assert!(!s.is_slashed(&founder()));
    assert_eq!(s.channel, channel());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn join_establishes_and_leave_dissolves_membership() {
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    assert!(s1.is_member(&alice()));

    let s2 = apply_channel(&s1, &ChannelAction::AdmitLeave { device: alice() }).unwrap();
    assert!(!s2.is_member(&alice()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn join_is_idempotent_on_membership() {
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(&s1, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    assert_eq!(s1, s2);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn founder_can_grant_operator_and_voice() {
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    assert_eq!(s2.role_of(&alice()), Some(ChannelRole::Operator));

    let s3 = apply_channel(&s2, &ChannelAction::AdmitJoin { device: bob() }).unwrap();
    let s4 = apply_channel(
        &s3,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: bob(),
            role: ChannelRole::Voice,
        },
    )
    .unwrap();
    assert_eq!(s4.role_of(&bob()), Some(ChannelRole::Voice));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn operator_can_voice_but_not_op() {
    // Saltzer & Schroeder (1975) least privilege: with op_can_grant_op clear
    // (the default open channel), an Operator's authority does not reach the
    // Operator tier — op-grants-op is dropped; voice is below operator, so
    // that grant is admitted.
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    let s3 = apply_channel(&s2, &ChannelAction::AdmitJoin { device: bob() }).unwrap();

    let voiced = apply_channel(
        &s3,
        &ChannelAction::AdmitRoleGrant {
            granter: alice(),
            grantee: bob(),
            role: ChannelRole::Voice,
        },
    );
    assert!(voiced.is_ok());

    let opped = apply_channel(
        &s3,
        &ChannelAction::AdmitRoleGrant {
            granter: alice(),
            grantee: bob(),
            role: ChannelRole::Operator,
        },
    );
    assert!(opped.is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn op_can_grant_op_mode_lets_operator_op() {
    // RFC 2811 §4.1 / prx op_can_grant_op: with the mode set, an Operator may
    // grant the Operator tier to a fellow — the one exception to strict rank.
    let s0 = op_grantable_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    let s3 = apply_channel(&s2, &ChannelAction::AdmitJoin { device: bob() }).unwrap();
    let opped = apply_channel(
        &s3,
        &ChannelAction::AdmitRoleGrant {
            granter: alice(),
            grantee: bob(),
            role: ChannelRole::Operator,
        },
    );
    assert!(opped.is_ok());
    assert_eq!(opped.unwrap().role_of(&bob()), Some(ChannelRole::Operator));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nobody_grants_founder() {
    // The Founder tier is constitutive; can_grant(_, Founder) is false even
    // when op_can_grant_op is set (prx: nobody grants Founder).
    let s0 = op_grantable_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let granted = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Founder,
        },
    );
    assert!(granted.is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn granter_without_authority_is_dropped() {
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    // alice is a member but holds no role — her grant carries no authority.
    let result = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: alice(),
            grantee: founder(),
            role: ChannelRole::Voice,
        },
    );
    assert!(result.is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn leave_dissolves_authority_with_membership() {
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    let s3 = apply_channel(&s2, &ChannelAction::AdmitLeave { device: alice() }).unwrap();
    assert_eq!(s3.role_of(&alice()), None);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn revoke_strips_role_and_is_idempotent_on_no_role() {
    // prx apply_revoke: strips the target's current role; revoking a device
    // with no role is an idempotent accept (no-op).
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    let s3 = apply_channel(
        &s2,
        &ChannelAction::AdmitRoleRevoke {
            revoker: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert_eq!(s3.role_of(&alice()), None);

    // Idempotent no-op: revoking again (alice now has no role) is accepted
    // and leaves the state unchanged.
    let s4 = apply_channel(
        &s3,
        &ChannelAction::AdmitRoleRevoke {
            revoker: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert_eq!(s3, s4);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn kick_strips_role_and_records_transient_kick() {
    // RFC 1459 §4.2.8 / prx apply_kick: strips the target's granted authority
    // and records a transient kick; a fresh join clears the kick.
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    let s3 = apply_channel(
        &s2,
        &ChannelAction::AdmitKick {
            kicker: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert_eq!(s3.role_of(&alice()), None);
    assert!(s3.is_kicked(&alice()));
    assert!(!s3.is_banned(&alice()));

    // A fresh accepted join clears the kick (prx clear_kick).
    let s4 = apply_channel(&s3, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    assert!(!s4.is_kicked(&alice()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn unkick_lifts_a_kick() {
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitKick {
            kicker: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert!(s2.is_kicked(&alice()));
    let s3 = apply_channel(
        &s2,
        &ChannelAction::AdmitUnkick {
            lifter: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert!(!s3.is_kicked(&alice()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ban_entails_kick_and_blocks_rejoin_until_unban() {
    // RFC 2811 §4.3.1 / prx apply_ban: ban strips authority, records the kick,
    // AND records a durable ban that refuses rejoin until Unban.
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitBan {
            banner: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert!(s2.is_kicked(&alice()));
    assert!(s2.is_banned(&alice()));
    assert_eq!(s2.role_of(&alice()), None);

    // Rejoin is refused while banned.
    assert!(apply_channel(&s2, &ChannelAction::AdmitJoin { device: alice() }).is_err());

    // Unban lifts the ban; rejoin is then admitted.
    let s3 = apply_channel(
        &s2,
        &ChannelAction::AdmitUnban {
            lifter: founder(),
            target: alice(),
        },
    )
    .unwrap();
    assert!(!s3.is_banned(&alice()));
    assert!(apply_channel(&s3, &ChannelAction::AdmitJoin { device: alice() }).is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn founder_is_untouchable() {
    // prx: the founder is never kickable or bannable (constitutive tier).
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Operator,
        },
    )
    .unwrap();
    // Even an operator cannot kick, ban, or revoke the founder.
    assert!(
        apply_channel(
            &s2,
            &ChannelAction::AdmitKick {
                kicker: alice(),
                target: founder(),
            }
        )
        .is_err()
    );
    assert!(
        apply_channel(
            &s2,
            &ChannelAction::AdmitBan {
                banner: alice(),
                target: founder(),
            }
        )
        .is_err()
    );
    assert!(
        apply_channel(
            &s2,
            &ChannelAction::AdmitRoleRevoke {
                revoker: alice(),
                target: founder(),
            }
        )
        .is_err()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn mode_change_amends_modes_without_touching_identity() {
    // Hart (1961) / Lessig (1999): a mode change amends the operative rules
    // under the existing constitution; the channel identity is untouched.
    let s0 = open_channel();
    assert!(!s0.modes.op_can_grant_op);
    let s1 = apply_channel(
        &s0,
        &ChannelAction::AdmitModeChange {
            actor: founder(),
            modes: ChannelModeSet {
                op_can_grant_op: true,
            },
        },
    )
    .unwrap();
    assert!(s1.modes.op_can_grant_op);
    assert_eq!(s1.channel, s0.channel);
    assert_eq!(s1.members, s0.members);
    assert_eq!(s1.roles, s0.roles);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn voice_cannot_moderate_or_change_modes() {
    // can_grant(Voice, Voice) is false: a voiced user holds no moderation or
    // mode authority (Saltzer & Schroeder 1975 least privilege).
    let s0 = open_channel();
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(
        &s1,
        &ChannelAction::AdmitRoleGrant {
            granter: founder(),
            grantee: alice(),
            role: ChannelRole::Voice,
        },
    )
    .unwrap();
    let s3 = apply_channel(&s2, &ChannelAction::AdmitJoin { device: bob() }).unwrap();

    assert!(
        apply_channel(
            &s3,
            &ChannelAction::AdmitKick {
                kicker: alice(),
                target: bob(),
            }
        )
        .is_err()
    );
    assert!(
        apply_channel(
            &s3,
            &ChannelAction::AdmitModeChange {
                actor: alice(),
                modes: ChannelModeSet {
                    op_can_grant_op: true,
                },
            }
        )
        .is_err()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn identical_pair_is_not_a_fork_proof() {
    // SUNDR §3: an equivocation is two DIFFERENT signed events at the same
    // seq; the same digest twice proves nothing.
    let s0 = open_channel();
    let claim = ForkProofClaim {
        device: founder(),
        seq: SeqNumber(1),
        first: EventDigest([7; DEVICE_ID_BYTES]),
        second: EventDigest([7; DEVICE_ID_BYTES]),
    };
    assert!(!claim.is_equivocation());
    assert!(apply_channel(&s0, &ChannelAction::ObserveForkProof { proof: claim }).is_err());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn fork_proof_recording_is_idempotent() {
    let s0 = open_channel();
    let claim = ForkProofClaim {
        device: founder(),
        seq: SeqNumber(1),
        first: EventDigest([7; DEVICE_ID_BYTES]),
        second: EventDigest([8; DEVICE_ID_BYTES]),
    };
    let s1 = apply_channel(&s0, &ChannelAction::ObserveForkProof { proof: claim }).unwrap();
    let s2 = apply_channel(&s1, &ChannelAction::ObserveForkProof { proof: claim }).unwrap();
    assert_eq!(s1, s2);
    assert!(s2.is_slashed(&founder()));
}

// ---------------------------------------------------------------------------
// Proptests
// ---------------------------------------------------------------------------

mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    fn arb_device() -> impl Strategy<Value = DeviceIdValue> {
        any::<[u8; DEVICE_ID_BYTES]>().prop_map(DeviceIdValue)
    }

    fn arb_role() -> impl Strategy<Value = ChannelRole> {
        proptest::sample::select(vec![
            ChannelRole::Founder,
            ChannelRole::Operator,
            ChannelRole::Voice,
        ])
    }

    proptest! {
        #[test]
        fn prop_gate_zero_excludes_all_authored_events(
            device in arb_device(),
            other in arb_device(),
            role in arb_role(),
        ) {
            // Distinct fixture digests: the differing pair is the proof.
            prop_assume!(device != other);
            let s0 = ChannelSituation::founded(
                ChannelIdValue([0; DEVICE_ID_BYTES]),
                other,
                ChannelModeSet::default(),
            );
            let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device })
                .expect("join before slash is admitted");
            let proof = ForkProofClaim {
                device,
                seq: SeqNumber(1),
                first: EventDigest([9; DEVICE_ID_BYTES]),
                second: EventDigest([10; DEVICE_ID_BYTES]),
            };
            let s2 = apply_channel(&s1, &ChannelAction::ObserveForkProof { proof })
                .expect("valid fork proof is recorded");
            let join_dropped =
                apply_channel(&s2, &ChannelAction::AdmitJoin { device }).is_err();
            let leave_dropped =
                apply_channel(&s2, &ChannelAction::AdmitLeave { device }).is_err();
            let grant_dropped = apply_channel(&s2, &ChannelAction::AdmitRoleGrant {
                granter: device,
                grantee: other,
                role,
            })
            .is_err();
            prop_assert!(join_dropped);
            prop_assert!(leave_dropped);
            prop_assert!(grant_dropped);
        }

        #[test]
        fn prop_role_grant_obeys_can_grant(
            granter_role in arb_role(),
            target_role in arb_role(),
            op_can_grant_op in any::<bool>(),
            granter in arb_device(),
            grantee in arb_device(),
        ) {
            prop_assume!(granter != grantee);
            let situation = ChannelSituation {
                channel: ChannelIdValue([0; DEVICE_ID_BYTES]),
                members: vec![granter, grantee],
                roles: vec![(granter, granter_role)],
                kicked: Vec::new(),
                banned: Vec::new(),
                modes: ChannelModeSet { op_can_grant_op },
                slashed: Vec::new(),
            };
            let result = apply_channel(&situation, &ChannelAction::AdmitRoleGrant {
                granter,
                grantee,
                role: target_role,
            });
            // Admitted exactly when the granter can_grant the target tier
            // (prx Role::can_grant; RFC 2811 §4.1; Saltzer & Schroeder 1975).
            prop_assert_eq!(
                result.is_ok(),
                granter_role.can_grant(target_role, op_can_grant_op)
            );
        }

        #[test]
        fn prop_default_mode_moderation_is_strict_rank(
            actor_role in arb_role(),
            target_role in arb_role(),
            actor in arb_device(),
            target in arb_device(),
        ) {
            prop_assume!(actor != target);
            // Default (open) channel: op_can_grant_op clear, so moderation
            // reduces to strict rank superiority (Saltzer & Schroeder 1975).
            let situation = ChannelSituation {
                channel: ChannelIdValue([0; DEVICE_ID_BYTES]),
                members: vec![actor, target],
                roles: vec![(actor, actor_role), (target, target_role)],
                kicked: Vec::new(),
                banned: Vec::new(),
                modes: ChannelModeSet::default(),
                slashed: Vec::new(),
            };
            let expected = actor_role.rank() > target_role.rank();
            let kick_ok = apply_channel(&situation, &ChannelAction::AdmitKick {
                kicker: actor,
                target,
            })
            .is_ok();
            let ban_ok = apply_channel(&situation, &ChannelAction::AdmitBan {
                banner: actor,
                target,
            })
            .is_ok();
            let revoke_ok = apply_channel(&situation, &ChannelAction::AdmitRoleRevoke {
                revoker: actor,
                target,
            })
            .is_ok();
            prop_assert_eq!(kick_ok, expected);
            prop_assert_eq!(ban_ok, expected);
            prop_assert_eq!(revoke_ok, expected);
        }

        #[test]
        fn prop_ban_refuses_rejoin(
            founder in arb_device(),
            member in arb_device(),
        ) {
            prop_assume!(founder != member);
            let s0 = ChannelSituation::founded(
                ChannelIdValue([0; DEVICE_ID_BYTES]),
                founder,
                ChannelModeSet::default(),
            );
            let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: member })
                .expect("member joins");
            let s2 = apply_channel(&s1, &ChannelAction::AdmitBan {
                banner: founder,
                target: member,
            })
            .expect("founder bans member");
            // Ban refuses rejoin; unban restores it.
            let rejoin_while_banned =
                apply_channel(&s2, &ChannelAction::AdmitJoin { device: member });
            prop_assert!(rejoin_while_banned.is_err());
            let s3 = apply_channel(&s2, &ChannelAction::AdmitUnban {
                lifter: founder,
                target: member,
            })
            .expect("founder unbans member");
            let rejoin_after_unban =
                apply_channel(&s3, &ChannelAction::AdmitJoin { device: member });
            prop_assert!(rejoin_after_unban.is_ok());
        }

        #[test]
        fn prop_mode_change_preserves_channel_identity(
            founder in arb_device(),
            op_flag in any::<bool>(),
        ) {
            let channel = ChannelIdValue([0; DEVICE_ID_BYTES]);
            let s0 = ChannelSituation::founded(channel, founder, ChannelModeSet::default());
            let s1 = apply_channel(&s0, &ChannelAction::AdmitModeChange {
                actor: founder,
                modes: ChannelModeSet { op_can_grant_op: op_flag },
            })
            .expect("founder amends modes");
            prop_assert_eq!(s1.channel, s0.channel);
            prop_assert_eq!(s1.modes.op_can_grant_op, op_flag);
        }

        #[test]
        fn prop_apply_channel_is_deterministic(
            device in arb_device(),
            other in arb_device(),
        ) {
            let s0 = ChannelSituation::founded(
                ChannelIdValue([0; DEVICE_ID_BYTES]),
                other,
                ChannelModeSet::default(),
            );
            let a = ChannelAction::AdmitJoin { device };
            let r1 = apply_channel(&s0, &a);
            let r2 = apply_channel(&s0, &a);
            prop_assert_eq!(r1, r2);
        }
    }

    pr4xis::register_praxis_value!(prop_gate_zero_excludes_all_authored_events, Verifiable);
    pr4xis::register_praxis_value!(prop_role_grant_obeys_can_grant, Verifiable);
    pr4xis::register_praxis_value!(prop_default_mode_moderation_is_strict_rank, Verifiable);
    pr4xis::register_praxis_value!(prop_ban_refuses_rejoin, Verifiable);
    pr4xis::register_praxis_value!(prop_mode_change_preserves_channel_identity, Verifiable);
    pr4xis::register_praxis_value!(prop_apply_channel_is_deterministic, Deterministic);
}
