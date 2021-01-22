use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityLink {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    child_id: i32,
    parent_id: i32,
}

impl TryFrom<AbstractEvent> for EntityLink {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
        let child_id = abstract_event.object_id;
        let parent_id = abstract_event.uuid_parameters.try_get("parent")?;

        Ok(EntityLink {
            abstract_event,

            child_id,
            parent_id,
        })
    }
}
