use super::event::AbstractEvent;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    #[serde(rename(serialize = "__typename"))]
    pub __typename: String,
    pub entity_id: i32,
    pub entity_revision_id: i32,
}

impl CreateEntityRevision {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateEntityRevision {
            __typename: "CreateEntityRevisionNotificationEvent".to_string(),
            // uses "repository" parameter
            entity_id: abstract_event.parameter_uuid_id,
            entity_revision_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
