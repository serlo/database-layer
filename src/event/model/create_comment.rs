use std::convert::TryFrom;

use serde::Serialize;

use super::abstract_event::AbstractEvent;
use super::EventError;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComment {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    thread_id: i32,
    comment_id: i32,
}

impl TryFrom<AbstractEvent> for CreateComment {
    type Error = EventError;

    fn try_from(abstract_event: AbstractEvent) -> Result<Self, Self::Error> {
        let thread_id = abstract_event.uuid_parameters.try_get("discussion")?;
        let comment_id = abstract_event.object_id;

        Ok(CreateComment {
            abstract_event,

            thread_id,
            comment_id,
        })
    }
}
