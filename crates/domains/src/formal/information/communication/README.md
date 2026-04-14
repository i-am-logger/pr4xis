# Communication -- Shannon + Jakobson communication ontology

Models the universal structure of information transfer, unifying Shannon's (1948) information-theoretic chain — source, encoder, channel, decoder, destination, noise — with Jakobson's (1960) six language functions and Wiener's (1948) cybernetic feedback loop. The category's objects are the eight communication components; morphisms are the kinded relations (produces, transmitted-through, interprets, encodes/decodes, corrupts, flows-back, grounds, shares) that connect them.

Key references:
- Shannon 1948: *A Mathematical Theory of Communication* (source-channel-destination + noise)
- Jakobson 1960: *Linguistics and Poetics* (six language functions, addresser/addressee/context/message/contact/code)
- Lasswell 1948: *The Structure and Function of Communication in Society*
- Wiener 1948: *Cybernetics* (feedback as communication)

## Entities (8)

| Category | Entities |
|---|---|
| Sender side (2) | Sender, Code |
| Signal (3) | Message, Channel, Noise |
| Receiver side (1) | Receiver |
| Context and feedback (2) | Context, Feedback |

## Category

The eight `CommunicationConcept` objects form the Shannon chain (Sender → Message → Channel → Receiver) extended with Jakobson's Context/Code and Wiener's Feedback loop. Composed edges close the category under composition, covering Sender → Channel, Sender → Receiver, Noise → Message and the Receiver → Sender feedback round-trip.

## Qualities

| Quality | Type | Description |
|---|---|---|
| CommunicationFunctionQuality | JakobsonFunction | Maps each component to its Jakobson function: Context=Referential, Sender=Emotive, Receiver=Conative, Channel=Phatic, Code=Metalingual, Message=Poetic |

## Axioms

The ontology relies on the auto-generated structural axioms from `define_ontology!` (category laws on the Shannon+Jakobson kinded relation graph). No domain axioms are declared beyond those.

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the kinded relation graph | auto-generated |

## Functors

**Outgoing (1):**

| Functor | Target | File |
|---|---|---|
| CommunicationToControl | control theory | `control_functor.rs` |

**Incoming (2):**

| Functor | Source | File |
|---|---|---|
| DialogueToCommunication | dialogue | `../dialogue/communication_functor.rs` |
| OrthographyChannelFunctor | linguistics/orthography | `../../../cognitive/linguistics/orthography/channel_communication_functor.rs` |

## Files

- `ontology.rs` -- `CommunicationConcept`, Shannon+Jakobson category, Jakobson function quality, tests
- `control_functor.rs` -- Functor from communication into control theory
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
