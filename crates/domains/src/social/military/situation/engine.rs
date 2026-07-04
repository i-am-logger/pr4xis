use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::social::compliance::classification::{Confidence, EntityType};
use crate::social::military::situation::combat_identity::CombatIdentityConcept;
use crate::social::military::situation::kinematic_relation::{
    KinematicRelationConcept, RelationCriteria, RelativeKinematics, classify,
};
use crate::social::military::situation::ontology::SituationConcept;

/// A tracked entity in the situation assessment.
///
/// Every field is a typed ontological concept, not a primitive: the two identity
/// dimensions are `entity_type` (the platform kind,
/// `compliance::classification::EntityType`) and `identity` (the STANAG 1241
/// combat identity / allegiance, which projects into the IFF / engagement layer
/// — see [`combat_identity`](crate::social::military::situation::combat_identity));
/// the kinematics are dimension-general position/velocity [`Vector`]s expressed
/// in a [`ReferenceFrame`] (2-D, 3-D, or any dimension — nothing is hardwired to
/// a plane); and the track quality is an ordinal [`Confidence`], not a bare float.
#[derive(Debug, Clone)]
pub struct TrackedEntity {
    pub id: usize,
    /// Platform classification (aircraft, watercraft, …) — JDL Level-1 object type.
    pub entity_type: EntityType,
    /// STANAG 1241 combat identity (allegiance).
    pub identity: CombatIdentityConcept,
    /// The reference frame the position/velocity are expressed in.
    pub frame: ReferenceFrame,
    /// Position vector in `frame` (any dimension).
    pub position: Vector,
    /// Velocity vector in `frame` (any dimension).
    pub velocity: Vector,
    /// Track-identification confidence (ordinal).
    pub confidence: Confidence,
}

/// A relationship between two entities.
///
/// `relation_type` is the [`KinematicRelationConcept`] classified from the
/// pair's relative motion — the `KinematicRelation` ontology concept, not an
/// ad-hoc engine enum.
#[derive(Debug, Clone)]
pub struct EntityRelationship {
    pub entity_a: usize,
    pub entity_b: usize,
    pub relation_type: KinematicRelationConcept,
    /// Ordinal confidence of the relationship (weakest-link of the two tracks).
    pub confidence: Confidence,
}

/// Situation assessment state.
#[derive(Debug, Clone)]
pub struct SituationAssessment {
    pub entities: Vec<TrackedEntity>,
    pub relationships: Vec<EntityRelationship>,
    pub current_level: SituationConcept,
}

impl Default for SituationAssessment {
    fn default() -> Self {
        Self::new()
    }
}

impl SituationAssessment {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            relationships: Vec::new(),
            current_level: SituationConcept::Concept,
        }
    }

    /// Add an identified entity.
    pub fn add_entity(&mut self, entity: TrackedEntity) {
        self.entities.push(entity);
    }

    /// Assess relationships between all entity pairs.
    ///
    /// Only pairs sharing a reference frame yield a defined relationship
    /// (`classify_relationship` returns `None` across frames); differently-framed
    /// pairs are skipped until a frame transform aligns them.
    pub fn assess_relationships(&mut self) {
        self.relationships.clear();
        let n = self.entities.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if let Some(rel) = classify_relationship(&self.entities[i], &self.entities[j]) {
                    self.relationships.push(rel);
                }
            }
        }
        self.current_level = SituationConcept::Relationship;
    }

    /// Number of entities.
    pub fn num_entities(&self) -> usize {
        self.entities.len()
    }

    /// Number of assessed relationships.
    pub fn num_relationships(&self) -> usize {
        self.relationships.len()
    }
}

/// Classify the relationship between two entities from their relative motion.
///
/// The decision is delegated to the `KinematicRelation` ontology: the raw planar
/// state becomes a typed [`RelativeKinematics`], and [`classify`] matches it
/// against the ontology's cited criteria ([`RelationCriteria`]) — no thresholds
/// live in this engine. The relationship confidence is the weakest-link
/// (conjunctive) combination of the two entity confidences (Zadeh 1965 min
/// t-norm): a derived relation is no more certain than its least-certain
/// constituent.
///
/// Returns `None` when the two entities are not in a common reference frame (or
/// are dimensioned differently) — their relative motion is then undefined until
/// a frame transform aligns them.
pub fn classify_relationship(a: &TrackedEntity, b: &TrackedEntity) -> Option<EntityRelationship> {
    let kinematics = RelativeKinematics::from_states(
        a.frame,
        &a.position,
        &a.velocity,
        b.frame,
        &b.position,
        &b.velocity,
    )?;
    Some(EntityRelationship {
        entity_a: a.id,
        entity_b: b.id,
        relation_type: classify(&kinematics, &RelationCriteria::standard()),
        // Weakest-link (conjunctive) combination — a derived relation is no more
        // certain than its least-certain constituent (Zadeh 1965 min t-norm).
        confidence: a.confidence.min(b.confidence),
    })
}
