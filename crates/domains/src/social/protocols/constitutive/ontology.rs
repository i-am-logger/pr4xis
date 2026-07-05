//! ConstitutiveProtocol — the conceptual layer of the prx p2p trust protocol.
//!
//! Re-manifests the prose ontology of the prx protocol (its `docs/ontology.md`)
//! as a pr4xis `ontology!` block: six top-level categories — Identity,
//! Authority, Constitution, Praxis, Membership, Slashing — plus the
//! axiom-support concepts (transport addresses, signature schemes) that the
//! protocol's two non-negotiable axioms (`Ipv6Only`, `PostQuantumOnly`)
//! quantify over.
//!
//! # Literature
//!
//! - **Searle (1969)** *Speech Acts* — the illocutionary taxonomy that maps
//!   onto the fourteen praxis event types (via the existing
//!   `cognitive::linguistics::pragmatics::speech_act::SearleCategory`).
//! - **Austin (1962)** *How to Do Things With Words* — performative
//!   utterances; a `RoleGrant` is true by being said (published, signed).
//! - **Oikarinen & Reed (1993)** RFC 1459 / **Kalt (2000)** RFC 2811 — IRC
//!   channel management: the KICK command (§4.2.8), the +b ban mask
//!   (RFC 2811 §4.3.1), and the MODE command (§4.2) that the five moderation
//!   / mode event types re-manifest.
//! - **Hart (1961)** *The Concept of Law* — primary vs. secondary rules; the
//!   `Constitution` is the protocol's secondary-rule layer.
//! - **Lamport (1979)** SRI CSL-98 — to be able to sign is what an identity is.
//! - **Lampson (1971/1974)** ACM OSR 8(1) — the access matrix behind
//!   `Authority` as a (Channel, Identity, Role) triple.
//! - **Buterin & Griffith (2017)** arXiv:1710.09437 — slashing conditions as
//!   cryptographic penalties.
//! - **Li, Krohn, Mazieres & Shasha (2004)** OSDI SUNDR — fork-consistency as
//!   the operational definition of equivocation.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// Reuse — never redefine — the Searle illocutionary taxonomy already encoded
// in the pragmatics ontology (Searle 1969; Searle 1976 taxonomy).
use crate::cognitive::linguistics::pragmatics::speech_act::SearleCategory;

pr4xis::ontology! {
    name: "ConstitutiveProtocol",
    source: "Searle (1969); Austin (1962); Hart (1961); Lamport (1979); Lampson (1971/1974); Buterin & Griffith (2017); Li, Krohn, Mazieres & Shasha (2004)",

    concepts: [
        // === Identity cluster (prx ontology §2) ===
        Identity,
        Seed,
        AuthoringAgency,
        ConstitutiveAgency,
        DeviceId,

        // === Authority cluster (prx ontology §3) ===
        Authority,
        Role,
        FounderRole,
        OperatorRole,
        VoiceRole,
        ChannelModes,
        RoleGrant,
        RoleRevoke,
        Kick,
        Ban,
        Unban,
        Unkick,
        ChannelModeChange,

        // === Constitution cluster (prx ontology §4) ===
        Constitution,
        ChannelManifest,
        ChannelId,

        // === Praxis cluster (prx ontology §5) ===
        PraxisEvent,
        Chain,
        Seq,
        ChannelJoin,
        Leave,
        ProfileUpdate,
        StreamOpen,
        StreamChunk,
        StreamClose,
        ClientPolicy,

        // === Membership cluster (prx ontology §6) ===
        Membership,

        // === Slashing cluster (prx ontology §7) ===
        Slashing,
        Equivocation,
        ForkProof,
        SlashRegistry,

        // === Axiom-support concepts (prx ontology §0, Axioms A1/A2) ===
        TransportAddress,
        Ipv6Address,
        Ipv4Address,
        SignatureScheme,
        MlDsa,
        SlhDsa,
        Ed25519,
    ],

    labels: {
        Identity: ("en", "Identity", "The triple (seed, authoring agency, constitutive agency): to be able to sign is what an identity is in a public-key world. Lamport (1979) SRI CSL-98; Diffie & Hellman (1976) IEEE TIT 22(6)."),
        Seed: ("en", "Seed", "The 256-bit secret entropy from which both cryptographic agencies are deterministically derived; axiom-neutral entropy. prx ontology, Identity cluster."),
        AuthoringAgency: ("en", "Authoring agency", "The capacity to author events authentically; the verb is 'to author'. Realised by ML-DSA-65 (NIST FIPS 204, 2024)."),
        ConstitutiveAgency: ("en", "Constitutive agency", "The capacity to constitute a channel — to bring it into being by signing its manifest; the verb is 'to constitute'. Realised by SLH-DSA-SHA2-128s (NIST FIPS 205, 2024)."),
        DeviceId: ("en", "Device id", "The external name of an identity: blake3 of the authoring public key. Not the identity itself, just the handle peers use — the meaning of a word is its use in the language, Wittgenstein (1953) §43."),
        Authority: ("en", "Authority", "A (Channel, Identity, Role) triple specifying what an identity may do within a channel. Lampson (1971/1974) ACM OSR 8(1): the access matrix — subject x object -> right."),
        Role: ("en", "Role", "The closed-set authority tier: Founder over Operator over Voice, a strict total order on rank. Closed set per Saltzer & Schroeder (1975) IEEE Proc 63(9) least privilege; RFC 1459 (Oikarinen & Reed 1993); RFC 2811 (Kalt 2000)."),
        FounderRole: ("en", "Founder role", "The constitutive tier — held by the identity whose constitutive agency signed the channel manifest; top of the ladder. RFC 2811 (Kalt 2000), channel creator."),
        OperatorRole: ("en", "Operator role", "The operational tier — may grant lower tiers and moderate praxis; the IRC channel operator. RFC 1459 (Oikarinen & Reed 1993), mode +o."),
        VoiceRole: ("en", "Voice role", "The participation tier — may speak in moderated channels; the IRC voice. RFC 1459 (Oikarinen & Reed 1993), mode +v."),
        ChannelModes: ("en", "Channel modes", "The modulating bits on a channel's manifest — constraints on how authorities are exercised, not authorities themselves. Lessig (1999): code is law — modes are the constitutional law of the channel."),
        RoleGrant: ("en", "Role grant", "Performative authority creation: the role exists in the channel by virtue of the signed declaration — saying it is what makes it true. Austin (1962)."),
        RoleRevoke: ("en", "Role revoke", "Performative authority destruction; the symmetric counterpart of the grant. Austin (1962)."),
        Kick: ("en", "Kick", "Forcible removal of a member by a channel operator; the removed member may rejoin subject to the channel's join policy — a transient moderation act. Oikarinen & Reed (1993) RFC 1459 §4.2.8 (KICK command)."),
        Ban: ("en", "Ban", "Durable exclusion of an identity from rejoining, recorded as a ban entry on the channel; in IRC the +b ban mask channel mode, which refuses matching devices including a fresh join. Kalt (2000) RFC 2811 §4.3.1 (Channel Ban and Exception); Oikarinen & Reed (1993) RFC 1459 §4.2.3 (MODE, +b ban mask)."),
        Unban: ("en", "Unban", "The symmetric lift of a Ban — removes the ban entry (-b in IRC), restoring the identity's eligibility to rejoin. Kalt (2000) RFC 2811 §4.3.1 (-b); Oikarinen & Reed (1993) RFC 1459 §4.2.3 (MODE)."),
        Unkick: ("en", "Unkick", "The explicit lift of a Kick prior to rejoin. IRC has no UNKICK command — a kick there lapses only on a fresh accepted join; this is prx's explicit symmetric counterpart, grounded in the same moderation authority (RFC 1459 §4.2.8) and Austin's performative symmetry (like RoleRevoke to RoleGrant). Oikarinen & Reed (1993) RFC 1459 §4.2.8; Austin (1962)."),
        ChannelModeChange: ("en", "Channel mode change", "A runtime change to the channel's effective modes under the existing constitution — the MODE command — without reissuing the channel under a new id (which would lose history). Names the EVENT; the state it amends is the ChannelModes concept. Kalt (2000) RFC 2811 §4.2 / Oikarinen & Reed (1993) RFC 1459 §4.2.3 (MODE command); Lessig (1999) code-as-law."),
        Constitution: ("en", "Constitution", "The founding act of a channel: a manifest signed by a constitutive agency; once constituted, amendments happen under it as praxis events. Hart (1961) secondary rules; Schmitt (1928) Verfassungslehre, constitutive vs. constituted power (cited honestly — see citings.md)."),
        ChannelManifest: ("en", "Channel manifest", "The constitutive document: the founder's constitutive public key, the channel modes, and an SLH-DSA signature over the canonical bytes."),
        ChannelId: ("en", "Channel id", "blake3 of the canonical manifest bytes — the channel's unforgeable external name, containing its constitution by reference."),
        PraxisEvent: ("en", "Praxis event", "A signed event authored under a constitution — the lived activity of the channel. Aristotle, Nicomachean Ethics VI (praxis as purposeful activity); Habermas (1981) communicative action."),
        Chain: ("en", "Chain", "The per-device linear order over events: monotonic seq, no gaps, no forks. Li, Krohn, Mazieres & Shasha (2004) OSDI SUNDR fork-consistency."),
        Seq: ("en", "Seq", "The monotonic per-device sequence number that orders a device's events into its chain."),
        ChannelJoin: ("en", "Channel join", "Declaration: brings a Membership relation into existence — saying it, signed and accepted, makes you a member. Searle (1969); Austin (1962)."),
        Leave: ("en", "Leave", "Declaration: dissolves the Membership relation; the symmetric counterpart of the join. Searle (1969)."),
        ProfileUpdate: ("en", "Profile update", "Assertive: asserts 'my display name is now X' — a truth-value bearer receivers may believe or doubt. Searle (1969)."),
        StreamOpen: ("en", "Stream open", "Commissive: commits the device to a stream of subsequent chunks — close cleanly or be detectably incomplete. Searle (1969)."),
        StreamChunk: ("en", "Stream chunk", "Assertive: asserts the next slice of stream content under the prior open's commitment. Searle (1969)."),
        StreamClose: ("en", "Stream close", "Declaration: brings the stream to a definitive end. Searle (1969)."),
        ClientPolicy: ("en", "Client policy", "Self-directed directive: the author binds future-themselves; verifiers enforce it as a self-imposed constraint — Ulysses' mast. Elster (1979) Ulysses and the Sirens; Searle (1969)."),
        Membership: ("en", "Membership", "The relation between an Identity and a Channel, established by a ChannelJoin accepted under that channel's Constitution. Locke (1689) consent as the ground of membership; Weyl, Ohlhaver & Buterin (2022) SSRN 4105763 soulbound tokens."),
        Slashing: ("en", "Slashing", "The structural penalty for a protocol-detectable violation — one whose proof of misbehaviour is a constructable artifact, not a verdict requiring social adjudication. Buterin & Griffith (2017) arXiv:1710.09437 (Casper)."),
        Equivocation: ("en", "Equivocation", "Same device, same seq, two different signed events — misbehaviour detectable from message inconsistency alone. Lamport, Shostak & Pease (1982) ACM TOPLAS 4(3); Li et al. (2004) SUNDR §3."),
        ForkProof: ("en", "Fork proof", "The constructable equivocation proof: the existence of the pair IS the proof; every peer who sees both halves reaches the same verdict. Li et al. (2004) SUNDR §3."),
        SlashRegistry: ("en", "Slash registry", "The receiver-side mirror of every observed slash; once a slash is recorded, the device is excluded from the receiver's pipeline (gate 0)."),
        TransportAddress: ("en", "Transport address", "A network locator at which a peer can be reached — the transport layer's naming surface, quantified over by prx Axiom A1."),
        Ipv6Address: ("en", "IPv6 address", "An IPv6 transport address: per-session source-address rotation under SLAAC removes the stable-IP fingerprint that v4+NAT always carries. RFC 8981 (2021)."),
        Ipv4Address: ("en", "IPv4 address", "An IPv4 transport address: carries a stable-IP fingerprint; admitted only as a developer-environment loopback exception, never as durable addressing. prx Axiom A1; RFC 8981 (2021) contrast."),
        SignatureScheme: ("en", "Signature scheme", "A digital signature scheme — the cryptographic realisation of an agency. Lamport (1979) SRI CSL-98."),
        MlDsa: ("en", "ML-DSA-65", "The lattice-based authoring scheme: signs every event a device authors. NIST FIPS 204 (2024)."),
        SlhDsa: ("en", "SLH-DSA-SHA2-128s", "The hash-based constituting scheme: the most conservative cryptographic assumption available, chosen because the constitutive slot is the channel's root of trust. NIST FIPS 205 (2024)."),
        Ed25519: ("en", "ed25519", "Transient transport-identity only: authenticates live connections, never durable claims. prx Axiom A2."),
    },

    is_a: [
        // The three-tier authority ladder (RFC 1459 / RFC 2811).
        (FounderRole, Role),
        (OperatorRole, Role),
        (VoiceRole, Role),

        // The fourteen praxis event types (prx ontology §5.3 table +
        // the moderation / mode events RFC 1459 §4.2.8 / RFC 2811 §4).
        (ChannelJoin, PraxisEvent),
        (Leave, PraxisEvent),
        (ProfileUpdate, PraxisEvent),
        (StreamOpen, PraxisEvent),
        (StreamChunk, PraxisEvent),
        (StreamClose, PraxisEvent),
        (ClientPolicy, PraxisEvent),
        (RoleGrant, PraxisEvent),
        (RoleRevoke, PraxisEvent),
        (Kick, PraxisEvent),
        (Ban, PraxisEvent),
        (Unban, PraxisEvent),
        (Unkick, PraxisEvent),
        (ChannelModeChange, PraxisEvent),

        // Transport addresses (prx Axiom A1 support).
        (Ipv6Address, TransportAddress),
        (Ipv4Address, TransportAddress),

        // Signature schemes (prx Axiom A2 support).
        (MlDsa, SignatureScheme),
        (SlhDsa, SignatureScheme),
        (Ed25519, SignatureScheme),
    ],

    has_a: [
        // Identity is the triple (seed, AuthoringAgency, ConstitutiveAgency).
        (Identity, Seed),
        (Identity, AuthoringAgency),
        (Identity, ConstitutiveAgency),
        // The constitution carries the constitutive document.
        (Constitution, ChannelManifest),
        // Every praxis event carries its per-device sequence number.
        (PraxisEvent, Seq),
    ],

    edges: [
        // Both agencies are deterministically derived from the seed (prx §2.1).
        (Seed, AuthoringAgency, Derives),
        (Seed, ConstitutiveAgency, Derives),

        // External names: blake3 content addresses (prx §2.3, §4.3).
        (Identity, DeviceId, NamedBy),
        (Constitution, ChannelId, NamedBy),

        // Cross-cutting relations (prx §8).
        (Identity, PraxisEvent, Authors),
        (ConstitutiveAgency, Constitution, Constitutes),
        (Constitution, Authority, Founds),
        (Authority, Role, Confers),
        (Authority, PraxisEvent, Gates),
        (PraxisEvent, Chain, OrderedInto),

        // Membership lifecycle (prx §6).
        (ChannelJoin, Membership, Establishes),
        (Leave, Membership, Dissolves),

        // Slashing chain (prx §7).
        (Equivocation, ForkProof, ProvenBy),
        (ForkProof, Slashing, Triggers),
        (Slashing, PraxisEvent, ExcludesFrom),
        (SlashRegistry, Slashing, Records),

        // Modes constrain how authorities are exercised (prx §3.3; Lessig 1999).
        (ChannelModes, Authority, Modulates),

        // Moderation edges (RFC 1459 §4.2.8; RFC 2811 §4.3.1; prx role.rs).
        // A kick strips the target's granted authority (prx apply_kick removes
        // the target's role grant).
        (Kick, Authority, Strips),
        // A ban entails a kick — it does everything the kick does plus records
        // a durable ban (prx apply_ban: grants.remove + kicked.insert +
        // banned.insert).
        (Ban, Kick, Entails),
        // The explicit symmetric lifts (prx apply_unkick / apply_unban).
        (Unkick, Kick, Lifts),
        (Unban, Ban, Lifts),
        // A mode change amends the channel's modes under the existing
        // constitution (prx apply_mode_change sets the effective-modes
        // override; Hart 1961 secondary rules; Lessig 1999).
        (ChannelModeChange, ChannelModes, Amends),
    ],
}

// ---------------------------------------------------------------------------
// Typed values
// ---------------------------------------------------------------------------

/// Position on the closed role ladder — the RFC 1459 / RFC 2811 tier ordinal.
///
/// Higher ordinal = more authority. A newtype (not a bare `u8`) so ladder
/// positions are only ever compared with each other; the strict total order
/// `Founder > Operator > Voice` is Saltzer & Schroeder (1975) least privilege
/// applied to IRC's closed authority set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RankOrdinal(pub u8);

/// Founder's ladder position: the constitutive tier, above operator.
/// RFC 2811 (Kalt 2000) §4: the channel creator outranks channel operators.
/// Third (topmost) rung of the three-tier ladder.
pub const FOUNDER_RANK: RankOrdinal = RankOrdinal(3);

/// Operator's ladder position: RFC 1459 (Oikarinen & Reed 1993) mode +o,
/// above voice. Second rung of the three-tier ladder.
pub const OPERATOR_RANK: RankOrdinal = RankOrdinal(2);

/// Voice's ladder position: RFC 1459 (Oikarinen & Reed 1993) mode +v.
/// First (lowest) rung of the three-tier ladder.
pub const VOICE_RANK: RankOrdinal = RankOrdinal(1);

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The illocutionary force of each praxis event type (prx ontology §5.3).
///
/// Value type is the EXISTING `SearleCategory` from the pragmatics ontology
/// (Searle 1969; Austin 1962) — the protocol's event taxonomy maps onto the
/// speech-act taxonomy rather than redefining it. `None` for every concept
/// that is not one of the fourteen event types.
#[derive(Debug, Clone)]
pub struct IllocutionaryForce;

impl Quality for IllocutionaryForce {
    type Individual = ConstitutiveProtocolConcept;
    type Value = SearleCategory;

    fn get(&self, c: &ConstitutiveProtocolConcept) -> Option<SearleCategory> {
        use ConstitutiveProtocolConcept as C;
        match c {
            // Declarations: performative — saying it (signed, accepted) makes
            // it so. The five moderation / mode events join this family: a
            // signed, accepted kick / ban / unban / unkick / mode change makes
            // the exclusion or amendment true by being uttered (Austin 1962).
            C::ChannelJoin
            | C::Leave
            | C::StreamClose
            | C::RoleGrant
            | C::RoleRevoke
            | C::Kick
            | C::Ban
            | C::Unban
            | C::Unkick
            | C::ChannelModeChange => Some(SearleCategory::Declaration),
            // Assertives: truth-value bearers receivers may believe or doubt.
            C::ProfileUpdate | C::StreamChunk => Some(SearleCategory::Assertive),
            // Commissive: commits the author to subsequent chunks.
            C::StreamOpen => Some(SearleCategory::Commissive),
            // Self-directed directive: Ulysses' mast (Elster 1979).
            C::ClientPolicy => Some(SearleCategory::Directive),
            _ => None,
        }
    }
}

/// Whether a signature scheme's security holds against quantum adversaries
/// (prx Axiom A2 support). `None` for non-scheme concepts.
#[derive(Debug, Clone)]
pub struct IsPostQuantum;

impl Quality for IsPostQuantum {
    type Individual = ConstitutiveProtocolConcept;
    type Value = bool;

    fn get(&self, c: &ConstitutiveProtocolConcept) -> Option<bool> {
        use ConstitutiveProtocolConcept as C;
        match c {
            // Lattice-based, NIST FIPS 204 (2024).
            C::MlDsa => Some(true),
            // Hash-based, NIST FIPS 205 (2024).
            C::SlhDsa => Some(true),
            // Elliptic-curve; broken by Shor's algorithm — transient use only.
            C::Ed25519 => Some(false),
            _ => None,
        }
    }
}

/// Whether a signature scheme signs DURABLE claims on the wire (prx Axiom A2:
/// authoring and constituting are durable; ed25519 authenticates only live
/// connections). `None` for non-scheme concepts.
#[derive(Debug, Clone)]
pub struct SignsDurably;

impl Quality for SignsDurably {
    type Individual = ConstitutiveProtocolConcept;
    type Value = bool;

    fn get(&self, c: &ConstitutiveProtocolConcept) -> Option<bool> {
        use ConstitutiveProtocolConcept as C;
        match c {
            // Authoring: signs every event the device authors.
            C::MlDsa => Some(true),
            // Constituting: signs the channel manifest — the root of trust.
            C::SlhDsa => Some(true),
            // Transient transport identity only; never durable claims.
            C::Ed25519 => Some(false),
            _ => None,
        }
    }
}

/// Whether a transport address family is admitted as DURABLE peer addressing
/// (prx Axiom A1). IPv6 rotates source addresses per RFC 8981, removing the
/// stable-IP fingerprint; v4 loopback is a dev-environment exception, not
/// durable addressing. `None` for non-address concepts.
#[derive(Debug, Clone)]
pub struct DurablyAddressable;

impl Quality for DurablyAddressable {
    type Individual = ConstitutiveProtocolConcept;
    type Value = bool;

    fn get(&self, c: &ConstitutiveProtocolConcept) -> Option<bool> {
        use ConstitutiveProtocolConcept as C;
        match c {
            C::Ipv6Address => Some(true),
            C::Ipv4Address => Some(false),
            _ => None,
        }
    }
}

/// The ladder position of each role concept (RFC 1459 / RFC 2811;
/// Saltzer & Schroeder 1975 least privilege). `None` for non-role concepts —
/// including the abstract `Role` parent, which has no single rank.
#[derive(Debug, Clone)]
pub struct RoleRank;

impl Quality for RoleRank {
    type Individual = ConstitutiveProtocolConcept;
    type Value = RankOrdinal;

    fn get(&self, c: &ConstitutiveProtocolConcept) -> Option<RankOrdinal> {
        use ConstitutiveProtocolConcept as C;
        match c {
            C::FounderRole => Some(FOUNDER_RANK),
            C::OperatorRole => Some(OPERATOR_RANK),
            C::VoiceRole => Some(VOICE_RANK),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The fourteen praxis event concepts — the rows of the prx ontology §5.3
/// table plus the five moderation / mode events (RFC 1459 §4.2.8;
/// RFC 2811 §4).
pub fn praxis_event_concepts() -> Vec<ConstitutiveProtocolConcept> {
    use ConstitutiveProtocolConcept as C;
    vec![
        C::ChannelJoin,
        C::Leave,
        C::ProfileUpdate,
        C::StreamOpen,
        C::StreamChunk,
        C::StreamClose,
        C::ClientPolicy,
        C::RoleGrant,
        C::RoleRevoke,
        C::Kick,
        C::Ban,
        C::Unban,
        C::Unkick,
        C::ChannelModeChange,
    ]
}

fn kinded_edge_exists(
    from: ConstitutiveProtocolConcept,
    to: ConstitutiveProtocolConcept,
    kind: ConstitutiveProtocolRelationKind,
) -> bool {
    ConstitutiveProtocolCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

fn verdict_from(axiom: &dyn Axiom, ok: bool) -> Verdict {
    if ok {
        Ok(Box::new(SimpleProof::new(axiom.meta())))
    } else {
        Err(Box::new(SimpleCounterexample::new(axiom.meta())))
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// prx Axiom A1 — every durably-addressable transport-address concept is
/// `Ipv6Address`; v4 loopback is a dev-environment exception, not durable
/// addressing.
pub struct Ipv6Only;

impl Axiom for Ipv6Only {
    fn verify(&self) -> Verdict {
        use ConstitutiveProtocolConcept as C;
        let q = DurablyAddressable;
        // Universal: anything durably addressable is the IPv6 concept.
        let only_v6 = ConstitutiveProtocolConcept::variants()
            .into_iter()
            .all(|c| q.get(&c) != Some(true) || c == C::Ipv6Address);
        // Witness: IPv6 actually is durably addressable, and both address
        // concepts are transport addresses.
        let witness = q.get(&C::Ipv6Address) == Some(true)
            && kinded_edge_exists(
                C::Ipv6Address,
                C::TransportAddress,
                ConstitutiveProtocolRelationKind::Subsumption,
            )
            && kinded_edge_exists(
                C::Ipv4Address,
                C::TransportAddress,
                ConstitutiveProtocolRelationKind::Subsumption,
            );
        verdict_from(self, only_v6 && witness)
    }

    pr4xis::axiom_meta!(
        "Ipv6Only",
        "every transport-address concept with DurablyAddressable = true is Ipv6Address; v4 loopback is a dev-environment exception, not durable addressing",
        "RFC 8981 (2021) Temporary Address Extensions for Stateless Address Autoconfiguration in IPv6; prx Axiom A1"
    );
}
pr4xis::register_axiom!(
    Ipv6Only,
    "RFC 8981 (2021) Temporary Address Extensions for Stateless Address Autoconfiguration in IPv6; prx Axiom A1"
);

/// prx Axiom A2 — every scheme that signs durable claims is post-quantum;
/// ed25519 appears only transiently at the transport-identity layer.
pub struct PostQuantumOnly;

impl Axiom for PostQuantumOnly {
    fn verify(&self) -> Verdict {
        use ConstitutiveProtocolConcept as C;
        let durable = SignsDurably;
        let pq = IsPostQuantum;
        // Universal: durable signer => post-quantum.
        let implication = ConstitutiveProtocolConcept::variants()
            .into_iter()
            .all(|c| durable.get(&c) != Some(true) || pq.get(&c) == Some(true));
        // Witnesses: both durable slots are occupied and PQ; ed25519 is neither.
        let witness = durable.get(&C::MlDsa) == Some(true)
            && durable.get(&C::SlhDsa) == Some(true)
            && durable.get(&C::Ed25519) == Some(false)
            && pq.get(&C::Ed25519) == Some(false);
        verdict_from(self, implication && witness)
    }

    pr4xis::axiom_meta!(
        "PostQuantumOnly",
        "every signature scheme with SignsDurably = true has IsPostQuantum = true; ed25519 is transient transport identity only",
        "NIST FIPS 204 (2024) ML-DSA; NIST FIPS 205 (2024) SLH-DSA; prx Axiom A2"
    );
}
pr4xis::register_axiom!(
    PostQuantumOnly,
    "NIST FIPS 204 (2024) ML-DSA; NIST FIPS 205 (2024) SLH-DSA; prx Axiom A2"
);

/// prx §8.1 — every event in Praxis is authored by an Identity: the Authors
/// edge exists AND each of the fourteen event concepts descends from PraxisEvent.
pub struct EveryEventAuthored;

impl Axiom for EveryEventAuthored {
    fn verify(&self) -> Verdict {
        use ConstitutiveProtocolConcept as C;
        let authors_edge = kinded_edge_exists(
            C::Identity,
            C::PraxisEvent,
            ConstitutiveProtocolRelationKind::Authors,
        );
        let all_events_subsumed = praxis_event_concepts().into_iter().all(|e| {
            kinded_edge_exists(
                e,
                C::PraxisEvent,
                ConstitutiveProtocolRelationKind::Subsumption,
            )
        });
        verdict_from(self, authors_edge && all_events_subsumed)
    }

    pr4xis::axiom_meta!(
        "EveryEventAuthored",
        "the Authors edge Identity -> PraxisEvent exists and each of the fourteen event concepts is a Subsumption-descendant of PraxisEvent",
        "Lamport (1979) SRI CSL-98; Austin (1962)"
    );
}
pr4xis::register_axiom!(
    EveryEventAuthored,
    "Lamport (1979) SRI CSL-98; Austin (1962)"
);

/// prx §8.2 / Hart (1961) — all authority descends from the founding act:
/// ConstitutiveAgency constitutes the Constitution, which founds Authority.
pub struct ConstitutionFoundsAllAuthority;

impl Axiom for ConstitutionFoundsAllAuthority {
    fn verify(&self) -> Verdict {
        use ConstitutiveProtocolConcept as C;
        let ok = kinded_edge_exists(
            C::ConstitutiveAgency,
            C::Constitution,
            ConstitutiveProtocolRelationKind::Constitutes,
        ) && kinded_edge_exists(
            C::Constitution,
            C::Authority,
            ConstitutiveProtocolRelationKind::Founds,
        );
        verdict_from(self, ok)
    }

    pr4xis::axiom_meta!(
        "ConstitutionFoundsAllAuthority",
        "the Constitutes (ConstitutiveAgency -> Constitution) and Founds (Constitution -> Authority) edges are both present: all authority descends from the founding act",
        "Hart (1961) The Concept of Law, secondary rules"
    );
}
pr4xis::register_axiom!(
    ConstitutionFoundsAllAuthority,
    "Hart (1961) The Concept of Law, secondary rules"
);

/// prx §7 / §8.4 — slashing excludes from future praxis, structurally
/// (ProvenBy -> Triggers -> ExcludesFrom chain) AND operationally (engine
/// gate 0: after a fork proof is observed for device d, no further event
/// authored by d is admitted).
pub struct SlashingExcludesFromPraxis;

impl Axiom for SlashingExcludesFromPraxis {
    fn verify(&self) -> Verdict {
        use super::engine::{
            ChannelAction, ChannelIdValue, ChannelModeSet, ChannelRole, ChannelSituation,
            DEVICE_ID_BYTES, DeviceIdValue, EventDigest, ForkProofClaim, SeqNumber, apply_channel,
        };
        use ConstitutiveProtocolConcept as C;

        // Structural half: the slashing chain of edges.
        let chain = kinded_edge_exists(
            C::Equivocation,
            C::ForkProof,
            ConstitutiveProtocolRelationKind::ProvenBy,
        ) && kinded_edge_exists(
            C::ForkProof,
            C::Slashing,
            ConstitutiveProtocolRelationKind::Triggers,
        ) && kinded_edge_exists(
            C::Slashing,
            C::PraxisEvent,
            ConstitutiveProtocolRelationKind::ExcludesFrom,
        );
        if !chain {
            return verdict_from(self, false);
        }

        // Operational half: engine gate 0 over a documented structural
        // fixture. The byte patterns are arbitrary distinct fixture ids
        // standing in for blake3 digests; the seq is the fixture chain
        // position at which the equivocation pair is observed.
        let founder = DeviceIdValue([1; DEVICE_ID_BYTES]);
        let joiner = DeviceIdValue([2; DEVICE_ID_BYTES]);
        let s0 = ChannelSituation::founded(
            ChannelIdValue([0; DEVICE_ID_BYTES]),
            founder,
            ChannelModeSet::default(),
        );
        let Ok(s1) = apply_channel(&s0, &ChannelAction::AdmitJoin { device: joiner }) else {
            return verdict_from(self, false);
        };

        // Two DIFFERENT signed events at the same seq: the pair is the proof.
        let proof = ForkProofClaim {
            device: joiner,
            seq: SeqNumber(1),
            first: EventDigest([3; DEVICE_ID_BYTES]),
            second: EventDigest([4; DEVICE_ID_BYTES]),
        };
        let Ok(s2) = apply_channel(&s1, &ChannelAction::ObserveForkProof { proof }) else {
            return verdict_from(self, false);
        };

        // Gate 0: every further event authored by the slashed device drops.
        let joiner_excluded = apply_channel(&s2, &ChannelAction::AdmitJoin { device: joiner })
            .is_err()
            && apply_channel(&s2, &ChannelAction::AdmitLeave { device: joiner }).is_err()
            && apply_channel(
                &s2,
                &ChannelAction::AdmitRoleGrant {
                    granter: joiner,
                    grantee: founder,
                    role: ChannelRole::Voice,
                },
            )
            .is_err();

        // Gate 0 precedes authority: even the Founder's grants drop once the
        // founder's own device is slashed.
        let founder_proof = ForkProofClaim {
            device: founder,
            seq: SeqNumber(1),
            first: EventDigest([5; DEVICE_ID_BYTES]),
            second: EventDigest([6; DEVICE_ID_BYTES]),
        };
        let Ok(s3) = apply_channel(
            &s2,
            &ChannelAction::ObserveForkProof {
                proof: founder_proof,
            },
        ) else {
            return verdict_from(self, false);
        };
        let founder_excluded = apply_channel(
            &s3,
            &ChannelAction::AdmitRoleGrant {
                granter: founder,
                grantee: joiner,
                role: ChannelRole::Voice,
            },
        )
        .is_err();

        verdict_from(self, joiner_excluded && founder_excluded)
    }

    pr4xis::axiom_meta!(
        "SlashingExcludesFromPraxis",
        "the ProvenBy -> Triggers -> ExcludesFrom edge chain is present, and in the engine no event authored by a device is admitted after a fork proof for that device is observed (gate 0)",
        "Li, Krohn, Mazieres & Shasha (2004) OSDI SUNDR §3; Buterin & Griffith (2017) arXiv:1710.09437"
    );
}
pr4xis::register_axiom!(
    SlashingExcludesFromPraxis,
    "Li, Krohn, Mazieres & Shasha (2004) OSDI SUNDR §3; Buterin & Griffith (2017) arXiv:1710.09437"
);

/// prx §5.3 — the illocutionary taxonomy is total over exactly the fourteen
/// event concepts and matches the published table (the nine original rows
/// plus the five moderation / mode declarations).
pub struct IllocutionaryForceTotal;

impl Axiom for IllocutionaryForceTotal {
    fn verify(&self) -> Verdict {
        use ConstitutiveProtocolConcept as C;
        let q = IllocutionaryForce;
        // The §5.3 table, row for row.
        let table = [
            (C::ChannelJoin, SearleCategory::Declaration),
            (C::Leave, SearleCategory::Declaration),
            (C::ProfileUpdate, SearleCategory::Assertive),
            (C::StreamOpen, SearleCategory::Commissive),
            (C::StreamChunk, SearleCategory::Assertive),
            (C::StreamClose, SearleCategory::Declaration),
            (C::ClientPolicy, SearleCategory::Directive),
            (C::RoleGrant, SearleCategory::Declaration),
            (C::RoleRevoke, SearleCategory::Declaration),
            // The five moderation / mode events are all Declarations: a
            // signed, accepted utterance makes the exclusion or amendment so.
            (C::Kick, SearleCategory::Declaration),
            (C::Ban, SearleCategory::Declaration),
            (C::Unban, SearleCategory::Declaration),
            (C::Unkick, SearleCategory::Declaration),
            (C::ChannelModeChange, SearleCategory::Declaration),
        ];
        let matches_table = table.iter().all(|(c, force)| q.get(c) == Some(*force));
        // Some for exactly the event concepts, None everywhere else.
        let events = praxis_event_concepts();
        let exactly_events = ConstitutiveProtocolConcept::variants()
            .into_iter()
            .all(|c| q.get(&c).is_some() == events.contains(&c));
        verdict_from(self, matches_table && exactly_events)
    }

    pr4xis::axiom_meta!(
        "IllocutionaryForceTotal",
        "IllocutionaryForce is Some for exactly the fourteen praxis event concepts and matches the prx §5.3 illocutionary table (nine original rows + five moderation/mode Declarations)",
        "Searle (1969) Speech Acts; Austin (1962) How to Do Things With Words"
    );
}
pr4xis::register_axiom!(
    IllocutionaryForceTotal,
    "Searle (1969) Speech Acts; Austin (1962) How to Do Things With Words"
);

/// prx §3.3 — the role ladder is a strict total order Founder > Operator >
/// Voice, defined for exactly the three role concepts.
pub struct RoleLadderTotalOrder;

impl Axiom for RoleLadderTotalOrder {
    fn verify(&self) -> Verdict {
        use ConstitutiveProtocolConcept as C;
        let q = RoleRank;
        let roles = [C::FounderRole, C::OperatorRole, C::VoiceRole];
        let exactly_roles = ConstitutiveProtocolConcept::variants()
            .into_iter()
            .all(|c| q.get(&c).is_some() == roles.contains(&c));
        let strictly_decreasing = FOUNDER_RANK > OPERATOR_RANK && OPERATOR_RANK > VOICE_RANK;
        verdict_from(self, exactly_roles && strictly_decreasing)
    }

    pr4xis::axiom_meta!(
        "RoleLadderTotalOrder",
        "RoleRank is Some for exactly the three role concepts and strictly decreasing Founder > Operator > Voice",
        "Oikarinen & Reed (1993) RFC 1459; Kalt (2000) RFC 2811; Saltzer & Schroeder (1975) IEEE Proc 63(9)"
    );
}
pr4xis::register_axiom!(
    RoleLadderTotalOrder,
    "Oikarinen & Reed (1993) RFC 1459; Kalt (2000) RFC 2811; Saltzer & Schroeder (1975) IEEE Proc 63(9)"
);

/// prx role.rs / RFC 1459 §4.2.8 — every admitted moderation act (Kick, Ban,
/// RoleRevoke) requires the actor to hold grant-authority over the target's
/// current tier (prx `Role::can_grant`), and the Founder tier is never
/// strippable.
///
/// The predicate is `can_grant`, not a raw "strictly superior rank": with the
/// channel's `op_can_grant_op` mode set, an Operator may moderate a fellow
/// Operator (equal rank) — RFC 2811 §4.1's channel-creator/operator exception.
/// With that mode clear (the default) `can_grant` reduces exactly to strict
/// rank superiority, so `prop_default_mode_moderation_is_strict_rank` records
/// the sharper "superior rank" guarantee for the default channel.
pub struct ModerationRequiresGrantAuthority;

impl Axiom for ModerationRequiresGrantAuthority {
    fn verify(&self) -> Verdict {
        use super::engine::{
            ChannelAction, ChannelIdValue, ChannelModeSet, ChannelRole, ChannelSituation,
            DEVICE_ID_BYTES, DeviceIdValue, apply_channel,
        };

        let roles = [
            ChannelRole::Founder,
            ChannelRole::Operator,
            ChannelRole::Voice,
        ];
        let actor = DeviceIdValue([2; DEVICE_ID_BYTES]);
        let target = DeviceIdValue([3; DEVICE_ID_BYTES]);

        // Exhaustive over (actor tier, target tier, op_can_grant_op): a kick,
        // ban, or revoke is admitted iff the actor can_grant over the target's
        // tier AND the target is not the Founder.
        for &actor_role in &roles {
            for &target_role in &roles {
                for &op_flag in &[false, true] {
                    let situation = ChannelSituation {
                        channel: ChannelIdValue([1; DEVICE_ID_BYTES]),
                        members: vec![actor, target],
                        roles: vec![(actor, actor_role), (target, target_role)],
                        kicked: Vec::new(),
                        banned: Vec::new(),
                        modes: ChannelModeSet {
                            op_can_grant_op: op_flag,
                        },
                        slashed: Vec::new(),
                    };
                    let expected = target_role != ChannelRole::Founder
                        && actor_role.can_grant(target_role, op_flag);
                    let kick_ok = apply_channel(
                        &situation,
                        &ChannelAction::AdmitKick {
                            kicker: actor,
                            target,
                        },
                    )
                    .is_ok();
                    let ban_ok = apply_channel(
                        &situation,
                        &ChannelAction::AdmitBan {
                            banner: actor,
                            target,
                        },
                    )
                    .is_ok();
                    let revoke_ok = apply_channel(
                        &situation,
                        &ChannelAction::AdmitRoleRevoke {
                            revoker: actor,
                            target,
                        },
                    )
                    .is_ok();
                    if kick_ok != expected || ban_ok != expected || revoke_ok != expected {
                        return verdict_from(self, false);
                    }
                }
            }
        }
        verdict_from(self, true)
    }

    pr4xis::axiom_meta!(
        "ModerationRequiresGrantAuthority",
        "in the engine a Kick/Ban/RoleRevoke is admitted iff the actor holds grant-authority (Role::can_grant) over the target's current tier and the target is not the Founder; with op_can_grant_op clear this is strict rank superiority",
        "Saltzer & Schroeder (1975) IEEE Proc 63(9) least privilege; Oikarinen & Reed (1993) RFC 1459 §4.2.8 (KICK); Kalt (2000) RFC 2811 §4.1 (member status)"
    );
}
pr4xis::register_axiom!(
    ModerationRequiresGrantAuthority,
    "Saltzer & Schroeder (1975) IEEE Proc 63(9) least privilege; Oikarinen & Reed (1993) RFC 1459 §4.2.8 (KICK); Kalt (2000) RFC 2811 §4.1 (member status)"
);

/// prx role.rs / RFC 2811 §4.3.1 — a ban is durable: after an admitted Ban the
/// banned device's fresh `ChannelJoin` is refused until an `Unban` lifts it,
/// whereas a kick alone does not block rejoin (a fresh join clears the kick).
pub struct BanExcludesRejoin;

impl Axiom for BanExcludesRejoin {
    fn verify(&self) -> Verdict {
        use super::engine::{
            ChannelAction, ChannelIdValue, ChannelModeSet, ChannelSituation, DEVICE_ID_BYTES,
            DeviceIdValue, apply_channel,
        };

        let founder = DeviceIdValue([1; DEVICE_ID_BYTES]);
        let member = DeviceIdValue([2; DEVICE_ID_BYTES]);
        let s0 = ChannelSituation::founded(
            ChannelIdValue([9; DEVICE_ID_BYTES]),
            founder,
            ChannelModeSet::default(),
        );

        // Member joins, then the founder bans them.
        let Ok(s1) = apply_channel(&s0, &ChannelAction::AdmitJoin { device: member }) else {
            return verdict_from(self, false);
        };
        let Ok(banned) = apply_channel(
            &s1,
            &ChannelAction::AdmitBan {
                banner: founder,
                target: member,
            },
        ) else {
            return verdict_from(self, false);
        };
        // Rejoin is refused while banned...
        let rejoin_refused =
            apply_channel(&banned, &ChannelAction::AdmitJoin { device: member }).is_err();
        // ...and admitted again once the ban is lifted.
        let Ok(unbanned) = apply_channel(
            &banned,
            &ChannelAction::AdmitUnban {
                lifter: founder,
                target: member,
            },
        ) else {
            return verdict_from(self, false);
        };
        let rejoin_after_unban =
            apply_channel(&unbanned, &ChannelAction::AdmitJoin { device: member }).is_ok();

        // A kick alone does NOT block rejoin, and a fresh join clears it.
        let Ok(kicked) = apply_channel(
            &s1,
            &ChannelAction::AdmitKick {
                kicker: founder,
                target: member,
            },
        ) else {
            return verdict_from(self, false);
        };
        let kick_blocks_rejoin =
            apply_channel(&kicked, &ChannelAction::AdmitJoin { device: member }).is_err();
        let Ok(rejoined) = apply_channel(&kicked, &ChannelAction::AdmitJoin { device: member })
        else {
            return verdict_from(self, false);
        };
        let kick_cleared_on_rejoin = !rejoined.is_kicked(&member);

        verdict_from(
            self,
            rejoin_refused && rejoin_after_unban && !kick_blocks_rejoin && kick_cleared_on_rejoin,
        )
    }

    pr4xis::axiom_meta!(
        "BanExcludesRejoin",
        "after an admitted Ban the banned device's fresh ChannelJoin is refused until Unban; a kick alone does not block rejoin (a fresh join clears the kick)",
        "Kalt (2000) RFC 2811 §4.3.1 (Channel Ban and Exception, +b ban masks); Oikarinen & Reed (1993) RFC 1459 §4.2.8 (KICK)"
    );
}
pr4xis::register_axiom!(
    BanExcludesRejoin,
    "Kalt (2000) RFC 2811 §4.3.1 (Channel Ban and Exception, +b ban masks); Oikarinen & Reed (1993) RFC 1459 §4.2.8 (KICK)"
);

/// prx role.rs / Hart (1961) — a mode change is a constituted amendment: it
/// overrides the effective modes UNDER the existing constitution and never
/// mints a new channel identity, leaving the constituted structure
/// (membership, roles) intact.
pub struct ModeChangeIsConstitutedAmendment;

impl Axiom for ModeChangeIsConstitutedAmendment {
    fn verify(&self) -> Verdict {
        use super::engine::{
            ChannelAction, ChannelIdValue, ChannelModeSet, ChannelSituation, DEVICE_ID_BYTES,
            DeviceIdValue, apply_channel,
        };

        let founder = DeviceIdValue([1; DEVICE_ID_BYTES]);
        let s0 = ChannelSituation::founded(
            ChannelIdValue([42; DEVICE_ID_BYTES]),
            founder,
            ChannelModeSet::default(),
        );
        let new_modes = ChannelModeSet {
            op_can_grant_op: true,
        };
        let Ok(s1) = apply_channel(
            &s0,
            &ChannelAction::AdmitModeChange {
                actor: founder,
                modes: new_modes,
            },
        ) else {
            return verdict_from(self, false);
        };

        // Identity untouched (amendment under the constitution, not a
        // re-founding), modes actually amended, constituted structure intact.
        let identity_preserved = s1.channel == s0.channel;
        let modes_amended = s1.modes == new_modes && s0.modes != new_modes;
        let structure_preserved = s1.members == s0.members && s1.roles == s0.roles;
        verdict_from(
            self,
            identity_preserved && modes_amended && structure_preserved,
        )
    }

    pr4xis::axiom_meta!(
        "ModeChangeIsConstitutedAmendment",
        "an admitted ChannelModeChange overrides the effective modes while leaving the channel identity and the constituted structure (membership, roles) untouched: amendment under the constitution, not re-founding",
        "Hart (1961) The Concept of Law, primary/secondary rules; Lessig (1999) Code and Other Laws of Cyberspace"
    );
}
pr4xis::register_axiom!(
    ModeChangeIsConstitutedAmendment,
    "Hart (1961) The Concept of Law, primary/secondary rules; Lessig (1999) Code and Other Laws of Cyberspace"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for ConstitutiveProtocolOntology {
    type Cat = ConstitutiveProtocolCategory;
    type Qual = IllocutionaryForce;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(Ipv6Only));
        axioms.push(Box::new(PostQuantumOnly));
        axioms.push(Box::new(EveryEventAuthored));
        axioms.push(Box::new(ConstitutionFoundsAllAuthority));
        axioms.push(Box::new(SlashingExcludesFromPraxis));
        axioms.push(Box::new(IllocutionaryForceTotal));
        axioms.push(Box::new(RoleLadderTotalOrder));
        axioms.push(Box::new(ModerationRequiresGrantAuthority));
        axioms.push(Box::new(BanExcludesRejoin));
        axioms.push(Box::new(ModeChangeIsConstitutedAmendment));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ConstitutiveProtocolCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ConstitutiveProtocolOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ipv6_only_holds() {
        assert!(Ipv6Only.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn post_quantum_only_holds() {
        assert!(PostQuantumOnly.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_event_authored_holds() {
        assert!(EveryEventAuthored.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn constitution_founds_all_authority_holds() {
        assert!(ConstitutionFoundsAllAuthority.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn slashing_excludes_from_praxis_holds() {
        assert!(SlashingExcludesFromPraxis.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn illocutionary_force_total_holds() {
        assert!(IllocutionaryForceTotal.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn role_ladder_total_order_holds() {
        assert!(RoleLadderTotalOrder.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn moderation_requires_grant_authority_holds() {
        assert!(ModerationRequiresGrantAuthority.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ban_excludes_rejoin_holds() {
        assert!(BanExcludesRejoin.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mode_change_is_constituted_amendment_holds() {
        assert!(ModeChangeIsConstitutedAmendment.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = ConstitutiveProtocolConcept> {
        proptest::sample::select(ConstitutiveProtocolConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_illocutionary_force_total_over_events(c in arb_concept()) {
            // Some exactly when the concept subsumes into PraxisEvent.
            let is_event = kinded_edge_exists(
                c,
                ConstitutiveProtocolConcept::PraxisEvent,
                ConstitutiveProtocolRelationKind::Subsumption,
            );
            prop_assert_eq!(IllocutionaryForce.get(&c).is_some(), is_event);
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ConstitutiveProtocolCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ConstitutiveProtocolOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_illocutionary_force_total_over_events, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
