use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,
    pub entity_id: i32,
    pub entity_revision_id: i32,
}

impl CreateEntityRevision {
    pub fn new(abstract_event: AbstractEvent) -> Self {
        CreateEntityRevision {
            // uses "repository" parameter
            entity_id: abstract_event.parameter_uuid_id,
            entity_revision_id: abstract_event.object_id,
            abstract_event,
        }
    }
}
