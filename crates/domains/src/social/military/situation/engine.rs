#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::social::military::situation::ontology::SituationElement;

/// A tracked entity in the situation assessment.
#[derive(Debug, Clone)]
pub struct TrackedEntity {
    pub id: usize,
    pub classification: &'static str,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
    pub confidence: f64,
}

/// A relationship between two entities.
#[derive(Debug, Clone)]
pub struct EntityRelationship {
    pub entity_a: usize,
    pub entity_b: usize,
    pub relation_type: RelationType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationType {
    /// Entities are moving together (formation).
    Formation,
    /// One entity is following another.
    Following,
    /// Entities are on converging paths.
    Converging,
    /// Entities are diverging.
    Diverging,
    /// No significant relationship detected.
    None,
}

/// Situation assessment state.
#[derive(Debug, Clone)]
pub struct SituationAssessment {
    pub entities: Vec<TrackedEntity>,
    pub relationships: Vec<EntityRelationship>,
    pub current_level: SituationElement,
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
            current_level: SituationElement::Concept,
        }
    }

    /// Add an identified entity.
    pub fn add_entity(&mut self, entity: TrackedEntity) {
        self.entities.push(entity);
    }

    /// Assess relationships between all entity pairs.
    pub fn assess_relationships(&mut self) {
        self.relationships.clear();
        let n = self.entities.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let rel = classify_relationship(&self.entities[i], &self.entities[j]);
                self.relationships.push(rel);
            }
        }
        self.current_level = SituationElement::Relationship;
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

/// Classify the relationship between two entities based on kinematics.
pub fn classify_relationship(a: &TrackedEntity, b: &TrackedEntity) -> EntityRelationship {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dist = (dx * dx + dy * dy).sqrt();

    let dvx = b.velocity[0] - a.velocity[0];
    let dvy = b.velocity[1] - a.velocity[1];
    let rel_speed = (dvx * dvx + dvy * dvy).sqrt();

    // Closing rate (negative = converging)
    let closing_rate = if dist > 0.0 {
        (dx * dvx + dy * dvy) / dist
    } else {
        0.0
    };

    let relation_type = if rel_speed < 0.5 && dist < 100.0 {
        RelationType::Formation
    } else if closing_rate < -1.0 {
        RelationType::Converging
    } else if closing_rate > 1.0 {
        RelationType::Diverging
    } else {
        RelationType::None
    };

    let confidence = (1.0 - rel_speed / 10.0).clamp(0.1, 1.0);

    EntityRelationship {
        entity_a: a.id,
        entity_b: b.id,
        relation_type,
        confidence,
    }
}
