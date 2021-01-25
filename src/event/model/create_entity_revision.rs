use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityRevision {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    entity_id: i32,
    entity_revision_id: i32,
}

impl TryFrom<AbstractEvent> for CreateEntityRevision {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
        let entity_id = abstract_event.uuid_parameters.try_get("repository")?;
        let entity_revision_id = abstract_event.object_id;

        Ok(CreateEntityRevision {
            abstract_event,

            entity_id,
            entity_revision_id,
        })
    }
}
