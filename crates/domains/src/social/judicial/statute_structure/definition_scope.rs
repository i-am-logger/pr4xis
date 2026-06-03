//! Lex-specialis resolution of statutory definitions — which definition of a
//! term governs a given use (the Q2 realization, sibling of the lexical
//! sense-elevation order).
//!
//! A term like "person" may be defined at several scopes: in the enacting
//! section, for a whole title (26 U.S.C. §7701, "For purposes of this title"),
//! by the Dictionary Act for the entire U.S. Code (1 U.S.C. §1), or not at all
//! (ordinary meaning). When a use falls under more than one, the MORE SPECIFIC
//! definition displaces the more general — the general/specific canon (Scalia &
//! Garner 2012 §28). This is modelled as a defeasible PRIORITY ordering (Reiter
//! 1980; Nute's defeasible logic; Prakken & Sartor 1996), NOT disjoint
//! namespaces, because resolution is a FALL-THROUGH (section → title →
//! Dictionary Act → ordinary) and fall-through is an ordering. A separate SOFT
//! defeater carries "unless the context indicates otherwise" (1 U.S.C. §1) and
//! §7701's "manifestly incompatible" (Rowland v. California Men's Colony, 1993).
//!
//! The applicability of an `Enacted` definition is the subtree its
//! [`PinpointCite`] governs (the use lies within it iff the scope cite is a
//! prefix of the use cite), and its specificity rises with the scope's depth —
//! so a section scope outranks a title scope, which outranks the Dictionary Act,
//! which outranks ordinary meaning. The precedence is a strict partial order
//! (verified by [`DefinitionScopePrecedenceIsStrictPartialOrder`]), so a
//! well-defined governing definition always exists.
//!
//! # Literature
//!
//! - **Scalia, A. & Garner, B. A. (2012)** *Reading Law: The Interpretation of
//!   Legal Texts* §28 (the general/specific canon) — the more specific
//!   provision governs.
//! - **1 U.S.C. §1** (the Dictionary Act); **26 U.S.C. §7701** ("For purposes of
//!   this title …") — the statutory scope language.
//! - **Reiter, R. (1980)** "A Logic for Default Reasoning", *Artificial
//!   Intelligence* 13 — defaults applied unless defeated.
//! - **Prakken, H. & Sartor, G. (1996)** "A Dialectical Model of Assessing
//!   Conflicting Arguments in Legal Reasoning", *AI & Law* 4 — defeasible rule
//!   priorities (lex specialis as a priority relation).
//! - **Rowland v. California Men's Colony, 506 U.S. 194 (1993)** — the
//!   "unless the context indicates otherwise" defeater is a soft, contextual one.

#[allow(unused_imports)]
use alloc::{boxed::Box, string::String, string::ToString, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::cognitive::linguistics::lemon::lexicon::{ConceptRef, Lexicon};
use crate::formal::meta::identifier_format::Identifier;
use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::ontology::{LegalRelation, RelationType};

/// The applicability scope of a statutory definition — the lex-specialis ladder.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefinitionScope {
    /// A definition enacted with a stated applicability subtree — the
    /// [`PinpointCite`] it governs ("in this section / chapter / title …"). It
    /// governs every use whose citation it is a prefix of; its specificity
    /// rises with the scope's depth (a section scope outranks a title scope).
    Enacted(PinpointCite),
    /// The Dictionary Act (1 U.S.C. §1) — the default for the whole U.S. Code,
    /// displaced by any `Enacted` scope that governs the use.
    DictionaryAct,
    /// No statutory definition: the ordinary lexical meaning (the general sense).
    OrdinaryMeaning,
}

impl DefinitionScope {
    /// Does this scope govern a use at `use_cite`? An `Enacted` scope governs a
    /// use iff the scope's citation is a (non-strict) prefix of the use's — the
    /// use lies within the subtree the definition applies to. The Dictionary Act
    /// and ordinary meaning govern everywhere (the fall-through floors).
    #[must_use]
    pub fn governs(&self, use_cite: &PinpointCite) -> bool {
        match self {
            DefinitionScope::Enacted(scope) => use_cite.segments.starts_with(&scope.segments),
            DefinitionScope::DictionaryAct | DefinitionScope::OrdinaryMeaning => true,
        }
    }

    /// The lex-specialis specificity — a higher value displaces a lower one
    /// (Scalia & Garner §28). An `Enacted` scope's specificity rises with its
    /// depth, always above the Dictionary-Act default, which is above ordinary
    /// meaning.
    #[must_use]
    pub fn specificity(&self) -> u32 {
        match self {
            DefinitionScope::Enacted(cite) => 2 + cite.segments.len() as u32,
            DefinitionScope::DictionaryAct => 1,
            DefinitionScope::OrdinaryMeaning => 0,
        }
    }

    /// Lex-specialis precedence: `self` displaces `other` for a term they both
    /// govern iff `self` is strictly more specific. A strict partial order
    /// (irreflexive, asymmetric, transitive) — see
    /// [`DefinitionScopePrecedenceIsStrictPartialOrder`].
    #[must_use]
    pub fn displaces(&self, other: &DefinitionScope) -> bool {
        self.specificity() > other.specificity()
    }
}

/// A statutory definition: a defined `term`, its applicability `scope`, and the
/// legal concept it binds the term to (`defines` — a reference into the legal
/// lexicon, the sense a use elevates to in the legal domain).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegalDefinition {
    pub term: String,
    pub scope: DefinitionScope,
    pub defines: ConceptRef,
}

impl LegalDefinition {
    /// The `Defines` morphism this definition contributes to the legal relation
    /// graph: the defining provision (source) establishes the meaning of the
    /// `term` (target). `None` for `OrdinaryMeaning` — no statutory definition,
    /// hence no morphism. Source and target are CURIE-typed [`Identifier`]s
    /// (W3C CURIE Syntax 1.0): the provision as `usc:<labels>` (the Dictionary
    /// Act is `usc:1-1`, i.e. 1 U.S.C. §1) and the term as `term:<term>`.
    pub fn as_defines_relation(&self) -> Option<LegalRelation> {
        let from_local = match &self.scope {
            DefinitionScope::Enacted(cite) => cite
                .segments
                .iter()
                .map(|s| s.label.as_str())
                .collect::<Vec<_>>()
                .join("-"),
            DefinitionScope::DictionaryAct => "1-1".to_string(),
            DefinitionScope::OrdinaryMeaning => return None,
        };
        let from = Identifier::curie(alloc::format!("usc:{from_local}")).ok()?;
        let to = Identifier::curie(alloc::format!("term:{}", self.term.replace(' ', "_"))).ok()?;
        Some(LegalRelation {
            from,
            to,
            relation: RelationType::Defines,
        })
    }
}

/// Resolve which definition of a term governs a use at `use_cite`: the MOST
/// SPECIFIC `candidate` whose scope governs the use (lex specialis), skipping
/// any the `contextual_defeater` rejects ("unless the context indicates
/// otherwise" / "manifestly incompatible", Rowland 1993) so resolution falls
/// through to the next-most-specific governing definition. `None` when nothing
/// governs (the caller falls back to ordinary meaning).
pub fn resolve_definition<'a, F>(
    use_cite: &PinpointCite,
    candidates: &'a [LegalDefinition],
    contextual_defeater: F,
) -> Option<&'a LegalDefinition>
where
    F: Fn(&LegalDefinition) -> bool,
{
    candidates
        .iter()
        .filter(|d| d.scope.governs(use_cite) && !contextual_defeater(d))
        .max_by_key(|d| d.scope.specificity())
}

/// The definitional layer a statute contributes — the terms it defines, each
/// with its applicability scope and the concept it binds. Title 1 (the
/// Dictionary Act) is the canonical inhabitant: it defines "person", "whoever",
/// "vessel", … for the whole U.S. Code, and every other title resolves a use of
/// those terms through it, a more specific title definition displacing it (lex
/// specialis). This is the `LegalLexicon` a `Statute` `Adjoins` in the source
/// taxonomy, made queryable.
#[derive(Debug, Clone, Default)]
pub struct DefinitionLexicon {
    definitions: Vec<LegalDefinition>,
}

impl DefinitionLexicon {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a definition this layer contributes.
    pub fn define(&mut self, definition: LegalDefinition) {
        self.definitions.push(definition);
    }

    /// The concept a use of `term` at `use_cite` resolves to — the most specific
    /// governing definition (lex specialis), skipping any the `defeater` rejects
    /// ("unless the context indicates otherwise"). `None` for ordinary meaning
    /// (no governing statutory definition).
    pub fn resolve<F>(
        &self,
        term: &str,
        use_cite: &PinpointCite,
        defeater: F,
    ) -> Option<&ConceptRef>
    where
        F: Fn(&LegalDefinition) -> bool,
    {
        self.definitions
            .iter()
            .filter(|d| d.term == term && d.scope.governs(use_cite) && !defeater(d))
            .max_by_key(|d| d.scope.specificity())
            .map(|d| &d.defines)
    }

    /// Mint this layer's terms into `lexicon` as legal-domain senses — each
    /// defined term gains a `"legal"`-register `Sense` on its ONE shared entry,
    /// alongside whatever general sense it already carries, so a use in the legal
    /// register elevates to the statutory meaning (the sense-elevation order)
    /// while the general meaning stays reachable.
    pub fn mint_into(&self, lexicon: &mut Lexicon) {
        for d in &self.definitions {
            lexicon.add_sense(
                d.term.clone(),
                d.defines.ontology.clone(),
                d.defines.concept.clone(),
                Some("legal".to_string()),
            );
        }
    }

    /// The definitions this layer holds.
    #[must_use]
    pub fn definitions(&self) -> &[LegalDefinition] {
        &self.definitions
    }
}

/// The lex-specialis definition-precedence order is a STRICT PARTIAL ORDER.
///
/// For a term defined at several scopes to have a well-defined governing
/// definition, the precedence `a ≺ b ⟺ a.specificity() > b.specificity()` must
/// be irreflexive, asymmetric and transitive. Since it is induced by a total
/// grading into a totally-ordered codomain it always is; this axiom discharges
/// the three laws over the four tiers a term such as "person" can carry: a
/// section-enacted definition, a title-enacted one, the Dictionary Act, and
/// ordinary meaning (Scalia & Garner §28; Prakken & Sartor 1996).
#[derive(Debug)]
pub struct DefinitionScopePrecedenceIsStrictPartialOrder;

impl Axiom for DefinitionScopePrecedenceIsStrictPartialOrder {
    fn verify(&self) -> Verdict {
        use crate::social::judicial::citation::ontology::PinpointCitationConcept as L;

        let scopes = [
            DefinitionScope::Enacted(
                PinpointCite::new()
                    .push(L::Title, "26")
                    .push(L::Section, "7701")
                    .push(L::Subsection, "a"),
            ),
            DefinitionScope::Enacted(
                PinpointCite::new()
                    .push(L::Title, "26")
                    .push(L::Section, "7701"),
            ),
            DefinitionScope::DictionaryAct,
            DefinitionScope::OrdinaryMeaning,
        ];
        for a in &scopes {
            // Irreflexive: ¬(a ≺ a).
            if a.displaces(a) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            for b in &scopes {
                // Asymmetric: a ≺ b ⟹ ¬(b ≺ a).
                if a.displaces(b) && b.displaces(a) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                for c in &scopes {
                    // Transitive: a ≺ b ∧ b ≺ c ⟹ a ≺ c.
                    if a.displaces(b) && b.displaces(c) && !a.displaces(c) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DefinitionScopePrecedenceIsStrictPartialOrder",
        "the lex-specialis definition-precedence order (more specific displaces more general) is a strict partial order (irreflexive, asymmetric, transitive)",
        "Scalia & Garner (2012) Reading Law §28; 1 U.S.C. §1; 26 U.S.C. §7701; Reiter (1980) AIJ 13; Prakken & Sartor (1996) AI & Law 4"
    );
}
pr4xis::register_axiom!(
    DefinitionScopePrecedenceIsStrictPartialOrder,
    "Scalia & Garner (2012) Reading Law §28; Reiter (1980) AIJ 13; Prakken & Sartor (1996) AI & Law 4"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::judicial::citation::ontology::PinpointCitationConcept as L;

    fn cite(parts: &[(L, &str)]) -> PinpointCite {
        let mut c = PinpointCite::new();
        for (lvl, lbl) in parts {
            c = c.push(*lvl, *lbl);
        }
        c
    }

    fn person(scope: DefinitionScope) -> LegalDefinition {
        LegalDefinition {
            term: "person".to_string(),
            scope,
            defines: ConceptRef {
                ontology: "us_legal_lexicon".to_string(),
                concept: "person".to_string(),
            },
        }
    }

    #[test]
    fn precedence_axiom_holds() {
        assert!(
            DefinitionScopePrecedenceIsStrictPartialOrder
                .verify()
                .is_ok()
        );
    }

    #[test]
    fn more_specific_scope_displaces_more_general() {
        let title = DefinitionScope::Enacted(cite(&[(L::Title, "26"), (L::Section, "7701")]));
        let section = DefinitionScope::Enacted(cite(&[
            (L::Title, "26"),
            (L::Section, "7701"),
            (L::Subsection, "a"),
        ]));
        assert!(section.displaces(&title));
        assert!(title.displaces(&DefinitionScope::DictionaryAct));
        assert!(DefinitionScope::DictionaryAct.displaces(&DefinitionScope::OrdinaryMeaning));
        // …and the order does NOT run the other way.
        assert!(!title.displaces(&section));
    }

    #[test]
    fn governs_by_containment() {
        let title_def = DefinitionScope::Enacted(cite(&[(L::Title, "26")]));
        // A use anywhere within Title 26 is governed…
        assert!(title_def.governs(&cite(&[(L::Title, "26"), (L::Section, "61")])));
        // …a use in Title 18 is not.
        assert!(!title_def.governs(&cite(&[(L::Title, "18"), (L::Section, "1")])));
    }

    #[test]
    fn resolve_picks_most_specific_governing_definition() {
        let use_cite = cite(&[(L::Title, "26"), (L::Section, "7701"), (L::Subsection, "a")]);
        let candidates = [
            person(DefinitionScope::DictionaryAct),
            person(DefinitionScope::Enacted(cite(&[
                (L::Title, "26"),
                (L::Section, "7701"),
            ]))),
        ];
        let chosen = resolve_definition(&use_cite, &candidates, |_| false).expect("a definition");
        assert!(
            matches!(chosen.scope, DefinitionScope::Enacted(_)),
            "the title definition displaces the Dictionary Act"
        );
    }

    #[test]
    fn contextual_defeater_falls_through_to_next() {
        // "unless the context indicates otherwise": defeat the title definition,
        // and the Dictionary Act governs instead.
        let use_cite = cite(&[(L::Title, "26"), (L::Section, "7701")]);
        let title_def = person(DefinitionScope::Enacted(cite(&[
            (L::Title, "26"),
            (L::Section, "7701"),
        ])));
        let candidates = [person(DefinitionScope::DictionaryAct), title_def.clone()];
        let chosen =
            resolve_definition(&use_cite, &candidates, |d| *d == title_def).expect("a definition");
        assert!(
            matches!(chosen.scope, DefinitionScope::DictionaryAct),
            "the defeated title definition falls through to the Dictionary Act"
        );
    }

    #[test]
    fn definition_lexicon_resolves_lex_specialis_and_mints_legal_sense() {
        use crate::cognitive::linguistics::lemon::lexicon::Lexicon;

        // Title 1 (the Dictionary Act) defines "person" code-wide; Title 26
        // §7701 redefines it for its own title.
        let mut layer = DefinitionLexicon::new();
        layer.define(LegalDefinition {
            term: "person".to_string(),
            scope: DefinitionScope::DictionaryAct,
            defines: ConceptRef {
                ontology: "us_legal_lexicon".to_string(),
                concept: "dictionary_act_person".to_string(),
            },
        });
        layer.define(LegalDefinition {
            term: "person".to_string(),
            scope: DefinitionScope::Enacted(cite(&[(L::Title, "26"), (L::Section, "7701")])),
            defines: ConceptRef {
                ontology: "us_legal_lexicon".to_string(),
                concept: "title26_person".to_string(),
            },
        });

        // A use within Title 26 §7701 resolves to the title-specific definition…
        let in_t26 = cite(&[(L::Title, "26"), (L::Section, "7701"), (L::Subsection, "a")]);
        assert_eq!(
            layer.resolve("person", &in_t26, |_| false).unwrap().concept,
            "title26_person"
        );
        // …a use elsewhere falls through to the Dictionary Act.
        let in_t18 = cite(&[(L::Title, "18"), (L::Section, "1")]);
        assert_eq!(
            layer.resolve("person", &in_t18, |_| false).unwrap().concept,
            "dictionary_act_person"
        );

        // Minting puts the legal senses onto the shared lexicon atom, alongside
        // the general sense, and the legal register elevates to a legal sense.
        let mut lex = Lexicon::new("en");
        lex.add_sense("person", "english_wordnet", "person.n.01", None);
        layer.mint_into(&mut lex);
        assert!(lex.lookup("person").unwrap().senses.len() >= 2);
        let legal = lex.resolve("person", Some("legal")).expect("a sense");
        assert_eq!(legal.reference.ontology, "us_legal_lexicon");
        // The general sense is still reachable by default.
        assert_eq!(
            lex.resolve("person", None).unwrap().reference.ontology,
            "english_wordnet"
        );
    }

    #[test]
    fn dictionary_act_definition_yields_a_defines_morphism() {
        let def = person(DefinitionScope::DictionaryAct);
        let rel = def.as_defines_relation().expect("a Defines morphism");
        assert_eq!(rel.relation, RelationType::Defines);
        assert_eq!(rel.from.value(), "usc:1-1"); // 1 U.S.C. §1, the Dictionary Act
        assert_eq!(rel.to.value(), "term:person");
        // An Enacted definition names its provision; ordinary meaning yields none.
        let enacted = person(DefinitionScope::Enacted(cite(&[
            (L::Title, "26"),
            (L::Section, "7701"),
        ])));
        assert_eq!(
            enacted.as_defines_relation().unwrap().from.value(),
            "usc:26-7701"
        );
        assert!(
            person(DefinitionScope::OrdinaryMeaning)
                .as_defines_relation()
                .is_none()
        );
    }
}
