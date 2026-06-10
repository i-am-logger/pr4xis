//! Legal actor ontology — concepts, is_a hierarchy, axioms.
//!
//! See `mod.rs` for the literature inventory.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "LegalActor",
    source: "Federal Rules of Civil Procedure (Rule 17 — Plaintiff/Defendant Capacity; Rule 38 — Jury Trial; Rule 72 — Magistrate Judges); Federal Rules of Evidence Rule 702 (Expert Witness); Restatement (Second) of Judgments §27 (1982); 5 U.S.C. § 551 (Administrative Procedure Act Definitions); Tumey v. Ohio, 273 U.S. 510 (1927); ABA Model Rule of Professional Conduct 3.7 (Lawyer as Witness); Garner, Black's Law Dictionary 11th ed. (2019)",

    concepts: [
        // Root
        LegalActor,

        // Role families (level 1)
        Party,
        Adjudicator,
        Witness,
        Counsel,

        // Party leaves
        Plaintiff,
        Defendant,
        Petitioner,
        Respondent,
        Movant,
        Appellant,
        Appellee,

        // Adjudicator leaves
        Court,
        Judge,
        Magistrate,
        Jury,
        Agency,

        // Witness leaves
        FactWitness,
        ExpertWitness,
    ],

    labels: {
        LegalActor: ("en", "Legal actor",
            "Any participant in a legal proceeding — party, adjudicator, witness, or counsel."),
        Party: ("en", "Party",
            "FRCP Rule 17: a participant with a stake in the outcome; capable of carrying burdens of proof."),
        Adjudicator: ("en", "Adjudicator",
            "The decision-maker — court, judge, magistrate, jury, or administrative agency."),
        Witness: ("en", "Witness",
            "FRE 601 et seq.: a person who provides testimony in the proceeding."),
        Counsel: ("en", "Counsel",
            "ABA Model Rules: an attorney representing a party in the proceeding."),
        Plaintiff: ("en", "Plaintiff",
            "FRCP Rule 17: the initiating party in a civil action."),
        Defendant: ("en", "Defendant",
            "FRCP Rule 17: the party against whom a civil action is brought."),
        Petitioner: ("en", "Petitioner",
            "The initiating party in an equitable / appellate / administrative proceeding."),
        Respondent: ("en", "Respondent",
            "The responding party in an equitable / appellate / administrative proceeding."),
        Movant: ("en", "Movant",
            "Black's Law: any party making a motion before the court."),
        Appellant: ("en", "Appellant",
            "The party seeking appellate review of a lower-court decision."),
        Appellee: ("en", "Appellee",
            "The party defending the lower-court decision on appeal."),
        Court: ("en", "Court",
            "An Article III judicial body."),
        Judge: ("en", "Judge",
            "An individual jurist presiding over a proceeding."),
        Magistrate: ("en", "Magistrate judge",
            "FRCP Rule 72: a magistrate judge handling pretrial matters by referral."),
        Jury: ("en", "Jury",
            "FRCP Rule 38: the finder of fact in a jury-tried civil action."),
        Agency: ("en", "Agency",
            "5 U.S.C. § 551(1): a federal administrative agency exercising adjudicatory authority."),
        FactWitness: ("en", "Fact witness",
            "A witness testifying to perceived facts (FRE 602 personal-knowledge requirement)."),
        ExpertWitness: ("en", "Expert witness",
            "FRE 702: a witness qualified by knowledge, skill, experience, training, or education offering opinion testimony."),
    },

    is_a: [
        // Level 1: role families under root
        (Party, LegalActor),
        (Adjudicator, LegalActor),
        (Witness, LegalActor),
        (Counsel, LegalActor),

        // Level 2: Party leaves
        (Plaintiff, Party),
        (Defendant, Party),
        (Petitioner, Party),
        (Respondent, Party),
        (Movant, Party),
        (Appellant, Party),
        (Appellee, Party),

        // Level 2: Adjudicator leaves
        (Court, Adjudicator),
        (Judge, Adjudicator),
        (Magistrate, Adjudicator),
        (Jury, Adjudicator),
        (Agency, Adjudicator),

        // Level 2: Witness leaves
        (FactWitness, Witness),
        (ExpertWitness, Witness),
    ],

    opposes: [
        // Tumey v. Ohio (1927): Party and Adjudicator roles cannot
        // overlap in the same proceeding — judges with stakes are
        // disqualified.
        (Party, Adjudicator),
        (Adjudicator, Party),

        // ABA Model Rule 3.7: Counsel cannot simultaneously be Witness
        // (with limited exceptions).
        (Counsel, Witness),
        (Witness, Counsel),

        // Plaintiff/Defendant duality at the trial level.
        (Plaintiff, Defendant),
        (Defendant, Plaintiff),

        // Petitioner/Respondent duality at the equity / appellate level.
        (Petitioner, Respondent),
        (Respondent, Petitioner),

        // Appellant/Appellee duality.
        (Appellant, Appellee),
        (Appellee, Appellant),
    ],
}

// ---------------------------------------------------------------------------
// Quality: bears burdens of proof? Parties do; adjudicators / witnesses /
// counsel don't.
// ---------------------------------------------------------------------------

/// Quality: can this kind of actor *carry a burden of proof* in the
/// proceeding? Parties can (FRCP Rule 17 + the burden-shifting case
/// law); adjudicators, witnesses, and counsel cannot.
///
/// Returns `None` for the abstract root and for the four role-family
/// concepts (Party, Adjudicator, Witness, Counsel) — those are abstract
/// kinds, only the leaves are answerable.
#[derive(Debug, Clone)]
pub struct CarriesBurden;

impl Quality for CarriesBurden {
    type Individual = LegalActorConcept;
    type Value = bool;

    fn get(&self, c: &LegalActorConcept) -> Option<bool> {
        use LegalActorConcept as L;
        match c {
            // Party leaves
            L::Plaintiff
            | L::Defendant
            | L::Petitioner
            | L::Respondent
            | L::Movant
            | L::Appellant
            | L::Appellee => Some(true),
            // Adjudicator leaves
            L::Court | L::Judge | L::Magistrate | L::Jury | L::Agency => Some(false),
            // Witness leaves
            L::FactWitness | L::ExpertWitness => Some(false),
            // Family-level concepts and the root are abstract.
            L::LegalActor | L::Party | L::Adjudicator | L::Witness | L::Counsel => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn party_leaves() -> [LegalActorConcept; 7] {
    [
        LegalActorConcept::Plaintiff,
        LegalActorConcept::Defendant,
        LegalActorConcept::Petitioner,
        LegalActorConcept::Respondent,
        LegalActorConcept::Movant,
        LegalActorConcept::Appellant,
        LegalActorConcept::Appellee,
    ]
}

pub fn adjudicator_leaves() -> [LegalActorConcept; 5] {
    [
        LegalActorConcept::Court,
        LegalActorConcept::Judge,
        LegalActorConcept::Magistrate,
        LegalActorConcept::Jury,
        LegalActorConcept::Agency,
    ]
}

pub fn witness_leaves() -> [LegalActorConcept; 2] {
    [
        LegalActorConcept::FactWitness,
        LegalActorConcept::ExpertWitness,
    ]
}

/// True iff `c` is in the Party subtree (or is Party itself).
pub fn is_party(c: LegalActorConcept) -> bool {
    matches!(
        c,
        LegalActorConcept::Party
            | LegalActorConcept::Plaintiff
            | LegalActorConcept::Defendant
            | LegalActorConcept::Petitioner
            | LegalActorConcept::Respondent
            | LegalActorConcept::Movant
            | LegalActorConcept::Appellant
            | LegalActorConcept::Appellee
    )
}

/// True iff `c` is a leaf (not an abstract family or the root).
pub fn is_leaf(c: LegalActorConcept) -> bool {
    !matches!(
        c,
        LegalActorConcept::LegalActor
            | LegalActorConcept::Party
            | LegalActorConcept::Adjudicator
            | LegalActorConcept::Witness
            | LegalActorConcept::Counsel
    )
}

/// Parse a canonical English actor name into a typed concept. Case-
/// insensitive. Covers the 14 leaf concepts plus the family names.
pub fn parse_actor(name: &str) -> Option<LegalActorConcept> {
    use LegalActorConcept as L;
    let lower = name.to_lowercase();
    Some(match lower.as_str() {
        "party" => L::Party,
        "plaintiff" => L::Plaintiff,
        "defendant" => L::Defendant,
        "petitioner" => L::Petitioner,
        "respondent" => L::Respondent,
        "movant" => L::Movant,
        "appellant" => L::Appellant,
        "appellee" => L::Appellee,
        "adjudicator" => L::Adjudicator,
        "court" => L::Court,
        "judge" => L::Judge,
        "magistrate" | "magistrate judge" => L::Magistrate,
        "jury" => L::Jury,
        "agency" => L::Agency,
        "witness" => L::Witness,
        "fact witness" | "lay witness" => L::FactWitness,
        "expert witness" | "expert" => L::ExpertWitness,
        "counsel" | "attorney" | "lawyer" => L::Counsel,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

impl Ontology for LegalActorOntology {
    type Cat = LegalActorCategory;
    type Qual = CarriesBurden;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(PartyAdjudicatorOpposition));
        axioms.push(Box::new(CounselWitnessOpposition));
        axioms.push(Box::new(OnlyPartiesCarryBurden));
        axioms
    }
}

/// Axiom: Party and Adjudicator oppose each other — Tumey v. Ohio
/// (1927) forbids the same person from playing both roles.
pub struct PartyAdjudicatorOpposition;

impl Axiom for PartyAdjudicatorOpposition {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = LegalActorCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == LegalActorRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        let has_pa = opp.contains(&(LegalActorConcept::Party, LegalActorConcept::Adjudicator));
        let has_ap = opp.contains(&(LegalActorConcept::Adjudicator, LegalActorConcept::Party));
        if has_pa && has_ap {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PartyAdjudicatorOpposition",
        "Party and Adjudicator oppose each other (no overlapping roles)",
        "Tumey v. Ohio, 273 U.S. 510 (1927)"
    );
}

pr4xis::register_axiom!(
    PartyAdjudicatorOpposition,
    "Tumey v. Ohio, 273 U.S. 510 (1927)"
);

/// Axiom: Counsel and Witness oppose each other — ABA Model Rule 3.7
/// generally forbids a lawyer from being both counsel and witness.
pub struct CounselWitnessOpposition;

impl Axiom for CounselWitnessOpposition {
    fn verify(&self) -> Verdict {
        let opp: Vec<_> = LegalActorCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == LegalActorRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        let has_cw = opp.contains(&(LegalActorConcept::Counsel, LegalActorConcept::Witness));
        let has_wc = opp.contains(&(LegalActorConcept::Witness, LegalActorConcept::Counsel));
        if has_cw && has_wc {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CounselWitnessOpposition",
        "Counsel and Witness oppose each other (no overlapping roles)",
        "ABA Model Rule of Professional Conduct 3.7 (Lawyer as Witness)"
    );
}

pr4xis::register_axiom!(
    CounselWitnessOpposition,
    "ABA Model Rule of Professional Conduct 3.7"
);

/// Axiom: only Party leaves carry burdens of proof. Adjudicators and
/// Witnesses do not. FRCP Rule 17 + Restatement (Second) of Judgments
/// §27 ground the doctrine that burdens attach to parties-in-interest.
pub struct OnlyPartiesCarryBurden;

impl Axiom for OnlyPartiesCarryBurden {
    fn verify(&self) -> Verdict {
        let q = CarriesBurden;
        for c in LegalActorConcept::variants() {
            let bears = q.get(&c);
            match (is_party(c), is_leaf(c), bears) {
                // Party leaves must carry burdens.
                (true, true, Some(true)) => {}
                // Non-party leaves must not.
                (false, true, Some(false)) => {}
                // Abstract concepts (root, families) have no answer.
                (_, false, None) => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "OnlyPartiesCarryBurden",
        "burdens of proof attach to Party leaves only — not Adjudicators or Witnesses",
        "FRCP Rule 17; Restatement (Second) of Judgments §27 (1982)"
    );
}

pr4xis::register_axiom!(
    OnlyPartiesCarryBurden,
    "FRCP Rule 17; Restatement (Second) of Judgments §27"
);
