//! Channel praxis engine — the receiver-side admittance state machine.
//!
//! Materialises the prx protocol's receiver state: the Membership relation
//! (established by accepted joins, dissolved by leaves — Locke 1689 consent),
//! the Authority assignments (the Lampson 1971/1974 access-matrix rows for
//! this channel), and the slash registry (Buterin & Griffith 2017; Li et al.
//! 2004 SUNDR fork-consistency).
//!
//! Two admittance rules, in order:
//!
//! 1. **Gate 0** — an event authored by a slashed device is dropped before
//!    any further validation (prx §7.3 / §8.4: slashing operates on future
//!    admittance).
//! 2. **Least privilege** — a role grant is admitted only if the granter's
//!    ladder rank strictly exceeds the target rank (Saltzer & Schroeder 1975;
//!    RFC 2811 operator powers).
//!
//! Pure `no_std` + `alloc`: the transition function has no side effects.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use super::ontology::{FOUNDER_RANK, OPERATOR_RANK, RankOrdinal, VOICE_RANK};

/// Width in bytes of a device / event content address: prx names devices and
/// events by blake3 digests, whose default output is 256 bits (32 bytes) —
/// the same width as the Seed's 256-bit entropy (prx ontology, Identity
/// cluster). A documented protocol constant, not a magic number.
pub const DEVICE_ID_BYTES: usize = 32;

/// The external name of an identity: blake3 of the authoring public key
/// (prx ontology §2.3; Wittgenstein 1953 §43 — the handle peers use, not the
/// identity itself).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceIdValue(pub [u8; DEVICE_ID_BYTES]);

/// Content address of a signed event — blake3 over its canonical bytes.
/// Two equivocating events at the same seq have DIFFERENT digests; the pair
/// of digests is what a fork proof carries (Li et al. 2004 SUNDR §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventDigest(pub [u8; DEVICE_ID_BYTES]);

/// The monotonic per-device sequence number ordering events into the chain
/// (prx ontology §5.1: linear order, no gaps, no forks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeqNumber(pub u64);

/// The closed-set authority tier a device may hold in a channel —
/// `Founder > Operator > Voice` (RFC 1459 Oikarinen & Reed 1993; RFC 2811
/// Kalt 2000). Closed because every authority check has to know the universe
/// of possible authorities; arbitrary authority strings would defeat the
/// enforcement layer (Saltzer & Schroeder 1975, economy of mechanism).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelRole {
    /// The constitutive tier: the identity whose constitutive agency signed
    /// the channel manifest.
    Founder,
    /// The operational tier: IRC mode +o (RFC 1459).
    Operator,
    /// The participation tier: IRC mode +v (RFC 1459).
    Voice,
}

impl ChannelRole {
    /// Ladder position — the strict total order the least-privilege gate
    /// compares (Saltzer & Schroeder 1975; prx §3.3).
    pub fn rank(&self) -> RankOrdinal {
        match self {
            ChannelRole::Founder => FOUNDER_RANK,
            ChannelRole::Operator => OPERATOR_RANK,
            ChannelRole::Voice => VOICE_RANK,
        }
    }
}

/// A claimed equivocation proof: same device, same seq, two signed events.
/// The claim is only a proof when the two events actually differ — the
/// existence of the differing pair IS the proof (Li et al. 2004 SUNDR §3;
/// Lamport, Shostak & Pease 1982: misbehaviour detectable from message
/// inconsistency alone).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForkProofClaim {
    /// The equivocating device.
    pub device: DeviceIdValue,
    /// The chain position at which both events were published.
    pub seq: SeqNumber,
    /// Digest of the first signed event.
    pub first: EventDigest,
    /// Digest of the second signed event.
    pub second: EventDigest,
}

impl ForkProofClaim {
    /// SUNDR §3 equivocation: the two events at the same seq must differ.
    /// An identical pair proves nothing — it is the same event twice.
    pub fn is_equivocation(&self) -> bool {
        self.first != self.second
    }
}

/// Receiver-side channel state: who is a member, what authority each member
/// holds, and which devices are slashed.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSituation {
    /// The materialised Membership relation (prx §6.3: derivable from
    /// accepted joins, not separately asserted).
    pub members: Vec<DeviceIdValue>,
    /// The channel's rows of the access matrix (Lampson 1971/1974):
    /// device -> role.
    pub roles: Vec<(DeviceIdValue, ChannelRole)>,
    /// The receiver-side slash registry mirror (prx §7.3).
    pub slashed: Vec<DeviceIdValue>,
}

impl Situation for ChannelSituation {}

impl ChannelSituation {
    /// The founding act (Hart 1961 secondary rules; prx §8.2): constituting
    /// the channel creates the seed Founder authority from which all
    /// subsequent authority descends.
    pub fn founded(founder: DeviceIdValue) -> Self {
        ChannelSituation {
            members: vec![founder],
            roles: vec![(founder, ChannelRole::Founder)],
            slashed: Vec::new(),
        }
    }

    /// Is the device in the materialised Membership relation?
    pub fn is_member(&self, device: &DeviceIdValue) -> bool {
        self.members.contains(device)
    }

    /// The role this device holds in the channel, if any.
    pub fn role_of(&self, device: &DeviceIdValue) -> Option<ChannelRole> {
        self.roles
            .iter()
            .find(|(d, _)| d == device)
            .map(|(_, r)| *r)
    }

    /// Has a fork proof been observed for this device?
    pub fn is_slashed(&self, device: &DeviceIdValue) -> bool {
        self.slashed.contains(device)
    }
}

/// The admittance decisions a receiver takes on observed events.
#[derive(Debug, Clone)]
pub enum ChannelAction {
    /// Admit a `ChannelJoin` authored by `device` — a Declaration: accepted,
    /// it establishes the Membership relation (Austin 1962; Locke 1689).
    AdmitJoin { device: DeviceIdValue },
    /// Admit a `Leave` authored by `device` — dissolves the Membership
    /// relation and, with it, the channel authorities held under it
    /// (RFC 1459: channel modes are per-membership).
    AdmitLeave { device: DeviceIdValue },
    /// Admit a `RoleGrant` authored by `granter` conferring `role` on
    /// `grantee` — performative authority creation, gated by the ladder
    /// (Austin 1962; Saltzer & Schroeder 1975).
    AdmitRoleGrant {
        granter: DeviceIdValue,
        grantee: DeviceIdValue,
        role: ChannelRole,
    },
    /// Record an observed fork proof — not an authored event but receiver-
    /// side evidence; recording it slashes the equivocating device
    /// (Buterin & Griffith 2017; Li et al. 2004 SUNDR §3).
    ObserveForkProof { proof: ForkProofClaim },
}

impl Action for ChannelAction {
    type Sit = ChannelSituation;
}

/// Gate 0 (prx §7.3 / §8.4): an event authored by a slashed device is
/// dropped before any further validation. Slashing operates on future
/// admittance — past praxis stays in the chain as evidence.
fn gate_zero(situation: &ChannelSituation, author: &DeviceIdValue) -> Result<(), String> {
    if situation.is_slashed(author) {
        Err("gate 0: author is slashed; event dropped (prx §8.4)".to_string())
    } else {
        Ok(())
    }
}

/// The pure admittance transition. `Err` means the event was dropped (with
/// the reason); `Ok` carries the successor situation.
pub fn apply_channel(
    situation: &ChannelSituation,
    action: &ChannelAction,
) -> Result<ChannelSituation, String> {
    match action {
        ChannelAction::AdmitJoin { device } => {
            gate_zero(situation, device)?;
            let mut next = situation.clone();
            if !next.is_member(device) {
                next.members.push(*device);
            }
            Ok(next)
        }
        ChannelAction::AdmitLeave { device } => {
            gate_zero(situation, device)?;
            let mut next = situation.clone();
            next.members.retain(|m| m != device);
            // Authorities are held under the membership; dissolving the
            // membership dissolves them (RFC 1459: modes are per-membership).
            next.roles.retain(|(d, _)| d != device);
            Ok(next)
        }
        ChannelAction::AdmitRoleGrant {
            granter,
            grantee,
            role,
        } => {
            gate_zero(situation, granter)?;
            let granter_rank = situation
                .role_of(granter)
                .map(|r| r.rank())
                .ok_or_else(|| {
                    "role grant dropped: granter holds no authority in this channel".to_string()
                })?;
            // Least privilege (Saltzer & Schroeder 1975; RFC 2811): a grant
            // is admitted only if the granter strictly outranks the target
            // tier — an Operator may voice, only the Founder may op.
            if granter_rank > role.rank() {
                let mut next = situation.clone();
                next.roles.retain(|(d, _)| d != grantee);
                next.roles.push((*grantee, *role));
                Ok(next)
            } else {
                Err(
                    "role grant dropped: granter's rank does not strictly exceed the target rank (least privilege)"
                        .to_string(),
                )
            }
        }
        ChannelAction::ObserveForkProof { proof } => {
            if !proof.is_equivocation() {
                return Err(
                    "fork proof rejected: the two events are identical, not an equivocation (SUNDR §3)"
                        .to_string(),
                );
            }
            let mut next = situation.clone();
            if !next.is_slashed(&proof.device) {
                next.slashed.push(proof.device);
            }
            Ok(next)
        }
    }
}
