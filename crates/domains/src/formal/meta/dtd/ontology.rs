//! DTD concept inventory — every leaf cites W3C XML 1.0 Fifth Edition
//! (Bray et al. 2008) section that defines it.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Ontology declaration — five top-level declaration kinds; the entity
// declaration splits into four sub-kinds per §4.2.1 / §4.2.2.
// =============================================================================

pr4xis::ontology! {
    name: "Dtd",
    source: "Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. & Yergeau, F. (2008) Extensible Markup Language (XML) 1.0 (Fifth Edition), W3C Recommendation 26 November 2008",

    concepts: [
        // The document type declaration as a whole (§2.8). Plays the
        // same root role SchemaDocument plays for XSD (super::xsd::§2.5).
        DocumentTypeDefinition,

        // The four markup-declaration kinds (§2.8 [29] markupdecl).
        // Each gets its own concept; the parser emits one per parsed
        // declaration. They sit under DocumentTypeDefinition as the
        // declaration body's component kinds.
        MarkupDecl,
        ElementDecl,
        AttListDecl,
        EntityDecl,
        NotationDecl,

        // Entity sub-kinds (§4.2.1 + §4.2.2 + §4.7): the cartesian
        // product of (general | parameter) × (parsed | unparsed).
        // Parsed parameter entities are the most common in practice
        // (DTD reuse); unparsed general entities pair with notations
        // for non-XML attached resources (§4.5).
        GeneralEntity,
        ParameterEntity,
        ParsedEntity,
        UnparsedEntity,
    ],

    labels: {
        DocumentTypeDefinition: ("en", "Document Type Definition",
            "W3C XML 1.0 Fifth Edition §2.8: a document type declaration `<!DOCTYPE ...>` plus its internal and/or external subset of markup declarations — the schema-document container for the DTD's element / attribute / entity / notation declarations."),
        MarkupDecl: ("en", "Markup declaration",
            "W3C XML 1.0 Fifth Edition §2.8 production [29] `markupdecl`: the union of elementdecl, AttlistDecl, EntityDecl, NotationDecl, processing instructions, and comments that may appear in a DTD's subset. Abstract parent of the four concrete declaration kinds."),
        ElementDecl: ("en", "Element-type declaration",
            "W3C XML 1.0 Fifth Edition §3.2 production [45] `elementdecl`: `<!ELEMENT name content>` declaring an element type and its content model (EMPTY, ANY, mixed, or children-element regular expression)."),
        AttListDecl: ("en", "Attribute-list declaration",
            "W3C XML 1.0 Fifth Edition §3.3 production [52] `AttlistDecl`: `<!ATTLIST element attr type default ...>` declaring the attributes that may appear on an element. Attribute types per §3.3.1 (StringType / TokenizedType / EnumeratedType); defaults per §3.3.2 (#REQUIRED / #IMPLIED / #FIXED / default value)."),
        EntityDecl: ("en", "Entity declaration",
            "W3C XML 1.0 Fifth Edition §4.2 production [70] `EntityDecl`: `<!ENTITY name ...>` declaring a general or parameter entity, parsed or unparsed. Splits by (general|parameter) × (parsed|unparsed) — see GeneralEntity / ParameterEntity / ParsedEntity / UnparsedEntity sub-concepts."),
        NotationDecl: ("en", "Notation declaration",
            "W3C XML 1.0 Fifth Edition §4.7 production [82] `NotationDecl`: `<!NOTATION name PUBLIC|SYSTEM literal>` declaring a notation — a binding from a name to an external identifier, used by unparsed entities (§4.5) and the NOTATION attribute type (§3.3.1)."),
        GeneralEntity: ("en", "General entity",
            "W3C XML 1.0 Fifth Edition §4.2 production [71] `GEDecl`: an entity declared without the `%` prefix, referenced from element content via `&name;` (§4.1)."),
        ParameterEntity: ("en", "Parameter entity",
            "W3C XML 1.0 Fifth Edition §4.2 production [72] `PEDecl`: an entity declared with the `%` prefix, referenced from within the DTD via `%name;` (§4.1)."),
        ParsedEntity: ("en", "Parsed entity",
            "W3C XML 1.0 Fifth Edition §4.2.2: an entity whose replacement text is XML — parsed and merged into the document infoset on reference."),
        UnparsedEntity: ("en", "Unparsed entity",
            "W3C XML 1.0 Fifth Edition §4.2.2 + §4.5: an entity whose content has a non-XML notation (referenced from a NOTATION attribute via `NDATA` rather than parsed inline)."),
    },

    is_a: [
        // The four markup-declaration kinds are sub-kinds of MarkupDecl,
        // which is a sub-kind of DocumentTypeDefinition (the whole
        // declaration body).
        (MarkupDecl,        DocumentTypeDefinition),
        (ElementDecl,       MarkupDecl),
        (AttListDecl,       MarkupDecl),
        (EntityDecl,        MarkupDecl),
        (NotationDecl,      MarkupDecl),

        // Entity sub-kinds: each entity is exactly one
        // (general | parameter) × (parsed | unparsed) cell.
        (GeneralEntity,     EntityDecl),
        (ParameterEntity,   EntityDecl),
        (ParsedEntity,      EntityDecl),
        (UnparsedEntity,    EntityDecl),
    ],
}

// =============================================================================
// Quality: EntityKind — the (general|parameter) × (parsed|unparsed)
// classification, total on EntityDecl.
// =============================================================================

/// Quality: which (scope, parsedness) cell an [`DtdConcept::EntityDecl`]
/// instance belongs to. The four cells are exactly the §4.2.2 partition
/// of entity declarations.
#[derive(Debug, Clone)]
pub struct EntityKindQuality;

/// One of the four cells of the entity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    /// General + Parsed: the most common — `<!ENTITY name "value">`
    /// expanded into element content on `&name;` reference (§4.4.4).
    GeneralParsed,
    /// General + Unparsed: paired with a notation for non-XML
    /// attached resources, e.g. `<!ENTITY logo SYSTEM "logo.png" NDATA png>`
    /// (§4.2.2 + §4.5).
    GeneralUnparsed,
    /// Parameter + Parsed: DTD-reuse — `<!ENTITY % name "value">`
    /// expanded inside the DTD on `%name;` reference (§4.1).
    ParameterParsed,
    /// Parameter + Unparsed: not permitted by XML 1.0 §4.2.2
    /// (parameter entities must be parsed). Included for total
    /// classification; instances are validation errors.
    ParameterUnparsed,
}

impl Quality for EntityKindQuality {
    type Individual = DtdConcept;
    type Value = EntityKind;

    fn get(&self, c: &DtdConcept) -> Option<EntityKind> {
        // The quality maps each entity sub-concept to its kind cell
        // when used as an instance-marker. The four entity sub-kinds
        // are pairwise *attributes* (a concrete entity is GE + Parsed,
        // PE + Parsed, etc.) — the quality returns the cell only when
        // the caller queries an entity-sub-kind, never on the
        // abstract `EntityDecl` parent or the non-entity declarations.
        match c {
            DtdConcept::GeneralEntity => Some(EntityKind::GeneralParsed),
            DtdConcept::ParameterEntity => Some(EntityKind::ParameterParsed),
            DtdConcept::ParsedEntity => Some(EntityKind::GeneralParsed),
            DtdConcept::UnparsedEntity => Some(EntityKind::GeneralUnparsed),
            _ => None,
        }
    }
}

// =============================================================================
// Axioms.
// =============================================================================

impl Ontology for DtdOntology {
    type Cat = DtdCategory;
    type Qual = EntityKindQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MarkupDeclFourPartition));
        axioms
    }
}

/// Axiom: the four concrete markup-declaration kinds (`ElementDecl`,
/// `AttListDecl`, `EntityDecl`, `NotationDecl`) are exactly the
/// children of `MarkupDecl` per W3C XML 1.0 §2.8 production \[29\].
pub struct MarkupDeclFourPartition;

impl Axiom for MarkupDeclFourPartition {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        // Every concept that subsumes into MarkupDecl, direct OR
        // transitive (the ontology macro emits transitive-closure
        // morphisms per OBO-RO Smith 2005). The four §2.8 [29]
        // markup-declaration kinds must all appear.
        let subsumed: Vec<DtdConcept> = DtdCategory::morphisms()
            .into_iter()
            .filter(|m| {
                m.target() == DtdConcept::MarkupDecl
                    && matches!(m.kind(), DtdRelationKind::Subsumption)
            })
            .map(|m| m.source())
            .collect();
        let expected = [
            DtdConcept::ElementDecl,
            DtdConcept::AttListDecl,
            DtdConcept::EntityDecl,
            DtdConcept::NotationDecl,
        ];
        // Every expected kind reaches MarkupDecl via subsumption.
        let all_present = expected.iter().all(|c| subsumed.contains(c));
        if all_present {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MarkupDeclFourPartition",
        "MarkupDecl partitions into exactly four kinds — ElementDecl, AttListDecl, EntityDecl, NotationDecl — per W3C XML 1.0 Fifth Edition §2.8 production [29]",
        "W3C XML 1.0 Fifth Edition (Bray et al. 2008) §2.8 production [29] markupdecl"
    );
}

pr4xis::register_axiom!(
    MarkupDeclFourPartition,
    "W3C XML 1.0 Fifth Edition (Bray et al. 2008) §2.8 [29] markupdecl"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nine_concepts() {
        // DocumentTypeDefinition, MarkupDecl, ElementDecl, AttListDecl,
        // EntityDecl, NotationDecl, GeneralEntity, ParameterEntity,
        // ParsedEntity, UnparsedEntity. That's 10.
        assert_eq!(DtdConcept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_markup_decl_four_partition() {
        assert!(MarkupDeclFourPartition.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn entity_kind_quality_total_on_entity_sub_kinds() {
        assert!(EntityKindQuality.get(&DtdConcept::GeneralEntity).is_some());
        assert!(
            EntityKindQuality
                .get(&DtdConcept::ParameterEntity)
                .is_some()
        );
        assert!(EntityKindQuality.get(&DtdConcept::ParsedEntity).is_some());
        assert!(EntityKindQuality.get(&DtdConcept::UnparsedEntity).is_some());
        // Non-entity concepts have no kind classification.
        assert!(EntityKindQuality.get(&DtdConcept::ElementDecl).is_none());
        assert!(
            EntityKindQuality
                .get(&DtdConcept::DocumentTypeDefinition)
                .is_none()
        );
    }
}
