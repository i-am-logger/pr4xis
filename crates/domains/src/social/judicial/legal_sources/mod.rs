//! Legal sources — the formal sources of law and their subsumption
//! hierarchy.
//!
//! ```text
//! LegalSource  (genus — "law")
//!   ├── LegalDocument                     (lkif:Legal_Document)
//!   │     ├── Statute                     (lkif:Statute)
//!   │     ├── Regulation                  (lkif:Regulation)
//!   │     ├── Constitution                (DOCTRINAL addition — Hart 1961)
//!   │     ├── Treaty                       (lkif:Treaty)
//!   │     └── Code                         (lkif:Code)
//!   ├── Precedent                          (lkif:Precedent — DIRECT under Legal_Source)
//!   └── CustomaryLaw                       (lkif:Customary_Law — DIRECT under Legal_Source)
//! ```
//!
//! `Precedent` and `CustomaryLaw` sit *directly* under `LegalSource`,
//! **not** under `LegalDocument` — faithful to LKIF-Core, where a
//! precedent is a source of law but not a written document instrument.
//!
//! # Relationship to `authority_strength`
//!
//! This ontology captures *what kind of source* a rule comes from
//! (statute vs. regulation vs. case law). The sibling
//! `authority_strength` ontology captures *how much binding force* a
//! source carries. The two are orthogonal dimensions related by the
//! [`functor_from_authority_strength`] cross-functor — a binding-force
//! leaf maps to its source-type — not by merging the taxonomies.
//!
//! # Citation-tier table (provenance discipline)
//!
//! Each edge and concept is tiered honestly. LKIF-Core edges are
//! **machine-verifiable** against the published `norm.owl`; the
//! genus/species doctrine is **doctrinal** (Salmond/Hart/Garner);
//! `Constitution` is a **doctrinal-only** honest addition, flagged.
//!
//! | Concept / edge | Tier | Source |
//! |---|---|---|
//! | `LegalSource` | doctrinal + LKIF | Salmond; lkif:Legal_Source |
//! | `LegalDocument ⊑ LegalSource` | LKIF machine-verifiable | norm.owl |
//! | `Statute ⊑ LegalDocument` | LKIF machine-verifiable + doctrinal | norm.owl; Salmond |
//! | `Regulation ⊑ LegalDocument` | LKIF machine-verifiable + doctrinal | norm.owl; Chevron (1984) |
//! | `Constitution ⊑ LegalDocument` | **doctrinal-only (FLAGGED)** | Hart (1961) — NOT LKIF |
//! | `Treaty ⊑ LegalDocument` | LKIF machine-verifiable + doctrinal | norm.owl; U.S. Const. Art. II §2 |
//! | `Code ⊑ LegalDocument` | LKIF machine-verifiable | norm.owl |
//! | `Precedent ⊑ LegalSource` (direct) | LKIF machine-verifiable + doctrinal | norm.owl; Garner (2016) |
//! | `CustomaryLaw ⊑ LegalSource` (direct) | LKIF machine-verifiable + doctrinal | norm.owl; Salmond |
//!
//! # LKIF-Core machine-verification record
//!
//! The `is_a` edges were verified by fetching LKIF-Core `norm.owl`
//! (Hoekstra/lkif-core, master) and reading each class's
//! `rdfs:subClassOf`. Confirmed against `norm.owl`:
//!
//! - `Legal_Source` ⊑ `expression:Medium` — the genus.
//! - `Legal_Document` ⊑ `Legal_Source` **and** `expression:Document`.
//! - `Statute`, `Regulation`, `Code` ⊑ `Legal_Document`.
//! - `Treaty` ⊑ `International_Agreement` **and** `Legal_Document`.
//! - `Precedent` ⊑ `Legal_Source` (direct — **not** `Legal_Document`).
//! - `Customary_Law` ⊑ `Legal_Source` **and** `Custom` (direct).
//!
//! **Could NOT verify** (absent from `norm.owl`): `Constitution`. LKIF-Core
//! has no `Constitution` class. It is retained here as an honest doctrinal
//! addition grounded in Hart's rule of recognition, placed under
//! `LegalDocument` by analogy to the other enacted written instruments —
//! flagged in both the concept label and the `is_a` comment so no reader
//! mistakes it for an LKIF edge.
//!
//! # Literature
//!
//! - **Hoekstra, R., Breuker, J., Di Bello, M. & Boer, A. (2007)** "The
//!   LKIF Core Ontology of Basic Legal Concepts", *Proc. LOAIT 2007*
//!   (CEUR-WS Vol. 321) — the source ontology; `norm.owl` defines
//!   Legal_Source, Legal_Document, Statute, Regulation, Code, Treaty,
//!   Precedent, and Customary_Law with the subclass edges above.
//! - **Salmond, J.** *Salmond on Jurisprudence* — the *formal sources*
//!   of law (that which gives a rule its legal force) and the enacted /
//!   unenacted (case law + customary) division grounding `IsEnactedOf`.
//! - **Hart, H.L.A. (1961)** *The Concept of Law*, Oxford — Ch. VI, the
//!   *rule of recognition* supplies the ultimate criteria of legal
//!   validity; grounds `Constitution` as an honest addition.
//! - **Garner, Bryan A. (2016)** *Black's Law Dictionary* (10th/11th
//!   ed.), Thomson Reuters — the "precedent" / "case law" entries.
//! - **Chevron U.S.A. v. NRDC**, 467 U.S. 837 (1984) — administrative
//!   rulemaking (regulation) as a source of binding norms.
//! - **U.S. Const. Art. II, §2** — the treaty power.

pub mod functor_from_authority_strength;
pub mod ontology;

#[cfg(test)]
mod tests;
