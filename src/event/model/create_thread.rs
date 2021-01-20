use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateThread {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub object_id: i32,
    pub thread_id: i32,
}

impl CreateThread {
    pub async fn new(abstract_event: AbstractEvent) -> Result<Self> {
        // uses "on" parameter
        let object_id = abstract_event.parameter_uuid_id;
        let thread_id = abstract_event.object_id;

        Ok(CreateThread {
            abstract_event,

            object_id,
            thread_id,
        })
    }
}
