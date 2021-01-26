use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadEvent {
    object_id: i32,
    thread_id: i32,
}

impl TryFrom<&AbstractEvent> for CreateThreadEvent {
    type Error = EventError;

    fn try_from(abstract_event: &AbstractEvent) -> Result<Self, Self::Error> {
        let object_id = abstract_event.uuid_parameters.try_get("on")?;
        let thread_id = abstract_event.object_id;

        Ok(CreateThreadEvent {
            object_id,
            thread_id,
        })
    }
}
