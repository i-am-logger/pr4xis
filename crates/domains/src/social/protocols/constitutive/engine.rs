//! Channel praxis engine — the receiver-side admittance state machine.
//!
//! Materialises the prx protocol's receiver state: the Membership relation
//! (established by accepted joins, dissolved by leaves — Locke 1689 consent),
//! the Authority assignments (the Lampson 1971/1974 access-matrix rows for
//! this channel), the moderation registries (kicked / banned — RFC 1459
//! Oikarinen & Reed 1993 §4.2.8; RFC 2811 Kalt 2000 §4.3.1), the currently-
//! effective channel modes (RFC 2811 §4.2; Lessig 1999 code-as-law), and the
//! slash registry (Buterin & Griffith 2017; Li et al. 2004 SUNDR
//! fork-consistency).
//!
//! Admittance rules, in order:
//!
//! 1. **Gate 0** — an event authored by a slashed device is dropped before
//!    any further validation (prx §7.3 / §8.4: slashing operates on future
//!    admittance).
//! 2. **Ban gate** — a `ChannelJoin` from a banned device is refused; the ban
//!    persists across rejoin attempts until an `Unban` lifts it (RFC 2811
//!    §4.3.1 ban masks). A fresh accepted join instead *clears* a prior kick.
//! 3. **Grant authority** — every performative that creates or strips
//!    authority (grant, revoke, kick, ban) is admitted only if the actor
//!    holds grant-authority over the target's tier, per prx `Role::can_grant`:
//!    the Founder grants anything below, an Operator grants Voice always and a
//!    fellow Operator only when the channel's `op_can_grant_op` mode is set,
//!    Voice grants nothing, and nobody grants or strips the Founder (Saltzer &
//!    Schroeder 1975 least privilege; RFC 2811 §4.1 member status).
//!
//! Pure `no_std` + `alloc`: the transition function has no side effects.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

use super::ontology::{FOUNDER_RANK, OPERATOR_RANK, RankOrdinal, VOICE_RANK};

/// Width in bytes of a device / event / channel content address: prx names
/// devices, events, and channels by blake3 digests, whose default output is
/// 256 bits (32 bytes) — the same width as the Seed's 256-bit entropy (prx
/// ontology, Identity cluster). A documented protocol constant, not a magic
/// number.
pub const DEVICE_ID_BYTES: usize = 32;

/// The external name of an identity: blake3 of the authoring public key
/// (prx ontology §2.3; Wittgenstein 1953 §43 — the handle peers use, not the
/// identity itself).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceIdValue(pub [u8; DEVICE_ID_BYTES]);

/// The external name of a channel: blake3 of the canonical channel-manifest
/// bytes (prx ontology §4.3). Because the manifest is content-addressed, the
/// channel's identity is fixed at the founding act — amendments to the
/// operative rules (mode changes) happen *under* this identity, never by
/// re-minting it (Hart 1961 primary vs. secondary rules).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelIdValue(pub [u8; DEVICE_ID_BYTES]);

/// Content address of a signed event — blake3 over its canonical bytes.
/// Two equivocating events at the same seq have DIFFERENT digests; the pair
/// of digests is what a fork proof carries (Li et al. 2004 SUNDR §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventDigest(pub [u8; DEVICE_ID_BYTES]);

/// The monotonic per-device sequence number ordering events into the chain
/// (prx ontology §5.1: linear order, no gaps, no forks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeqNumber(pub u64);

/// The channel's currently-effective IRC-style mode bits (prx role.rs
/// `ChannelModes`; RFC 2811 §4.2 Channel Flags). Only `op_can_grant_op`
/// gates authority admittance in this constitutive engine; the other modes
/// prx carries — `moderated` (+m), `secret` (+s), `member_limit` (+l) — gate
/// posting, discovery, and capacity, which this engine does not adjudicate.
///
/// A struct (mirroring prx's typed `ChannelModes`) rather than a bare flag so
/// future modes are added as fields, not magic characters in a mode string
/// (RFC 1459's flat `+m` / `+o` strings; prx's forward-compatible typed form).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChannelModeSet {
    /// prx `op_can_grant_op` — RFC 2811 §4.1's channel-creator-versus-operator
    /// member-status distinction, collapsed to one bit. When set, an Operator
    /// may grant or strip a fellow Operator; when clear (the default), only
    /// the Founder may promote or demote at the Operator tier (Saltzer &
    /// Schroeder 1975 least privilege).
    pub op_can_grant_op: bool,
}

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

    /// prx `Role::can_grant` (role.rs) — the single authority predicate that
    /// gates every performative touching authority: grant, revoke, kick, ban.
    ///
    /// - Nobody grants or strips the Founder tier: it is constitutive, pinned
    ///   in the manifest's signed bytes (Hart 1961 secondary rules), so even
    ///   the Founder cannot re-grant it.
    /// - The Founder grants/strips anything below.
    /// - An Operator grants/strips Voice always, and a fellow Operator only
    ///   when the channel's `op_can_grant_op` mode is set (RFC 2811 §4.1
    ///   channel-creator versus operator).
    /// - Voice grants/strips nobody (Saltzer & Schroeder 1975 least
    ///   privilege).
    ///
    /// With `op_can_grant_op` clear (the default), this reduces exactly to the
    /// strict-rank ladder `self.rank() > target.rank()`; the mode is the sole
    /// exception that admits an equal-tier Operator action.
    pub fn can_grant(self, target: ChannelRole, op_can_grant_op: bool) -> bool {
        match (self, target) {
            // Founder is structural — tied to the manifest; nobody grants it.
            (_, ChannelRole::Founder) => false,
            (ChannelRole::Founder, _) => true,
            (ChannelRole::Operator, ChannelRole::Voice) => true,
            (ChannelRole::Operator, ChannelRole::Operator) => op_can_grant_op,
            (ChannelRole::Voice, _) => false,
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

/// Receiver-side channel state: the channel's identity, who is a member, what
/// authority each member holds, which devices are kicked or banned, the
/// currently-effective modes, and which devices are slashed.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSituation {
    /// The channel's content-addressed identity, fixed at the founding act
    /// (prx §4.3). No admittance rule ever rewrites it — amendments run under
    /// it (Hart 1961 secondary rules).
    pub channel: ChannelIdValue,
    /// The materialised Membership relation (prx §6.3: derivable from
    /// accepted joins, not separately asserted).
    pub members: Vec<DeviceIdValue>,
    /// The channel's rows of the access matrix (Lampson 1971/1974):
    /// device -> role.
    pub roles: Vec<(DeviceIdValue, ChannelRole)>,
    /// Devices currently kicked: further chain events from them are refused
    /// until a fresh accepted `ChannelJoin` clears the entry (prx role.rs
    /// `clear_kick`; RFC 1459 §4.2.8 — a kick is transient).
    pub kicked: Vec<DeviceIdValue>,
    /// Devices currently banned: even a fresh `ChannelJoin` is refused until
    /// an `Unban` lifts the entry (prx role.rs `is_banned`; RFC 2811 §4.3.1
    /// ban masks — a ban is durable).
    pub banned: Vec<DeviceIdValue>,
    /// The currently-effective channel modes — the manifest default as
    /// amended by any accepted mode change (prx `effective_modes`).
    pub modes: ChannelModeSet,
    /// The receiver-side slash registry mirror (prx §7.3).
    pub slashed: Vec<DeviceIdValue>,
}

impl Situation for ChannelSituation {}

impl ChannelSituation {
    /// The founding act (Hart 1961 secondary rules; prx §8.2): constituting a
    /// channel with the given identity and manifest-pinned `modes` seats the
    /// seed Founder authority from which all subsequent authority descends.
    pub fn founded(channel: ChannelIdValue, founder: DeviceIdValue, modes: ChannelModeSet) -> Self {
        ChannelSituation {
            channel,
            members: vec![founder],
            roles: vec![(founder, ChannelRole::Founder)],
            kicked: Vec::new(),
            banned: Vec::new(),
            modes,
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

    /// Is this device currently kicked (transient — cleared by a fresh join)?
    pub fn is_kicked(&self, device: &DeviceIdValue) -> bool {
        self.kicked.contains(device)
    }

    /// Is this device currently banned (durable — refused even a fresh join)?
    pub fn is_banned(&self, device: &DeviceIdValue) -> bool {
        self.banned.contains(device)
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
    /// it establishes the Membership relation (Austin 1962; Locke 1689). A
    /// fresh accepted join clears a prior kick; a banned device is refused
    /// (RFC 2811 §4.3.1 ban masks).
    AdmitJoin { device: DeviceIdValue },
    /// Admit a `Leave` authored by `device` — dissolves the Membership
    /// relation and, with it, the channel authorities held under it
    /// (RFC 1459: channel modes are per-membership).
    AdmitLeave { device: DeviceIdValue },
    /// Admit a `RoleGrant` authored by `granter` conferring `role` on
    /// `grantee` — performative authority creation, gated by
    /// `ChannelRole::can_grant` (Austin 1962; RFC 2811 §4.1; Saltzer &
    /// Schroeder 1975).
    AdmitRoleGrant {
        granter: DeviceIdValue,
        grantee: DeviceIdValue,
        role: ChannelRole,
    },
    /// Admit a `RoleRevoke` authored by `revoker` stripping `target`'s current
    /// role — the symmetric counterpart of the grant, gated by the same
    /// `can_grant` predicate against the target's current tier. Revoking a
    /// device that holds no role is an idempotent accept (prx role.rs
    /// `apply_revoke`).
    AdmitRoleRevoke {
        revoker: DeviceIdValue,
        target: DeviceIdValue,
    },
    /// Admit a `Kick` authored by `kicker` ejecting `target` — strips the
    /// target's granted authority and records a transient kick, gated by the
    /// same `can_grant` predicate against the target's tier (a regular member
    /// is a Voice-tier target). The Founder is never kickable (RFC 1459
    /// §4.2.8; prx role.rs `apply_kick`).
    AdmitKick {
        kicker: DeviceIdValue,
        target: DeviceIdValue,
    },
    /// Admit a `Ban` authored by `banner` against `target` — the durable
    /// counterpart of a kick: it entails the kick (strips authority, records
    /// the kick) and additionally records a ban that refuses the target's
    /// rejoin until lifted. Same authority gate as `Kick`; the Founder is
    /// never bannable (RFC 2811 §4.3.1; prx role.rs `apply_ban`).
    AdmitBan {
        banner: DeviceIdValue,
        target: DeviceIdValue,
    },
    /// Admit an `Unban` authored by `lifter` — the symmetric lift of a ban.
    /// Founder-or-Operator authority (`can_grant` against the Voice tier);
    /// idempotent on a non-banned target. Does not clear a kick (prx role.rs
    /// `apply_unban`; RFC 2811 §4.3.1 `-b`).
    AdmitUnban {
        lifter: DeviceIdValue,
        target: DeviceIdValue,
    },
    /// Admit an `Unkick` authored by `lifter` — the explicit lift of a kick,
    /// prx's symmetric counterpart to `Kick` (IRC has no UNKICK: a kick there
    /// lapses only on a fresh accepted join). Founder-or-Operator authority;
    /// idempotent on a non-kicked target (prx role.rs `apply_unkick`).
    AdmitUnkick {
        lifter: DeviceIdValue,
        target: DeviceIdValue,
    },
    /// Admit a `ChannelModeChange` authored by `actor` — a runtime amendment
    /// of the effective modes under the existing constitution (the channel's
    /// identity is untouched). Founder-or-Operator authority (RFC 2811 §4.2
    /// MODE; Lessig 1999 code-as-law; prx role.rs `apply_mode_change`).
    AdmitModeChange {
        actor: DeviceIdValue,
        modes: ChannelModeSet,
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

/// The moderation authority gate shared by revoke, kick, and ban (prx role.rs:
/// all three gate on `Role::can_grant` against the target's current tier). The
/// `actor` must hold a role, and must hold grant-authority over `target_tier`
/// given the channel's `op_can_grant_op` mode. The Founder tier is never a
/// valid target — `can_grant(_, Founder)` is false — so the Founder is
/// structurally unstrippable (Saltzer & Schroeder 1975 least privilege;
/// RFC 2811 §4.1).
fn may_moderate(
    situation: &ChannelSituation,
    actor: &DeviceIdValue,
    target_tier: ChannelRole,
) -> Result<(), String> {
    let actor_role = situation.role_of(actor).ok_or_else(|| {
        "moderation dropped: actor holds no authority in this channel".to_string()
    })?;
    if actor_role.can_grant(target_tier, situation.modes.op_can_grant_op) {
        Ok(())
    } else {
        Err(
            "moderation dropped: actor lacks grant-authority over the target's tier (prx Role::can_grant; Saltzer & Schroeder 1975)"
                .to_string(),
        )
    }
}

/// The Founder tier is constitutive (pinned in the manifest); no moderation
/// action may strip it (prx: founder untouchable). Redundant with
/// `can_grant(_, Founder) == false`, but made explicit so the invariant is
/// legible at each call site and carries a precise drop reason.
fn refuse_if_founder(situation: &ChannelSituation, target: &DeviceIdValue) -> Result<(), String> {
    if situation.role_of(target) == Some(ChannelRole::Founder) {
        Err(
            "moderation dropped: the Founder tier is constitutive and cannot be stripped"
                .to_string(),
        )
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
            // A ban is durable: it refuses even a fresh join until lifted
            // (RFC 2811 §4.3.1 ban masks; prx `is_banned`).
            if situation.is_banned(device) {
                return Err(
                    "join dropped: device is banned; rejoin refused until Unban (RFC 2811 §4.3.1)"
                        .to_string(),
                );
            }
            let mut next = situation.clone();
            // A fresh accepted join clears a prior (transient) kick
            // (prx `clear_kick`; RFC 1459 §4.2.8).
            next.kicked.retain(|d| d != device);
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
            let granter_role = situation.role_of(granter).ok_or_else(|| {
                "role grant dropped: granter holds no authority in this channel".to_string()
            })?;
            // Grant authority (prx `Role::can_grant`; RFC 2811 §4.1; Saltzer &
            // Schroeder 1975): the Founder grants anything below, an Operator
            // grants Voice always and Operator only when op_can_grant_op is
            // set, Voice grants nothing, nobody grants Founder.
            if granter_role.can_grant(*role, situation.modes.op_can_grant_op) {
                let mut next = situation.clone();
                next.roles.retain(|(d, _)| d != grantee);
                next.roles.push((*grantee, *role));
                Ok(next)
            } else {
                Err(
                    "role grant dropped: granter lacks grant-authority over the target tier (prx Role::can_grant)"
                        .to_string(),
                )
            }
        }
        ChannelAction::AdmitRoleRevoke { revoker, target } => {
            gate_zero(situation, revoker)?;
            refuse_if_founder(situation, target)?;
            // The actor must hold a role even to no-op-revoke (prx order:
            // actor-authority is checked before the idempotent branch).
            if situation.role_of(revoker).is_none() {
                return Err(
                    "role revoke dropped: revoker holds no authority in this channel".to_string(),
                );
            }
            let Some(target_role) = situation.role_of(target) else {
                // Idempotent: revoking a role that isn't held is a no-op
                // accept (prx `apply_revoke`; mirrors LinkOutcome::AlreadyKnown).
                return Ok(situation.clone());
            };
            may_moderate(situation, revoker, target_role)?;
            let mut next = situation.clone();
            next.roles.retain(|(d, _)| d != target);
            Ok(next)
        }
        ChannelAction::AdmitKick { kicker, target } => {
            gate_zero(situation, kicker)?;
            refuse_if_founder(situation, target)?;
            // A regular member (no role) is a Voice-tier target: kicking a
            // plain user is a Voice-tier action (prx `apply_kick`).
            let target_tier = situation.role_of(target).unwrap_or(ChannelRole::Voice);
            may_moderate(situation, kicker, target_tier)?;
            let mut next = situation.clone();
            // A kick strips the target's granted authority (RFC 1459 §4.2.8;
            // prx `apply_kick` removes the target's grant) and records the
            // transient kick.
            next.roles.retain(|(d, _)| d != target);
            if !next.is_kicked(target) {
                next.kicked.push(*target);
            }
            Ok(next)
        }
        ChannelAction::AdmitBan { banner, target } => {
            gate_zero(situation, banner)?;
            refuse_if_founder(situation, target)?;
            let target_tier = situation.role_of(target).unwrap_or(ChannelRole::Voice);
            may_moderate(situation, banner, target_tier)?;
            let mut next = situation.clone();
            // A ban entails a kick (strip authority + record kick) and
            // additionally records the durable ban (prx `apply_ban`:
            // grants.remove + kicked.insert + banned.insert).
            next.roles.retain(|(d, _)| d != target);
            if !next.is_kicked(target) {
                next.kicked.push(*target);
            }
            if !next.is_banned(target) {
                next.banned.push(*target);
            }
            Ok(next)
        }
        ChannelAction::AdmitUnban { lifter, target } => {
            gate_zero(situation, lifter)?;
            // Founder-or-Operator authority — same bar as the ban itself
            // (prx `apply_unban`: `can_grant(Voice)`).
            may_moderate(situation, lifter, ChannelRole::Voice)?;
            let mut next = situation.clone();
            // Idempotent lift; the kicked-set is NOT cleared here — the
            // device's rejoin clears that separately (prx `apply_unban`).
            next.banned.retain(|d| d != target);
            Ok(next)
        }
        ChannelAction::AdmitUnkick { lifter, target } => {
            gate_zero(situation, lifter)?;
            may_moderate(situation, lifter, ChannelRole::Voice)?;
            let mut next = situation.clone();
            next.kicked.retain(|d| d != target);
            Ok(next)
        }
        ChannelAction::AdmitModeChange { actor, modes } => {
            gate_zero(situation, actor)?;
            // Channel-wide changes outrank a single user's role, so the bar is
            // Founder-or-Operator (prx `apply_mode_change`: `can_grant(Voice)`
            // against the CURRENT effective modes).
            may_moderate(situation, actor, ChannelRole::Voice)?;
            let mut next = situation.clone();
            // The amendment overrides the effective modes UNDER the existing
            // constitution: the channel's identity is left untouched (Hart
            // 1961 secondary rules; Lessig 1999). `next.channel` is unchanged.
            next.modes = *modes;
            Ok(next)
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
