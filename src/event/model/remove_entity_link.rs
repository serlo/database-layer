use anyhow::Result;
use serde::Serialize;

use super::event::AbstractEvent;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveEntityLink {
    #[serde(flatten)]
    pub abstract_event: AbstractEvent,

    pub child_id: i32,
    pub parent_id: i32,
}

impl RemoveEntityLink {
    pub async fn fetch(abstract_event: AbstractEvent) -> Result<Self> {
        let child_id = abstract_event.object_id;
        // uses "parent" parameter
        let parent_id = abstract_event.parameter_uuid_id;

        Ok(RemoveEntityLink {
            abstract_event,

            child_id,
            parent_id,
        })
    }
}
