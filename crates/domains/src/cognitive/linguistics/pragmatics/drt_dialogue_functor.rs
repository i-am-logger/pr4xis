use pr4xis::category::Functor;

use super::reference::*;
use crate::formal::information::dialogue::ontology::*;

// DRT → Dialogue functor.
//
// Proof that discourse reference IS dialogue structure (Kamp 1981 + Grosz 1995).
// The DRS tracks what entities exist in the conversation.
// Centering tracks which entity is most salient.
// Both are structural components of dialogue.
//
// The mapping:
//   Referent           → Participant (an entity in the conversation)
//   DRS                → DialogueState (the accumulated discourse model)
//   Condition          → Topic (constraints on what we're talking about)
//   Accessibility      → History (what can be referenced from where)
//   CenteringState     → TurnManagement (who/what is salient)
//   Transition         → DialogueAct (shift in topic/salience)
//   AnaphoricExpression → Utterance (the expression that needs resolution)
//   Binding            → Grounding (the resolved reference = mutual understanding)

pub struct ReferencToDialogue;

impl Functor for ReferencToDialogue {
    type Source = ReferenceCategory;
    type Target = DialogueCategory;

    fn map_object(obj: &ReferenceConcept) -> DialogueConcept {
        match obj {
            ReferenceConcept::Referent => DialogueConcept::Participant,
            ReferenceConcept::DRS => DialogueConcept::DialogueState,
            ReferenceConcept::Condition => DialogueConcept::Topic,
            ReferenceConcept::Accessibility => DialogueConcept::History,
            ReferenceConcept::CenteringState => DialogueConcept::TurnManagement,
            ReferenceConcept::Transition => DialogueConcept::DialogueAct,
            ReferenceConcept::AnaphoricExpression => DialogueConcept::Utterance,
            ReferenceConcept::Binding => DialogueConcept::Grounding,
        }
    }

    fn map_morphism(m: &ReferenceRelation) -> DialogueRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            ReferenceRelationKind::Identity => DialogueRelationKind::Identity,
            ReferenceRelationKind::Introduces => DialogueRelationKind::Produces,
            ReferenceRelationKind::Resolves => DialogueRelationKind::Interprets,
            ReferenceRelationKind::Constrains => DialogueRelationKind::Addresses,
            ReferenceRelationKind::Contains => DialogueRelationKind::AppendedTo,
            ReferenceRelationKind::Subordinates => DialogueRelationKind::AppendedTo,
            ReferenceRelationKind::Accessible => DialogueRelationKind::AppendedTo,
            ReferenceRelationKind::Updates => DialogueRelationKind::Updates,
            ReferenceRelationKind::Ranks => DialogueRelationKind::Controls,
            ReferenceRelationKind::Links => DialogueRelationKind::Controls,
            ReferenceRelationKind::Binds => DialogueRelationKind::ArisesFrom,
            ReferenceRelationKind::Composed => DialogueRelationKind::Composed,
        };
        DialogueRelation { from, to, kind }
    }
}
pr4xis::register_functor!(ReferencToDialogue);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<ReferencToDialogue>().unwrap();
    }

    #[test]
    fn drs_is_dialogue_state() {
        assert_eq!(
            ReferencToDialogue::map_object(&ReferenceConcept::DRS),
            DialogueConcept::DialogueState
        );
    }

    #[test]
    fn binding_is_grounding() {
        assert_eq!(
            ReferencToDialogue::map_object(&ReferenceConcept::Binding),
            DialogueConcept::Grounding
        );
    }
}
