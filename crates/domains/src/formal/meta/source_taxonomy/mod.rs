//! Source taxonomy — the kinds of external corpora praxis can ingest.
//!
//! A meta ontology declaring **what kinds of things** external data can be,
//! independent of **how** they are encoded (`artifact_identity`) or **where**
//! they live (`applied/data_provisioning`). The registry declares instances
//! (specific corpora — "english_wordnet 2025", "sox_1514a 2002"); this
//! ontology declares the abstract kinds (`Language`, `Statute`, `Regulation`)
//! those instances inhabit, along with the adjunction graph that connects
//! them.
//!
//! # Why this exists
//!
//! Without a typed taxonomy, the registry has to encode three things implicitly
//! in `content_type` strings:
//! 1. The decoder family (XML, plaintext, JSON, PDF) — a *format* property
//! 2. The destination ontology — a *semantic* property
//! 3. The adjunction wiring — a *relational* property
//!
//! The taxonomy makes those three explicit and ties them to the kind
//! hierarchy. A new statute becomes a one-line `praxis.toml` addition: the
//! decoder is implied by `Statute → LegalCorpus → text parser`; the code-path
//! convention is implied by `Statute → social::compliance::law::<name>`; the
//! adjunctions are implied by the `Statute ⊣ LegalLexicon` edge in this
//! ontology — applied automatically to every Statute instance.
//!
//! # Concept families
//!
//! - **Lexicon family** — lexical resources. `Language` (a Lexicon for a
//!   natural language, e.g. English WordNet), `DomainLexicon` (a Lexicon
//!   restricted to a specialty, e.g. Black's Law for legal terms).
//!   `LegalLexicon` is a `DomainLexicon` specialization.
//! - **LegalCorpus family** — Hart's primary + secondary rules realized as
//!   distinct concept leaves: `Statute`, `Regulation`,
//!   `ConstitutionalArticle`, `ProceduralRule`, `CaseLaw`.
//!
//! # Adjunctions
//!
//! Declared once at the concept level via the `adjoins` ontology field.
//! Every pair of *loaded instances* whose concepts are adjacent in this graph
//! gets a generated adjunction functor at build time — no per-source
//! declaration. MacLane (1971) §IV.1 grounds the adjoint-pair semantics.
//!
//! # Literature
//!
//! - **Hart (1961)** *The Concept of Law*, Oxford — primary vs secondary
//!   rules (the LegalCorpus subtree's taxonomic principle).
//! - **MacLane (1971)** *Categories for the Working Mathematician*,
//!   Springer-Verlag, §IV.1 — adjoint functors.
//! - **Pustejovsky (1995)** *The Generative Lexicon*, MIT Press — the
//!   Lexicon family's taxonomic principle (qualia structure separates
//!   general from domain lexicons).
//! - **Vossen (1998)** *EuroWordNet: A Multilingual Database with Lexical
//!   Semantic Networks*, Springer — `Language` as a Lexicon for a natural
//!   language.
//! - **Sartor (2005)** *Legal Reasoning: A Cognitive Approach to the Law*,
//!   Springer — case law and procedural rules as secondary instruments
//!   interpreting primary rules.
//! - **Solan (1993)** *The Language of Judges*, University of Chicago Press —
//!   legal English as a distinct domain lexicon adjacent to common English
//!   (grounds the `LegalLexicon ⊣ Language` bridge).
//! - **Marbury v. Madison**, 5 U.S. (1 Cranch) 137 (1803) —
//!   `ConstitutionalArticle` as the supreme primary rule, authorizing
//!   judicial review of statutes.
//! - **Wilkinson et al. (2016)** "The FAIR Guiding Principles for scientific
//!   data management and stewardship", *Scientific Data* 3:160018 — `Source`
//!   as the unifying root for findable, accessible, interoperable, reusable
//!   data.
//! - **Dolstra (2006)** *The Purely Functional Software Deployment Model*,
//!   PhD thesis, Utrecht University — content-addressable artifacts as the
//!   basis for `Source` identity (works in concert with `artifact_identity`).
pub mod ontology;

#[cfg(test)]
mod tests;
