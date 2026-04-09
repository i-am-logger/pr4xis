use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

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

impl Entity for TemporalTense {
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

impl Entity for Aspect {
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

impl Entity for TenseAspect {
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

/// Tense transformation — a morphism between tense-aspect combinations.
/// These are the functors that change temporal reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenseShift {
    pub from: TenseAspect,
    pub to: TenseAspect,
}

impl Relationship for TenseShift {
    type Object = TenseAspect;
    fn source(&self) -> TenseAspect {
        self.from
    }
    fn target(&self) -> TenseAspect {
        self.to
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
        Some(TenseShift {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<TenseShift> {
        let all = Self::all_combinations();
        let mut m = Vec::new();

        // Identities
        for ta in &all {
            m.push(TenseShift { from: *ta, to: *ta });
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
    use praxis::category::validate::check_category_laws;

    #[test]
    fn twelve_combinations() {
        assert_eq!(TenseCategory::all_combinations().len(), 12);
    }

    #[test]
    fn category_laws() {
        check_category_laws::<TenseCategory>().unwrap();
    }

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
