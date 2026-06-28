use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

// Tense/Aspect ontology — the temporal structure of events in language.
//
// Tense locates events in time. Aspect describes the internal temporal
// structure of events. Together they form a 2D system.
//
// References:
// - Reichenbach, Elements of Symbolic Logic (1947) — S/R/E model
// - Comrie, Tense (1985) — cross-linguistic tense systems
// - Comrie, Aspect (1976) — cross-linguistic aspect systems

/// Tense — when an event occurs relative to the utterance time.
/// Reichenbach (1947): tense is the relation between Speech time (S)
/// and Event time (E).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemporalTense {
    /// E before S: "The dog ran."
    Past,
    /// E overlaps S: "The dog runs."
    Present,
    /// E after S: "The dog will run."
    Future,
}

impl Concept for TemporalTense {}
impl FinitelyGenerated for TemporalTense {
    fn variants() -> Vec<Self> {
        vec![Self::Past, Self::Present, Self::Future]
    }
}

/// Aspect — the internal temporal structure of an event.
/// Comrie (1976): how the event unfolds over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aspect {
    /// Simple/perfective: event as a whole. "She wrote a letter."
    Simple,
    /// Progressive/imperfective: event in progress. "She is writing a letter."
    Progressive,
    /// Perfect: event completed with present relevance. "She has written a letter."
    Perfect,
    /// Perfect progressive: ongoing event with duration. "She has been writing."
    PerfectProgressive,
}

impl Concept for Aspect {}
impl FinitelyGenerated for Aspect {
    fn variants() -> Vec<Self> {
        vec![
            Self::Simple,
            Self::Progressive,
            Self::Perfect,
            Self::PerfectProgressive,
        ]
    }
}

/// A tense-aspect combination — the full temporal specification.
/// English has 12 tense-aspect combinations (3 tenses × 4 aspects).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenseAspect {
    pub tense: TemporalTense,
    pub aspect: Aspect,
}

impl Concept for TenseAspect {}
impl FinitelyGenerated for TenseAspect {
    fn variants() -> Vec<Self> {
        let mut v = Vec::new();
        for tense in TemporalTense::variants() {
            for aspect in Aspect::variants() {
                v.push(TenseAspect { tense, aspect });
            }
        }
        v
    }
}

/// Relation-kind tag for the tense-aspect category.
///
/// Per OBO-RO (Smith 2005), every arrow carries a canonical kind.
/// The tense category has a single relation type: a temporal shift
/// between tense-aspect combinations (Reichenbach 1947; Comrie 1976).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenseRelationKind {
    TemporalShift,
}

/// Tense transformation — a morphism between tense-aspect combinations.
/// These are the functors that change temporal reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenseShift {
    pub from: TenseAspect,
    pub to: TenseAspect,
}

impl Arrow for TenseShift {
    type Object = TenseAspect;
    type Kind = TenseRelationKind;

    fn source(&self) -> TenseAspect {
        self.from
    }
    fn target(&self) -> TenseAspect {
        self.to
    }
    fn kind(&self) -> TenseRelationKind {
        TenseRelationKind::TemporalShift
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("TenseShift"),
            description: Label::new_static(
                "temporal shift between tense-aspect combinations (Reichenbach 1947)",
            ),
            citation: Citation::parse_static("Reichenbach (1947); Comrie (1976)"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The tense-aspect category.
/// Objects are tense-aspect pairs. Morphisms are temporal shifts.
pub struct TenseCategory;

impl TenseCategory {
    /// All 12 English tense-aspect combinations.
    pub fn all_combinations() -> Vec<TenseAspect> {
        let mut combos = Vec::new();
        for tense in TemporalTense::variants() {
            for aspect in Aspect::variants() {
                combos.push(TenseAspect { tense, aspect });
            }
        }
        combos
    }
}

impl Category for TenseCategory {
    type Object = TenseAspect;
    type Morphism = TenseShift;

    fn identity(obj: &TenseAspect) -> TenseShift {
        TenseShift {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &TenseShift, g: &TenseShift) -> Option<TenseShift> {
        if f.to != g.from {
            return None;
        }
        if f.from == f.to {
            return Some(g.clone());
        }
        if g.from == g.to {
            return Some(f.clone());
        }
        let candidate = TenseShift {
            from: f.from,
            to: g.to,
        };
        // Closure law (Mac Lane CWM Ch. I §1): the composite must be in
        // `morphisms()`. The morphism set now includes diagonal shifts
        // (tense + aspect changing simultaneously), so general composition
        // closes.
        if Self::morphisms().contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<TenseShift> {
        let all = Self::all_combinations();
        let mut m = Vec::new();

        // Identities
        for ta in &all {
            m.push(TenseShift { from: *ta, to: *ta });
        }

        // Every ordered pair of distinct (tense, aspect) combinations is a
        // structural morphism. This guarantees closure under composition
        // for tense ∘ aspect chains and supports both single-dimension and
        // diagonal shifts (Comrie 1976 *Aspect*).
        for &from in &all {
            for &to in &all {
                if from != to {
                    m.push(TenseShift { from, to });
                }
            }
        }

        // Tense shifts (same aspect, different tense)
        for aspect in Aspect::variants() {
            let tenses = TemporalTense::variants();
            for &from_t in &tenses {
                for &to_t in &tenses {
                    if from_t != to_t {
                        m.push(TenseShift {
                            from: TenseAspect {
                                tense: from_t,
                                aspect,
                            },
                            to: TenseAspect {
                                tense: to_t,
                                aspect,
                            },
                        });
                    }
                }
            }
        }

        // Aspect shifts (same tense, different aspect)
        for tense in TemporalTense::variants() {
            let aspects = Aspect::variants();
            for &from_a in &aspects {
                for &to_a in &aspects {
                    if from_a != to_a {
                        m.push(TenseShift {
                            from: TenseAspect {
                                tense,
                                aspect: from_a,
                            },
                            to: TenseAspect {
                                tense,
                                aspect: to_a,
                            },
                        });
                    }
                }
            }
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn twelve_combinations() {
        assert_eq!(TenseCategory::all_combinations().len(), 12);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<TenseCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn past_to_present_shift() {
        let past_simple = TenseAspect {
            tense: TemporalTense::Past,
            aspect: Aspect::Simple,
        };
        let present_simple = TenseAspect {
            tense: TemporalTense::Present,
            aspect: Aspect::Simple,
        };
        let morphisms = TenseCategory::morphisms();
        assert!(morphisms.contains(&TenseShift {
            from: past_simple,
            to: present_simple,
        }));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn simple_to_progressive_shift() {
        let present_simple = TenseAspect {
            tense: TemporalTense::Present,
            aspect: Aspect::Simple,
        };
        let present_progressive = TenseAspect {
            tense: TemporalTense::Present,
            aspect: Aspect::Progressive,
        };
        let morphisms = TenseCategory::morphisms();
        assert!(morphisms.contains(&TenseShift {
            from: present_simple,
            to: present_progressive,
        }));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn composition_tense_then_aspect() {
        let past_simple = TenseAspect {
            tense: TemporalTense::Past,
            aspect: Aspect::Simple,
        };
        let present_simple = TenseAspect {
            tense: TemporalTense::Present,
            aspect: Aspect::Simple,
        };
        let present_progressive = TenseAspect {
            tense: TemporalTense::Present,
            aspect: Aspect::Progressive,
        };

        let shift1 = TenseShift {
            from: past_simple,
            to: present_simple,
        };
        let shift2 = TenseShift {
            from: present_simple,
            to: present_progressive,
        };

        let composed = TenseCategory::compose(&shift1, &shift2);
        assert_eq!(
            composed,
            Some(TenseShift {
                from: past_simple,
                to: present_progressive,
            })
        );
    }
}
