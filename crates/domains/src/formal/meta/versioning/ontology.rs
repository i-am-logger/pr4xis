//! Version adjunction — the version-polymorphic structure shared by
//! every artifact published in several versions, as a Praxis ontology.
//!
//! Specifications, schemas, formats, guides, statutes, and ontologies
//! are revised over time, and the *same content* lands at different
//! places (sections, syntaxes, feature sets) in different versions —
//! XSD 1.0 vs 1.1, XML 1.0 vs 1.1, PDF (ISO 32000-1) vs PDF 2.0
//! (ISO 32000-2), the LRC USLM User Guide 0.1.4 vs the current
//! edition. Pinning a single version is brittle; the durable thing is
//! the **version-invariant** content, located in each version.
//!
//! ## The literature
//!
//! The version-invariant / version-specific split is the core of
//! schema- and ontology-versioning theory (Roddick 1995, *A survey of
//! schema versioning issues*; Noy & Klein 2004, *Ontology Evolution:
//! Not the Same as Schema Evolution*). The categorical treatment of
//! version-to-version mapping is the patch/merge category (Mimram &
//! Di Giusto 2013, *A Categorical Theory of Patches* — merges as
//! pushouts) and asymmetric delta lenses (Diskin, Xiong & Czarnecki
//! 2011). Semantic Versioning (Preston-Werner, SemVer 2.0.0) is the
//! de-facto version-identifier scheme.
//!
//! ## The adjunction
//!
//! On the closed category of versioning roles
//! ([`VersioningConcept`]) there are two endofunctors:
//!
//! - [`LocalizeVersion`] (left adjoint, *free*) — realizes the
//!   `VersionInvariant` as its `VersionFiber` in a distinguished
//!   version.
//! - [`AbstractVersion`] (right adjoint, *forgetful*) — generalizes a
//!   `VersionFiber` back to the `VersionInvariant` it realizes.
//!
//! `LocalizeVersion ⊣ AbstractVersion` is a **reflection** (Mac Lane
//! 1998 §IV.1; Awodey 2010 §9): the unit `η: id ⇒ AbstractVersion ∘
//! LocalizeVersion` is the identity on `VersionInvariant` — realizing
//! the invariant in a version then forgetting the version recovers the
//! invariant. This is the *constant-complement* property (Bancilhon &
//! Spyratos 1981): the invariant is the complement held constant
//! across versions, each fiber is a view of it, and the round-trip is
//! lossless on the invariant side. The fibers range over the version
//! index — the base of a Grothendieck fibration (Grothendieck 1971
//! SGA1 Exposé VI).
//!
//! `VersionFiber is_a VersionInvariant` (a fiber is a
//! version-specialised invariant), so the unit/counit components are
//! exactly the identity and that subsumption edge, both present in the
//! category.
//!
//! ## Citation
//!
//! - **Roddick, J. F.** "A survey of schema versioning issues for
//!   database systems", *Information and Software Technology* 37(7),
//!   1995, pp. 383-393.
//! - **Noy, N. F. & Klein, M.** "Ontology Evolution: Not the Same as
//!   Schema Evolution", *Knowledge and Information Systems* 6(4),
//!   2004, pp. 428-440.
//! - **Mimram, S. & Di Giusto, C.** "A Categorical Theory of Patches",
//!   MFPS 2013, *ENTCS* 298, pp. 283-307.
//! - **Diskin, Z., Xiong, Y. & Czarnecki, K.** "From State- to
//!   Delta-Based Bidirectional Model Transformations: the Asymmetric
//!   Case", *Journal of Object Technology* 10, 2011.
//! - **Mac Lane, S.** *Categories for the Working Mathematician*,
//!   2nd ed., Springer GTM 5, 1998. §IV.1 (adjunctions).
//! - **Bancilhon, F. & Spyratos, N.** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4), 1981 (constant complement).
//! - **Grothendieck, A.** SGA 1, Exposé VI (fibered categories),
//!   Springer LNM 224, 1971.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Concept inventory — the roles of a versioned artifact.
// =============================================================================

pr4xis::ontology! {
    name: "Versioning",
    source: "Roddick (1995) A survey of schema versioning issues for database systems, Information and Software Technology 37(7); Noy & Klein (2004) Ontology Evolution: Not the Same as Schema Evolution, Knowledge and Information Systems 6(4); Mimram & Di Giusto (2013) A Categorical Theory of Patches, ENTCS 298; Mac Lane (1998) Categories for the Working Mathematician §IV.1; Bancilhon & Spyratos (1981) Update Semantics of Relational Views, ACM TODS 6(4); Grothendieck (1971) SGA1 Exposé VI",

    concepts: [
        VersionedArtifact,
        Version,
        VersionInvariant,
        VersionFiber,
    ],

    labels: {
        VersionedArtifact: ("en", "Versioned artifact",
            "Any artifact published in several versions — a spec, schema, format, guide, statute, or ontology (XSD, XML, PDF/ISO 32000, USLM, ...). The aggregate of a version-invariant content and its per-version fibers (Roddick 1995)."),
        Version: ("en", "Version",
            "A specific published version / edition of the artifact (e.g. XSD 1.1, PDF 2.0 / ISO 32000-2, USLM User Guide 0.1.4) — the base index the fibers range over. Identifiers per SemVer 2.0.0 where applicable (Preston-Werner)."),
        VersionInvariant: ("en", "Version invariant",
            "The version-independent content — a feature, concept, or claim that persists across versions; the constant complement (Bancilhon & Spyratos 1981) held invariant under revision (Noy & Klein 2004)."),
        VersionFiber: ("en", "Version fiber",
            "A version invariant realized in one version: where (which section / syntax / feature form) that version places it. A version-specialised invariant, hence `is_a VersionInvariant`; the fiber of the version fibration (Grothendieck 1971 Exposé VI)."),
    },

    // `VersionFiber is_a VersionInvariant`: a fiber is the same content
    // specialised to one version (its instances are those occurring in
    // that version). `Version` and `VersionInvariant` are the two
    // facets of a `VersionedArtifact`.
    is_a: [
        (Version,          VersionedArtifact),
        (VersionInvariant, VersionedArtifact),
        (VersionFiber,     VersionInvariant),
    ],
}

// =============================================================================
// The two adjoint endofunctors' object maps.
// =============================================================================

/// `AbstractVersion` object map (right adjoint, forgetful): a
/// `VersionFiber` generalises to the `VersionInvariant` it realizes;
/// everything else is fixed.
pub fn abstract_version_concept(c: VersioningConcept) -> VersioningConcept {
    match c {
        VersioningConcept::VersionFiber => VersioningConcept::VersionInvariant,
        other => other,
    }
}

/// `LocalizeVersion` object map (left adjoint, free): a
/// `VersionInvariant` realizes as its `VersionFiber` (in the
/// distinguished version); everything else is fixed.
pub fn localize_version_concept(c: VersioningConcept) -> VersioningConcept {
    match c {
        VersioningConcept::VersionInvariant => VersioningConcept::VersionFiber,
        other => other,
    }
}

/// Build a morphism `from → to`, collapsing to the identity when the
/// endpoints coincide (a functor sends a morphism between objects it
/// identifies to an identity; Mac Lane §I.3).
fn morphism(
    from: VersioningConcept,
    to: VersioningConcept,
    kind: VersioningRelationKind,
) -> VersioningRelation {
    let kind = if from == to {
        VersioningRelationKind::Identity
    } else {
        kind
    };
    VersioningRelation { from, to, kind }
}

pr4xis::functor! {
    name: LocalizeVersion,
    source: VersioningCategory,
    target: VersioningCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (functors), §IV.1 (adjunctions); Bancilhon & Spyratos (1981) Update Semantics of Relational Views, ACM TODS 6(4); Diskin, Xiong & Czarnecki (2011) delta lenses (JOT 10)",
    map_object: |c: &VersioningConcept| -> VersioningConcept { localize_version_concept(*c) },
    map_morphism: |m: &VersioningRelation| -> VersioningRelation {
        morphism(
            localize_version_concept(m.from),
            localize_version_concept(m.to),
            m.kind,
        )
    },
}

pr4xis::functor! {
    name: AbstractVersion,
    source: VersioningCategory,
    target: VersioningCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (functors), §IV.1 (adjunctions); Grothendieck (1971) SGA1 Exposé VI (fibered categories); Noy & Klein (2004) ontology evolution (KAIS 6(4))",
    map_object: |c: &VersioningConcept| -> VersioningConcept { abstract_version_concept(*c) },
    map_morphism: |m: &VersioningRelation| -> VersioningRelation {
        morphism(
            abstract_version_concept(m.from),
            abstract_version_concept(m.to),
            m.kind,
        )
    },
}

pr4xis::adjunction! {
    name: VersionAdjunction,
    left:  LocalizeVersion,
    right: AbstractVersion,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §IV.1 (adjunctions); Awodey (2010) Category Theory §9 (reflective subcategories); Bancilhon & Spyratos (1981) ACM TODS 6(4) (constant complement); Mimram & Di Giusto (2013) A Categorical Theory of Patches, ENTCS 298; Grothendieck (1971) SGA1 Exposé VI",
    unit: |obj: &VersioningConcept| -> VersioningRelation {
        // η_X : X → AbstractVersion(LocalizeVersion(X)). On
        // `VersionInvariant` the composite is the identity (localize
        // then abstract recovers the invariant — the reflection /
        // constant-complement law). On `VersionFiber` it is the
        // `is_a VersionInvariant` subsumption.
        let target = abstract_version_concept(localize_version_concept(*obj));
        morphism(*obj, target, VersioningRelationKind::Subsumption)
    },
    counit: |obj: &VersioningConcept| -> VersioningRelation {
        // ε_Y : LocalizeVersion(AbstractVersion(Y)) → Y. On
        // `VersionFiber` the composite is the identity; on
        // `VersionInvariant` it is the `VersionFiber is_a VersionInvariant`
        // subsumption.
        let source = localize_version_concept(abstract_version_concept(*obj));
        morphism(source, *obj, VersioningRelationKind::Subsumption)
    },
}

// =============================================================================
// Quality: VersionDependence — the axis the adjunction moves along.
// =============================================================================

/// Whether a concept's meaning depends on a specific version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionDependence {
    /// Invariant across versions (the constant complement):
    /// `VersionInvariant`.
    Independent,
    /// Tied to one version: `VersionFiber`, `Version`.
    Dependent,
}

/// Quality assigning each role its version-dependence. `None` for the
/// aggregate `VersionedArtifact` (it spans both).
#[derive(Debug, Clone)]
pub struct VersionDependenceOf;

impl Quality for VersionDependenceOf {
    type Individual = VersioningConcept;
    type Value = VersionDependence;

    fn get(&self, c: &VersioningConcept) -> Option<VersionDependence> {
        use VersioningConcept as C;
        match c {
            C::VersionInvariant => Some(VersionDependence::Independent),
            C::VersionFiber | C::Version => Some(VersionDependence::Dependent),
            C::VersionedArtifact => None,
        }
    }
}

// =============================================================================
// Runtime instance level — the adjunction over actual versioned data,
// generic over the per-version realization type `T` (a section string,
// a feature set, a parsed schema, ...). XSD, XML, PDF, USLM, and
// citations all instantiate this.
// =============================================================================

/// One published version's fiber of a versioned artifact: the version
/// identifier and the artifact's realization in that version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionFiber<T> {
    pub version: String,
    pub realization: T,
}

/// A versioned-artifact instance: a version-invariant identity (the
/// constant complement) and one fiber per published version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedArtifact<T> {
    pub invariant: String,
    pub fibers: Vec<VersionFiber<T>>,
}

impl<T> VersionedArtifact<T> {
    /// `AbstractVersion` at the instance level: drop the version and
    /// recover the version-invariant identity. Constant over every
    /// fiber (the constant complement, Bancilhon & Spyratos 1981).
    pub fn abstract_version(&self) -> &str {
        &self.invariant
    }

    /// `LocalizeVersion` at the instance level: find the fiber for a
    /// given version, or `None` if the invariant is not realized in
    /// that version.
    pub fn localize(&self, version: &str) -> Option<&VersionFiber<T>> {
        self.fibers.iter().find(|f| f.version == version)
    }

    /// The versions this artifact has been published in.
    pub fn versions(&self) -> impl Iterator<Item = &str> {
        self.fibers.iter().map(|f| f.version.as_str())
    }
}

// =============================================================================
// Axioms.
// =============================================================================

impl Ontology for VersioningOntology {
    type Cat = VersioningCategory;
    type Qual = VersionDependenceOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AdjunctionUnitReflectsInvariant));
        axioms.push(Box::new(VersionFiberSpecialisesInvariant));
        axioms.push(Box::new(InvariantIsConstantComplement));
        axioms.push(Box::new(LocalizeRecoversEachFiber));
        axioms.push(Box::new(AdjunctionAppliesAcrossDomains));
        axioms
    }
}

/// Axiom: the adjunction unit is the identity on `VersionInvariant` —
/// realizing the invariant in a version then forgetting the version
/// recovers it (`AbstractVersion ∘ LocalizeVersion = id` on
/// `VersionInvariant`). This is the reflection / constant-complement
/// law (Mac Lane §IV.1; Bancilhon & Spyratos 1981).
pub struct AdjunctionUnitReflectsInvariant;

impl Axiom for AdjunctionUnitReflectsInvariant {
    fn verify(&self) -> Verdict {
        use VersioningConcept as C;
        let round_trip = |c| abstract_version_concept(localize_version_concept(c));
        let ok = round_trip(C::VersionInvariant) == C::VersionInvariant
            && abstract_version_concept(C::VersionFiber) == C::VersionInvariant
            && localize_version_concept(C::VersionInvariant) == C::VersionFiber
            && round_trip(C::Version) == C::Version
            && round_trip(C::VersionedArtifact) == C::VersionedArtifact;
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AdjunctionUnitReflectsInvariant",
        "AbstractVersion ∘ LocalizeVersion is the identity on VersionInvariant — realizing the invariant in a version then forgetting the version recovers it (the reflection / constant-complement law)",
        "Mac Lane (1998) §IV.1; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
    );
}

pr4xis::register_axiom!(
    AdjunctionUnitReflectsInvariant,
    "Mac Lane (1998) §IV.1; Bancilhon & Spyratos (1981) ACM TODS 6(4)"
);

/// Axiom: a `VersionFiber` is a specialised `VersionInvariant`
/// (`is_a`), so the forgetful functor's collapse VersionFiber →
/// VersionInvariant and the unit / counit components are valid
/// category morphisms (the subsumption edge exists).
pub struct VersionFiberSpecialisesInvariant;

impl Axiom for VersionFiberSpecialisesInvariant {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let exists = VersioningCategory::morphisms().iter().any(|m| {
            m.source() == VersioningConcept::VersionFiber
                && m.target() == VersioningConcept::VersionInvariant
                && matches!(m.kind(), VersioningRelationKind::Subsumption)
        });
        if exists {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "VersionFiberSpecialisesInvariant",
        "VersionFiber is_a VersionInvariant — a fiber is a version-specialised invariant, so the adjunction's collapse and unit/counit components are valid category morphisms",
        "Mac Lane (1998) §IV.1; Grothendieck (1971) SGA1 Exposé VI (fibered categories)"
    );
}

pr4xis::register_axiom!(
    VersionFiberSpecialisesInvariant,
    "Mac Lane (1998) §IV.1; Grothendieck (1971) SGA1 Exposé VI"
);

/// Axiom: at the instance level the invariant is the constant
/// complement — `abstract_version` returns the same value regardless
/// of which fiber is in view (Bancilhon & Spyratos 1981).
pub struct InvariantIsConstantComplement;

impl Axiom for InvariantIsConstantComplement {
    fn verify(&self) -> Verdict {
        for art in sample_artifacts() {
            if art.fibers.is_empty() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            let constant = art.fibers.iter().all(|f| {
                let _ = art.localize(&f.version);
                art.abstract_version() == art.invariant
            });
            if !constant {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "InvariantIsConstantComplement",
        "abstract_version returns the same invariant regardless of which version is localized — the invariant is the constant complement shared by every fiber",
        "Bancilhon & Spyratos (1981) Update Semantics of Relational Views, ACM TODS 6(4)"
    );
}

pr4xis::register_axiom!(
    InvariantIsConstantComplement,
    "Bancilhon & Spyratos (1981) ACM TODS 6(4)"
);

/// Axiom: `localize` recovers each registered fiber — for every fiber
/// `f`, `localize(f.version)` is `Some(f)` (the counit is the identity
/// on localized fibers), and an unregistered version is not localized.
pub struct LocalizeRecoversEachFiber;

impl Axiom for LocalizeRecoversEachFiber {
    fn verify(&self) -> Verdict {
        for art in sample_artifacts() {
            let all = art
                .fibers
                .iter()
                .all(|f| art.localize(&f.version) == Some(f));
            if !all || art.localize("no-such-version").is_some() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LocalizeRecoversEachFiber",
        "localize recovers each registered fiber (the counit is the identity on localized fibers) and returns None for an unregistered version",
        "Mac Lane (1998) §IV.1 (counit)"
    );
}

pr4xis::register_axiom!(LocalizeRecoversEachFiber, "Mac Lane (1998) §IV.1");

/// Axiom: the version adjunction applies across artifact domains — the
/// same constant-complement structure holds for an XSD spec, an XML
/// spec, a PDF/ISO 32000 format, a USLM guide, and a literature
/// citation. Each [`sample_artifacts`] member has ≥2 distinct version
/// fibers sharing one invariant.
pub struct AdjunctionAppliesAcrossDomains;

impl Axiom for AdjunctionAppliesAcrossDomains {
    fn verify(&self) -> Verdict {
        let arts = sample_artifacts();
        // At least the five named domains are present.
        if arts.len() < 5 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        let ok = arts.iter().all(|a| {
            a.fibers.len() >= 2 && {
                // All fiber versions are distinct.
                let mut vs: Vec<&str> = a.versions().collect();
                vs.sort_unstable();
                vs.dedup();
                vs.len() == a.fibers.len()
            }
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AdjunctionAppliesAcrossDomains",
        "the version adjunction is domain-general: the same invariant + fibers structure holds for XSD, XML, PDF/ISO 32000, USLM, and citations, each with ≥2 distinct versions sharing one invariant",
        "Roddick (1995) Inf. & Softw. Tech. 37(7); Noy & Klein (2004) KAIS 6(4)"
    );
}

pr4xis::register_axiom!(
    AdjunctionAppliesAcrossDomains,
    "Roddick (1995) Inf. & Softw. Tech. 37(7); Noy & Klein (2004) KAIS 6(4)"
);

/// Sample versioned artifacts spanning five domains, demonstrating the
/// version adjunction's generality. The realization type is a
/// descriptive string (`&'static str`); other instantiations carry
/// richer per-version content (a parsed schema, a feature set, ...).
pub fn sample_artifacts() -> Vec<VersionedArtifact<&'static str>> {
    vec![
        VersionedArtifact {
            invariant: "XML Schema Definition Language (XSD)".to_string(),
            fibers: vec![
                VersionFiber {
                    version: "1.0".to_string(),
                    realization: "W3C Recommendation 2004-10-28",
                },
                VersionFiber {
                    version: "1.1".to_string(),
                    realization: "W3C Recommendation 2012-04-05",
                },
            ],
        },
        VersionedArtifact {
            invariant: "Extensible Markup Language (XML)".to_string(),
            fibers: vec![
                VersionFiber {
                    version: "1.0".to_string(),
                    realization: "W3C Recommendation, Fifth Edition (2008)",
                },
                VersionFiber {
                    version: "1.1".to_string(),
                    realization: "W3C Recommendation, Second Edition (2006)",
                },
            ],
        },
        VersionedArtifact {
            invariant: "Portable Document Format (PDF)".to_string(),
            fibers: vec![
                VersionFiber {
                    version: "ISO 32000-1 (PDF 1.7)".to_string(),
                    realization: "ISO 32000-1:2008",
                },
                VersionFiber {
                    version: "ISO 32000-2 (PDF 2.0)".to_string(),
                    realization: "ISO 32000-2:2020",
                },
            ],
        },
        VersionedArtifact {
            invariant: "USLM @identifier convention (/us/usc/t<N> path)".to_string(),
            fibers: vec![
                VersionFiber {
                    version: "0.1.4".to_string(),
                    realization: "§12.5 Identifiers; §13 Referencing Model",
                },
                VersionFiber {
                    version: "current".to_string(),
                    realization: "§11.5 Identifiers; §13 Referencing Model",
                },
            ],
        },
        VersionedArtifact {
            invariant: "Mac Lane, adjunctions".to_string(),
            fibers: vec![
                VersionFiber {
                    version: "1st ed. (1971)".to_string(),
                    realization: "§IV.1",
                },
                VersionFiber {
                    version: "2nd ed. (1998)".to_string(),
                    realization: "§IV.1",
                },
            ],
        },
    ]
}
