/// Identification Friend or Foe (IFF) classification.
///
/// The first step in any engagement sequence. An entity MUST be
/// classified before any action can be taken.
///
/// Source: NATO STANAG 4162 (IFF systems), LOAC Art. 48 (distinction principle).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IffClassification {
    /// Not yet classified. No action permitted.
    Unknown,
    /// Positively identified as friendly.
    Friend,
    /// Positively identified as hostile.
    Hostile,
    /// Not hostile, not friendly. Protected under LOAC.
    Neutral,
    /// Cannot determine. Treated as civilian until proven otherwise.
    /// LOAC Protocol I, Art. 50(1): "In case of doubt, a person shall
    /// be considered to be a civilian."
    AssumedCivilian,
}

/// Entity type classification.
///
/// What the sensor system has detected. Classification determines
/// which rules apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// Unknown — not yet recognized. No action beyond observation.
    Unknown,
    /// Person detected. Immediately receives protected status.
    /// Further classification requires human judgment.
    Person,
    /// Ground vehicle.
    GroundVehicle,
    /// Aircraft.
    Aircraft,
    /// Watercraft.
    Watercraft,
    /// Fixed structure / building.
    Structure,
    /// Electronic signal source (emitter).
    SignalSource,
    /// Unmanned system (drone, UGV, USV).
    UnmannedSystem,
    /// Equipment / materiel (not a person, not a vehicle).
    Equipment,
}

/// Protected status under international humanitarian law.
///
/// Geneva Conventions (1949) and Additional Protocols (1977)
/// define categories of protected persons and objects.
///
/// Sources:
///   - Geneva Convention I, Art. 12 (wounded/sick in the field)
///   - Geneva Convention III, Art. 4 (prisoners of war)
///   - Geneva Convention IV, Art. 4 (civilians)
///   - Protocol I, Art. 48 (distinction), Art. 50 (civilians), Art. 52 (civilian objects)
///   - 1954 Hague Convention (cultural property)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectedStatus {
    /// Not yet determined.
    Undetermined,
    /// Protected person: civilian, medical, religious, POW, wounded, surrendered.
    /// NO engagement permitted under any circumstances without human authorization
    /// and only under narrowly defined LOAC exceptions.
    Protected,
    /// Protected object: hospital, cultural property, place of worship, civilian infrastructure.
    ProtectedObject,
    /// Legitimate military objective as defined by LOAC Protocol I, Art. 52(2):
    /// "objects which by their nature, location, purpose or use make an effective
    /// contribution to military action and whose total or partial destruction,
    /// capture or neutralization, in the circumstances ruling at the time,
    /// offers a definite military advantage."
    MilitaryObjective,
}

/// Confidence level of a classification.
///
/// Actions require minimum confidence thresholds.
/// Lower confidence = more restrictive rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Confidence {
    /// No information. Entity is Unknown.
    None,
    /// Low confidence. Treat as protected / civilian.
    Low,
    /// Moderate confidence. Observation and alert permitted.
    Moderate,
    /// High confidence from multiple corroborating sources.
    High,
    /// Positive identification by human operator.
    PositiveId,
}

/// A classified entity — the output of the identification process.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassifiedEntity {
    pub entity_type: EntityType,
    pub iff: IffClassification,
    pub protected_status: ProtectedStatus,
    pub confidence: Confidence,
}

impl ClassifiedEntity {
    /// Construct with cross-field validation. Returns `None` if the
    /// combination is ontologically contradictory.
    ///
    /// Invariants enforced:
    /// - `Confidence::None` requires `IffClassification::Unknown` (no
    ///   information cannot imply identification).
    /// - A `Protected` entity classified as `Hostile` requires at least
    ///   `Confidence::High` (edge case: civilian who took up arms).
    pub fn new(
        entity_type: EntityType,
        iff: IffClassification,
        protected_status: ProtectedStatus,
        confidence: Confidence,
    ) -> Option<Self> {
        // Confidence::None requires Unknown IFF and Undetermined status
        if confidence == Confidence::None && iff != IffClassification::Unknown {
            return None;
        }
        // Protected entities classified Hostile is an edge case — require high confidence
        if protected_status == ProtectedStatus::Protected
            && iff == IffClassification::Hostile
            && confidence < Confidence::High
        {
            return None;
        }
        Some(Self {
            entity_type,
            iff,
            protected_status,
            confidence,
        })
    }

    /// An unknown, unclassified entity. Default state.
    pub fn unknown() -> Self {
        Self {
            entity_type: EntityType::Unknown,
            iff: IffClassification::Unknown,
            protected_status: ProtectedStatus::Undetermined,
            confidence: Confidence::None,
        }
    }

    /// Is this entity currently protected under LOAC?
    /// Protocol I Art. 50(1): "In case of doubt... considered to be a civilian."
    pub fn is_protected(&self) -> bool {
        matches!(
            self.protected_status,
            ProtectedStatus::Protected | ProtectedStatus::ProtectedObject
        )
    }

    /// Is this entity a person? Persons always require human authorization.
    pub fn is_person(&self) -> bool {
        self.entity_type == EntityType::Person
    }

    /// Can this entity potentially be engaged (meets minimum LOAC thresholds)?
    /// This does NOT authorize engagement — it only says the classification
    /// is sufficient to proceed to the escalation ladder.
    pub fn meets_engagement_threshold(&self) -> bool {
        self.protected_status == ProtectedStatus::MilitaryObjective
            && self.iff == IffClassification::Hostile
            && self.confidence >= Confidence::High
            && !self.is_person() // persons always require human PositiveId
    }
}
