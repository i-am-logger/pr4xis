//! VerbNet as typed Rust data — the class-hierarchy shape [`super::reader::read_verbnet`]
//! populates.
//!
//! Mirrors [`crate::social::software::markup::xml::lmf::ontology`]'s
//! `WordNet`/`Synset` shape: a plain, hand-written struct family describing
//! what the loaded XML MEANS, populated by a hand-written reader over the
//! generic `XmlDocument` tree, not derived by any schema-driven codegen (no
//! source in this codebase gets one — see the `read_wordnet`/`read_owl`
//! precedent this mirrors).
//!
//! References:
//! - Kipper, K., Korhonen, A., Ryant, N. & Palmer, M. (2008). "A Large-scale
//!   Classification of English Verbs". Language Resources and Evaluation
//!   42(1):21-40.
//! - Levin, B. (1993). *English Verb Classes and Alternations*. University of
//!   Chicago Press.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

/// One `<MEMBER>` — a verb belonging to a [`VerbNetClass`], keyed by its
/// Princeton WordNet sense-key(s) (the `wn` attribute; space-separated when a
/// member spans multiple WordNet senses, e.g. `"kill%2:30:08 kill%2:30:03"`).
/// `wn` is `None` for the rare member with no attested WordNet sense at all
/// (VerbNet predates full WordNet coverage for a handful of lemmas).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbNetMember {
    /// The verb's surface lemma (e.g. `"cut"`).
    pub name: String,
    /// The raw Princeton WordNet sense-key token(s) from the `wn` attribute,
    /// space-split, unparsed (the sense-key → WordNet-synset crosswalk is
    /// [`crate::cognitive::linguistics::verbnet::store::oewn_sense_id_for_sense_key`]'s
    /// job, kept separate from this plain data carrier).
    pub wn_sense_keys: Vec<String>,
}

/// One `<SYNTAX>` child element of a `<FRAME>` — an ordered syntactic
/// constituent (`NP`, `VERB`, `PREP`, ...) of that frame's realization, in
/// document order. `value` carries the thematic role an `NP`/`PREP`
/// constituent is bound to (the `value="Theme"` attribute); a `VERB`
/// constituent carries none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbNetSyntaxRole {
    /// The syntactic element's tag name (`"NP"`, `"VERB"`, `"PREP"`, ...).
    pub element: String,
    /// The thematic role bound to this constituent (the `value=` attribute),
    /// when the element carries one.
    pub value: Option<String>,
}

/// One `<FRAME>` — one syntactic realization of a `VerbNetClass`'s members,
/// e.g. `representation-110.1`'s `"NP V NP"` / `"Basic Transitive"` frame for
/// "Black symbolizes mourning."
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbNetFrame {
    /// The `<DESCRIPTION primary="...">` — the coarse syntactic pattern
    /// (e.g. `"NP V NP"`).
    pub primary: String,
    /// The `<DESCRIPTION secondary="...">` — the frame's Levin (1993)
    /// alternation name (e.g. `"Basic Transitive"`).
    pub secondary: String,
    /// The `<SYNTAX>` children, in document order.
    pub syntax: Vec<VerbNetSyntaxRole>,
}

/// One `<VNCLASS>` or `<VNSUBCLASS>` — Levin's (1993) syntactic-alternation
/// verb classes are the same shape at every nesting depth (VerbNet's DTD
/// declares `VNSUBCLASS` as a structural clone of `VNCLASS`), so one
/// recursive type covers both: `stop-55.4`'s direct `<MEMBERS>` are this
/// class's `members`; its nested `<SUBCLASSES><VNSUBCLASS ID="stop-55.4-1">`
/// is one entry of `subclasses`, itself carrying `stop-55.4-1-1` one level
/// deeper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbNetClass {
    /// The class identifier (e.g. `"stop-55.4"`, `"stop-55.4-1-1"`).
    pub id: String,
    /// Verbs directly declared as members of THIS class (not inherited from
    /// an ancestor or contributed by a descendant).
    pub members: Vec<VerbNetMember>,
    /// Nested subclasses, each a syntactic-semantic refinement of this class
    /// (Levin 1993's diagnostic alternations narrow the membership).
    pub subclasses: Vec<VerbNetClass>,
    /// The `<THEMROLES><THEMROLE type="...">` thematic roles this class's
    /// frames draw on, in document order (e.g. `["Theme", "Co-Theme",
    /// "Context"]`).
    pub theme_roles: Vec<String>,
    /// The `<FRAMES><FRAME>` syntactic realizations declared directly on
    /// THIS class (not inherited).
    pub frames: Vec<VerbNetFrame>,
}

/// The full loaded VerbNet class collection — one entry per top-level
/// `<VNCLASS>` file in the archived collection (332 in VerbNet 3.3); each
/// entry's own `subclasses` carries its nested hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerbNet {
    pub classes: Vec<VerbNetClass>,
}

impl VerbNetClass {
    /// This class and every class nested under it, depth-first, self first —
    /// the traversal [`crate::cognitive::linguistics::verbnet::store::VerbNetStore`]
    /// uses to flatten the recursive tree into a flat, indexable class table.
    pub fn self_and_descendants(&self) -> Vec<&VerbNetClass> {
        let mut out = alloc::vec![self];
        for sub in &self.subclasses {
            out.extend(sub.self_and_descendants());
        }
        out
    }
}

impl VerbNet {
    /// Does `lemma` have a VerbNet class membership whose "Basic Transitive"
    /// frame realizes it as `NP V NP` with the FIRST `NP` bound to `Theme`
    /// and the SECOND to `Co-Theme` — the syntactic shape "the term 'X'
    /// means Y" needs (subject = Theme = the definiendum, object = Co-Theme
    /// = the definiens)? Returns the owning class id as the citable witness
    /// (e.g. `"representation-110.1"` for `"mean"`) when confirmed; `None`
    /// when `lemma` is not a member of any class, or is a member but no
    /// frame carries that exact `[Theme, Co-Theme]` NP-role sequence — this
    /// is a LOADED confirmation, never an assumption from SVO word order.
    ///
    /// Kipper, Korhonen, Ryant & Palmer (2008) "A Large-scale Classification
    /// of English Verbs", *Language Resources and Evaluation* 42(1):21-40 —
    /// VerbNet's `<THEMROLES>`/`<FRAMES>`/`<SYNTAX>` structure.
    #[must_use]
    pub fn basic_transitive_theme_order(&self, lemma: &str) -> Option<&str> {
        for top in &self.classes {
            for class in top.self_and_descendants() {
                if !class.members.iter().any(|m| m.name == lemma) {
                    continue;
                }
                for frame in &class.frames {
                    let np_roles: Vec<&str> = frame
                        .syntax
                        .iter()
                        .filter(|r| r.element == "NP")
                        .filter_map(|r| r.value.as_deref())
                        .collect();
                    if np_roles == ["Theme", "Co-Theme"] {
                        return Some(class.id.as_str());
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real `representation-110.1` shape (byte-exact-verified against
    /// `crates/domains/data/verbnet/verbnet-3.3.verbnet` — see
    /// `reader::tests::REPRESENTATION_110_1`), trimmed to the frame-1
    /// `"NP V NP"`/`"Basic Transitive"` shape this query needs.
    fn representation_110_1() -> VerbNet {
        VerbNet {
            classes: alloc::vec![VerbNetClass {
                id: "representation-110.1".to_string(),
                members: alloc::vec![
                    VerbNetMember {
                        name: "mean".to_string(),
                        wn_sense_keys: alloc::vec!["mean%2:32:03".to_string()],
                    },
                    VerbNetMember {
                        name: "denote".to_string(),
                        wn_sense_keys: alloc::vec!["denote%2:32:00".to_string()],
                    },
                ],
                subclasses: Vec::new(),
                theme_roles: alloc::vec![
                    "Theme".to_string(),
                    "Co-Theme".to_string(),
                    "Context".to_string(),
                ],
                frames: alloc::vec![
                    VerbNetFrame {
                        primary: "NP V NP".to_string(),
                        secondary: "Basic Transitive".to_string(),
                        syntax: alloc::vec![
                            VerbNetSyntaxRole {
                                element: "NP".to_string(),
                                value: Some("Theme".to_string()),
                            },
                            VerbNetSyntaxRole {
                                element: "VERB".to_string(),
                                value: None,
                            },
                            VerbNetSyntaxRole {
                                element: "NP".to_string(),
                                value: Some("Co-Theme".to_string()),
                            },
                        ],
                    },
                    VerbNetFrame {
                        primary: "NP V NP PP.manner".to_string(),
                        secondary: "NP-PP; Manner-PP".to_string(),
                        syntax: alloc::vec![
                            VerbNetSyntaxRole {
                                element: "NP".to_string(),
                                value: Some("Theme".to_string()),
                            },
                            VerbNetSyntaxRole {
                                element: "VERB".to_string(),
                                value: None,
                            },
                            VerbNetSyntaxRole {
                                element: "NP".to_string(),
                                value: Some("Co-Theme".to_string()),
                            },
                            VerbNetSyntaxRole {
                                element: "PREP".to_string(),
                                value: Some("in | for | to".to_string()),
                            },
                            VerbNetSyntaxRole {
                                element: "NP".to_string(),
                                value: Some("Context".to_string()),
                            },
                        ],
                    },
                ],
            }],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn finds_mean_via_representation_110_1() {
        let vn = representation_110_1();
        assert_eq!(
            vn.basic_transitive_theme_order("mean"),
            Some("representation-110.1")
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn returns_none_for_an_uncovered_lemma() {
        let vn = representation_110_1();
        assert_eq!(vn.basic_transitive_theme_order("eat"), None);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn the_query_is_per_class_not_per_member_and_none_for_an_absent_lemma() {
        // "denote" is a DIFFERENT member of the same class than "mean", but
        // the query checks the CLASS's frames (shared across all its
        // members), so it also resolves — proving the query is per-class,
        // not per-member. A lemma that is not a member at all yields None.
        let vn = representation_110_1();
        assert_eq!(
            vn.basic_transitive_theme_order("denote"),
            Some("representation-110.1")
        );
        assert_eq!(vn.basic_transitive_theme_order("nonexistent-verb"), None);
    }
}
