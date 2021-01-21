use anyhow::Result;
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
    pub async fn fetch(abstract_event: AbstractEvent) -> Result<Self> {
        // uses "repository" parameter
        let entity_id = abstract_event.parameter_uuid_id;
        let entity_revision_id = abstract_event.object_id;

        Ok(CreateEntityRevision {
            abstract_event,

            entity_id,
            entity_revision_id,
        })
    }
}
