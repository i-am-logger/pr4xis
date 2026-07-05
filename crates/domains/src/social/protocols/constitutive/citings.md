# ConstitutiveProtocol ontology — bibliography

Reproduces the master reference list of the prx protocol's prose ontology
(`docs/ontology.md` §10), which this module re-manifests in `ontology!` form.

## Master reference list (prx ontology §10)

- **Aristotle**, *Nicomachean Ethics*, Book VI (~340 BCE; trans. Ross 1925) — *praxis* as purposeful activity.
- **Austin, J. L.** (1962). *How to Do Things With Words*. Harvard — performative utterances; the speech-act foundation. Grounds `RoleGrant` / `RoleRevoke` as performatives, the Declaration rows of `IllocutionaryForce` (including the five moderation / mode events), and `Unkick` as the performative-symmetry counterpart of `Kick`.
- **Buterin, V. & Griffith, V.** (2017). *"Casper the Friendly Finality Gadget"*. arXiv:1710.09437 — slashing conditions as cryptographic penalties. Grounds `Slashing` and `SlashingExcludesFromPraxis`.
- **Diffie, W. & Hellman, M.** (1976). *"New Directions in Cryptography"*. IEEE Transactions on Information Theory 22(6) — public-key as constitutive of identity.
- **Elster, J.** (1979). *Ulysses and the Sirens*. Cambridge — pre-commitment / self-binding rationality. Grounds `ClientPolicy` as a self-directed directive.
- **Habermas, J.** (1981). *Theorie des kommunikativen Handelns*. Suhrkamp — communicative action as praxis. Grounds `PraxisEvent`.
- **Hart, H. L. A.** (1961). *The Concept of Law*. OUP — primary vs. secondary rules. Grounds `Constitution`, `ConstitutionFoundsAllAuthority`, and `ModeChangeIsConstitutedAmendment` (a mode change amends under the constitution rather than re-founding).
- **Kalt, C.** (2000). *"Internet Relay Chat: Channel Management"*. RFC 2811 — channel modes and the authority ladder. §4.1 member status (channel creator vs. operator), §4.2 channel flags / MODE, §4.3.1 Channel Ban and Exception (+b ban masks). Grounds `Role` / `FounderRole` / `ChannelModes` and `RoleLadderTotalOrder`, the `Ban` / `Unban` / `ChannelModeChange` events, and `BanExcludesRejoin` / `ModerationRequiresGrantAuthority`.
- **Lamport, L.** (1979). *"Constructing Digital Signatures from a One-Way Function"*. SRI tech report CSL-98 — to be able to sign is what an identity is. Grounds `Identity` / `SignatureScheme` and `EveryEventAuthored`.
- **Lamport, L., Shostak, R. & Pease, M.** (1982). *"The Byzantine Generals Problem"*. ACM TOPLAS 4(3) — misbehaviour detectable from message inconsistency alone. Grounds `Equivocation`.
- **Lampson, B.** (1971/1974). *"Protection"*. ACM Operating Systems Review 8(1) — the access matrix (subject × object → right). Grounds `Authority`.
- **Lessig, L.** (1999). *Code and Other Laws of Cyberspace*. Basic Books — code-as-law. Grounds `ChannelModes` (`Modulates` edge), `ChannelModeChange` (`Amends` edge), and `ModeChangeIsConstitutedAmendment`.
- **Li, J., Krohn, M., Mazieres, D. & Shasha, D.** (2004). *"Secure Untrusted Data Repository (SUNDR)"*. OSDI 2004 — fork-consistency as the operational definition of equivocation (§3). Grounds `Chain` / `ForkProof` and `SlashingExcludesFromPraxis`.
- **Locke, J.** (1689). *Second Treatise of Government* — consent as the ground of membership. Grounds `Membership`.
- **NIST FIPS 204** (August 2024). *Module-Lattice-Based Digital Signature Standard (ML-DSA)* — grounds `MlDsa` / `AuthoringAgency` and `PostQuantumOnly`.
- **NIST FIPS 205** (August 2024). *Stateless Hash-Based Digital Signature Standard (SLH-DSA)* — grounds `SlhDsa` / `ConstitutiveAgency` and `PostQuantumOnly`.
- **Oikarinen, J. & Reed, D.** (1993). *"Internet Relay Chat Protocol"*. RFC 1459 — IRC's authority ladder (modes +o / +v), the KICK command (§4.2.8), and the MODE command with the +b ban mask (§4.2.3 / §4.2.3.1). Grounds `OperatorRole` / `VoiceRole` and `RoleLadderTotalOrder`, the `Kick` / `Unkick` events, and `ModerationRequiresGrantAuthority` / `BanExcludesRejoin`.
- **Saltzer, J. & Schroeder, M.** (1975). *"The Protection of Information in Computer Systems"*. Proceedings of the IEEE 63(9) — least privilege; why the role ladder is closed-set. Grounds the engine's role-grant and moderation gates, `RoleLadderTotalOrder`, and `ModerationRequiresGrantAuthority`.
- **Schmitt, C.** (1928). *Verfassungslehre*. Duncker & Humblot — constitutive vs. constituted power. Citing Schmitt is *uncomfortable* — his later politics are abhorrent — but the *Verfassungslehre* analysis of constitutive vs. constituted authority is load-bearing in constitutional theory and worth naming honestly. Grounds the `Constitution` / `ConstitutiveAgency` distinction.
- **Searle, J.** (1969). *Speech Acts: An Essay in the Philosophy of Language*. Cambridge — the illocutionary taxonomy. Grounds `IllocutionaryForce` (valued in the existing `SearleCategory`) and `IllocutionaryForceTotal`.
- **Wittgenstein, L.** (1953). *Philosophische Untersuchungen*. Blackwell — meaning as use (§43). Grounds `DeviceId` / `ChannelId` semantics (`NamedBy` edges).

## Cited in the prx ontology outside its §10 list

- **RFC 8981** (2021). *Temporary Address Extensions for Stateless Address Autoconfiguration in IPv6* (Gont, F., Krishnan, S., Narten, T. & Draves, R.) — per-session source-address rotation removes the stable-IP fingerprint. Grounds `Ipv6Address` / `Ipv4Address` and `Ipv6Only` (prx Axiom A1, ontology §0).
- **Weyl, E. G., Ohlhaver, P. & Buterin, V.** (2022). *"Decentralized Society: Finding Web3's Soul"*. SSRN 4105763 — soulbound tokens as the cryptographic analogue of membership (prx ontology §6.2). Grounds `Membership`.

## Cross-references

- Source document: the prx protocol's `docs/ontology.md` — the prose ontology this module re-manifests.
- Related workspace ontologies:
  - `cognitive::linguistics::pragmatics::speech_act` — the `SearleCategory` taxonomy reused as `IllocutionaryForce`'s value type.
  - `social::compliance`, `social::judicial` — neighbouring rule-system ontologies under `social`.
