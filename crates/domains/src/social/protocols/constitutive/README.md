# ConstitutiveProtocol — the prx p2p trust protocol's conceptual layer

Re-manifests the prose ontology of the prx protocol (its `docs/ontology.md`) as a pr4xis ontology: the six top-level categories a peer-to-peer trust protocol admits — Identity, Authority, Constitution, Praxis, Membership, Slashing — plus the transport-address and signature-scheme concepts its two foundational axioms quantify over. The prose document is the lineage; this module is the machine-checkable form.

## Verification

```
cargo test -p pr4xis-domains constitutive
```

Runs category laws, ontology validation, the ten domain axioms (single-point + proptest sweeps), and the engine admittance mechanics.

## Concepts (43)

| Cluster | Concepts |
|---|---|
| Identity | `Identity`, `Seed`, `AuthoringAgency`, `ConstitutiveAgency`, `DeviceId` |
| Authority | `Authority`, `Role`, `FounderRole`, `OperatorRole`, `VoiceRole`, `ChannelModes`, `RoleGrant`, `RoleRevoke`, `Kick`, `Ban`, `Unban`, `Unkick`, `ChannelModeChange` |
| Constitution | `Constitution`, `ChannelManifest`, `ChannelId` |
| Praxis | `PraxisEvent`, `Chain`, `Seq`, and the fourteen event types (`ChannelJoin`, `Leave`, `ProfileUpdate`, `StreamOpen`, `StreamChunk`, `StreamClose`, `ClientPolicy`, `RoleGrant`, `RoleRevoke`, `Kick`, `Ban`, `Unban`, `Unkick`, `ChannelModeChange`) |
| Membership | `Membership` |
| Slashing | `Slashing`, `Equivocation`, `ForkProof`, `SlashRegistry` |
| Axiom support | `TransportAddress`, `Ipv6Address`, `Ipv4Address`, `SignatureScheme`, `MlDsa`, `SlhDsa`, `Ed25519` |

## Qualities (typed)

| Quality | Value type | Defined over |
|---|---|---|
| `IllocutionaryForce` | `SearleCategory` (reused from `cognitive::linguistics::pragmatics::speech_act`) | exactly the fourteen event types, per the prx §5.3 table (the five moderation/mode events are all Declarations) |
| `IsPostQuantum` | `bool` | the three signature schemes |
| `SignsDurably` | `bool` | the three signature schemes |
| `DurablyAddressable` | `bool` | the two address families |
| `RoleRank` | `RankOrdinal` (RFC 1459/2811 ladder position) | the three role tiers |

## Domain axioms

| Axiom | Source | Claim |
|---|---|---|
| `Ipv6Only` | RFC 8981 (2021); prx Axiom A1 | Every durably-addressable transport address is `Ipv6Address` |
| `PostQuantumOnly` | NIST FIPS 204/205 (2024); prx Axiom A2 | Every durable-signing scheme is post-quantum; ed25519 is transient only |
| `EveryEventAuthored` | Lamport (1979); Austin (1962) | The `Authors` edge exists and all fourteen event types subsume into `PraxisEvent` |
| `ConstitutionFoundsAllAuthority` | Hart (1961) | `Constitutes` and `Founds` edges present: all authority descends from the founding act |
| `SlashingExcludesFromPraxis` | Li et al. (2004) SUNDR §3; Buterin & Griffith (2017) | The `ProvenBy → Triggers → ExcludesFrom` chain holds, and engine gate 0 drops every event authored by a slashed device |
| `IllocutionaryForceTotal` | Searle (1969); Austin (1962) | `IllocutionaryForce` is total over exactly the fourteen event types and matches the prx §5.3 table |
| `RoleLadderTotalOrder` | RFC 1459/2811; Saltzer & Schroeder (1975) | `RoleRank` is defined for exactly the three tiers, strictly `Founder > Operator > Voice` |
| `ModerationRequiresGrantAuthority` | Saltzer & Schroeder (1975); RFC 1459 §4.2.8; RFC 2811 §4.1 | A Kick/Ban/RoleRevoke is admitted iff the actor `can_grant` over the target's tier and the target is not the Founder (strict rank when `op_can_grant_op` is clear) |
| `BanExcludesRejoin` | RFC 2811 §4.3.1 (ban masks); RFC 1459 §4.2.8 | A banned device's fresh join is refused until `Unban`; a kick alone does not block rejoin (a fresh join clears the kick) |
| `ModeChangeIsConstitutedAmendment` | Hart (1961); Lessig (1999) | A `ChannelModeChange` overrides the effective modes while leaving the channel identity and the constituted structure untouched — amendment, not re-founding |

## Engine

`engine.rs` — the receiver-side admittance state machine. `ChannelSituation` holds the channel identity, the materialised Membership relation, the channel's access-matrix rows (Lampson), the kicked / banned moderation registries, the currently-effective `ChannelModeSet`, and the slash registry mirror. `apply_channel` admits or drops `AdmitJoin` / `AdmitLeave` / `AdmitRoleGrant` / `AdmitRoleRevoke` / `AdmitKick` / `AdmitBan` / `AdmitUnban` / `AdmitUnkick` / `AdmitModeChange` / `ObserveForkProof` under three rules, in order: gate 0 (events from slashed devices are dropped), the ban gate (a banned device's join is refused; a fresh join clears a prior kick), and grant authority (grants, revokes, kicks, and bans gate on prx `ChannelRole::can_grant` — the Founder acts on anything below, an Operator on Voice always and on a fellow Operator only when `op_can_grant_op` is set, Voice on nobody, nobody on the Founder). Pure `no_std` + `alloc`.

## Files

- `ontology.rs` — `ConstitutiveProtocolOntology`, five Qualities, ten domain axioms, proptests
- `engine.rs` — `ChannelSituation` / `ChannelAction` / `apply_channel`
- `tests.rs` — engine mechanics tests + proptests
- `README.md`, `citings.md` — this file + bibliography
