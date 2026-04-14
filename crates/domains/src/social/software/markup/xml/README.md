# XML -- W3C XML Ontology

Models XML as an extension of the parent markup ontology: the universal node kinds are enriched with XML-specific constructs (CDATA, XML declaration, DocType, namespaces) and the W3C well-formedness rules are enforced as category and axioms. Rich types `XmlDocument`, `XmlElement`, `XmlName`, `XmlNamespace`, and `XmlAttribute` carry the loaded form.

Key references:
- W3C XML 1.1 (2008) — *Extensible Markup Language 1.1 (Second Edition)*
- W3C XML 1.0 Fifth Edition (2008) — *Extensible Markup Language 1.0*
- W3C XML Namespaces 1.1 (2006)

## Entities

| Category | Entities |
|---|---|
| XML node kinds (10) | Document, Element, Attribute, Text, CData, Comment, ProcessingInstruction, XmlDeclaration, DocType, Namespace |

## Category

`XmlCategory` has `XmlNodeKind` as objects and `XmlContains` as morphisms. The edge set encodes the W3C production rules: `Document → {XmlDeclaration, DocType, Element, Comment, ProcessingInstruction}`; `Element → {Element, Attribute, Namespace, Text, CData, Comment, ProcessingInstruction}`; plus the transitive closure `Document → {Attribute, Namespace, Text, CData}`.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsContentNode | () | Element, Text, CData, Comment, ProcessingInstruction are content nodes; Attribute, Namespace, Document, XmlDeclaration, DocType are not |

## Axioms (2)

| Axiom | Description | Source |
|---|---|---|
| SingleRootElement | An XML document must have exactly one root element | W3C XML 1.0 §2.1 |
| ProperNesting | XML elements must be properly nested — no overlapping tags | W3C XML 1.0 §2.4 |

Plus the auto-generated structural axioms from category laws.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../../docs/use/compose-via-functor.md) to add one. XML extends the parent `markup` ontology through its richer node-kind set; `XmlNode::to_markup` and `XmlDocument::to_markup` realise the forgetful direction by projecting XML trees onto generic `MarkupNode` trees.

## Files

- `ontology.rs` -- `XmlNodeKind`, `XmlContains`, `XmlCategory`/`XmlOntology`, `XmlDocument`/`XmlElement`/`XmlName`/`XmlNamespace`/`XmlAttribute`/`XmlNode` rich types, `XmlSymbols`, `IsContentNode` quality, `SingleRootElement`/`ProperNesting` axioms, tests
- `reader.rs` -- streaming XML reader producing `XmlDocument`
- `lmf/` -- WordNet Lexical Markup Framework (XML application)
- `owl/` -- W3C OWL 2 (XML application)
- `rdf/` -- W3C RDF 1.1 / RDFS 1.1 (XML application)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
