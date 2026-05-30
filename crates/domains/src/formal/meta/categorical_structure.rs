//! Praxis as composable lenses + composable adjunctions.
//!
//! Praxis is a category where loaded sources flow through TWO kinds
//! of morphisms:
//!
//! - **Lenses** — structure-preserving, lossless. Compose
//!   horizontally. Round-trip recovers the source byte-for-byte
//!   under canonical form.
//!
//! - **Adjunctions** — projections to interpretation views. May be
//!   lossy. Compose with complements (Bancilhon & Spyratos 1981);
//!   together with the complement, recover full structure.
//!
//! # The architectural diagram
//!
//! ```text
//!                                              ┌──> English ontology
//!                                              │     (adjunction; lossy
//!                                              │     projection)
//!                                              │
//!                                              ├──> Statute domain
//!                                              │     ontology
//!                                              │     (adjunction; lossy)
//!                                              │
//! Source bytes ←lens→ XML AST ←lens→ XSD ontology ─────┤
//!     (quick-xml)         (xsd-parser)         │
//!                                              ├──> Authority ontology
//!                                              │     (adjunction; lossy)
//!                                              │
//!                                              └──> OWL / RDF graph
//!                                                    (adjunction; lossy)
//! ```
//!
//! # Horizontal axis: lenses
//!
//! Each step is a well-behaved lens (Foster, Greenwald, Moore,
//! Pierce & Schmitt 2007 — "Combinators for Bidirectional Tree
//! Transformations", *ACM TOPLAS* 29(3)).
//!
//! Composition is lossless: `Source bytes ↔ XSD ontology` is a
//! single lens formed by composing the per-step lenses. Round-trip
//! preserved (Foster et al. §3 — lens composition combinators).
//!
//! The PutGet law (Foster §2.2) is the empirical signature:
//! canonical(put(get(bytes))) == canonical(bytes).
//!
//! # Vertical axis: adjunctions
//!
//! Each fan-out is an adjunction (Mac Lane 1971, *Categories for
//! the Working Mathematician*, §IV.1). The right adjoint projects
//! information (lossy); the left adjoint lifts queries from the
//! projection back to the structural ontology.
//!
//! The "complement" (Bancilhon & Spyratos 1981, *ACM TODS* 6(4))
//! is the structural data the projection drops. For praxis:
//! - English projection ⊥ structural complement = XSD ontology
//! - Statute projection ⊥ structural complement = XSD ontology
//! - etc.
//!
//! Chat traverses lens + adjunction together (`&English` +
//! `&UsCode`) — adjoint plus complement.
//!
//! # Categorical foundation
//!
//! Praxis ontologies form a **fibered category** (Grothendieck 1971
//! — *FGA / SGA*; Jacobs 1999 — *Categorical Logic and Type
//! Theory*, Ch. 1). The base category is "source kinds"
//! (XSD-document, JSON-document, RDF-graph). The fiber over each
//! source kind is the stack of lenses-and-adjunctions above it.
//!
//! # Practical implications
//!
//! - Every loaded source has a chain of lenses giving full
//!   reconstruction.
//! - The chat / query layer takes the COMPLEMENT (structural
//!   ontology) PLUS the adjoint (English projection); together
//!   they answer queries faithfully.
//! - Adding a new ontology projection (e.g. statute → authority
//!   strength) is adding a new adjunction in the fan; doesn't
//!   touch the existing lens chain.
//! - Adding a new source kind (e.g. JSON, RDF) is adding a new
//!   base-category object with its own lens chain.
//!
//! # Literature
//!
//! - **Bancilhon, F. & Spyratos, N. (1981).** "Update Semantics of
//!   Relational Views". *ACM Transactions on Database Systems*
//!   6(4):557-575. Constant-complement formalization.
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007).** "Combinators for Bidirectional Tree
//!   Transformations". *ACM TOPLAS* 29(3) Article 17. Lens definition
//!   + composition.
//! - **Hofmann, M.; Pierce, B. C.; Wagner, D. (2011).** "Symmetric
//!   Lenses". *POPL '11*. Explicit-complement symmetric lenses.
//! - **Mac Lane, S. (1971).** *Categories for the Working
//!   Mathematician*. Springer. §IV.1 adjunctions, §IV.4 equivalence.
//! - **Grothendieck, A. (1971).** "Catégories fibrées et descente".
//!   *SGA 1, Exposé VI*. Fibered categories.
//! - **Jacobs, B. (1999).** *Categorical Logic and Type Theory*.
//!   Elsevier. Ch. 1 fibred category theory; Ch. 9 advanced fibred categories.
