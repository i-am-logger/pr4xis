//! Statute-structure parser — produces a typed `ClauseTree` from
//! plain statutory text.
//!
//! The first source-understanding deliverable. Takes verbatim statute
//! text (the same kind in `crates/domains/data/canonical_text/`) and
//! emits a recursive tree of [`ClauseNode`]s, each carrying its
//! `PinpointCite` subdivision path plus the body text that lives
//! at that subdivision. Downstream extractors (term-extractor,
//! relation-extractor — future commits) consume the tree.
//!
//! # Pipeline position
//!
//! ```text
//!   PDF/HTML bytes  →  text   →  ClauseTree  →  Vec<(Term, Relation)>  →  praxis.lock structural
//!  ─────────────  ───  ──────  ───────────────  ─────────────────────  ─────────────────────────
//!  wb-format-pdf   ↑    ↑      THIS MODULE     term/relation extractor  Static-store codegen
//!                  │    │                       (later)
//!                  │    └── format-specific text extraction
//!                  └── PDF parsing
//! ```
//!
//! Today the input side (extracted text) is supplied by hand-
//! transcribed canonical-text fixtures in
//! `crates/domains/data/canonical_text/`; once the PDF/HTML loader
//! lands, the same module consumes loader-extracted text without
//! any changes.
//!
//! # What's parsed
//!
//! The parser recognises **Bluebook §3.3 subdivision markers**:
//! `(a)`, `(1)`, `(A)`, `(i)`, `(I)`. Depths follow the canonical
//! convention — lowercase letter → arabic numeral → uppercase letter
//! → lowercase roman → uppercase roman. Disambiguation between
//! letter "i" and roman numeral "i" is **context-based**: a single
//! "i" parses as a roman clause when the current stack has an open
//! `Subparagraph` (depth 3) parent, otherwise as a top-level
//! lowercase letter `Subsection`.
//!
//! Text between markers is attached to the most recent open clause.
//! Free-form leading text (before any marker) is attached to the
//! root.
//!
//! # Literature
//!
//! - **USLM schema (Office of the Law Revision Counsel)** — the
//!   subdivision `<level>` element hierarchy; the machine-loaded praxis
//!   source for the structure (machine-verified).
//! - **The Bluebook: A Uniform System of Citation, 21st ed. (2020)**
//!   §3.2 (pinpoint citations) and §3.3 (multiple subdivisions) —
//!   the parenthesized citation-rendering convention. LLM-checked (web).
//! - **Wyner, Adam & Bench-Capon, Trevor (2007)** "Argument schemes
//!   for legal case-based reasoning" *Proc. JURIX 2007*; and
//!   **Wyner, Adam (2008)** "Towards Annotating and Extracting
//!   Textual Legal Case Elements", *Informatica e Diritto* XVII —
//!   structural extraction from statutory and case-law text.
//! - **Sartor, Giovanni (2005)** *Legal Reasoning: A Cognitive
//!   Approach to the Law*, Springer (Treatise vol. 5) — Ch. 12
//!   formal treatment of normative texts.
//!
//! # Praxis-way compliance
//!
//! No new ontology concepts. The parser produces values of existing
//! types (`PinpointCite` from `judicial::citation` and `SourceTextRef`
//! from `judicial::source_text`). The clause-tree shape itself is
//! data, not a new ontology. The seven structural invariants checked
//! in [`invariants`] are property-style checks over instances — they
//! verify the parser's output conforms to Bluebook §3.3 + Wyner's
//! clause-structure rules, but don't introduce new conceptual layers.

pub mod bridge;
pub mod definition_scope;
pub mod english_adjunction;
/// The written-form `denotes` floor producer — statute prose → typed pointers
/// into the English `ontolex:Form` atoms. Needs `std` (it reaches the
/// `english::bridge` runtime projection).
#[cfg(feature = "std")]
pub mod grounding;
pub mod invariants;
pub mod parser;
pub mod relation_extractor;
pub mod statute_report;
pub mod statute_understanding;
pub mod term_extractor;
pub mod us_legal_lexicon;

pub use english_adjunction::{
    LemmaSenseMapping, resolve_form_to_senses, resolve_lemmas_to_senses,
    resolve_term_name_to_senses,
};
pub use parser::{ClauseNode, ClauseTree, LabelKind, ParseError, parse_statute_text};
pub use relation_extractor::{RelationCandidate, RelationKind, extract_relations};
pub use statute_report::{ReportContext, ReportGap, ReportParaphrase, generate_statute_report};
pub use term_extractor::{ExtractedTerm, extract_terms};

#[cfg(test)]
mod tests;
