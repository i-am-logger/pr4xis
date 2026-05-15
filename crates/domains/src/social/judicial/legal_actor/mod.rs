//! Legal actor — the typed kind of participant in a legal proceeding.
//!
//! Replaces bare-`String` "actor" / "borne_by" fields in `Obligation`
//! and `BurdenOfProof` with a typed concept from a finite, literature-
//! grounded taxonomy. Every participant in U.S. civil + administrative
//! litigation is one of these kinds.
//!
//! # Concept hierarchy
//!
//! ```text
//! LegalActor
//!   ├── Party                 — has a stake in the outcome
//!   │     ├── Plaintiff       — initiating party (civil)
//!   │     ├── Defendant       — responding party (civil)
//!   │     ├── Petitioner      — initiating party (equitable / appellate)
//!   │     ├── Respondent      — responding party (equitable / appellate)
//!   │     ├── Movant          — party making a motion
//!   │     ├── Appellant       — party seeking appellate review
//!   │     └── Appellee        — party defending below decision
//!   ├── Adjudicator           — decides the matter
//!   │     ├── Court           — Article III court
//!   │     ├── Judge           — individual jurist
//!   │     ├── Magistrate      — magistrate judge (FRCP 72)
//!   │     ├── Jury            — finder of fact (FRCP 38)
//!   │     └── Agency          — federal agency (APA 5 U.S.C. § 551)
//!   ├── Witness               — provides testimony
//!   │     ├── FactWitness     — perceives facts
//!   │     └── ExpertWitness   — opinion under FRE 702
//!   └── Counsel               — represents a party (Model Rules)
//! ```
//!
//! `Party`, `Adjudicator`, `Witness`, `Counsel` form the four mutually-
//! exclusive role-families: a person playing one role cannot
//! simultaneously play another in the same proceeding (e.g. *Tumey v.
//! Ohio*, 273 U.S. 510 (1927) — judge cannot have a financial stake;
//! ABA Model Rule 3.7 — counsel cannot also testify).
//!
//! # Literature
//!
//! - **Federal Rules of Civil Procedure, Rule 17** *Plaintiff and
//!   Defendant; Capacity; Public Officers* — establishes the
//!   "real party in interest" doctrine for the Plaintiff/Defendant
//!   pair.
//! - **Federal Rules of Civil Procedure, Rule 38** *Right to a Jury
//!   Trial; Demand* — Jury as adjudicator.
//! - **Federal Rules of Civil Procedure, Rule 72** *Magistrate Judges:
//!   Pretrial Order* — Magistrate as adjudicator.
//! - **Federal Rules of Evidence, Rule 702** *Testimony by Expert
//!   Witnesses* — ExpertWitness vs FactWitness distinction.
//! - **Restatement (Second) of Judgments §27 (1982)** *Issue
//!   Preclusion: General Rule* — party identity / privity for
//!   issue-preclusion analysis.
//! - **5 U.S.C. § 551** *Administrative Procedure Act — Definitions*
//!   — federal Agency as adjudicator.
//! - **Tumey v. Ohio**, 273 U.S. 510 (1927) — judge-impartiality
//!   doctrine prohibiting overlapping Party/Adjudicator roles.
//! - **ABA Model Rule of Professional Conduct 3.7** *Lawyer as Witness*
//!   — prohibition on Counsel-Witness role overlap.
//! - **Garner, *Black's Law Dictionary*, 11th ed. (2019)** — canonical
//!   role names and definitions.

pub mod ontology;

#[cfg(test)]
mod tests;
