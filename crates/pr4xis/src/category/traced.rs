use super::category::Category;
use super::monad::Writer;
use super::monoid::Monoid;

// Traced Category — the writer monad applied to categories.
//
// Lifts any Category C into a traced version where compose()
// automatically accumulates provenance records through the
// Writer monad. The trace emerges from composition itself.
//
// TracedMorphism<M> = Writer<Vec<TraceRecord>, M>
//
// The monoid is Vec<TraceRecord> (concatenation).
// The monad is Writer: bind sequences computations and accumulates logs.
//
// Categorically: this is Joyal-Street-Verity (1996) trace operator.
// Algebraically: this is Moggi (1991) writer monad.
//
// References:
// - Joyal, Street & Verity, "Traced Monoidal Categories" (1996)
// - Moggi, "Notions of Computation and Monads" (1991)
// - W3C PROV-O (2013) — trace records are PROV Activities

/// A trace record produced by a single computation step.
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

/// A traced morphism: the writer monad Writer<Vec<TraceRecord>, M>.
///
/// The morphism carries its provenance. Composition concatenates traces
/// via the Vec monoid — no manual instrumentation needed.
pub type TracedMorphism<M> = Writer<Vec<TraceRecord>, M>;

/// Convenience constructors for traced morphisms.
pub trait TracedMorphismExt<M: Clone + std::fmt::Debug> {
    /// Create a traced morphism with an initial trace record.
    fn traced(morphism: M, ontology: &str, operation: &str, detail: &str) -> TracedMorphism<M>;

    /// Create from a bare morphism with no trace.
    fn bare(morphism: M) -> TracedMorphism<M>;

    /// Add a trace record.
    fn record(&mut self, ontology: &str, operation: &str, detail: &str);

    /// Add a warning trace record.
    fn warn(&mut self, ontology: &str, operation: &str, detail: &str);
}

impl<M: Clone + std::fmt::Debug> TracedMorphismExt<M> for TracedMorphism<M> {
    fn traced(morphism: M, ontology: &str, operation: &str, detail: &str) -> Self {
        Writer::new(
            morphism,
            vec![TraceRecord {
                ontology: ontology.into(),
                operation: operation.into(),
                detail: detail.into(),
                status: TraceRecordStatus::Ok,
            }],
        )
    }

    fn bare(morphism: M) -> Self {
        Writer::pure(morphism)
    }

    fn record(&mut self, ontology: &str, operation: &str, detail: &str) {
        self.log.push(TraceRecord {
            ontology: ontology.into(),
            operation: operation.into(),
            detail: detail.into(),
            status: TraceRecordStatus::Ok,
        });
    }

    fn warn(&mut self, ontology: &str, operation: &str, detail: &str) {
        self.log.push(TraceRecord {
            ontology: ontology.into(),
            operation: operation.into(),
            detail: detail.into(),
            status: TraceRecordStatus::Warning,
        });
    }
}

/// A traced category: wraps any Category C with the writer monad.
///
/// Every morphism becomes Writer<Vec<TraceRecord>, M>.
/// Composition accumulates traces via the Vec monoid.
pub struct TracedCategory<C: Category>(std::marker::PhantomData<C>);

impl<C: Category> TracedCategory<C>
where
    C::Morphism: Clone,
    C::Object: Clone,
{
    /// Compose two traced morphisms.
    ///
    /// Writer bind: (m₁, w₁) >>= (m₂, w₂) → (m₁∘m₂, w₁ ++ w₂)
    pub fn compose(
        f: &TracedMorphism<C::Morphism>,
        g: &TracedMorphism<C::Morphism>,
    ) -> Option<TracedMorphism<C::Morphism>> {
        let composed = C::compose(&f.value, &g.value)?;
        Some(Writer::new(composed, f.log.combine(&g.log)))
    }

    /// Identity with empty trace.
    pub fn identity(obj: &C::Object) -> TracedMorphism<C::Morphism> {
        Writer::pure(C::identity(obj))
    }
}

// ---- Algebraic structure integrations ----

/// Convert a flat trace (Vec<TraceRecord>) into a Cofree tree.
///
/// The Cofree comonad gives each trace record its full context.
/// The root is the first record, children are subsequent records.
///
/// Reference: Uustalu & Vene, "Comonadic Notions of Computation" (2008)
pub fn trace_to_cofree(records: &[TraceRecord]) -> super::comonad::Cofree<TraceRecord> {
    if records.is_empty() {
        return super::comonad::Cofree::leaf(TraceRecord {
            ontology: "empty".into(),
            operation: "none".into(),
            detail: "no trace".into(),
            status: TraceRecordStatus::Ok,
        });
    }
    if records.len() == 1 {
        return super::comonad::Cofree::leaf(records[0].clone());
    }
    super::comonad::Cofree::node(
        records[0].clone(),
        records[1..]
            .iter()
            .map(|r| super::comonad::Cofree::leaf(r.clone()))
            .collect(),
    )
}

/// Fold a trace tree using a catamorphism to produce a summary.
///
/// Reference: Meijer, Fokkinga & Paterson (1991)
pub fn fold_trace<F: 'static>(
    tree: &super::comonad::Cofree<TraceRecord>,
    alg: &super::algebra::Algebra<TraceRecord, F>,
) -> F {
    super::algebra::cata(alg, tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::entity::Entity;
    use crate::category::relationship::Relationship;

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
        type Kind = ();
        fn source(&self) -> TestObj {
            self.from
        }
        fn target(&self) -> TestObj {
            self.to
        }
        fn kind(&self) {}
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
        let f = TracedMorphism::traced(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "TestOntology",
            "step1",
            "A → B",
        );
        let g = TracedMorphism::traced(
            TestMorph {
                from: TestObj::B,
                to: TestObj::C,
            },
            "TestOntology",
            "step2",
            "B → C",
        );

        let h = TracedCategory::<TestCat>::compose(&f, &g).unwrap();
        assert_eq!(h.value.from, TestObj::A);
        assert_eq!(h.value.to, TestObj::C);
        assert_eq!(h.log.len(), 2);
        assert_eq!(h.log[0].operation, "step1");
        assert_eq!(h.log[1].operation, "step2");
    }

    #[test]
    fn traced_identity_has_no_trace() {
        let id = TracedCategory::<TestCat>::identity(&TestObj::A);
        assert_eq!(id.value.from, TestObj::A);
        assert_eq!(id.value.to, TestObj::A);
        assert!(id.log.is_empty());
    }

    #[test]
    fn traced_compose_with_identity_preserves_trace() {
        let f = TracedMorphism::traced(
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
        assert_eq!(h.log.len(), 1);
        assert_eq!(h.log[0].detail, "found");
    }

    #[test]
    fn trace_records_have_status() {
        let mut f = TracedMorphism::traced(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "Test",
            "parse",
            "success",
        );
        f.warn("Test", "parse", "ambiguous — multiple parses");

        assert_eq!(f.log.len(), 2);
        assert_eq!(f.log[0].status, TraceRecordStatus::Ok);
        assert_eq!(f.log[1].status, TraceRecordStatus::Warning);
    }

    #[test]
    fn compose_incompatible_returns_none() {
        let f = TracedMorphism::traced(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "Test",
            "step1",
            "A → B",
        );
        let g = TracedMorphism::traced(
            TestMorph {
                from: TestObj::A,
                to: TestObj::C,
            },
            "Test",
            "step2",
            "A → C",
        );
        assert!(TracedCategory::<TestCat>::compose(&f, &g).is_none());
    }

    // --- Writer monad law verification for TracedCategory ---

    #[test]
    fn writer_monad_left_identity() {
        // pure(m) >>= f = f(m)
        let m = TestMorph {
            from: TestObj::A,
            to: TestObj::B,
        };
        let traced = TracedMorphism::<TestMorph>::bare(m.clone());
        let result = traced.bind(|morph| TracedMorphism::traced(morph, "Test", "step", "applied"));
        assert_eq!(result.value, m);
        assert_eq!(result.log.len(), 1);
    }

    #[test]
    fn writer_monad_right_identity() {
        // m >>= pure = m
        let traced = TracedMorphism::traced(
            TestMorph {
                from: TestObj::A,
                to: TestObj::B,
            },
            "Test",
            "step",
            "done",
        );
        let original_log = traced.log.clone();
        let result = traced.bind(TracedMorphism::<TestMorph>::pure);
        assert_eq!(result.log, original_log);
    }

    // --- Algebraic integration tests ---

    #[test]
    fn trace_to_cofree_tree() {
        let records = vec![
            TraceRecord {
                ontology: "Language".into(),
                operation: "tokenize".into(),
                detail: "5 tokens".into(),
                status: TraceRecordStatus::Ok,
            },
            TraceRecord {
                ontology: "Grammar".into(),
                operation: "parse".into(),
                detail: "S[dcl]".into(),
                status: TraceRecordStatus::Ok,
            },
        ];

        let tree = trace_to_cofree(&records);
        assert_eq!(tree.extract().ontology, "Language");
        assert_eq!(tree.tail.len(), 1);
        assert_eq!(tree.tail[0].extract().ontology, "Grammar");
    }

    #[test]
    fn fold_trace_counts_steps() {
        let records = vec![
            TraceRecord {
                ontology: "A".into(),
                operation: "op".into(),
                detail: "d".into(),
                status: TraceRecordStatus::Ok,
            },
            TraceRecord {
                ontology: "B".into(),
                operation: "op".into(),
                detail: "d".into(),
                status: TraceRecordStatus::Ok,
            },
            TraceRecord {
                ontology: "C".into(),
                operation: "op".into(),
                detail: "d".into(),
                status: TraceRecordStatus::Warning,
            },
        ];

        let tree = trace_to_cofree(&records);
        let count_alg =
            crate::category::algebra::Algebra::new(|_record: &TraceRecord, children: &[usize]| {
                1 + children.iter().sum::<usize>()
            });
        let total = fold_trace(&tree, &count_alg);
        assert_eq!(total, 3);
    }

    #[test]
    fn fold_trace_collects_ontologies() {
        let records = vec![
            TraceRecord {
                ontology: "Language".into(),
                operation: "tok".into(),
                detail: "d".into(),
                status: TraceRecordStatus::Ok,
            },
            TraceRecord {
                ontology: "Grammar".into(),
                operation: "parse".into(),
                detail: "d".into(),
                status: TraceRecordStatus::Ok,
            },
        ];

        let tree = trace_to_cofree(&records);
        let collect_alg = crate::category::algebra::Algebra::new(
            |record: &TraceRecord, children: &[Vec<String>]| {
                let mut names = vec![record.ontology.clone()];
                for c in children {
                    names.extend(c.iter().cloned());
                }
                names
            },
        );
        let names = fold_trace(&tree, &collect_alg);
        assert_eq!(names, vec!["Language", "Grammar"]);
    }
}
