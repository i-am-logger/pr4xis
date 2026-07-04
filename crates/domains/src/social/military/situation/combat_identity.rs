//! Combat identity — the STANAG 1241 standard-identity (allegiance) dimension of
//! a tracked entity, as an ontology that projects into the compliance IFF layer.
//!
//! A tracked entity's allegiance is not a free-form string — and "aircraft" is a
//! platform *type* (`compliance::classification::EntityType`), not an identity.
//! STANAG 1241 defines a fixed set of *standard identities* for exchanging
//! tactical identity information. This ontology carries that set, and
//! [`CombatIdentityConcept::to_iff`] projects it — **conservatively** — onto the
//! coarser LOAC IFF classification the compliance engagement stack consumes, so
//! that only a *confirmed* hostile is ever treated as engageable.
//!
//! # Literature
//!
//! - **STANAG 1241** — NATO Standard Identity Description Structure for Tactical
//!   Use (the NATO standard; restricted).
//! - **MIL-STD-2525D (2014)** *Common Warfighting Symbology* §5 — the public
//!   implementing standard defining the standard-identity / affiliation set:
//!   Pending, Unknown, Assumed Friend, Friend, Neutral, Suspect, Hostile.
//! - **Additional Protocol I (1977)** Art. 48 (distinction), Art. 50(1)
//!   (in-doubt-civilian) — why the projection to IFF is conservative.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::social::compliance::classification::IffClassification;

pr4xis::ontology! {
    name: "CombatIdentity",
    source: "STANAG 1241 NATO Standard Identity Description Structure for Tactical Use; MIL-STD-2525D (2014) Common Warfighting Symbology §5; Additional Protocol I (1977) Art. 48, 50",

    concepts: [Pending, Unknown, AssumedFriend, Friend, Neutral, Suspect, Hostile],

    labels: {
        Pending: ("en", "Pending",
            "STANAG 1241: identity has not yet been evaluated — evaluation is in progress."),
        Unknown: ("en", "Unknown",
            "STANAG 1241: evaluated, but the identity could not be determined."),
        AssumedFriend: ("en", "Assumed friend",
            "STANAG 1241: assumed friendly from behaviour/characteristics, not confirmed."),
        Friend: ("en", "Friend",
            "STANAG 1241: positively identified as friendly."),
        Neutral: ("en", "Neutral",
            "STANAG 1241: identified as neither friendly nor hostile (e.g. a non-belligerent)."),
        Suspect: ("en", "Suspect",
            "STANAG 1241: potentially hostile from behaviour/characteristics, not confirmed."),
        Hostile: ("en", "Hostile",
            "STANAG 1241: positively identified as hostile."),
    },

    opposes: [
        // Friend and Hostile are the polar confirmed allegiances.
        (Friend, Hostile),
        (Hostile, Friend),
    ],
}

/// The coarse threat disposition of a standard identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatDisposition {
    /// Friendly or assumed-friendly.
    Friendly,
    /// Neither friend nor foe.
    Neutral,
    /// Identity not resolved (pending / unknown).
    Uncertain,
    /// Confirmed or suspected hostile.
    Threatening,
}

/// Quality: the [`ThreatDisposition`] of each standard identity (a social status,
/// per DOLCE — allegiance inheres in a social relationship, not a physical body).
#[derive(Debug, Clone)]
pub struct DispositionOf;

impl Quality for DispositionOf {
    type Individual = CombatIdentityConcept;
    type Value = ThreatDisposition;
    const KIND: QualityKind = QualityKind::Social;

    fn get(&self, id: &CombatIdentityConcept) -> Option<ThreatDisposition> {
        use CombatIdentityConcept as C;
        use ThreatDisposition as D;
        Some(match id {
            C::Friend | C::AssumedFriend => D::Friendly,
            C::Neutral => D::Neutral,
            C::Pending | C::Unknown => D::Uncertain,
            C::Suspect | C::Hostile => D::Threatening,
        })
    }
}

impl Ontology for CombatIdentityOntology {
    type Cat = CombatIdentityCategory;
    type Qual = DispositionOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(IffProjectionIsConservative));
        axioms
    }
}

impl CombatIdentityConcept {
    /// Project a STANAG 1241 standard identity onto the LOAC IFF classification
    /// (`compliance::classification::IffClassification`) the engagement stack
    /// consumes.
    ///
    /// **Conservative by design** (Additional Protocol I Art. 48, 50(1) —
    /// distinction and the in-doubt rule): only a *confirmed* `Hostile` becomes
    /// `IffClassification::Hostile` (engageable under ROE). `Suspect` — merely
    /// *potentially* hostile — projects to `Unknown` (no engagement), never to
    /// Hostile; unresolved identities (`Pending` / `Unknown`) likewise project to
    /// `Unknown`.
    pub fn to_iff(&self) -> IffClassification {
        use CombatIdentityConcept as C;
        match self {
            C::Friend | C::AssumedFriend => IffClassification::Friend,
            C::Neutral => IffClassification::Neutral,
            C::Hostile => IffClassification::Hostile,
            // Not confirmed hostile — cannot be engaged as hostile.
            C::Suspect | C::Unknown | C::Pending => IffClassification::Unknown,
        }
    }
}

/// Axiom: the IFF projection is conservative — only a *confirmed* hostile
/// identity ever maps to the engageable `IffClassification::Hostile`.
///
/// This makes "positive identification required for engagement" a structural
/// fact: no `Suspect`, `Unknown`, or `Pending` entity can be projected onto the
/// engageable-hostile IFF state (Additional Protocol I Art. 48, the distinction
/// principle). Verified over every standard identity.
pub struct IffProjectionIsConservative;

impl Axiom for IffProjectionIsConservative {
    fn verify(&self) -> Verdict {
        use pr4xis::category::FinitelyGenerated;
        // `to_iff(id) == Hostile` holds for exactly the confirmed Hostile identity.
        let conservative = CombatIdentityConcept::variants().iter().all(|id| {
            (id.to_iff() == IffClassification::Hostile) == (*id == CombatIdentityConcept::Hostile)
        });
        if conservative {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "IffProjectionIsConservative",
        "only a confirmed Hostile standard identity projects to the engageable Hostile IFF — Suspect/Unknown/Pending never do (positive-ID-for-engagement)",
        "Additional Protocol I (1977) Art. 48 (distinction); STANAG 1241"
    );
}
pr4xis::register_axiom!(
    IffProjectionIsConservative,
    "Additional Protocol I (1977) Art. 48 (distinction); STANAG 1241"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CombatIdentityCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        CombatIdentityOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn seven_standard_identities() {
        assert_eq!(CombatIdentityConcept::variants().len(), 7);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn friend_hostile_oppose() {
        let opp: Vec<_> = CombatIdentityCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CombatIdentityRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            CombatIdentityConcept::Friend,
            CombatIdentityConcept::Hostile
        )));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn iff_projection_conservative_holds() {
        assert!(IffProjectionIsConservative.verify().is_ok());
        // Spot-check the safety-critical cases.
        assert_eq!(
            CombatIdentityConcept::Hostile.to_iff(),
            IffClassification::Hostile
        );
        assert_eq!(
            CombatIdentityConcept::Suspect.to_iff(),
            IffClassification::Unknown,
            "a suspect is not a confirmed hostile — must not be engageable",
        );
        assert_eq!(
            CombatIdentityConcept::Friend.to_iff(),
            IffClassification::Friend
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn disposition_total() {
        let q = DispositionOf;
        for id in CombatIdentityConcept::variants() {
            assert!(q.get(&id).is_some(), "{id:?} missing disposition");
        }
    }
}
