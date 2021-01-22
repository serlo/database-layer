use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    #[serde(flatten)]
    abstract_event: AbstractEvent,

    object_id: i32,
    thread_id: i32,
}

impl TryFrom<AbstractEvent> for CreateThread {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
        let object_id = abstract_event.uuid_parameters.try_get("on")?;
        let thread_id = abstract_event.object_id;

        Ok(CreateThread {
            abstract_event,

            object_id,
            thread_id,
        })
    }
}
