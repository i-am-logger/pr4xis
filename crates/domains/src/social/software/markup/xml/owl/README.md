# OWL -- W3C OWL 2 Web Ontology Language Metamodel

Models the OWL 2 abstract syntax as a category over twenty-three construct kinds — class expressions, property expressions, property characteristics, individuals, restriction fillers, and the ontology header. Rich types `OwlClass`, `OwlObjectProperty`, `OwlIndividual`, and `OwlOntology` carry the loaded form; `OwlVocabulary` holds the canonical W3C IRIs used for ontological lookup (not string matching).

Key references:
- W3C OWL 2 Structural Specification and Functional-Style Syntax (2012)
- W3C OWL 2 Direct Semantics (2012) — SROIQ model theory
- Baader et al. 2003: *The Description Logic Handbook*
- Horrocks, Patel-Schneider & van Harmelen 2003: *From SHIQ and RDF to OWL*

## Entities

| Category | Entities |
|---|---|
| Class expressions (6) | Class, Restriction, UnionOf, IntersectionOf, ComplementOf, OneOf |
| Property expressions (3) | ObjectProperty, DatatypeProperty, AnnotationProperty |
| Property characteristics (7) | FunctionalProperty, InverseFunctionalProperty, TransitiveProperty, SymmetricProperty, AsymmetricProperty, ReflexiveProperty, IrreflexiveProperty |
| Individuals (1) | NamedIndividual |
| Restriction fillers (6) | SomeValuesFrom, AllValuesFrom, HasValue, MinCardinality, MaxCardinality, ExactCardinality |
| Header (1) | Ontology |

## Category

`OwlCategory` has `OwlConcept` as objects and `OwlRelation` as morphisms. The edge set encodes W3C OWL 2 §5–§10: class constructors specialize to `Class`, property characteristics specialize to `ObjectProperty`, `Restriction` relates to its fillers and to the property it restricts, `NamedIndividual` points to `Class`, and `Ontology` points to its component kinds.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsClassExpression | () | Marks the six class-expression concepts (Class, Restriction, UnionOf, IntersectionOf, ComplementOf, OneOf) |

## Axioms (1)

| Axiom | Description | Source |
|---|---|---|
| RestrictionNeedsProperty | Every `owl:Restriction` must have exactly one `owl:onProperty` | W3C OWL 2 §8.2 |

Plus the auto-generated structural axioms from category laws.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../../../docs/use/compose-via-functor.md) to add one. OWL sits above RDF in the W3C Semantic Web stack; a forgetful functor to the sibling `rdf` ontology is the natural next step.

## Files

- `ontology.rs` -- `OwlConcept`, `OwlRelation`, `OwlCategory`/`OwlMetaOntology`, `OwlVocabulary` (canonical IRIs), `OwlClass`/`OwlObjectProperty`/`OwlIndividual`/`OwlOntology` rich types, `IsClassExpression` quality, `RestrictionNeedsProperty` axiom, tests
- `reader.rs` -- OWL/XML reader producing an `OwlOntology`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
