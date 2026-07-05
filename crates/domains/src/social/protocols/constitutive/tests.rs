//! Engine mechanics tests — the receiver-side admittance rules.
//!
//! The domain CLAIMS (slashing exclusion, illocutionary totality, the role
//! ladder, the two prx axioms) are asserted by `verify()`-ing the Axiom
//! structs in `ontology.rs`; the tests here exercise the engine's concrete
//! transition mechanics around those claims.

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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn founding_seats_the_founder() {
    // Hart (1961): the founding act creates the seed Founder authority.
    let s = ChannelSituation::founded(founder());
    assert!(s.is_member(&founder()));
    assert_eq!(s.role_of(&founder()), Some(ChannelRole::Founder));
    assert!(!s.is_slashed(&founder()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn join_establishes_and_leave_dissolves_membership() {
    let s0 = ChannelSituation::founded(founder());
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    assert!(s1.is_member(&alice()));

    let s2 = apply_channel(&s1, &ChannelAction::AdmitLeave { device: alice() }).unwrap();
    assert!(!s2.is_member(&alice()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn join_is_idempotent_on_membership() {
    let s0 = ChannelSituation::founded(founder());
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    let s2 = apply_channel(&s1, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    assert_eq!(s1, s2);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn founder_can_grant_operator_and_voice() {
    let s0 = ChannelSituation::founded(founder());
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
    // Saltzer & Schroeder (1975) least privilege: strict ladder — an
    // Operator's rank does not strictly exceed Operator, so op-grants-op
    // is dropped; voice is below operator, so that grant is admitted.
    let s0 = ChannelSituation::founded(founder());
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
fn granter_without_authority_is_dropped() {
    let s0 = ChannelSituation::founded(founder());
    let s1 = apply_channel(&s0, &ChannelAction::AdmitJoin { device: alice() }).unwrap();
    // alice is a member but holds no role — her grant carries no rank.
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
    let s0 = ChannelSituation::founded(founder());
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
fn identical_pair_is_not_a_fork_proof() {
    // SUNDR §3: an equivocation is two DIFFERENT signed events at the same
    // seq; the same digest twice proves nothing.
    let s0 = ChannelSituation::founded(founder());
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
    let s0 = ChannelSituation::founded(founder());
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
            let s0 = ChannelSituation::founded(other);
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
        fn prop_role_grant_obeys_strict_ladder(
            granter_role in arb_role(),
            target_role in arb_role(),
            granter in arb_device(),
            grantee in arb_device(),
        ) {
            prop_assume!(granter != grantee);
            let situation = ChannelSituation {
                members: vec![granter, grantee],
                roles: vec![(granter, granter_role)],
                slashed: Vec::new(),
            };
            let result = apply_channel(&situation, &ChannelAction::AdmitRoleGrant {
                granter,
                grantee,
                role: target_role,
            });
            // Admitted exactly when the granter strictly outranks the target
            // tier (Saltzer & Schroeder 1975 least privilege).
            prop_assert_eq!(result.is_ok(), granter_role.rank() > target_role.rank());
        }

        #[test]
        fn prop_apply_channel_is_deterministic(
            device in arb_device(),
            other in arb_device(),
        ) {
            let s0 = ChannelSituation::founded(other);
            let a = ChannelAction::AdmitJoin { device };
            let r1 = apply_channel(&s0, &a);
            let r2 = apply_channel(&s0, &a);
            prop_assert_eq!(r1, r2);
        }
    }

    pr4xis::register_praxis_value!(prop_gate_zero_excludes_all_authored_events, Verifiable);
    pr4xis::register_praxis_value!(prop_role_grant_obeys_strict_ladder, Verifiable);
    pr4xis::register_praxis_value!(prop_apply_channel_is_deterministic, Deterministic);
}
