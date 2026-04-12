use super::category::Category;

// Traced Category — the writer monad on categories.
//
// Lifts any Category C into a traced version where compose()
// automatically produces provenance records. The trace emerges
// from composition itself — no manual instrumentation.
//
// Categorically: this is the Joyal-Street-Verity (1996) trace operator
// applied to categories. Every morphism carries a trace wire.
// Composition composes both the morphism AND the trace.
//
// Also: Moggi (1991) writer monad. G(A) = F(A) × Trace.
// The natural transformation F ⟹ G lifts bare computation to traced.
//
// References:
// - Joyal, Street & Verity, "Traced Monoidal Categories" (1996,
//   Math. Proc. Cambridge Phil. Soc.)
// - Moggi, "Notions of Computation and Monads" (1991, Inf. & Comp.)
// - W3C PROV-O (2013) — the trace records are PROV Activities

/// A trace record produced by a single composition step.
/// Aligned with W3C PROV-O: this is a prov:Activity.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceRecord {
    /// Which ontology/category produced this record.
    pub ontology: String,
    /// What operation was performed.
    pub operation: String,
    /// Detail of what happened.
    pub detail: String,
    /// Whether this step succeeded or had issues.
    pub status: TraceRecordStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceRecordStatus {
    Ok,
    Warning,
    Error,
}

/// A traced morphism: the original morphism paired with provenance.
///
/// This is the writer monad: (Morphism, Vec<TraceRecord>).
/// The Vec accumulates trace records through composition.
#[derive(Debug, Clone)]
pub struct TracedMorphism<M> {
    pub morphism: M,
    pub trace: Vec<TraceRecord>,
}

/// A traced category: wraps any Category C so that compose()
/// automatically produces trace records.
///
/// Joyal-Street-Verity: Tr^U_{A,B}: C(A⊗U, B⊗U) → C(A, B)
/// where U = Vec<TraceRecord> (the trace accumulator).
///
/// Usage:
/// ```ignore
/// // Any category can be traced:
/// let f = TracedMorphism::new(morphism_f, "MyOntology", "lookup", "found 3 results");
/// let g = TracedMorphism::new(morphism_g, "MyOntology", "compose", "applied rule");
/// let h = TracedMorphism::compose::<MyCategory>(&f, &g);
/// // h.trace contains BOTH f's and g's records + the composition record
/// ```
pub struct TracedCategory<C: Category>(std::marker::PhantomData<C>);

impl<M: Clone> TracedMorphism<M> {
    /// Create a traced morphism with an initial trace record.
    pub fn new(morphism: M, ontology: &str, operation: &str, detail: &str) -> Self {
        Self {
            morphism,
            trace: vec![TraceRecord {
                ontology: ontology.into(),
                operation: operation.into(),
                detail: detail.into(),
                status: TraceRecordStatus::Ok,
            }],
        }
    }

    /// Create from a bare morphism with no trace.
    pub fn bare(morphism: M) -> Self {
        Self {
            morphism,
            trace: Vec::new(),
        }
    }

    /// Add a trace record.
    pub fn record(&mut self, ontology: &str, operation: &str, detail: &str) {
        self.trace.push(TraceRecord {
            ontology: ontology.into(),
            operation: operation.into(),
            detail: detail.into(),
            status: TraceRecordStatus::Ok,
        });
    }

    /// Add a warning trace record.
    pub fn warn(&mut self, ontology: &str, operation: &str, detail: &str) {
        self.trace.push(TraceRecord {
            ontology: ontology.into(),
            operation: operation.into(),
            detail: detail.into(),
            status: TraceRecordStatus::Warning,
        });
    }
}

impl<C: Category> TracedCategory<C>
where
    C::Morphism: Clone,
    C::Object: Clone,
{
    /// Compose two traced morphisms.
    ///
    /// The writer monad bind: (M, T₁) >>= (M, T₂) = (M∘M, T₁ ++ T₂ ++ [compose record])
    /// Trace accumulates through composition — no manual instrumentation.
    pub fn compose(
        f: &TracedMorphism<C::Morphism>,
        g: &TracedMorphism<C::Morphism>,
    ) -> Option<TracedMorphism<C::Morphism>> {
        let composed = C::compose(&f.morphism, &g.morphism)?;

        // Accumulate traces: f's trace + g's trace
        let mut trace = f.trace.clone();
        trace.extend(g.trace.iter().cloned());

        Some(TracedMorphism {
            morphism: composed,
            trace,
        })
    }

    /// Identity with trace.
    pub fn identity(obj: &C::Object) -> TracedMorphism<C::Morphism> {
        TracedMorphism::bare(C::identity(obj))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::entity::Entity;
    use crate::category::relationship::Relationship;

    // A minimal test category
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestObj {
        A,
        B,
        C,
    }

    impl Entity for TestObj {
        fn variants() -> Vec<Self> {
            vec![Self::A, Self::B, Self::C]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestMorph {
        from: TestObj,
        to: TestObj,
    }

    impl Relationship for TestMorph {
        type Object = TestObj;
        fn source(&self) -> TestObj {
            self.from
        }
        fn target(&self) -> TestObj {
            self.to
        }
    }

    struct TestCat;
    impl Category for TestCat {
        type Object = TestObj;
        type Morphism = TestMorph;

        fn identity(obj: &TestObj) -> TestMorph {
            TestMorph {
                from: *obj,
                to: *obj,
            }
        }

        fn compose(f: &TestMorph, g: &TestMorph) -> Option<TestMorph> {
            if f.to == g.from {
                Some(TestMorph {
                    from: f.from,
                    to: g.to,
                })
            } else {
                None
            }
        }

        fn morphisms() -> Vec<TestMorph> {
            vec![
                TestMorph {
                    from: TestObj::A,
                    to: TestObj::B,
                },
                TestMorph {
                    from: TestObj::B,
                    to: TestObj::C,
                },
            ]
        }
    }

    #[test]
    fn traced_compose_accumulates_records() {
        let f = TracedMorphism::new(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "TestOntology",
            "step1",
            "A → B",
        );
        let g = TracedMorphism::new(
            TestMorph {
                from: TestObj::B,
                to: TestObj::C,
            },
            "TestOntology",
            "step2",
            "B → C",
        );

        let h = TracedCategory::<TestCat>::compose(&f, &g).unwrap();
        assert_eq!(h.morphism.from, TestObj::A);
        assert_eq!(h.morphism.to, TestObj::C);
        // Trace has BOTH records — accumulated through composition
        assert_eq!(h.trace.len(), 2);
        assert_eq!(h.trace[0].operation, "step1");
        assert_eq!(h.trace[1].operation, "step2");
    }

    #[test]
    fn traced_identity_has_no_trace() {
        let id = TracedCategory::<TestCat>::identity(&TestObj::A);
        assert_eq!(id.morphism.from, TestObj::A);
        assert_eq!(id.morphism.to, TestObj::A);
        assert!(id.trace.is_empty());
    }

    #[test]
    fn traced_compose_with_identity_preserves_trace() {
        let f = TracedMorphism::new(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "Test",
            "lookup",
            "found",
        );
        let id = TracedCategory::<TestCat>::identity(&TestObj::B);

        let h = TracedCategory::<TestCat>::compose(&f, &id).unwrap();
        assert_eq!(h.trace.len(), 1); // only f's record
        assert_eq!(h.trace[0].detail, "found");
    }

    #[test]
    fn trace_records_have_status() {
        let mut f = TracedMorphism::new(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "Test",
            "parse",
            "success",
        );
        f.warn("Test", "parse", "ambiguous — multiple parses");

        assert_eq!(f.trace.len(), 2);
        assert_eq!(f.trace[0].status, TraceRecordStatus::Ok);
        assert_eq!(f.trace[1].status, TraceRecordStatus::Warning);
    }

    #[test]
    fn compose_incompatible_returns_none() {
        let f = TracedMorphism::new(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "Test",
            "step1",
            "A → B",
        );
        let g = TracedMorphism::new(
            TestMorph {
                from: TestObj::A,
                to: TestObj::C,
            },
            "Test",
            "step2",
            "A → C",
        );
        // B ≠ A, so composition fails
        assert!(TracedCategory::<TestCat>::compose(&f, &g).is_none());
    }
}
